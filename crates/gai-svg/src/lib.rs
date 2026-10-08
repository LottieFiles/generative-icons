//! Animated SVG export: a Generative Icons Lottie animation becomes an SVG that browsers
//! animate with SMIL, without a player or script.
//!
//! This is a port of `packages/runtime/src/svg.mjs` and stays byte-identical with it, the
//! way the rest of the workspace matches its JavaScript references: the same arithmetic
//! in the same order, JavaScript's `toFixed`/`toString` and V8's `Math.hypot`.
//!
//! It converts exactly the Lottie the icon compiler emits (`gai-spec`'s `plan.rs`): shape
//! layers with `sh`/`rc`/`el` paths, stroke, fill and trim, parenting, layer opacity,
//! position (split or not), rotation about the anchor, path keyframes, trim end, subtract
//! masks and alpha-inverted mattes, bezier and hold keys, and the family's colour, style
//! and weight slots. Anything else is an error, never a silent approximation.
//!
//! Static attributes hold the rest pose, so renderers without SMIL show the icon at rest.
//!
//! Browser traps the output avoids (each was found by diffing against dotLottie):
//! - keyTimes that decrease after rounding make Chrome drop the animation, and engines
//!   read equal keyTimes differently: holds end in a `1e-6` ramp rounded down;
//! - `-` and `.` inside an id break SMIL syncbase and event references;
//! - WebKit drops a cubic whose ends coincide, joins included: such segments are halved;
//! - a trim end of exactly 0 still draws a round-cap dot: the stroke is hidden then;
//! - keys with different vertex counts stop interpolation: short keys are padded;
//! - renderers without SMIL often ignore `pathLength`: no dash is left on at rest.

use serde_json::Value;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::Write;

const KAPPA: f64 = 0.5519150244935105;

/// Why an SVG could not be made.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SvgError {
    /// The animation uses Lottie the exporter doesn't convert.
    #[error("Unsupported Lottie for SVG export: {0}")]
    Unsupported(String),
    /// An option is out of range, or doesn't fit the icon.
    #[error("{0}")]
    Options(String),
    /// The icon record or an animation is missing something.
    #[error("{0}")]
    Invalid(String),
}

type Result<T> = std::result::Result<T, SvgError>;

fn unsupported(what: &str) -> SvgError {
    SvgError::Unsupported(what.to_string())
}

/// Icon styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Outline strokes (Light, Regular or Bold by weight).
    Outline,
    /// Filled shapes.
    Solid,
    /// Outline over an 18% tint.
    Duotone,
}

impl Style {
    /// `outline`, `solid` or `duotone`.
    pub fn name(self) -> &'static str {
        match self {
            Style::Outline => "outline",
            Style::Solid => "solid",
            Style::Duotone => "duotone",
        }
    }

    /// Parses a style name.
    pub fn parse(name: &str) -> Option<Style> {
        [Style::Outline, Style::Solid, Style::Duotone].into_iter().find(|s| s.name() == name)
    }
}

/// How the SVG starts playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// `toggle` for stateful icons, `loop` for loaders, `hover` otherwise.
    Auto,
    /// The action plays when the pointer enters (inline SVG only).
    Hover,
    /// The action plays on click (inline SVG only).
    Click,
    /// Stateful icons: each click plays the other direction (inline SVG only).
    Toggle,
    /// The whole timeline repeats.
    Loop,
    /// The action plays once when the SVG loads.
    Load,
    /// Script starts it with `beginElement()` (inline SVG only).
    Js,
}

impl Trigger {
    /// Every trigger, `auto` first.
    pub const ALL: [Trigger; 7] =
        [Trigger::Auto, Trigger::Hover, Trigger::Click, Trigger::Toggle, Trigger::Loop, Trigger::Load, Trigger::Js];

    /// The trigger's name.
    pub fn name(self) -> &'static str {
        match self {
            Trigger::Auto => "auto",
            Trigger::Hover => "hover",
            Trigger::Click => "click",
            Trigger::Toggle => "toggle",
            Trigger::Loop => "loop",
            Trigger::Load => "load",
            Trigger::Js => "js",
        }
    }

    /// Parses a trigger name.
    pub fn parse(name: &str) -> Option<Trigger> {
        Trigger::ALL.into_iter().find(|t| t.name() == name)
    }
}

/// How an icon is exported. `Default` matches the runtime's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgOptions {
    /// `light` or `strong`.
    pub motion: String,
    /// The style.
    pub style: Style,
    /// Outline weight from 1 to 3.
    pub weight: f64,
    /// `currentColor` or a hex value.
    pub color: String,
    /// The trigger.
    pub trigger: Trigger,
    /// Prefix of the SVG's element ids; the icon id when `None`.
    pub id: Option<String>,
    /// Width in CSS pixels; the icon's own width when `None`.
    pub size: Option<f64>,
    /// Stateful icons: the state shown first; the icon's `endState` when `None`.
    pub state: Option<String>,
}

impl Default for SvgOptions {
    fn default() -> SvgOptions {
        SvgOptions {
            motion: "light".into(),
            style: Style::Outline,
            weight: 2.0,
            color: "currentColor".into(),
            trigger: Trigger::Auto,
            id: None,
            size: None,
            state: None,
        }
    }
}

// ---------- numbers and paths ----------

/// ECMAScript `Number.prototype.toFixed(dp)` read back as a number: the nearest multiple
/// of `10^-dp`, exact ties away from zero (Rust's formatting rounds ties to even).
fn to_fixed(x: f64, dp: usize) -> f64 {
    if !x.is_finite() || x.abs() >= 1e21 {
        return x;
    }
    let a = x.abs();
    let text = if is_tie(a, dp) {
        // `a` has exactly dp + 1 decimals ending in 5: drop the 5 and round up.
        let exact = format!("{a:.prec$}", prec = dp + 1);
        let mut digits: Vec<u8> = exact[..exact.len() - 1].bytes().collect();
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, b'1');
                break;
            }
            i -= 1;
            match digits[i] {
                b'.' => continue,
                b'9' => digits[i] = b'0',
                d => {
                    digits[i] = d + 1;
                    break;
                }
            }
        }
        String::from_utf8(digits).expect("ascii digits")
    } else {
        format!("{a:.dp$}")
    };
    let value: f64 = text.parse().expect("formatted number");
    if x < 0.0 { -value } else { value }
}

/// Whether `a * 10^dp` lies exactly halfway between two integers: `a * 2 * 10^dp` is odd.
fn is_tie(a: f64, dp: usize) -> bool {
    if a == 0.0 {
        return false;
    }
    let bits = a.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = bits & ((1u64 << 52) - 1);
    let (m, e) = if exponent == 0 { (fraction, -1074) } else { (fraction | (1u64 << 52), exponent - 1075) };
    // a * 2 * 10^dp = m * 5^dp * 2^(e + dp + 1); 5^dp is odd.
    let k = e + dp as i64 + 1;
    match k {
        0 => m & 1 == 1,
        k if k < 0 => i64::from(m.trailing_zeros()) == -k,
        _ => false,
    }
}

/// `num` of svg.mjs: `toFixed(dp)`, printed the JavaScript way, without a leading zero.
fn num(n: f64, dp: usize) -> String {
    let mut s = gai_core::js_number_to_string(to_fixed(n, dp));
    if s == "-0" {
        s = "0".into();
    }
    if let Some(rest) = s.strip_prefix("0.") {
        return format!(".{rest}");
    }
    if let Some(rest) = s.strip_prefix("-0.") {
        return format!("-.{rest}");
    }
    s
}

fn n3(n: f64) -> String {
    num(n, 3)
}

fn pt(x: f64, y: f64) -> String {
    format!("{} {}", n3(x), n3(y)).replacen(" -", "-", 1)
}

/// V8's `Math.hypot`: Kahan-compensated sum of squares scaled by the largest magnitude.
fn js_hypot(values: &[f64]) -> f64 {
    let max = values.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    if max == 0.0 {
        return 0.0;
    }
    if max.is_infinite() {
        return f64::INFINITY;
    }
    let (mut sum, mut compensation) = (0.0, 0.0);
    for v in values {
        let n = v.abs() / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

fn f(value: &Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

fn at(value: &Value, index: usize) -> f64 {
    f(&value[index])
}

/// A `{v, i, o}` shape as `[vx, vy, ix, iy, ox, oy, ...]`, linear in the key.
fn flat_shape(sh: &Value) -> Vec<f64> {
    let (v, i, o) = (&sh["v"], &sh["i"], &sh["o"]);
    let n = v.as_array().map_or(0, Vec::len);
    let mut out = Vec::with_capacity(n * 6);
    for k in 0..n {
        out.extend([at(&v[k], 0), at(&v[k], 1), at(&i[k], 0), at(&i[k], 1), at(&o[k], 0), at(&o[k], 1)]);
    }
    out
}

fn shape_conv(s: &Value) -> Vec<f64> {
    flat_shape(if s.is_array() { &s[0] } else { s })
}

fn scalar(k: &Value) -> Vec<f64> {
    vec![if k.is_array() { at(k, 0) } else { f(k) }]
}

fn vec2(k: &Value) -> Vec<f64> {
    vec![at(k, 0), at(k, 1)]
}

fn shape_len(key: &Value) -> usize {
    key["s"][0]["v"].as_array().map_or(0, Vec::len)
}

/// Pads shorter keys of a path with a zero-length vertex (a copy of the last one that
/// takes over its out-tangent), so every key has one command list.
fn pad_morph(prop: &Value) -> Cow<'_, Value> {
    if prop["a"].as_f64() != Some(1.0) {
        return Cow::Borrowed(prop);
    }
    let keys = prop["k"].as_array().map(Vec::as_slice).unwrap_or_default();
    let with_shape = || keys.iter().filter(|k| !k["s"].is_null());
    let Some(n) = with_shape().map(shape_len).max() else { return Cow::Borrowed(prop) };
    if with_shape().all(|k| shape_len(k) == n) {
        return Cow::Borrowed(prop);
    }
    let mut padded = prop.clone();
    for key in padded["k"].as_array_mut().into_iter().flatten() {
        if key["s"].is_null() {
            continue;
        }
        let sh = &mut key["s"][0];
        while sh["v"].as_array().map_or(0, Vec::len) < n {
            let j = sh["v"].as_array().map_or(0, Vec::len) - 1;
            let (v, o) = (sh["v"][j].clone(), sh["o"][j].clone());
            sh["v"].as_array_mut().expect("array").push(v);
            sh["i"].as_array_mut().expect("array").push(serde_json::json!([0, 0]));
            sh["o"].as_array_mut().expect("array").push(o);
            sh["o"][j] = serde_json::json!([0, 0]);
        }
    }
    Cow::Owned(padded)
}

/// Segments whose ends meet in any key (the round end of a collapsed pill). WebKit drops
/// such a cubic together with its joins, so these are written as two halves.
fn hairpins<'a>(keys: impl Iterator<Item = &'a Vec<f64>>, closed: bool) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    for flat in keys {
        let n = flat.len() / 6;
        let last = if closed { n } else { n.saturating_sub(1) };
        for a in 0..last {
            let b = (a + 1) % n;
            if js_hypot(&[flat[6 * a] - flat[6 * b], flat[6 * a + 1] - flat[6 * b + 1]]) < 0.01 {
                out.insert(a);
            }
        }
    }
    out
}

