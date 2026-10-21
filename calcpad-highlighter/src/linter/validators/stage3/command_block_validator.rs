//! CommandBlockValidator.cs
use crate::content_resolution::result::CommandBlockInfo;
use crate::linter::constants::builtins;
use crate::linter::constants::patterns;
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::parameter_parser::parse_parameters;
use crate::linter::helpers::parsing_helpers::{find_matching_close, skip_whitespace};
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::{index_of, trim, StrMap, StrSet};
use crate::tokenizer::models::TokenType;
use crate::tokenizer::CalcpadTokenizer;

#[derive(Default)]
pub struct CommandBlockValidator;

struct ValidationError {
    code: &'static str,
    message: String,
}

impl CommandBlockValidator {
    pub fn new() -> Self {
        CommandBlockValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        if stage3.command_block_functions.is_empty() {
            return;
        }
        validate_no_hash_directives_in_blocks(stage3, result);
        validate_command_block_calls(stage3, result, tp);
    }
}

fn validate_no_hash_directives_in_blocks(stage3: &Stage3Context, result: &mut LinterResult) {
    for (_, block) in stage3.command_block_functions.iter() {
        for statement in &block.statements {
            let sc = cs::chars(statement);
            if let Some(hash_idx) = sc.iter().position(|c| *c == '#') {
                let full = cs::chars(&block.full_line);
                let statement_idx = index_of(&full, &sc, 0);
                let col = match statement_idx {
                    Some(s) => (s + hash_idx) as i32,
                    None => hash_idx as i32,
                };
                result.add_error(
                    block.line_number,
                    col,
                    col + 1,
                    "CPD-3405",
                    "'#' directives are not allowed inside command blocks. Use if(), $Repeat, etc. instead",
                );
            }
        }
    }
}

fn validate_command_block_calls(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) {
            continue;
        }
        if line_parser::should_skip_line(line) {
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

        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Function {
                continue;
            }
            let func_name = &token.text;
            let Some(block) = stage3.command_block_functions.get(func_name) else {
                continue;
            };
            let Some(call_params) = extract_call_parameters(&lc, (token.column + token.length) as usize) else {
                continue;
            };
            let required = match stage3.defined_functions.get(func_name) {
                Some(f) => f.required_param_count,
                None => block.parameters.len() as i32,
            };
            let n = call_params.len() as i32;
            if n < required || n > block.parameters.len() as i32 {
                continue;
            }

            let errors = validate_block_statements(block, stage3);
            if !errors.is_empty() {
                let call_end = find_call_end(&lc, token.column as usize);
                for e in errors {
                    result.add_error(i, token.column, call_end, e.code, &e.message);
                }
            }
        }
    }
}

fn extract_call_parameters(line: &[char], after_func_name: usize) -> Option<Vec<String>> {
    let mut pos = after_func_name;
    skip_whitespace(line, &mut pos);
    if pos >= line.len() || line[pos] != '(' {
        return None;
    }
    let start = pos + 1;
    let close = find_matching_close(line, pos, '(', ')');
    if close < 0 {
        return None;
    }
    let params: String = line[start..close as usize].iter().collect();
    Some(parse_parameters(&params, None, 0, None))
}

fn find_call_end(line: &[char], func_start: usize) -> i32 {
    let mut pos = func_start;
    while pos < line.len() && line[pos] != '(' {
        pos += 1;
    }
    if pos >= line.len() {
        return func_start as i32 + 1;
    }
    let close = find_matching_close(line, pos, '(', ')');
    if close >= 0 {
        close + 1
    } else {
        line.len() as i32
    }
}

fn validate_block_statements(block: &CommandBlockInfo, stage3: &Stage3Context) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut local_variables = StrSet::new_ci();
    let mut local_assignments: StrMap<String> = StrMap::new_ci();
    let mut tokenizer = CalcpadTokenizer::new();

    for statement in &block.statements {
        collect_assigned_variables(
            &mut tokenizer,
            statement,
            &mut local_variables,
            Some(&mut local_assignments),
        );
        collect_command_loop_variables(&mut tokenizer, statement, &mut local_variables);
    }
    for statement in &block.statements {
        validate_statement(
            &mut tokenizer,
            statement,
            stage3,
            &local_variables,
            &block.parameters,
            &mut errors,
        );
    }
    errors
}

