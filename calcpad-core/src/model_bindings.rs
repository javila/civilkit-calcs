//! Model bindings: comments that tie a worksheet input to a quantity of a structural model
//! (docs/model-bindings.md). This module only reads them and marks the bound inputs in the
//! output; nothing here fetches a value from a model.
//!
//! A binding is a comment line placed directly before the input it binds:
//!
//! ```text
//! '<!--civilkit: bind N_d = member.N_c @ ULS envelope -->
//! 'Design axial compression -'N_d = ? {150}kN
//! ```

/// What a bound quantity measures. Every dimensioned quantity arrives as a number with a unit, so
/// its input must carry a unit; a ratio has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    Force,
    Moment,
    ForcePerLength,
    Length,
    Area,
    SectionModulus,
    SecondMoment,
    WarpingConstant,
    Stress,
    Ratio,
}

impl Dimension {
    pub fn name(self) -> &'static str {
        match self {
            Dimension::Force => "a force",
            Dimension::Moment => "a moment",
            Dimension::ForcePerLength => "a force per length",
            Dimension::Length => "a length",
            Dimension::Area => "an area",
            Dimension::SectionModulus => "a section modulus",
            Dimension::SecondMoment => "a second moment of area",
            Dimension::WarpingConstant => "a warping constant",
            Dimension::Stress => "a stress",
            Dimension::Ratio => "a ratio",
        }
    }
}

/// An action effect comes from an analysis and needs a load selector; a property does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ActionEffect,
    Property,
}

/// The quantities each object offers: (object, quantity, dimension, kind).
pub const QUANTITIES: &[(&str, &str, Dimension, Kind)] = {
    use Dimension::*;
    use Kind::*;
    &[
        ("member", "N_c", Force, ActionEffect),
        ("member", "N_t", Force, ActionEffect),
        ("member", "V_x", Force, ActionEffect),
        ("member", "V_y", Force, ActionEffect),
        ("member", "M_x", Moment, ActionEffect),
        ("member", "M_y", Moment, ActionEffect),
        ("member", "T", Moment, ActionEffect),
        ("member", "M_x_end1", Moment, ActionEffect),
        ("member", "M_x_end2", Moment, ActionEffect),
        ("member", "w", ForcePerLength, ActionEffect),
        ("member", "delta", Length, ActionEffect),
        ("member", "length", Length, Property),
        ("member", "L_y", Length, Property),
        ("member", "L_b", Length, Property),
        ("member", "k_e_x", Ratio, Property),
        ("member", "k_e_y", Ratio, Property),
        ("node", "R_x", Force, ActionEffect),
        ("node", "R_y", Force, ActionEffect),
        ("node", "R_z", Force, ActionEffect),
        ("node", "M_rx", Moment, ActionEffect),
        ("node", "M_ry", Moment, ActionEffect),
        ("node", "M_rz", Moment, ActionEffect),
        ("node", "D_x", Length, ActionEffect),
        ("node", "D_y", Length, ActionEffect),
        ("node", "D_z", Length, ActionEffect),
        ("footing", "N", Force, ActionEffect),
        ("footing", "V", Force, ActionEffect),
        ("footing", "M_x", Moment, ActionEffect),
        ("footing", "M_y", Moment, ActionEffect),
        ("footing", "B_x", Length, Property),
        ("footing", "B_y", Length, Property),
        ("footing", "D", Length, Property),
        ("footing", "column_b_x", Length, Property),
        ("footing", "column_b_y", Length, Property),
        ("connection", "N", Force, ActionEffect),
        ("connection", "V", Force, ActionEffect),
        ("connection", "M", Moment, ActionEffect),
        ("section", "d", Length, Property),
        ("section", "b_f", Length, Property),
        ("section", "t_f", Length, Property),
        ("section", "t_w", Length, Property),
        ("section", "b", Length, Property),
        ("section", "D", Length, Property),
        ("section", "A_g", Area, Property),
        ("section", "I_x", SecondMoment, Property),
        ("section", "I_y", SecondMoment, Property),
        ("section", "Z_x", SectionModulus, Property),
        ("section", "Z_y", SectionModulus, Property),
        ("section", "S_x", SectionModulus, Property),
        ("section", "S_y", SectionModulus, Property),
        ("section", "J", SecondMoment, Property),
        ("section", "I_w", WarpingConstant, Property),
        ("section", "r_x", Length, Property),
        ("section", "r_y", Length, Property),
        ("material", "f_y", Stress, Property),
        ("material", "f_y_flange", Stress, Property),
        ("material", "f_y_web", Stress, Property),
        ("material", "f_u", Stress, Property),
        ("material", "E", Stress, Property),
        ("material", "G", Stress, Property),
        ("material", "f_c", Stress, Property),
    ]
};

