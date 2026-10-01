//! The JavaScript semantics the legacy verifiers depend on: truthiness, `Object.is`,
//! `===`, `assert.deepStrictEqual` over parsed JSON, the default `Array.prototype.sort`
//! order and `Number.prototype.toFixed`.
//!
//! A missing property (`undefined`) is `None` throughout.

pub use gai_core::js::{to_js_string, truthy};
use serde_json::Value;
use std::cmp::Ordering;

/// `Object.is` for numbers: `-0` differs from `0`, and NaN equals NaN.
pub fn same_value(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// `Object.is(a, b)` (what `assert.equal` uses). Objects and arrays from separate parses
/// are distinct references, so they never compare equal.
pub fn object_is(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(Value::Number(x)), Some(Value::Number(y))) => same_value(
            x.as_f64().unwrap_or(f64::NAN),
            y.as_f64().unwrap_or(f64::NAN),
        ),
        (Some(Value::Null), Some(Value::Null)) => true,
        (Some(Value::Bool(x)), Some(Value::Bool(y))) => x == y,
        (Some(Value::String(x)), Some(Value::String(y))) => x == y,
        _ => false,
    }
}

/// `a === b`: like [`object_is`] except that `-0 === 0` and `NaN !== NaN`.
pub fn strict_equals(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => x.as_f64() == y.as_f64(),
        _ => object_is(a, b),
    }
}

/// `a` equals the number `b` under `Object.is`.
pub fn is_number(a: Option<&Value>, b: f64) -> bool {
    a.and_then(Value::as_f64).is_some_and(|x| same_value(x, b))
}

/// `assert.deepStrictEqual`: numbers by `Object.is` (JSON `16` and `16.0` are one number),
/// arrays index-wise, objects by key set regardless of order.
pub fn deep_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| deep_equal(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| deep_equal(v, w)))
        }
        _ => object_is(Some(a), Some(b)),
    }
}

/// [`deep_equal`] where either side may be `undefined`.
pub fn deep_equal_opt(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => deep_equal(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// `assert.deepStrictEqual` of two number arrays.
pub fn same_numbers(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| same_value(x, y))
}

/// `deepStrictEqual(a, JSON.parse(JSON.stringify(raw)))` for JSON-sourced `raw`: the round
/// trip only turns each `-0` of `raw` into `0`, so compare with that in place.
pub fn deep_equal_round_trip(a: &Value, raw: &Value) -> bool {
    match (a, raw) {
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| deep_equal_round_trip(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| deep_equal_round_trip(v, w)))
        }
        (Value::Number(x), Value::Number(y)) => {
            let y = y
                .as_f64()
                .map_or(f64::NAN, |y| if y == 0.0 { 0.0 } else { y });
            same_value(x.as_f64().unwrap_or(f64::NAN), y)
        }
        _ => object_is(Some(a), Some(raw)),
    }
}