/// Every segment of a keyed path is a `C`, so all keys share one command list.
fn shape_d(flat: &[f64], closed: bool, force_curves: bool, split: &BTreeSet<usize>) -> String {
    let n = flat.len() / 6;
    if n == 0 {
        return String::new();
    }
    let v = |k: usize| (flat[6 * k], flat[6 * k + 1]);
    let i = |k: usize| (flat[6 * k + 2], flat[6 * k + 3]);
    let o = |k: usize| (flat[6 * k + 4], flat[6 * k + 5]);
    let (x0, y0) = v(0);
    let mut d = format!("M{}", pt(x0, y0));
    let seg = |a: usize, b: usize| -> String {
        let ((ax, ay), (bx, by), (ox, oy), (ix, iy)) = (v(a), v(b), o(a), i(b));
        if split.contains(&a) {
            // de Casteljau at 0.5: exact, and linear in the control points, so morphs stay exact
            let (p1, p2) = ((ax + ox, ay + oy), (bx + ix, by + iy));
            let m = |u: (f64, f64), w: (f64, f64)| ((u.0 + w.0) / 2.0, (u.1 + w.1) / 2.0);
            let (q0, q1, q2) = (m((ax, ay), p1), m(p1, p2), m(p2, (bx, by)));
            let (r0, r1) = (m(q0, q1), m(q1, q2));
            let c = m(r0, r1);
            return format!(
                "C{} {} {}C{} {} {}",
                pt(q0.0, q0.1),
                pt(r0.0, r0.1),
                pt(c.0, c.1),
                pt(r1.0, r1.1),
                pt(q2.0, q2.1),
                pt(bx, by)
            );
        }
        if !force_curves && ox == 0.0 && oy == 0.0 && ix == 0.0 && iy == 0.0 {
            return format!("L{}", pt(bx, by));
        }
        format!("C{} {} {}", pt(ax + ox, ay + oy), pt(bx + ix, by + iy), pt(bx, by))
    };
    for k in 0..n - 1 {
        d.push_str(&seg(k, k + 1));
    }
    if closed {
        d.push_str(&seg(n - 1, 0));
        d.push('Z');
    }
    d
}

/// lottie-web's rect path (direction 1), so trims start where Lottie's do.
fn rect_shape(p: &Value, s: &Value, r: f64) -> Vec<f64> {
    let (p0, p1, v0, v1) = (at(p, 0), at(p, 1), at(s, 0) / 2.0, at(s, 1) / 2.0);
    let round = v0.min(v1).min(r);
    let c = round * (1.0 - KAPPA);
    let mut t = Vec::new();
    let mut add = |vx: f64, vy: f64, ox: f64, oy: f64, ix: f64, iy: f64| {
        t.extend([vx, vy, ix - vx, iy - vy, ox - vx, oy - vy]);
    };
    if round != 0.0 {
        add(p0 + v0, p1 - v1 + round, p0 + v0, p1 - v1 + round, p0 + v0, p1 - v1 + c);
        add(p0 + v0, p1 + v1 - round, p0 + v0, p1 + v1 - c, p0 + v0, p1 + v1 - round);
        add(p0 + v0 - round, p1 + v1, p0 + v0 - round, p1 + v1, p0 + v0 - c, p1 + v1);
        add(p0 - v0 + round, p1 + v1, p0 - v0 + c, p1 + v1, p0 - v0 + round, p1 + v1);
        add(p0 - v0, p1 + v1 - round, p0 - v0, p1 + v1 - round, p0 - v0, p1 + v1 - c);
        add(p0 - v0, p1 - v1 + round, p0 - v0, p1 - v1 + c, p0 - v0, p1 - v1 + round);
        add(p0 - v0 + round, p1 - v1, p0 - v0 + round, p1 - v1, p0 - v0 + c, p1 - v1);
        add(p0 + v0 - round, p1 - v1, p0 + v0 - c, p1 - v1, p0 + v0 - round, p1 - v1);
    } else {
        add(p0 + v0, p1 - v1, p0 + v0, p1 - v1, p0 + v0, p1 - v1);
        add(p0 + v0, p1 + v1, p0 + v0, p1 + v1, p0 + v0, p1 + v1);
        add(p0 - v0, p1 + v1, p0 - v0, p1 + v1, p0 - v0, p1 + v1);
        add(p0 - v0, p1 - v1, p0 - v0, p1 - v1, p0 - v0, p1 - v1);
    }
    t
}

/// lottie-web's ellipse path (direction 1).
fn ellipse_shape(p: &Value, s: &Value) -> Vec<f64> {
    let (x, y, rx, ry) = (at(p, 0), at(p, 1), at(s, 0) / 2.0, at(s, 1) / 2.0);
    let (cx, cy) = (rx * KAPPA, ry * KAPPA);
    vec![
        x, y - ry, -cx, 0.0, cx, 0.0, x + rx, y, 0.0, -cy, 0.0, cy, x, y + ry, cx, 0.0, -cx, 0.0, x - rx, y, 0.0, cy,
        0.0, -cy,
    ]
}

// ---------- easing ----------

type Ease = [f64; 4];
const LINEAR: Ease = [0.0, 0.0, 1.0, 1.0];

fn bz(p0: f64, p1: f64, p2: f64, p3: f64, u: f64) -> f64 {
    let m = 1.0 - u;
    m * m * m * p0 + 3.0 * m * m * u * p1 + 3.0 * m * u * u * p2 + u * u * u * p3
}

