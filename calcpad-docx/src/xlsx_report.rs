//! The Excel export of a calculated worksheet (calcpad-rs's own feature): a "Report" sheet that
//! mirrors the rendered page, then the sheets of the variables.
//!
//! * headings and text paragraphs are cells, in document order (headings bold and larger);
//! * an equation is a row: name, expression, expression with the values put in, value, unit;
//! * a table of plain text is a cell range; a table that lays out other content is read
//!   cell by cell, in order;
//! * plots and drawings (inline SVG, images) are pictures at their place in the text, SVG
//!   rendered to PNG at twice its size with the same code as the Word export;
//! * a vector or matrix value links to the sheet that holds it.
//!
//! A variable that carries no unit but is followed in the text by one ("= 550" then "mm") gets
//! that unit in the Unit column, when the text is a valid unit (checked with the unit parser).
use std::collections::HashMap;

use calcpad_core::openxml::variables_export::Sheet as DataSheet;

use crate::html::{self, Element, Node};
use crate::image::{anchor_plot_text, bitmap_size, check_type, extension_of, rasterize_svg, svg_size, ImageResolver};
use crate::xlsx::{self, Sheet, Style, Value, Workbook};
use calcpad_core::parsers::expression_parser::base64_decode;

/// Characters per line of the merged text cells (columns A to F).
const TEXT_WIDTH: usize = 128;
const COLS: u32 = 6;
const MAX_PICTURE_WIDTH: f64 = 760.0;

const BLOCKS: [&str; 30] = [
    "p",
    "div",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "table",
    "ul",
    "ol",
    "li",
    "pre",
    "blockquote",
    "center",
    "section",
    "article",
    "header",
    "footer",
    "svg",
    "img",
    "hr",
    "tr",
    "td",
    "th",
    "body",
    "html",
    "form",
    "thead",
    "tbody",
];

fn is_hidden(e: &Element) -> bool {
    e.attr("style")
        .map(|s| s.to_ascii_lowercase().replace(' ', "").contains("display:none"))
        .unwrap_or(false)
}

fn superscript(s: &str) -> Option<String> {
    s.chars()
        .map(|c| match c {
            '0' => Some('\u{2070}'),
            '1' => Some('\u{00B9}'),
            '2' => Some('\u{00B2}'),
            '3' => Some('\u{00B3}'),
            '4'..='9' => char::from_u32(0x2070 + c as u32 - '0' as u32),
            '+' => Some('\u{207A}'),
            '-' | '\u{2212}' => Some('\u{207B}'),
            _ => None,
        })
        .collect()
}

fn subscript(s: &str) -> Option<String> {
    s.chars()
        .map(|c| match c {
            '0'..='9' => char::from_u32(0x2080 + c as u32 - '0' as u32),
            _ => None,
        })
        .collect()
}

fn unsuperscript(c: char) -> Option<char> {
    match c {
        '\u{2070}' => Some('0'),
        '\u{00B9}' => Some('1'),
        '\u{00B2}' => Some('2'),
        '\u{00B3}' => Some('3'),
        '\u{2074}'..='\u{2079}' => char::from_u32('4' as u32 + c as u32 - 0x2074),
        '\u{207A}' => Some('+'),
        '\u{207B}' => Some('-'),
        _ => None,
    }
}

