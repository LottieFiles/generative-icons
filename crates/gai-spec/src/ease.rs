//! Keys and eases: a spec key says how the motion ARRIVES at it. This module turns key
//! lists into Lottie keys with exact cubic-bezier handles, evaluates them as a Lottie
//! player does, reverses them exactly, and splits a track at its zero crossings (the
//! contact pivot: one rig per sign, every pivot constant).
//!
//! - `auto`, `hit`, `launch`, `pass`: a cubic Hermite with v1 `flow`'s tangents, written
//!   as `[1/3, m0·h/3Δ, 2/3, 1 − m1·h/3Δ]`. Exact. Unlike v1, `hit` keeps its 1.6× in-tangent
//!   when a hold follows and on the last key (hit-then-hold), and `launch` its 1.6×
//!   out-tangent on the first key.
//! - `linear`, `in`, `out`, `inOut`: exact cubics. `sine`: v1's cosine to 7.9e-5.
//! - `step`: hold the previous value, jump at this key (`h:1`).

use crate::spec::Ease;

pub const LINEAR: [f64; 4] = [1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0];
pub const IN: [f64; 4] = [1.0 / 3.0, 0.0, 2.0 / 3.0, 0.0];
pub const OUT: [f64; 4] = [1.0 / 3.0, 1.0, 2.0 / 3.0, 1.0];
pub const IN_OUT: [f64; 4] = [1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0];
/// v1's cosine in-out (`track`'s default), minimax fit: within 7.9e-5 of the move.
pub const SINE: [f64; 4] = [0.3628, -0.0016, 0.6372, 1.0016];

/// A key as authored (after preset expansion): arrival ease included.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub t: f64,
    pub v: f64,
    pub ease: Ease,
}

impl Key {
    pub fn new(t: f64, v: f64, ease: Ease) -> Key {
        Key { t, v, ease }
    }
}

/// The ease of the segment that starts at a key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Handle {
    /// `h:1`: keep this key's value until the next key.
    Hold,
    /// Lottie `o.x, o.y, i.x, i.y`.
    Bez([f64; 4]),
}

/// A Lottie key: value and the handle of the segment it starts (none on the last key).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CKey {
    pub t: f64,
    pub v: f64,
    pub out: Option<Handle>,
}

/// Lottie keys with exact handles for an authored key list (times strictly increasing).
pub fn compile(keys: &[Key]) -> Vec<CKey> {
    let n = keys.len();
    let sec = |k: usize| (keys[k + 1].v - keys[k].v) / (keys[k + 1].t - keys[k].t);
    let tangent = |k: usize, incoming: bool| -> f64 {
        match keys[k].ease {
            Ease::Hit | Ease::Launch => {
                let (gain_in, gain_out) = if keys[k].ease == Ease::Hit { (1.6, 0.7) } else { (0.7, 1.6) };
                match incoming {
                    true if k > 0 => gain_in * sec(k - 1),
                    false if k + 1 < n => gain_out * sec(k),
                    _ => 0.0,
                }
            }
            tag => {
                if k == 0 || k + 1 == n {
                    return 0.0;
                }
                let (a, b) = (sec(k - 1), sec(k));
                if a == 0.0 || b == 0.0 || a * b <= 0.0 {
                    return 0.0;
                }
                let h0 = keys[k].t - keys[k - 1].t;
                let h1 = keys[k + 1].t - keys[k].t;
                let (w0, w1) = (2.0 * h1 + h0, h1 + 2.0 * h0);
                let slope = (w0 + w1) / (w0 / a + w1 / b) * if tag == Ease::Pass { 1.35 } else { 1.0 };
                a.signum() * slope.abs().min(3.0 * a.abs().min(b.abs()))
            }
        }
    };
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let k = keys[i];
        if i + 1 == n {
            out.push(CKey { t: k.t, v: k.v, out: None });
            break;
        }
        let next = keys[i + 1];
        let delta = next.v - k.v;
        let h = next.t - k.t;
        let handle = if next.ease == Ease::Step || delta == 0.0 {
            Handle::Hold
        } else {
            Handle::Bez(match next.ease {
                Ease::Linear => LINEAR,
                Ease::In => IN,
                Ease::Out => OUT,
                Ease::InOut => IN_OUT,
                Ease::Sine => SINE,
                Ease::Bezier(b) => b,
                Ease::Auto | Ease::Hit | Ease::Launch | Ease::Pass | Ease::Step => {
                    let m0 = tangent(i, false);
                    let m1 = tangent(i + 1, true);
                    [1.0 / 3.0, m0 * h / (3.0 * delta), 2.0 / 3.0, 1.0 - m1 * h / (3.0 * delta)]
                }
            })
        };
        out.push(CKey { t: k.t, v: k.v, out: Some(handle) });
    }
    out
}