fn dbz(p0: f64, p1: f64, p2: f64, p3: f64, u: f64) -> f64 {
    let m = 1.0 - u;
    3.0 * m * m * (p1 - p0) + 6.0 * m * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

/// The curve parameter where `x(u) = x`; `x` is monotonic.
fn solve_u(c: &Ease, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut u = x;
    for _ in 0..8 {
        let fx = bz(0.0, c[0], c[2], 1.0, u) - x;
        let d = dbz(0.0, c[0], c[2], 1.0, u);
        if fx.abs() < 1e-9 {
            return u;
        }
        if d.abs() < 1e-6 {
            break;
        }
        u -= fx / d;
        if !(0.0..=1.0).contains(&u) {
            break;
        }
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    u = x;
    for _ in 0..60 {
        let fx = bz(0.0, c[0], c[2], 1.0, u);
        if (fx - x).abs() < 1e-10 {
            break;
        }
        if fx < x {
            lo = u;
        } else {
            hi = u;
        }
        u = (lo + hi) / 2.0;
    }
    u
}

type Point = [f64; 2];

/// The part of the easing curve between parameters `ua` and `ub` (de Casteljau).
fn sub_curve(c: &Ease, ua: f64, ub: f64) -> [Point; 4] {
    let split = |q: [Point; 4], u: f64| -> ([Point; 4], [Point; 4]) {
        let l = |a: Point, b: Point| [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
        let (a, b, cc) = (l(q[0], q[1]), l(q[1], q[2]), l(q[2], q[3]));
        let (d, e) = (l(a, b), l(b, cc));
        let f = l(d, e);
        ([q[0], a, d, f], [f, e, cc, q[3]])
    };
    let mut q = [[0.0, 0.0], [c[0], c[1]], [c[2], c[3]], [1.0, 1.0]];
    if ub < 1.0 {
        q = split(q, ub).0;
    }
    if ua > 0.0 {
        q = split(q, ua / ub).1;
    }
    q
}

fn y_extrema(c: &Ease) -> Vec<f64> {
    let (p0, p1, p2, p3) = (0.0, c[1], c[3], 1.0);
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let cc = p1 - p0;
    let mut r = Vec::new();
    if a.abs() < 1e-12 {
        if b.abs() > 1e-12 {
            r.push(-cc / b);
        }
    } else {
        let disc = b * b - 4.0 * a * cc;
        if disc >= 0.0 {
            let s = disc.sqrt();
            r.push((-b + s) / (2.0 * a));
            r.push((-b - s) / (2.0 * a));
        }
    }
    r.retain(|&u| u > 1e-6 && u < 1.0 - 1e-6);
    r.sort_by(|x, y| x.partial_cmp(y).expect("finite"));
    r
}

/// keySplines must lie in 0-1. Overshooting curves are cut at their value extrema, then
/// halved until every piece's normalized control points fit; each piece is exact.
fn valid_pieces(c: &Ease, u0: f64, u1: f64, depth: u32) -> Vec<(f64, f64, Ease)> {
    let q = sub_curve(c, u0, u1);
    let (dx, dy) = (q[3][0] - q[0][0], q[3][1] - q[0][1]);
    if dx <= 1e-12 || dy.abs() < 1e-12 {
        return vec![(u0, u1, LINEAR)];
    }
    let n = |p: Point| [(p[0] - q[0][0]) / dx, (p[1] - q[0][1]) / dy];
    let (a, b) = (n(q[1]), n(q[2]));
    let ok = [a[0], a[1], b[0], b[1]].iter().all(|&v| (-1e-9..=1.0 + 1e-9).contains(&v));
    if ok || depth >= 6 {
        let spl = [a[0], a[1], b[0], b[1]].map(|v| js_min(1.0, js_max(0.0, v)));
        let linear = (spl[0] - spl[1]).abs() < 1e-6 && (spl[2] - spl[3]).abs() < 1e-6;
        return vec![(u0, u1, if linear { LINEAR } else { spl })];
    }
    let ext: Vec<f64> =
        if depth == 0 { y_extrema(c).into_iter().filter(|&u| u > u0 + 1e-9 && u < u1 - 1e-9).collect() } else { vec![] };
    if !ext.is_empty() {
        let mut cuts = vec![u0];
        cuts.extend(ext);
        cuts.push(u1);
        return cuts.windows(2).flat_map(|w| valid_pieces(c, w[0], w[1], 1)).collect();
    }
    let m = (u0 + u1) / 2.0;
    let mut out = valid_pieces(c, u0, m, depth + 1);
    out.extend(valid_pieces(c, m, u1, depth + 1));
    out
}

/// `Math.max(a, b)` for numbers that are never NaN: `+0` beats `-0`.
fn js_max(a: f64, b: f64) -> f64 {
    if a == b { if a.is_sign_negative() { b } else { a } } else if a > b { a } else { b }
}

/// `Math.min(a, b)` for numbers that are never NaN: `-0` beats `+0`.
fn js_min(a: f64, b: f64) -> f64 {
    if a == b { if a.is_sign_negative() { a } else { b } } else if a < b { a } else { b }
}

// ---------- timelines ----------

#[derive(Debug, Clone)]
struct Seg {
    t0: f64,
    t1: f64,
    v0: Vec<f64>,
    v1: Vec<f64>,
    /// `None` for a hold.
    ease: Option<Ease>,
}

#[derive(Debug, Clone)]
enum Timeline {
    Constant(Vec<f64>),
    Keyed { first: (f64, Vec<f64>), last: (f64, Vec<f64>), segs: Vec<Seg> },
}

fn first_of(x: &Value) -> f64 {
    if x.is_array() { at(x, 0) } else { f(x) }
}

fn timeline(prop: &Value, conv: &dyn Fn(&Value) -> Vec<f64>) -> Timeline {
    if prop["a"].as_f64() != Some(1.0) {
        return Timeline::Constant(conv(&prop["k"]));
    }
    // Engine exports carry f32 noise in key times (28.00002, 4.7999997): snap to 1e-4 frame.
    let keys: Vec<(f64, &Value)> = prop["k"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|k| (gai_core::js_round(f(&k["t"]) * 1e4) / 1e4, k))
        .collect();
    if keys.is_empty() {
        return Timeline::Constant(Vec::new());
    }
    let mut segs = Vec::new();
    for j in 0..keys.len().saturating_sub(1) {
        let ((t0, a), (t1, b)) = (keys[j], keys[j + 1]);
        let v0 = conv(&a["s"]);
        let v1 = conv(if a["e"].is_null() { &b["s"] } else { &a["e"] });
        if a["h"].as_f64() == Some(1.0) {
            segs.push(Seg { t0, t1, v1: v0.clone(), v0, ease: None });
        } else {
            let ease = [first_of(&a["o"]["x"]), first_of(&a["o"]["y"]), first_of(&a["i"]["x"]), first_of(&a["i"]["y"])];
            segs.push(Seg { t0, t1, v0, v1, ease: Some(ease) });
        }
    }
    let (t_first, k_first) = keys[0];
    let (t_last, k_last) = keys[keys.len() - 1];
    let last = match keys.len() {
        n if n >= 2 && k_last["s"].is_null() => conv(&keys[n - 2].1["e"]),
        _ => conv(&k_last["s"]),
    };
    Timeline::Keyed { first: (t_first, conv(&k_first["s"])), last: (t_last, last), segs }
}

fn lerp_v(a: &[f64], b: &[f64], y: f64) -> Vec<f64> {
    a.iter().zip(b).map(|(x, bx)| x + (bx - x) * y).collect()
}

fn value_at(tl: &Timeline, t: f64) -> Vec<f64> {
    match tl {
        Timeline::Constant(v) => v.clone(),
        Timeline::Keyed { first, last, segs } => {
            if t <= first.0 {
                return first.1.clone();
            }
            for s in segs {
                if t < s.t1 {
                    let Some(ease) = s.ease else { return s.v0.clone() };
                    let u = solve_u(&ease, (t - s.t0) / (s.t1 - s.t0));
                    return lerp_v(&s.v0, &s.v1, bz(0.0, ease[1], ease[3], 1.0, u));
                }
            }
            last.1.clone()
        }
    }
}

fn tl_keys(tl: &Timeline) -> Vec<&Vec<f64>> {
    match tl {
        Timeline::Constant(v) => vec![v],
        Timeline::Keyed { first, last, segs } => {
            std::iter::once(&first.1).chain(segs.iter().map(|s| &s.v1)).chain(std::iter::once(&last.1)).collect()
        }
    }
}

fn same_v(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-6)
}

#[derive(Debug, Clone)]
struct Pt {
    t: f64,
    v: Vec<f64>,
    /// The spline of the interval ending here.
    spline: Option<Ease>,
    floor: bool,
}

fn clamp_pts(pts: &[Pt], lo: f64, hi: f64) -> Vec<Pt> {
    let c = |v: &[f64]| vec![js_min(hi, js_max(lo, v[0]))];
    let mut out = Vec::new();
    for (j, p) in pts.iter().enumerate() {
        if j > 0 {
            let q = &pts[j - 1];
            for bnd in [lo, hi] {
                if (q.v[0] - bnd) * (p.v[0] - bnd) < 0.0 {
                    let frac = (bnd - q.v[0]) / (p.v[0] - q.v[0]);
                    out.push(Pt { t: q.t + (p.t - q.t) * frac, v: vec![bnd], spline: Some(LINEAR), floor: false });
                }
            }
        }
        out.push(Pt { v: c(&p.v), ..p.clone() });
    }
    out
}

/// A keyTime: its value, and whether it rounds down at 7 decimals (hold ramps).
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyTime {
    Plain(f64),
    Floor(f64),
}

#[derive(Debug, Clone)]
struct Keys<T> {
    key_times: Vec<KeyTime>,
    times: Vec<f64>,
    values: Vec<T>,
    splines: Vec<Ease>,
}

/// SMIL keys of a timeline over frames `[w0, w1]`: `None` when constant. `clamp` bounds
/// scalar values (trim).
fn smil_keys(tl: &Timeline, w0: f64, w1: f64, clamp: Option<(f64, f64)>) -> Option<Keys<Vec<f64>>> {
    let Timeline::Keyed { segs, .. } = tl else { return None };
    let d = w1 - w0;
    let kt = |t: f64| (t - w0) / d;
    let mut pts: Vec<Pt> = Vec::new();
    let push = |pts: &mut Vec<Pt>, t: f64, v: Vec<f64>, spline: Option<Ease>| {
        if let Some(last) = pts.last()
            && (last.t - t).abs() < 1e-9
            && same_v(&last.v, &v)
        {
            return;
        }
        pts.push(Pt { t, v, spline, floor: false });
    };
    push(&mut pts, w0, value_at(tl, w0), None);
    for s in segs {
        let (a, b) = (s.t0.max(w0), s.t1.min(w1));
        if b <= a {
            continue;
        }
        if s.t0 > w0 && pts.last().expect("start").t < s.t0 {
            push(&mut pts, s.t0, s.v0.clone(), Some(LINEAR));
        }
        let Some(ease) = s.ease else {
            // Lottie: v0 while t < t1, the next value from t1 on. The hold ends in a 1e-6
            // ramp whose ends round down, so the new value shows on the key's own frame and
            // no two keyTimes are equal.
            let next = value_at(tl, s.t1);
            if b == s.t1 && b < w1 && !same_v(&next, &s.v0) {
                push(&mut pts, a.max(b - 1e-6 * d), s.v0.clone(), Some(LINEAR));
                pts.last_mut().expect("pushed").floor = true;
                push(&mut pts, b, next, Some(LINEAR));
                pts.last_mut().expect("pushed").floor = true;
            } else {
                push(&mut pts, b, s.v0.clone(), Some(LINEAR));
            }
            continue;
        };
        let d0 = s.t1 - s.t0;
        let (xa, xb) = ((a - s.t0) / d0, (b - s.t0) / d0);
        for (_, u1, spl) in valid_pieces(&ease, solve_u(&ease, xa), solve_u(&ease, xb), 0) {
            let x1 = bz(0.0, ease[0], ease[2], 1.0, u1);
            let y1 = bz(0.0, ease[1], ease[3], 1.0, u1);
            push(&mut pts, s.t0 + x1 * d0, lerp_v(&s.v0, &s.v1, y1), Some(spl));
        }
    }
    if pts.last().expect("start").t < w1 - 1e-9 {
        push(&mut pts, w1, value_at(tl, w1), Some(LINEAR));
    }
    let out = match clamp {
        Some((lo, hi)) => clamp_pts(&pts, lo, hi),
        None => pts,
    };
    if out.iter().all(|p| same_v(&p.v, &out[0].v)) {
        return None;
    }
    let mut merged: Vec<Pt> = vec![out[0].clone()];
    for p in &out[1..] {
        let n = merged.len();
        if n >= 2
            && same_v(&merged[n - 2].v, &merged[n - 1].v)
            && same_v(&merged[n - 1].v, &p.v)
            && p.spline.is_some()
            && merged[n - 1].spline.is_some()
        {
            merged[n - 1].t = p.t;
            merged[n - 1].floor = p.floor;
            continue;
        }
        merged.push(p.clone());
    }
    Some(Keys {
        key_times: merged
            .iter()
            .map(|p| if p.floor { KeyTime::Floor((kt(p.t) * 1e7 + 1e-6).floor() / 1e7) } else { KeyTime::Plain(kt(p.t)) })
            .collect(),
        times: merged.iter().map(|p| p.t).collect(),
        splines: merged[1..].iter().map(|p| p.spline.unwrap_or(LINEAR)).collect(),
        values: merged.into_iter().map(|p| p.v).collect(),
    })
}

// ---------- slots ----------

const SLOT_IDS: [&str; 6] =
    ["style.outline.light", "style.outline.regular", "style.outline.bold", "style.outline.width", "style.solid", "style.tint"];

/// The runtime's `paintIcon`, as static values, in `SLOT_IDS` order.
fn slot_values(style: Style, weight: f64) -> [f64; 6] {
    let authored = [1.5, 2.0, 2.5].contains(&weight);
    let outline = |name: &str, w: f64| -> f64 {
        let on = style != Style::Solid && if authored { weight == w } else { name == "regular" };
        if on { 100.0 } else { 0.0 }
    };
    [
        outline("light", 1.5),
        outline("regular", 2.0),
        outline("bold", 2.5),
        if authored { 2.0 } else { weight },
        if style == Style::Solid { 100.0 } else { 0.0 },
        if style == Style::Duotone { 18.0 } else { 0.0 },
    ]
}

/// The property with its slot value when it is bound to one of `slots` and not keyed.
fn resolve<'a>(prop: &'a Value, slots: &[f64; 6]) -> Cow<'a, Value> {
    if let Some(sid) = prop["sid"].as_str()
        && let Some(index) = SLOT_IDS.iter().position(|id| *id == sid)
        && prop["a"].as_f64() != Some(1.0)
    {
        return Cow::Owned(serde_json::json!({"a": 0, "k": slots[index]}));
    }
    Cow::Borrowed(prop)
}

