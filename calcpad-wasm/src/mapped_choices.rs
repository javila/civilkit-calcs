//! The choices of select, radio and checkbox elements that are mapped to text boxes
//! (docs/programming.md, "Advanced UI with Html and CSS"), put back into the rendered page.
//!
//! The input form stores a choice next to the entered values, as an override entry:
//!
//! * `map:<target>|select`: the value of the chosen option,
//! * `map:<target>|radio`: the value of the checked radio button,
//! * `map:<target>|checkbox|<n>`: `true` or `false` for the n-th checkbox of the target.
//!
//! A page rendered with those values (the form, the Report, the PDF, the Word export) would
//! otherwise show every select at its first option while the boxes hold the chosen values.
//! [`mark_mapped_choices`] sets `selected` / `checked` accordingly. `<target>` is the element's
//! `data-target`, else its `name`; an HTML parser ends a `<select>` when the next one starts, so
//! options belong to the select that last opened.
use std::collections::HashMap;

/// One attribute of a tag: lower case name, decoded value, and the span of the whole attribute
/// including the whitespace before it.
struct Attr {
    name: String,
    value: Option<String>,
    start: usize,
    end: usize,
}

fn decode(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// The attributes of a tag (`tag` is `<name ...>`).
fn attrs(tag: &str) -> Vec<Attr> {
    let b = tag.as_bytes();
    let mut i = 1;
    while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'>' && b[i] != b'/' {
        i += 1;
    }
    let mut out = Vec::new();
    loop {
        let start = i;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() || b[i] == b'>' || b[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'=' && b[i] != b'>' && b[i] != b'/' {
            i += 1;
        }
        let name = tag[name_start..i].to_ascii_lowercase();
        let mut j = i;
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        let mut value = None;
        if j < b.len() && b[j] == b'=' {
            j += 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                let q = b[j];
                let v0 = j + 1;
                let mut k = v0;
                while k < b.len() && b[k] != q {
                    k += 1;
                }
                value = Some(decode(&tag[v0..k]));
                i = (k + 1).min(b.len());
            } else {
                let v0 = j;
                let mut k = v0;
                while k < b.len() && !b[k].is_ascii_whitespace() && b[k] != b'>' {
                    k += 1;
                }
                value = Some(decode(&tag[v0..k]));
                i = k;
            }
        }
        out.push(Attr {
            name,
            value,
            start,
            end: i,
        });
    }
    out
}

fn attr<'a>(attrs: &'a [Attr], name: &str) -> Option<&'a str> {
    attrs.iter().find(|a| a.name == name).and_then(|a| a.value.as_deref())
}

/// `tag` with the boolean attribute `flag` removed, and added when `on`.
fn set_flag(tag: &str, flag: &str, on: bool) -> String {
    let mut t = tag.to_string();
    if let Some(a) = attrs(tag).iter().find(|a| a.name == flag) {
        t.replace_range(a.start..a.end, "");
    }
    if on {
        let close = if t.ends_with("/>") { t.len() - 2 } else { t.len() - 1 };
        t.insert_str(close, &format!(" {flag}"));
    }
    t
}

/// The source of the element: its target, unless it is a #UI control.
fn target(a: &[Attr]) -> Option<String> {
    if attr(a, "class").is_some_and(|c| c.split_whitespace().any(|w| w.starts_with("calcpad-ui"))) {
        return None;
    }
    attr(a, "data-target")
        .filter(|t| !t.is_empty())
        .or_else(|| attr(a, "name").filter(|t| !t.is_empty()))
        .map(str::to_string)
}