/// The cubic-bezier ease `B(s)` of one coordinate with control values `p1`, `p2` (0 and 1
/// at the ends).
fn bez(p1: f64, p2: f64, s: f64) -> f64 {
    let u = 1.0 - s;
    3.0 * u * u * s * p1 + 3.0 * u * s * s * p2 + s * s * s
}

/// The parameter where the ease's time coordinate reaches `x` (handles' x in 0..1).
pub fn solve_x(x1: f64, x2: f64, x: f64) -> f64 {
    let (x1, x2) = (x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0));
    let mut s = x;
    for _ in 0..12 {
        let f = bez(x1, x2, s) - x;
        let u = 1.0 - s;
        let d = 3.0 * u * u * x1 + 6.0 * u * s * (x2 - x1) + 3.0 * s * s * (1.0 - x2);
        if d.abs() < 1e-9 {
            break;
        }
        let next = s - f / d;
        if !(0.0..=1.0).contains(&next) {
            break;
        }
        s = next;
        if f.abs() < 1e-13 {
            return s;
        }
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if bez(x1, x2, mid) < x { lo = mid } else { hi = mid }
    }
    (lo + hi) / 2.0
}

/// Progress (0 → 1, may overshoot) of a segment with this handle at `u` of its duration.
pub fn progress(handle: Handle, u: f64) -> f64 {
    match handle {
        Handle::Hold => 0.0,
        Handle::Bez([x1, y1, x2, y2]) => bez(y1, y2, solve_x(x1, x2, u)),
    }
}

/// The value of Lottie keys at time `t` (held before the first and after the last key).
pub fn eval(keys: &[CKey], t: f64) -> f64 {
    let Some(first) = keys.first() else { return 0.0 };
    if t <= first.t {
        return first.v;
    }
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        if t < b.t {
            let u = (t - a.t) / (b.t - a.t);
            return a.v + (b.v - a.v) * progress(a.out.unwrap_or(Handle::Bez(LINEAR)), u);
        }
    }
    keys.last().map_or(0.0, |k| k.v)
}

/// Each key's value mapped by `f`, which must be affine (`a·v + b`), so handles stay exact.
pub fn map_values(keys: &[CKey], f: impl Fn(f64) -> f64) -> Vec<CKey> {
    keys.iter().map(|k| CKey { v: f(k.v), ..*k }).collect()
}

/// The exact time reversal over `[0, end]`: key `(end − t, v)`, each segment's handle
/// `[1−x2, 1−y2, 1−x1, 1−y1]`. A hold becomes a jump one frame (or half the segment) after
/// the mirrored start of its segment.
pub fn mirror(keys: &[CKey], end: f64) -> Vec<CKey> {
    let n = keys.len();
    let mut out: Vec<CKey> = Vec::new();
    for j in 0..n {
        let k = keys[n - 1 - j];
        let t = end - k.t;
        if j + 1 == n {
            out.push(CKey { t, v: k.v, out: None });
            break;
        }
        let prev = keys[n - 2 - j];
        match prev.out {
            Some(Handle::Bez([x1, y1, x2, y2])) => {
                out.push(CKey { t, v: k.v, out: Some(Handle::Bez([1.0 - x2, 1.0 - y2, 1.0 - x1, 1.0 - y1])) })
            }
            _ if prev.v == k.v => out.push(CKey { t, v: k.v, out: Some(Handle::Hold) }),
            _ => {
                let eps = (1.0 / 60.0f64).min((k.t - prev.t) / 2.0);
                out.push(CKey { t, v: k.v, out: Some(Handle::Hold) });
                out.push(CKey { t: t + eps, v: prev.v, out: Some(Handle::Hold) });
            }
        }
    }
    out
}