/// `Number(v)` for JSON primitives: `undefined` is NaN, `null` and `""` are 0, booleans are
/// 0 or 1, and strings parse as decimals. Arrays and objects read as NaN.
pub fn to_number(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => f64::from(u8::from(*b)),
        Some(Value::String(s)) => match s.trim() {
            "" => 0.0,
            "Infinity" | "+Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            t if t
                .bytes()
                .all(|b| b.is_ascii_digit() || b"+-.eE".contains(&b)) =>
            {
                t.parse().unwrap_or(f64::NAN)
            }
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

/// `Array.prototype.sort()` with no comparator: `undefined` last, everything else by
/// `String(v)` in UTF-16 code-unit order. The sort is stable, as V8's is.
pub fn default_sort(values: &mut [Option<Value>]) {
    values.sort_by(|a, b| match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => {
            let (a, b) = (to_js_string(Some(a)), to_js_string(Some(b)));
            a.encode_utf16().cmp(b.encode_utf16())
        }
    });
}

/// `new Set(values)` insertion order under SameValueZero: numbers, strings, booleans and
/// null deduplicate by value (`-0` joins `0`); every array or object is its own entry.
pub fn same_value_zero_set(values: impl IntoIterator<Item = Option<Value>>) -> Vec<Option<Value>> {
    let mut out: Vec<Option<Value>> = Vec::new();
    for v in values {
        let seen = out.iter().any(|o| match (o, &v) {
            (None, None) => true,
            (Some(Value::Number(x)), Some(Value::Number(y))) => {
                let (x, y) = (
                    x.as_f64().unwrap_or(f64::NAN),
                    y.as_f64().unwrap_or(f64::NAN),
                );
                x == y || (x.is_nan() && y.is_nan())
            }
            (Some(Value::Array(_) | Value::Object(_)), _) => false,
            (Some(a), Some(b)) => object_is(Some(a), Some(b)),
            _ => false,
        });
        if !seen {
            out.push(v);
        }
    }
    out
}

/// `x.toFixed(digits)`: the exact decimal value rounded half up at `digits` places;
/// `|x| >= 1e21` falls back to `Number::toString`, and `-0` prints unsigned.
pub fn to_fixed(x: f64, digits: usize) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.abs() >= 1e21 {
        return gai_core::js_number_to_string(x);
    }
    let sign = if x < 0.0 { "-" } else { "" };
    // Every double below 1e21 has an exact decimal expansion within 1100 fractional digits.
    let exact = format!("{:.1100}", x.abs());
    let (int, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut kept: Vec<u8> = int.bytes().chain(frac.bytes().take(digits)).collect();
    if frac.as_bytes().get(digits).is_some_and(|&d| d >= b'5') {
        let mut i = kept.len();
        loop {
            if i == 0 {
                kept.insert(0, b'1');
                break;
            }
            i -= 1;
            if kept[i] == b'9' {
                kept[i] = b'0';
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    let split = kept.len() - digits;
    let text = String::from_utf8(kept).expect("ASCII digits");
    if digits == 0 {
        format!("{sign}{text}")
    } else {
        format!("{sign}{}.{}", &text[..split], &text[split..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deep_equal_is_numeric_and_order_insensitive() {
        let a: Value = serde_json::from_str(r#"{"a":[16,1.5],"b":{"x":1,"y":2}}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"b":{"y":2.0,"x":1},"a":[16.0,1.5]}"#).unwrap();
        assert!(deep_equal(&a, &b));
        assert!(!deep_equal(&json!([0.0]), &json!([-0.0])));
        assert!(!deep_equal(&json!({"a": 1}), &json!({"a": 1, "b": 2})));
    }

    #[test]
    fn json_parse_keeps_negative_zero() {
        for text in ["-0.0", "-0"] {
            let v: Value = serde_json::from_str(text).unwrap();
            assert!(v.as_f64().unwrap().is_sign_negative(), "{text}");
        }
        let raw: Value = serde_json::from_str(r#"{"r":{"k":[{"s":[-0.0]}]}}"#).unwrap();
        let packaged = json!({"r": {"k": [{"s": [0]}]}});
        assert!(!deep_equal(&packaged, &raw));
        assert!(deep_equal_round_trip(&packaged, &raw));
        // A -0 on the packaged side is still a change.
        assert!(!deep_equal_round_trip(&raw, &packaged));
    }

    #[test]
    fn equality_flavours() {
        let (z, nz) = (json!(0.0), serde_json::from_str::<Value>("-0.0").unwrap());
        assert!(!object_is(Some(&z), Some(&nz)));
        assert!(strict_equals(Some(&z), Some(&nz)));
        assert!(object_is(None, None) && strict_equals(None, None));
        assert!(!object_is(Some(&json!([])), Some(&json!([]))));
        assert!(truthy(Some(&json!(-1))) && !truthy(Some(&json!(""))) && !truthy(None));
        assert!(to_number(None).is_nan() && to_number(Some(&Value::Null)) == 0.0);
        assert_eq!(to_number(Some(&json!(" 62.5 "))), 62.5);
        assert!(to_number(Some(&json!("0x"))).is_nan());
    }

    #[test]
    fn default_sort_compares_strings() {
        let mut v: Vec<Option<Value>> = [json!(2.5), json!(10), json!(1.5), json!(2)]
            .into_iter()
            .map(Some)
            .collect();
        v.insert(1, None);
        default_sort(&mut v);
        let text: Vec<String> = v.iter().map(|x| to_js_string(x.as_ref())).collect();
        assert_eq!(text, ["1.5", "10", "2", "2.5", "undefined"]);
        let set = same_value_zero_set([
            Some(json!(2)),
            Some(json!(2.0)),
            Some(json!([1])),
            Some(json!([1])),
        ]);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn to_fixed_matches_js() {
        let cases: [(f64, usize, &str); 12] = [
            (0.125, 2, "0.13"),
            (1.005, 2, "1.00"),
            (-0.0004, 3, "-0.000"),
            (-0.0, 3, "0.000"),
            (2.2, 2, "2.20"),
            (1.9994670370370364, 3, "1.999"),
            (0.5, 0, "1"),
            (-2.5, 0, "-3"),
            (9.9995, 3, "9.999"),
            (99.9996, 3, "100.000"),
            (1e21, 2, "1e+21"),
            (123.456, 1, "123.5"),
        ];
        for (x, d, want) in cases {
            assert_eq!(to_fixed(x, d), want, "({x}).toFixed({d})");
        }
        assert_eq!(to_fixed(f64::NAN, 2), "NaN");
        assert_eq!(
            to_js_string(Some(&json!([3.5, 5.75, 28.5, 22.6]))),
            "3.5,5.75,28.5,22.6"
        );
    }
}
