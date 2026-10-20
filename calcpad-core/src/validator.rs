//! Port of Validator.cs.
use crate::extension_methods::{is_letter as char_is_letter, trim_start};
use crate::netfmt;

const CURRENCY_CHARS: [char; 9] = [
    '\u{20AC}', '\u{00A3}', '\u{20A4}', '\u{00A5}', '\u{00A2}', '\u{20BD}', '\u{20B9}', '\u{20A9}', '\u{20AA}',
];
const UNIT_SYMBOL_CHARS: [char; 6] = ['\u{00B0}', '\u{2032}', '\u{2033}', '%', '\u{2030}', '\u{2031}'];
const VAR_SYMBOL_CHARS: [char; 9] = [
    ',', '_', '\u{203E}', '\u{2034}', '\u{2057}', '\u{0301}', '\u{0304}', '\u{0307}', '\u{0308}',
];
const VAR_NON_LETTER_CHARS: [char; 2] = ['\u{2127}', '\u{2221}'];
const VAR_LETTER_CHARS: [char; 4] = ['\u{03D1}', '\u{03D5}', '\u{00F8}', '\u{00D8}'];
const SUPERSCRIPT_CHARS: [char; 15] = [
    '\u{2070}', '\u{00B9}', '\u{00B2}', '\u{00B3}', '\u{2074}', '\u{2075}', '\u{2076}', '\u{2077}', '\u{2078}',
    '\u{2079}', '\u{207A}', '\u{207B}', '\u{207C}', '\u{207D}', '\u{207E}',
];
const SUBSCRIPT_CHARS: [char; 15] = [
    '\u{2080}', '\u{2081}', '\u{2082}', '\u{2083}', '\u{2084}', '\u{2085}', '\u{2086}', '\u{2087}', '\u{2088}',
    '\u{2089}', '\u{208A}', '\u{208B}', '\u{208C}', '\u{208D}', '\u{208E}',
];

/// `Validator.UnitChars`
pub fn is_unit_char(c: char) -> bool {
    UNIT_SYMBOL_CHARS.contains(&c) || CURRENCY_CHARS.contains(&c)
}

fn is_var_starting_symbol(c: char) -> bool {
    is_unit_char(c) || VAR_NON_LETTER_CHARS.contains(&c)
}

fn is_var_chars(c: char) -> bool {
    is_var_starting_symbol(c)
        || VAR_SYMBOL_CHARS.contains(&c)
        || SUPERSCRIPT_CHARS.contains(&c)
        || SUBSCRIPT_CHARS.contains(&c)
        || VAR_LETTER_CHARS.contains(&c)
}

pub fn is_variable(name: &str) -> bool {
    if name.trim().is_empty() {
        return false;
    }
    let mut it = name.chars();
    let c = it.next().unwrap();
    if !is_var_starting_char(c) {
        return false;
    }
    it.all(is_var_char)
}

pub fn is_keyword(s: &str, keyword: &str) -> bool {
    starts_with_ignore_case(trim_start(s), keyword)
}

pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    let mut a = s.chars();
    for p in prefix.chars() {
        match a.next() {
            Some(c) if c.to_lowercase().eq(p.to_lowercase()) => {}
            _ => return false,
        }
    }
    true
}

pub fn is_macro_letter(c: char, position: usize) -> bool {
    c.is_ascii_alphabetic() || c == '_' || crate::extension_methods::is_digit(c) && position > 0
}

/// `Validator.IsLetter`
pub fn is_letter(c: char) -> bool {
    char_is_letter(c) || is_var_chars(c)
}

pub fn is_latin_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// `Validator.IsDigit`: 0-9 and the decimal symbol.
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit() || c == '.'
}

pub fn is_unit_start(c: char) -> bool {
    is_unit_char(c)
}

pub fn is_white_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

pub fn is_var_additional_char(c: char) -> bool {
    is_var_chars(c)
}

pub fn is_var_starting_char(c: char) -> bool {
    char_is_letter(c) || is_var_starting_symbol(c)
}

pub fn is_var_char(c: char) -> bool {
    is_letter(c) || c.is_ascii_digit()
}

/// `^[FCEGND]\d{0,2}$|^[0#]+(,[0#]+)?(\.[0#]+)?([eE][+-]?0+)?$` (case insensitive).
fn matches_format_regex(f: &str) -> bool {
    let chars: Vec<char> = f.chars().collect();
    if chars.is_empty() {
        return false;
    }
    let c0 = chars[0].to_ascii_uppercase();
    if "FCEGND".contains(c0) && chars.len() <= 3 && chars[1..].iter().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let mut i = 0;
    let placeholder = |c: char| c == '0' || c == '#';
    let start = i;
    while i < chars.len() && placeholder(chars[i]) {
        i += 1;
    }
    if i == start {
        return false;
    }
    if i < chars.len() && chars[i] == ',' {
        let s = i + 1;
        let mut k = s;
        while k < chars.len() && placeholder(chars[k]) {
            k += 1;
        }
        if k > s {
            i = k;
        } else {
            return false;
        }
    }
    if i < chars.len() && chars[i] == '.' {
        let s = i + 1;
        let mut k = s;
        while k < chars.len() && placeholder(chars[k]) {
            k += 1;
        }
        if k > s {
            i = k;
        } else {
            return false;
        }
    }
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        let mut k = i + 1;
        if k < chars.len() && (chars[k] == '+' || chars[k] == '-') {
            k += 1;
        }
        let s = k;
        while k < chars.len() && chars[k] == '0' {
            k += 1;
        }
        if k > s {
            i = k;
        } else {
            return false;
        }
    }
    i == chars.len()
}

pub fn is_valid_format_string(format: &str) -> bool {
    // `1d.ToString(format)` throws FormatException only for malformed standard formats,
    // which the regex rejects anyway.
    let _ = netfmt::double_to_string_format;
    matches_format_regex(format)
}

pub fn is_comment(s: &str) -> bool {
    let mut count = 0;
    let mut comment_char: Option<char> = None;
    for c in s.chars() {
        match comment_char {
            None => {
                if c == '"' || c == '\'' {
                    comment_char = Some(c);
                    count = 1;
                }
            }
            Some(cc) => {
                if c == cc {
                    count += 1;
                }
            }
        }
    }
    count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats() {
        assert!(is_valid_format_string("F2"));
        assert!(is_valid_format_string("0.00"));
        assert!(is_valid_format_string("#,##0.0"));
        assert!(is_valid_format_string("0.0E+00"));
        assert!(!is_valid_format_string("abc"));
        assert!(!is_valid_format_string("F123"));
    }
}