/// One piece of a track: control points in (time, value).
#[derive(Debug, Clone, Copy)]
struct Piece {
    q: [(f64, f64); 4],
    hold: bool,
}

impl Piece {
    fn value_at(&self, s: f64) -> f64 {
        let u = 1.0 - s;
        u * u * u * self.q[0].1 + 3.0 * u * u * s * self.q[1].1 + 3.0 * u * s * s * self.q[2].1 + s * s * s * self.q[3].1
    }

    /// De Casteljau at `s`.
    fn split(&self, s: f64) -> (Piece, Piece) {
        let l = |a: (f64, f64), b: (f64, f64)| (a.0 + (b.0 - a.0) * s, a.1 + (b.1 - a.1) * s);
        let [p0, p1, p2, p3] = self.q;
        let (a, b, c) = (l(p0, p1), l(p1, p2), l(p2, p3));
        let (d, e) = (l(a, b), l(b, c));
        let m = l(d, e);
        (Piece { q: [p0, a, d, m], hold: false }, Piece { q: [m, e, c, p3], hold: false })
    }

    fn handle(&self) -> Handle {
        let [p0, p1, p2, p3] = self.q;
        let dt = p3.0 - p0.0;
        let dv = p3.1 - p0.1;
        if self.hold || dv.abs() < 1e-12 {
            return Handle::Hold;
        }
        Handle::Bez([(p1.0 - p0.0) / dt, (p1.1 - p0.1) / dv, (p2.0 - p0.0) / dt, (p2.1 - p0.1) / dv])
    }
}

/// Splits a track at its zero crossings into the part above zero and the part below:
/// `(max(v, 0), min(v, 0))`, each exact (de Casteljau). A rig per sign with a constant
/// pivot each reproduces a pivot that switches at zero.
pub fn split_sign(keys: &[CKey]) -> (Vec<CKey>, Vec<CKey>) {
    let mut pieces: Vec<Piece> = Vec::new();
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        let h = b.t - a.t;
        let dv = b.v - a.v;
        match a.out {
            Some(Handle::Hold) | None => pieces.push(Piece { q: [(a.t, a.v), (a.t, a.v), (b.t, a.v), (b.t, b.v)], hold: true }),
            Some(Handle::Bez([x1, y1, x2, y2])) => {
                let piece = Piece { q: [(a.t, a.v), (a.t + x1 * h, a.v + y1 * dv), (a.t + x2 * h, a.v + y2 * dv), (b.t, b.v)], hold: false };
                pieces.extend(split_at_roots(piece));
            }
        }
    }
    let build = |positive: bool| -> Vec<CKey> {
        let clamp = |v: f64| if positive { v.max(0.0) } else { v.min(0.0) };
        let Some(first) = keys.first() else { return Vec::new() };
        if pieces.is_empty() {
            return vec![CKey { t: first.t, v: clamp(first.v), out: None }];
        }
        let mut out = vec![CKey { t: first.t, v: clamp(first.v), out: None }];
        for p in &pieces {
            let keep = if p.hold {
                false
            } else {
                let mid = p.value_at(0.5);
                if positive { mid > 0.0 } else { mid < 0.0 }
            };
            let handle = if keep { p.handle() } else { Handle::Hold };
            if let Some(last) = out.last_mut() {
                last.out = Some(handle);
            }
            out.push(CKey { t: p.q[3].0, v: clamp_zero(clamp(p.q[3].1)), out: None });
        }
        simplify(out)
    };
    (build(true), build(false))
}

fn clamp_zero(v: f64) -> f64 {
    if v.abs() < 1e-12 { 0.0 } else { v }
}

