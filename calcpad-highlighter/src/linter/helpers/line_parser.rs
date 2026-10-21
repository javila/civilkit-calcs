//! LineParser.cs
use super::cs;
use crate::text::is_white_space;

#[derive(Clone, Debug, Default)]
pub struct CodeSegment {
    pub text: String,
    pub start_pos: i32,
    pub line_number: i32,
}

#[derive(Clone, Debug, Default)]
pub struct StringSegment {
    pub text: String,
    pub start_pos: i32,
    pub end_pos: i32,
    pub line_number: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedLine {
    pub code_segments: Vec<CodeSegment>,
    pub string_segments: Vec<StringSegment>,
    pub line_number: i32,
    pub original_line: String,
}

pub fn parse_line(line: &str, line_number: i32) -> ParsedLine {
    let mut result = ParsedLine {
        line_number,
        original_line: line.to_string(),
        ..Default::default()
    };
    if line.is_empty() {
        return result;
    }
    let chars: Vec<char> = line.chars().collect();
    let mut in_single = false;
    let mut in_double = false;
    let mut segment_start = 0i32;
    let mut text = String::new();

    for (i, &c) in chars.iter().enumerate() {
        let i = i as i32;
        if c == '\'' && !in_double {
            if !in_single {
                if !text.is_empty() {
                    result.code_segments.push(CodeSegment {
                        text: text.clone(),
                        start_pos: segment_start,
                        line_number,
                    });
                }
                in_single = true;
                segment_start = i;
                text.clear();
                text.push(c);
            } else {
                text.push(c);
                result.string_segments.push(StringSegment {
                    text: text.clone(),
                    start_pos: segment_start,
                    end_pos: i + 1,
                    line_number,
                });
                in_single = false;
                segment_start = i + 1;
                text.clear();
            }
        } else if c == '"' && !in_single {
            if !in_double {
                if !text.is_empty() {
                    result.code_segments.push(CodeSegment {
                        text: text.clone(),
                        start_pos: segment_start,
                        line_number,
                    });
                }
                in_double = true;
                segment_start = i;
                text.clear();
                text.push(c);
            } else {
                text.push(c);
                result.string_segments.push(StringSegment {
                    text: text.clone(),
                    start_pos: segment_start,
                    end_pos: i + 1,
                    line_number,
                });
                in_double = false;
                segment_start = i + 1;
                text.clear();
            }
        } else {
            text.push(c);
        }
    }

    if !text.is_empty() {
        if in_single || in_double {
            result.string_segments.push(StringSegment {
                text,
                start_pos: segment_start,
                end_pos: chars.len() as i32,
                line_number,
            });
        } else {
            result.code_segments.push(CodeSegment {
                text,
                start_pos: segment_start,
                line_number,
            });
        }
    }
    result
}

pub fn is_comment_line(line: &str) -> bool {
    match cs::trim_start_str(line).chars().next() {
        Some(c) => c == '\'' || c == '"',
        None => false,
    }
}

pub fn is_empty_or_whitespace(line: &str) -> bool {
    cs::is_blank(line)
}

/// Whether the line should be skipped during validation (empty, whitespace, or comment).
pub fn should_skip_line(line: &str) -> bool {
    if cs::is_blank(line) {
        return true;
    }
    is_comment_line(line)
}

pub fn is_directive_line(trimmed_line: &[char]) -> bool {
    trimmed_line.first() == Some(&'#')
}

pub fn is_directive_line_str(trimmed_line: &str) -> bool {
    trimmed_line.starts_with('#')
}

/// Directives whose remainder is an expression, longest keyword first.
const EXPRESSION_DIRECTIVES: [&str; 17] = [
    "#else if", "#varsub", "#nosub", "#novar", "#while", "#repeat", "#round", "#post", "#hide", "#show", "#pre",
    "#val", "#equ", "#noc", "#for", "#if", "#ui",
];

/// The column where a directive's trailing expression starts, or -1.
pub fn get_directive_expression_start(line: &[char]) -> i32 {
    let mut i = 0;
    while i < line.len() && is_white_space(line[i]) {
        i += 1;
    }
    let rest = &line[i..];
    for directive in EXPRESSION_DIRECTIVES {
        if !cs::starts_with_ic(rest, directive) {
            continue;
        }
        let mut after = i + directive.chars().count();
        if after < line.len() && !is_white_space(line[after]) && line[after] != '{' {
            continue;
        }
        while after < line.len() && is_white_space(line[after]) {
            after += 1;
        }
        if after < line.len() && line[after] == '{' && directive.eq_ignore_ascii_case("#ui") {
            let Some(rel) = line[after..].iter().position(|c| *c == '}') else {
                return -1;
            };
            after += rel + 1;
            while after < line.len() && is_white_space(line[after]) {
                after += 1;
            }
        }
        if after >= line.len() {
            return -1;
        }
        return if cs::equals_ic(crate::text::trim(&line[after..]), "default") {
            -1
        } else {
            after as i32
        };
    }
    -1
}

pub fn is_def_statement(trimmed_line: &[char]) -> bool {
    cs::starts_with_ic(trimmed_line, "#def ") || cs::equals_ic(trimmed_line, "#def")
}

pub fn is_def_statement_str(trimmed_line: &str) -> bool {
    is_def_statement(&cs::chars(trimmed_line))
}

pub fn is_end_def_statement(trimmed_line: &[char]) -> bool {
    cs::equals_ic(trimmed_line, "#end def")
}

pub fn is_end_def_statement_str(trimmed_line: &str) -> bool {
    is_end_def_statement(&cs::chars(trimmed_line))
}
