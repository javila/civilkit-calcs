//! Port of Calcpad.Core/UiSyntax.cs: the right hand side shapes a `#UI` control accepts.
use crate::extension_methods::{enumerate_comments, is_letter};
use crate::messages;
use crate::netfmt::fmt;

/// `UiSyntax.EnumerateAssignments`
pub fn enumerate_assignments(s: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for segment in enumerate_comments(s) {
        if !is_code(segment) {
            continue;
        }
        let i = match segment.find('=') {
            Some(i) if i >= 1 => i,
            _ => continue,
        };
        let name = segment[..i].trim();
        if !name.is_empty() {
            found.push((name.to_string(), segment[i + 1..].trim().to_string()));
        }
    }
    found
}

/// True for a comment enumerator segment that is code, not a comment.
pub fn is_code(segment: &str) -> bool {
    !segment.is_empty() && !segment.starts_with('\'') && !segment.starts_with('"')
}

/// The control type a directive renders: the declared one, or the shape of the right hand side.
pub fn resolve_type(declared_type: Option<&str>, rhs: &str) -> String {
    match declared_type {
        Some(t) => t.to_string(),
        None if is_datagrid_rhs(rhs) => "datagrid".to_string(),
        None => "entry".to_string(),
    }
}

fn first_last(s: &str) -> (Option<char>, Option<char>) {
    (s.chars().next(), s.chars().next_back())
}

pub fn is_datagrid_rhs(rhs: &str) -> bool {
    let (f, l) = first_last(rhs);
    rhs.chars().count() > 1 && f == Some('[') && l == Some(']')
        || starts_with_function(rhs, "vector")
        || starts_with_function(rhs, "matrix")
}

fn starts_with_ignore_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let n = prefix.len();
    if s.len() >= n && s.is_char_boundary(n) && s[..n].eq_ignore_ascii_case(prefix) {
        Some(&s[n..])
    } else {
        None
    }
}

pub fn starts_with_function(rhs: &str, func_name: &str) -> bool {
    match starts_with_ignore_case(rhs, func_name) {
        Some(rest) => rest.trim_start().starts_with('('),
        None => false,
    }
}

/// A grid and an `allowExpression` box both replace the right hand side whole.
pub fn is_value_for(rhs: &str, ty: &str, allow_expression: bool) -> bool {
    ty == "datagrid" || allow_expression || is_value(rhs)
}

/// Why a declared type cannot render its value, or None when they agree.
pub fn type_mismatch(declared_type: Option<&str>, rhs: &str, allow_expression: bool) -> Option<String> {
    match declared_type {
        Some("datagrid") if is_number(rhs.trim()) => {
            Some(messages::The_UI_datagrid_requires_a_vector_or_matrix_value.to_string())
        }
        Some(t @ ("entry" | "dropdown" | "radio" | "checkbox")) if !allow_expression && is_datagrid_rhs(rhs) => {
            Some(fmt(messages::The_UI_0_cannot_hold_a_vector_or_matrix_value, &[t]))
        }
        _ => None,
    }
}

pub fn is_value(rhs: &str) -> bool {
    let rhs = rhs.trim();
    if rhs.is_empty() {
        return true;
    }
    if rhs.starts_with('[') {
        return is_matrix_literal(rhs);
    }
    is_constructor(rhs, "vector") || is_constructor(rhs, "matrix") || is_number(rhs)
}

fn is_matrix_literal(s: &str) -> bool {
    if !s.ends_with(']') || s.len() < 2 {
        return false;
    }
    for row in s[1..s.len() - 1].split('|') {
        for cell in row.split(';') {
            let value = cell.trim();
            if !value.is_empty() && !is_number(value) {
                return false;
            }
        }
    }
    true
}

fn is_constructor(s: &str, name: &str) -> bool {
    match starts_with_ignore_case(s, name) {
        Some(rest) => {
            let rest = rest.trim_start();
            rest.chars().count() > 1 && rest.starts_with('(') && closes_at_end(rest)
        }
        None => false,
    }
}

fn closes_at_end(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut depth = 0;
    for (i, &c) in chars.iter().enumerate() {
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                return i == chars.len() - 1;
            }
        }
    }
    false
}

fn skip_digits(s: &[char], i: &mut usize) -> usize {
    let start = *i;
    while *i < s.len() && s[*i].is_ascii_digit() {
        *i += 1;
    }
    *i - start
}

pub(crate) fn is_number(s: &str) -> bool {
    let s: Vec<char> = s.chars().collect();
    if s.is_empty() {
        return false;
    }
    let mut i = if matches!(s[0], '+' | '-' | '\u{2212}') { 1 } else { 0 };
    let mut digits = skip_digits(&s, &mut i);
    if i < s.len() && s[i] == '.' {
        i += 1;
        digits += skip_digits(&s, &mut i);
    }
    digits > 0 && is_units(&s[i..])
}

fn is_units(s: &[char]) -> bool {
    if s.iter().all(|c| c.is_whitespace()) {
        return true;
    }
    let mut expect_name = true;
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if c == ' ' {
            i += 1;
        } else if expect_name {
            if !is_unit_char(c) {
                return false;
            }
            while i < s.len() && is_unit_char(s[i]) {
                i += 1;
            }
            expect_name = false;
        } else if c == '^' {
            i += 1;
            if !is_power(s, &mut i) {
                return false;
            }
        } else if matches!(c, '*' | '/' | '\u{00B7}' | '\u{00D7}' | '\u{2219}') {
            i += 1;
            expect_name = true;
        } else {
            return false;
        }
    }
    !expect_name
}

fn is_power(s: &[char], i: &mut usize) -> bool {
    while *i < s.len() && s[*i] == ' ' {
        *i += 1;
    }
    if *i < s.len() && matches!(s[*i], '+' | '-' | '\u{2212}') {
        *i += 1;
    }
    let mut digits = skip_digits(s, i);
    if *i < s.len() && s[*i] == '.' {
        *i += 1;
        digits += skip_digits(s, i);
    }
    digits > 0
}

fn is_unit_char(c: char) -> bool {
    is_letter(c)
        || matches!(
            c,
            '\u{00B0}' | '%' | '\u{2030}' | '\u{2031}' | '\u{2032}' | '\u{2033}' | '\u{2127}' | '_'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert!(is_value("5"));
        assert!(is_value("-2.5 kN/m^2"));
        assert!(is_value("[1; 2 | 3; 4]"));
        assert!(is_value("vector(3)"));
        assert!(!is_value("a + 1"));
        assert!(!is_value("2*a"));
        assert!(is_datagrid_rhs("[1; 2]"));
        assert!(!is_datagrid_rhs("5"));
    }

    #[test]
    fn assignments_skip_comments() {
        let a = enumerate_assignments("a = 5'note = 3'b = 2");
        assert_eq!(a, vec![("a".into(), "5".into()), ("b".into(), "2".into())]);
    }
}