/// Text as a cell shows it: entities decoded, the thin and hair spaces of the equations dropped.
fn tidy(s: &str) -> String {
    let d = html::decode(s);
    let mut out = String::with_capacity(d.len());
    for c in d.chars() {
        match c {
            '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{200B}' | '\u{20D7}' => {}
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\r' | '\n' | '\t' => out.push(' '),
            '\u{2215}' => out.push('/'),
            c => out.push(c),
        }
    }
    out
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A number from a cell's text: plain, with a true minus, or `m·10ⁿ` as the equations write it.
fn parse_number(text: &str) -> Option<f64> {
    let t: String = text
        .trim()
        .replace('\u{2212}', "-")
        .chars()
        .filter(|c| *c != ' ')
        .collect();
    if t.is_empty() {
        return None;
    }
    let plain = |s: &str| -> Option<f64> {
        let first = s.chars().next()?;
        if !(first.is_ascii_digit() || first == '-' || first == '+' || first == '.') {
            return None;
        }
        if s.chars().any(|c| c.is_ascii_alphabetic() && c != 'e' && c != 'E') {
            return None;
        }
        s.parse::<f64>().ok().filter(|v| v.is_finite())
    };
    if let Some(v) = plain(&t) {
        return Some(v);
    }
    // 1.5·10⁻³ or 1.5·10^-3
    let (m, e) = t.split_once("\u{00B7}10").or_else(|| t.split_once("\u{00D7}10"))?;
    let exp: String = if let Some(x) = e.strip_prefix('^') {
        x.to_string()
    } else {
        e.chars().map(|c| unsuperscript(c).unwrap_or('?')).collect()
    };
    let mantissa = if m.is_empty() { 1.0 } else { plain(m)? };
    let exp: i32 = exp.parse().ok()?;
    Some(mantissa * 10f64.powi(exp))
}

struct Seg {
    full: String,
    value: String,
    unit: String,
    seen_unit: bool,
}

impl Seg {
    fn new() -> Seg {
        Seg {
            full: String::new(),
            value: String::new(),
            unit: String::new(),
            seen_unit: false,
        }
    }
    fn push(&mut self, s: &str) {
        self.full.push_str(s);
        if self.seen_unit {
            self.unit.push_str(s);
        } else {
            self.value.push_str(s);
        }
    }
}

/// The text of an inline node: sub and sup as `_x` / superscript characters, a fraction as
/// `(a)/(b)`, a matrix as `[a; b | c; d]`.
fn lin(node: &Node, out: &mut String) {
    match node {
        Node::Text(t) => out.push_str(&tidy(t)),
        Node::Comment(_) => {}
        Node::Element(e) => lin_element(e, out),
    }
}

fn lin_children(e: &Element, out: &mut String) {
    for c in &e.children {
        lin(c, out);
    }
}

fn lin_element(e: &Element, out: &mut String) {
    if is_hidden(e) || matches!(e.name.as_str(), "script" | "style") {
        return;
    }
    match e.name.as_str() {
        "sub" => {
            let mut inner = String::new();
            lin_children(e, &mut inner);
            let inner = inner.trim().to_string();
            match subscript(&inner) {
                Some(s) if e.has_class("raised") => out.push_str(&s),
                _ => {
                    out.push('_');
                    out.push_str(&inner);
                }
            }
        }
        "sup" => {
            let mut inner = String::new();
            lin_children(e, &mut inner);
            let inner = inner.trim().to_string();
            match superscript(&inner) {
                Some(s) => out.push_str(&s),
                None => {
                    out.push('^');
                    out.push_str(&inner);
                }
            }
        }
        "br" => out.push(' '),
        "select" => out.push_str(&selected_option(e)),
        "option" => {}
        _ => {
            if e.has_class("dvc") {
                let split = e
                    .children
                    .iter()
                    .position(|c| matches!(c, Node::Element(x) if x.has_class("dvl")))
                    .unwrap_or(e.children.len());
                let (mut a, mut b) = (String::new(), String::new());
                for c in &e.children[..split] {
                    lin(c, &mut a);
                }
                for c in e.children.iter().skip(split + 1) {
                    lin(c, &mut b);
                }
                out.push_str(&format!("({})/({})", a.trim(), b.trim()));
            } else if e.has_class("matrix") {
                let rows: Vec<Vec<String>> = e
                    .children
                    .iter()
                    .filter_map(|c| match c {
                        Node::Element(r) if r.has_class("tr") => Some(
                            r.children
                                .iter()
                                .filter_map(|d| match d {
                                    Node::Element(td) if td.has_class("td") => {
                                        let mut s = String::new();
                                        lin_children(td, &mut s);
                                        Some(collapse(&s))
                                    }
                                    _ => None,
                                })
                                .filter(|s| !s.is_empty())
                                .collect(),
                        ),
                        _ => None,
                    })
                    .filter(|r: &Vec<String>| !r.is_empty())
                    .collect();
                out.push_str(&rows.iter().map(|r| r.join("; ")).collect::<Vec<_>>().join(" | "));
            } else if e.name == "table" && e.has_class("matrix") {
                let mut rows = Vec::new();
                collect_rows(e, &mut rows);
                out.push_str(
                    &rows
                        .iter()
                        .map(|r| {
                            r.iter()
                                .map(|c| {
                                    let mut s = String::new();
                                    lin_children(c, &mut s);
                                    collapse(&s)
                                })
                                .collect::<Vec<_>>()
                                .join("; ")
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                );
            } else {
                lin_children(e, out);
            }
        }
    }
}

fn selected_option(select: &Element) -> String {
    let options: Vec<&Element> = select
        .children
        .iter()
        .filter_map(|c| match c {
            Node::Element(o) if o.name == "option" => Some(o),
            _ => None,
        })
        .collect();
    let chosen = options
        .iter()
        .find(|o| o.attr("selected").is_some())
        .or_else(|| options.first());
    chosen
        .map(|o| {
            let mut s = String::new();
            lin_children(o, &mut s);
            collapse(&s)
        })
        .unwrap_or_default()
}

fn collect_rows<'a>(e: &'a Element, rows: &mut Vec<Vec<&'a Element>>) {
    for c in &e.children {
        if let Node::Element(x) = c {
            match x.name.as_str() {
                "tr" => rows.push(
                    x.children
                        .iter()
                        .filter_map(|d| match d {
                            Node::Element(cell) if cell.name == "td" || cell.name == "th" => Some(cell),
                            _ => None,
                        })
                        .collect(),
                ),
                "thead" | "tbody" | "tfoot" => collect_rows(x, rows),
                _ => {}
            }
        }
    }
}

fn contains_matching(e: &Element, f: &dyn Fn(&Element) -> bool) -> bool {
    e.children.iter().any(|c| match c {
        Node::Element(x) => f(x) || contains_matching(x, f),
        _ => false,
    })
}

fn is_equation(e: &Element) -> bool {
    e.name == "span" && e.has_class("eq")
}

/// True when a cell holds only text and inline markup.
fn plain_cell(e: &Element) -> bool {
    !contains_matching(e, &|x| {
        BLOCKS.contains(&x.name.as_str()) && x.name != "td" && x.name != "th"
            || is_equation(x)
            || x.name == "select"
            || x.name == "input"
    })
}

fn is_plain_table(t: &Element) -> bool {
    if t.has_class("matrix") {
        return false;
    }
    let mut rows = Vec::new();
    collect_rows(t, &mut rows);
    !rows.is_empty() && rows.iter().all(|r| r.iter().all(|c| plain_cell(c)))
}

struct Inferred {
    /// name -> unit, or None when the text gave different ones.
    units: HashMap<String, Option<String>>,
}

struct Builder<'a> {
    wb: Workbook,
    report: Sheet,
    row: u32,
    pending: String,
    heading: Option<u32>,
    first_segment: bool,
    /// The equation row just written and the name in it, while only its unit may still follow.
    after_eq: Option<(u32, String)>,
    inferred: Inferred,
    sheet_of: HashMap<String, String>,
    resolver: Option<&'a ImageResolver<'a>>,
    s_text: usize,
    s_name: usize,
    s_expr: usize,
    s_value: usize,
    s_unit: usize,
    s_link: usize,
}

impl<'a> Builder<'a> {
    fn new(sheet_of: HashMap<String, String>, resolver: Option<&'a ImageResolver<'a>>) -> Builder<'a> {
        let mut wb = Workbook::new();
        let s_text = wb.style(Style {
            wrap: true,
            top: true,
            ..Default::default()
        });
        let s_name = wb.style(Style {
            bold: true,
            top: true,
            ..Default::default()
        });
        let s_expr = wb.style(Style {
            wrap: true,
            top: true,
            ..Default::default()
        });
        let s_value = wb.style(Style {
            top: true,
            align: "right",
            ..Default::default()
        });
        let s_unit = wb.style(Style {
            top: true,
            italic: true,
            color: "375623",
            ..Default::default()
        });
        let s_link = wb.style(Style {
            top: true,
            underline: true,
            color: "0563C1",
            ..Default::default()
        });
        let mut report = Sheet::new("Report");
        report.widths = vec![
            (1, 1, 16.0),
            (2, 2, 36.0),
            (3, 3, 36.0),
            (4, 4, 16.0),
            (5, 5, 12.0),
            (6, 6, 12.0),
            (7, 14, 14.0),
        ];
        Builder {
            wb,
            report,
            row: 1,
            pending: String::new(),
            heading: None,
            first_segment: true,
            after_eq: None,
            inferred: Inferred { units: HashMap::new() },
            sheet_of,
            resolver,
            s_text,
            s_name,
            s_expr,
            s_value,
            s_unit,
            s_link,
        }
    }

    fn header(&mut self) {
        let st = self.wb.style(Style {
            bold: true,
            color: "FFFFFF",
            fill: "44546A",
            ..Default::default()
        });
        for (i, t) in ["Description", "Name", "Expression", "With the values", "Value", "Unit"]
            .iter()
            .enumerate()
        {
            self.report.set(1, i as u32 + 1, Value::Text(t.to_string()), st);
        }
        self.report.freeze_rows = 1;
        self.row = 2;
    }

    fn lines(len: usize, width: usize) -> f64 {
        (len.div_ceil(width.max(1))).max(1) as f64
    }

    /// The pending text as a paragraph row.
    fn flush(&mut self) {
        let text = collapse(&std::mem::take(&mut self.pending));
        let only_punctuation = text.chars().all(|c| matches!(c, ',' | ';' | ':' | '.' | ' ' | '-'));
        if only_punctuation {
            return;
        }
        self.after_eq = None;
        let (style, height) = match (self.heading, self.first_segment) {
            (Some(level), true) => {
                let size = match level {
                    1 => 20,
                    2 => 16,
                    3 => 14,
                    4 => 12,
                    _ => 11,
                };
                (
                    self.wb.style(Style {
                        bold: true,
                        size,
                        color: "1F3864",
                        wrap: true,
                        top: true,
                        ..Default::default()
                    }),
                    Self::lines(text.chars().count(), TEXT_WIDTH * 11 / size as usize) * (size as f64 * 1.4),
                )
            }
            _ => (self.s_text, Self::lines(text.chars().count(), TEXT_WIDTH) * 15.0),
        };
        if self.heading.is_some() {
            self.first_segment = false;
        }
        let r = self.row;
        self.report.set(r, 1, Value::Text(text), style);
        self.report.merge(r, 1, COLS);
        if height > 15.0 {
            self.report.heights.insert(r, height);
        }
        self.row += 1;
    }

    fn push_text(&mut self, t: &str) {
        let tidy_text = tidy(t);
        if let Some((row, name)) = self.after_eq.clone() {
            // A unit right after an equation's value.
            if let Some((unit, consumed)) = leading_unit(&tidy_text, "") {
                if self.try_unit(row, &name, &unit) {
                    self.pending.push_str(&tidy_text[consumed..]);
                    // A `<sup>` may still follow; keep the equation open for it.
                    return;
                }
            }
            self.after_eq = None;
        }
        self.pending.push_str(&tidy_text);
    }

    /// Writes `unit` into the unit cell of the equation row when it is empty and valid.
    fn try_unit(&mut self, row: u32, name: &str, unit: &str) -> bool {
        let empty = matches!(
            self.report.cells.get(&row).and_then(|c| c.get(&6)).map(|c| &c.value),
            None | Some(Value::Empty)
        );
        let numeric = matches!(
            self.report.cells.get(&row).and_then(|c| c.get(&5)).map(|c| &c.value),
            Some(Value::Number(_))
        );
        if !empty || !numeric || !valid_unit(unit) {
            return false;
        }
        self.report.set(row, 6, Value::Text(unit.to_string()), self.s_unit);
        self.inferred
            .units
            .entry(name.to_string())
            .and_modify(|u| {
                if u.as_deref() != Some(unit) {
                    *u = None;
                }
            })
            .or_insert(Some(unit.to_string()));
        true
    }

    fn equation(&mut self, e: &Element) {
        // A short label before the equation ("Yield strength -") is its description.
        let pending = collapse(&self.pending);
        let mut description = String::new();
        if self.heading.is_none() && !pending.is_empty() && pending.chars().count() <= 70 {
            let d = pending.trim_end_matches(['-', ':', ',', ' ', '\u{2013}']);
            description = d.to_string();
            self.pending.clear();
        } else {
            self.flush();
        }
        // Segments between the top level "=" signs; the last one is the result.
        let mut segs = vec![Seg::new()];
        for c in &e.children {
            match c {
                Node::Text(t) => {
                    let t = tidy(t);
                    let mut first = true;
                    for part in t.split('=') {
                        if !first {
                            segs.push(Seg::new());
                        }
                        first = false;
                        segs.last_mut().unwrap().push(part);
                    }
                }
                Node::Element(x) => {
                    let mut s = String::new();
                    lin_element(x, &mut s);
                    let seg = segs.last_mut().unwrap();
                    if x.name == "i" && !seg.seen_unit && !x.has_class("unit") {
                        seg.seen_unit = true;
                    }
                    seg.push(&s);
                }
                Node::Comment(_) => {}
            }
        }
        let clean = |s: &str| collapse(s);
        let n = segs.len();
        let name = clean(&segs[0].full);
        let mut expr = String::new();
        let mut with_values = String::new();
        if n >= 3 {
            expr = clean(&segs[1].full);
        }
        if n >= 4 {
            with_values = clean(&segs[n - 2].full);
            if n > 4 {
                with_values = (2..n - 1).map(|i| clean(&segs[i].full)).collect::<Vec<_>>().join(" = ");
            }
        }
        // A vector or matrix: written as a matrix block, or a variable that has a sheet of its own.
        let has_matrix =
            contains_matching(e, &|x| x.has_class("matrix")) || (n >= 2 && self.sheet_of.contains_key(&name));
        let result_full = clean(&segs[n - 1].full);
        if has_matrix && n == 3 && with_values.is_empty() {
            with_values = result_full.clone();
        }
        let (value_text, unit_text) = if n >= 2 {
            let last = &segs[n - 1];
            (clean(&last.value), clean(&last.unit))
        } else {
            (String::new(), String::new())
        };
        let r = self.row;
        let (c_name, c_expr, c_with) = if n >= 2 {
            (name.clone(), expr, with_values)
        } else {
            (String::new(), clean(&segs[0].full), String::new())
        };
        if !description.is_empty() {
            self.report.set(r, 1, Value::Text(description.clone()), self.s_text);
        }
        self.report.set(r, 2, Value::Text(c_name.clone()), self.s_name);
        if !c_expr.is_empty() {
            self.report.set(r, 3, Value::Text(c_expr.clone()), self.s_expr);
        }
        if !c_with.is_empty() {
            self.report.set(r, 4, Value::Text(c_with.clone()), self.s_expr);
        }
        let mut link = None;
        if has_matrix && n >= 2 {
            if n == 2 && !result_full.is_empty() {
                // The matrix as written, with the values.
                self.report.set(r, 3, Value::Text(result_full.clone()), self.s_expr);
            }
            if let Some(sheet) = self.sheet_of.get(&name).cloned() {
                self.report
                    .set(r, 5, Value::Text(format!("see sheet \"{}\"", sheet)), self.s_link);
                link = Some(sheet);
            } else {
                self.report.set(r, 5, Value::Text(value_text.clone()), self.s_value);
            }
        } else if n >= 2 && name.contains('(') && parse_number(&value_text).is_none() && unit_text.is_empty() {
            // A function definition, f(x) = expression: the right hand side is its expression.
            self.report.set(r, 3, Value::Text(value_text.clone()), self.s_expr);
        } else if n >= 2 {
            let value = match parse_number(&value_text) {
                Some(v) => Value::Number(v),
                None => Value::Text(value_text.clone()),
            };
            self.report.set(r, 5, value, self.s_value);
            if !unit_text.is_empty() {
                self.report.set(r, 6, Value::Text(unit_text.clone()), self.s_unit);
            }
        }
        if let Some(sheet) = link {
            self.report.links.push((xlsx::cell_ref(5, r), sheet));
        }
        let longest = [
            c_expr.chars().count(),
            c_with.chars().count(),
            if has_matrix && n == 2 {
                result_full.chars().count()
            } else {
                0
            },
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        let lines = Self::lines(longest, 32).max(Self::lines(description.chars().count(), 36));
        if lines > 1.0 {
            self.report.heights.insert(r, lines.min(8.0) * 15.0);
        }
        self.row += 1;
        // The unit may follow in the text, for a plain unitless number.
        self.after_eq = if n >= 2 && !has_matrix && unit_text.is_empty() {
            Some((r, name))
        } else {
            None
        };
    }

    fn inline(&mut self, n: &Node) {
        match n {
            Node::Text(t) => self.push_text(t),
            Node::Comment(_) => {}
            Node::Element(e) => {
                if is_hidden(e)
                    || matches!(
                        e.name.as_str(),
                        "script" | "style" | "head" | "title" | "meta" | "link" | "input" | "noscript"
                    )
                {
                    return;
                }
                if is_equation(e) {
                    self.equation(e);
                } else if e.name == "br" {
                    self.flush();
                } else if e.name == "sup" && self.after_eq.is_some() {
                    // The exponent of a unit written after an equation: mm<sup>2</sup>.
                    let mut inner = String::new();
                    lin_children(e, &mut inner);
                    let (row, name) = self.after_eq.clone().unwrap();
                    let sup = superscript(inner.trim());
                    if let (Some(sup), Some(unit)) = (sup, self.unit_cell_text(row)) {
                        let combined = format!("{}{}", unit, sup);
                        if self.replace_unit(row, &name, &combined) {
                            return;
                        }
                    }
                    self.after_eq = None;
                    let mut s = String::new();
                    lin_element(e, &mut s);
                    self.pending.push_str(&s);
                } else if e.name == "select" {
                    self.after_eq = None;
                    let t = selected_option(e);
                    self.pending.push_str(&t);
                } else if BLOCKS.contains(&e.name.as_str()) {
                    self.block(e);
                } else {
                    self.after_eq = None;
                    // Inline markup: its children go on the same line.
                    for c in &e.children {
                        match c {
                            Node::Element(x) if is_equation(x) => self.equation(x),
                            Node::Element(x) if !is_hidden(x) && (x.name == "sub" || x.name == "sup") => {
                                let mut s = String::new();
                                lin_element(x, &mut s);
                                self.pending.push_str(&s);
                            }
                            _ => self.inline(c),
                        }
                    }
                }
            }
        }
    }

    fn unit_cell_text(&self, row: u32) -> Option<String> {
        match self.report.cells.get(&row).and_then(|c| c.get(&6)).map(|c| &c.value) {
            Some(Value::Text(t)) => Some(t.clone()),
            _ => None,
        }
    }

    fn replace_unit(&mut self, row: u32, name: &str, unit: &str) -> bool {
        if !valid_unit(unit) {
            return false;
        }
        self.report.set(row, 6, Value::Text(unit.to_string()), self.s_unit);
        self.inferred.units.insert(name.to_string(), Some(unit.to_string()));
        true
    }

    fn children(&mut self, e: &Element) {
        for c in &e.children {
            self.inline(c);
        }
    }

    fn block(&mut self, e: &Element) {
        if is_hidden(e) {
            return;
        }
        self.flush();
        match e.name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = e.name[1..].parse().unwrap_or(4);
                let saved = (self.heading, self.first_segment);
                self.heading = Some(level);
                self.first_segment = true;
                self.children(e);
                self.flush();
                (self.heading, self.first_segment) = saved;
            }
            "hr" => {}
            "table" => self.table(e),
            "svg" => self.picture(&e.outer_html(), e),
            "img" => {
                if let Some(src) = e.attr("src") {
                    let src = html::decode(src);
                    self.picture(&src, e);
                }
            }
            "ul" | "ol" => {
                let ordered = e.name == "ol";
                let mut n = 0;
                for c in &e.children {
                    if let Node::Element(li) = c {
                        if li.name == "li" {
                            n += 1;
                            self.pending = if ordered {
                                format!("{}. ", n)
                            } else {
                                "\u{2022} ".to_string()
                            };
                            self.after_eq = None;
                            self.children(li);
                            self.flush();
                        }
                    }
                }
            }
            "pre" => {
                let mut t = String::new();
                lin_children(e, &mut t);
                for line in html::decode(&t).lines() {
                    self.pending = line.to_string();
                    self.flush();
                }
            }
            "tr" | "td" | "th" | "thead" | "tbody" => self.children(e),
            _ => self.children(e),
        }
        self.flush();
    }

    fn table(&mut self, t: &Element) {
        if !is_plain_table(t) {
            // A layout table (or an equation grid): its cells, row by row, as the page reads.
            let mut rows = Vec::new();
            collect_rows(t, &mut rows);
            for r in rows {
                for cell in r {
                    if is_hidden(cell) {
                        continue;
                    }
                    self.children(cell);
                    self.flush();
                }
            }
            return;
        }
        let mut rows = Vec::new();
        collect_rows(t, &mut rows);
        let header = self.wb.style(Style {
            bold: true,
            fill: "D9E1F2",
            border: true,
            wrap: true,
            top: true,
            ..Default::default()
        });
        let body = self.wb.style(Style {
            border: true,
            wrap: true,
            top: true,
            ..Default::default()
        });
        let body_num = self.wb.style(Style {
            border: true,
            top: true,
            align: "right",
            ..Default::default()
        });
        let mut occupied: HashMap<(u32, u32), ()> = HashMap::new();
        let first = self.row;
        for (ri, r) in rows.iter().enumerate() {
            let row = first + ri as u32;
            let mut col = 1u32;
            for cell in r {
                while occupied.contains_key(&(row, col)) {
                    col += 1;
                }
                let colspan = cell.attr_int("colspan", 1).clamp(1, 40) as u32;
                let rowspan = cell.attr_int("rowspan", 1).clamp(1, 200) as u32;
                let mut s = String::new();
                lin_children(cell, &mut s);
                let text = collapse(&s);
                let is_header = cell.name == "th";
                let (value, style) = match parse_number(&text) {
                    Some(v) if !is_header => (Value::Number(v), body_num),
                    _ => (Value::Text(text), if is_header { header } else { body }),
                };
                self.report.set(row, col, value, style);
                for dr in 0..rowspan {
                    for dc in 0..colspan {
                        occupied.insert((row + dr, col + dc), ());
                        if dr > 0 || dc > 0 {
                            self.report.set(row + dr, col + dc, Value::Empty, style);
                        }
                    }
                }
                if colspan > 1 || rowspan > 1 {
                    self.report.merges.push(format!(
                        "{}:{}",
                        xlsx::cell_ref(col, row),
                        xlsx::cell_ref(col + colspan - 1, row + rowspan - 1)
                    ));
                }
                col += colspan;
            }
        }
        self.row = first + rows.len() as u32 + 1;
    }

    fn picture(&mut self, src: &str, e: &Element) {
        let Some((png, ext, w, h)) = self.load_picture(src, e) else {
            return;
        };
        let (mut w, mut h) = (w, h);
        if w > MAX_PICTURE_WIDTH {
            h = h * MAX_PICTURE_WIDTH / w;
            w = MAX_PICTURE_WIDTH;
        }
        let (w, h) = (w.round().max(1.0) as u32, h.round().max(1.0) as u32);
        let name = e
            .attr("alt")
            .map(html::decode)
            .filter(|a| !a.trim().is_empty())
            .unwrap_or_else(|| format!("Drawing {}", self.report.pictures.len() + 1));
        self.report.pictures.push(xlsx::Picture {
            col: 0,
            row: self.row - 1,
            width: w,
            height: h,
            png,
            ext,
            name,
        });
        self.row += (h as f64 / 20.0).ceil() as u32 + 1;
        self.after_eq = None;
    }

    /// (bytes, extension, width, height) of the picture in pixels as shown.
    fn load_picture(&self, src: &str, e: &Element) -> Option<(Vec<u8>, &'static str, f64, f64)> {
        let attr_w = e.attr("width").map(crate::css::parse_size_px).filter(|v| *v > 0.0);
        let attr_h = e.attr("height").map(crate::css::parse_size_px).filter(|v| *v > 0.0);
        let style = e.attr("style").unwrap_or("");
        let css_w = Some(crate::css::css_size_px(style, "width")).filter(|v| *v > 0.0);
        let css_h = Some(crate::css::css_size_px(style, "height")).filter(|v| *v > 0.0);
        let (want_w, want_h) = (css_w.or(attr_w), css_h.or(attr_h));
        let svg_text: Option<String>;
        let bytes: Option<Vec<u8>>;
        let ext: String;
        if src.trim_start().to_ascii_lowercase().starts_with("<svg") {
            svg_text = Some(src.to_string());
            bytes = None;
            ext = "svg".into();
        } else if src.to_ascii_lowercase().starts_with("data:image/") {
            let start = src.find('/')? + 1;
            let end = src.find(';')?;
            let comma = src.find(',')?;
            ext = check_type(&src[start..end].to_ascii_lowercase()).ok()?;
            let b = base64_decode(&src[comma + 1..])?;
            if ext == "svg" {
                svg_text = Some(String::from_utf8_lossy(&b).to_string());
                bytes = None;
            } else {
                svg_text = None;
                bytes = Some(b);
            }
        } else {
            let b = self.resolver.and_then(|r| r(src))?;
            ext = check_type(&extension_of(src)).ok()?;
            if ext == "svg" {
                svg_text = Some(String::from_utf8_lossy(&b).to_string());
                bytes = None;
            } else {
                svg_text = None;
                bytes = Some(b);
            }
        }
        if let Some(svg) = svg_text {
            let (sw, sh) = svg_size(&svg).unwrap_or((want_w.unwrap_or(400.0), want_h.unwrap_or(300.0)));
            let (mut w, mut h) = (sw, sh);
            match (want_w, want_h) {
                (Some(a), Some(b)) => {
                    w = a;
                    h = b;
                }
                (Some(a), None) => {
                    h = sh * a / sw;
                    w = a;
                }
                (None, Some(b)) => {
                    w = sw * b / sh;
                    h = b;
                }
                _ => {}
            }
            let (w, h) = if w > MAX_PICTURE_WIDTH {
                (MAX_PICTURE_WIDTH, h * MAX_PICTURE_WIDTH / w)
            } else {
                (w, h)
            };
            let png = rasterize_svg(&anchor_plot_text(&svg), w, h).ok()?;
            return Some((png, "png", w, h));
        }
        let data = bytes?;
        let (bw, bh) = bitmap_size(&data).map(|(a, b)| (a as f64, b as f64))?;
        let kind: &'static str = match ext.as_str() {
            "png" => "png",
            "jpeg" => "jpeg",
            "gif" => "gif",
            _ => return None,
        };
        let (w, h) = match (want_w, want_h) {
            (Some(a), Some(b)) => (a, b),
            (Some(a), None) => (a, bh * a / bw),
            (None, Some(b)) => (bw * b / bh, b),
            _ => (bw, bh),
        };
        Some((data, kind, w, h))
    }
}