pub fn mark_mapped_choices(html: &str, overrides: &HashMap<String, String>) -> String {
    if !overrides.keys().any(|k| k.starts_with("map:")) {
        return html.to_string();
    }
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len() + 64);
    let mut pos = 0;
    // The value to select among the options of the select that is open.
    let mut open_select: Option<String> = None;
    let mut checkbox_count: HashMap<String, usize> = HashMap::new();
    while let Some(rel) = html[pos..].find('<') {
        let lt = pos + rel;
        out.push_str(&html[pos..lt]);
        // Comments, scripts and styles are copied whole.
        let skip_to = if html[lt..].starts_with("<!--") {
            html[lt..].find("-->").map(|e| lt + e + 3)
        } else if lower[lt..].starts_with("<script") {
            lower[lt..]
                .find("</script")
                .and_then(|e| html[lt + e..].find('>').map(|g| lt + e + g + 1))
        } else if lower[lt..].starts_with("<style") {
            lower[lt..]
                .find("</style")
                .and_then(|e| html[lt + e..].find('>').map(|g| lt + e + g + 1))
        } else {
            None
        };
        if let Some(end) = skip_to {
            out.push_str(&html[lt..end]);
            pos = end;
            continue;
        }
        // The tag ends at the first '>' outside quotes.
        let bytes = html.as_bytes();
        let mut k = lt + 1;
        let mut quote = 0u8;
        while k < bytes.len() {
            let c = bytes[k];
            if quote != 0 {
                if c == quote {
                    quote = 0;
                }
            } else if c == b'"' || c == b'\'' {
                // Only inside an attribute value; a stray quote in text before '=' is not one.
                if bytes[lt + 1..k].iter().rev().find(|b| !b.is_ascii_whitespace()) == Some(&b'=') {
                    quote = c;
                }
            } else if c == b'>' {
                break;
            }
            k += 1;
        }
        if k >= bytes.len() {
            out.push_str(&html[lt..]);
            pos = html.len();
            break;
        }
        let tag = &html[lt..=k];
        pos = k + 1;
        let name_end = tag[1..]
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .map_or(tag.len() - 1, |e| e + 1);
        let name = tag[1..name_end].to_ascii_lowercase();
        let a = attrs(tag);
        let rewritten = match name.as_str() {
            "select" => {
                open_select = target(&a).and_then(|t| overrides.get(&format!("map:{t}|select")).cloned());
                None
            }
            "/select" => {
                open_select = None;
                None
            }
            "option" => open_select.as_ref().and_then(|want| {
                let value = attr(&a, "value")?;
                Some(set_flag(tag, "selected", value == want))
            }),
            "input" => {
                let t = target(&a);
                match (t, attr(&a, "type").map(str::to_ascii_lowercase).as_deref()) {
                    (Some(t), Some("radio")) => {
                        let want = overrides.get(&format!("map:{t}|radio"));
                        want.and_then(|w| attr(&a, "value").map(|v| set_flag(tag, "checked", v == w)))
                    }
                    (Some(t), Some("checkbox")) => {
                        let n = checkbox_count.entry(t.clone()).or_insert(0);
                        let key = format!("map:{t}|checkbox|{n}");
                        *n += 1;
                        overrides.get(&key).map(|w| set_flag(tag, "checked", w == "true"))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        match rewritten {
            Some(t) => out.push_str(&t),
            None => out.push_str(tag),
        }
    }
    out.push_str(&html[pos..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ov(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn marks_the_chosen_option_with_unquoted_values() {
        let html = "<p>Pick <select data-target=\"section\"><option value=80;3.9>IPN 80</option><option value=100;4.5 id=\"x\">IPN 100</option></select></p>";
        let got = mark_mapped_choices(html, &ov(&[("map:section|select", "100;4.5")]));
        assert!(got.contains("<option value=80;3.9>"), "{got}");
        assert!(got.contains("<option value=100;4.5 id=\"x\" selected>"), "{got}");
    }

    #[test]
    fn a_second_select_takes_the_options_that_follow_it() {
        // The sample opens two selects for one list of options; the parser ends the first.
        let html = "<select data-target=\"s\"><select data-target=\"s\" style=\"x\"><option value=\"1\">a</option><option value=\"2\">b</option></select>";
        let got = mark_mapped_choices(html, &ov(&[("map:s|select", "2")]));
        assert!(got.contains("<option value=\"2\" selected>"), "{got}");
        assert!(!got.contains("<option value=\"1\" selected"), "{got}");
    }

    #[test]
    fn replaces_an_existing_selection() {
        let html = "<select name=\"t\"><option value=\"1\" selected>a</option><option value=\"2\">b</option></select>";
        let got = mark_mapped_choices(html, &ov(&[("map:t|select", "2")]));
        assert_eq!(
            got,
            "<select name=\"t\"><option value=\"1\">a</option><option value=\"2\" selected>b</option></select>"
        );
    }

    #[test]
    fn radio_and_checkbox() {
        let html = "<input name=\"r\" type=\"radio\" value=\"1\"/><input name=\"r\" type=\"radio\" value=\"2\" checked/><input name=\"c\" type=\"checkbox\" value=\"3\"/>";
        let got = mark_mapped_choices(html, &ov(&[("map:r|radio", "1"), ("map:c|checkbox|0", "true")]));
        assert_eq!(got, "<input name=\"r\" type=\"radio\" value=\"1\" checked/><input name=\"r\" type=\"radio\" value=\"2\"/><input name=\"c\" type=\"checkbox\" value=\"3\" checked/>");
        let off = mark_mapped_choices(
            "<input name=\"c\" type=\"checkbox\" value=\"3\" checked>",
            &ov(&[("map:c|checkbox|0", "false")]),
        );
        assert_eq!(off, "<input name=\"c\" type=\"checkbox\" value=\"3\">");
    }

    #[test]
    fn leaves_everything_else_alone() {
        let html = "<script>if (a<b) {}</script><!-- <select name=\"t\"> --><select name=\"other\"><option value=\"1\">a</option></select>";
        assert_eq!(mark_mapped_choices(html, &ov(&[("map:t|select", "1")])), html);
        assert_eq!(mark_mapped_choices(html, &ov(&[])), html);
    }

    #[test]
    fn calcpad_ui_controls_are_not_sources() {
        let html = "<select class=\"calcpad-ui-dropdown\" name=\"t\"><option value=\"1\">a</option><option value=\"2\">b</option></select>";
        assert_eq!(mark_mapped_choices(html, &ov(&[("map:t|select", "2")])), html);
    }
}
