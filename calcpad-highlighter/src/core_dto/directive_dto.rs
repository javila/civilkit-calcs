//! DirectiveDto.cs
use crate::text::eq_ignore_case;
use calcpad_core::netfmt::double_to_string;
pub use calcpad_core::ui_dto::JsonError;
use serde_json::{Map, Value};

/// A value the directive supplied that its own rules reject.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectiveError<K> {
    pub key: K,
    pub message: String,
}

/// `KnownKeys.Contains(name)`: the recognised key names, ignore case.
pub fn is_known_key(names: &[&str], name: &str) -> bool {
    names.iter().any(|n| eq_ignore_case(n, name))
}

/// The JSON root as an object, `Ok(None)` for JSON `null`, an error for any other root.
pub fn root_object(json: &str) -> Result<Option<Map<String, Value>>, JsonError> {
    let v: Value = serde_json::from_str(json).map_err(|_| JsonError)?;
    object_of(v)
}

pub fn object_of(v: Value) -> Result<Option<Map<String, Value>>, JsonError> {
    match v {
        Value::Null => Ok(None),
        Value::Object(o) => Ok(Some(o)),
        _ => Err(JsonError),
    }
}

pub fn get_str(v: &Value) -> Result<Option<String>, JsonError> {
    match v {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s.clone())),
        _ => Err(JsonError),
    }
}

pub fn get_bool(v: &Value) -> Result<Option<bool>, JsonError> {
    match v {
        Value::Null => Ok(None),
        Value::Bool(b) => Ok(Some(*b)),
        _ => Err(JsonError),
    }
}

pub fn get_int(v: &Value) -> Result<Option<i32>, JsonError> {
    match v {
        Value::Null => Ok(None),
        Value::Number(n) => match n.as_i64() {
            Some(i) if i >= i32::MIN as i64 && i <= i32::MAX as i64 => Ok(Some(i as i32)),
            _ => Err(JsonError),
        },
        _ => Err(JsonError),
    }
}

pub fn get_double(v: &Value) -> Result<Option<f64>, JsonError> {
    match v {
        Value::Null => Ok(None),
        Value::Number(n) => n.as_f64().map(Some).ok_or(JsonError),
        _ => Err(JsonError),
    }
}

/// `CheckRange`: an error entry when `value` lies outside [min, max] (`max` None for no upper bound).
pub fn check_range<K: Copy>(
    errors: &mut Vec<DirectiveError<K>>,
    key: K,
    name: &str,
    value: Option<f64>,
    min: f64,
    max: Option<f64>,
) {
    let Some(d) = value else {
        return;
    };
    if d < min || max.is_some_and(|m| d > m) {
        let message = match max {
            Some(m) => format!(
                "'{}' must be between {} and {}",
                name,
                double_to_string(min),
                double_to_string(m)
            ),
            None => format!("'{}' must be at least {}", name, double_to_string(min)),
        };
        errors.push(DirectiveError { key, message });
    }
}

/// `Enum.TryParse<T>(s, ignoreCase: true, out _)` for an enum with the given names. Like .NET it
/// accepts a decimal integer (even one that names no member) and a comma separated list of names.
pub fn enum_try_parse(names: &[&str], s: &str) -> bool {
    let t = s.trim_matches(crate::text::is_white_space);
    if t.is_empty() {
        return false;
    }
    let first = t.chars().next().unwrap();
    if first.is_ascii_digit() || first == '-' || first == '+' {
        return t.parse::<i32>().is_ok();
    }
    t.split(',').all(|part| {
        let p = part.trim_matches(crate::text::is_white_space);
        !p.is_empty() && names.iter().any(|n| eq_ignore_case(n, p))
    })
}
