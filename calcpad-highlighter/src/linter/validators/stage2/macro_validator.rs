//! MacroValidator.cs
use crate::linter::constants::builtins::{self, ControlBlockType};
use crate::linter::constants::patterns;
use crate::linter::helpers::cs;
use crate::linter::helpers::diagnostic_extensions::LineStage;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::parsing_helpers::{parse_param_list, required_after_optional};
use crate::linter::models::{LinterResult, Stage2Context};
use crate::text::{is_letter, StrSet};

#[derive(Default)]
pub struct MacroValidator;

impl MacroValidator {
    pub fn new() -> Self {
        MacroValidator
    }

    pub fn validate(&self, stage2: &Stage2Context, result: &mut LinterResult) {
        validate_duplicate_macros(stage2, result);
        validate_macro_definitions(stage2, result);
    }
}

fn validate_duplicate_macros(stage2: &Stage2Context, result: &mut LinterResult) {
    for d in &stage2.duplicate_macros {
        // C# indexes Lines directly (would throw on a bad index); fall back to 0 length.
        let len = usize::try_from(d.duplicate_line_number)
            .ok()
            .and_then(|i| stage2.lines.get(i))
            .map_or(0, |l| cs::len(l));
        result.add_error_stage(
            d.duplicate_line_number,
            0,
            len,
            "CPD-2201",
            &format!(
                "'{}' was already defined at line {}",
                d.name,
                d.original_line_number + 1
            ),
            LineStage::Stage2,
        );
    }
}

fn validate_macro_definitions(stage2: &Stage2Context, result: &mut LinterResult) {
    let mut in_macro_def = false;
    let mut macro_def_start_line = -1;
    let mut stack: Vec<(ControlBlockType, i32)> = Vec::new();

    for (i, line) in stage2.lines.iter().enumerate() {
        let i = i as i32;
        let trimmed = cs::trim_str(line);
        let line_len = cs::len(line);

        let block_type = builtins::get_block_type(line);
        if builtins::is_block_starter(block_type) && block_type != ControlBlockType::Def {
            stack.push((block_type, i));
        } else if block_type == ControlBlockType::EndIf || block_type == ControlBlockType::Loop {
            if let Some(top) = stack.last() {
                if builtins::matches_ender(top.0, block_type) {
                    stack.pop();
                }
            }
        }

        if block_type == ControlBlockType::Def {
            if in_macro_def {
                result.add_error_stage(
                    i,
                    0,
                    line_len,
                    "CPD-2207",
                    &format!("(outer macro started at line {})", macro_def_start_line + 1),
                    LineStage::Stage2,
                );
            }
            if let Some((outer, _)) = stack.last() {
                let keyword = builtins::get_keyword_string(*outer);
                result.add_warning_stage(
                    i,
                    0,
                    line_len,
                    "CPD-2209",
                    &format!("(inside {keyword} block)"),
                    LineStage::Stage2,
                );
            }

            validate_macro_syntax(line, trimmed, i, result);

            let is_inline = patterns::inline_macro_def().is_match(trimmed);
            if !is_inline && patterns::multiline_macro_def().is_match(trimmed) {
                in_macro_def = true;
                macro_def_start_line = i;
            }
        } else if line_parser::is_end_def_statement(&cs::chars(trimmed)) {
            if !in_macro_def {
                result.add_error_stage(
                    i,
                    0,
                    line_len,
                    "CPD-2206",
                    "#end def without matching #def",
                    LineStage::Stage2,
                );
            }
            in_macro_def = false;
            macro_def_start_line = -1;
        }
    }

    if in_macro_def {
        result.add_error_stage(
            macro_def_start_line,
            0,
            999,
            "CPD-2206",
            "#def without matching #end def",
            LineStage::Stage2,
        );
    }
}

