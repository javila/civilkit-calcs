//! CalcpadTokenizer.Helpers.cs: static utility methods.
use super::SpecialContentType;
use crate::text::{is_letter, is_letter_or_digit, trim_start};

/// Permissive macro-identifier scan: Unicode letters allowed, `$` allowed after position 0.
pub(super) fn is_macro_ident_char(c: char, position: usize) -> bool {
    if position == 0 {
        return is_letter(c) || c == '_';
    }
    is_letter_or_digit(c) || c == '_' || c == '$'
}

fn starts_with_ignore_case(s: &[char], prefix: &str) -> bool {
    let p: Vec<char> = prefix.chars().collect();
    s.len() >= p.len() && crate::text::eq_ignore_case(&s[..p.len()].iter().collect::<String>(), prefix)
}

pub(super) fn is_plot_line(text: &[char]) -> bool {
    let t = trim_start(text);
    starts_with_ignore_case(t, "$plot") || starts_with_ignore_case(t, "$map")
}

pub(super) fn is_closing_special_content_tag(text: &[char], i: usize, content: SpecialContentType) -> bool {
    let tag = match content {
        SpecialContentType::Script => "script",
        SpecialContentType::Style => "style",
        SpecialContentType::Svg => "svg",
        SpecialContentType::None => return false,
    };
    is_closing_special_tag(text, i, tag)
}

/// True if `text[i..]` starts `</tag>` (not inside a JS/CSS string literal).
pub(super) fn is_closing_special_tag(text: &[char], i: usize, tag_name: &str) -> bool {
    let n = tag_name.chars().count();
    let needed = 3 + n;
    if i + needed > text.len() {
        return false;
    }
    if text[i] != '<' || text[i + 1] != '/' {
        return false;
    }
    let name: String = text[i + 2..i + 2 + n].iter().collect();
    if !crate::text::eq_ignore_case(&name, tag_name) {
        return false;
    }
    if text[i + 2 + n] != '>' {
        return false;
    }
    if i > 0 && (text[i - 1] == '"' || text[i - 1] == '`') {
        return false;
    }
    true
}

/// `builder` equals `keyword` (already lower case) after per-char lowering.
pub(super) fn is_keyword_builder(builder: &[char], keyword: &str) -> bool {
    let k: Vec<char> = keyword.chars().collect();
    if builder.len() != k.len() {
        return false;
    }
    builder
        .iter()
        .zip(k.iter())
        .all(|(b, kc)| b.to_lowercase().next() == Some(*kc) && b.to_lowercase().count() == 1)
}

pub(super) fn is_continued_condition_builder(builder: &[char]) -> bool {
    is_keyword_builder(builder, "#else") || is_keyword_builder(builder, "#end") || is_keyword_builder(builder, "#md")
}

/// A file path ends at a space followed by the `sep` or `type` keyword.
pub(super) fn is_file_path_end_marker(text: &[char], space_index: usize) -> bool {
    if space_index + 1 >= text.len() {
        return false;
    }
    let remaining = trim_start(&text[space_index + 1..]);
    if remaining.is_empty() {
        return false;
    }
    starts_with_ignore_case(remaining, "sep") || starts_with_ignore_case(remaining, "type")
}

/// True if the line has `name(params) =` from `start_pos` on (and the `=` is not `==`).
pub(super) fn is_function_definition_line(text: &[char], start_pos: i32) -> bool {
    let mut paren_depth = 0;
    let mut found_open = false;
    let start = start_pos.max(0) as usize;
    for i in start..text.len() {
        let c = text[i];
        if c == '\'' || c == '"' {
            return false;
        }
        if c == '(' {
            paren_depth += 1;
            found_open = true;
        } else if c == ')' {
            paren_depth -= 1;
            if found_open && paren_depth == 0 {
                let after = trim_start(&text[i + 1..]);
                return !after.is_empty() && after[0] == '=' && (after.len() == 1 || after[1] != '=');
            }
        }
    }
    false
}
