//! The `.xmcd` front door: plain XML in, a [`Sheet`] out.
//!
//! Namespaces are matched by their last path segment (`worksheet30`, `math30`, `units10`), not by
//! host, so the reader does not depend on who publishes the schema.
//!
//! The XML is untrusted: a DTD (and so every entity definition) is refused, the input size and the
//! node count are capped, and every recursion below has a depth limit.

use roxmltree::{Document, Node, ParsingOptions};

use crate::ir::*;
use crate::report::Code;

/// Limits applied while reading. The defaults suit worksheets of a few megabytes.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Largest input accepted, in bytes.
    pub max_bytes: usize,
    /// Most XML nodes accepted.
    pub max_nodes: u32,
    /// Deepest expression or region nesting followed; deeper parts become `too-deep` gaps.
    pub max_depth: usize,
    /// Deepest XML element nesting accepted at all (checked before the parser runs).
    pub max_xml_depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_bytes: 64 * 1024 * 1024,
            max_nodes: 4_000_000,
            max_depth: 64,
            max_xml_depth: 256,
        }
    }
}

/// Why a file could not be read at all.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadError {
    TooLarge,
    Xml(String),
    NotAWorksheet,
}

/// A cheap scan for runaway element nesting, done before the real parser sees the input. It
/// understands comments, CDATA, processing instructions and self-closing tags; malformed input is
/// left for the parser to reject.
fn xml_depth_exceeds(xml: &str, max: usize) -> bool {
    let b = xml.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &b[i..];
        if rest.starts_with(b"<!--") {
            i += 4;
            match xml[i..].find("-->") {
                Some(p) => i += p + 3,
                None => return false,
            }
        } else if rest.starts_with(b"<![CDATA[") {
            i += 9;
            match xml[i..].find("]]>") {
                Some(p) => i += p + 3,
                None => return false,
            }
        } else if rest.starts_with(b"<?") || rest.starts_with(b"<!") {
            match xml[i..].find('>') {
                Some(p) => i += p + 1,
                None => return false,
            }
        } else if rest.starts_with(b"</") {
            depth = depth.saturating_sub(1);
            match xml[i..].find('>') {
                Some(p) => i += p + 1,
                None => return false,
            }
        } else {
            // A start tag; find its end, honouring quoted attribute values.
            let mut j = i + 1;
            let mut quote = 0u8;
            while j < b.len() {
                let c = b[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    break;
                }
                j += 1;
            }
            if j >= b.len() {
                return false;
            }
            if b[j - 1] != b'/' {
                depth += 1;
                if depth > max {
                    return true;
                }
            }
            i = j + 1;
        }
    }
    false
}

fn ns_tail<'a>(n: Node<'a, '_>) -> &'a str {
    n.tag_name()
        .namespace()
        .map(|u| u.trim_end_matches('/').rsplit('/').next().unwrap_or(""))
        .unwrap_or("")
}

fn is_ws(n: Node, local: &str) -> bool {
    n.is_element() && n.tag_name().name() == local && ns_tail(n) == "worksheet30"
}

fn is_ml(n: Node) -> bool {
    n.is_element() && ns_tail(n) == "math30"
}

fn ml_kids<'a, 'i>(n: Node<'a, 'i>) -> Vec<Node<'a, 'i>> {
    n.children().filter(|c| is_ml(*c)).collect()
}

fn local<'a>(n: Node<'a, '_>) -> &'a str {
    n.tag_name().name()
}

