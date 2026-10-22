//! The translator: a [`Sheet`] in, Calcpad text and a report out. It does no arithmetic and
//! evaluates nothing; every number it writes is a literal that was in the source.
//!
//! Safety rule: a line of output is built only from (a) text that goes through the escaping in
//! [`text_line`], (b) names that passed [`valid_ident`], (c) numbers that parsed as finite
//! numbers, (d) entries of the mapping tables, or (e) a placeholder line that starts with
//! [`GAP_MARKER`]. Nothing else from the input reaches the output.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ir::*;
use crate::picture::{self, SvgError};
use crate::report::{Code, ImportReport, Issue, KindStats, Severity};
use crate::tables::{self, Constant, FnRule};
use crate::{Asset, ImportOptions};

/// Every placeholder line starts with this (after the leading `'` that makes it a text line).
pub const GAP_MARKER: &str = "[import-gap";

/// Rows closer than this many points (by baseline) are one row for reading order.
pub const ROW_TOLERANCE: f64 = 4.0;

// Precedence levels of the rendered Calcpad text.
const P_LOGIC: u8 = 1;
const P_CMP: u8 = 2;
const P_ADD: u8 = 3;
const P_MUL: u8 = 4;
const P_NEG: u8 = 5;
const P_POW: u8 = 6;
const P_ATOM: u8 = 7;

struct R {
    s: String,
    prec: u8,
}

fn atom(s: impl Into<String>) -> R {
    R {
        s: s.into(),
        prec: P_ATOM,
    }
}

struct Pending {
    code: Code,
    detail: Option<String>,
    root: Option<Code>,
}

