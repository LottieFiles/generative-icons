//! JavaScript value semantics over parsed JSON, as the ported scripts read their data.
//! A missing property (`undefined`) is `None`.

use crate::jsmath::js_number_to_string;
use serde_json::Value;

/// JS truthiness: `undefined`, `null`, `false`, `0`, `NaN` and `""` are false; `[]` and
/// `{}` are true.
pub fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(_) | Value::Object(_)) => true,
    }
}

/// `String(value)`, as a template literal or `+` concatenation applies it. Numbers print
/// as `Number::toString` of the parsed double, so integers above 2^53 round as in JS.
pub fn to_js_string(value: Option<&Value>) -> String {
    match value {
        None => "undefined".into(),
        Some(Value::Null) => "null".into(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => js_number_to_string(n.as_f64().unwrap_or(f64::NAN)),
        Some(Value::String(s)) => s.clone(),
        // Array.prototype.join: null and undefined items print as "".
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| if item.is_null() { String::new() } else { to_js_string(Some(item)) })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn truthiness_matches_js() {
        for falsy in [None, Some(json!(null)), Some(json!(false)), Some(json!(0)), Some(json!(-0.0)), Some(json!(""))] {
            assert!(!truthy(falsy.as_ref()), "{falsy:?}");
        }
        for value in [json!(true), json!(-1), json!(0.5), json!("0"), json!([]), json!({})] {
            assert!(truthy(Some(&value)), "{value}");
        }
    }

    #[test]
    fn strings_match_js() {
        assert_eq!(to_js_string(None), "undefined");
        assert_eq!(to_js_string(Some(&json!(null))), "null");
        assert_eq!(to_js_string(Some(&json!(true))), "true");
        assert_eq!(to_js_string(Some(&json!(1e21))), "1e+21");
        assert_eq!(to_js_string(Some(&json!(-0.0))), "0");
        assert_eq!(to_js_string(Some(&json!([1, null, "a", [2.5, 3]]))), "1,,a,2.5,3");
        assert_eq!(to_js_string(Some(&json!({"a": 1}))), "[object Object]");
        // String(JSON.parse("9007199254740993")) in V8.
        let big: Value = serde_json::from_str("[9007199254740993,18446744073709551615]").unwrap();
        assert_eq!(to_js_string(Some(&big)), "9007199254740992,18446744073709552000");
    }
}
