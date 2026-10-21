//! DirectiveJsonReporter.cs: reports diagnostics for a JSON payload carried by a directive.
use crate::linter::models::LinterResult;
use serde_json::Value;

/// Binds the diagnostic code and the span of the payload once. The `LinterResult` is passed to
/// each call instead of being held (the C# struct holds a reference to it).
#[derive(Clone, Debug)]
pub struct DirectiveJsonReporter {
    line_index: i32,
    from: i32,
    to: i32,
    code: String,
    /// The directive as written, e.g. "#UI". Used in messages.
    directive: String,
}

impl DirectiveJsonReporter {
    pub fn new(line_index: i32, from: i32, to: i32, code: &str, directive: &str) -> Self {
        DirectiveJsonReporter {
            line_index,
            from,
            to,
            code: code.to_string(),
            directive: directive.to_string(),
        }
    }

    /// The same span, reporting under a different code and label.
    pub fn for_code(&self, code: &str, directive: &str) -> Self {
        Self::new(self.line_index, self.from, self.to, code, directive)
    }

    pub fn warn(&self, result: &mut LinterResult, message: &str) {
        result.add_warning(self.line_index, self.from, self.to, &self.code, message);
    }

    pub fn warn_invalid_json(&self, result: &mut LinterResult) {
        self.warn(
            result,
            &calcpad_core::messages::Invalid_JSON_in_0.replace("{0}", &self.directive),
        );
    }

    /// Parses the payload, returning None and reporting the failure when it is malformed.
    pub fn try_parse(&self, result: &mut LinterResult, json: &str) -> Option<Value> {
        match parse_json_document(json) {
            Some(v) => Some(v),
            None => {
                self.warn_invalid_json(result);
                None
            }
        }
    }

    pub fn require_object(&self, result: &mut LinterResult, root: &Value) -> bool {
        if root.is_object() {
            return true;
        }
        self.warn(result, &format!("{} payload must be a JSON object", self.directive));
        false
    }

    /// `keyNoun` completes "'x' is not a recognized ...", e.g. "setting".
    pub fn check_known_keys(
        &self,
        result: &mut LinterResult,
        root: &Value,
        is_known: impl Fn(&str) -> bool,
        key_noun: &str,
    ) {
        if let Some(obj) = root.as_object() {
            for (name, _) in obj.iter() {
                if !is_known(name) {
                    self.warn(result, &format!("'{name}' is not a recognized {key_noun}"));
                }
            }
        }
    }
}

/// `JsonDocument.Parse` as a `serde_json::Value`; None on a parse failure. Like
/// `JsonDocumentOptions` it rejects nesting deeper than 64 levels.
pub fn parse_json_document(json: &str) -> Option<Value> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for c in json.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == '{' || c == '[' {
            depth += 1;
            if depth > 64 {
                return None;
            }
        } else if c == '}' || c == ']' {
            depth -= 1;
        }
    }
    serde_json::from_str(json).ok()
}
