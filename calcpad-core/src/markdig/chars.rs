//! Character classes and small scanners shared by the block and inline parsers
//! (Markdig Helpers/CharHelper.cs, HtmlHelper.cs and the CommonMark regular expressions).

/// `CharHelper.IsWhitespace`: Unicode Zs, tab, line feed, form feed, carriage return.
pub fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r')
        || matches!(
            c,
            '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
        )
}

pub fn is_space_or_tab(c: char) -> bool {
    c == ' ' || c == '\t'
}

pub fn is_ascii_punctuation(c: char) -> bool {
    c.is_ascii_punctuation()
}

/// `CharHelper.CheckUnicodeCategory`: (is whitespace, is punctuation). The NUL character
/// (start or end of the text) counts as both. Non ASCII punctuation is the Unicode P and S
/// categories, approximated here as "not alphanumeric, not whitespace, not a control or
/// combining character".
pub fn check_unicode_category(c: char) -> (bool, bool) {
    if is_whitespace(c) {
        return (true, false);
    }
    if (c as u32) <= 127 {
        return (c == '\0', c == '\0' || c.is_ascii_punctuation());
    }
    let combining =
        matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F);
    (false, !c.is_alphanumeric() && !c.is_control() && !combining)
}

fn is_punctuation_exception(c: char) -> bool {
    matches!(c, '\u{2212}' | '-' | '\u{2020}' | '\u{2021}')
}

/// `CharHelper.CheckOpenCloseDelimiter`
pub fn check_open_close_delimiter(pc: char, c: char, enable_within_word: bool) -> (bool, bool) {
    let (prev_is_white_space, prev_is_punctuation) = check_unicode_category(pc);
    let (next_is_white_space, next_is_punctuation) = check_unicode_category(c);
    let prev_is_excepted = prev_is_punctuation && is_punctuation_exception(pc);
    let next_is_excepted = next_is_punctuation && is_punctuation_exception(c);
    let mut can_open = !next_is_white_space
        && ((!next_is_punctuation || next_is_excepted) || prev_is_white_space || prev_is_punctuation);
    let mut can_close = !prev_is_white_space
        && ((!prev_is_punctuation || prev_is_excepted) || next_is_white_space || next_is_punctuation);
    if !enable_within_word {
        let temp = can_open;
        can_open = can_open && (!can_close || prev_is_punctuation);
        can_close = can_close && (!temp || next_is_punctuation);
    }
    (can_open, can_close)
}