/// Sorts regions into the order the source evaluates them: top to bottom, then left to right,
/// where regions whose baselines are within [`ROW_TOLERANCE`] of the first region of a row share
/// that row. The baseline (not the top edge) is used because a tall region next to a short one
/// sits on the same line while their top edges differ.
pub fn reading_order(regions: &mut Vec<Region>) {
    let key = |r: &Region| r.baseline.unwrap_or(r.top);
    let mut idx: Vec<usize> = (0..regions.len()).collect();
    idx.sort_by(|&a, &b| {
        key(&regions[a])
            .partial_cmp(&key(&regions[b]))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    // Group into rows, then order each row by left edge.
    let mut ordered: Vec<usize> = Vec::with_capacity(idx.len());
    let mut i = 0;
    while i < idx.len() {
        let anchor = key(&regions[idx[i]]);
        let mut j = i;
        while j < idx.len() && key(&regions[idx[j]]) - anchor <= ROW_TOLERANCE {
            j += 1;
        }
        let mut row: Vec<usize> = idx[i..j].to_vec();
        row.sort_by(|&a, &b| {
            regions[a]
                .left
                .partial_cmp(&regions[b].left)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        ordered.extend(row);
        i = j;
    }
    let mut old: Vec<Option<Region>> = std::mem::take(regions).into_iter().map(Some).collect();
    regions.extend(ordered.iter().filter_map(|&src| old[src].take()));
}

/// Calcpad identifiers: a letter, then letters, digits, `_` and primes.
pub fn valid_ident(s: &str) -> bool {
    let mut ch = s.chars();
    match ch.next() {
        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    ch.all(|c| c.is_alphanumeric() || c == '_' || c == '\u{2032}' || c == '\u{2033}')
}

fn is_int_literal(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

/// Calcpad does not read `1.5e-3`: rewrites a literal with an exponent as plain digits (exactly,
/// by moving the decimal point) or, for a large exponent, as `(m*10^e)`.
fn expand_literal(s: &str) -> Option<String> {
    let Some(epos) = s.find(['e', 'E']) else {
        return Some(s.to_string());
    };
    let (mant, exp) = (&s[..epos], &s[epos + 1..]);
    let exp: i32 = exp.trim_start_matches('+').parse().ok()?;
    let (neg, mant) = match mant.strip_prefix('-') {
        Some(m) => (true, m),
        None => (false, mant),
    };
    let (int, frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    if int.is_empty() && frac.is_empty() || !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let sign = if neg { "-" } else { "" };
    if exp.abs() > 20 {
        return Some(format!("({}{}*10^({}))", sign, mant, exp));
    }
    let digits = format!("{}{}", int, frac);
    let point = int.len() as i32 + exp; // position of the decimal point within `digits`
    let body = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
    } else {
        format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
    };
    // Tidy leading zeros of the integer part and trailing zeros of the fraction.
    let body = match body.split_once('.') {
        Some((i, f)) => {
            let i = i.trim_start_matches('0');
            let f = f.trim_end_matches('0');
            let i = if i.is_empty() { "0" } else { i };
            if f.is_empty() {
                i.to_string()
            } else {
                format!("{}.{}", i, f)
            }
        }
        None => {
            let b = body.trim_start_matches('0');
            if b.is_empty() {
                "0".to_string()
            } else {
                b.to_string()
            }
        }
    };
    Some(format!("{}{}", sign, body))
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(16)
        .collect()
}

/// Escapes one line of prose for a Calcpad text line (without the leading quote mark).
fn escape_text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '\'' => o.push_str("''"),
            '"' => o.push_str("\"\""),
            c if c.is_control() => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

struct Tr {
    origin: i64,
    defined: HashSet<String>,
    funcs: HashMap<String, usize>,
    /// Names defined by regions that did not convert, with the code that ultimately blocks them.
    gap_names: HashMap<String, Code>,
    /// Defined names known to hold an array.
    arrays: HashSet<String>,
    params: Vec<String>,
    cur: Vec<Pending>,
    uses_trig: bool,
    lines: Vec<String>,
    report: ImportReport,
    depth: usize,
    opts: ImportOptions,
    assets: Vec<Asset>,
    used_paths: HashSet<String>,
    asset_total: usize,
    has_references: bool,
}

const MAX_RENDER_DEPTH: usize = 300;

impl Tr {
    fn issue(&mut self, code: Code, detail: Option<String>) {
        self.cur.push(Pending {
            code,
            detail,
            root: None,
        });
    }

    // ---- expressions ----------------------------------------------------------------------

    fn expr(&mut self, e: &Expr) -> R {
        if self.depth > MAX_RENDER_DEPTH {
            self.issue(Code::TooDeep, None);
            return atom("?too-deep?");
        }
        self.depth += 1;
        let r = self.expr_inner(e);
        self.depth -= 1;
        r
    }

    /// Renders `e` and wraps it in parentheses when it binds looser than `min`.
    fn operand(&mut self, e: &Expr, min: u8, allow_bare_neg: bool) -> String {
        let r = self.expr(e);
        let is_neg = matches!(e, Expr::Un(UnOp::Neg, _)) || matches!(e, Expr::Num(s) if s.starts_with('-'));
        if r.prec < min || (!allow_bare_neg && is_neg) {
            format!("({})", r.s)
        } else {
            r.s
        }
    }

    fn expr_inner(&mut self, e: &Expr) -> R {
        match e {
            Expr::Num(s) => self.number(s),
            Expr::Str(_) => {
                self.issue(Code::StringValue, None);
                atom("?string?")
            }
            Expr::Name(n) => self.name_ref(n),
            Expr::Un(op, x) => self.unary(*op, x),
            Expr::Bin(op, a, b) => self.binary(*op, a, b),
            Expr::Call(n, args) => self.call(n, args),
            Expr::Postfix(unit, v) => {
                let u = self.unit_only(unit);
                let val = self.operand(v, P_MUL, true);
                R {
                    s: format!("{}*{}", val, u),
                    prec: P_MUL,
                }
            }
            Expr::Index(base, idx) => self.index(base, idx),
            Expr::Matrix { rows, cols, items } => self.matrix(*rows, *cols, items),
            Expr::Range { first, second, last } => {
                // Render for the placeholder, record the gap.
                let a = self.expr(first).s;
                let b = second.as_ref().map(|s| self.expr(s).s);
                let c = self.expr(last).s;
                self.issue(Code::RangeVariable, None);
                let s = match b {
                    Some(b) => format!("{}, {} .. {}", a, b, c),
                    None => format!("{} .. {}", a, c),
                };
                atom(s)
            }
            Expr::Unsupported { code, detail } => {
                self.issue(*code, detail.clone());
                atom(format!("?{}?", code.as_str()))
            }
        }
    }

    fn number(&mut self, s: &str) -> R {
        let ok = !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'))
            && s.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false);
        let text = if ok { expand_literal(s) } else { None };
        match text {
            None => {
                self.issue(Code::BadNumber, None);
                atom("?number?")
            }
            Some(t) => {
                let prec = if t.starts_with('-') { P_NEG } else { P_ATOM };
                R { s: t, prec }
            }
        }
    }

    fn unary(&mut self, op: UnOp, x: &Expr) -> R {
        match op {
            UnOp::Neg => {
                let inner = self.operand(x, P_POW, true);
                R {
                    s: format!("-{}", inner),
                    prec: P_NEG,
                }
            }
            UnOp::Abs => {
                if let Expr::Name(n) = x {
                    if self.arrays.contains(&n.flat()) {
                        self.issue(Code::AbsOfArray, None);
                    }
                }
                let inner = self.expr(x).s;
                atom(format!("abs({})", inner))
            }
            UnOp::Sqrt => {
                let inner = self.expr(x).s;
                atom(format!("sqrt({})", inner))
            }
            UnOp::Factorial => {
                let inner = self.operand(x, P_ATOM, true);
                atom(format!("{}!", inner))
            }
            UnOp::Transpose => {
                let inner = self.expr(x).s;
                atom(format!("transp({})", inner))
            }
            UnOp::Not => {
                let _ = self.expr(x);
                self.issue(Code::UnknownElement, Some("not".into()));
                atom("?not?")
            }
        }
    }

    fn binary(&mut self, op: BinOp, a: &Expr, b: &Expr) -> R {
        let (sym, lmin, rmin, prec, rneg): (&str, u8, u8, u8, bool) = match op {
            BinOp::Add => (" + ", P_ADD, P_ADD, P_ADD, false),
            BinOp::Sub => (" - ", P_ADD, P_MUL, P_ADD, false),
            BinOp::Mul => ("*", P_MUL, P_MUL, P_MUL, false),
            BinOp::Div => ("/", P_MUL, P_NEG, P_MUL, false),
            BinOp::Pow => ("^", P_ATOM, P_ATOM, P_POW, false),
            BinOp::Eq => (" == ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Ne => (" ≠ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Lt => (" < ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Gt => (" > ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Le => (" ≤ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Ge => (" ≥ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::And => (" ∧ ", P_CMP, P_CMP, P_LOGIC, false),
            BinOp::Or => (" ∨ ", P_CMP, P_CMP, P_LOGIC, false),
            BinOp::Xor => (" ⊕ ", P_CMP, P_CMP, P_LOGIC, false),
        };
        let _ = rneg;
        let l = self.operand(a, lmin, true);
        // A product's right operand that is itself a division keeps its parentheses.
        let rmin = if op == BinOp::Mul && matches!(b, Expr::Bin(BinOp::Div, ..)) {
            P_NEG
        } else {
            rmin
        };
        let r = self.operand(b, rmin, false);
        R {
            s: format!("{}{}{}", l, sym, r),
            prec,
        }
    }

    fn render_args(&mut self, args: &[Expr]) -> Vec<String> {
        args.iter().map(|a| self.expr(a).s).collect()
    }

    fn unit_only(&mut self, n: &Name) -> String {
        if n.sub.is_none() {
            if let Some(u) = tables::map_unit(&n.base) {
                return u.to_string();
            }
            if tables::is_unmapped_unit(&n.base) {
                self.issue(Code::UnmappedUnit, Some(n.base.clone()));
                return format!("?{}?", n.base);
            }
        }
        self.issue(Code::UnmappedUnit, Some(n.flat()));
        "?unit?".into()
    }

    fn name_ref(&mut self, n: &Name) -> R {
        let flat = tables::normalise_name(&n.flat());
        if self.params.contains(&flat) || self.defined.contains(&flat) || self.funcs.contains_key(&flat) {
            if !valid_ident(&flat) {
                self.issue(Code::InvalidName, Some(flat));
                return atom("?name?");
            }
            return atom(flat);
        }
        if let Some(root) = self.gap_names.get(&flat).copied() {
            self.cur.push(Pending {
                code: Code::DependsOnGap,
                detail: Some(flat),
                root: Some(root),
            });
            return atom("?gap?");
        }
        if n.sub.is_none() {
            if tables::is_solve_name(&n.base) {
                self.issue(Code::SolveBlock, Some(n.base.clone()));
                return atom("?solve?");
            }
            if let Some(c) = tables::lookup_constant(&n.base) {
                return match c {
                    Constant::Text(t) => R {
                        s: t.to_string(),
                        prec: P_ATOM,
                    },
                    Constant::Origin => {
                        if self.origin != 0 {
                            self.issue(Code::ArrayOrigin, None);
                        }
                        atom(self.origin.to_string())
                    }
                };
            }
            if let Some(u) = tables::map_unit(&n.base) {
                return atom(u);
            }
            if tables::is_unmapped_unit(&n.base) {
                self.issue(Code::UnmappedUnit, Some(n.base.clone()));
                return atom(format!("?{}?", n.base));
            }
        }
        self.issue(Code::UndefinedVariable, Some(flat));
        atom("?name?")
    }

    fn call(&mut self, n: &Name, args: &[Expr]) -> R {
        let flat = tables::normalise_name(&n.flat());
        // A variable applied like a function is a product.
        if (self.defined.contains(&flat) || self.params.contains(&flat)) && !self.funcs.contains_key(&flat) {
            if args.len() == 1 {
                self.issue(Code::CallAsMultiplication, None);
                let a = self.operand(&args[0], P_MUL, false);
                return R {
                    s: format!("{}*{}", flat, a),
                    prec: P_MUL,
                };
            }
            self.issue(Code::BadStructure, Some("variable applied to several arguments".into()));
            return atom("?call?");
        }
        let rendered = self.render_args(args);
        if let Some(&arity) = self.funcs.get(&flat) {
            if arity != args.len() {
                self.issue(Code::ArityMismatch, Some(flat.clone()));
            }
            return atom(format!("{}({})", flat, rendered.join("; ")));
        }
        if let Some(root) = self.gap_names.get(&flat).copied() {
            self.cur.push(Pending {
                code: Code::DependsOnGap,
                detail: Some(flat),
                root: Some(root),
            });
            return atom("?gap?");
        }
        if tables::is_solve_name(&flat) {
            self.issue(Code::SolveBlock, Some(flat));
            return atom("?solve?");
        }
        if n.sub.is_none() {
            if let Some((rule, lo, hi)) = tables::lookup_function(&flat) {
                if args.len() < lo || args.len() > hi {
                    self.issue(Code::ArityMismatch, Some(flat));
                    return atom("?call?");
                }
                return self.apply_rule(rule, &rendered);
            }
        }
        if n.sub.is_none() && tables::is_builtin_missing(&flat) {
            self.issue(Code::BuiltinMissing, Some(flat));
        } else if self.has_references {
            self.issue(Code::ReferenceFunction, Some(flat));
        } else {
            self.issue(Code::AddinFunction, Some(flat));
        }
        atom("?call?")
    }

    fn apply_rule(&mut self, rule: FnRule, a: &[String]) -> R {
        match rule {
            FnRule::Direct(name) => atom(format!("{}({})", name, a.join("; "))),
            FnRule::Trig(name) => {
                self.uses_trig = true;
                atom(format!("{}({})", name, a.join("; ")))
            }
            FnRule::Log => {
                if a.len() == 1 {
                    atom(format!("log({})", a[0]))
                } else {
                    self.issue(Code::LogBase, None);
                    atom(format!("(ln({})/ln({}))", a[0], a[1]))
                }
            }
            FnRule::Round => {
                if a.len() == 1 {
                    atom(format!("round({})", a[0]))
                } else {
                    self.issue(Code::RoundDigits, None);
                    atom(format!("(round(({})*10^({}))/10^({}))", a[0], a[1], a[1]))
                }
            }
        }
    }

    /// An array index shifted from the sheet's origin to Calcpad's 1.
    fn index_expr(&mut self, e: &Expr) -> String {
        let delta = 1 - self.origin;
        if delta != 0 {
            self.issue(Code::ArrayOrigin, None);
        }
        if let Expr::Num(s) = e {
            if let Some(v) = is_int_literal(s) {
                return (v + delta).to_string();
            }
        }
        let r = self.operand(e, P_ADD, true);
        match delta {
            0 => r,
            d if d > 0 => format!("{} + {}", r, d),
            d => format!("{} - {}", r, -d),
        }
    }

    fn index(&mut self, base: &Expr, idx: &[Expr]) -> R {
        let b = self.operand(base, P_ATOM, true);
        if idx.is_empty() || idx.len() > 2 {
            self.issue(Code::BadStructure, Some("index list".into()));
            return atom("?index?");
        }
        let parts: Vec<String> = idx.iter().map(|i| self.index_expr(i)).collect();
        atom(format!("{}.({})", b, parts.join("; ")))
    }

    fn matrix(&mut self, rows: usize, cols: usize, items: &[Expr]) -> R {
        // The file stores items column by column.
        let cell: Vec<String> = items.iter().map(|i| self.expr(i).s).collect();
        let at = |r: usize, c: usize| cell[c * rows + r].clone();
        if rows * cols > 1_000_000 {
            self.issue(Code::BadStructure, Some("matrix too large".into()));
            return atom("?matrix?");
        }
        if cols == 1 && rows == 1 {
            return atom(at(0, 0));
        }
        if cols == 1 {
            let v: Vec<String> = (0..rows).map(|r| at(r, 0)).collect();
            return atom(format!("[{}]", v.join("; ")));
        }
        if rows == 1 {
            let v: Vec<String> = (0..cols).map(|c| at(0, c)).collect();
            return atom(format!("transp([{}])", v.join("|")));
        }
        let body: Vec<String> = (0..rows)
            .map(|r| (0..cols).map(|c| at(r, c)).collect::<Vec<_>>().join("; "))
            .collect();
        atom(format!("[{}]", body.join("|")))
    }

    // ---- regions ---------------------------------------------------------------------------

    fn flush_issues(&mut self, id: &str, line: u32, severity_filter: Option<Severity>) {
        let pend = std::mem::take(&mut self.cur);
        for p in pend {
            let sev = p.code.severity();
            if let Some(f) = severity_filter {
                if sev != f {
                    continue;
                }
            }
            self.report.issues.push(Issue {
                code: p.code,
                severity: sev,
                detail: p.detail,
                region_id: Some(id.to_string()),
                line: if line > 0 { Some(line) } else { None },
                reason: p.code.reason().to_string(),
                root: p.root,
            });
        }
    }

    fn bump(map: &mut BTreeMap<String, KindStats>, key: &str, converted: bool, notes: bool) {
        Self::bump_p(map, key, converted, notes, false)
    }

    fn bump_p(map: &mut BTreeMap<String, KindStats>, key: &str, converted: bool, notes: bool, pictured: bool) {
        let s = map.entry(key.to_string()).or_default();
        s.total += 1;
        if converted {
            s.converted += 1;
            if notes {
                s.with_notes += 1;
            }
        } else {
            s.attention += 1;
            if pictured {
                s.pictured += 1;
            }
        }
    }

    fn placeholder(&mut self, id: &str, codes: &[Code], attempt: Option<&str>) {
        let mut names: Vec<&str> = Vec::new();
        for c in codes {
            if !names.contains(&c.as_str()) {
                names.push(c.as_str());
            }
        }
        let mut line = format!(
            "'{} R{} {}] {}",
            GAP_MARKER,
            sanitize_id(id),
            names.join(","),
            escape_text(codes[0].reason())
        );
        if let Some(a) = attempt {
            let a: String = a.chars().take(160).collect();
            line.push_str(" | ");
            line.push_str(&escape_text(&a));
        }
        self.lines.push(line);
    }

    /// Registers a binary asset and returns its library path, or `None` over the size budget.
    fn add_asset(&mut self, id: &str, ext: &str, mime: &'static str, bytes: Vec<u8>) -> Option<String> {
        if self.asset_total.saturating_add(bytes.len()) > self.opts.max_total_asset_bytes {
            return None;
        }
        let base = format!("{}/R{}", self.opts.asset_prefix, sanitize_id(id));
        let mut path = format!("{}.{}", base, ext);
        let mut n = 2;
        while self.used_paths.contains(&path) {
            path = format!("{}-{}.{}", base, n, ext);
            n += 1;
        }
        self.used_paths.insert(path.clone());
        self.asset_total += bytes.len();
        self.assets.push(Asset {
            path: path.clone(),
            mime,
            bytes,
        });
        Some(path)
    }

    fn img_line(path: &str, r: &Region) -> String {
        let style = if r.width > 0.0 {
            format!("width:{:.1}pt;max-width:100%;", r.width.min(900.0))
        } else {
            "max-width:100%;".to_string()
        };
        format!(
            "'<img style=\"{}\" src=\"{}\" alt=\"R{}\">",
            style,
            path,
            sanitize_id(&r.id)
        )
    }

    /// Output for a region that did not convert: a static picture of the original when the option is
    /// on and the file has one, otherwise the placeholder line (plus a labelled saved result when
    /// that option is on). Returns true when a picture was written.
    fn show_gap(&mut self, r: &Region, codes: &[Code], attempt: Option<&str>, saved: Option<&SavedResult>) -> bool {
        if saved.is_some() {
            self.report.pictures.saved_result_available += 1;
        }
        if self.opts.pictures {
            if let Some(png) = &r.rendering {
                let len = png.len();
                if let Some(path) = self.add_asset(&r.id, "png", "image/png", png.clone()) {
                    // One line, so the caption stays under its picture.
                    self.lines.push(format!(
                        "{}<br/><i>{} (R{} {})</i>",
                        Self::img_line(&path, r),
                        picture::PICTURE_CAPTION,
                        sanitize_id(&r.id),
                        codes[0].as_str()
                    ));
                    self.report.pictures.fallback_shown += 1;
                    self.report.pictures.png_bytes += len;
                    self.issue(Code::StaticPicture, None);
                    return true;
                }
            }
        }
        self.placeholder(&r.id, codes, attempt);
        if self.opts.saved_results {
            if let Some(sr) = saved {
                let unit = sr.unit.as_ref().map(|u| format!(" {}", u)).unwrap_or_default();
                self.lines.push(format!(
                    "'<i>saved result from the original, not live (R{}): {}{}</i>",
                    sanitize_id(&r.id),
                    escape_text(&sr.value),
                    escape_text(&unit)
                ));
                self.report.pictures.saved_result_emitted += 1;
            }
        }
        false
    }

    fn simple_gap(&mut self, key: &str, r: &Region, code: Code, detail: Option<String>) {
        self.issue(code, detail);
        let pictured = self.show_gap(r, &[code], None, None);
        self.flush_issues(&r.id, r.line, None);
        Self::bump_p(&mut self.report.kinds, key, false, false, pictured);
    }

    fn picture(&mut self, r: &Region, p: &Picture) {
        if !self.opts.pictures {
            self.simple_gap("image", r, Code::Image, None);
            return;
        }
        let mut note_fallback = false;
        if let Some(mf) = &p.metafile {
            match picture::metafile_to_svg(mf, self.opts.svg_limits) {
                Ok(svg) => {
                    let len = svg.len();
                    if let Some(path) = self.add_asset(&r.id, "svg", "image/svg+xml", svg.into_bytes()) {
                        self.lines.push(Self::img_line(&path, r));
                        self.report.pictures.svg_ok += 1;
                        self.report.pictures.svg_bytes += len;
                        Self::bump(&mut self.report.kinds, "image", true, false);
                        return;
                    }
                    note_fallback = true;
                }
                Err(e) => {
                    note_fallback = true;
                    let why = match e {
                        SvgError::InputTooLarge => "input too large",
                        SvgError::Failed => "conversion failed",
                        SvgError::Aborted => "conversion aborted",
                        SvgError::OutputTooLarge => "output too large",
                        SvgError::Unsafe => "unsafe content",
                    };
                    self.cur.push(Pending {
                        code: Code::SvgFallback,
                        detail: Some(why.to_string()),
                        root: None,
                    });
                }
            }
            self.report.pictures.svg_failed += 1;
        }
        let png = p.png.clone().or_else(|| r.rendering.clone());
        if let Some(png) = png {
            let len = png.len();
            if let Some(path) = self.add_asset(&r.id, "png", "image/png", png) {
                self.lines.push(Self::img_line(&path, r));
                self.report.pictures.png_used += 1;
                self.report.pictures.png_bytes += len;
                if note_fallback && self.cur.is_empty() {
                    self.issue(Code::SvgFallback, None);
                }
                self.flush_issues(&r.id, r.line, None);
                Self::bump_p(&mut self.report.kinds, "image", true, true, false);
                return;
            }
        }
        // Neither usable: the placeholder.
        self.cur.retain(|p| p.code != Code::SvgFallback);
        self.simple_gap("image", r, Code::Image, None);
    }

    fn region(&mut self, r: &Region) {
        match &r.kind {
            RegionKind::PageBreak => self.report.page_breaks += 1,
            RegionKind::Text(paras) => {
                for p in paras {
                    if let Some(l) = text_line(p) {
                        self.lines.push(l);
                    }
                }
                Self::bump(&mut self.report.kinds, "text", true, false);
            }
            RegionKind::Math(m) => self.math(r, m),
            RegionKind::Plot => self.simple_gap("plot", r, Code::Plot, None),
            RegionKind::Picture(p) => self.picture(r, p),
            RegionKind::Ole => self.simple_gap("ole", r, Code::OleObject, None),
            RegionKind::Component => self.simple_gap("component", r, Code::Component, None),
            RegionKind::Reference => self.simple_gap("reference", r, Code::WorksheetReference, None),
            RegionKind::Other(tag) => {
                let d = Some(
                    tag.chars()
                        .filter(|c| c.is_ascii_alphanumeric())
                        .take(32)
                        .collect::<String>(),
                );
                self.simple_gap("other", r, Code::OtherRegion, d)
            }
        }
    }

    fn math(&mut self, r: &Region, m: &MathRegion) {
        self.cur.clear();
        self.params.clear();
        self.depth = 0;
        if m.disabled {
            self.simple_gap("math", r, Code::DisabledRegion, None);
            Self::bump(&mut self.report.math_shapes, "other", false, false);
            return;
        }
        let mut emitted: Option<String> = None;
        let mut attempt: Option<String> = None;
        let mut define_name: Option<(String, Option<usize>, bool)> = None; // name, arity, is array
        let mut origin_assign: Option<i64> = None;
        let shape: &str;

        match &m.stmt {
            Stmt::Define { target, value, global } => {
                shape = if matches!(target, Target::Func(..)) {
                    "function"
                } else {
                    "definition"
                };
                match target {
                    Target::Var(n) => {
                        let flat = tables::normalise_name(&n.flat());
                        if !valid_ident(&flat) {
                            self.issue(Code::InvalidName, Some(flat.clone()));
                        }
                        if flat == "ORIGIN" {
                            if let Expr::Num(s) = value {
                                if let Some(v) = is_int_literal(s) {
                                    origin_assign = Some(v);
                                }
                            }
                        }
                        let rhs = self.expr(value);
                        let is_array = matches!(value, Expr::Matrix { .. })
                            || matches!(value, Expr::Call(f, _) if matches!(f.base.as_str(), "identity" | "matrix" | "augment" | "stack"));
                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                            emitted = Some(format!("{} = {}", flat, rhs.s));
                        } else {
                            attempt = Some(format!("{} = {}", flat, rhs.s));
                        }
                        define_name = Some((flat, None, is_array));
                        if *global {
                            self.issue(Code::GlobalDefinition, None);
                        }
                    }
                    Target::Func(n, params) => {
                        let flat = tables::normalise_name(&n.flat());
                        if !valid_ident(&flat) {
                            self.issue(Code::InvalidName, Some(flat.clone()));
                        }
                        let mut plist = Vec::new();
                        for p in params {
                            let pf = tables::normalise_name(&p.flat());
                            if !valid_ident(&pf) {
                                self.issue(Code::InvalidName, Some(pf.clone()));
                            }
                            plist.push(pf);
                        }
                        self.params = plist.clone();
                        let rhs = self.expr(value);
                        self.params.clear();
                        let head = format!("{}({})", flat, plist.join("; "));
                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                            emitted = Some(format!("{} = {}", head, rhs.s));
                        } else {
                            attempt = Some(format!("{} = {}", head, rhs.s));
                        }
                        define_name = Some((flat, Some(plist.len()), false));
                        if *global {
                            self.issue(Code::GlobalDefinition, None);
                        }
                    }
                    Target::Element => {
                        self.issue(Code::ElementAssignment, None);
                    }
                }
            }
            Stmt::Eval { expr, unit } => {
                shape = "evaluation";
                let e = self.expr(expr);
                let mut text = e.s.clone();
                if let Some(u) = unit {
                    let ur = self.expr(u);
                    text = if e.prec >= P_ATOM { e.s } else { format!("({})", e.s) };
                    text = format!("{}|{}", text, ur.s);
                    self.issue(Code::UnitOverride, None);
                }
                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                    emitted = Some(text);
                } else {
                    attempt = Some(text);
                }
            }
            Stmt::Bare(expr) => {
                shape = "expression";
                let e = self.expr(expr);
                self.issue(Code::BareExpression, None);
                attempt = Some(e.s);
            }
            Stmt::Unsupported { code, detail } => {
                shape = "other";
                self.issue(*code, detail.clone());
            }
        }

        let gap_codes: Vec<Code> = self
            .cur
            .iter()
            .filter(|p| p.code.severity() == Severity::Gap)
            .map(|p| p.code)
            .collect();
        let has_notes = self.cur.iter().any(|p| p.code.severity() == Severity::Note);
        let converted = gap_codes.is_empty();
        let mut pictured = false;

        if converted {
            if let Some(v) = origin_assign {
                self.origin = v;
                self.issue(Code::OriginAssigned, None);
                self.lines
                    .push(format!("'<!-- ORIGIN assigned {} in the source -->", v));
            } else if let Some(line) = emitted.take() {
                self.lines.push(line);
            }
            if let Some((name, arity, is_array)) = define_name {
                self.gap_names.remove(&name);
                match arity {
                    Some(a) => {
                        self.funcs.insert(name.clone(), a);
                        self.defined.remove(&name);
                    }
                    None => {
                        if name != "ORIGIN" {
                            self.defined.insert(name.clone());
                        }
                        self.funcs.remove(&name);
                    }
                }
                if is_array {
                    self.arrays.insert(name);
                } else {
                    self.arrays.remove(&name);
                }
            }
        } else {
            // The root cause this region passes on to the regions that use its name.
            let root = self
                .cur
                .iter()
                .find(|p| p.code == Code::DependsOnGap)
                .and_then(|p| p.root)
                .or_else(|| gap_codes.iter().copied().find(|c| *c != Code::DependsOnGap))
                .unwrap_or(Code::DependsOnGap);
            if let Some((name, arity, _)) = define_name {
                self.defined.remove(&name);
                self.funcs.remove(&name);
                self.arrays.remove(&name);
                let _ = arity;
                self.gap_names.insert(name, root);
            }
            pictured = self.show_gap(r, &gap_codes, attempt.as_deref(), m.saved_result.as_ref());
        }
        self.flush_issues(&r.id, r.line, None);
        Self::bump_p(&mut self.report.kinds, "math", converted, has_notes, pictured);
        Self::bump_p(&mut self.report.math_shapes, shape, converted, has_notes, pictured);
    }
}

/// One output line for a paragraph, or `None` if it holds no text.
fn text_line(p: &Paragraph) -> Option<String> {
    let heading = match p.style.as_str() {
        "Heading 1" => Some(1),
        "Heading 2" => Some(2),
        "Heading 3" => Some(3),
        _ => None,
    };
    // Merge runs of equal formatting (whitespace-only runs take the formatting of their
    // neighbour), then collapse repeated spaces across run boundaries.
    enum Seg {
        Text(Run),
        Br,
    }
    let mut segs: Vec<Seg> = Vec::new();
    for piece in &p.pieces {
        match piece {
            Piece::Break => segs.push(Seg::Br),
            Piece::Run(r) => {
                if r.text.is_empty() {
                    continue;
                }
                let mut r = r.clone();
                if r.text.trim().is_empty() {
                    if let Some(Seg::Text(prev)) = segs.last() {
                        r = Run {
                            text: r.text,
                            ..prev.clone()
                        };
                    } else {
                        r = Run {
                            text: r.text,
                            ..Run::default()
                        };
                    }
                }
                match segs.last_mut() {
                    Some(Seg::Text(prev))
                        if (prev.bold, prev.italic, prev.underline, prev.sub, prev.sup)
                            == (r.bold, r.italic, r.underline, r.sub, r.sup) =>
                    {
                        prev.text.push_str(&r.text);
                    }
                    _ => segs.push(Seg::Text(r)),
                }
            }
        }
    }
    let mut prev_space = true;
    let mut any = false;
    for seg in segs.iter_mut() {
        match seg {
            Seg::Br => prev_space = true,
            Seg::Text(r) => {
                let mut o = String::with_capacity(r.text.len());
                for c in r.text.chars() {
                    if c == ' ' {
                        if !prev_space {
                            o.push(' ');
                        }
                        prev_space = true;
                    } else {
                        o.push(c);
                        prev_space = false;
                        any = true;
                    }
                }
                r.text = o;
            }
        }
    }
    if !any {
        return None;
    }
    // Trim a trailing space before each break and at the end.
    let n = segs.len();
    for i in 0..n {
        let at_end = i + 1 == n || matches!(segs[i + 1], Seg::Br);
        if at_end {
            if let Seg::Text(r) = &mut segs[i] {
                while r.text.ends_with(' ') {
                    r.text.pop();
                }
            }
        }
    }
    let mut html = String::new();
    for seg in &segs {
        match seg {
            Seg::Br => html.push_str("<br/>"),
            Seg::Text(r) => {
                if r.text.is_empty() {
                    continue;
                }
                let mut s = escape_text(&r.text);
                if heading.is_none() {
                    if r.bold {
                        s = format!("<b>{}</b>", s);
                    }
                    if r.italic {
                        s = format!("<i>{}</i>", s);
                    }
                    if r.underline {
                        s = format!("<u>{}</u>", s);
                    }
                }
                if r.sub {
                    s = format!("<sub>{}</sub>", s);
                }
                if r.sup {
                    s = format!("<sup>{}</sup>", s);
                }
                html.push_str(&s);
            }
        }
    }
    Some(match heading {
        Some(n) => format!("'<h{n}>{html}</h{n}>"),
        None => {
            if p.style == "List" {
                format!("'&bull; {}", html)
            } else {
                format!("'{}", html)
            }
        }
    })
}

/// Translates a sheet. See the crate documentation.
pub fn translate(mut sheet: Sheet, opts: &ImportOptions) -> (String, ImportReport, Vec<Asset>) {
    reading_order(&mut sheet.regions);
    let has_references = sheet.regions.iter().any(|r| matches!(r.kind, RegionKind::Reference));
    let mut tr = Tr {
        origin: sheet.origin,
        defined: HashSet::new(),
        funcs: HashMap::new(),
        gap_names: HashMap::new(),
        arrays: HashSet::new(),
        params: Vec::new(),
        cur: Vec::new(),
        uses_trig: false,
        lines: Vec::new(),
        report: ImportReport {
            origin: sheet.origin,
            ..Default::default()
        },
        depth: 0,
        opts: opts.clone(),
        assets: Vec::new(),
        used_paths: HashSet::new(),
        asset_total: 0,
        has_references,
    };
    for r in &sheet.regions {
        tr.region(r);
    }
    let mut out = String::new();
    out.push_str(&format!(
        "'<!-- Imported from a foreign worksheet: {} of {} regions need attention. Lines that start with {} mark what did not convert. -->\n",
        tr.report.regions_needing_attention(),
        tr.report.regions_total(),
        GAP_MARKER
    ));
    if tr.uses_trig {
        out.push_str("#rad\n");
        tr.report.issues.insert(
            0,
            Issue {
                code: Code::AngleMode,
                severity: Severity::Note,
                detail: None,
                region_id: None,
                line: None,
                reason: Code::AngleMode.reason().to_string(),
                root: None,
            },
        );
    }
    for l in &tr.lines {
        out.push_str(l);
        out.push('\n');
    }
    (out, tr.report, tr.assets)
}
