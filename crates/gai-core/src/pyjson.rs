//! Python `json.dumps` output, for files the old Python packaging wrote
//! (dotLottie manifests, set.json rewrites). Differences from `JSON.stringify`:
//! `", "`/`": "` separators without indent, `ensure_ascii` escaping (including DEL),
//! and float `repr` (`16.0`, `1e-07`, `1e+16`).

use serde_json::Value;
use std::fmt::Write;

/// `json.dumps(value, indent=indent, ensure_ascii=ensure_ascii)`.
pub fn to_py_json(value: &Value, indent: Option<usize>, ensure_ascii: bool) -> String {
    let mut out = String::new();
    write_value(&mut out, value, indent, ensure_ascii, 0);
    out
}

/// Python `repr(float)` as `json.dumps` writes it: shortest round-trip digits; exponent
/// form when the decimal exponent is < -4 or >= 16; integral values keep a trailing `.0`.
pub fn py_float_repr(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let (digits, exp) = shortest_digits(x.abs());
    let sign = if x < 0.0 { "-" } else { "" };
    if !(-4..16).contains(&exp) {
        let (head, tail) = digits.split_at(1);
        let frac = if tail.is_empty() { String::new() } else { format!(".{tail}") };
        let esign = if exp < 0 { '-' } else { '+' };
        return format!("{sign}{head}{frac}e{esign}{:02}", exp.abs());
    }
    let n = digits.len() as i32;
    let body = if exp < 0 {
        format!("0.{}{}", "0".repeat((-exp - 1) as usize), digits)
    } else if exp + 1 >= n {
        format!("{}{}.0", digits, "0".repeat((exp + 1 - n) as usize))
    } else {
        let (int, frac) = digits.split_at((exp + 1) as usize);
        format!("{int}.{frac}")
    };
    format!("{sign}{body}")
}

/// The shortest digits that round-trip a finite `x > 0`, closest to `x` with ties to even
/// (as CPython's `repr` and ECMAScript's `Number::toString` choose them), and the decimal
/// exponent of the first digit. `format!("{:e}")` breaks those ties upward instead.
fn shortest_digits(x: f64) -> (String, i32) {
    let mut buf = ryu_js::Buffer::new();
    let text = buf.format_finite(x); // "123.45", "0.00012", "1.5e+21" or "5e-324"
    let (mantissa, exp) = match text.split_once('e') {
        Some((mantissa, exp)) => (mantissa, exp.parse::<i32>().expect("ryu exponent")),
        None => (text, 0),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all = format!("{int}{frac}");
    let leading = all.len() - all.trim_start_matches('0').len();
    (all.trim_matches('0').to_owned(), exp + int.len() as i32 - 1 - leading as i32)
}

fn write_string(out: &mut String, s: &str, ensure_ascii: bool) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (ensure_ascii && (c as u32) >= 0x7f) => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn newline(out: &mut String, indent: Option<usize>, depth: usize) {
    if let Some(width) = indent {
        out.push('\n');
        out.extend(std::iter::repeat_n(' ', width * depth));
    }
}

fn write_value(out: &mut String, value: &Value, indent: Option<usize>, ascii: bool, depth: usize) {
    let item_sep = if indent.is_some() { "," } else { ", " };
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                let _ = write!(out, "{i}");
            } else if let Some(u) = n.as_u64() {
                let _ = write!(out, "{u}");
            } else {
                out.push_str(&py_float_repr(n.as_f64().unwrap_or(f64::NAN)));
            }
        }
        Value::String(s) => write_string(out, s, ascii),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                newline(out, indent, depth + 1);
                write_value(out, item, indent, ascii, depth + 1);
            }
            newline(out, indent, depth);
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (key, item)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                newline(out, indent, depth + 1);
                write_string(out, key, ascii);
                out.push_str(": ");
                write_value(out, item, indent, ascii, depth + 1);
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
    fn float_repr_matches_python() {
        for (x, s) in [
            (16.0, "16.0"),
            (0.1, "0.1"),
            (1e-7, "1e-07"),
            (0.0001, "0.0001"),
            (1e16, "1e+16"),
            (123456789012345.6, "123456789012345.6"),
            (2.5e-5, "2.5e-05"),
            (-0.0, "-0.0"),
            (0.8666666666666667, "0.8666666666666667"),
        ] {
            assert_eq!(py_float_repr(x), s);
        }
    }

    #[test]
    fn dumps_matches_python() {
        let v: Value = serde_json::from_str(r#"{"a":[1,2.0],"b":"360°\u007f","c":{}}"#).unwrap();
        assert_eq!(to_py_json(&v, None, true), "{\"a\": [1, 2.0], \"b\": \"360\\u00b0\\u007f\", \"c\": {}}");
        assert_eq!(to_py_json(&v, Some(2), true), "{\n  \"a\": [\n    1,\n    2.0\n  ],\n  \"b\": \"360\\u00b0\\u007f\",\n  \"c\": {}\n}");
    }
}
