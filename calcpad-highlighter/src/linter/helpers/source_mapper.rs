//! SourceMapper.cs
use crate::content_resolution::result::LineContinuationSegment;
use crate::linter::models::{
    DiagnosticRange, LinterDiagnostic, LinterSeverity, Stage1Context, Stage2Context, Stage3Context,
};

/// Result of mapping a position from a processed stage line back to the original source.
#[derive(Clone, Debug, Default)]
pub struct MappedPosition {
    pub line: i32,
    pub column: i32,
    pub end_column: i32,
    /// (Line, Column, EndColumn) of further ranges when the diagnostic spans several original lines.
    pub additional_ranges: Option<Vec<(i32, i32, i32)>>,
}

pub fn map_stage1_to_original(stage1_line: i32, stage1: &Stage1Context) -> i32 {
    stage1.source_map.get(&stage1_line).copied().unwrap_or(stage1_line)
}

pub fn map_stage2_to_original(stage2_line: i32, stage2: &Stage2Context, stage1: &Stage1Context) -> i32 {
    let s1 = stage2
        .stage2_to_stage1_map
        .get(&stage2_line)
        .copied()
        .unwrap_or(stage2_line);
    stage1.source_map.get(&s1).copied().unwrap_or(s1)
}

pub fn map_stage3_to_original(
    stage3_line: i32,
    stage3: &Stage3Context,
    stage2: &Stage2Context,
    stage1: &Stage1Context,
) -> i32 {
    let s2 = stage3
        .stage3_to_stage2_map
        .get(&stage3_line)
        .copied()
        .unwrap_or(stage3_line);
    let s1 = stage2.stage2_to_stage1_map.get(&s2).copied().unwrap_or(s2);
    stage1.source_map.get(&s1).copied().unwrap_or(s1)
}

/// Maps a position from a Stage 3 line back to the original source, handling line continuations.
pub fn map_position_to_original(
    stage3_line: i32,
    column: i32,
    end_column: i32,
    stage3: &Stage3Context,
    stage2: &Stage2Context,
    stage1: &Stage1Context,
) -> MappedPosition {
    let stage2_line = stage3
        .stage3_to_stage2_map
        .get(&stage3_line)
        .copied()
        .unwrap_or(stage3_line);
    let stage1_line = stage2
        .stage2_to_stage1_map
        .get(&stage2_line)
        .copied()
        .unwrap_or(stage2_line);

    if stage3.macro_expansions.contains_key(&stage3_line) {
        return map_whole_line_to_original(stage1_line, stage1);
    }

    if let Some(segments) = stage1.line_continuation_segments.get(&stage1_line) {
        if segments.len() > 1 {
            return map_position_with_continuations(column, end_column, segments);
        }
    }

    let original = stage1.source_map.get(&stage1_line).copied().unwrap_or(stage1_line);
    MappedPosition {
        line: original,
        column,
        end_column,
        additional_ranges: None,
    }
}

/// Columns are Unicode scalar counts throughout the port (the JSON layer converts to UTF-16).
fn utf16_len(s: &str) -> i32 {
    s.chars().count() as i32
}

fn map_whole_line_to_original(stage1_line: i32, stage1: &Stage1Context) -> MappedPosition {
    if let Some(segments) = stage1.line_continuation_segments.get(&stage1_line) {
        if !segments.is_empty() {
            let additional = if segments.len() > 1 {
                Some(segments[1..].iter().map(|s| (s.original_line, 0, s.length)).collect())
            } else {
                None
            };
            return MappedPosition {
                line: segments[0].original_line,
                column: 0,
                end_column: segments[0].length,
                additional_ranges: additional,
            };
        }
    }
    let original = stage1.source_map.get(&stage1_line).copied().unwrap_or(stage1_line);
    let length = if stage1_line >= 0 && (stage1_line as usize) < stage1.lines.len() {
        utf16_len(&stage1.lines[stage1_line as usize])
    } else {
        0
    };
    MappedPosition {
        line: original,
        column: 0,
        end_column: length,
        additional_ranges: None,
    }
}

fn map_position_with_continuations(
    column: i32,
    end_column: i32,
    segments: &[LineContinuationSegment],
) -> MappedPosition {
    let mut start_idx: Option<usize> = None;
    let mut end_idx: Option<usize> = None;
    let mut start_offset = 0;
    let mut end_offset = 0;

    for (i, seg) in segments.iter().enumerate() {
        let seg_end = seg.start_column + seg.length;
        if start_idx.is_none() && column >= seg.start_column && column < seg_end {
            start_idx = Some(i);
            start_offset = column - seg.start_column;
        }
        if end_column > seg.start_column && end_column <= seg_end {
            end_idx = Some(i);
            end_offset = end_column - seg.start_column;
        }
    }

    let start_idx = start_idx.unwrap_or_else(|| {
        start_offset = 0;
        0
    });
    let end_idx = end_idx.unwrap_or_else(|| {
        end_offset = segments[segments.len() - 1].length;
        segments.len() - 1
    });
    let start_seg = &segments[start_idx];
    let end_seg = &segments[end_idx];

    if start_seg.original_line == end_seg.original_line {
        return MappedPosition {
            line: start_seg.original_line,
            column: start_offset,
            end_column: end_offset,
            additional_ranges: None,
        };
    }

    // C# compares segment objects by reference, i.e. by position in the list.
    let mut additional = Vec::new();
    let mut in_range = false;
    for (i, seg) in segments.iter().enumerate() {
        if i == start_idx {
            in_range = true;
            continue;
        }
        if in_range {
            if i == end_idx {
                additional.push((seg.original_line, 0, end_offset));
                break;
            } else {
                additional.push((seg.original_line, 0, seg.length));
            }
        }
    }

    MappedPosition {
        line: start_seg.original_line,
        column: start_offset,
        end_column: start_seg.length,
        additional_ranges: Some(additional),
    }
}

/// End column for a diagnostic that should highlight to the end of a continuation segment.
pub fn get_original_line_end_column(
    stage1_line: i32,
    column_in_merged_line: i32,
    stage1: &Stage1Context,
    original_lines: &[String],
) -> i32 {
    let Some(segments) = stage1.line_continuation_segments.get(&stage1_line) else {
        let original = stage1.source_map.get(&stage1_line).copied().unwrap_or(stage1_line);
        if original >= 0 && (original as usize) < original_lines.len() {
            return utf16_len(&original_lines[original as usize]);
        }
        return column_in_merged_line;
    };
    for seg in segments {
        if column_in_merged_line >= seg.start_column && column_in_merged_line < seg.start_column + seg.length {
            return seg.length;
        }
    }
    column_in_merged_line
}

/// Creates a LinterDiagnostic with proper line continuation mapping.
pub fn create_mapped_diagnostic(
    stage3_line: i32,
    column: i32,
    end_column: i32,
    code: &str,
    message: &str,
    severity: LinterSeverity,
    stage3: &Stage3Context,
    stage2: &Stage2Context,
    stage1: &Stage1Context,
) -> LinterDiagnostic {
    let m = map_position_to_original(stage3_line, column, end_column, stage3, stage2, stage1);
    let mut d = LinterDiagnostic {
        line: m.line,
        column: m.column,
        end_column: m.end_column,
        code: code.to_string(),
        message: message.to_string(),
        severity,
        ..Default::default()
    };
    if let Some(r) = m.additional_ranges.filter(|r| !r.is_empty()) {
        d.additional_ranges = Some(
            r.into_iter()
                .map(|(line, column, end_column)| DiagnosticRange {
                    line,
                    column,
                    end_column,
                })
                .collect(),
        );
    }
    d
}