/// Splits a piece at every zero of its value in (0, 1); a piece that starts and ends at
/// zero but bulges is split at its middle so each part has a change to normalise by.
fn split_at_roots(piece: Piece) -> Vec<Piece> {
    let mut roots = Vec::new();
    let steps = 64;
    let mut prev = piece.value_at(0.0);
    for i in 1..=steps {
        let s = i as f64 / steps as f64;
        let v = piece.value_at(s);
        let crosses = (prev < 0.0 && v > 0.0) || (prev > 0.0 && v < 0.0);
        if crosses {
            let (mut lo, mut hi) = ((i - 1) as f64 / steps as f64, s);
            let lo_sign = prev.signum();
            for _ in 0..80 {
                let mid = (lo + hi) / 2.0;
                if piece.value_at(mid).signum() == lo_sign { lo = mid } else { hi = mid }
            }
            let root = (lo + hi) / 2.0;
            if root > 1e-9 && root < 1.0 - 1e-9 {
                roots.push(root);
            }
        }
        prev = v;
    }
    let mut out = Vec::new();
    let mut rest = piece;
    let mut consumed = 0.0;
    for r in roots {
        let local = (r - consumed) / (1.0 - consumed);
        let (a, b) = rest.split(local);
        let mut a = a;
        a.q[3].1 = 0.0;
        let mut b = b;
        b.q[0].1 = 0.0;
        out.push(a);
        rest = b;
        consumed = r;
    }
    out.push(rest);
    // Pieces from zero back to zero cannot be normalised: split them at the middle.
    let mut final_pieces = Vec::new();
    for p in out {
        if (p.q[3].1 - p.q[0].1).abs() < 1e-12 && p.value_at(0.5).abs() > 1e-9 {
            let (a, b) = p.split(0.5);
            final_pieces.push(a);
            final_pieces.push(b);
        } else {
            final_pieces.push(p);
        }
    }
    final_pieces
}

/// Drops inner keys between two flat segments at the same value.
pub fn simplify(keys: Vec<CKey>) -> Vec<CKey> {
    let mut out: Vec<CKey> = Vec::with_capacity(keys.len());
    for (i, k) in keys.iter().enumerate() {
        let redundant = i > 0
            && i + 1 < keys.len()
            && out.last().is_some_and(|p| p.v == k.v && matches!(p.out, Some(Handle::Hold)))
            && keys[i + 1].v == k.v;
        if !redundant {
            out.push(*k);
        }
    }
    out
}

