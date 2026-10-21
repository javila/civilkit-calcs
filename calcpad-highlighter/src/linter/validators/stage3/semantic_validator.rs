//! SemanticValidator.cs
use crate::linter::constants::builtins;
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::lower_invariant;
use crate::tokenizer::models::TokenType;

#[derive(Default)]
pub struct SemanticValidator;

impl SemanticValidator {
    pub fn new() -> Self {
        SemanticValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        validate_operators(stage3, result, tp);
        validate_commands(stage3, result, tp);
        validate_directives(stage3, result, tp);
        validate_assignments(stage3, result, tp);
        validate_incomplete_expressions(stage3, result, tp);
        validate_const_reassignment(stage3, result);
    }
}

fn validate_operators(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let tokens = tp.get_tokens_for_line(i);
        for t in 0..tokens.len().saturating_sub(1) {
            let token = &tokens[t];
            let next = &tokens[t + 1];
            if token.token_type != TokenType::Operator {
                continue;
            }
            if next.token_type == TokenType::Operator && token.column + token.length == next.column {
                let combined = format!("{}{}", token.text, next.text);
                if matches!(combined.as_str(), "++" | "--" | "**" | "//" | "&&" | "||") {
                    result.add_error(
                        i,
                        token.column,
                        next.column + next.length,
                        "CPD-3401",
                        &format!("'{combined}' is not valid in Calcpad"),
                    );
                }
            }
        }
    }
}

fn validate_commands(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Command {
                continue;
            }
            if !builtins::commands().contains(&token.text) {
                result.add_error(
                    i,
                    token.column,
                    token.column + token.length,
                    "CPD-3402",
                    &format!("'{}' is not a recognized command", token.text),
                );
            }
        }
    }
}

fn is_valid_keyword(keyword: &str) -> bool {
    builtins::keywords().contains(keyword)
        || builtins::control_block_keywords().contains(keyword)
        || builtins::end_keywords().contains(keyword)
}

fn validate_directives(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let lc = cs::chars(line);
        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Keyword {
                continue;
            }
            let keyword = lower_invariant(&token.text);
            if is_valid_keyword(&keyword) {
                continue;
            }
            let trimmed = keyword.trim_start_matches('#');
            if builtins::is_multi_word_first_word(trimmed) {
                // C# Substring throws when the token ends past the line; the port treats that as empty.
                let from = (token.column + token.length) as usize;
                let rest: &[char] = if from <= lc.len() { &lc[from..] } else { &[] };
                let remaining = crate::text::trim_start(rest);
                if !remaining.is_empty() {
                    let space = remaining.iter().position(|c| *c == ' ');
                    let second: String = match space {
                        Some(s) => remaining[..s].iter().collect(),
                        None => remaining.iter().collect(),
                    };
                    let two_word = format!("{} {}", keyword, lower_invariant(&second));
                    if is_valid_keyword(&two_word) {
                        continue;
                    }
                }
            }
            result.add_error(
                i,
                token.column,
                token.column + token.length,
                "CPD-3403",
                &format!("'{keyword}' is not a recognized directive"),
            );
        }
    }
}

fn is_compound_prev(prev: &crate::tokenizer::models::Token, token: &crate::tokenizer::models::Token) -> bool {
    prev.token_type == TokenType::Operator
        && prev.column + prev.length == token.column
        && matches!(prev.text.as_str(), "<" | ">" | "!" | "=")
}

fn validate_assignments(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        if line_parser::is_directive_line_str(cs::trim_str(line)) {
            continue;
        }
        let tokens = tp.get_tokens_for_line(i);
        let mut assignment_count = 0;
        let mut paren_depth = 0;
        let mut has_command = false;
        let mut has_multiple = false;

        for t in 0..tokens.len() {
            let token = &tokens[t];
            if matches!(token.token_type, TokenType::Comment | TokenType::HtmlComment) {
                assignment_count = 0;
                continue;
            }
            if token.token_type == TokenType::Bracket {
                if token.text == "(" {
                    paren_depth += 1;
                } else if token.text == ")" {
                    paren_depth -= 1;
                }
            }
            if token.token_type == TokenType::Command {
                has_command = true;
            } else if token.token_type == TokenType::Operator
                && (token.text == "=" || token.text == "\u{2190}")
                && paren_depth == 0
            {
                let mut is_comparison = false;
                if token.text == "=" {
                    if t > 0 && is_compound_prev(&tokens[t - 1], token) {
                        is_comparison = true;
                    }
                    if t + 1 < tokens.len() {
                        let next = &tokens[t + 1];
                        if next.token_type == TokenType::Operator
                            && token.column + token.length == next.column
                            && next.text == "="
                        {
                            is_comparison = true;
                        }
                    }
                }
                if !is_comparison {
                    assignment_count += 1;
                    if assignment_count > 1 {
                        has_multiple = true;
                    }
                }
            }
        }

        if has_multiple && !has_command {
            result.add_warning(i, 0, cs::len(line), "CPD-3404", "multiple assignments on the same line");
        }
    }
}

fn is_binary_operator(op: &str) -> bool {
    matches!(
        op,
        "+" | "-"
            | "*"
            | "/"
            | "^"
            | "\u{00F7}"
            | "\u{00B7}"
            | "\u{2219}"
            | "\u{22C5}"
            | "<"
            | ">"
            | "\u{2264}"
            | "\u{2265}"
            | "\u{2260}"
            | "&"
            | "\u{2228}"
            | "\u{2295}"
            | "\\"
            | "%"
            | "\u{2227}"
            | "\u{2190}"
    )
}

fn validate_incomplete_expressions(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        if line_parser::is_directive_line_str(cs::trim_str(line)) {
            continue;
        }
        let tokens = tp.get_tokens_for_line(i);
        if tokens.is_empty() {
            continue;
        }
        let last = &tokens[tokens.len() - 1];
        if last.token_type == TokenType::Operator && is_binary_operator(&last.text) {
            result.add_error(
                i,
                last.column,
                last.column + last.length,
                "CPD-3407",
                &format!("expression cannot end with operator '{}'", last.text),
            );
        } else if last.token_type == TokenType::Operator && (last.text == "=" || last.text == "\u{2190}") {
            let mut is_comparison = false;
            if last.text == "=" && tokens.len() >= 2 {
                if is_compound_prev(&tokens[tokens.len() - 2], last) {
                    is_comparison = true;
                }
            }
            if !is_comparison {
                result.add_error(
                    i,
                    last.column,
                    last.column + last.length,
                    "CPD-3407",
                    "incomplete assignment: missing right-hand side",
                );
            }
        }
    }
}

fn validate_const_reassignment(stage3: &Stage3Context, result: &mut LinterResult) {
    for (name, line, column) in &stage3.variable_reassignments {
        if let Some(info) = stage3.type_tracker.get_variable_info(name) {
            if info.is_const {
                result.add_error(
                    *line,
                    *column,
                    column + cs::len(name),
                    "CPD-3409",
                    &format!("cannot reassign constant '{name}'"),
                );
            }
        }
    }
    for (name, line, column) in &stage3.outer_scope_assignments {
        let info = stage3.type_tracker.get_variable_info(name);
        if info.is_none() && !stage3.defined_variables.contains(name) {
            result.add_error(
                *line,
                *column,
                column + cs::len(name),
                "CPD-3410",
                &format!("outer scope assignment to undefined variable '{name}'"),
            );
        } else if info.is_some_and(|i| i.is_const) {
            result.add_error(
                *line,
                *column,
                column + cs::len(name),
                "CPD-3409",
                &format!("cannot reassign constant '{name}'"),
            );
        }
    }
}
