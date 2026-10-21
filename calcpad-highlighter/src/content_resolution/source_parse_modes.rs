//! SourceParseModes.cs: the parse mode (#html, #markdown) of each source line.
use super::result::StagedResolvedContent;
use crate::linter::models::ParseMode;
use crate::text::trim;
use crate::tokenizer::parse_mode_tracker::ParseModeTracker;

/// A run of source lines, inclusive, in #html or #markdown mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseModeRange {
    pub start_line: i32,
    pub end_line: i32,
    pub mode: ParseMode,
}

pub struct SourceParseModes;

impl SourceParseModes {
    /// Parse mode of each source line, read from the macro-expanded Stage 3 content so that a macro or
    /// #include switching mode counts, as it does for Core. A line takes the mode in effect before its
    /// own content; Calcpad lines are omitted from the result.
    pub fn build(staged: &StagedResolvedContent, source_line_count: i32) -> Vec<ParseModeRange> {
        let count = source_line_count.max(0) as usize;
        let mut before: Vec<Option<ParseMode>> = vec![None; count];
        let mut after: Vec<Option<ParseMode>> = vec![None; count];
        let mut tracker = ParseModeTracker::new();
        let stage3 = &staged.stage3;
        for (i, text) in stage3.lines.iter().enumerate() {
            let mode_before = tracker.mode();
            let chars: Vec<char> = text.chars().collect();
            tracker.apply(trim(&chars));
            let line = to_source_line(i as i32, staged);
            if line < 0 || line as usize >= count {
                continue;
            }
            let line = line as usize;
            if before[line].is_none() {
                before[line] = Some(mode_before);
            }
            after[line] = Some(tracker.mode());
        }

        // Lines missing from Stage 3, such as #def bodies, carry the mode left by the line above
        let mut ranges = Vec::new();
        let mut carry = ParseMode::Cpd;
        let mut start = 0i32;
        let mut current = ParseMode::Cpd;
        for line in 0..count {
            let mode = before[line].unwrap_or(carry);
            carry = after[line].unwrap_or(carry);
            if mode == current {
                continue;
            }

            if current != ParseMode::Cpd {
                ranges.push(ParseModeRange {
                    start_line: start,
                    end_line: line as i32 - 1,
                    mode: current,
                });
            }

            start = line as i32;
            current = mode;
        }
        if current != ParseMode::Cpd {
            ranges.push(ParseModeRange {
                start_line: start,
                end_line: source_line_count - 1,
                mode: current,
            });
        }

        ranges
    }
}

fn to_source_line(stage3_line: i32, staged: &StagedResolvedContent) -> i32 {
    let stage2_line = staged
        .stage3
        .source_map
        .get(&stage3_line)
        .copied()
        .unwrap_or(stage3_line);
    let stage1_line = staged
        .stage2
        .source_map
        .get(&stage2_line)
        .copied()
        .unwrap_or(stage2_line);
    staged
        .stage1
        .source_map
        .get(&stage1_line)
        .copied()
        .unwrap_or(stage1_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staged(lines: &[&str]) -> StagedResolvedContent {
        // Identity maps: every stage keeps the line numbers
        let mut s = StagedResolvedContent::default();
        s.stage3.lines = lines.iter().map(|l| l.to_string()).collect();
        s
    }

    #[test]
    fn html_block_range_starts_at_the_opener_line() {
        // The opener line itself is Cpd (mode before its content); the range starts on the next line
        let r = SourceParseModes::build(&staged(&["x = 1", "#html", "<p>", "#end html", "y = 2"]), 5);
        assert_eq!(
            r,
            vec![ParseModeRange {
                start_line: 2,
                end_line: 3,
                mode: ParseMode::Html
            }]
        );
    }

    #[test]
    fn open_ended_block_runs_to_the_last_line() {
        let r = SourceParseModes::build(&staged(&["#markdown", "text", "more"]), 3);
        assert_eq!(
            r,
            vec![ParseModeRange {
                start_line: 1,
                end_line: 2,
                mode: ParseMode::Markdown
            }]
        );
    }

    #[test]
    fn lines_missing_from_stage3_carry_the_previous_mode() {
        // Source line 2 (a #def body) is not in stage 3; it carries the mode of the line above
        let mut s = staged(&["#html", "<b>", "<i>", "#end html"]);
        s.stage3.source_map.insert(2, 3);
        s.stage3.source_map.insert(3, 4);
        let r = SourceParseModes::build(&s, 5);
        assert_eq!(
            r,
            vec![ParseModeRange {
                start_line: 1,
                end_line: 4,
                mode: ParseMode::Html
            }]
        );
    }

    #[test]
    fn calcpad_only_content_has_no_ranges() {
        assert!(SourceParseModes::build(&staged(&["a = 1", "b = 2"]), 2).is_empty());
        assert!(SourceParseModes::build(&staged(&[]), 0).is_empty());
    }
}
