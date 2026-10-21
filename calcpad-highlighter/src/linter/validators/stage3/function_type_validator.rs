//! FunctionTypeValidator.cs: validates function calls against their signatures.
use crate::linter::constants::function_signatures;
use crate::linter::constants::patterns;
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::parameter_parser::parse_parameters;
use crate::linter::helpers::parsing_helpers::{
    extract_params_string, find_closing_paren, get_function_params_from_line, skip_whitespace,
};
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{CalcpadType, FunctionSignature, LinterResult, Stage3Context};
use crate::snippets::ParameterType;
use crate::text::{trim, StrSet};
use crate::tokenizer::models::{Token, TokenType};

#[derive(Default)]
pub struct FunctionTypeValidator;

impl FunctionTypeValidator {
    pub fn new() -> Self {
        FunctionTypeValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        for (idx, line) in stage3.lines.iter().enumerate() {
            let i = idx as i32;
            if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
                continue;
            }
            let lc = cs::chars(line);
            let trimmed = trim(&lc);
            if line_parser::is_directive_line(trimmed) {
                continue;
            }
            let trimmed_s: String = trimmed.iter().collect();
            if patterns::function_with_command_block().is_match(&trimmed_s) {
                continue;
            }
            validate_function_calls_on_line(line, &lc, tp.get_tokens_for_line(i), i, stage3, result);
        }
    }
}

fn validate_function_calls_on_line(
    line: &str,
    lc: &[char],
    tokens: &[Token],
    stage3_line: i32,
    stage3: &Stage3Context,
    result: &mut LinterResult,
) {
    let function_params = get_function_params_from_line(line);

    for token in tokens {
        if token.token_type != TokenType::Function {
            continue;
        }
        let func_name = &token.text;
        let after_name = (token.column + token.length) as usize;
        let (found, params_str) = extract_params_string(lc, after_name);
        if !found {
            continue;
        }
        let mut paren_col = after_name;
        skip_whitespace(lc, &mut paren_col);
        let param_start_col = paren_col as i32 + 1;

        let mut groups: Vec<Vec<Token>> = Vec::new();
        let parameters = parse_parameters(&params_str, Some(tokens), param_start_col, Some(&mut groups));
        let param_count = parameters.len() as i32;

        for (pi, p) in parameters.iter().enumerate() {
            if cs::is_blank(p) {
                let end_col = find_closing_paren(lc, after_name) as i32;
                result.add_error(
                    stage3_line,
                    token.column,
                    end_col,
                    "CPD-3311",
                    &format!("Empty parameter {} in call to '{}'", pi + 1, func_name),
                );
                break;
            }
        }

        if function_signatures::has_signature(func_name) {
            let overloads = function_signatures::get_all_overloads(func_name);
            let matching = find_matching_overloads(overloads, param_count);
            if matching.is_empty() {
                // C# dereferences the signature; HasSignature guarantees it exists.
                let Some(signature) = function_signatures::get_signature(func_name) else {
                    continue;
                };
                let end_col = find_closing_paren(lc, after_name) as i32;
                if param_count < signature.min_params {
                    result.add_error(
                        stage3_line,
                        token.column,
                        end_col,
                        "CPD-3307",
                        &format!(
                            "'{}' requires at least {} parameter(s), got {}",
                            func_name, signature.min_params, param_count
                        ),
                    );
                } else if signature.max_params >= 0 && param_count > signature.max_params {
                    result.add_error(
                        stage3_line,
                        token.column,
                        end_col,
                        "CPD-3308",
                        &format!(
                            "'{}' accepts at most {} parameter(s), got {}",
                            func_name, signature.max_params, param_count
                        ),
                    );
                }
                continue;
            }

            validate_parameter_types_against_overloads(
                &parameters,
                &groups,
                &matching,
                func_name,
                token,
                stage3_line,
                lc,
                stage3,
                &function_params,
                result,
            );
        }
    }
}

fn find_matching_overloads(overloads: &[FunctionSignature], param_count: i32) -> Vec<&FunctionSignature> {
    overloads
        .iter()
        .filter(|s| {
            s.accepts_any_count || (param_count >= s.min_params && (s.max_params < 0 || param_count <= s.max_params))
        })
        .collect()
}