fn truthy(value: &Value) -> bool {
    gai_core::js::truthy(Some(value))
}

fn keyed(prop: &Value) -> bool {
    prop["a"].as_f64() == Some(1.0)
}

// ---------- renderer ----------

struct Variant<'a> {
    anim: &'a Value,
    w0: f64,
    w1: f64,
    timing: String,
}

/// What a property does in one variant.
enum Vk<T> {
    Keys(Keys<T>),
    Constant(T),
}

type PerVariant<T> = Option<Vec<Option<Vk<T>>>>;

struct Geom {
    tls: Vec<Timeline>,
    closed: bool,
    split: BTreeSet<usize>,
    prim: Option<usize>,
}

enum Ui {
    Hover,
    Click,
    Js,
    Toggle,
    JsStates,
    None,
}

struct Renderer<'a> {
    base: &'a Value,
    variants: Vec<Variant<'a>>,
    rest: f64,
    rest_variant: usize,
    id: String,
    color: String,
    slots: [f64; 6],
    many: bool,
    defs: Vec<String>,
    masks: usize,
}

impl<'a> Renderer<'a> {
    fn layers(&self) -> &'a [Value] {
        self.base["layers"].as_array().map(Vec::as_slice).unwrap_or_default()
    }

    fn layer(&self, variant: usize, li: usize) -> &'a Value {
        &self.variants[variant].anim["layers"][li]
    }

    fn track(&self, get: &dyn Fn(&'a Value) -> Cow<'a, Value>, conv: &dyn Fn(&Value) -> Vec<f64>) -> Vec<Timeline> {
        self.variants.iter().map(|v| timeline(&get(v.anim), conv)).collect()
    }

    fn rest_of(&self, tls: &[Timeline]) -> Vec<f64> {
        value_at(&tls[self.rest_variant], self.rest)
    }

    /// Per variant: keys, a constant (when it differs from rest, or when variants share
    /// the element and must override each other's frozen values), or nothing.
    fn keys_of(&self, tls: &[Timeline], clamp: Option<(f64, f64)>) -> PerVariant<Vec<f64>> {
        let rest_val = self.rest_of(tls);
        let cl = |v: Vec<f64>| match clamp {
            Some((lo, hi)) => vec![js_min(hi, js_max(lo, v[0]))],
            None => v,
        };
        let ks: Vec<Option<Keys<Vec<f64>>>> =
            tls.iter().zip(&self.variants).map(|(tl, v)| smil_keys(tl, v.w0, v.w1, clamp)).collect();
        let consts: Vec<Option<Vec<f64>>> = tls
            .iter()
            .zip(&self.variants)
            .zip(&ks)
            .map(|((tl, v), k)| if k.is_some() { None } else { Some(cl(value_at(tl, v.w0))) })
            .collect();
        let rest_cl = cl(rest_val);
        let differs = |c: &Option<Vec<f64>>| c.as_ref().is_some_and(|c| !same_v(c, &rest_cl));
        if !(ks.iter().any(Option::is_some) || consts.iter().any(differs)) {
            return None;
        }
        Some(
            ks.into_iter()
                .zip(consts)
                .map(|(k, c)| match k {
                    Some(k) => Some(Vk::Keys(k)),
                    None => {
                        let c = c.expect("constant");
                        (self.many || !same_v(&c, &rest_cl)).then_some(Vk::Constant(c))
                    }
                })
                .collect(),
        )
    }

    fn emit<T>(&self, tag: &str, attr: &str, ks: &PerVariant<T>, fmt: &dyn Fn(&T) -> String) -> String {
        let Some(ks) = ks else { return String::new() };
        let what = if tag == "animateTransform" {
            format!("attributeName=\"transform\" type=\"{attr}\"")
        } else {
            format!("attributeName=\"{attr}\"")
        };
        let mut out = String::new();
        for (k, v) in ks.iter().zip(&self.variants) {
            match k {
                None => {}
                Some(Vk::Constant(c)) => {
                    let _ = write!(out, "<{tag} {what} values=\"{}\" {}/>", fmt(c), v.timing);
                }
                Some(Vk::Keys(k)) => {
                    let values: Vec<String> = k.values.iter().map(fmt).collect();
                    let _ = write!(out, "<{tag} {what} values=\"{}\" {} {}/>", values.join(";"), k_attr(k), v.timing);
                }
            }
        }
        out
    }

    /// A layer's own transform groups, outermost first: translate(p - a), then rotate(r, a).
    fn layer_transform(&self, li: usize) -> Result<Vec<String>> {
        let ks = &self.layers()[li]["ks"];
        let a = if truthy(&ks["a"]) { vec2(&ks["a"]["k"]) } else { vec![0.0, 0.0] };
        let s = &ks["s"];
        if truthy(s) && (keyed(s) || at(&s["k"], 0) != 100.0 || at(&s["k"], 1) != 100.0) {
            return Err(unsupported("layer scale"));
        }
        if (0..self.variants.len()).any(|v| keyed(&self.layer(v, li)["ks"]["a"])) {
            return Err(unsupported("an animated anchor"));
        }
        let sk = &ks["sk"];
        if truthy(sk) && (keyed(sk) || f(&sk["k"]) != 0.0) {
            return Err(unsupported("skew"));
        }
        let t = |x: f64, y: f64| format!("{},{}", n3(x), n3(y));
        let mut p_els = Vec::new();
        let p_rest: [f64; 2];
        // One animation may key a layer's position per axis and another as a pair.
        if (0..self.variants.len()).any(|v| truthy(&self.layer(v, li)["ks"]["p"]["s"])) {
            let axis = |i: usize| -> Vec<Timeline> {
                (0..self.variants.len())
                    .map(|v| {
                        let p = &self.layer(v, li)["ks"]["p"];
                        if truthy(&p["s"]) {
                            timeline(&p[if i == 0 { "x" } else { "y" }], &scalar)
                        } else {
                            timeline(p, &|k: &Value| vec![at(k, i)])
                        }
                    })
                    .collect()
            };
            let (tx, ty) = (axis(0), axis(1));
            let (rx, ry) = (self.rest_of(&tx)[0], self.rest_of(&ty)[0]);
            p_rest = [rx - a[0], ry - a[1]];
            let (kx, ky) = (self.keys_of(&tx, None), self.keys_of(&ty, None));
            match (&kx, &ky) {
                (Some(_), Some(_)) => {
                    p_els.push(format!(
                        "<g transform=\"translate({} 0)\">{}",
                        n3(p_rest[0]),
                        self.emit("animateTransform", "translate", &kx, &|v| t(v[0] - a[0], 0.0))
                    ));
                    p_els.push(format!(
                        "<g transform=\"translate(0 {})\">{}",
                        n3(p_rest[1]),
                        self.emit("animateTransform", "translate", &ky, &|v| t(0.0, v[0] - a[1]))
                    ));
                }
                (Some(_), None) => p_els.push(format!(
                    "<g transform=\"translate({} {})\">{}",
                    n3(p_rest[0]),
                    n3(p_rest[1]),
                    self.emit("animateTransform", "translate", &kx, &|v| t(v[0] - a[0], ry - a[1]))
                )),
                (None, Some(_)) => p_els.push(format!(
                    "<g transform=\"translate({} {})\">{}",
                    n3(p_rest[0]),
                    n3(p_rest[1]),
                    self.emit("animateTransform", "translate", &ky, &|v| t(rx - a[0], v[0] - a[1]))
                )),
                (None, None) => {}
            }
        } else {
            let tp = self.track(&|an| Cow::Borrowed(&an["layers"][li]["ks"]["p"]), &vec2);
            let r = self.rest_of(&tp);
            let kp = self.keys_of(&tp, None);
            p_rest = [r[0] - a[0], r[1] - a[1]];
            if kp.is_some() {
                p_els.push(format!(
                    "<g transform=\"translate({} {})\">{}",
                    n3(p_rest[0]),
                    n3(p_rest[1]),
                    self.emit("animateTransform", "translate", &kp, &|v| t(v[0] - a[0], v[1] - a[1]))
                ));
            }
        }
        let zero_rotation = serde_json::json!({"a": 0, "k": 0});
        let tr = self.track(
            &|an| {
                let r = &an["layers"][li]["ks"]["r"];
                if r.is_null() { Cow::Owned(zero_rotation.clone()) } else { Cow::Borrowed(r) }
            },
            &scalar,
        );
        let kr = self.keys_of(&tr, None);
        let r_rest = self.rest_of(&tr)[0];
        let rot = |r: f64| format!("{} {} {}", n3(r), n3(a[0]), n3(a[1]));
        let mut out: Vec<String> = Vec::new();
        let mut static_t: Vec<String> = Vec::new();
        let had_p = !p_els.is_empty();
        out.extend(p_els);
        if !had_p && (p_rest[0].abs() > 1e-9 || p_rest[1].abs() > 1e-9) {
            static_t.push(format!("translate({} {})", n3(p_rest[0]), n3(p_rest[1])));
        }
        if kr.is_some() {
            if !static_t.is_empty() {
                out.push(format!("<g transform=\"{}\">", static_t.join(" ")));
                static_t.clear();
            }
            out.push(format!(
                "<g transform=\"rotate({})\">{}",
                rot(r_rest),
                self.emit("animateTransform", "rotate", &kr, &|v| rot(v[0]))
            ));
        } else if r_rest.abs() > 1e-9 {
            static_t.push(format!("rotate({})", rot(r_rest)));
        }
        if !static_t.is_empty() {
            out.push(format!("<g transform=\"{}\">", static_t.join(" ")));
        }
        Ok(out)
    }

    fn wrap(&self, li: usize, inner: String) -> Result<String> {
        let opens = self.layer_transform(li)?;
        Ok(format!("{}{inner}{}", opens.join(""), "</g>".repeat(opens.len())))
    }

    fn chain(&self, li: usize) -> Result<Vec<usize>> {
        let layers = self.layers();
        let mut c = Vec::new();
        let mut parent = layers[li]["parent"].as_f64();
        while let Some(p) = parent {
            let pi = layers
                .iter()
                .position(|l| l["ind"].as_f64() == Some(p))
                .ok_or_else(|| unsupported("a missing parent layer"))?;
            if c.contains(&pi) {
                return Err(unsupported("a parenting cycle"));
            }
            c.insert(0, pi);
            parent = layers[pi]["parent"].as_f64();
        }
        Ok(c)
    }

    fn wrap_chain(&self, anc: &[usize], inner: String) -> Result<String> {
        let mut s = inner;
        for &li in anc.iter().rev() {
            s = self.wrap(li, s)?;
        }
        Ok(s)
    }

    /// The layer's opacity group: `(open tag, hidden for good)`, or `None` at 100%.
    fn opacity_wrap(&self, li: usize) -> Option<(String, bool)> {
        let full = serde_json::json!({"a": 0, "k": 100});
        let t = self.track(
            &|an| {
                let o = &an["layers"][li]["ks"]["o"];
                if o.is_null() { Cow::Owned(full.clone()) } else { Cow::Borrowed(o) }
            },
            &scalar,
        );
        let k = self.keys_of(&t, None);
        let r = self.rest_of(&t)[0];
        if k.is_none() {
            return if r >= 100.0 { None } else { Some((format!("<g opacity=\"{}\">", n3(r / 100.0)), r <= 0.0)) };
        }
        Some((
            format!("<g opacity=\"{}\">{}", n3(r / 100.0), self.emit("animate", "opacity", &k, &|v| n3(v[0] / 100.0))),
            false,
        ))
    }

    /// Per-variant keys of several paths drawn as one compound path (fill winding).
    fn multi_keys(&self, geoms: &[Geom]) -> (String, PerVariant<String>) {
        let rest_d: String =
            geoms.iter().map(|g| shape_d(&self.rest_of(&g.tls), g.closed, true, &g.split)).collect::<Vec<_>>().join("");
        let mut any = false;
        let ks = self
            .variants
            .iter()
            .enumerate()
            .map(|(j, v)| {
                let anim: Vec<Keys<Vec<f64>>> =
                    geoms.iter().filter_map(|g| smil_keys(&g.tls[j], v.w0, v.w1, None)).collect();
                let at = |t: f64| -> String {
                    geoms.iter().map(|g| shape_d(&value_at(&g.tls[j], t), g.closed, true, &g.split)).collect()
                };
                if anim.is_empty() {
                    let c = at(v.w0);
                    if c != rest_d {
                        any = true;
                    }
                    return (self.many || c != rest_d).then_some(Vk::Constant(c));
                }
                any = true;
                let a0 = &anim[0];
                let same = anim.iter().all(|k| {
                    k.times.len() == a0.times.len()
                        && k.times.iter().zip(&a0.times).all(|(x, y)| (x - y).abs() < 1e-6)
                        && k.splines.iter().zip(&a0.splines).all(|(s, t)| s.iter().zip(t).all(|(x, y)| (x - y).abs() < 1e-4))
                });
                if same {
                    return Some(Vk::Keys(Keys {
                        key_times: a0.key_times.clone(),
                        times: a0.times.clone(),
                        splines: a0.splines.clone(),
                        values: a0.times.iter().map(|&t| at(t)).collect(),
                    }));
                }
                // paths keyed at different times: one key per frame
                let mut times = Vec::new();
                let mut fr = v.w0;
                while fr < v.w1 {
                    times.push(fr);
                    fr += 1.0;
                }
                times.push(v.w1);
                Some(Vk::Keys(Keys {
                    key_times: times.iter().map(|t| KeyTime::Plain((t - v.w0) / (v.w1 - v.w0))).collect(),
                    splines: vec![LINEAR; times.len() - 1],
                    values: times.iter().map(|&t| at(t)).collect(),
                    times,
                }))
            })
            .collect();
        (rest_d, if any { Some(ks) } else { None })
    }

    /// Lottie draws nothing while a trim's end is 0; SVG would draw a round-cap dot. The
    /// state only changes at key times.
    fn visibility_keys(&self, te: &Timeline, j: usize, rest_hidden: bool) -> String {
        let v = &self.variants[j];
        let (w0, w1) = (v.w0, v.w1);
        let d = w1 - w0;
        let hid = |t: f64| value_at(te, t)[0] <= 1e-6;
        let mut times = vec![w0];
        if let Timeline::Keyed { segs, .. } = te {
            times.extend(segs.iter().flat_map(|s| [s.t0, s.t1]).filter(|&t| t > w0 && t < w1));
        }
        times.push(w1);
        times.sort_by(|x, y| x.partial_cmp(y).expect("finite"));
        // Each time is compared with its sorted predecessor, kept or not (as svg.mjs does).
        let kept: Vec<f64> =
            times.iter().enumerate().filter(|&(i, t)| i == 0 || t - times[i - 1] > 1e-9).map(|(_, &t)| t).collect();
        let fl = |x: f64| (x * 1e7 + 1e-6).floor() / 1e7;
        let mut changes: Vec<(f64, bool)> = Vec::new();
        let mut set = |kt: f64, h: bool| {
            if let Some(last) = changes.last_mut() {
                if last.1 == h {
                    return;
                }
                if (last.0 - kt).abs() < 1e-9 {
                    last.1 = h;
                    return;
                }
            }
            changes.push((kt, h));
        };
        for q in 0..kept.len() - 1 {
            let t = kept[q];
            let (mid, here) = (hid((t + kept[q + 1]) / 2.0), hid(t));
            if q == 0 {
                set(0.0, if here && mid { true } else { mid });
            } else if here && !mid {
                set(fl((t - w0) / d), true);
                set(fl((t - w0) / d) + 1e-6, false);
            } else {
                set(fl((t - w0) / d), mid);
            }
        }
        let word = |h: bool| if h { "hidden" } else { "visible" };
        if changes.len() == 1 && changes[0].1 == rest_hidden && !self.many {
            return String::new();
        }
        if changes.len() == 1 {
            return format!("<set attributeName=\"visibility\" to=\"{}\" {}/>", word(changes[0].1), v.timing);
        }
        let values: Vec<&str> = changes.iter().map(|c| word(c.1)).collect();
        let times: Vec<String> = changes.iter().map(|c| num(c.0, 7)).collect();
        format!(
            "<animate attributeName=\"visibility\" calcMode=\"discrete\" values=\"{}\" keyTimes=\"{}\" {}/>",
            values.join(";"),
            times.join(";"),
            v.timing
        )
    }

    /// A layer's shapes; `paint_override` paints a matte source.
    fn layer_content(&self, li: usize, paint_override: Option<&str>) -> Result<String> {
        let items = self.layers()[li]["shapes"].as_array().map(Vec::as_slice).unwrap_or_default();
        let kinds = |l: &Value| -> (Vec<Value>, usize) {
            let shapes = l["shapes"].as_array().map(|s| s.iter().map(|x| x["ty"].clone()).collect()).unwrap_or_default();
            (shapes, l["masksProperties"].as_array().map_or(0, Vec::len))
        };
        if (0..self.variants.len()).any(|v| kinds(self.layer(v, li)) != kinds(&self.layers()[li])) {
            return Err(unsupported(&format!(
                "the animations of one icon differ in the shapes of {}",
                gai_core::js::to_js_string(Some(&self.layers()[li]["nm"]))
            )));
        }
        let mut geoms: Vec<Geom> = Vec::new();
        let mut styles: Vec<&Value> = Vec::new();
        let mut trim_idx: Option<usize> = None;
        for (si, it) in items.iter().enumerate() {
            match it["ty"].as_str().unwrap_or_default() {
                "sh" => {
                    let tls = self.track(&|an| pad_morph(&an["layers"][li]["shapes"][si]["ks"]), &shape_conv);
                    let closed = if keyed(&it["ks"]) { &it["ks"]["k"][0]["s"][0]["c"] } else { &it["ks"]["k"]["c"] };
                    let closed = closed.as_bool().unwrap_or(false);
                    let split = hairpins(tls.iter().flat_map(tl_keys), closed);
                    geoms.push(Geom { tls, closed, split, prim: None });
                }
                ty @ ("rc" | "el") => {
                    if ["p", "s", "r"].iter().any(|p| keyed(&it[*p])) {
                        return Err(unsupported(if ty == "rc" { "an animated rectangle" } else { "an animated ellipse" }));
                    }
                    let flat = if ty == "rc" {
                        rect_shape(&it["p"]["k"], &it["s"]["k"], f(&it["r"]["k"]))
                    } else {
                        ellipse_shape(&it["p"]["k"], &it["s"]["k"])
                    };
                    let tls = self.variants.iter().map(|_| Timeline::Constant(flat.clone())).collect();
                    geoms.push(Geom { tls, closed: true, split: BTreeSet::new(), prim: Some(si) });
                }
                "tm" => trim_idx = Some(si),
                "st" | "fl" => styles.push(it),
                other => return Err(unsupported(&format!("shape item {other}"))),
            }
        }
        struct Stroke {
            o: f64,
            w: f64,
        }
        struct Fill {
            o: f64,
            even_odd: bool,
            before: bool,
        }
        let (mut stroke, mut fill): (Option<Stroke>, Option<Fill>) = (None, None);
        for st in styles {
            let o = resolve(&st["o"], &self.slots);
            if keyed(&o) {
                return Err(unsupported("animated paint opacity"));
            }
            let op = scalar(&o["k"])[0];
            if op <= 0.0 {
                continue;
            }
            if st["ty"] == "st" {
                let w = resolve(&st["w"], &self.slots);
                if keyed(&w) {
                    return Err(unsupported("animated stroke width"));
                }
                if st["d"].as_array().is_some_and(|d| !d.is_empty()) {
                    return Err(unsupported("dashed strokes"));
                }
                stroke = Some(Stroke { o: op, w: scalar(&w["k"])[0] });
            } else {
                fill = Some(Fill { o: op, even_odd: st["r"].as_f64() == Some(2.0), before: stroke.is_none() });
            }
        }
        if stroke.is_none() && fill.is_none() {
            return Ok(String::new());
        }
        let c = paint_override.unwrap_or(&self.color);
        let mut paint = String::new();
        match &fill {
            Some(fill) => {
                let _ = write!(paint, " fill=\"{c}\"");
                if fill.o < 100.0 {
                    let _ = write!(paint, " fill-opacity=\"{}\"", n3(fill.o / 100.0));
                }
                if fill.even_odd {
                    paint.push_str(" fill-rule=\"evenodd\"");
                }
            }
            None => paint.push_str(" fill=\"none\""),
        }
        if let Some(stroke) = &stroke {
            let _ = write!(paint, " stroke=\"{c}\" stroke-width=\"{}\"", n3(stroke.w));
            if stroke.o < 100.0 {
                let _ = write!(paint, " stroke-opacity=\"{}\"", n3(stroke.o / 100.0));
            }
        }
        // Lottie paints earlier items on top: a fill listed before its stroke covers it.
        if stroke.is_some() && fill.as_ref().is_some_and(|f| f.before) {
            paint.push_str(" paint-order=\"stroke\"");
        }
        let geom_el = |g: &Geom| -> (String, String) {
            let k = self.keys_of(&g.tls, None);
            let d = shape_d(&self.rest_of(&g.tls), g.closed, k.is_some(), &g.split);
            (d, self.emit("animate", "d", &k, &|v| shape_d(v, g.closed, true, &g.split)))
        };
        let close = |open: String, inner: &str| {
            if inner.is_empty() { format!("{open}/>") } else { format!("{open}>{inner}</path>") }
        };
        if let Some(ti) = trim_idx {
            // Trim end only (start and offset are 0 in every plan); "simultaneous" trims each
            // path. A fill under a trim stays whole: SVG can't trim a fill.
            for v in 0..self.variants.len() {
                let t = &self.layer(v, li)["shapes"][ti];
                if keyed(&t["s"])
                    || keyed(&t["o"])
                    || scalar(&t["s"]["k"])[0] != 0.0
                    || scalar(&t["o"]["k"])[0] != 0.0
                    || t["m"].as_f64().unwrap_or(1.0) != 1.0
                {
                    return Err(unsupported("trim start, offset or individual trims"));
                }
            }
            let te = self.track(&|an| Cow::Borrowed(&an["layers"][li]["shapes"][ti]["e"]), &scalar);
            let ke = self.keys_of(&te, Some((0.0, 100.0)));
            let e_rest = js_min(100.0, js_max(0.0, self.rest_of(&te)[0]));
            if ke.is_none() && e_rest <= 0.0 {
                return Ok(String::new());
            }
            let mut els = String::new();
            for g in &geoms {
                let (d, anim) = geom_el(g);
                let (mut dash, mut extra) = (String::new(), String::new());
                if ke.is_some() || e_rest < 100.0 {
                    // Renderers without SMIL often ignore pathLength too, so a stroke fully
                    // drawn or hidden at rest carries no dash; the dash is set while animating.
                    let partial = e_rest > 0.0 && e_rest < 100.0;
                    dash = " pathLength=\"1\"".into();
                    if partial {
                        let _ = write!(
                            dash,
                            " stroke-dasharray=\"1 2\" stroke-dashoffset=\"{}\"",
                            num(1.0 - e_rest / 100.0, 4)
                        );
                    }
                    if let Some(per) = &ke {
                        for (k, v) in per.iter().zip(&self.variants) {
                            if k.is_some() && !partial {
                                let _ = write!(extra, "<set attributeName=\"stroke-dasharray\" to=\"1 2\" {}/>", v.timing);
                            }
                        }
                        extra.push_str(&self.emit("animate", "stroke-dashoffset", &ke, &|v| num(1.0 - v[0] / 100.0, 4)));
                        for (j, k) in per.iter().enumerate() {
                            if k.is_some() {
                                extra.push_str(&self.visibility_keys(&te[j], j, e_rest <= 0.0));
                            }
                        }
                    }
                }
                let hidden = if e_rest <= 0.0 { " visibility=\"hidden\"" } else { "" };
                els.push_str(&close(format!("<path d=\"{d}\"{paint}{dash}{hidden}"), &(anim + &extra)));
            }
            return Ok(els);
        }
        if geoms.len() == 1
            && let Some(si) = geoms[0].prim
        {
            let pr = &items[si];
            let (p, s) = (&pr["p"]["k"], &pr["s"]["k"]);
            let (x, y, w, h) = (at(p, 0), at(p, 1), at(s, 0), at(s, 1));
            if pr["ty"] == "el" {
                return Ok(if w == h {
                    format!("<circle cx=\"{}\" cy=\"{}\" r=\"{}\"{paint}/>", n3(x), n3(y), n3(w / 2.0))
                } else {
                    format!(
                        "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"{paint}/>",
                        n3(x),
                        n3(y),
                        n3(w / 2.0),
                        n3(h / 2.0)
                    )
                });
            }
            let r = (w / 2.0).min(h / 2.0).min(f(&pr["r"]["k"]));
            let rx = if r != 0.0 { format!(" rx=\"{}\"", n3(r)) } else { String::new() };
            return Ok(format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rx}{paint}/>",
                n3(x - w / 2.0),
                n3(y - h / 2.0),
                n3(w),
                n3(h)
            ));
        }
        if geoms.len() == 1 {
            let (d, anim) = geom_el(&geoms[0]);
            return Ok(close(format!("<path d=\"{d}\"{paint}"), &anim));
        }
        let (rest_d, ks) = self.multi_keys(&geoms);
        if ks.is_none() {
            let d: String =
                geoms.iter().map(|g| shape_d(&self.rest_of(&g.tls), g.closed, false, &g.split)).collect::<Vec<_>>().join("");
            return Ok(format!("<path d=\"{d}\"{paint}/>"));
        }
        Ok(close(format!("<path d=\"{rest_d}\"{paint}"), &self.emit("animate", "d", &ks, &|v: &String| v.clone())))
    }

    /// Layer masks (subtract), in layer space.
    fn mask_def(&mut self, li: usize) -> Result<String> {
        let mid = format!("{}_k{}", self.id, self.masks);
        self.masks += 1;
        let mut s = format!(
            "<mask id=\"{mid}\" maskUnits=\"userSpaceOnUse\" x=\"-64\" y=\"-64\" width=\"160\" height=\"160\"><rect x=\"-64\" y=\"-64\" width=\"160\" height=\"160\" fill=\"#fff\"/>"
        );
        let masks = self.layers()[li]["masksProperties"].as_array().map(Vec::as_slice).unwrap_or_default();
        for (mi, m) in masks.iter().enumerate() {
            if m["mode"] != "s"
                || truthy(&m["inv"])
                || (truthy(&m["o"]) && scalar(&m["o"]["k"])[0] != 100.0)
                || (truthy(&m["x"]) && scalar(&m["x"]["k"])[0] != 0.0)
            {
                return Err(unsupported(&format!("mask mode {}", gai_core::js::to_js_string(Some(&m["mode"])))));
            }
            let tls = self.track(&|an| pad_morph(&an["layers"][li]["masksProperties"][mi]["pt"]), &shape_conv);
            let closed = if keyed(&m["pt"]) { &m["pt"]["k"][0]["s"][0]["c"] } else { &m["pt"]["k"]["c"] };
            let closed = closed.as_bool().unwrap_or(false);
            let k = self.keys_of(&tls, None);
            let split = hairpins(tls.iter().flat_map(tl_keys), closed);
            let _ = write!(s, "<path d=\"{}\" fill=\"#000\"", shape_d(&self.rest_of(&tls), closed, k.is_some(), &split));
            if k.is_some() {
                let _ = write!(s, ">{}</path>", self.emit("animate", "d", &k, &|v| shape_d(v, closed, true, &split)));
            } else {
                s.push_str("/>");
            }
        }
        s.push_str("</mask>");
        self.defs.push(s);
        Ok(mid)
    }

    /// One layer: its own transform, opacity, masks and content (not its parents).
    fn layer_body(&mut self, li: usize, paint_override: Option<&str>) -> Result<String> {
        let l = &self.layers()[li];
        let (base_ip, base_op) = (f(&self.base["ip"]), f(&self.base["op"]));
        if l["ty"].as_f64() != Some(4.0)
            || truthy(&l["tm"])
            || l["ef"].as_array().is_some_and(|e| !e.is_empty())
            || truthy(&l["bm"])
            || l["ip"].as_f64().unwrap_or(0.0) > base_ip
            || l["op"].as_f64().unwrap_or(base_op) < base_op
        {
            return Err(unsupported(&format!(
                "layer {} (type {}, time remap, effects, blend or in/out points)",
                gai_core::js::to_js_string(Some(&l["nm"])),
                gai_core::js::to_js_string(Some(&l["ty"]))
            )));
        }
        let content = self.layer_content(li, paint_override)?;
        if content.is_empty() {
            return Ok(String::new());
        }
        let op = self.opacity_wrap(li);
        if op.as_ref().is_some_and(|o| o.1) && !self.many {
            return Ok(String::new());
        }
        let mut s = content;
        if l["masksProperties"].as_array().is_some_and(|m| !m.is_empty()) {
            s = format!("<g mask=\"url(#{})\">{s}</g>", self.mask_def(li)?);
        }
        if let Some((open, _)) = op {
            s = format!("{open}{s}</g>");
        }
        self.wrap(li, s)
    }

    fn render(mut self, ui: Ui, size: Option<f64>) -> Result<String> {
        let layers = self.layers();
        let mut body = String::new();
        // bottom to top; neighbours with the same parents share their parents' groups
        let mut run: Option<(Vec<usize>, String)> = None;
        for li in (0..layers.len()).rev() {
            let l = &layers[li];
            if truthy(&l["td"]) {
                continue;
            }
            let anc = self.chain(li)?;
            if truthy(&l["tt"]) {
                if l["tt"].as_f64() != Some(2.0) {
                    return Err(unsupported(&format!("matte mode {}", gai_core::js::to_js_string(Some(&l["tt"])))));
                }
                if let Some((a, inner)) = run.take() {
                    body.push_str(&self.wrap_chain(&a, inner)?);
                }
                let inner = self.layer_body(li, None)?;
                if inner.is_empty() {
                    continue;
                }
                let si = match l["tp"].as_f64() {
                    Some(tp) => layers.iter().position(|m| m["ind"].as_f64() == Some(tp)),
                    None => li.checked_sub(1),
                }
                .filter(|&si| truthy(&layers[si]["td"]))
                .ok_or_else(|| unsupported("a missing matte source"))?;
                let mid = format!("{}_t{}", self.id, self.masks);
                self.masks += 1;
                let source_chain = self.chain(si)?;
                let source = self.layer_body(si, Some("#000"))?;
                let source = self.wrap_chain(&source_chain, source)?;
                self.defs.push(format!(
                    "<mask id=\"{mid}\" maskUnits=\"userSpaceOnUse\" x=\"-64\" y=\"-64\" width=\"160\" height=\"160\"><rect x=\"-64\" y=\"-64\" width=\"160\" height=\"160\" fill=\"#fff\"/>{source}</mask>"
                ));
                let _ = write!(body, "<g mask=\"url(#{mid})\">{}</g>", self.wrap_chain(&anc, inner)?);
                continue;
            }
            let inner = self.layer_body(li, None)?;
            if inner.is_empty() {
                continue;
            }
            match &mut run {
                Some((a, text)) if *a == anc => text.push_str(&inner),
                _ => {
                    if let Some((a, inner)) = run.take() {
                        body.push_str(&self.wrap_chain(&a, inner)?);
                    }
                    run = Some((anc, inner));
                }
            }
        }
        if let Some((a, inner)) = run.take()
            && !inner.is_empty()
        {
            body.push_str(&self.wrap_chain(&a, inner)?);
        }

        let (w, h, fps) = (f(&self.base["w"]), f(&self.base["h"]), f(&self.base["fr"]));
        let longest = self.variants.iter().map(|v| v.w1 - v.w0).fold(f64::NEG_INFINITY, f64::max);
        let longest = format!("{}s", num(longest / fps, 4));
        let (ws, hs) = (gai_core::js_number_to_string(w), gai_core::js_number_to_string(h));
        let id = &self.id;
        let hit = |hid: &str, extra: &str| {
            format!("<rect id=\"{hid}\" width=\"{ws}\" height=\"{hs}\" fill=\"none\" pointer-events=\"all\"{extra}>")
        };
        let controls = match ui {
            Ui::Hover | Ui::Click => format!(
                "{}<set id=\"{id}_m\" attributeName=\"x\" to=\"0\" begin=\"{id}_h.{}\" dur=\"{longest}\" restart=\"whenNotActive\"/></rect>",
                hit(&format!("{id}_h"), ""),
                if matches!(ui, Ui::Hover) { "mouseenter" } else { "click" }
            ),
            Ui::Js => format!(
                "<rect width=\"0\" height=\"0\"><set id=\"{id}_m\" attributeName=\"x\" to=\"0\" begin=\"indefinite\" dur=\"{longest}\"/></rect>"
            ),
            Ui::Toggle => format!(
                "{}<set attributeName=\"display\" to=\"none\" begin=\"{id}_a.click\" end=\"{id}_b.click\"/></rect>{}<set attributeName=\"display\" to=\"inline\" begin=\"{id}_a.click\" end=\"{id}_b.click\"/></rect>",
                hit(&format!("{id}_a"), ""),
                hit(&format!("{id}_b"), " display=\"none\"")
            ),
            Ui::JsStates => format!(
                "<rect width=\"0\" height=\"0\"><set id=\"{id}_a\" attributeName=\"x\" to=\"0\" begin=\"indefinite\" dur=\"{longest}\"/><set id=\"{id}_b\" attributeName=\"x\" to=\"0\" begin=\"indefinite\" dur=\"{longest}\"/></rect>"
            ),
            Ui::None => String::new(),
        };
        let width = size.unwrap_or(w);
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {ws} {hs}\" width=\"{}\" height=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            n3(width),
            n3(width * h / w)
        );
        if !self.defs.is_empty() {
            let _ = write!(svg, "<defs>{}</defs>", self.defs.join(""));
        }
        svg.push_str(&body);
        svg.push_str(&controls);
        svg.push_str("</svg>");
        Ok(svg)
    }
}

