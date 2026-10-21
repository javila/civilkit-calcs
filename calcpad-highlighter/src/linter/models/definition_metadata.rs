use super::CalcpadType;
use crate::text::{eq_ignore_case, lower_invariant};
use crate::tokenizer::models::TokenType;
use serde_json::Value;

/// Metadata read from a `<!--{...}-->` comment on the line before a definition.
#[derive(Clone, Debug, Default)]
pub struct DefinitionMetadata {
    pub description: Option<String>,
    pub param_types: Option<Vec<String>>,
    pub param_descriptions: Option<Vec<String>>,
    pub return_type: Option<String>,
}

impl DefinitionMetadata {
    pub const VALID_FUNCTION_PARAM_TYPES: [&'static str; 4] = ["value", "vector", "matrix", "any"];

    pub fn is_valid_function_param_type(s: &str) -> bool {
        Self::VALID_FUNCTION_PARAM_TYPES.iter().any(|t| eq_ignore_case(t, s))
    }

    /// Valid macro parameter types are the TokenType names (ignore case).
    pub fn is_valid_macro_param_type(s: &str) -> bool {
        TokenType::NAMES.iter().any(|t| eq_ignore_case(t, s))
    }

    pub fn parse_function_return_type(value: &str) -> Option<CalcpadType> {
        if value.is_empty() {
            return None;
        }
        match lower_invariant(value).as_str() {
            "value" => Some(CalcpadType::Value),
            "vector" => Some(CalcpadType::Vector),
            "matrix" => Some(CalcpadType::Matrix),
            "any" => Some(CalcpadType::Various),
            _ => None,
        }
    }

    /// Parses the first `<!--{...}-->` marker after the first comment quote on `line`.
    pub fn try_parse(line: &[char]) -> Option<DefinitionMetadata> {
        if line.is_empty() {
            return None;
        }
        let comment_start = line.iter().position(|c| *c == '\'' || *c == '"')?;
        let after = &line[comment_start + 1..];
        let marker_start = crate::text::index_of_str(after, "<!--", 0)?;
        let json_start = marker_start + 4;
        if json_start >= after.len() || after[json_start] != '{' {
            return None;
        }
        let marker_end = crate::text::index_of_str(after, "-->", 0)?;
        if marker_end <= json_start {
            return None;
        }
        let json: String = after[json_start..marker_end].iter().collect();
        if json.is_empty() {
            return None;
        }
        let root: Value = serde_json::from_str(&json).ok()?;
        let obj = root.as_object();
        let mut result = DefinitionMetadata::default();
        let mut has_any = false;
        if let Some(Value::String(s)) = obj.and_then(|o| o.get("desc")) {
            result.description = Some(s.clone());
            has_any = true;
        }
        if let Some(Value::Array(a)) = obj.and_then(|o| o.get("paramTypes")) {
            result.param_types = Some(string_items(a)?);
            has_any = true;
        }
        if let Some(Value::Array(a)) = obj.and_then(|o| o.get("paramDesc")) {
            result.param_descriptions = Some(string_items(a)?);
            has_any = true;
        }
        if let Some(Value::String(s)) = obj.and_then(|o| o.get("returnType")) {
            result.return_type = Some(s.clone());
            has_any = true;
        }
        if has_any {
            Some(result)
        } else {
            None
        }
    }

    /// The JSON text of a `<!--{...}-->` marker inside a comment token's text.
    pub fn extract_json_from_comment(token_text: &str) -> Option<String> {
        if token_text.is_empty() {
            return None;
        }
        let all: Vec<char> = token_text.chars().collect();
        let mut text: &[char] = &all;
        if !text.is_empty() && (text[0] == '\'' || text[0] == '"') {
            text = &text[1..];
        }
        if !text.is_empty() && (text[text.len() - 1] == '\'' || text[text.len() - 1] == '"') {
            text = &text[..text.len() - 1];
        }
        let open = crate::text::index_of_str(text, "<!--", 0)?;
        let after = &text[open + 4..];
        if after.is_empty() || after[0] != '{' {
            return None;
        }
        let close = crate::text::index_of_str(after, "-->", 0)?;
        let json = &after[..close];
        if json.is_empty() {
            None
        } else {
            Some(json.iter().collect())
        }
    }
}

/// `item.GetString() ?? ""` for every element. C# throws on a non-string, non-null element; the
/// port treats that comment as having no metadata.
fn string_items(a: &[Value]) -> Option<Vec<String>> {
    a.iter()
        .map(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Null => Some(String::new()),
            _ => None,
        })
        .collect()
}