/// Whether every key holds the same value.
pub fn is_constant(keys: &[CKey], value: f64) -> bool {
    keys.iter().all(|k| (k.v - value).abs() < 1e-12)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(list: &[(f64, f64, Ease)]) -> Vec<Key> {
        list.iter().map(|&(t, v, e)| Key::new(t, v, e)).collect()
    }

    /// v1's `flow` (direction.mjs), for comparison.
    fn flow(keys: &[Key], t: f64) -> f64 {
        let n = keys.len();
        if t <= keys[0].t {
            return keys[0].v;
        }
        if t >= keys[n - 1].t {
            return keys[n - 1].v;
        }
        let mut i = 0;
        while keys[i + 1].t <= t {
            i += 1;
        }
        let span = |k: usize| keys[k + 1].t - keys[k].t;
        let sec = |k: usize| (keys[k + 1].v - keys[k].v) / span(k);
        let tangent = |k: usize, inside: bool| -> f64 {
            if k == 0 || k == n - 1 {
                return 0.0;
            }
            let (a, b) = (sec(k - 1), sec(k));
            if a == 0.0 || b == 0.0 {
                return 0.0;
            }
            if keys[k].ease == Ease::Hit {
                return if inside { a * 1.6 } else { b * 0.7 };
            }
            if a * b <= 0.0 {
                return 0.0;
            }
            let (h0, h1) = (span(k - 1), span(k));
            let (w0, w1) = (2.0 * h1 + h0, h1 + 2.0 * h0);
            let slope = (w0 + w1) / (w0 / a + w1 / b);
            a.signum() * slope.abs().min(3.0 * a.abs().min(b.abs()))
        };
        let d = sec(i);
        if d == 0.0 {
            return keys[i].v;
        }
        let h = span(i);
        let u = (t - keys[i].t) / h;
        let (u2, u3) = (u * u, u * u * u);
        let m0 = tangent(i, false) * h;
        let m1 = tangent(i + 1, true) * h;
        keys[i].v * (2.0 * u3 - 3.0 * u2 + 1.0) + m0 * (u3 - 2.0 * u2 + u) + keys[i + 1].v * (-2.0 * u3 + 3.0 * u2) + m1 * (u3 - u2)
    }

    #[test]
    fn auto_reproduces_flow() {
        let a = Ease::Auto;
        let spring = keys(&[(0.0, 0.0, a), (0.07, 5.0, a), (0.17, -20.0, Ease::Hit), (0.29, 13.0, a), (0.41, -7.0, a), (0.53, 3.0, a), (0.68, 0.0, a)]);
        let compiled = compile(&spring);
        for f in 0..=60 {
            let t = f as f64 / 60.0 * 0.7;
            assert!((eval(&compiled, t) - flow(&spring, t)).abs() < 1e-9, "t {t}");
        }
        // The probe's alarm plan: hit in-tangent 1.6, out-tangent 0.7.
        assert!(matches!(compiled[1].out, Some(Handle::Bez(h)) if (h[3] - (1.0 - 1.6 / 3.0)).abs() < 1e-9));
        assert!(matches!(compiled[2].out, Some(Handle::Bez(h)) if (h[1] - 0.7 / 3.0).abs() < 1e-9));
    }

    #[test]
    fn hit_then_hold_keeps_its_speed() {
        let c = compile(&keys(&[(0.03, 0.0, Ease::Auto), (0.18, 112.0, Ease::Hit), (0.5, 112.0, Ease::Auto), (0.68, 0.0, Ease::In)]));
        assert!(matches!(c[0].out, Some(Handle::Bez(h)) if h[1] == 0.0 && (h[3] - (1.0 - 1.6 / 3.0)).abs() < 1e-12));
        assert_eq!(c[1].out, Some(Handle::Hold));
        assert_eq!(c[2].out, Some(Handle::Bez(IN)));
    }

    #[test]
    fn mirror_is_an_exact_time_reversal() {
        let c = compile(&keys(&[(0.0, 0.0, Ease::Auto), (0.2, 10.0, Ease::Out), (0.5, -3.0, Ease::Hit), (0.7, 0.0, Ease::Auto)]));
        let m = mirror(&c, 0.7);
        for f in 0..=42 {
            let t = f as f64 / 60.0;
            assert!((eval(&m, t) - eval(&c, 0.7 - t)).abs() < 1e-6, "t {t}: {} vs {}", eval(&m, t), eval(&c, 0.7 - t));
        }
    }

    #[test]
    fn sign_split_sums_back_exactly() {
        let a = Ease::Auto;
        let rock = keys(&[(0.0, 0.0, a), (0.07, -1.3, a), (0.17, 5.5, Ease::Hit), (0.33, -1.7, a), (0.48, 0.5, a), (0.66, 0.0, a)]);
        let c = compile(&rock);
        let (pos, neg) = split_sign(&c);
        for f in 0..=60 {
            let t = f as f64 / 60.0 * 0.7;
            let (p, n) = (eval(&pos, t), eval(&neg, t));
            assert!(p >= -1e-9 && n <= 1e-9, "t {t}: {p} {n}");
            assert!((p + n - eval(&c, t)).abs() < 1e-7, "t {t}: {} + {} vs {}", p, n, eval(&c, t));
        }
        // The probe found crossings at 0.109215 s, 0.272583 s and 0.433669 s.
        let times: Vec<f64> = pos.iter().map(|k| (k.t * 1e6).round() / 1e6).collect();
        for crossing in [0.109215, 0.272583, 0.433669] {
            assert!(times.iter().any(|t| (t - crossing).abs() < 2e-6), "{times:?}");
        }
    }
}
