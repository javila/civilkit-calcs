//! CalcpadPrettifier.cs
use super::prettifier_options::PrettifierOptions;
use crate::linter::constants::builtins::{get_block_type, ControlBlockType};
use crate::linter::models::ParseMode;
use crate::text::{to_string, trim};
use crate::tokenizer::parse_mode_tracker::ParseModeTracker;

/// Re-indents Calcpad source by tracking control-block depth across `#if`/`#else`/`#end if`,
/// `#for`/`#while`/`#repeat`/`#loop`, and multiline `#def`/`#end def`, where inline `#def name = ...`
/// does not open a block. Only leading whitespace is adjusted; line content, comments and the
/// original line-ending style are preserved. #html/#markdown content lines are left untouched.
pub struct CalcpadPrettifier;

impl CalcpadPrettifier {
    pub fn prettify(source: &str, options: Option<&PrettifierOptions>) -> String {
        if source.is_empty() {
            return String::new();
        }

        let default_options = PrettifierOptions::default();
        let options = options.unwrap_or(&default_options);
        let indent_unit = options.indent_unit.as_deref().unwrap_or("\t");
        let src: Vec<char> = source.chars().collect();
        let mut sb = String::with_capacity(source.len());
        let mut depth: usize = 0;
        let mut pos = 0usize;
        let mut parse_modes = ParseModeTracker::new();

        while pos < src.len() {
            // End of the line, and its line ending so it can be preserved
            let line_start = pos;
            while pos < src.len() && src[pos] != '\n' && src[pos] != '\r' {
                pos += 1;
            }
            let content_end = pos;

            let mut line_ending = "";
            if pos < src.len() {
                if src[pos] == '\r' && pos + 1 < src.len() && src[pos + 1] == '\n' {
                    line_ending = "\r\n";
                    pos += 2;
                } else {
                    line_ending = if src[pos] == '\r' { "\r" } else { "\n" };
                    pos += 1;
                }
            }

            let raw_line = &src[line_start..content_end];
            let trimmed_span = trim(raw_line);

            // Leading whitespace is significant in markdown, and HTML keeps the author's layout
            if parse_modes.mode() != ParseMode::Cpd && !ParseModeTracker::is_directive(trimmed_span, parse_modes.mode())
            {
                sb.push_str(&to_string(raw_line));
                sb.push_str(line_ending);
                continue;
            }
            parse_modes.apply(trimmed_span);

            let mut trimmed = to_string(trimmed_span);
            if options.trim_trailing_whitespace {
                trimmed = trim_trailing_whitespace(&trimmed).to_string();
            }

            if trimmed.is_empty() {
                sb.push_str(line_ending);
                continue;
            }

            let block_type = get_block_type(&trimmed);
            let render_depth;

            match block_type {
                // Block enders: dedent first, render at the new (outer) depth.
                ControlBlockType::EndIf | ControlBlockType::Loop | ControlBlockType::EndDef => {
                    depth = depth.saturating_sub(1);
                    render_depth = depth;
                }
                // Mid-block keywords: visually dedent for this line, keep the inner depth after it.
                ControlBlockType::Else | ControlBlockType::ElseIf => {
                    render_depth = depth.saturating_sub(1);
                }
                // Block starters: render at the current depth, then indent the following lines.
                ControlBlockType::If | ControlBlockType::Repeat | ControlBlockType::For | ControlBlockType::While => {
                    render_depth = depth;
                    depth += 1;
                }
                // #def is a starter only when multiline. The inline form (#def name = ... or
                // #def f(x) = ...) contains '=' and does not open a block (like BalanceValidator).
                ControlBlockType::Def => {
                    render_depth = depth;
                    if !trimmed.contains('=') {
                        depth += 1;
                    }
                }
                _ => render_depth = depth,
            }

            for _ in 0..render_depth {
                sb.push_str(indent_unit);
            }
            sb.push_str(&trimmed);
            sb.push_str(line_ending);
        }

        sb
    }
}

/// Trims trailing spaces and tabs only.
fn trim_trailing_whitespace(s: &str) -> &str {
    s.trim_end_matches([' ', '\t'])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> String {
        CalcpadPrettifier::prettify(s, None)
    }

    #[test]
    fn indents_control_blocks() {
        let src = "#if a > 1\nx = 1\n#else\nx = 2\n#end if\n";
        assert_eq!(p(src), "#if a > 1\n\tx = 1\n#else\n\tx = 2\n#end if\n");
        let src = "#for i = 1 : 3\n#if i > 1\ny = i\n#end if\n#loop";
        assert_eq!(p(src), "#for i = 1 : 3\n\t#if i > 1\n\t\ty = i\n\t#end if\n#loop");
    }

    #[test]
    fn def_inline_does_not_open_a_block() {
        assert_eq!(p("#def a$ = 1\nx = a$\n"), "#def a$ = 1\nx = a$\n");
        assert_eq!(p("#def a$\n  x = 1\n#end def\n"), "#def a$\n\tx = 1\n#end def\n");
    }

    #[test]
    fn preserves_line_endings_and_blank_lines() {
        assert_eq!(
            p("#if a\r\n\r\n   x   \r\n#end if\r\n"),
            "#if a\r\n\r\n\tx\r\n#end if\r\n"
        );
        assert_eq!(p("a\rb"), "a\rb");
    }

    #[test]
    fn html_and_markdown_content_is_untouched() {
        let src = "#html\n   <p>  \n#end html\n   x = 1  \n";
        assert_eq!(p(src), "#html\n   <p>  \n#end html\nx = 1\n");
    }

    #[test]
    fn options_and_unbalanced_enders_and_the_trim_option_is_moot() {
        let o = PrettifierOptions {
            indent_unit: Some("  ".to_string()),
            trim_trailing_whitespace: false,
        };
        assert_eq!(
            CalcpadPrettifier::prettify("#if a\nx  \n#end if\n#end if", Some(&o)),
            "#if a\n  x\n#end if\n#end if"
        );
        assert_eq!(p(""), "");
        assert_eq!(p("#else\nx"), "#else\nx");
    }
}
