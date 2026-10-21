//! BalanceValidator.cs
use crate::linter::constants::builtins::{self, ControlBlockType};
use crate::linter::helpers::cs;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::tokenizer::models::TokenType;

#[derive(Default)]
pub struct BalanceValidator;

impl BalanceValidator {
    pub fn new() -> Self {
        BalanceValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        validate_brackets(stage3, result, tp);
        validate_control_blocks(stage3, result, tp);
    }
}

fn validate_brackets(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for i in 0..stage3.lines.len() as i32 {
        if !tp.is_cpd_mode(i) {
            continue;
        }
        let mut paren: Vec<i32> = Vec::new();
        let mut square: Vec<i32> = Vec::new();
        let mut curly: Vec<i32> = Vec::new();

        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Bracket {
                continue;
            }
            let col = token.column;
            let c = token.text.chars().next().unwrap_or('\0');
            let at = format!("at column {}", col + 1);
            match c {
                '(' => paren.push(col),
                ')' => {
                    if paren.pop().is_none() {
                        result.add_error(i, col, col + 1, "CPD-3102", &at);
                    }
                }
                '[' => square.push(col),
                ']' => {
                    if square.pop().is_none() {
                        result.add_error(i, col, col + 1, "CPD-3104", &at);
                    }
                }
                '{' => curly.push(col),
                '}' => {
                    if curly.pop().is_none() {
                        result.add_error(i, col, col + 1, "CPD-3106", &at);
                    }
                }
                _ => {}
            }
        }
        while let Some(col) = paren.pop() {
            result.add_error(i, col, col + 1, "CPD-3101", &format!("at column {}", col + 1));
        }
        while let Some(col) = square.pop() {
            result.add_error(i, col, col + 1, "CPD-3103", &format!("at column {}", col + 1));
        }
        while let Some(col) = curly.pop() {
            result.add_error(i, col, col + 1, "CPD-3105", &format!("at column {}", col + 1));
        }
    }
}

fn validate_control_blocks(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    let mut stack: Vec<(ControlBlockType, i32)> = Vec::new();
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) {
            continue;
        }
        let block_type = builtins::get_block_type(line);
        if builtins::is_block_starter(block_type) {
            if block_type == ControlBlockType::Def && line.contains('=') {
                continue;
            }
            stack.push((block_type, i));
        } else if matches!(
            block_type,
            ControlBlockType::EndIf | ControlBlockType::Loop | ControlBlockType::EndDef
        ) {
            let matched = stack
                .last()
                .is_some_and(|top| builtins::matches_ender(top.0, block_type));
            if !matched {
                if block_type != ControlBlockType::EndDef {
                    let ender = builtins::get_keyword_string(block_type);
                    let expected = match block_type {
                        ControlBlockType::EndIf => "#if",
                        ControlBlockType::Loop => "#repeat, #for, or #while",
                        _ => "matching block",
                    };
                    result.add_error(
                        i,
                        0,
                        cs::len(line),
                        "CPD-3105",
                        &format!("{ender} without matching {expected}"),
                    );
                }
            } else {
                stack.pop();
            }
        }
    }
    while let Some((block_type, line_num)) = stack.pop() {
        let keyword = builtins::get_keyword_string(block_type);
        let expected = builtins::get_expected_ender_keyword(block_type);
        result.add_error(
            line_num,
            0,
            999,
            "CPD-3105",
            &format!("{keyword} without matching {expected}"),
        );
    }
}