pub fn read(xml: &str, limits: Limits) -> Result<Sheet, ReadError> {
    if xml.len() > limits.max_bytes {
        return Err(ReadError::TooLarge);
    }
    if xml_depth_exceeds(xml, limits.max_xml_depth) {
        return Err(ReadError::Xml("elements nested too deeply".into()));
    }
    let opts = ParsingOptions {
        allow_dtd: false,
        nodes_limit: limits.max_nodes,
    };
    let doc = Document::parse_with_options(xml, opts).map_err(|e| ReadError::Xml(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "worksheet" {
        return Err(ReadError::NotAWorksheet);
    }
    let mut origin = 0i64;
    for n in root.descendants() {
        if n.is_element() && n.tag_name().name() == "builtInVariables" {
            if let Some(v) = n.attribute("array-origin").and_then(|v| v.trim().parse::<i64>().ok()) {
                origin = v;
            }
            break;
        }
    }
    let mut regions = Vec::new();
    if let Some(container) = root.children().find(|c| is_ws(*c, "regions")) {
        for r in container.children().filter(|c| is_ws(*c, "region")) {
            collect_region(&doc, r, &limits, 0, &mut regions);
        }
    }
    Ok(Sheet { origin, regions })
}

fn num_attr(n: Node, name: &str) -> Option<f64> {
    n.attribute(name)
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
}

fn collect_region(doc: &Document, r: Node, lim: &Limits, depth: usize, out: &mut Vec<Region>) {
    let id = r.attribute("region-id").unwrap_or("?").to_string();
    let line = doc.text_pos_at(r.range().start).row;
    let left = num_attr(r, "left").unwrap_or(0.0);
    let top = num_attr(r, "top").unwrap_or(0.0);
    let baseline = num_attr(r, "align-y");
    // The content element is the first child that is not a link or a cached rendering.
    let content = r
        .children()
        .find(|c| c.is_element() && ns_tail(*c) == "worksheet30" && !matches!(local(*c), "link" | "rendering"));
    let idx = out.len();
    let kind = match content {
        None => RegionKind::Other("empty".into()),
        Some(c) => match local(c) {
            "math" => RegionKind::Math(read_math(c, lim)),
            "text" => {
                let mut paras = Vec::new();
                let mut nested = Vec::new();
                for p in c.children().filter(|x| is_ws(*x, "p")) {
                    let mut pieces = Vec::new();
                    walk_inline(p, &Run::default(), &mut pieces, &mut nested, depth, lim);
                    paras.push(Paragraph {
                        style: p.attribute("style").unwrap_or("Normal").to_string(),
                        pieces,
                    });
                }
                // Reserve the slot so that the text region precedes the regions nested in it.
                out.push(Region {
                    id: id.clone(),
                    left,
                    top,
                    baseline,
                    line,
                    kind: RegionKind::Text(paras),
                });
                for n in nested {
                    if depth < lim.max_depth {
                        collect_region(doc, n, lim, depth + 1, out);
                    }
                }
                return;
            }
            "plot" => RegionKind::Plot,
            "picture" => RegionKind::Image,
            "ole" => RegionKind::Ole,
            "component" => RegionKind::Component,
            "reference" => RegionKind::Reference,
            "pageBreak" => RegionKind::PageBreak,
            other => RegionKind::Other(other.to_string()),
        },
    };
    debug_assert!(idx == out.len());
    out.push(Region {
        id,
        left,
        top,
        baseline,
        line,
        kind,
    });
}

/// Walks the children of a paragraph or a formatting element, collecting runs. Regions found
/// inside are pushed to `nested` and left out of the text.
fn walk_inline<'a, 'i>(
    n: Node<'a, 'i>,
    fmt: &Run,
    pieces: &mut Vec<Piece>,
    nested: &mut Vec<Node<'a, 'i>>,
    depth: usize,
    lim: &Limits,
) {
    if depth > lim.max_depth {
        return;
    }
    for c in n.children() {
        if c.is_text() {
            let t = c.text().unwrap_or("");
            if !t.is_empty() {
                push_text(pieces, t, fmt);
            }
        } else if c.is_element() && ns_tail(c) == "worksheet30" {
            match local(c) {
                "region" => nested.push(c),
                "sp" | "tab" => push_text(pieces, " ", fmt),
                "br" => pieces.push(Piece::Break),
                "b" => walk_inline(
                    c,
                    &Run {
                        bold: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "i" => walk_inline(
                    c,
                    &Run {
                        italic: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "u" => walk_inline(
                    c,
                    &Run {
                        underline: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "sub" => walk_inline(
                    c,
                    &Run {
                        sub: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "sup" => walk_inline(
                    c,
                    &Run {
                        sup: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                // f (font), c (colour), inlineAttr and unknown wrappers: keep their text.
                _ => walk_inline(c, fmt, pieces, nested, depth + 1, lim),
            }
        }
    }
}

fn push_text(pieces: &mut Vec<Piece>, t: &str, fmt: &Run) {
    // Collapse whitespace runs (pretty-printing, tabs, newlines) into single spaces.
    let mut s = String::with_capacity(t.len());
    let mut prev_space = false;
    for ch in t.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                s.push(' ');
            }
            prev_space = true;
        } else {
            s.push(ch);
            prev_space = false;
        }
    }
    pieces.push(Piece::Run(Run { text: s, ..fmt.clone() }));
}

// ---------------------------------------------------------------------------------------------
// Math

fn read_math(m: Node, lim: &Limits) -> MathRegion {
    let disabled = m.attribute("disable-calc") == Some("true");
    // The statement is the first math-namespace child (provenance wrappers are peeled off).
    let stmt_node = m.children().find(|c| is_ml(*c));
    let stmt = match stmt_node {
        None => Stmt::Unsupported {
            code: Code::BadStructure,
            detail: Some("empty math region".into()),
        },
        Some(n) => read_stmt(unwrap_provenance(n), lim),
    };
    MathRegion { stmt, disabled }
}

fn unwrap_provenance<'a, 'i>(mut n: Node<'a, 'i>) -> Node<'a, 'i> {
    let mut guard = 0;
    while is_ml(n) && local(n) == "provenance" && guard < 8 {
        match n.children().find(|c| is_ml(*c)) {
            Some(c) => n = c,
            None => break,
        }
        guard += 1;
    }
    n
}

fn unsupported_code(tag: &str) -> Option<Code> {
    Some(match tag {
        "program" | "ifThen" | "otherwise" | "for" | "while" | "break" | "continue" | "return" | "onError"
        | "localDefine" => Code::ProgramBlock,
        "symEval" | "symbolicEval" | "symbolic" | "keywordEval" => Code::SymbolicEvaluation,
        "derivative" | "integral" | "nthDerivative" | "partialDerivative" => Code::CalculusOperator,
        "sum" | "product" | "limit" | "iteratedSum" | "iteratedProduct" => Code::SummationOperator,
        "vectorize" => Code::VectorizeOperator,
        "matcol" | "matrow" | "vectorSum" | "magnitude" | "dot" | "cross" => Code::MatrixOperator,
        _ => return None,
    })
}

fn read_stmt(n: Node, lim: &Limits) -> Stmt {
    let tag = local(n);
    match tag {
        "define" | "globalDefine" => {
            let kids = ml_kids(n);
            if kids.len() != 2 {
                return bad("definition without a target and a value");
            }
            let target = match read_target(kids[0], lim) {
                Some(t) => t,
                None => return bad("definition target not understood"),
            };
            Stmt::Define {
                target,
                value: read_expr(unwrap_provenance(kids[1]), lim, 0),
                global: tag == "globalDefine",
            }
        }
        "eval" => {
            let kids = ml_kids(n);
            let mut expr = None;
            let mut unit = None;
            for k in kids {
                match local(k) {
                    "unitOverride" => {
                        if let Some(u) = ml_kids(k).into_iter().next() {
                            unit = Some(read_expr(unwrap_provenance(u), lim, 0));
                        }
                    }
                    "result" => {}
                    _ => {
                        if expr.is_none() {
                            expr = Some(read_expr(unwrap_provenance(k), lim, 0));
                        }
                    }
                }
            }
            match expr {
                Some(expr) => Stmt::Eval { expr, unit },
                None => bad("evaluation without an expression"),
            }
        }
        _ => {
            if let Some(code) = unsupported_code(tag) {
                return Stmt::Unsupported {
                    code,
                    detail: Some(tag.to_string()),
                };
            }
            Stmt::Bare(read_expr(n, lim, 0))
        }
    }
}

fn bad(why: &str) -> Stmt {
    Stmt::Unsupported {
        code: Code::BadStructure,
        detail: Some(why.to_string()),
    }
}

fn read_name(n: Node) -> Name {
    Name {
        base: n.text().unwrap_or("").trim().to_string(),
        sub: n
            .attribute("subscript")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    }
}

fn read_target(n: Node, lim: &Limits) -> Option<Target> {
    let n = unwrap_provenance(n);
    match local(n) {
        "id" => Some(Target::Var(read_name(n))),
        "function" => {
            let kids = ml_kids(n);
            let name = kids.iter().find(|k| local(**k) == "id")?;
            let params: Vec<Name> = kids
                .iter()
                .find(|k| local(**k) == "boundVars")
                .map(|b| {
                    ml_kids(*b)
                        .into_iter()
                        .filter(|k| local(*k) == "id")
                        .map(read_name)
                        .collect()
                })
                .unwrap_or_default();
            Some(Target::Func(read_name(*name), params))
        }
        "apply" => {
            let _ = lim;
            Some(Target::Element)
        }
        _ => None,
    }
}

fn unsupported(code: Code, detail: &str) -> Expr {
    Expr::Unsupported {
        code,
        detail: Some(detail.to_string()),
    }
}

fn read_expr(n: Node, lim: &Limits, depth: usize) -> Expr {
    if depth > lim.max_depth {
        return Expr::Unsupported {
            code: Code::TooDeep,
            detail: None,
        };
    }
    let n = unwrap_provenance(n);
    let tag = local(n);
    match tag {
        "id" => Expr::Name(read_name(n)),
        "real" => Expr::Num(n.text().unwrap_or("").trim().to_string()),
        "str" => Expr::Str(n.text().unwrap_or("").to_string()),
        "parens" => {
            let kids = ml_kids(n);
            if kids.len() == 1 {
                read_expr(kids[0], lim, depth + 1)
            } else {
                unsupported(Code::BadStructure, "parentheses with several children")
            }
        }
        "apply" => read_apply(n, lim, depth),
        // `x := expr =`: the definition also shows its value. Only the expression matters; a
        // display unit on the result is dropped.
        "eval" => {
            let inner = ml_kids(n)
                .into_iter()
                .find(|k| !matches!(local(*k), "unitOverride" | "result"));
            match inner {
                Some(k) => read_expr(k, lim, depth + 1),
                None => unsupported(Code::BadStructure, "empty evaluation"),
            }
        }
        "matrix" => {
            let rows = n.attribute("rows").and_then(|v| v.parse::<usize>().ok());
            let cols = n.attribute("cols").and_then(|v| v.parse::<usize>().ok());
            let items: Vec<Expr> = ml_kids(n).into_iter().map(|k| read_expr(k, lim, depth + 1)).collect();
            match (rows, cols) {
                (Some(r), Some(c)) if r >= 1 && c >= 1 && r.checked_mul(c) == Some(items.len()) => Expr::Matrix {
                    rows: r,
                    cols: c,
                    items,
                },
                _ => unsupported(Code::BadStructure, "matrix size does not match its items"),
            }
        }
        "range" => {
            let kids = ml_kids(n);
            if kids.len() != 2 {
                return unsupported(Code::BadStructure, "range");
            }
            let (first, second) = if local(kids[0]) == "sequence" {
                let s = ml_kids(kids[0]);
                match s.len() {
                    2 => (
                        read_expr(s[0], lim, depth + 1),
                        Some(Box::new(read_expr(s[1], lim, depth + 1))),
                    ),
                    _ => return unsupported(Code::BadStructure, "range"),
                }
            } else {
                (read_expr(kids[0], lim, depth + 1), None)
            };
            Expr::Range {
                first: Box::new(first),
                second,
                last: Box::new(read_expr(kids[1], lim, depth + 1)),
            }
        }
        other => match unsupported_code(other) {
            Some(code) => Expr::Unsupported {
                code,
                detail: Some(other.to_string()),
            },
            None => unsupported(Code::UnknownElement, other),
        },
    }
}

/// The operands of an `apply` after the operator; a single `sequence` operand is the argument list.
fn operands<'a, 'i>(kids: &[Node<'a, 'i>]) -> Vec<Node<'a, 'i>> {
    let rest = &kids[1..];
    if rest.len() == 1 && local(rest[0]) == "sequence" {
        ml_kids(rest[0])
    } else {
        rest.to_vec()
    }
}

fn read_apply(n: Node, lim: &Limits, depth: usize) -> Expr {
    let kids = ml_kids(n);
    let Some(op) = kids.first().copied() else {
        return unsupported(Code::BadStructure, "empty apply");
    };
    let op = unwrap_provenance(op);
    let tag = local(op);
    let args = operands(&kids);
    let rd = |x: &Node| read_expr(*x, lim, depth + 1);
    let bin = |o: BinOp, a: &[Node]| -> Expr {
        if a.len() < 2 {
            return unsupported(Code::BadStructure, "binary operator with fewer than two operands");
        }
        // An n-ary operator folds into a left-leaning chain; cap it so the chain stays shallow.
        if a.len() > 100 {
            return Expr::Unsupported {
                code: Code::TooDeep,
                detail: None,
            };
        }
        let mut it = a.iter().map(&rd);
        let mut acc = it.next().unwrap();
        for next in it {
            acc = Expr::Bin(o, Box::new(acc), Box::new(next));
        }
        acc
    };
    let un = |o: UnOp, a: &[Node]| -> Expr {
        if a.len() != 1 {
            return unsupported(Code::BadStructure, "unary operator without exactly one operand");
        }
        Expr::Un(o, Box::new(rd(&a[0])))
    };
    match tag {
        "id" => {
            let name = read_name(op);
            if n.attribute("fixity") == Some("postfix") {
                if args.len() == 1 {
                    return Expr::Postfix(name, Box::new(rd(&args[0])));
                }
                return unsupported(Code::BadStructure, "postfix operator");
            }
            Expr::Call(name, args.iter().map(&rd).collect())
        }
        "plus" => bin(BinOp::Add, &args),
        "minus" => {
            if args.len() == 1 {
                un(UnOp::Neg, &args)
            } else {
                bin(BinOp::Sub, &args)
            }
        }
        "neg" => un(UnOp::Neg, &args),
        "mult" => bin(BinOp::Mul, &args),
        "div" => bin(BinOp::Div, &args),
        "pow" => bin(BinOp::Pow, &args),
        "equal" => bin(BinOp::Eq, &args),
        "notEqual" => bin(BinOp::Ne, &args),
        "lessThan" => bin(BinOp::Lt, &args),
        "greaterThan" => bin(BinOp::Gt, &args),
        "lessOrEqual" => bin(BinOp::Le, &args),
        "greaterOrEqual" => bin(BinOp::Ge, &args),
        "and" => bin(BinOp::And, &args),
        "or" => bin(BinOp::Or, &args),
        "xor" => bin(BinOp::Xor, &args),
        "not" => un(UnOp::Not, &args),
        "absval" => un(UnOp::Abs, &args),
        "sqrt" => un(UnOp::Sqrt, &args),
        "factorial" => un(UnOp::Factorial, &args),
        "transpose" => un(UnOp::Transpose, &args),
        "indexer" => {
            if args.len() < 2 {
                return unsupported(Code::BadStructure, "indexer");
            }
            let base = rd(&args[0]);
            // Indexes are either several operands or one sequence.
            let idx: Vec<Expr> = if args.len() == 2 && local(args[1]) == "sequence" {
                ml_kids(args[1]).iter().map(&rd).collect()
            } else {
                args[1..].iter().map(&rd).collect()
            };
            Expr::Index(Box::new(base), idx)
        }
        other => match unsupported_code(other) {
            Some(code) => Expr::Unsupported {
                code,
                detail: Some(other.to_string()),
            },
            None => unsupported(Code::UnknownElement, other),
        },
    }
}
