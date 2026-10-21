//! A small, forgiving HTML parser: the part of HtmlAgilityPack that Calcpad.OpenXml relies on.
//! Element and attribute names are lower-cased, void elements take no children, a closing tag
//! closes the nearest open element with its name (and is ignored when none is open), and the
//! contents of `script` and `style` are kept as raw text. Text and attribute values are kept as
//! written; text is decoded by the writer (`HttpUtility.HtmlDecode`), as upstream does.

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element {
    /// The tag name in lower case; `raw_name` keeps it as written (SVG is case-sensitive).
    pub name: String,
    pub raw_name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

const VOID: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr",
];

impl Element {
    pub fn new(name: &str) -> Element {
        Element {
            name: name.to_ascii_lowercase(),
            raw_name: name.to_string(),
            ..Default::default()
        }
    }

    /// `GetAttributeValue(name, null)`
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// `GetAttributeValue(name, int default)`
    pub fn attr_int(&self, name: &str, default: i64) -> i64 {
        self.attr(name).and_then(|v| v.trim().parse().ok()).unwrap_or(default)
    }

    /// `HasClass`
    pub fn has_class(&self, class: &str) -> bool {
        self.attr("class")
            .is_some_and(|c| c.split_ascii_whitespace().any(|x| x == class))
    }

    /// The first element named `name` in document order (`SelectSingleNode("//name")`).
    pub fn find(&self, name: &str) -> Option<&Element> {
        for c in &self.children {
            if let Node::Element(e) = c {
                if e.name == name {
                    return Some(e);
                }
                if let Some(f) = e.find(name) {
                    return Some(f);
                }
            }
        }
        None
    }

    /// The element whose `id` is `id` (`GetElementbyId`).
    pub fn by_id(&self, id: &str) -> Option<&Element> {
        for c in &self.children {
            if let Node::Element(e) = c {
                if e.attr("id") == Some(id) {
                    return Some(e);
                }
                if let Some(f) = e.by_id(id) {
                    return Some(f);
                }
            }
        }
        None
    }

    /// `OuterHtml`
    pub fn outer_html(&self) -> String {
        let mut s = String::new();
        write_element(self, &mut s);
        s
    }
}

fn write_element(e: &Element, s: &mut String) {
    s.push('<');
    s.push_str(&e.raw_name);
    for (k, v) in &e.attrs {
        s.push(' ');
        s.push_str(k);
        s.push_str("=\"");
        s.push_str(&v.replace('"', "&quot;"));
        s.push('"');
    }
    if VOID.contains(&e.name.as_str()) {
        s.push('>');
        return;
    }
    s.push('>');
    for c in &e.children {
        match c {
            Node::Element(x) => write_element(x, s),
            Node::Text(t) => s.push_str(t),
            Node::Comment(t) => {
                s.push_str("<!--");
                s.push_str(t);
                s.push_str("-->");
            }
        }
    }
    s.push_str("</");
    s.push_str(&e.raw_name);
    s.push('>');
}