fn k_attr(k: &Keys<impl Sized>) -> String {
    // SMIL drops an animation whose keyTimes decrease: keep the rounded list non-decreasing.
    let mut last = 0.0;
    let ks: Vec<String> = k
        .key_times
        .iter()
        .map(|x| {
            let (mut value, mut s) = match *x {
                KeyTime::Floor(v) => (to_fixed(v, 7), num(v, 7)),
                KeyTime::Plain(v) => (to_fixed(v, 5), num(v, 5)),
            };
            if value < last {
                s = num(last, 7);
                value = to_fixed(last, 7);
            }
            last = value;
            s
        })
        .collect();
    let linear = k.splines.iter().all(|s| s[0] == 0.0 && s[1] == 0.0 && s[2] == 1.0 && s[3] == 1.0);
    let mut out = format!("keyTimes=\"{}\"", ks.join(";"));
    if !linear {
        let sp: Vec<String> = k.splines.iter().map(|s| s.iter().map(|&x| num(x, 4)).collect::<Vec<_>>().join(" ")).collect();
        let _ = write!(out, " calcMode=\"spline\" keySplines=\"{}\"", sp.join(";"));
    }
    out
}

// ---------- options and entry points ----------

/// Checks the look options shared by every entry point.
fn check_look(options: &SvgOptions) -> Result<()> {
    if !(options.weight.is_finite() && (1.0..=3.0).contains(&options.weight)) {
        return Err(SvgError::Options("weight must be a number from 1 to 3".into()));
    }
    let hex = |s: &str| (s.len() == 4 || s.len() == 7) && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit());
    if options.color != "currentColor" && !hex(&options.color) {
        return Err(SvgError::Options("color must be currentColor or a hex value such as #008c76".into()));
    }
    if let Some(size) = options.size
        && !(size.is_finite() && size > 0.0 && size <= 4096.0)
    {
        return Err(SvgError::Options("size must be greater than 0 and at most 4096".into()));
    }
    Ok(())
}