/// `HtmlRenderer.WriteEscape`
pub fn escape_html(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

/// `HtmlRenderer.WriteEscapeUrl` (without the IDN mapping of non ASCII host names)
pub fn escape_url(s: &str, out: &mut String) {
    const KEEP: &str = "!#$%()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
    for c in s.chars() {
        if KEEP.contains(c) {
            out.push(c);
        } else if (c as u32) < 128 {
            if c == '&' {
                out.push_str("&amp;");
            } else {
                out.push_str(&format!("%{:02X}", c as u32));
            }
        } else {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
}

/// Matches an entity (`&name;`, `&#123;`, `&#x1F;`) at the start of `s`, returning its length.
pub fn match_entity(s: &[char]) -> Option<usize> {
    if s.first() != Some(&'&') {
        return None;
    }
    let mut i = 1;
    if s.get(i) == Some(&'#') {
        i += 1;
        let hex = matches!(s.get(i), Some('x') | Some('X'));
        if hex {
            i += 1;
        }
        let start = i;
        while i < s.len()
            && (if hex {
                s[i].is_ascii_hexdigit()
            } else {
                s[i].is_ascii_digit()
            })
        {
            i += 1;
        }
        let n = i - start;
        if n == 0 || n > if hex { 6 } else { 7 } {
            return None;
        }
    } else {
        let start = i;
        if !s.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        while i < s.len() && s[i].is_ascii_alphanumeric() {
            i += 1;
        }
        if i - start > 32 {
            return None;
        }
    }
    if s.get(i) == Some(&';') {
        Some(i + 1)
    } else {
        None
    }
}

fn tag_name_len(s: &[char], i: usize) -> usize {
    let mut j = i;
    if !s.get(j).is_some_and(|c| c.is_ascii_alphabetic()) {
        return 0;
    }
    while j < s.len() && (s[j].is_ascii_alphanumeric() || s[j] == '-') {
        j += 1;
    }
    j - i
}

fn skip_ws(s: &[char], mut i: usize) -> (usize, bool) {
    let start = i;
    while i < s.len() && matches!(s[i], ' ' | '\t' | '\n' | '\r' | '\u{000C}') {
        i += 1;
    }
    (i, i > start)
}

/// Matches an open tag `<name attr="v" ... />` at `s[0]`, returning its length.
fn match_open_tag(s: &[char]) -> Option<usize> {
    let n = tag_name_len(s, 1);
    if n == 0 {
        return None;
    }
    let mut i = 1 + n;
    loop {
        let (j, had_ws) = skip_ws(s, i);
        // attribute
        if had_ws
            && s.get(j)
                .is_some_and(|&c| c.is_ascii_alphabetic() || c == '_' || c == ':')
        {
            let mut k = j + 1;
            while k < s.len() && (s[k].is_ascii_alphanumeric() || matches!(s[k], '_' | '.' | ':' | '-')) {
                k += 1;
            }
            // optional value
            let (k2, _) = skip_ws(s, k);
            if s.get(k2) == Some(&'=') {
                let (k3, _) = skip_ws(s, k2 + 1);
                match s.get(k3) {
                    Some('"') | Some('\'') => {
                        let q = s[k3];
                        let mut e = k3 + 1;
                        while e < s.len() && s[e] != q {
                            e += 1;
                        }
                        if e >= s.len() {
                            return None;
                        }
                        i = e + 1;
                    }
                    Some(_) => {
                        let mut e = k3;
                        while e < s.len()
                            && !matches!(s[e], ' ' | '\t' | '\n' | '\r' | '"' | '\'' | '=' | '<' | '>' | '`')
                        {
                            e += 1;
                        }
                        if e == k3 {
                            return None;
                        }
                        i = e;
                    }
                    None => return None,
                }
            } else {
                i = k;
            }
            continue;
        }
        if s.get(j) == Some(&'/') && s.get(j + 1) == Some(&'>') {
            return Some(j + 2);
        }
        if s.get(j) == Some(&'>') {
            return Some(j + 1);
        }
        return None;
    }
}

fn match_closing_tag(s: &[char]) -> Option<usize> {
    if s.get(1) != Some(&'/') {
        return None;
    }
    let n = tag_name_len(s, 2);
    if n == 0 {
        return None;
    }
    let (j, _) = skip_ws(s, 2 + n);
    if s.get(j) == Some(&'>') {
        Some(j + 1)
    } else {
        None
    }
}

fn find_seq(s: &[char], from: usize, pat: &str) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    if s.len() < p.len() {
        return None;
    }
    (from..=s.len() - p.len()).find(|&i| s[i..i + p.len()] == p[..])
}

/// Inline raw HTML (CommonMark 6.6): open tag, closing tag, comment, processing
/// instruction, declaration or CDATA section at `s[0] == '<'`. Returns its length.
pub fn match_html_tag(s: &[char]) -> Option<usize> {
    if s.first() != Some(&'<') {
        return None;
    }
    match s.get(1) {
        Some('/') => match_closing_tag(s),
        Some('?') => find_seq(s, 2, "?>").map(|i| i + 2),
        Some('!') => {
            if s.get(2) == Some(&'-') && s.get(3) == Some(&'-') {
                if s.get(4) == Some(&'>') {
                    return Some(5);
                }
                if s.get(4) == Some(&'-') && s.get(5) == Some(&'>') {
                    return Some(6);
                }
                find_seq(s, 4, "-->").map(|i| i + 3)
            } else if s.len() >= 9 && s[2..9].iter().collect::<String>() == "[CDATA[" {
                find_seq(s, 9, "]]>").map(|i| i + 3)
            } else if s.get(2).is_some_and(|c| c.is_ascii_alphabetic()) {
                find_seq(s, 2, ">").map(|i| i + 1)
            } else {
                None
            }
        }
        _ => match_open_tag(s),
    }
}

const BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
];

fn lower(s: &[char]) -> String {
    s.iter().collect::<String>().to_ascii_lowercase()
}

/// The type (1 to 7) of an HTML block that starts at `s` (which begins with `<`).
pub fn html_block_start(s: &[char]) -> Option<u8> {
    if s.first() != Some(&'<') {
        return None;
    }
    // 1: <script, <pre, <style, <textarea
    for name in ["script", "pre", "style", "textarea"] {
        let n = name.len();
        if s.len() > n && lower(&s[1..1 + n]) == name {
            match s.get(1 + n) {
                None | Some(' ') | Some('\t') | Some('>') => return Some(1),
                _ => {}
            }
        }
    }
    if s.len() >= 4 && s[1..4].iter().collect::<String>() == "!--" {
        return Some(2);
    }
    if s.get(1) == Some(&'?') {
        return Some(3);
    }
    if s.get(1) == Some(&'!') && s.get(2).is_some_and(|c| c.is_ascii_alphabetic()) {
        return Some(4);
    }
    if s.len() >= 9 && s[1..9].iter().collect::<String>() == "![CDATA[" {
        return Some(5);
    }
    // 6: block tag names
    let start = if s.get(1) == Some(&'/') { 2 } else { 1 };
    let n = tag_name_len(s, start);
    if n > 0 {
        let name = lower(&s[start..start + n]);
        if BLOCK_TAGS.contains(&name.as_str()) {
            match s.get(start + n) {
                None | Some(' ') | Some('\t') | Some('>') => return Some(6),
                Some('/') if s.get(start + n + 1) == Some(&'>') => return Some(6),
                _ => {}
            }
        }
    }
    // 7: a complete open or closing tag alone on the line
    let len = if s.get(1) == Some(&'/') {
        match_closing_tag(s)
    } else {
        let n = tag_name_len(s, 1);
        if n > 0 && ["script", "style", "pre", "textarea"].contains(&lower(&s[1..1 + n]).as_str()) {
            None
        } else {
            match_open_tag(s)
        }
    };
    if let Some(len) = len {
        if s[len..].iter().all(|&c| is_space_or_tab(c)) {
            return Some(7);
        }
    }
    None
}

/// Does the line close an HTML block of the given type (1 to 5)?
pub fn html_block_end(kind: u8, line: &[char]) -> bool {
    let l = lower(line);
    match kind {
        1 => l.contains("</script>") || l.contains("</pre>") || l.contains("</style>") || l.contains("</textarea>"),
        2 => l.contains("-->"),
        3 => l.contains("?>"),
        4 => l.contains('>'),
        5 => l.contains("]]>"),
        _ => false,
    }
}

/// Backslash escapable characters (ASCII punctuation)
pub fn is_escapable(c: char) -> bool {
    c.is_ascii_punctuation()
}