/// Parses a document into a root element named `#document`.
pub fn parse(html: &str) -> Element {
    let mut stack: Vec<Element> = vec![Element::new("#document")];
    let bytes = html.as_bytes();
    let mut i = 0;
    let mut text_start = 0;
    let flush_text = |stack: &mut Vec<Element>, from: usize, to: usize| {
        if to > from {
            stack
                .last_mut()
                .unwrap()
                .children
                .push(Node::Text(html[from..to].to_string()));
        }
    };
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // A comment, doctype or processing instruction.
        if html[i..].starts_with("<!--") {
            flush_text(&mut stack, text_start, i);
            let end = html[i + 4..].find("-->").map(|e| i + 4 + e).unwrap_or(html.len());
            stack
                .last_mut()
                .unwrap()
                .children
                .push(Node::Comment(html[i + 4..end].to_string()));
            i = (end + 3).min(html.len());
            text_start = i;
            continue;
        }
        if html[i..].starts_with("<!") || html[i..].starts_with("<?") {
            flush_text(&mut stack, text_start, i);
            i = html[i..].find('>').map(|e| i + e + 1).unwrap_or(html.len());
            text_start = i;
            continue;
        }
        let closing = bytes.get(i + 1) == Some(&b'/');
        let name_start = if closing { i + 2 } else { i + 1 };
        let name_len = html[name_start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == ':' || c == '_'))
            .unwrap_or(html.len() - name_start);
        if name_len == 0 || !bytes[name_start].is_ascii_alphabetic() {
            // A lone "<" is text.
            i += 1;
            continue;
        }
        flush_text(&mut stack, text_start, i);
        let raw_name = html[name_start..name_start + name_len].to_string();
        let name = raw_name.to_ascii_lowercase();
        if closing {
            let end = html[i..].find('>').map(|e| i + e + 1).unwrap_or(html.len());
            if let Some(pos) = stack.iter().rposition(|e| e.name == name) {
                if pos > 0 {
                    while stack.len() > pos {
                        let e = stack.pop().unwrap();
                        stack.last_mut().unwrap().children.push(Node::Element(e));
                    }
                }
            }
            i = end;
            text_start = i;
            continue;
        }
        // Attributes.
        let mut j = name_start + name_len;
        let mut attrs = Vec::new();
        let mut self_closing = false;
        loop {
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
            if bytes[j] == b'>' {
                j += 1;
                break;
            }
            if bytes[j] == b'/' {
                self_closing = true;
                j += 1;
                continue;
            }
            let a0 = j;
            while j < bytes.len() && !bytes[j].is_ascii_whitespace() && !matches!(bytes[j], b'=' | b'>' | b'/') {
                j += 1;
            }
            if j == a0 {
                j += 1;
                continue;
            }
            let key = html[a0..j].to_string();
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            let mut value = String::new();
            if k < bytes.len() && bytes[k] == b'=' {
                k += 1;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k < bytes.len() && (bytes[k] == b'"' || bytes[k] == b'\'') {
                    let q = bytes[k] as char;
                    let end = html[k + 1..].find(q).map(|e| k + 1 + e).unwrap_or(html.len());
                    value = html[k + 1..end].to_string();
                    j = (end + 1).min(html.len());
                } else {
                    let v0 = k;
                    while k < bytes.len() && !bytes[k].is_ascii_whitespace() && bytes[k] != b'>' {
                        k += 1;
                    }
                    value = html[v0..k].to_string();
                    j = k;
                }
                self_closing = false;
            }
            if !attrs
                .iter()
                .any(|(a, _): &(String, String)| a.eq_ignore_ascii_case(&key))
            {
                attrs.push((key, value));
            }
        }
        i = j;
        // As browsers do: a select cannot hold another; the new one closes the open one.
        if name == "select" {
            if let Some(pos) = stack.iter().rposition(|e| e.name == "select") {
                while stack.len() > pos {
                    let e = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(Node::Element(e));
                }
            }
        }
        let mut e = Element {
            name: name.clone(),
            raw_name,
            attrs,
            children: Vec::new(),
        };
        if name == "script" || name == "style" {
            let close = format!("</{}", name);
            let lower = html[i..].to_ascii_lowercase();
            let end = lower.find(&close).map(|e| i + e).unwrap_or(html.len());
            if end > i {
                e.children.push(Node::Text(html[i..end].to_string()));
            }
            i = html[end..].find('>').map(|x| end + x + 1).unwrap_or(html.len());
            stack.last_mut().unwrap().children.push(Node::Element(e));
        } else if self_closing || VOID.contains(&name.as_str()) {
            stack.last_mut().unwrap().children.push(Node::Element(e));
        } else {
            stack.push(e);
        }
        text_start = i;
    }
    flush_text(&mut stack, text_start, html.len());
    while stack.len() > 1 {
        let e = stack.pop().unwrap();
        stack.last_mut().unwrap().children.push(Node::Element(e));
    }
    stack.pop().unwrap()
}

