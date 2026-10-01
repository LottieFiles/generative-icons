//! Clearance evidence: `JSON.stringify(report, null, 2) + "\n"` of an ordered report.

use serde_json::{Map, Value};

/// Where clearance evidence lives, relative to the set folder.
pub const EVIDENCE_FILE: &str = "evidence/clearance-verification.json";

/// The outcome of a clearance run that reached its evidence write.
#[derive(Debug, Clone, PartialEq)]
pub struct ClearanceReport {
    /// The evidence object, keys in output order.
    pub evidence: Value,
    /// Collected failure messages; empty on PASS.
    pub errors: Vec<String>,
}

impl ClearanceReport {
    /// Whether every check passed.
    pub fn passed(&self) -> bool {
        self.errors.is_empty()
    }

    /// The evidence file bytes.
    pub fn to_json(&self) -> String {
        evidence_json(&self.evidence)
    }
}

/// `JSON.stringify(value, null, 2) + "\n"` of an evidence object.
pub fn evidence_json(value: &Value) -> String {
    let mut out = gai_core::to_js_json_pretty(&js_normalize(value.clone()), 2);
    out.push('\n');
    out
}

/// Rewrites a JSON value the way `JSON.parse` and `JSON.stringify` would print it:
/// every number becomes a double, and object keys that are canonical array indices
/// (`"0"`…`"4294967294"`) move to the front in ascending order.
pub fn js_normalize(value: Value) -> Value {
    match value {
        Value::Number(n) => Value::from(n.as_f64().unwrap_or(f64::NAN)),
        Value::Array(items) => Value::Array(items.into_iter().map(js_normalize).collect()),
        Value::Object(map) => {
            let (mut indexed, named): (Vec<_>, Vec<_>) =
                map.into_iter().partition(|(k, _)| array_index(k).is_some());
            indexed.sort_by_key(|(k, _)| array_index(k));
            let mut out = Map::new();
            for (k, v) in indexed.into_iter().chain(named) {
                out.insert(k, js_normalize(v));
            }
            Value::Object(out)
        }
        other => other,
    }
}

fn array_index(key: &str) -> Option<u32> {
    let canonical = key == "0"
        || (!key.starts_with('0') && !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()));
    key.parse::<u32>()
        .ok()
        .filter(|&i| canonical && i < u32::MAX)
}

/// A JSON number that prints like JS: non-finite values become `null`.
pub fn js_number(x: f64) -> Value {
    Value::from(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stringify_layout_and_numbers() {
        let v = json!({"b": [], "a": {}, "n": [1.0, -0.0, 2.5, 1e21], "s": "°–\"", "i": 9007199254740993u64});
        let mut v = v;
        v["x"] = js_number(f64::INFINITY);
        assert_eq!(
            evidence_json(&v),
            "{\n  \"b\": [],\n  \"a\": {},\n  \"n\": [\n    1,\n    0,\n    2.5,\n    1e+21\n  ],\n  \"s\": \"°–\\\"\",\n  \"i\": 9007199254740992,\n  \"x\": null\n}\n"
        );
    }

    #[test]
    fn index_keys_come_first() {
        let v: Value =
            serde_json::from_str(r#"{"b": 1, "10": 2, "2": 3, "01": 4, "4294967295": 5, "0": 6}"#)
                .unwrap();
        let keys: Vec<String> = js_normalize(v)
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, ["0", "2", "10", "b", "01", "4294967295"]);
    }
}
