//! HtmlCommentValidator.cs: validates metadata comments (`'<!--{...}-->`).
use crate::html_comment::{HtmlCommentBlock, HtmlCommentParseStatus, HtmlCommentParser, PdfSettingsDto};
use crate::linter::constants::error_codes;
use crate::linter::helpers::cs;
use crate::linter::helpers::directive_json_reporter::DirectiveJsonReporter;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{DefinitionMetadata, LinterResult, Stage3Context};
use crate::text::eq_ignore_case;
use crate::tokenizer::models::TokenType;
use serde_json::Value;

const CODE: &str = "CPD-3412";
const TYPE_CODE: &str = "CPD-3411";
const PDF_CODE: &str = "CPD-3414";
const MISPLACED_UI_OVERRIDES_CODE: &str = "CPD-3416";
const DUPLICATE_UI_OVERRIDES_CODE: &str = "CPD-3417";
const MIXED_UI_OVERRIDES_CODE: &str = "CPD-3418";

/// Recognized top-level keys of a metadata comment (ignore case).
const KNOWN_KEYS: [&str; 8] = [
    "desc",
    "paramTypes",
    "paramDesc",
    "returnType",
    "LintIgnore",
    "EndLintIgnore",
    "pdf",
    "uiOverrides",
];

#[derive(Default)]
pub struct HtmlCommentValidator {
    parser: HtmlCommentParser,
}

impl HtmlCommentValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        let mut saw_ui_overrides = false;
        for block in self.parser.parse(tp.all_tokens()) {
            if block.start_line as usize >= stage3.lines.len()
                || block.start_line < 0
                || !tp.is_cpd_mode(block.start_line)
            {
                continue;
            }
            // A plain prose comment is an HTML comment too and carries no metadata.
            match &block.raw_json {
                Some(r) if r.starts_with('{') => {}
                _ => continue,
            }

            let reporter = DirectiveJsonReporter::new(
                block.start_line,
                block.start_column,
                block.end_column,
                CODE,
                "metadata comment",
            );

            if block.status != HtmlCommentParseStatus::Success || block.data.is_none() {
                reporter.warn_invalid_json(result);
                continue;
            }
            let root = block.data.as_ref().unwrap();
            if !reporter.require_object(result, root) {
                continue;
            }

            reporter.check_known_keys(
                result,
                root,
                |n| KNOWN_KEYS.iter().any(|k| eq_ignore_case(k, n)),
                "metadata property",
            );
            validate_types(&block, root, stage3, tp, &reporter, result);
            validate_lint_regions(root, &reporter, result);
            validate_ui_overrides(&block, root, stage3, &reporter, &mut saw_ui_overrides, result);
            validate_pdf(root, &reporter, result);
        }
    }
}

fn validate_types(
    block: &HtmlCommentBlock,
    root: &Value,
    stage3: &Stage3Context,
    tp: &TokenizedLineProvider,
    reporter: &DirectiveJsonReporter,
    result: &mut LinterResult,
) {
    let type_reporter = reporter.for_code(TYPE_CODE, "metadata comment");

    if let Some(Value::String(value)) = root.get("returnType") {
        if !value.is_empty() && !DefinitionMetadata::is_valid_function_param_type(value) {
            type_reporter.warn(
                result,
                &format!("'{value}' is not a valid returnType. Expected value, vector, matrix, or any"),
            );
        }
    }

    let Some(Value::Array(types)) = root.get("paramTypes") else {
        return;
    };
    let is_macro = is_next_definition_macro(block.end_line, stage3, tp);
    let valid_list = if is_macro {
        "a valid TokenType name"
    } else {
        "value, vector, matrix, or any"
    };

    for item in types {
        let Value::String(value) = item else {
            continue;
        };
        let valid = if is_macro {
            DefinitionMetadata::is_valid_macro_param_type(value)
        } else {
            DefinitionMetadata::is_valid_function_param_type(value)
        };
        if !value.is_empty() && !valid {
            type_reporter.warn(result, &format!("'{value}' is not valid. Expected {valid_list}"));
        }
    }
}