fn collect_assigned_variables(
    tokenizer: &mut CalcpadTokenizer,
    statement: &str,
    local_variables: &mut StrSet,
    mut local_assignments: Option<&mut StrMap<String>>,
) {
    let result = tokenizer.tokenize(statement);
    let mut found_variable = false;
    let mut variable_name = String::new();
    let mut is_element_access = false;

    for token in &result.tokens {
        if token.token_type == TokenType::None {
            continue;
        }
        if !found_variable {
            if token.token_type == TokenType::Variable {
                found_variable = true;
                variable_name = token.text.clone();
                is_element_access = token.text.ends_with('.');
            } else {
                break;
            }
        } else {
            if token.token_type == TokenType::Operator && token.text == "." {
                is_element_access = true;
                continue;
            }
            if token.token_type == TokenType::Operator && token.text == "=" {
                let mut name = variable_name.clone();
                if is_element_access && name.ends_with('.') {
                    name.pop();
                }
                local_variables.insert(&name);
                let equals_end = (token.column + token.length) as usize;
                let sc = cs::chars(statement);
                if let Some(la) = local_assignments.as_deref_mut() {
                    if !is_element_access && equals_end < sc.len() {
                        let rhs: String = trim(&sc[equals_end..]).iter().collect();
                        if !rhs.is_empty() {
                            la.insert(&name, rhs);
                        }
                    }
                }
            }
            break;
        }
    }
}

fn collect_command_loop_variables(tokenizer: &mut CalcpadTokenizer, statement: &str, local_variables: &mut StrSet) {
    let tokens = tokenizer.tokenize(statement).tokens;
    let mut after_at = false;
    for token in &tokens {
        if token.token_type == TokenType::Operator && token.text == "@" {
            after_at = true;
            continue;
        }
        if after_at && matches!(token.token_type, TokenType::LocalVariable | TokenType::Variable) {
            local_variables.insert(&token.text);
            after_at = false;
        }
    }
}

fn validate_statement(
    tokenizer: &mut CalcpadTokenizer,
    statement: &str,
    stage3: &Stage3Context,
    local_variables: &StrSet,
    param_names: &[String],
    errors: &mut Vec<ValidationError>,
) {
    let result = tokenizer.tokenize(statement);
    for token in &result.tokens {
        if token.token_type != TokenType::Variable {
            continue;
        }
        let identifier = &token.text;
        let is_element_access = identifier.ends_with('.');
        let base_name: &str = if is_element_access {
            &identifier[..identifier.len() - 1]
        } else {
            identifier
        };

        if param_names.iter().any(|p| p == base_name) {
            continue;
        }
        if local_variables.contains(base_name) {
            continue;
        }
        if is_keyword_or_command(base_name) {
            continue;
        }
        let is_defined = stage3.defined_variables.contains(base_name)
            || stage3.defined_functions.contains_key(base_name)
            || builtins::functions().contains(base_name)
            || builtins::common_constants().contains(base_name)
            || builtins::units().contains(base_name)
            || stage3.custom_units.contains(base_name)
            || stage3.defined_macros.contains_key(base_name);

        if !is_defined {
            if let Some(dot_idx) = base_name.find('.').filter(|d| *d > 0) {
                let before = &base_name[..dot_idx];
                if param_names.iter().any(|p| p == before) {
                    let after = &base_name[dot_idx + 1..];
                    errors.push(ValidationError {
                        code: "CPD-3301",
                        message: format!(
                            "'{base_name}' in command block. Function parameters require .() syntax for element access: {before}.({after})"
                        ),
                    });
                    continue;
                }
            }
            errors.push(ValidationError {
                code: "CPD-3301",
                message: format!("'{base_name}' in command block"),
            });
        }
    }
}

fn is_keyword_or_command(identifier: &str) -> bool {
    builtins::valid_hash_keywords().contains(identifier) || builtins::commands().contains(&format!("${identifier}"))
}