/// SMIL timing reads `-` and `.` inside an id as an offset or an event, so ids keep to
/// `[A-Za-z0-9_]`: `-` is written as `_`.
fn svg_id(id: &str) -> Result<String> {
    let mut chars = id.chars();
    let ok = id.len() <= 64
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err(SvgError::Options("id must start with a letter and hold at most 64 letters, digits, - and _".into()));
    }
    Ok(id.replace('-', "_"))
}

fn renderer<'a>(variants: Vec<Variant<'a>>, rest: f64, rest_variant: usize, id: String, options: &SvgOptions) -> Result<Renderer<'a>> {
    let base = variants[0].anim;
    let names = |a: &Value| -> Vec<Value> {
        a["layers"].as_array().map(|l| l.iter().map(|l| l["nm"].clone()).collect()).unwrap_or_default()
    };
    if variants.iter().any(|v| names(v.anim) != names(base)) {
        return Err(unsupported("the animations of one icon differ in layers"));
    }
    if base["layers"].as_array().is_none_or(Vec::is_empty) {
        return Err(SvgError::Invalid("The animation has no layers".into()));
    }
    let many = variants.len() > 1;
    Ok(Renderer {
        base,
        variants,
        rest,
        rest_variant,
        id,
        color: options.color.clone(),
        slots: slot_values(options.style, options.weight),
        many,
        defs: Vec::new(),
        masks: 0,
    })
}

