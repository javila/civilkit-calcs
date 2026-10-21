//! NamingValidator.cs
use crate::linter::constants::builtins;
use crate::linter::constants::patterns;
use crate::linter::helpers::character_helpers::{is_greek_letter, is_special_math_char};
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::parsing_helpers::{parse_param_list, required_after_optional};
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::{index_of, is_letter, lower_invariant, StrSet};

#[derive(Default)]
pub struct NamingValidator;

impl NamingValidator {
    pub fn new() -> Self {
        NamingValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        validate_variable_naming(stage3, result, tp);
        validate_function_naming(stage3, result, tp);
    }
}

fn find(line: &[char], needle: &str) -> Option<usize> {
    index_of(line, &cs::chars(needle), 0)
}

fn validate_variable_naming(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let trimmed = cs::trim_str(line);
        if line_parser::is_directive_line_str(trimmed) {
            continue;
        }
        if let Some(m) = patterns::variable_assignment().captures(trimmed) {
            let var_name = m.get(1).map_or("", |g| g.as_str());
            validate_identifier_name(var_name, line, i, true, result);
        }
    }
}

fn validate_function_naming(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    let mut defined = StrSet::new();
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let trimmed = cs::trim_str(line);
        if line_parser::is_directive_line_str(trimmed) {
            continue;
        }
        let Some(m) = patterns::function_definition().captures(trimmed) else {
            continue;
        };
        let func_name = m.get(1).map_or("", |g| g.as_str());
        let params_str = cs::trim_str(m.get(2).map_or("", |g| g.as_str()));
        let lc = cs::chars(line);
        let name_len = cs::len(func_name);
        let line_len = lc.len() as i32;

        if !func_name.ends_with('$') && !builtins::functions().contains(func_name) && !defined.insert(func_name) {
            let start = find(&lc, func_name);
            let col = start.map_or(0, |p| p as i32);
            result.add_information(
                i,
                col,
                col + name_len,
                "CPD-3313",
                &format!("Function '{func_name}' redefines an existing function"),
            );
        }

        if cs::is_blank(params_str) {
            // C# would throw when the name is not found; it always is, as it came from this line.
            let start = find(&lc, func_name);
            let from = start.unwrap_or(0);
            let paren_pos = lc[from..].iter().position(|c| *c == '(').map(|p| p + from);
            let paren_pos = paren_pos.map_or(-1, |p| p as i32);
            let end_search = if paren_pos < 0 { lc.len() } else { paren_pos as usize };
            let mut end_pos = lc
                .get(end_search..)
                .and_then(|s| s.iter().position(|c| *c == ')'))
                .map_or(-1, |p| (p + end_search) as i32);
            if end_pos < 0 {
                end_pos = paren_pos + 1;
            }
            result.add_error(
                i,
                start.map_or(0, |p| p as i32),
                end_pos + 1,
                "CPD-3206",
                &format!("Function '{func_name}' must have at least one parameter"),
            );
        } else {
            let (names, defaults) = parse_param_list(params_str);
            for name in required_after_optional(&names, defaults.as_ref()) {
                let start = find(&lc, func_name);
                result.add_error(
                    i,
                    start.map_or(0, |p| p as i32),
                    line_len,
                    "CPD-3207",
                    &format!("Required parameter '{name}' cannot follow an optional parameter"),
                );
            }
        }

        validate_identifier_name(func_name, line, i, false, result);
    }
}

fn validate_identifier_name(
    identifier: &str,
    line: &str,
    stage3_line: i32,
    is_variable: bool,
    result: &mut LinterResult,
) {
    if identifier.ends_with('$') {
        return;
    }
    let lc = cs::chars(line);
    let start = find(&lc, identifier);
    let end_pos = match start {
        Some(p) => p as i32 + cs::len(identifier),
        None => lc.len() as i32,
    };
    let col = start.map_or(0, |p| p as i32);

    if let Some(first) = identifier.chars().next() {
        if !is_letter(first) && !is_greek_letter(first) && !is_special_math_char(first) {
            let code = if is_variable { "CPD-3201" } else { "CPD-3202" };
            result.add_error(
                stage3_line,
                col,
                end_pos,
                code,
                &format!(
                    "Invalid character: '{first}'. Variables, functions and units must begin with a letter or \u{2221}"
                ),
            );
            return;
        }
    }

    if !is_variable && builtins::functions().contains(identifier) {
        result.add_error(
            stage3_line,
            col,
            end_pos,
            "CPD-3203",
            &format!("Function name '{identifier}' conflicts with built-in function"),
        );
        return;
    }

    if builtins::common_constants().contains(identifier) {
        result.add_warning(
            stage3_line,
            col,
            end_pos,
            "CPD-3205",
            &format!("Variable name '{identifier}' conflicts with built-in constant"),
        );
        return;
    }

    if is_variable && is_keyword_name(identifier) {
        result.add_error(
            stage3_line,
            col,
            end_pos,
            "CPD-3204",
            &format!("Variable name '{identifier}' conflicts with keyword"),
        );
    }
}

fn is_keyword_name(name: &str) -> bool {
    let lower = lower_invariant(name);
    matches!(
        lower.as_str(),
        "if" | "else"
            | "for"
            | "while"
            | "repeat"
            | "loop"
            | "break"
            | "continue"
            | "def"
            | "end"
            | "and"
            | "or"
            | "not"
            | "xor"
    )
}
