//! IncludeValidator.cs
use crate::linter::constants::patterns;
use crate::linter::helpers::cs;
use crate::linter::helpers::diagnostic_extensions::LineStage;
use crate::linter::models::{LinterResult, Stage1Context};
use crate::text::{is_white_space, to_string, trim, trim_start};

#[derive(Default)]
pub struct IncludeValidator;

impl IncludeValidator {
    pub fn new() -> Self {
        IncludeValidator
    }

    pub fn validate(&self, context: &Stage1Context, result: &mut LinterResult) {
        for (i, line) in context.lines.iter().enumerate() {
            let i = i as i32;
            let chars = cs::chars(line);
            let trimmed = trim_start(&chars);
            if !cs::starts_with_ic(trimmed, "#include") {
                continue;
            }
            let include_end = cs::index_of_ic(&chars, "#include", 0).map_or(7, |p| p + 8);
            let line_len = chars.len() as i32;

            validate_input_fields(i, &chars, include_end, result);

            if trimmed.len() <= 8 || trim(&trimmed[8..]).is_empty() {
                result.add_error_stage(
                    i,
                    0,
                    line_len,
                    "CPD-1102",
                    "#include requires a file path",
                    LineStage::Stage1,
                );
                continue;
            }

            let Some(m) = patterns::include_statement().captures(line) else {
                result.add_error_stage(
                    i,
                    0,
                    line_len,
                    "CPD-1101",
                    &format!("'{}'", to_string(trim(&chars))),
                    LineStage::Stage1,
                );
                continue;
            };
            let filename = m.get(1).map_or("", |g| g.as_str());
            if filename.chars().all(is_white_space) {
                result.add_error_stage(
                    i,
                    include_end as i32,
                    line_len,
                    "CPD-1102",
                    "#include requires a file path",
                    LineStage::Stage1,
                );
                continue;
            }
        }
    }
}

fn validate_input_fields(line_index: i32, span: &[char], after_keyword: usize, result: &mut LinterResult) {
    let Some(rel) = cs::last_index_of(&span[after_keyword..], "#{") else {
        return;
    };
    let marker = rel + after_keyword;
    let close = span[marker..].iter().position(|c| *c == '}');
    let end = match close {
        None => span.len(),
        Some(c) => marker + c + 1,
    };
    result.add_warning_stage(
        line_index,
        marker as i32,
        end as i32,
        "CPD-1103",
        &format!(
            "'{}' - declare the inputs with #UI instead",
            to_string(&span[marker..end])
        ),
        LineStage::Stage1,
    );
}