/// Which results an action effect is taken from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    /// The extreme over every strength combination (`ULS envelope`).
    UlsEnvelope,
    /// The extreme over every serviceability combination (`SLS envelope`).
    SlsEnvelope,
    /// One load case (`case G`).
    Case(String),
    /// One combination (`combination 1.2G+1.5Q`).
    Combination(String),
}

impl Selector {
    pub fn text(&self) -> String {
        match self {
            Selector::UlsEnvelope => "ULS envelope".to_string(),
            Selector::SlsEnvelope => "SLS envelope".to_string(),
            Selector::Case(n) => format!("case {n}"),
            Selector::Combination(n) => format!("combination {n}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    /// 1-based line of the comment.
    pub line: usize,
    pub variable: String,
    pub object: String,
    pub quantity: String,
    pub dimension: Dimension,
    pub selector: Option<Selector>,
    /// The comment as written, without the leading quote: how it appears in the output.
    pub comment: String,
}

impl Binding {
    pub fn path(&self) -> String {
        format!("{}.{}", self.object, self.quantity)
    }

    pub fn describe(&self) -> String {
        match &self.selector {
            Some(s) => format!("{} @ {}", self.path(), s.text()),
            None => self.path(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BindingError {
    pub line: usize,
    pub message: String,
    pub comment: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bindings {
    pub bindings: Vec<Binding>,
    pub errors: Vec<BindingError>,
}

const PREFIX: &str = "<!--civilkit:";

/// The comment text of a binding line (`<!--civilkit: ... -->`), if the line is one.
fn comment_of(line: &str) -> Option<&str> {
    let t = line.trim_start_matches([' ', '\t']).strip_prefix('\'')?.trim_end();
    if t.starts_with(PREFIX) && t.ends_with("-->") {
        Some(t)
    } else {
        None
    }
}

/// The input that a binding's next line defines: `(variable, text after the closing brace)`.
fn input_of<'a>(line: &'a str, variable: &str) -> Option<&'a str> {
    let pattern = format!("{variable} = ? {{");
    let mut from = 0;
    while let Some(i) = line[from..].find(&pattern).map(|i| i + from) {
        let before = line[..i].chars().last();
        if matches!(before, None | Some('\'' | '\t' | ' ')) {
            let rest = &line[i + pattern.len()..];
            let close = rest.find('}')?;
            return Some(&rest[close + 1..]);
        }
        from = i + pattern.len();
    }
    None
}

fn parse_one(line_no: usize, comment: &str, next: Option<&str>) -> Result<Binding, String> {
    let body = comment[PREFIX.len()..comment.len() - 3].trim();
    let rest = body.strip_prefix("bind ").ok_or_else(|| {
        format!(
            "unknown instruction \"{}\" (only \"bind\" is defined)",
            body.split(' ').next().unwrap_or("")
        )
    })?;
    let (variable, target) = rest.split_once('=').ok_or("expected \"bind <variable> = <path>\"")?;
    let variable = variable.trim();
    if variable.is_empty() || !crate::validator::is_variable(variable) {
        return Err(format!("\"{variable}\" is not a variable name"));
    }
    let (path, selector) = match target.split_once('@') {
        Some((p, s)) => (p.trim(), Some(s.trim())),
        None => (target.trim(), None),
    };
    let (object, quantity) = path
        .split_once('.')
        .ok_or_else(|| format!("\"{path}\" is not <object>.<quantity>"))?;
    let &(_, _, dimension, kind) = QUANTITIES
        .iter()
        .find(|q| q.0 == object && q.1 == quantity)
        .ok_or_else(|| {
            if QUANTITIES.iter().any(|q| q.0 == object) {
                format!("{object} has no quantity \"{quantity}\"")
            } else {
                format!("unknown object \"{object}\" (member, node, footing, connection, section or material)")
            }
        })?;
    let selector = match selector {
        None => None,
        Some("ULS envelope") => Some(Selector::UlsEnvelope),
        Some("SLS envelope") => Some(Selector::SlsEnvelope),
        Some(s) => match s.split_once(' ') {
            Some(("case", n)) if !n.trim().is_empty() => Some(Selector::Case(n.trim().to_string())),
            Some(("combination", n)) if !n.trim().is_empty() => Some(Selector::Combination(n.trim().to_string())),
            _ => {
                return Err(format!(
                    "unknown selector \"{s}\" (ULS envelope, SLS envelope, case <name> or combination <name>)"
                ))
            }
        },
    };
    match (kind, &selector) {
        (Kind::ActionEffect, None) => {
            return Err(format!(
                "{path} comes from an analysis and needs a selector, e.g. \"@ ULS envelope\""
            ))
        }
        (Kind::Property, Some(_)) => return Err(format!("{path} is a property and takes no selector")),
        _ => {}
    }
    let unit = next
        .and_then(|l| input_of(l, variable))
        .ok_or_else(|| format!("must be followed directly by the input \"{variable} = ? {{...}}\""))?;
    let unit = unit.split('\'').next().unwrap_or("").trim();
    match (dimension, unit.is_empty()) {
        (Dimension::Ratio, false) => return Err(format!("{path} is a ratio: the input takes no unit")),
        (d, true) if d != Dimension::Ratio => {
            return Err(format!("{path} is {}: the input needs a unit", d.name()));
        }
        _ => {}
    }
    Ok(Binding {
        line: line_no,
        variable: variable.to_string(),
        object: object.to_string(),
        quantity: quantity.to_string(),
        dimension,
        selector,
        comment: comment.to_string(),
    })
}

/// Reads every binding comment of a worksheet.
pub fn parse(source: &str) -> Bindings {
    // The macro parser's output ends each line with a vertical tab and the source line number.
    let lines: Vec<&str> = source.lines().map(|l| l.split('\u{b}').next().unwrap_or("")).collect();
    let mut out = Bindings::default();
    for (i, line) in lines.iter().enumerate() {
        let Some(comment) = comment_of(line) else { continue };
        let next = lines[i + 1..].iter().find(|l| !l.trim().is_empty()).copied();
        match parse_one(i + 1, comment, next) {
            Ok(b) if out.bindings.iter().any(|o| o.variable == b.variable) => out.errors.push(BindingError {
                line: i + 1,
                message: format!("{} is bound twice", b.variable),
                comment: comment.to_string(),
            }),
            Ok(b) => out.bindings.push(b),
            Err(message) => out.errors.push(BindingError {
                line: i + 1,
                message,
                comment: comment.to_string(),
            }),
        }
    }
    out
}

fn attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const MARK_STYLE: &str = "margin-left:0.6em;padding:0 0.35em;border:1px solid currentColor;border-radius:3px;\
font:75% sans-serif;opacity:0.8;white-space:nowrap";

fn marker(b: &Binding) -> String {
    let selector = b.selector.as_ref().map(|s| s.text()).unwrap_or_default();
    format!(
        "<span class=\"cprs-bind\" data-bind-var=\"{}\" data-bind-path=\"{}\" data-bind-selector=\"{}\" \
         title=\"Bound to the model: {}. No model is connected, so the value typed here is used.\" \
         style=\"{MARK_STYLE}\"><b>model</b> {}</span>",
        attr(&b.variable),
        attr(&b.path()),
        attr(&selector),
        attr(&b.describe()),
        attr(&b.describe())
    )
}

fn error_marker(e: &BindingError) -> String {
    format!(
        "<span class=\"cprs-bind cprs-bind-error\" title=\"{}\" style=\"{MARK_STYLE};color:#c00\"><b>model binding error</b> {}</span>",
        attr(&e.comment),
        attr(&e.message)
    )
}

/// Marks each bound input in the rendered output: a "model" tag with the binding path at the end
/// of the input's paragraph (the first one after the comment), or the error where a binding is
/// malformed. A binding in a branch that was not rendered has no comment in the output and is
/// left alone.
pub fn annotate_html(html: &str, bindings: &Bindings) -> String {
    let mut items: Vec<(usize, &str, String)> = bindings
        .bindings
        .iter()
        .map(|b| (b.line, b.comment.as_str(), marker(b)))
        .chain(
            bindings
                .errors
                .iter()
                .map(|e| (e.line, e.comment.as_str(), error_marker(e))),
        )
        .collect();
    items.sort_by_key(|i| i.0);
    let mut out = String::with_capacity(html.len());
    let mut pos = 0;
    for (_, comment, mark) in items {
        let Some(at) = html[pos..].find(comment).map(|i| i + pos + comment.len()) else {
            continue;
        };
        let insert = match html[at..].find("</p>") {
            Some(p) => at + p,
            None => at,
        };
        out.push_str(&html[pos..insert]);
        out.push_str(&mark);
        pos = insert;
    }
    out.push_str(&html[pos..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(src: &str) -> Bindings {
        parse(src)
    }

    #[test]
    fn reads_a_binding_with_its_selector() {
        let b = one("'<!--civilkit: bind N_d = member.N_c @ ULS envelope -->\n'Axial -'N_d = ? {150}kN\n");
        assert!(b.errors.is_empty(), "{:?}", b.errors);
        let b = &b.bindings[0];
        assert_eq!(
            (b.line, b.variable.as_str(), b.path()),
            (1, "N_d", "member.N_c".to_string())
        );
        assert_eq!(b.selector, Some(Selector::UlsEnvelope));
        assert_eq!(b.dimension, Dimension::Force);
    }

    #[test]
    fn reads_case_and_combination_selectors_and_properties() {
        let b = one(concat!(
            "'<!--civilkit: bind G = member.M_x @ case G -->\nG = ? {40}kNm\n",
            "'<!--civilkit: bind M = member.M_x @ combination 1.2G+1.5Q -->\nM = ? {1}kNm\n",
            "'<!--civilkit: bind k = member.k_e_x -->\nk = ? {1}\n",
            "'<!--civilkit: bind d = section.d -->\n\n'Depth -'d = ? {304}mm\n",
        ));
        assert!(b.errors.is_empty(), "{:?}", b.errors);
        let s: Vec<_> = b.bindings.iter().map(|b| b.describe()).collect();
        assert_eq!(
            s,
            [
                "member.M_x @ case G",
                "member.M_x @ combination 1.2G+1.5Q",
                "member.k_e_x",
                "section.d"
            ]
        );
    }

    #[test]
    fn refuses_malformed_bindings() {
        let cases = [
            ("'<!--civilkit: link N = member.N_c @ ULS envelope -->\nN = ? {1}kN", "unknown instruction"),
            ("'<!--civilkit: bind N = beam.N_c @ ULS envelope -->\nN = ? {1}kN", "unknown object"),
            ("'<!--civilkit: bind N = member.Q @ ULS envelope -->\nN = ? {1}kN", "no quantity"),
            ("'<!--civilkit: bind N = member.N_c -->\nN = ? {1}kN", "needs a selector"),
            ("'<!--civilkit: bind d = section.d @ case G -->\nd = ? {1}mm", "takes no selector"),
            ("'<!--civilkit: bind N = member.N_c @ worst -->\nN = ? {1}kN", "unknown selector"),
            ("'<!--civilkit: bind N = member.N_c @ ULS envelope -->\nN = 3kN", "must be followed"),
            ("'<!--civilkit: bind N = member.N_c @ ULS envelope -->\nX = ? {1}kN", "must be followed"),
            ("'<!--civilkit: bind N = member.N_c @ ULS envelope -->\nN = ? {1}", "needs a unit"),
            ("'<!--civilkit: bind k = member.k_e_x -->\nk = ? {1}m", "takes no unit"),
            ("'<!--civilkit: bind N = member.N_c @ ULS envelope -->\nN = ? {1}kN\n'<!--civilkit: bind N = member.N_t @ ULS envelope -->\nN = ? {1}kN", "bound twice"),
        ];
        for (src, want) in cases {
            let b = one(src);
            assert_eq!(b.errors.len(), 1, "{src}");
            assert!(b.errors[0].message.contains(want), "{src}: {}", b.errors[0].message);
        }
    }

    #[test]
    fn reads_the_macro_parser_output() {
        let b = one("'<!--civilkit: bind N_d = member.N_c @ ULS envelope -->\u{b}1\n'Axial -'N_d = ? {150}kN\u{b}2\n");
        assert!(b.errors.is_empty(), "{:?}", b.errors);
        assert_eq!(
            b.bindings[0].comment,
            "<!--civilkit: bind N_d = member.N_c @ ULS envelope -->"
        );
    }

    #[test]
    fn plain_comments_are_not_bindings() {
        let b = one("'<!-- a description -->\n'text\nx = ? {1}\n");
        assert_eq!(b, Bindings::default());
    }

    #[test]
    fn marks_the_bound_input_in_the_output() {
        let src = "'<!--civilkit: bind N_d = member.N_c @ ULS envelope -->\n'Axial -'N_d = ? {150}kN\n'<!--civilkit: bind x = member.N_c -->\nx = ? {1}kN\ny = 2\n";
        let mut p = crate::parsers::expression_parser::ExpressionParser::new();
        p.parse(src, true);
        let html = annotate_html(&p.html_result, &parse(src));
        let mark = html
            .find("<b>model</b> member.N_c @ ULS envelope</span>")
            .expect("marker");
        let input = html.find("150").unwrap();
        assert!(input < mark, "the marker follows the input");
        assert!(html[mark..].find("</p>").unwrap() < html[mark..].find("x</var>").unwrap());
        assert!(html.contains("<b>model binding error</b> member.N_c comes from an analysis"));
        assert!(html.contains("data-bind-var=\"N_d\""));
    }
}
