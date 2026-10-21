//! A small XML tree for the OpenXml parts that `#read`/`#write` touch. Element names keep
//! their prefix; lookups compare local names.

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

pub fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

impl Element {
    pub fn new(name: &str) -> Element {
        Element {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name || local(k) == name && !name.contains(':'))
            .map(|(_, v)| v.as_str())
    }

    pub fn set_attr(&mut self, name: &str, value: &str) {
        match self.attrs.iter_mut().find(|(k, _)| k == name) {
            Some(a) => a.1 = value.to_string(),
            None => self.attrs.push((name.to_string(), value.to_string())),
        }
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            _ => None,
        })
    }

    pub fn child(&self, local_name: &str) -> Option<&Element> {
        self.elements().find(|e| local(&e.name) == local_name)
    }

    pub fn child_mut(&mut self, local_name: &str) -> Option<&mut Element> {
        self.children.iter_mut().find_map(|n| match n {
            Node::Element(e) if local(&e.name) == local_name => Some(e),
            _ => None,
        })
    }

    /// Every descendant element with this local name, in document order.
    pub fn descendants<'a>(&'a self, local_name: &str, out: &mut Vec<&'a Element>) {
        for e in self.elements() {
            if local(&e.name) == local_name {
                out.push(e);
            }
            e.descendants(local_name, out);
        }
    }

    /// `OpenXmlElement.InnerText`: the text of every descendant, concatenated.
    /// Whitespace between child elements is formatting, not text (the SDK drops it).
    pub fn inner_text(&self) -> String {
        let mixed = self.elements().next().is_some();
        let mut s = String::new();
        for n in &self.children {
            match n {
                Node::Text(t) if mixed && t.trim().is_empty() => {}
                Node::Text(t) => s.push_str(t),
                Node::Element(e) => s.push_str(&e.inner_text()),
            }
        }
        s
    }

    pub fn write(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.name);
        for (k, v) in &self.attrs {
            out.push_str(&format!(" {}=\"{}\"", k, escape(v, true)));
        }
        if self.children.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        for n in &self.children {
            match n {
                Node::Text(t) => out.push_str(&escape(t, false)),
                Node::Element(e) => e.write(out),
            }
        }
        out.push_str(&format!("</{}>", self.name));
    }
}

pub fn escape(s: &str, attr: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if attr => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(end) = tail.find(';') else {
            out.push_str(tail);
            return out;
        };
        let ent = &tail[1..end];
        let c = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => ent
                .strip_prefix("#x")
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match c {
            Some(c) => out.push(c),
            None => out.push_str(&tail[..=end]),
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Parses a document and returns its root element.
pub fn parse(text: &str) -> Result<Element, String> {
    const BAD: &str = "The Excel workbook contains invalid XML.";
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let b = text.as_bytes();
    let mut i = 0;
    let mut stack: Vec<Element> = vec![Element::new("#document")];
    while i < b.len() {
        if b[i] != b'<' {
            let end = text[i..].find('<').map_or(b.len(), |k| i + k);
            let t = &text[i..end];
            if stack.len() > 1 {
                stack.last_mut().unwrap().children.push(Node::Text(unescape(t)));
            }
            i = end;
            continue;
        }
        let rest = &text[i..];
        if rest.starts_with("<?") {
            i += rest.find("?>").ok_or(BAD)? + 2;
        } else if rest.starts_with("<!--") {
            i += rest.find("-->").ok_or(BAD)? + 3;
        } else if rest.starts_with("<![CDATA[") {
            let end = rest.find("]]>").ok_or(BAD)?;
            stack
                .last_mut()
                .unwrap()
                .children
                .push(Node::Text(rest[9..end].to_string()));
            i += end + 3;
        } else if rest.starts_with("<!") {
            i += rest.find('>').ok_or(BAD)? + 1;
        } else if rest.starts_with("</") {
            let end = rest.find('>').ok_or(BAD)?;
            let e = stack.pop().ok_or(BAD)?;
            if stack.is_empty() {
                return Err(BAD.into());
            }
            stack.last_mut().unwrap().children.push(Node::Element(e));
            i += end + 1;
        } else {
            // Start tag: name, attributes, optional self close.
            let mut j = i + 1;
            while j < b.len() && !matches!(b[j], b' ' | b'\t' | b'\r' | b'\n' | b'>' | b'/') {
                j += 1;
            }
            let mut e = Element::new(&text[i + 1..j]);
            loop {
                while j < b.len() && matches!(b[j], b' ' | b'\t' | b'\r' | b'\n') {
                    j += 1;
                }
                if j >= b.len() {
                    return Err(BAD.into());
                }
                if b[j] == b'>' {
                    stack.push(e);
                    j += 1;
                    break;
                }
                if b[j] == b'/' {
                    if b.get(j + 1) != Some(&b'>') {
                        return Err(BAD.into());
                    }
                    stack.last_mut().unwrap().children.push(Node::Element(e));
                    j += 2;
                    break;
                }
                let k0 = j;
                while j < b.len() && !matches!(b[j], b'=' | b' ' | b'\t' | b'\r' | b'\n') {
                    j += 1;
                }
                let key = text[k0..j].to_string();
                while j < b.len() && b[j] != b'=' {
                    j += 1;
                }
                j += 1;
                while j < b.len() && matches!(b[j], b' ' | b'\t' | b'\r' | b'\n') {
                    j += 1;
                }
                let q = *b.get(j).ok_or(BAD)?;
                if q != b'"' && q != b'\'' {
                    return Err(BAD.into());
                }
                let v0 = j + 1;
                let v1 = text[v0..].find(q as char).ok_or(BAD)? + v0;
                e.attrs.push((key, unescape(&text[v0..v1])));
                j = v1 + 1;
            }
            i = j;
        }
    }
    if stack.len() != 1 {
        return Err(BAD.into());
    }
    let doc = stack.pop().unwrap();
    let root = doc.elements().next().cloned();
    root.ok_or_else(|| BAD.to_string())
}

pub fn to_document(root: &Element) -> String {
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
    root.write(&mut s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_writes() {
        let x = "<?xml version=\"1.0\"?><a xmlns:r=\"u\" r:id=\"rId1\"><b t=\"s\">x &amp; y</b><c/><![CDATA[<z>]]></a>";
        let e = parse(x).unwrap();
        assert_eq!(e.attr("r:id"), Some("rId1"));
        assert_eq!(e.child("b").unwrap().inner_text(), "x & y");
        assert_eq!(e.inner_text(), "x & y<z>");
        let mut s = String::new();
        e.write(&mut s);
        assert_eq!(parse(&s).unwrap().child("b").unwrap().attr("t"), Some("s"));
    }
}