fn validate_lint_regions(root: &Value, reporter: &DirectiveJsonReporter, result: &mut LinterResult) {
    for name in ["LintIgnore", "EndLintIgnore"] {
        let Some(prop) = root.get(name) else {
            continue;
        };
        let Value::Array(items) = prop else {
            reporter.warn(
                result,
                &format!("'{name}' must be an array of diagnostic codes (empty for all)"),
            );
            continue;
        };
        for item in items {
            let Value::String(code) = item else {
                reporter.warn(
                    result,
                    &format!("'{name}' must contain diagnostic codes, e.g. \"CPD-3301\""),
                );
                continue;
            };
            if !error_codes::DESCRIPTIONS.iter().any(|(c, _)| c == code) {
                reporter.warn(result, &format!("'{code}' is not a known diagnostic code"));
            }
        }
    }
}

fn validate_ui_overrides(
    block: &HtmlCommentBlock,
    root: &Value,
    stage3: &Stage3Context,
    reporter: &DirectiveJsonReporter,
    saw_ui_overrides: &mut bool,
    result: &mut LinterResult,
) {
    let Some(overrides) = root.get("uiOverrides") else {
        return;
    };
    if let Some(info) = stage3.include_map.get(&block.start_line) {
        if info.source == "include" {
            return;
        }
    }
    if has_key_other_than(root, "uiOverrides") {
        reporter.for_code(MIXED_UI_OVERRIDES_CODE, "metadata comment").warn(
            result,
            "'uiOverrides' should be the only key in its comment for predictable behavior",
        );
    }

    let mut misplaced_or_duplicate = false;
    if block.start_line != 0 {
        reporter.for_code(MISPLACED_UI_OVERRIDES_CODE, "metadata comment").warn(
            result,
            "'uiOverrides' has no effect unless its comment is on the first line of the file",
        );
        misplaced_or_duplicate = true;
    }
    if *saw_ui_overrides {
        reporter.for_code(DUPLICATE_UI_OVERRIDES_CODE, "metadata comment").warn(
            result,
            "a second 'uiOverrides' comment is ignored - only the first one in the file is read",
        );
        misplaced_or_duplicate = true;
    }
    *saw_ui_overrides = true;
    if misplaced_or_duplicate {
        return;
    }

    let Value::Object(map) = overrides else {
        reporter.warn(
            result,
            "'uiOverrides' must be an object of control identities to values, e.g. {\"L:1\": \"8\"}",
        );
        return;
    };
    for (name, value) in map.iter() {
        if !value.is_string() {
            reporter.warn(
                result,
                &format!("'uiOverrides' value for '{name}' must be a string, e.g. \"8\""),
            );
        }
    }
}

fn has_key_other_than(root: &Value, key: &str) -> bool {
    root.as_object().is_some_and(|o| o.keys().any(|k| k != key))
}

fn validate_pdf(root: &Value, reporter: &DirectiveJsonReporter, result: &mut LinterResult) {
    let Some(pdf) = root.get("pdf") else {
        return;
    };
    let pdf_reporter = reporter.for_code(PDF_CODE, "pdf settings");
    if !pdf_reporter.require_object(result, pdf) {
        return;
    }
    pdf_reporter.check_known_keys(result, pdf, PdfSettingsDto::is_known_key, "PDF setting");
    let dto = match PdfSettingsDto::parse_value(pdf) {
        Ok(d) => d,
        Err(_) => {
            pdf_reporter.warn(result, "a pdf value has the wrong type");
            return;
        }
    };
    for error in dto.validate() {
        pdf_reporter.warn(result, &error.message);
    }
}

/// Whether the next non-blank line with tokens starts with a `#def` keyword token.
fn is_next_definition_macro(comment_end_line: i32, stage3: &Stage3Context, tp: &TokenizedLineProvider) -> bool {
    let mut i = comment_end_line + 1;
    while (i as usize) < stage3.lines.len() {
        if cs::is_blank(&stage3.lines[i as usize]) {
            i += 1;
            continue;
        }
        let tokens = tp.get_tokens_for_line(i);
        if tokens.is_empty() {
            i += 1;
            continue;
        }
        for t in tokens {
            if t.token_type == TokenType::None {
                continue;
            }
            return t.token_type == TokenType::Keyword
                && cs::equals_ic(&cs::chars(t.text.trim_end_matches(crate::text::is_white_space)), "#def");
        }
        return false;
    }
    false
}
