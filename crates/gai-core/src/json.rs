//! `JSON.stringify` output for `serde_json::Value`s parsed with `preserve_order`.

use crate::jsmath::js_number_to_string;
use serde_json::Value;
use std::fmt::Write;

/// `JSON.stringify(value)`.
pub fn to_js_json(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, None, 0);
    out
}

/// `JSON.stringify(value, null, indent)`. The pipeline adds its own trailing newline.
pub fn to_js_json_pretty(value: &Value, indent: usize) -> String {
    let mut out = String::new();
    write_value(&mut out, value, Some(indent), 0);
    out
}

/// 2^53: integers up to this magnitude survive `JSON.parse` exactly.
const MAX_EXACT: u64 = 1 << 53;

fn write_number(out: &mut String, n: &serde_json::Number) {
    match (n.as_i64(), n.as_u64()) {
        (Some(i), _) if i.unsigned_abs() <= MAX_EXACT => {
            let _ = write!(out, "{i}");
        }
        (_, Some(u)) if u <= MAX_EXACT => {
            let _ = write!(out, "{u}");
        }
        // Larger integers become the nearest double, as `JSON.parse` makes them.
        _ => {
            let f = n.as_f64().unwrap_or(f64::NAN);
            if f.is_finite() { out.push_str(&js_number_to_string(f)) } else { out.push_str("null") }
        }
    }
}

fn write_string(out: &mut String, s: &str) {
    // serde_json escapes exactly like JSON.stringify for well-formed UTF-8:
    // `"`, `\`, \b \f \n \r \t, other C0 controls as \u00xx, nothing else.
    out.push_str(&serde_json::to_string(s).expect("string serializes"));
}

fn newline(out: &mut String, indent: Option<usize>, depth: usize) {
    if let Some(width) = indent {
        out.push('\n');
        out.extend(std::iter::repeat_n(' ', width * depth));
    }
}

fn write_value(out: &mut String, value: &Value, indent: Option<usize>, depth: usize) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => write_number(out, n),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent, depth + 1);
                write_value(out, item, indent, depth + 1);
            }
            newline(out, indent, depth);
            out.push(']');
        }
        Value::Object(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (key, item)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent, depth + 1);
                write_string(out, key);
                out.push(':');
                if indent.is_some() {
                    out.push(' ');
                }
                write_value(out, item, indent, depth + 1);
            }
            newline(out, indent, depth);
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_matches_json_stringify() {
        let v: Value = serde_json::from_str(r#"{"b":1.0,"a":[0.0,-0.0,1e21,"x\u0001\n\"/é"],"c":{},"d":[]}"#).unwrap();
        assert_eq!(to_js_json(&v), r#"{"b":1,"a":[0,0,1e+21,"x\u0001\n\"/é"],"c":{},"d":[]}"#);
    }

    #[test]
    fn integers_beyond_2_pow_53_round_like_json_parse() {
        // JSON.stringify(JSON.parse(text)) in V8.
        let v: Value = serde_json::from_str(
            "[9007199254740992,9007199254740993,-9007199254740993,1152921504606846976,18446744073709551615,-9223372036854775808]",
        )
        .unwrap();
        assert_eq!(
            to_js_json(&v),
            "[9007199254740992,9007199254740992,-9007199254740992,1152921504606847000,18446744073709552000,-9223372036854776000]"
        );
    }

    #[test]
    fn pretty_matches_json_stringify() {
        let v: Value = serde_json::from_str(r#"{"a":[1,{"b":[]}],"c":{}}"#).unwrap();
        assert_eq!(to_js_json_pretty(&v, 2), "{\n  \"a\": [\n    1,\n    {\n      \"b\": []\n    }\n  ],\n  \"c\": {}\n}");
    }
}