fn variant<'a>(anim: &'a Value, window: (f64, f64), begin: String, extra: &str) -> Variant<'a> {
    let fps = f(&anim["fr"]);
    let timing = format!("dur=\"{}s\" begin=\"{begin}\"{extra}", num((window.1 - window.0) / fps, 4));
    Variant { anim, w0: window.0, w1: window.1, timing }
}

/// One Lottie animation as an animated SVG (no stateful toggle). `window` is the frame
/// range played (the whole timeline when `None`), `rest` the frame shown before and after
/// (the window's start when `None`). Uses `options`' look, id, size and trigger (`hover`,
/// `click`, `loop`, `load` or `js`); `motion` and `state` don't apply.
pub fn lottie_to_svg(animation: &Value, window: Option<(f64, f64)>, rest: Option<f64>, options: &SvgOptions) -> Result<String> {
    let trigger = options.trigger;
    if !matches!(trigger, Trigger::Hover | Trigger::Click | Trigger::Loop | Trigger::Load | Trigger::Js) {
        return Err(SvgError::Options("lottie_to_svg plays one animation: use hover, click, loop, load or js".into()));
    }
    check_look(options)?;
    let id = svg_id(options.id.as_deref().unwrap_or("icon"))?;
    let window = window.unwrap_or((f(&animation["ip"]), f(&animation["op"])));
    let rest = rest.unwrap_or(window.0);
    let begin = if matches!(trigger, Trigger::Loop | Trigger::Load) { "0s".to_string() } else { format!("{id}_m.begin") };
    let extra = if trigger == Trigger::Loop { " repeatCount=\"indefinite\"" } else { "" };
    let ui = match trigger {
        Trigger::Hover => Ui::Hover,
        Trigger::Click => Ui::Click,
        Trigger::Js => Ui::Js,
        _ => Ui::None,
    };
    renderer(vec![variant(animation, window, begin, extra)], rest, 0, id, options)?.render(ui, options.size)
}