/// `HttpUtility.HtmlDecode`: named (the HTML 4 set) and numeric character references.
pub fn decode(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let end = rest[1..].find(';').map(|e| e + 1);
        let decoded = end.filter(|&e| e <= 10).and_then(|e| {
            let name = &rest[1..e];
            let c = if let Some(num) = name.strip_prefix('#') {
                let v = if let Some(h) = num.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(h, 16).ok()
                } else {
                    num.parse::<u32>().ok()
                };
                v.and_then(char::from_u32)
            } else {
                entity(name)
            };
            c.map(|c| (c, e))
        });
        match decoded {
            Some((c, e)) => {
                out.push(c);
                rest = &rest[e + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{A0}',
        "ensp" => '\u{2002}',
        "emsp" => '\u{2003}',
        "thinsp" => '\u{2009}',
        "hairsp" => '\u{200A}',
        "zwsp" => '\u{200B}',
        "zwnj" => '\u{200C}',
        "zwj" => '\u{200D}',
        "shy" => '\u{AD}',
        "le" => '\u{2264}',
        "ge" => '\u{2265}',
        "ne" => '\u{2260}',
        "equiv" => '\u{2261}',
        "minus" => '\u{2212}',
        "times" => '\u{D7}',
        "divide" => '\u{F7}',
        "middot" => '\u{B7}',
        "sdot" => '\u{22C5}',
        "plusmn" => '\u{B1}',
        "deg" => '\u{B0}',
        "micro" => '\u{B5}',
        "para" => '\u{B6}',
        "sect" => '\u{A7}',
        "copy" => '\u{A9}',
        "reg" => '\u{AE}',
        "trade" => '\u{2122}',
        "hellip" => '\u{2026}',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "ldquo" => '\u{201C}',
        "rdquo" => '\u{201D}',
        "bull" => '\u{2022}',
        "larr" => '\u{2190}',
        "rarr" => '\u{2192}',
        "uarr" => '\u{2191}',
        "darr" => '\u{2193}',
        "harr" => '\u{2194}',
        "infin" => '\u{221E}',
        "radic" => '\u{221A}',
        "sum" => '\u{2211}',
        "prod" => '\u{220F}',
        "int" => '\u{222B}',
        "part" => '\u{2202}',
        "nabla" => '\u{2207}',
        "isin" => '\u{2208}',
        "asymp" => '\u{2248}',
        "ang" => '\u{2220}',
        "and" => '\u{2227}',
        "or" => '\u{2228}',
        "oplus" => '\u{2295}',
        "otimes" => '\u{2297}',
        "perp" => '\u{22A5}',
        "prime" => '\u{2032}',
        "Prime" => '\u{2033}',
        "frac12" => '\u{BD}',
        "frac14" => '\u{BC}',
        "frac34" => '\u{BE}',
        "sup1" => '\u{B9}',
        "sup2" => '\u{B2}',
        "sup3" => '\u{B3}',
        "alpha" => '\u{3B1}',
        "beta" => '\u{3B2}',
        "gamma" => '\u{3B3}',
        "delta" => '\u{3B4}',
        "epsilon" => '\u{3B5}',
        "zeta" => '\u{3B6}',
        "eta" => '\u{3B7}',
        "theta" => '\u{3B8}',
        "iota" => '\u{3B9}',
        "kappa" => '\u{3BA}',
        "lambda" => '\u{3BB}',
        "mu" => '\u{3BC}',
        "nu" => '\u{3BD}',
        "xi" => '\u{3BE}',
        "omicron" => '\u{3BF}',
        "pi" => '\u{3C0}',
        "rho" => '\u{3C1}',
        "sigma" => '\u{3C3}',
        "tau" => '\u{3C4}',
        "upsilon" => '\u{3C5}',
        "phi" => '\u{3C6}',
        "chi" => '\u{3C7}',
        "psi" => '\u{3C8}',
        "omega" => '\u{3C9}',
        "Gamma" => '\u{393}',
        "Delta" => '\u{394}',
        "Theta" => '\u{398}',
        "Lambda" => '\u{39B}',
        "Xi" => '\u{39E}',
        "Pi" => '\u{3A0}',
        "Sigma" => '\u{3A3}',
        "Phi" => '\u{3A6}',
        "Psi" => '\u{3A8}',
        "Omega" => '\u{3A9}',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_tree_with_void_and_unclosed_elements() {
        let d =
            parse("<html><body><p class=\"a b\">x<br>y<img src='i.png' alt=z></p><p>open<div>in</div></body></html>");
        let body = d.find("body").unwrap();
        assert_eq!(body.children.len(), 2);
        let Node::Element(p) = &body.children[0] else { panic!() };
        assert!(p.has_class("b"));
        assert_eq!(p.children.len(), 4);
        let Node::Element(img) = &p.children[3] else { panic!() };
        assert_eq!((img.attr("src"), img.attr("alt")), (Some("i.png"), Some("z")));
    }

    #[test]
    fn keeps_script_text_raw_and_skips_stray_closers() {
        let d = parse("<body><script>if (a<b) x();</script></span><p>t</p></body>");
        let body = d.find("body").unwrap();
        let Node::Element(s) = &body.children[0] else { panic!() };
        assert_eq!(s.children, vec![Node::Text("if (a<b) x();".into())]);
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn a_select_closes_an_open_select() {
        let d = parse("<body><select a><p>x<select b><option>o</option></select><p>after</p></body>");
        let body = d.find("body").unwrap();
        let names: Vec<&str> = body
            .children
            .iter()
            .filter_map(|c| match c {
                Node::Element(e) => Some(e.name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(names, ["select", "select", "p"]);
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(
            decode("a&nbsp;&lt;b&#x3B1;&#946;&unknown; & c"),
            "a\u{A0}<b\u{3B1}\u{3B2}&unknown; & c"
        );
    }
}
