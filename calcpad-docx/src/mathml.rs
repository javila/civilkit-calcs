//! MathML in a worksheet page (the typeset equations of an imported worksheet) for the Word and
//! Excel exports. Each `<math>` element becomes an equation span of the kind the engine writes:
//! its `data-xml` holds Office Math (so Word shows a real equation: fractions, subscripts, sums,
//! brackets) and its text is a plain linear form with the cells of a matrix apart (so Excel shows
//! readable text). Anything this module does not know is kept as its text.

#[derive(Debug)]
enum Node {
    El {
        name: String,
        attrs: String,
        kids: Vec<Node>,
    },
    Text(String),
}

fn decode(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#36;", "$")
        .replace("&#123;", "{")
        .replace("&#125;", "}")
        .replace("&#35;", "#")
        .replace("&amp;", "&")
}

/// Parses well-formed MathML (the page writes it that way) into a tree. `None` when it is not.
fn parse(s: &str) -> Option<Node> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut stack: Vec<(String, String, Vec<Node>)> = vec![("#root".into(), String::new(), Vec::new())];
    while i < b.len() {
        if b[i] == '<' {
            let end = (i..b.len()).find(|&k| b[k] == '>')?;
            let tag: String = b[i + 1..end].iter().collect();
            i = end + 1;
            if let Some(name) = tag.strip_prefix('/') {
                let (n, a, kids) = stack.pop()?;
                if n != name.trim() || stack.is_empty() {
                    return None;
                }
                stack.last_mut()?.2.push(Node::El {
                    name: n,
                    attrs: a,
                    kids,
                });
            } else {
                let selfclose = tag.ends_with('/');
                let tag = tag.trim_end_matches('/');
                let (name, attrs) = match tag.split_once(char::is_whitespace) {
                    Some((n, a)) => (n.to_string(), a.to_string()),
                    None => (tag.to_string(), String::new()),
                };
                if selfclose {
                    stack.last_mut()?.2.push(Node::El {
                        name,
                        attrs,
                        kids: Vec::new(),
                    });
                } else {
                    stack.push((name, attrs, Vec::new()));
                }
            }
        } else {
            let end = (i..b.len()).find(|&k| b[k] == '<').unwrap_or(b.len());
            let t: String = b[i..end].iter().collect();
            if !t.trim().is_empty() {
                stack.last_mut()?.2.push(Node::Text(decode(t.trim())));
            }
            i = end;
        }
    }
    if stack.len() != 1 {
        return None;
    }
    let (_, _, mut kids) = stack.pop()?;
    if kids.len() == 1 {
        kids.pop()
    } else {
        None
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn kids(n: &Node) -> &[Node] {
    match n {
        Node::El { kids, .. } => kids,
        Node::Text(_) => &[],
    }
}

fn name(n: &Node) -> &str {
    match n {
        Node::El { name, .. } => name,
        Node::Text(_) => "",
    }
}

fn text(n: &Node) -> String {
    match n {
        Node::Text(t) => t.clone(),
        Node::El { kids, .. } => kids.iter().map(text).collect(),
    }
}

fn is_fence(n: &Node, set: &str) -> bool {
    name(n) == "mo" && text(n).chars().count() == 1 && set.contains(&text(n))
}

fn run(t: &str, upright: bool) -> String {
    let sty = if upright {
        "<m:rPr><m:sty m:val=\"p\"/></m:rPr>"
    } else {
        ""
    };
    format!("<m:r>{sty}<m:t xml:space=\"preserve\">{}</m:t></m:r>", esc(t))
}

fn is_large(n: &Node) -> Option<String> {
    match n {
        Node::El { name, kids, .. } if matches!(name.as_str(), "munderover" | "munder" | "msubsup") => {
            let t = text(kids.first()?);
            matches!(t.as_str(), "\u{2211}" | "\u{220F}" | "\u{222B}").then_some(t)
        }
        _ => None,
    }
}

/// A sequence of siblings as Office Math; a large operator takes the sibling after it as its body.
fn seq(list: &[Node]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < list.len() {
        if let Some(op) = is_large(&list[i]) {
            let k = kids(&list[i]);
            let (sub, sup) = match name(&list[i]) {
                "munderover" | "msubsup" => (k.get(1), k.get(2)),
                _ => (k.get(1), None),
            };
            let loc = if op == "\u{222B}" { "subSup" } else { "undOvr" };
            let body = list.get(i + 1).map(omml).unwrap_or_default();
            out.push_str(&format!(
                "<m:nary><m:naryPr><m:chr m:val=\"{op}\"/><m:limLoc m:val=\"{loc}\"/>{}{}</m:naryPr><m:sub>{}</m:sub><m:sup>{}</m:sup><m:e>{body}</m:e></m:nary>",
                if sub.is_none() { "<m:subHide m:val=\"1\"/>" } else { "" },
                if sup.is_none() { "<m:supHide m:val=\"1\"/>" } else { "" },
                sub.map(omml).unwrap_or_default(),
                sup.map(omml).unwrap_or_default(),
            ));
            i += 2;
        } else {
            out.push_str(&omml(&list[i]));
            i += 1;
        }
    }
    out
}

fn omml(n: &Node) -> String {
    let Node::El {
        name: tag,
        attrs,
        kids: k,
        ..
    } = n
    else {
        return run(
            &match n {
                Node::Text(t) => t.clone(),
                _ => String::new(),
            },
            false,
        );
    };
    let e = |x: Option<&Node>| x.map(omml).unwrap_or_default();
    match tag.as_str() {
        "mn" => run(&text(n), true),
        "mo" => run(&text(n), true),
        "mtext" | "ms" => run(&text(n), true),
        "mi" => {
            let t = text(n);
            run(&t, attrs.contains("mathvariant=\"normal\""))
        }
        "mspace" => run(" ", true),
        "mrow" | "mpadded" | "math" => {
            // A bracketed row is a delimiter object.
            if k.len() >= 2 && is_fence(&k[0], "([|") && is_fence(&k[k.len() - 1], ")]|") {
                let (b, c) = (text(&k[0]), text(&k[k.len() - 1]));
                return format!(
                    "<m:d><m:dPr><m:begChr m:val=\"{b}\"/><m:endChr m:val=\"{c}\"/></m:dPr><m:e>{}</m:e></m:d>",
                    seq(&k[1..k.len() - 1])
                );
            }
            seq(k)
        }
        "mfrac" => format!(
            "<m:f><m:num>{}</m:num><m:den>{}</m:den></m:f>",
            e(k.first()),
            e(k.get(1))
        ),
        "msub" => format!(
            "<m:sSub><m:e>{}</m:e><m:sub>{}</m:sub></m:sSub>",
            e(k.first()),
            e(k.get(1))
        ),
        "msup" => format!(
            "<m:sSup><m:e>{}</m:e><m:sup>{}</m:sup></m:sSup>",
            e(k.first()),
            e(k.get(1))
        ),
        "msubsup" => format!(
            "<m:sSubSup><m:e>{}</m:e><m:sub>{}</m:sub><m:sup>{}</m:sup></m:sSubSup>",
            e(k.first()),
            e(k.get(1)),
            e(k.get(2))
        ),
        "munderover" => format!(
            "<m:sSubSup><m:e>{}</m:e><m:sub>{}</m:sub><m:sup>{}</m:sup></m:sSubSup>",
            e(k.first()),
            e(k.get(1)),
            e(k.get(2))
        ),
        "munder" => format!(
            "<m:limLow><m:e>{}</m:e><m:lim>{}</m:lim></m:limLow>",
            e(k.first()),
            e(k.get(1))
        ),
        "mover" => {
            let chr = k.get(1).map(text).unwrap_or_default();
            format!(
                "<m:acc><m:accPr><m:chr m:val=\"{chr}\"/></m:accPr><m:e>{}</m:e></m:acc>",
                e(k.first())
            )
        }
        "msqrt" => format!(
            "<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e>{}</m:e></m:rad>",
            seq(k)
        ),
        "mtable" => {
            let mut rows = String::new();
            for tr in k {
                rows.push_str("<m:mr>");
                for td in kids(tr) {
                    rows.push_str(&format!("<m:e>{}</m:e>", seq(kids(td))));
                }
                rows.push_str("</m:mr>");
            }
            format!("<m:m>{rows}</m:m>")
        }
        _ => run(&text(n), false),
    }
}

/// A plain linear form: cells of a matrix apart, fractions and powers marked.
fn linear(n: &Node) -> String {
    let k = kids(n);
    let l = |x: Option<&Node>| x.map(linear).unwrap_or_default();
    let wrap = |s: String| {
        if s.chars().count() > 1 && !s.starts_with('(') {
            format!("({s})")
        } else {
            s
        }
    };
    match name(n) {
        "mfrac" => format!("{}/{}", wrap(l(k.first())), wrap(l(k.get(1)))),
        "msub" => format!("{}_{}", l(k.first()), wrap(l(k.get(1)))),
        "msup" => format!("{}^{}", l(k.first()), wrap(l(k.get(1)))),
        "msubsup" | "munderover" => format!("{} {}..{}", l(k.first()), l(k.get(1)), l(k.get(2))),
        "munder" => format!("{} {}", l(k.first()), l(k.get(1))),
        "mover" => format!("{} {}", l(k.first()), l(k.get(1))),
        "msqrt" => format!("\u{221A}({})", k.iter().map(linear).collect::<String>()),
        "mspace" => " ".to_string(),
        "mtable" => k
            .iter()
            .map(|tr| {
                kids(tr)
                    .iter()
                    .map(|td| kids(td).iter().map(linear).collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("; "),
        "mo" => {
            let t = text(n);
            if matches!(
                t.as_str(),
                ":=" | "=" | "\u{2261}" | "<" | ">" | "\u{2264}" | "\u{2265}" | "\u{2260}"
            ) {
                format!(" {t} ")
            } else {
                t
            }
        }
        "" => text(n),
        _ => k.iter().map(linear).collect(),
    }
}

/// One `<math>...</math>` element as an equation span, or `None` when it cannot be read.
fn span(math: &str) -> Option<String> {
    let tree = parse(math)?;
    let xml = omml(&tree);
    let lin = linear(&tree);
    Some(format!(
        "<span class=\"eq\" data-xml=\"{}\">{}</span>",
        xml.replace('"', "&quot;"),
        esc(&lin.split_whitespace().collect::<Vec<_>>().join(" "))
    ))
}

/// The page with every `<math>` element replaced by an equation span. A page with none is returned as it is.
pub fn convert_page(html: &str) -> String {
    if !html.contains("<math") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(a) = rest.find("<math") {
        let after = &rest[a..];
        let Some(b) = after.find("</math>") else { break };
        out.push_str(&rest[..a]);
        let el = &after[..b + "</math>".len()];
        match span(el) {
            Some(s) => out.push_str(&s),
            None => out.push_str(&esc(&text_of(el))),
        }
        rest = &after[b + "</math>".len()..];
    }
    out.push_str(rest);
    out
}

fn text_of(el: &str) -> String {
    let mut o = String::new();
    let mut tag = false;
    for c in el.chars() {
        match c {
            '<' => tag = true,
            '>' => {
                tag = false;
                o.push(' ');
            }
            c if !tag => o.push(c),
            _ => {}
        }
    }
    o.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fraction_and_a_sum_become_office_math() {
        let h = "<p><math class=\"imp-m\"><mi>C</mi><mo>:=</mo><mfrac><mrow><mi>a</mi></mrow><mrow><munderover><mo>\u{2211}</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover><mrow><mo>(</mo><mi>b</mi><mo>)</mo></mrow></mrow></mfrac></math></p>";
        let out = convert_page(h);
        assert!(out.contains("<m:f><m:num>"), "{out}");
        assert!(out.contains("<m:nary>") && out.contains("<m:d>"), "{out}");
        assert!(!out.contains("<math"), "{out}");
    }

    #[test]
    fn matrix_cells_stay_apart_in_the_text_form() {
        let h = "<math><mo>[</mo><mtable><mtr><mtd><mn>1</mn></mtd><mtd><mn>2</mn></mtd></mtr></mtable><mo>]</mo><mspace width=\"0.25em\"></mspace><mi mathvariant=\"normal\">kip</mi></math>";
        let out = convert_page(h);
        assert!(out.contains(">[1 2] kip</span>"), "{out}");
        assert!(out.contains("<m:m><m:mr><m:e>"), "{out}");
    }

    #[test]
    fn a_page_without_math_is_untouched() {
        assert_eq!(convert_page("<p>x</p>"), "<p>x</p>");
    }
}