/// One motion of an icon record (descriptor or `set.json` entry).
struct Action<'a> {
    name: &'a str,
    fps: f64,
    frames: f64,
    active: f64,
    rest: f64,
    continuous: bool,
}

impl<'a> Action<'a> {
    fn read(m: &'a Value) -> Result<Action<'a>> {
        let number = |key: &str| {
            m[key].as_f64().filter(|n| n.is_finite()).ok_or_else(|| SvgError::Invalid(format!("The icon's motion has no {key}")))
        };
        Ok(Action {
            name: m["name"].as_str().ok_or_else(|| SvgError::Invalid("The icon's motion has no name".into()))?,
            fps: number("fps")?,
            frames: number("frames")?,
            active: number("active")?,
            rest: number("rest")?,
            continuous: truthy(&m["continuous"]),
        })
    }

    /// The runtime plays frame 0 to `ceil(active * fps)`, then rests on `rest`; loaders run
    /// their whole timeline.
    fn window(&self) -> (f64, f64) {
        if self.continuous {
            (0.0, self.frames)
        } else {
            (0.0, (self.frames - 1.0).min((self.active * self.fps).ceil()).max(self.rest))
        }
    }
}

/// The trigger `auto` resolves to: toggle for stateful icons, loop for loaders, else hover.
pub fn default_trigger(icon: &Value, motion: &str) -> Trigger {
    let motions = icon["motions"].as_array().map(Vec::as_slice).unwrap_or_default();
    let action = motions.iter().find(|m| m["motionId"] == motion).or(motions.first());
    if truthy(&icon["stateful"]) {
        Trigger::Toggle
    } else if action.is_some_and(|a| truthy(&a["continuous"])) {
        Trigger::Loop
    } else {
        Trigger::Hover
    }
}

/// An icon as an animated SVG, the way the runtime plays it. `animation` returns the
/// Lottie JSON of an animation name (`motions[].name`).
pub fn icon_to_svg(icon: &Value, animation: &dyn Fn(&str) -> Option<Value>, options: &SvgOptions) -> Result<String> {
    let icon_id = icon["id"].as_str().unwrap_or("icon");
    let motions = icon["motions"].as_array().map(Vec::as_slice).unwrap_or_default();
    let motion = motions
        .iter()
        .find(|m| m["motionId"] == options.motion.as_str())
        .ok_or_else(|| SvgError::Options(format!("{icon_id} has no {} motion", options.motion)))?;
    let trigger = if options.trigger == Trigger::Auto { default_trigger(icon, &options.motion) } else { options.trigger };
    check_look(options)?;
    let id = svg_id(options.id.as_deref().unwrap_or(icon_id))?;
    let load = |name: &str| animation(name).ok_or_else(|| SvgError::Invalid(format!("Animation {name} is missing")));
    let forward = Action::read(motion)?;
    if truthy(&icon["stateful"]) {
        if !matches!(trigger, Trigger::Toggle | Trigger::Js) {
            return Err(SvgError::Options(format!("{icon_id} is stateful: use the toggle or js trigger")));
        }
        if motion["reverse"].is_null() {
            return Err(SvgError::Invalid(format!("{icon_id} has no reverse action")));
        }
        let reverse = Action::read(&motion["reverse"])?;
        let (start, end) = (icon["startState"].as_str(), icon["endState"].as_str());
        let state = options.state.as_deref().or(end);
        if state.is_none() || (state != start && state != end) {
            return Err(SvgError::Options(format!("Unknown state: {}", state.unwrap_or_default())));
        }
        // The first click leaves the initial state; the second comes back.
        let (leave, back) = if state == end { (reverse, forward) } else { (forward, reverse) };
        let (leave_anim, back_anim) = (load(leave.name)?, load(back.name)?);
        let begins: [&str; 2] = if trigger == Trigger::Toggle { ["_a.click", "_b.click"] } else { ["_a.begin", "_b.begin"] };
        let variants = vec![
            variant(&leave_anim, leave.window(), format!("{id}{}", begins[0]), " fill=\"freeze\""),
            variant(&back_anim, back.window(), format!("{id}{}", begins[1]), " fill=\"freeze\""),
        ];
        let ui = if trigger == Trigger::Toggle { Ui::Toggle } else { Ui::JsStates };
        return renderer(variants, back.rest, 1, id, options)?.render(ui, options.size);
    }
    if trigger == Trigger::Toggle {
        return Err(SvgError::Options(format!("{icon_id} is not stateful: use hover, click, loop, load or js")));
    }
    let window = if trigger == Trigger::Loop { (0.0, forward.frames) } else { forward.window() };
    let anim = load(forward.name)?;
    let options = SvgOptions { trigger, id: Some(id), ..options.clone() };
    lottie_to_svg(&anim, Some(window), Some(forward.rest), &options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_like_javascript_to_fixed() {
        // Exact binary ties round away from zero in JS and to even in Rust's formatter.
        assert_eq!(num(0.0625, 3), ".063");
        assert_eq!(num(-0.0625, 3), "-.063");
        assert_eq!(num(0.1875, 3), ".188");
        assert_eq!(num(2.5, 0), "3");
        assert_eq!(num(0.125, 2), ".13");
        assert_eq!(num(1.005, 2), "1"); // 1.00499999999999989...
        assert_eq!(num(-0.0001, 3), "0");
        assert_eq!(num(16.0, 3), "16");
        assert_eq!(num(1e-7, 7), "1e-7");
        assert_eq!(num(0.1666666, 5), ".16667");
        assert_eq!(num(-12.34567, 3), "-12.346");
        assert_eq!(pt(1.5, -2.0), "1.5-2");
    }

    #[test]
    fn hypot_follows_v8() {
        assert_eq!(js_hypot(&[3.0, 4.0]), 5.0);
        assert_eq!(js_hypot(&[0.0, 0.0]), 0.0);
        assert_eq!(js_hypot(&[-0.006, 0.008]), (0.006f64 / 0.008 * (0.006 / 0.008) + 1.0).sqrt() * 0.008);
    }

    #[test]
    fn ids_and_options_are_checked() {
        assert_eq!(svg_id("bell-strong").unwrap(), "bell_strong");
        assert!(svg_id("9lives").is_err() && svg_id("a.b").is_err() && svg_id("").is_err());
        let bad = |o: SvgOptions| check_look(&o).unwrap_err().to_string();
        assert!(bad(SvgOptions { weight: 3.5, ..Default::default() }).contains("weight"));
        assert!(bad(SvgOptions { color: "red".into(), ..Default::default() }).contains("color"));
        assert!(bad(SvgOptions { color: "#12345".into(), ..Default::default() }).contains("color"));
        assert!(check_look(&SvgOptions { color: "#ABC".into(), size: Some(24.0), ..Default::default() }).is_ok());
    }

    #[test]
    fn overshooting_easing_is_split_into_valid_splines() {
        // a back-out curve: y peaks above 1
        let pieces = valid_pieces(&[0.3, 1.6, 0.6, 1.0], 0.0, 1.0, 0);
        assert!(pieces.len() > 1);
        for (_, _, s) in pieces {
            assert!(s.iter().all(|v| (0.0..=1.0).contains(v)), "{s:?}");
        }
        assert_eq!(valid_pieces(&[1.0, 1.0, 0.0, 0.0], 0.0, 1.0, 0), vec![(0.0, 1.0, LINEAR)]);
    }
}
