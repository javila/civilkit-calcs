//! Port of ExtensionMethods.cs, CommentEnumerator.cs and SplitEnumerator.cs, plus the
//! .NET string helpers (EnumerateLines, Trim) whose exact semantics the engine relies on.

/// `double.AlmostEquals`: equal within 3 units in the last place.
pub fn almost_equals(a: f64, b: f64) -> bool {
    let l1 = a.to_bits() as i64;
    let l2 = b.to_bits() as i64;
    if (l1 >> 63) != (l2 >> 63) {
        return a == b || a.is_nan() && b.is_nan();
    }
    (l1.wrapping_sub(l2)).wrapping_abs() < 4
}

/// .NET `char.IsWhiteSpace`.
pub fn is_white_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | '\u{0085}' | '\u{00A0}'
    ) || matches!(
        c,
        '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// .NET `string.Trim()` / `ReadOnlySpan<char>.Trim()`.
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_white_space)
}

pub fn trim_start(s: &str) -> &str {
    s.trim_start_matches(is_white_space)
}

pub fn trim_end(s: &str) -> &str {
    s.trim_end_matches(is_white_space)
}

/// .NET `char.IsLetter` (Unicode L categories). Rust's `is_alphabetic` also accepts some
/// marks and letter numbers; those are excluded for the ranges that matter in practice.
pub fn is_letter(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphabetic();
    }
    if !c.is_alphabetic() {
        return false;
    }
    // Exclude combining marks (Mn/Mc) and letter numbers (Nl) that Rust treats as alphabetic.
    let u = c as u32;
    !matches!(u,
        0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05C7 | 0x0610..=0x061A | 0x064B..=0x065F
        | 0x0670 | 0x06D6..=0x06ED | 0x16EE..=0x16F0 | 0x2160..=0x2188 | 0x3007 | 0x3021..=0x3029
        | 0x3038..=0x303A | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F
    )
}

/// .NET `char.IsDigit` (Unicode Nd).
pub fn is_digit(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_digit();
    }
    c.is_numeric()
        && !matches!(c as u32, 0x00B2 | 0x00B3 | 0x00B9 | 0x00BC..=0x00BE | 0x2070..=0x209F | 0x2150..=0x218F | 0x2460..=0x24FF)
}

/// `string.EnumerateLines()` (the Calcpad extension): drops one trailing newline, then
/// splits like `MemoryExtensions.EnumerateLines` on \r\n, \r, \n, \f, \u0085,  ,  .
pub fn enumerate_lines(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return Vec::new();
    }
    let mut t = s;
    if t.ends_with('\n') {
        t = &t[..t.len() - 1];
        if t.ends_with('\r') {
            t = &t[..t.len() - 1];
        }
    } else if t.ends_with('\r') {
        t = &t[..t.len() - 1];
    }
    span_enumerate_lines(t)
}

/// `ReadOnlySpan<char>.EnumerateLines()`.
pub fn span_enumerate_lines(t: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let bytes = t.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < t.len() {
        let c = t[i..].chars().next().unwrap();
        let cl = c.len_utf8();
        match c {
            '\r' => {
                lines.push(&t[start..i]);
                if i + 1 < t.len() && bytes[i + 1] == b'\n' {
                    i += 2;
                } else {
                    i += 1;
                }
                start = i;
                continue;
            }
            '\n' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}' => {
                lines.push(&t[start..i]);
                i += cl;
                start = i;
                continue;
            }
            _ => {}
        }
        i += cl;
    }
    lines.push(&t[start..]);
    lines
}

/// Port of CommentEnumerator: splits a line into code and quoted comment segments.
pub fn enumerate_comments(span: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = span;
    let mut comment_char: Option<char> = None;
    while !rest.is_empty() {
        let mut i: isize;
        if let Some(cc) = comment_char {
            // i = _span[1..].IndexOf(_commentChar) + 2
            let first_len = rest.chars().next().unwrap().len_utf8();
            i = match rest[first_len..].find(cc) {
                Some(p) => (p + first_len + cc.len_utf8()) as isize,
                None => rest.len() as isize,
            };
            comment_char = None;
        } else {
            match rest.find(['\'', '"']) {
                Some(p) => {
                    i = p as isize;
                    comment_char = rest[p..].chars().next();
                }
                None => {
                    i = -1;
                    comment_char = None;
                }
            }
        }
        let j = rest.find('\n').map(|x| x as isize).unwrap_or(-1);
        if j > -1 && j < i {
            comment_char = None;
            i = j;
        }
        if i < 0 {
            out.push(rest);
            rest = "";
            continue;
        }
        if i == 0 {
            out.push("");
            // C# yields default and keeps the span unchanged; the next call handles the comment.
            if comment_char.is_none() {
                // Should not happen; avoid infinite loop.
                rest = &rest[rest.chars().next().unwrap().len_utf8()..];
            }
            continue;
        }
        let mut iu = i as usize;
        if i == j {
            iu += 1;
        }
        let iu = iu.min(rest.len());
        out.push(&rest[..iu]);
        rest = &rest[iu..];
    }
    out
}

/// `ReadOnlySpan<char>.TagEndIndex()`: index of the tag closing '>' skipping quoted values.
pub fn tag_end_index(s: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in s.char_indices().skip(1) {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == '>' {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn almost() {
        assert!(almost_equals(1.0, 1.0 + f64::EPSILON));
        assert!(!almost_equals(1.0, 1.0001));
        assert!(almost_equals(0.0, -0.0));
    }

    #[test]
    fn comments() {
        let parts = enumerate_comments("a = 1 'note' b");
        assert_eq!(parts, vec!["a = 1 ", "'note'", " b"]);
        let parts = enumerate_comments("'Text");
        assert_eq!(parts, vec!["", "'Text"]);
    }

    #[test]
    fn lines() {
        assert_eq!(enumerate_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(enumerate_lines("a\r\nb"), vec!["a", "b"]);
    }
}