/// The unit a text starts with, when the text is one ("mm", "kN/m", "mm,"): the unit and the
/// byte length consumed. A word of the text such as "with" is not a unit.
fn leading_unit(text: &str, _suffix: &str) -> Option<(String, usize)> {
    let trimmed = text.trim_start_matches(' ');
    let skipped = text.len() - trimmed.len();
    if skipped > 1 {
        return None;
    }
    let end = trimmed
        .find(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':' | ')' | '('))
        .unwrap_or(trimmed.len());
    let token = &trimmed[..end];
    if token.is_empty()
        || !token
            .chars()
            .next()
            .map(|c| c.is_alphabetic() || matches!(c, '\u{00B0}' | '%' | '\u{00B5}'))
            .unwrap_or(false)
    {
        return None;
    }
    valid_unit(token).then(|| (token.to_string(), skipped + end))
}

fn valid_unit(unit: &str) -> bool {
    // Unicode exponents and the middle dot to what the unit parser reads.
    let mut s = String::new();
    for c in unit.chars() {
        if let Some(d) = unsuperscript(c) {
            s.push('^');
            s.push(d);
        } else if c == '\u{00B7}' {
            s.push('*');
        } else {
            s.push(c);
        }
    }
    if s.is_empty() || s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    matches!(calcpad_core::parsers::units_parser::parse(&s, None, false), Ok(Some(_)))
}

