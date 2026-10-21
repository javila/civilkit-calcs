//! ContentResolver.Stage1.cs: line continuation processing.
use super::result::{LineContinuationSegment, Stage1Result};
use super::ContentResolver;
use crate::linter::models::ParseMode;
use crate::parsing::char_classifier::is_line_extension;
use crate::text::{is_white_space, trim};
use crate::tokenizer::parse_mode_tracker::ParseModeTracker;
use std::collections::BTreeMap;

/// The result of `GetLineContinuationWithDepth`.
pub(crate) struct LineContinuation {
    pub explicit_continuation: bool,
    pub implicit_continuation: bool,
    /// The line without the trailing " _" for an explicit continuation, otherwise the line.
    pub content: String,
    pub paren_depth: i32,
    pub bracket_depth: i32,
    pub brace_depth: i32,
}

impl ContentResolver {
    /// STAGE 1: process line continuations only.
    ///
    /// A line continues when it ends with " _" (explicit), or when it ends with one of `;|&@:({[`
    /// outside a comment and its brackets, braces or parentheses are unbalanced (implicit).
    pub(crate) fn process_stage1(lines: &[String]) -> Stage1Result {
        let mut processed_lines: Vec<String> = Vec::new();
        let mut source_map: BTreeMap<i32, i32> = BTreeMap::new();
        let mut line_continuation_map: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        let mut line_continuation_segments: BTreeMap<i32, Vec<LineContinuationSegment>> = BTreeMap::new();

        let mut parse_modes = ParseModeTracker::new();
        let mut i = 0usize;
        while i < lines.len() {
            let line = &lines[i];
            let lc = get_line_continuation_with_depth(line);

            let has_unbalanced = lc.paren_depth > 0 || lc.bracket_depth > 0 || lc.brace_depth > 0;
            let mut should_continue = lc.explicit_continuation || (lc.implicit_continuation && has_unbalanced);

            // HTML/markdown content never continues, e.g. CSS lines ending in ';'
            let line_chars: Vec<char> = line.chars().collect();
            let trimmed = trim(&line_chars);
            if parse_modes.mode() != ParseMode::Cpd && !ParseModeTracker::is_directive(trimmed, parse_modes.mode()) {
                should_continue = false;
            } else {
                parse_modes.apply(trimmed);
            }

            if should_continue && i + 1 < lines.len() {
                let mut continued_lines: Vec<i32> = vec![i as i32];
                let mut segments: Vec<LineContinuationSegment> = Vec::new();

                let mut full_line = lc.content.clone();
                let mut full_len = lc.content.chars().count();

                segments.push(LineContinuationSegment {
                    original_line: i as i32,
                    start_column: 0,
                    length: full_len as i32,
                });

                let mut current_paren_depth = lc.paren_depth;
                let mut current_bracket_depth = lc.bracket_depth;
                let mut current_brace_depth = lc.brace_depth;

                let mut j = i + 1;
                while j < lines.len() {
                    let next_line = &lines[j];
                    continued_lines.push(j as i32);

                    let next = get_line_continuation_with_depth(next_line);

                    current_paren_depth += next.paren_depth;
                    current_bracket_depth += next.bracket_depth;
                    current_brace_depth += next.brace_depth;

                    let still_unbalanced =
                        current_paren_depth > 0 || current_bracket_depth > 0 || current_brace_depth > 0;
                    let still_continuing =
                        next.explicit_continuation || (next.implicit_continuation && still_unbalanced);

                    if still_continuing && j + 1 < lines.len() {
                        let content_len = next.content.chars().count();
                        segments.push(LineContinuationSegment {
                            original_line: j as i32,
                            start_column: full_len as i32,
                            length: content_len as i32,
                        });
                        full_line.push_str(&next.content);
                        full_len += content_len;
                        j += 1;
                    } else {
                        // Last line in continuation (balanced or no continuation marker)
                        let line_len = next_line.chars().count();
                        segments.push(LineContinuationSegment {
                            original_line: j as i32,
                            start_column: full_len as i32,
                            length: line_len as i32,
                        });
                        full_line.push_str(next_line);
                        break;
                    }
                }

                processed_lines.push(full_line);
                let new_index = (processed_lines.len() - 1) as i32;
                source_map.insert(new_index, continued_lines[0]);
                line_continuation_map.insert(new_index, continued_lines);
                line_continuation_segments.insert(new_index, segments);
                i = j + 1;
            } else {
                processed_lines.push(line.clone());
                source_map.insert((processed_lines.len() - 1) as i32, i as i32);
                i += 1;
            }
        }

        Stage1Result {
            lines: processed_lines,
            source_map,
            line_continuation_map,
            line_continuation_segments,
        }
    }
}