fn validate_macro_syntax(line: &str, trimmed: &str, stage2_line: i32, result: &mut LinterResult) {
    let line_len = cs::len(line);
    if let Some(m) = patterns::inline_macro_def().captures(trimmed) {
        validate_macro_name_and_params(
            line,
            m.get(1).map_or("", |g| g.as_str()),
            m.get(2).map_or("", |g| g.as_str()),
            stage2_line,
            result,
        );
        return;
    }
    if let Some(m) = patterns::multiline_macro_def().captures(trimmed) {
        validate_macro_name_and_params(
            line,
            m.get(1).map_or("", |g| g.as_str()),
            m.get(2).map_or("", |g| g.as_str()),
            stage2_line,
            result,
        );
        return;
    }
    if let Some(m) = patterns::loose_macro_name_extract().captures(trimmed) {
        let attempted = m.get(1).map_or("", |g| g.as_str());
        if !patterns::valid_macro_name().is_match(attempted) {
            result.add_error_stage(
                stage2_line,
                0,
                line_len,
                "CPD-2210",
                &format!("'{attempted}'. Macro names can only contain ASCII letters (a-z, A-Z), digits (0-9), and underscores (_)."),
                LineStage::Stage2,
            );
            return;
        }
    }
    result.add_error_stage(
        stage2_line,
        0,
        line_len,
        "CPD-2205",
        &format!("'{trimmed}'"),
        LineStage::Stage2,
    );
}

fn validate_macro_name_and_params(
    line: &str,
    macro_name: &str,
    params_str: &str,
    stage2_line: i32,
    result: &mut LinterResult,
) {
    let line_chars = cs::chars(line);
    let line_len = line_chars.len() as i32;
    let name_len = cs::len(macro_name);
    let start_pos = cs::index_of_ic(&line_chars, macro_name, 0);
    let col = start_pos.map_or(0, |p| p as i32);
    let end_col = start_pos.map_or(line_len, |p| p as i32 + name_len);

    let name_chars = cs::chars(macro_name);
    if name_chars.last() != Some(&'$') {
        result.add_error_stage(
            stage2_line,
            col,
            end_col,
            "CPD-2202",
            &format!("'{macro_name}' should be '{macro_name}$'"),
            LineStage::Stage2,
        );
    }
    if let Some(&first) = name_chars.first() {
        if !is_letter(first) && first != '_' {
            result.add_error_stage(
                stage2_line,
                col,
                end_col,
                "CPD-2204",
                &format!("'{macro_name}'"),
                LineStage::Stage2,
            );
        }
    }

    if !cs::is_blank(params_str) {
        let (names, defaults) = parse_param_list(params_str);
        for name in required_after_optional(&names, defaults.as_ref()) {
            result.add_error_stage(
                stage2_line,
                0,
                line_len,
                "CPD-2213",
                &format!("'{name}'"),
                LineStage::Stage2,
            );
        }
        let mut seen = StrSet::new();
        for p in &names {
            if !seen.insert(p) {
                result.add_error_stage(
                    stage2_line,
                    0,
                    line_len,
                    "CPD-2212",
                    &format!("'{p}'"),
                    LineStage::Stage2,
                );
                continue;
            }
            if !patterns::valid_macro_param().is_match(p) {
                result.add_error_stage(
                    stage2_line,
                    0,
                    line_len,
                    "CPD-2211",
                    &format!("'{p}'. Macro parameters can only contain ASCII letters (a-z, A-Z), digits (0-9), and underscores (_)."),
                    LineStage::Stage2,
                );
                continue;
            }
            if !p.ends_with('$') {
                result.add_error_stage(
                    stage2_line,
                    0,
                    line_len,
                    "CPD-2203",
                    &format!("'{p}'"),
                    LineStage::Stage2,
                );
                continue;
            }
            if let Some(first) = p.chars().next() {
                if !is_letter(first) && first != '_' {
                    result.add_error_stage(
                        stage2_line,
                        0,
                        line_len,
                        "CPD-2208",
                        &format!("'{p}'"),
                        LineStage::Stage2,
                    );
                }
            }
        }
    }
}
