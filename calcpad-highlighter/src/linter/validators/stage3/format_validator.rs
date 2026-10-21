//! FormatValidator.cs
use crate::linter::helpers::cs;
use crate::linter::helpers::line_parser;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::tokenizer::models::TokenType;
use regex::Regex;
use std::sync::OnceLock;

/// The Core format regex (Validator.cs): standard specifiers F, C, E, G, N, D plus 0-2 digits, or
/// custom patterns. Ignore case. `$` matches only at the end of the text here, whereas .NET also
/// matches before a final "\n"; tokens never contain a newline, so the two agree.
fn format_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^[FCEGND]\d{0,2}$|^[0#]+(,[0#]+)?(\.[0#]+)?([eE][+-]?0+)?$").expect("valid pattern")
    })
}

#[derive(Default)]
pub struct FormatValidator;

impl FormatValidator {
    pub fn new() -> Self {
        FormatValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        for (idx, line) in stage3.lines.iter().enumerate() {
            let i = idx as i32;
            if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
                continue;
            }
            let tokens = tp.get_tokens_for_line(i);
            if tokens.is_empty() {
                continue;
            }
            let is_format_keyword_line =
                tokens[0].token_type == TokenType::Keyword && cs::equals_ic(&cs::chars(&tokens[0].text), "#format");
            for token in tokens {
                if token.token_type != TokenType::Format {
                    continue;
                }
                let text = &token.text;
                if cs::equals_ic(&cs::chars(text), "default") {
                    if !is_format_keyword_line {
                        result.add_warning(
                            i,
                            token.column,
                            token.end_column(),
                            "CPD-3601",
                            &format!("Invalid format specifier: '{text}' - 'default' is only valid with #format"),
                        );
                    }
                    continue;
                }
                if !format_regex().is_match(text) {
                    result.add_warning(
                        i,
                        token.column,
                        token.end_column(),
                        "CPD-3601",
                        &format!("Invalid format specifier: '{text}'"),
                    );
                }
            }
        }
    }
}
