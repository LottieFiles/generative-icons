/// ECMAScript `Math.round`: nearest integer, ties toward +∞, keeping the sign of zero.
///
/// Rust's `f64::round` rounds ties away from zero (`-2.5 → -3`), and the naive
/// `(x + 0.5).floor()` is wrong for `0.49999999999999994`. This follows the spec.
pub fn js_round(x: f64) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    if (0.0..0.5).contains(&x) {
        return 0.0;
    }
    if (-0.5..0.0).contains(&x) {
        return -0.0;
    }
    let floor = x.floor();
    if x - floor >= 0.5 { floor + 1.0 } else { floor }
}

/// The builders' `round`: `Math.round(value * 1e5) / 1e5`.
pub fn round5(value: f64) -> f64 {
    js_round(value * 1e5) / 1e5
}

/// ECMAScript `Number::toString(x)` (what `String(x)` and `JSON.stringify` print).
/// `NaN`/`±Infinity` print as in JS; `JSON.stringify` maps them to `null` instead.
pub fn js_number_to_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    let mut buf = ryu_js::Buffer::new();
    buf.format(x).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_round_matches_ecmascript() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(-2.6), -3.0);
        assert_eq!(js_round(0.49999999999999994), 0.0);
        assert!(js_round(-0.2).is_sign_negative() && js_round(-0.2) == 0.0);
        assert!(js_round(-0.0).is_sign_negative());
        assert_eq!(js_round(4503599627370497.0), 4503599627370497.0);
    }

    #[test]
    fn round5_matches_builders() {
        assert_eq!(round5(22.075141), 22.07514);
        assert_eq!(round5(9.924865), 9.92487);
        assert_eq!(js_number_to_string(round5(0.496170001)), "0.49617");
    }

    #[test]
    fn number_to_string_matches_v8() {
        for (x, s) in [
            (16.0, "16"),
            (-0.0, "0"),
            (0.1 + 0.2, "0.30000000000000004"),
            (1e21, "1e+21"),
            (1e-7, "1e-7"),
            (123456789012345680000.0, "123456789012345680000"),
            (0.016666666666666666, "0.016666666666666666"),
            (5.2730002, "5.2730002"),
            (-0.0027049984501246894, "-0.0027049984501246894"),
        ] {
            assert_eq!(js_number_to_string(x), s, "{x:e}");
        }
    }
}
