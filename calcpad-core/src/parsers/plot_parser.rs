//! Port of Parsers/PlotParser.cs: the shared part of the $Plot and $Map parsers.
use crate::extension_methods::trim;

pub const PARTS: [&str; 4] = ["function", "variable name", "left margin", "right margin"];

/// Splits a plot command at `delimiters` (outside nested braces), as the C# parsers do
/// with a TextSpan. Returns the parts found and the number of delimiters seen.
pub fn split_command(script: &str, delimiters: &[char], n: usize) -> (Vec<Option<String>>, usize) {
    let chars: Vec<char> = script.chars().collect();
    let mut input: Vec<Option<String>> = vec![None; n];
    let mut index = 0usize;
    let mut bracket_count = 0i32;
    let (mut start, mut end) = (0usize, 0usize);
    let mut del = delimiters[0];
    for (i, &c) in chars.iter().enumerate() {
        if c == '{' {
            bracket_count += 1;
        }
        if c == del && bracket_count == 1 {
            if index > 0 {
                let s: String = chars[start..end].iter().collect();
                input[index - 1] = Some(trim(&s).to_string());
            }
            index += 1;
            if index > n {
                break;
            }
            del = delimiters[index];
            start = i + 1;
            end = i + 1;
        } else {
            end += 1;
        }
        if c == '}' {
            bracket_count -= 1;
        }
    }
    (input, index)
}

/// `string.IsNullOrWhiteSpace`
pub fn is_null_or_white_space(s: &Option<String>) -> bool {
    match s {
        None => true,
        Some(s) => s.chars().all(char::is_whitespace),
    }
}
