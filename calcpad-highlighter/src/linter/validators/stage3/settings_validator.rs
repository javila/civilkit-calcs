//! SettingsValidator.cs: validates the JSON payload of the #settings directive.
use crate::core_dto::SettingsDto;
use crate::linter::helpers::cs;
use crate::linter::helpers::directive_json_reporter::DirectiveJsonReporter;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::trim_start;
use crate::tokenizer::models::{Token, TokenType};

const CODE: &str = "CPD-3413";

#[derive(Default)]
pub struct SettingsValidator;

impl SettingsValidator {
    pub fn new() -> Self {
        SettingsValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        for (idx, line) in stage3.lines.iter().enumerate() {
            let i = idx as i32;
            if !tp.is_cpd_mode(i) {
                continue;
            }
            // #UI shares the SettingsJson token but not the key set, so skip its payload.
            if !cs::starts_with_ic(trim_start(&cs::chars(line)), "#settings") {
                continue;
            }
            for token in tp.get_tokens_for_line(i) {
                if token.token_type == TokenType::SettingsJson {
                    validate_settings_json(i, token, result);
                }
            }
        }
    }
}

fn validate_settings_json(line_index: i32, token: &Token, result: &mut LinterResult) {
    let json = cs::trim_str(&token.text);
    if json.is_empty() {
        return;
    }
    let reporter = DirectiveJsonReporter::new(line_index, token.column, token.column + token.length, CODE, "#settings");

    let Some(doc) = reporter.try_parse(result, json) else {
        return;
    };
    if !reporter.require_object(result, &doc) {
        return;
    }
    reporter.check_known_keys(result, &doc, SettingsDto::is_known_key, "setting");

    let dto = match SettingsDto::parse(json) {
        Ok(Some(d)) => d,
        // A JSON null is not an object and was rejected above.
        Ok(None) => return,
        Err(_) => {
            reporter.warn(result, "a #settings value has the wrong type");
            return;
        }
    };
    for error in dto.validate() {
        reporter.warn(result, &error.message);
    }
}
