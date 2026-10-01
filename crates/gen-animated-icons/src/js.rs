//! JavaScript semantics the ported tools relied on.

use serde_json::Value;

/// `JSON.stringify(value, null, 2) + "\n"`: the bytes of every JSON file this tool writes.
pub fn json_file(value: &Value) -> Vec<u8> {
    let mut text = gai_core::to_js_json_pretty(value, 2);
    text.push('\n');
    text.into_bytes()
}

/// `String(value)` for a JSON value, as `Array.prototype.join` stringifies elements.
/// `null` becomes empty, as join does.
pub fn to_js_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => match n.as_i64() {
            Some(i) => i.to_string(),
            None => gai_core::js_number_to_string(n.as_f64().unwrap_or(f64::NAN)),
        },
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(to_js_string).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// `String(value)` as `+` concatenation sees it: `undefined` (`None`) and `null` print
/// their names; everything else as [`to_js_string`].
pub fn js_concat_string(value: Option<&Value>) -> String {
    match value {
        None => "undefined".into(),
        Some(Value::Null) => "null".into(),
        Some(value) => to_js_string(value),
    }
}

/// JavaScript truthiness of a JSON value.
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(false) => false,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Bool(true) | Value::Array(_) | Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn stringifies_like_join() {
        let value = json!([null, 1.5, [2, "x"], {"a": 1}, true]);
        assert_eq!(super::to_js_string(&value), ",1.5,2,x,[object Object],true");
    }

    #[test]
    fn concatenation_names_missing_values() {
        assert_eq!(super::js_concat_string(None), "undefined");
        assert_eq!(super::js_concat_string(Some(&json!(null))), "null");
        assert_eq!(super::js_concat_string(Some(&json!([1, null]))), "1,");
    }

    #[test]
    fn matches_js() {
        for falsy in [json!(null), json!(false), json!(0), json!(-0.0), json!("")] {
            assert!(!super::truthy(&falsy), "{falsy}");
        }
        for truthy in [json!(true), json!(1), json!("0"), json!([]), json!({})] {
            assert!(super::truthy(&truthy), "{truthy}");
        }
    }
}
