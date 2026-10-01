//! ECMAScript number semantics the legacy checks depend on: V8's `Math.hypot`,
//! `Math.min`/`Math.max` (NaN propagation, `-0 < +0`) and `Array.prototype.slice` bounds.

/// V8's `Math.hypot`: max-normalised, Kahan-compensated sum of squares.
///
/// Rust's `f64::hypot` differs from V8 in about 40% of inputs, and `sqrt(x*x + y*y)`
/// in about 42%, so every distance in the checks goes through this.
pub fn hypot(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut max = 0.0f64;
    let mut any_nan = false;
    for &x in xs {
        if x.is_nan() {
            any_nan = true;
        } else if x.abs() > max {
            max = x.abs();
        }
    }
    if max == f64::INFINITY {
        return f64::INFINITY;
    }
    if any_nan {
        return f64::NAN;
    }
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut compensation) = (0.0f64, 0.0f64);
    for &x in xs {
        let n = x.abs() / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// `Math.hypot(x, y)`.
#[inline]
pub fn hypot2(x: f64, y: f64) -> f64 {
    hypot(&[x, y])
}

/// `Math.min(a, b)`: NaN wins, and `-0` is below `+0`.
#[inline]
pub fn min2(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else if b < a {
        b
    } else if a == 0.0 && (a.is_sign_negative() || b.is_sign_negative()) {
        -0.0
    } else {
        a
    }
}

/// `Math.max(a, b)`: NaN wins, and `+0` is above `-0`.
#[inline]
pub fn max2(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else if b > a {
        b
    } else if a == 0.0 && (a.is_sign_positive() || b.is_sign_positive()) {
        0.0
    } else {
        a
    }
}

/// `Math.min(...xs)`; `+Infinity` when empty.
pub fn min_of(xs: impl IntoIterator<Item = f64>) -> f64 {
    xs.into_iter().fold(f64::INFINITY, min2)
}

/// `Math.max(...xs)`; `-Infinity` when empty.
pub fn max_of(xs: impl IntoIterator<Item = f64>) -> f64 {
    xs.into_iter().fold(f64::NEG_INFINITY, max2)
}

/// `xs.reduce((a, b) => a + b, 0)`: a left fold from `+0`.
///
/// `Iterator::sum` for floats may start from `-0.0`, which changes an all-`-0` sum.
pub fn sum(xs: impl IntoIterator<Item = f64>) -> f64 {
    xs.into_iter().fold(0.0, |a, b| a + b)
}

/// The index range `arr.slice(start, end)` selects from an array of `len` items:
/// negative bounds count from the end, both clamp to `[0, len]`, and a missing end is `len`.
pub fn slice_range(len: usize, start: i64, end: Option<i64>) -> std::ops::Range<usize> {
    let len_i = len as i64;
    let clamp = |i: i64| if i < 0 { (len_i + i).max(0) } else { i.min(len_i) } as usize;
    let s = clamp(start);
    let e = end.map_or(len, clamp);
    if s >= e { s..s } else { s..e }
}

/// `arr.at(i)`: negative indices count from the end.
pub fn at_index(len: usize, i: i64) -> Option<usize> {
    let k = if i < 0 { len as i64 + i } else { i };
    (0..len as i64).contains(&k).then_some(k as usize)
}

/// `Number.isInteger(x)`.
pub fn is_integer(x: f64) -> bool {
    x.is_finite() && x.fract() == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(x: f64) -> u64 {
        x.to_bits()
    }

    #[test]
    fn hypot_matches_v8_vectors() {
        // (x, y, Math.hypot(x, y)) captured from Node v26.7.0; several differ from f64::hypot.
        let vectors: [(f64, f64, f64); 20] = [
            (3.0, 4.0, 5.0),
            (0.1, 0.2, 0.223606797749979),
            (1.0, 1.0, std::f64::consts::SQRT_2),
            (-2.5, 0.0003, 2.5000000179999997),
            (12.34567, -7.654321, 14.52598353709452),
            (0.30000000000000004, 0.1, 0.316227766016838),
            (1e-300, 1e-300, 1.4142135623730952e-300),
            (
                std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
                1.0000000000000002,
            ),
            (6.21158754825592, -12.847053527832031, 14.269919558870312),
            (2.7066879272460938, 31.17388153076172, 31.29116567386818),
            (11.019866943359375, 1.868988037109375, 11.177235066518135),
            (23.673187255859375, 26.580902099609375, 35.59443989276978),
            (-27.228179931640625, 31.831880569458008, 41.888451904767685),
            (-18.58343505859375, -19.625303268432617, 27.02770036379202),
            (31.53664779663086, -28.221817016601562, 42.32057549191981),
            (-7.41987419128418, -30.06377410888672, 30.965869060703792),
            (-27.39241623878479, 11.115792274475098, 29.56188940660121),
            (27.136863708496094, 19.81304168701172, 33.600089178821484),
            (15.438316345214844, -27.92237091064453, 31.90611867411227),
            (7.559595108032227, -0.2586517333984375, 7.564018701495574),
        ];
        for (x, y, expected) in vectors {
            assert_eq!(bits(hypot2(x, y)), bits(expected), "hypot({x}, {y})");
        }
        assert_eq!(hypot(&[]), 0.0);
        assert!(hypot(&[f64::NAN, 1.0]).is_nan());
        assert_eq!(hypot(&[f64::NAN, f64::NEG_INFINITY]), f64::INFINITY);
        assert_eq!(bits(hypot(&[-0.0, 0.0])), bits(0.0));
        assert_eq!(hypot(&[1.0, 2.0, 2.0]), 3.0);
    }

    #[test]
    fn min_max_follow_math_min_max() {
        assert!(min2(f64::NAN, 1.0).is_nan());
        assert!(max2(1.0, f64::NAN).is_nan());
        assert_eq!(bits(min2(0.0, -0.0)), bits(-0.0));
        assert_eq!(bits(min2(-0.0, 0.0)), bits(-0.0));
        assert_eq!(bits(max2(-0.0, 0.0)), bits(0.0));
        assert_eq!(bits(max2(0.0, -0.0)), bits(0.0));
        assert_eq!(min_of([]), f64::INFINITY);
        assert_eq!(max_of([]), f64::NEG_INFINITY);
        assert_eq!(min_of([3.0, 1.0, 2.0]), 1.0);
        assert!(max_of([3.0, f64::NAN, 2.0]).is_nan());
        assert_eq!(bits(sum([-0.0, -0.0])), bits(0.0));
    }

    #[test]
    fn slice_and_at_follow_js() {
        assert_eq!(slice_range(10, -2, Some(4)), 8..8);
        assert_eq!(slice_range(10, 2, None), 2..10);
        assert_eq!(slice_range(10, -4, Some(-2)), 6..8);
        assert_eq!(slice_range(3, 0, Some(99)), 0..3);
        assert_eq!(slice_range(3, 5, Some(1)), 3..3);
        assert_eq!(at_index(4, -1), Some(3));
        assert_eq!(at_index(4, 4), None);
        assert_eq!(at_index(4, -5), None);
        assert!(is_integer(2.0) && !is_integer(1.5) && !is_integer(f64::INFINITY));
    }
}