fn rows_to_sheet(wb: &mut Workbook, name: &str, rows: &[Vec<String>], note: Option<&HashMap<String, String>>) -> Sheet {
    let header = wb.style(Style {
        bold: true,
        fill: "D9E1F2",
        border: true,
        ..Default::default()
    });
    let cell = wb.style(Style {
        border: true,
        ..Default::default()
    });
    let mut sheet = Sheet::new(name);
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
    let mut widths = vec![8usize; cols + 1];
    for (ri, r) in rows.iter().enumerate() {
        for (ci, t) in r.iter().enumerate() {
            let (value, style) = if ri == 0 {
                (Value::Text(t.clone()), header)
            } else {
                match parse_cell_number(t) {
                    Some(v) => (Value::Number(v), cell),
                    None => (Value::Text(t.clone()), cell),
                }
            };
            widths[ci] = widths[ci].max(t.chars().count() + 2).min(48);
            sheet.set(ri as u32 + 1, ci as u32 + 1, value, style);
        }
    }
    if let Some(notes) = note {
        if !notes.is_empty() {
            sheet.set(1, 4, Value::Text("Note".into()), header);
            for (ri, r) in rows.iter().enumerate().skip(1) {
                if let Some(n) = r.first().and_then(|name| notes.get(name)) {
                    sheet.set(ri as u32 + 1, 4, Value::Text(n.clone()), cell);
                    widths[3] = widths[3].max(n.chars().count() + 2).min(48);
                }
            }
        }
    }
    sheet.widths = widths
        .iter()
        .enumerate()
        .filter(|(i, _)| *i < cols + 1)
        .map(|(i, w)| (i as u32 + 1, i as u32 + 1, *w as f64))
        .collect();
    sheet.freeze_rows = 1;
    sheet
}