/// `GetLineContinuationWithDepth`: continuation flags and bracket depths of one line, using
/// lightweight comment detection instead of tokenizing. Brackets are counted outside comments only.
pub(crate) fn get_line_continuation_with_depth(line: &str) -> LineContinuation {
    let none = |content: &str| LineContinuation {
        explicit_continuation: false,
        implicit_continuation: false,
        content: content.to_string(),
        paren_depth: 0,
        bracket_depth: 0,
        brace_depth: 0,
    };
    if line.is_empty() {
        return none(line);
    }

    let span: Vec<char> = line.chars().collect();
    let len = span.len();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut brace_depth = 0;

    // Last non-whitespace character position
    let mut last_char_pos = len as isize - 1;
    while last_char_pos >= 0 && is_white_space(span[last_char_pos as usize]) {
        last_char_pos -= 1;
    }
    if last_char_pos < 0 {
        return none(line);
    }
    let last_char_pos = last_char_pos as usize;

    // A comment starts with ' or " and extends to the matching quote or the end of the line.
    let mut in_comment = false;
    let mut comment_char = '\0';
    let mut comment_start_col: isize = -1;

    for (ci, &c) in span.iter().enumerate() {
        if !in_comment {
            if c == '\'' || c == '"' {
                in_comment = true;
                comment_char = c;
                if comment_start_col < 0 {
                    comment_start_col = ci as isize;
                }
            } else if c == '(' {
                paren_depth += 1;
            } else if c == ')' {
                paren_depth -= 1;
            } else if c == '[' {
                bracket_depth += 1;
            } else if c == ']' {
                bracket_depth -= 1;
            } else if c == '{' {
                brace_depth += 1;
            } else if c == '}' {
                brace_depth -= 1;
            }
        } else if c == comment_char {
            in_comment = false;
            comment_char = '\0';
        }
    }

    // Explicit continuation (line ends with " _") works in both code and comments
    if last_char_pos >= 1 && span[last_char_pos] == '_' && span[last_char_pos - 1] == ' ' {
        let content: String = span[..last_char_pos - 1].iter().collect();
        return LineContinuation {
            explicit_continuation: true,
            implicit_continuation: false,
            content,
            paren_depth,
            bracket_depth,
            brace_depth,
        };
    }

    // If the end is in a comment, no implicit continuation is allowed
    let end_is_in_comment = in_comment && comment_start_col >= 0 && last_char_pos as isize >= comment_start_col;
    if end_is_in_comment {
        return LineContinuation {
            explicit_continuation: false,
            implicit_continuation: false,
            content: line.to_string(),
            paren_depth,
            bracket_depth,
            brace_depth,
        };
    }

    let has_implicit_continuation = is_line_extension(span[last_char_pos]);
    LineContinuation {
        explicit_continuation: false,
        implicit_continuation: has_implicit_continuation,
        content: line.to_string(),
        paren_depth,
        bracket_depth,
        brace_depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn explicit_continuation_merges_and_maps_columns() {
        let r = ContentResolver::process_stage1(&lines(&["a = 1 + _", "2 + _", "3", "b = 4"]));
        assert_eq!(r.lines, vec!["a = 1 +2 +3".to_string(), "b = 4".to_string()]);
        assert_eq!(r.source_map[&0], 0);
        assert_eq!(r.source_map[&1], 3);
        assert_eq!(r.line_continuation_map[&0], vec![0, 1, 2]);
        let seg = &r.line_continuation_segments[&0];
        assert_eq!(seg.len(), 3);
        assert_eq!((seg[0].original_line, seg[0].start_column, seg[0].length), (0, 0, 7));
        assert_eq!((seg[1].original_line, seg[1].start_column, seg[1].length), (1, 7, 3));
        assert_eq!((seg[2].original_line, seg[2].start_column, seg[2].length), (2, 10, 1));
        assert!(!r.line_continuation_map.contains_key(&1));
    }

    #[test]
    fn implicit_continuation_needs_unbalanced_brackets() {
        let r = ContentResolver::process_stage1(&lines(&["f(a;", "b)", "x = 1;", "y = 2"]));
        assert_eq!(
            r.lines,
            vec!["f(a;b)".to_string(), "x = 1;".to_string(), "y = 2".to_string()]
        );
        assert_eq!(r.source_map[&2], 3);
    }

    #[test]
    fn a_comment_never_continues_implicitly_but_does_explicitly() {
        let r = ContentResolver::process_stage1(&lines(&["'text (;", "b"]));
        assert_eq!(r.lines.len(), 2);
        let r = ContentResolver::process_stage1(&lines(&["'text _", "b"]));
        assert_eq!(r.lines, vec!["'textb".to_string()]);
    }

    #[test]
    fn html_content_does_not_continue() {
        let r = ContentResolver::process_stage1(&lines(&[
            "#html",
            "a { color: red; _",
            "b",
            "#end html",
            "x = (1;",
            "2)",
        ]));
        assert_eq!(r.lines.len(), 5);
        assert_eq!(r.lines[4], "x = (1;2)");
    }

    #[test]
    fn the_last_line_never_continues() {
        let r = ContentResolver::process_stage1(&lines(&["a = 1 _"]));
        assert_eq!(r.lines, vec!["a = 1 _".to_string()]);
        let r = ContentResolver::process_stage1(&lines(&["a = 1 + _", "2 + _"]));
        assert_eq!(r.lines, vec!["a = 1 +2 + _".to_string()]);
    }

    #[test]
    fn depth_counting_ignores_comments() {
        let lc = get_line_continuation_with_depth("x = (1 'note (' + [2");
        assert_eq!((lc.paren_depth, lc.bracket_depth, lc.brace_depth), (1, 1, 0));
        assert!(!lc.implicit_continuation);
        let lc = get_line_continuation_with_depth("   ");
        assert!(!lc.explicit_continuation && !lc.implicit_continuation);
    }
}
