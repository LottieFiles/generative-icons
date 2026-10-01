//! File and JSON helpers shared by the packaging steps.

use crate::error::{PackageError, Result, invalid};
use gai_core::to_py_json;
use serde_json::{Map, Value};
use std::path::Path;

pub(crate) fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| PackageError::io(path, e))
}

pub(crate) fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&read(path)?).map_err(|source| PackageError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// Writes `bytes`, creating missing parent folders.
pub(crate) fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| PackageError::io(parent, e))?;
    }
    std::fs::write(path, bytes).map_err(|e| PackageError::io(path, e))
}

/// Python `json.dumps(value)`: `", "`/`": "` separators, ASCII-only escapes.
pub fn py_compact(value: &Value) -> String {
    to_py_json(value, None, true)
}

/// Python `json.dumps(value, indent=2)` without a trailing newline.
pub fn py_pretty(value: &Value) -> String {
    to_py_json(value, Some(2), true)
}

/// Python truthiness of a parsed JSON value.
pub(crate) fn py_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
    }
}

/// `value[key]` for a JSON object, or an error naming what was missing (Python's `KeyError`).
pub(crate) fn field<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a Value> {
    value
        .get(key)
        .ok_or_else(|| invalid(format!("{context}: missing \"{key}\"")))
}

pub(crate) fn str_field<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str> {
    field(value, key, context)?
        .as_str()
        .ok_or_else(|| invalid(format!("{context}: \"{key}\" is not a string")))
}

pub(crate) fn array_field<'a>(
    value: &'a Value,
    key: &str,
    context: &str,
) -> Result<&'a Vec<Value>> {
    field(value, key, context)?
        .as_array()
        .ok_or_else(|| invalid(format!("{context}: \"{key}\" is not an array")))
}

pub(crate) fn object<'a>(
    value: &'a mut Value,
    context: &str,
) -> Result<&'a mut Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| invalid(format!("{context} is not an object")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn truthiness_follows_python() {
        for (value, truthy) in [
            (json!(null), false),
            (json!(0), false),
            (json!(0.0), false),
            (json!(""), false),
            (json!([]), false),
            (json!({}), false),
            (json!(1), true),
            (json!("x"), true),
            (json!([0]), true),
        ] {
            assert_eq!(py_truthy(&value), truthy, "{value}");
        }
    }
}
