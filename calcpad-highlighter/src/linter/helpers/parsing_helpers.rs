//! ParsingHelpers.cs
use super::character_helpers::{is_identifier_char, is_identifier_start_char_with_underscore};
use super::parameter_parser::split_by_delimiter;
use crate::parsing::split_enumerator;
use crate::text::{is_letter, is_white_space, to_string, trim};

pub fn skip_whitespace(text: &[char], pos: &mut usize) {
    while *pos < text.len() && is_white_space(text[*pos]) {
        *pos += 1;
    }
}

/// Index of the bracket that closes the one at `open_pos` (counting nesting from there), -1 if none.
pub fn find_matching_close(text: &[char], open_pos: usize, open: char, close: char) -> i32 {
    let mut depth = 0;
    for (i, &c) in text.iter().enumerate().skip(open_pos) {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return i as i32;
            }
        }
    }
    -1
}

/// Splits `name = default` at the first top-level `=`. Returns (name, default, has_default).
pub fn split_parameter_default(parameter: &[char]) -> (&[char], &[char], bool) {
    let mut depth = 0;
    for (i, &c) in parameter.iter().enumerate() {
        if c == '(' || c == '[' || c == '{' {
            depth += 1;
        } else if c == ')' || c == ']' || c == '}' {
            depth -= 1;
        } else if c == '=' && depth == 0 {
            return (trim(&parameter[..i]), trim(&parameter[i + 1..]), true);
        }
    }
    (trim(parameter), &parameter[0..0], false)
}

/// Parses a `;` separated parameter list. The defaults list is `None` when no parameter has one.
pub fn parse_param_list(params_str: &str) -> (Vec<String>, Option<Vec<Option<String>>>) {
    let mut names = Vec::new();
    let mut defaults: Vec<Option<String>> = Vec::new();
    for param in split_by_delimiter(params_str, ';') {
        if param.is_empty() {
            continue;
        }
        let chars: Vec<char> = param.chars().collect();
        let (name, value, has_default) = split_parameter_default(&chars);
        names.push(to_string(name));
        defaults.push(if has_default { Some(to_string(value)) } else { None });
    }
    let any = defaults.iter().any(|d| d.is_some());
    (names, if any { Some(defaults) } else { None })
}

pub fn count_required_params(param_count: i32, defaults: Option<&Vec<Option<String>>>) -> i32 {
    match defaults {
        Some(d) => d.iter().filter(|x| x.is_none()).count() as i32,
        None => param_count,
    }
}

/// Names of required parameters that follow an optional one.
pub fn required_after_optional(names: &[String], defaults: Option<&Vec<Option<String>>>) -> Vec<String> {
    let mut out = Vec::new();
    let first = defaults.and_then(|d| d.iter().position(|x| x.is_some()));
    if let (Some(first), Some(d)) = (first, defaults) {
        for i in first + 1..names.len() {
            if d[i].is_none() {
                out.push(names[i].clone());
            }
        }
    }
    out
}

/// `name=value` keyword argument: returns (name, value).
pub fn try_parse_keyword_arg(arg: &str) -> Option<(String, String)> {
    if arg.chars().all(is_white_space) {
        return None;
    }
    let chars: Vec<char> = arg.chars().collect();
    let (name, value, has_default) = split_parameter_default(trim(&chars));
    if !has_default || value.first() == Some(&'=') || !is_identifier(name) {
        return None;
    }
    Some((to_string(name), to_string(value)))
}

fn is_identifier(s: &[char]) -> bool {
    if s.is_empty() || !is_identifier_start_char_with_underscore(s[0]) {
        return false;
    }
    let last = if s[s.len() - 1] == '$' { s.len() - 1 } else { s.len() };
    (1..last).all(|i| is_identifier_char(s[i]))
}

/// Parameter names of a function definition line `name(params) = ...`.
pub fn get_function_params_from_line(line: &str) -> crate::text::StrSet {
    let mut result = crate::text::StrSet::new();
    let l: Vec<char> = line.chars().collect();
    let Some(paren_open) = l.iter().position(|c| *c == '(') else {
        return result;
    };
    let paren_close = find_matching_close(&l, paren_open, '(', ')');
    if paren_close < 0 {
        return result;
    }
    let paren_close = paren_close as usize;
    if !l[paren_close..].contains(&'=') {
        return result;
    }
    let before = trim(&l[..paren_open]);
    if before.is_empty() || !is_letter(before[0]) {
        return result;
    }
    for p in split_enumerator(&l[paren_open + 1..paren_close], ';') {
        let t = trim(p);
        if !t.is_empty() {
            let (name, _, _) = split_parameter_default(t);
            if !name.is_empty() {
                result.insert(&to_string(name));
            }
        }
    }
    result
}

/// (found, params text) for the parenthesised list that follows a function name at `after_func_name`.
pub fn extract_params_string(line: &[char], after_func_name: usize) -> (bool, String) {
    let mut pos = after_func_name;
    skip_whitespace(line, &mut pos);
    if pos >= line.len() || line[pos] != '(' {
        return (false, String::new());
    }
    let start = pos + 1;
    let close = find_matching_close(line, pos, '(', ')');
    if close >= 0 {
        return (true, to_string(&line[start..close as usize]));
    }
    (true, to_string(&line[start..]))
}

pub fn find_closing_paren(line: &[char], after_func_name: usize) -> usize {
    let mut pos = after_func_name;
    skip_whitespace(line, &mut pos);
    if pos >= line.len() || line[pos] != '(' {
        return after_func_name;
    }
    let close = find_matching_close(line, pos, '(', ')');
    if close >= 0 {
        close as usize + 1
    } else {
        line.len()
    }
}

pub fn extract_block_content(line: &[char], brace_start: usize) -> String {
    let start = brace_start + 1;
    let close = find_matching_close(line, brace_start, '{', '}');
    if close >= 0 {
        to_string(&line[start..close as usize])
    } else {
        to_string(&line[start..])
    }
}

pub fn find_closing_brace(line: &[char], brace_start: usize) -> usize {
    let close = find_matching_close(line, brace_start, '{', '}');
    if close >= 0 {
        close as usize
    } else {
        line.len()
    }
}