fn validate_parameter_types_against_overloads(
    parameters: &[String],
    groups: &[Vec<Token>],
    matching: &[&FunctionSignature],
    func_name: &str,
    token: &Token,
    stage3_line: i32,
    lc: &[char],
    stage3: &Stage3Context,
    function_params: &StrSet,
    result: &mut LinterResult,
) {
    let mut actual_types = vec![CalcpadType::Unknown; parameters.len()];
    for (i, p) in parameters.iter().enumerate() {
        let param = cs::trim_str(p);
        if param.is_empty() {
            actual_types[i] = CalcpadType::Unknown;
        } else if i < groups.len() && !groups[i].is_empty() {
            actual_types[i] = infer_type_from_tokens(&groups[i], stage3, function_params);
        } else {
            actual_types[i] = infer_parameter_type(param, stage3, function_params);
        }
    }

    let mut best_match = matching[0];
    let mut best_score: i32 = -1;
    for sig in matching {
        let mut score = 0;
        let mut all_match = true;
        for (i, p) in parameters.iter().enumerate() {
            if cs::trim_str(p).is_empty() {
                continue;
            }
            let expected = sig.get_parameter_type(i);
            if expected == ParameterType::Any || expected == ParameterType::Various {
                score += 1;
                continue;
            }
            if sig.is_argument_compatible(expected, actual_types[i]) {
                score += 1;
            } else {
                all_match = false;
            }
        }
        if all_match {
            return;
        }
        if score > best_score {
            best_score = score;
            best_match = sig;
        }
    }

    for (i, p) in parameters.iter().enumerate() {
        if cs::trim_str(p).is_empty() {
            continue;
        }
        let expected = best_match.get_parameter_type(i);
        if expected == ParameterType::Any || expected == ParameterType::Various {
            continue;
        }
        if !best_match.is_argument_compatible(expected, actual_types[i]) {
            let end_col = find_closing_paren(lc, (token.column + token.length) as usize) as i32;
            result.add_warning(
                stage3_line,
                token.column,
                end_col,
                "CPD-3309",
                &format!(
                    "'{}' parameter {} expects {} but got {}",
                    func_name,
                    i + 1,
                    FunctionSignature::get_type_name(expected),
                    get_calcpad_type_name(actual_types[i])
                ),
            );
        }
    }
}

fn infer_parameter_type(expression: &str, stage3: &Stage3Context, function_params: &StrSet) -> CalcpadType {
    let trimmed = cs::trim_str(expression);
    if function_params.contains(trimmed) {
        return CalcpadType::Various;
    }
    if let Some(info) = stage3.type_tracker.get_variable_info(trimmed) {
        if info.type_ == CalcpadType::Function {
            return info.return_type;
        }
        return info.type_;
    }
    stage3.type_tracker.infer_type_from_expression(trimmed, -1)
}

fn get_calcpad_type_name(t: CalcpadType) -> String {
    match t {
        CalcpadType::Value => "scalar".to_string(),
        CalcpadType::Vector => "vector".to_string(),
        CalcpadType::Matrix => "matrix".to_string(),
        CalcpadType::Various => "various".to_string(),
        CalcpadType::Unknown => "unknown".to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn infer_type_from_tokens(tokens: &[Token], stage3: &Stage3Context, function_params: &StrSet) -> CalcpadType {
    if tokens.len() == 1 {
        let t = &tokens[0];
        if t.token_type == TokenType::Variable || t.token_type == TokenType::LocalVariable {
            if function_params.contains(&t.text) {
                return CalcpadType::Various;
            }
            if let Some(info) = stage3.type_tracker.get_variable_info(&t.text) {
                return if info.type_ == CalcpadType::Function {
                    info.return_type
                } else {
                    info.type_
                };
            }
        }
        if t.token_type == TokenType::Const {
            return CalcpadType::Value;
        }
        return CalcpadType::Unknown;
    }

    if tokens[0].token_type == TokenType::Bracket && tokens[0].text == "[" {
        let mut has_pipe = false;
        let mut d = 0;
        for t in tokens {
            if t.token_type == TokenType::Bracket && t.text == "[" {
                d += 1;
            } else if t.token_type == TokenType::Bracket && t.text == "]" {
                d -= 1;
            } else if d == 1 && t.text == "|" {
                has_pipe = true;
            }
        }
        return if has_pipe {
            CalcpadType::Matrix
        } else {
            CalcpadType::Vector
        };
    }

    let mut highest = CalcpadType::Unknown;
    let mut depth = 0;
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        if t.token_type == TokenType::Bracket {
            match t.text.as_str() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                _ => {}
            }
            i += 1;
            continue;
        }
        if depth > 0 {
            i += 1;
            continue;
        }
        if t.token_type == TokenType::Function {
            let ret = stage3.type_tracker.get_function_return_type(&t.text);
            if ret == CalcpadType::Matrix {
                return CalcpadType::Matrix;
            }
            if ret == CalcpadType::Vector && highest != CalcpadType::Matrix {
                highest = CalcpadType::Vector;
            }
            i += 1;
            continue;
        }
        if t.token_type == TokenType::Variable || t.token_type == TokenType::LocalVariable {
            if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Operator && tokens[i + 1].text == "." {
                i += 1;
                if i + 1 < tokens.len() {
                    let idx = &tokens[i + 1];
                    if matches!(
                        idx.token_type,
                        TokenType::Variable | TokenType::LocalVariable | TokenType::Const
                    ) {
                        i += 1;
                    }
                }
                i += 1;
                continue;
            }
            if function_params.contains(&t.text) {
                if highest == CalcpadType::Unknown {
                    highest = CalcpadType::Various;
                }
                i += 1;
                continue;
            }
            if let Some(info) = stage3.type_tracker.get_variable_info(&t.text) {
                let vt = if info.type_ == CalcpadType::Function {
                    info.return_type
                } else {
                    info.type_
                };
                if vt == CalcpadType::Matrix {
                    return CalcpadType::Matrix;
                }
                if vt == CalcpadType::Vector && highest != CalcpadType::Matrix {
                    highest = CalcpadType::Vector;
                }
            }
        }
        i += 1;
    }
    highest
}
