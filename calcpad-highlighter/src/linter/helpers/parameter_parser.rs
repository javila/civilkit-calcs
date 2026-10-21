//! ParameterParser.cs
use crate::text::trim;
use crate::tokenizer::models::Token;

fn is_blank(s: &str) -> bool {
    s.chars().all(crate::text::is_white_space)
}

/// Splits `content` on `delimiter` outside (), {} and [] and trims each segment.
pub fn split_by_delimiter(content: &str, delimiter: char) -> Vec<String> {
    let span: Vec<char> = content.chars().collect();
    split_chars_by_delimiter(&span, delimiter)
}

pub fn split_chars_by_delimiter(span: &[char], delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut seg_start = 0;
    let (mut paren, mut brace, mut bracket) = (0i32, 0i32, 0i32);
    for (i, &c) in span.iter().enumerate() {
        match c {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ => {}
        }
        if c == delimiter && paren == 0 && brace == 0 && bracket == 0 {
            result.push(crate::text::to_string(trim(&span[seg_start..i])));
            seg_start = i + 1;
        }
    }
    if seg_start < span.len() || !result.is_empty() {
        result.push(crate::text::to_string(trim(&span[seg_start..])));
    }
    result
}

/// `ParseParameters(paramsStr, lineTokens, paramStartCol, tokenGroups)`. With `None` token
/// arguments it is `SplitByDelimiter(paramsStr, ';')`.
pub fn parse_parameters(
    params_str: &str,
    line_tokens: Option<&[Token]>,
    param_start_col: i32,
    token_groups: Option<&mut Vec<Vec<Token>>>,
) -> Vec<String> {
    if is_blank(params_str) {
        return Vec::new();
    }
    let (line_tokens, token_groups) = match (line_tokens, token_groups) {
        (Some(l), Some(g)) => (l, g),
        _ => return split_by_delimiter(params_str, ';'),
    };
    let span: Vec<char> = params_str.chars().collect();
    let mut parameters = Vec::new();
    let mut boundaries: Vec<(i32, i32)> = Vec::new();
    let mut seg_start = 0usize;
    let (mut paren, mut brace, mut bracket) = (0i32, 0i32, 0i32);
    for (i, &c) in span.iter().enumerate() {
        match c {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ => {}
        }
        if c == ';' && paren == 0 && brace == 0 && bracket == 0 {
            parameters.push(crate::text::to_string(trim(&span[seg_start..i])));
            boundaries.push((param_start_col + seg_start as i32, param_start_col + i as i32));
            seg_start = i + 1;
        }
    }
    if seg_start < span.len() || !parameters.is_empty() {
        parameters.push(crate::text::to_string(trim(&span[seg_start..])));
        boundaries.push((param_start_col + seg_start as i32, param_start_col + span.len() as i32));
    }

    let mut b_idx = 0usize;
    let mut group: Vec<Token> = Vec::new();
    for token in line_tokens {
        if b_idx >= boundaries.len() {
            break;
        }
        if token.column < boundaries[b_idx].0 {
            continue;
        }
        while b_idx < boundaries.len() && token.column >= boundaries[b_idx].1 {
            token_groups.push(std::mem::take(&mut group));
            b_idx += 1;
        }
        if b_idx < boundaries.len() && token.column >= boundaries[b_idx].0 && token.column < boundaries[b_idx].1 {
            group.push(token.clone());
        }
    }
    if b_idx < boundaries.len() {
        token_groups.push(group);
    }
    while token_groups.len() < parameters.len() {
        token_groups.push(Vec::new());
    }
    parameters
}

pub fn count_parameters(params_str: &str) -> i32 {
    if is_blank(params_str) {
        return 0;
    }
    let mut count = 1;
    let (mut paren, mut brace, mut bracket) = (0i32, 0i32, 0i32);
    for c in params_str.chars() {
        match c {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            ';' if paren == 0 && brace == 0 && bracket == 0 => count += 1,
            _ => {}
        }
    }
    count
}

/// Splits macro parameters on `;` outside parentheses. A blank string gives one empty parameter.
pub fn parse_macro_parameters(params_str: &str) -> Vec<String> {
    if is_blank(params_str) {
        return vec![String::new()];
    }
    let span: Vec<char> = params_str.chars().collect();
    let mut result = Vec::new();
    let mut seg_start = 0;
    let mut paren = 0i32;
    for (i, &c) in span.iter().enumerate() {
        if c == '(' {
            paren += 1;
        } else if c == ')' {
            paren -= 1;
        }
        if c == ';' && paren == 0 {
            result.push(crate::text::to_string(trim(&span[seg_start..i])));
            seg_start = i + 1;
        }
    }
    if seg_start < span.len() || !result.is_empty() {
        result.push(crate::text::to_string(trim(&span[seg_start..])));
    }
    result
}