/// A cell that is a plain number (a complex number, a name or a reference stays text).
fn parse_cell_number(t: &str) -> Option<f64> {
    let first = t.chars().next()?;
    if !(first.is_ascii_digit() || first == '-') {
        return None;
    }
    if t.chars().any(|c| c.is_ascii_alphabetic() && c != 'E') || t.contains(' ') {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// The workbook: the report sheet, then `sheets` (the variables, the vectors and matrices, the
/// inputs).
pub fn html_to_xlsx(html: &str, sheets: &[DataSheet], resolver: Option<&ImageResolver>) -> Result<Vec<u8>, String> {
    let doc = html::parse(html);
    let body = doc.find("body").unwrap_or(&doc);
    let sheet_of: HashMap<String, String> = sheets
        .iter()
        .filter_map(|s| s.variable.clone().map(|v| (v, s.name.clone())))
        .collect();
    let mut b = Builder::new(sheet_of, resolver);
    b.header();
    b.children(body);
    b.flush();
    let inferred: HashMap<String, String> = b
        .inferred
        .units
        .iter()
        .filter_map(|(k, v)| v.clone().map(|u| (k.clone(), u)))
        .collect();
    let mut wb = std::mem::take(&mut b.wb);
    let report = std::mem::take(&mut b.report);
    wb.sheets.push(report);
    // The variables take the units the text gave them, and say so.
    let mut notes: HashMap<String, String> = HashMap::new();
    let mut sheets = sheets.to_vec();
    if let Some(v) = sheets.iter_mut().find(|s| s.name == "Variables") {
        for row in v.rows.iter_mut().skip(1) {
            if row.len() >= 3 && row[2].is_empty() && parse_cell_number(&row[1]).is_some() {
                if let Some(u) = inferred.get(&row[0]) {
                    row[2] = u.clone();
                    notes.insert(row[0].clone(), "unit read from the text after the value".to_string());
                }
            }
        }
    }
    for s in &sheets {
        let note = (s.name == "Variables").then_some(&notes);
        let sh = rows_to_sheet(&mut wb, &s.name, &s.rows, note);
        wb.sheets.push(sh);
    }
    Ok(wb.finish())
}
