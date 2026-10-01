//! `project()`: JS values to ordered JSON without QuickJS formatting any number.
//!
//! The host's `tape()` runs `JSON.stringify` with a replacer that swaps every number for
//! `"\0<index>"` and returns the doubles in a `Float64Array`, so key order, dropped
//! `undefined`/functions, `toJSON` and cycle errors are exactly `JSON.stringify`'s. The
//! result equals `serde_json::from_str(JSON.stringify(value))` under V8: integers that V8
//! prints without a fraction or exponent become JSON integers, other numbers stay `f64`.

use crate::runtime::Fail;
use rquickjs::{Function, Object, TypedArray, Value};
use serde_json::Value as Json;

/// `None` when `JSON.stringify` returns `undefined` (for `undefined`, a function or a symbol).
pub(crate) fn project<'js>(host: &Object<'js>, value: Value<'js>) -> Result<Option<Json>, Fail> {
    let tape: Function = host.get("tape")?;
    let out: Object = tape.call((value,))?;
    let text: Value = out.get("text")?;
    let numbers: TypedArray<f64> = out.get("numbers")?;
    let Some(text) = text.as_string() else {
        return Ok(None);
    };
    decode(&text.to_string()?, numbers.as_ref()).map(Some)
}

fn decode(text: &str, numbers: &[f64]) -> Result<Json, Fail> {
    let mut json: Json =
        serde_json::from_str(text).map_err(|error| Fail::Protocol(format!("tape is not JSON: {error}")))?;
    restore(&mut json, numbers)?;
    Ok(json)
}

fn restore(json: &mut Json, numbers: &[f64]) -> Result<(), Fail> {
    match json {
        Json::String(s) => {
            if let Some(rest) = s.strip_prefix('\0') {
                if rest.starts_with('\0') {
                    *s = rest.to_owned();
                } else {
                    let number = rest
                        .parse::<usize>()
                        .ok()
                        .and_then(|index| numbers.get(index))
                        .ok_or_else(|| Fail::Protocol(format!("bad number placeholder {rest:?}")))?;
                    *json = js_number(*number);
                }
            }
        }
        Json::Array(items) => {
            for item in items {
                restore(item, numbers)?;
            }
        }
        Json::Object(map) => {
            for item in map.values_mut() {
                restore(item, numbers)?;
            }
        }
        Json::Null | Json::Bool(_) | Json::Number(_) => {}
    }
    Ok(())
}

/// The JSON value serde reads back from V8's text for `x`.
pub(crate) fn js_number(x: f64) -> Json {
    const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;
    if !x.is_finite() {
        return Json::Null;
    }
    if x.fract() == 0.0 && x.abs() < 1e21 {
        // V8 prints these as plain digits (-0 as "0"); serde keeps them integral when they fit.
        if x == 0.0 {
            return Json::from(0u64);
        }
        if x < 0.0 && x >= i64::MIN as f64 {
            return Json::from(x as i64);
        }
        if x > 0.0 && x < TWO_POW_64 {
            return Json::from(x as u64);
        }
    }
    serde_json::Number::from_f64(x).map_or(Json::Null, Json::Number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_match_serde_reading_v8_text() {
        for (x, text) in [
            (0.0, "0"),
            (-0.0, "0"),
            (16.0, "16"),
            (-3.0, "-3"),
            (0.5, "0.5"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"),
            (-1e19, "-10000000000000000000"),
            (2f64.powi(-24), "5.960464477539063e-8"),
        ] {
            let expected: Json = serde_json::from_str(text).unwrap();
            assert_eq!(js_number(x), expected, "{x:e}");
        }
        assert_eq!(js_number(f64::NAN), Json::Null);
        assert_eq!(js_number(f64::NEG_INFINITY), Json::Null);
    }

    #[test]
    fn placeholders_and_escaped_strings_round_trip() {
        let json = decode(r#"{"a":"\u00000","b":["\u0000\u0000x","\u00001"],"c":"plain"}"#, &[1.5, 2.0]).unwrap();
        assert_eq!(json, serde_json::json!({"a": 1.5, "b": ["\u{0}x", 2], "c": "plain"}));
        assert!(decode(r#"["\u00009"]"#, &[]).is_err());
    }
}
