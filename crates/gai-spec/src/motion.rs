//! Motions: each track's key list for Light and Strong with presets expanded and follows
//! resolved, the derived timing, and the reverse motions of a stateful icon.

use crate::diag::{Diagnostic, Diagnostics, at};
use crate::ease::{self, CKey, Handle, Key};
use crate::spec::{DURATION, Ease, Follow, Items, KeyItem, MAX_ACTIVE, MotionValue, Prop, ReverseMode, Spec, Target, TrackSrc};
use indexmap::IndexMap;

/// Light or Strong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Which {
    Light,
    Strong,
}

impl Which {
    pub fn id(self) -> &'static str {
        match self {
            Which::Light => "light",
            Which::Strong => "strong",
        }
    }
}

/// One resolved track: authored keys (after presets) and where they came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub target: Target,
    pub keys: Vec<Key>,
    pub path: String,
}

/// A track as Lottie keys.
#[derive(Debug, Clone, PartialEq)]
pub struct CTrack {
    pub target: Target,
    pub keys: Vec<CKey>,
    pub path: String,
}

/// One animation of the icon.
#[derive(Debug, Clone, PartialEq)]
pub struct Motion {
    /// `<id>`, `<id>-strong`, `<id>-reverse`, `<id>-reset-strong`, …
    pub name: String,
    pub which: Which,
    pub reverse: bool,
    /// Seconds: the end of the action.
    pub active: f64,
    /// Seconds: whole frames after `active + 0.04`.
    pub rest: f64,
    /// Authored keys of each track (forward motions and auto reverses).
    pub raw: Vec<Track>,
    pub tracks: Vec<CTrack>,
}

/// `ceil(60 × (active + 0.04)) / 60`, as v1's host computes it.
pub fn rest_of(active: f64) -> f64 {
    ((active + 0.04) * 60.0).ceil() / 60.0
}

/// The rest value of a property: offsets are 0, factors 1, opacity 1 (0 for a hidden part),
/// draw 1.
pub fn rest_value(prop: Option<Prop>, opacity: f64) -> f64 {
    match prop {
        Some(Prop::Scale | Prop::Squash | Prop::Draw) => 1.0,
        Some(Prop::Opacity) => opacity,
        _ => 0.0,
    }
}

/// Expands a key list's presets into keys. `rest` is the value before the first key.
pub fn expand(items: &Items, rest: f64, d: &mut Diagnostics) -> Option<Vec<Key>> {
    let mut out: Vec<Key> = Vec::new();
    let mut ok = true;
    for (item, path) in items {
        let last = out.last().copied();
        let prev = if out.len() >= 2 { Some(out[out.len() - 2]) } else { None };
        let push = |k: Key, out: &mut Vec<Key>, d: &mut Diagnostics| {
            if let Some(l) = out.last()
                && k.t <= l.t + 1e-9
            {
                d.push(
                    Diagnostic::error("KEY_ORDER", path.clone(), format!("key at {} s is not after the previous key at {} s", r4(k.t), r4(l.t)))
                        .hint("key times must increase; a preset's time is when it ends"),
                );
                return false;
            }
            out.push(k);
            true
        };
        match *item {
            KeyItem::Key { t, v, ease } => ok &= push(Key::new(t, v, ease), &mut out, d),
            KeyItem::Hold { t } => match last {
                Some(l) => ok &= push(Key::new(t, l.v, Ease::Auto), &mut out, d),
                None => {
                    d.push(Diagnostic::error("PRESET_START", path.clone(), "hold needs a key before it").hint("start the list with [t, value]"));
                    ok = false;
                }
            },
            KeyItem::Bounce { t, amp } => match (last, prev) {
                (Some(l), Some(p)) => {
                    let mid = (l.t + t) / 2.0;
                    ok &= push(Key::new(mid, l.v - amp * (l.v - p.v), Ease::Auto), &mut out, d);
                    ok &= push(Key::new(t, l.v, Ease::Hit), &mut out, d);
                }
                _ => {
                    d.push(Diagnostic::error("PRESET_START", path.clone(), "bounce needs the move it rebounds from: two keys before it"));
                    ok = false;
                }
            },
            KeyItem::Settle { t, to, amp, decay, swings } => {
                let Some(l) = last else {
                    d.push(Diagnostic::error("PRESET_START", path.clone(), "settle needs a key before it"));
                    ok = false;
                    continue;
                };
                let target = to.unwrap_or(l.v);
                let first = if (target - l.v).abs() > 1e-12 {
                    -(l.v - target) * decay
                } else {
                    match prev {
                        Some(p) if (l.v - p.v).abs() > 1e-12 => amp * (l.v - p.v),
                        _ => {
                            d.push(
                                Diagnostic::error("PRESET_START", path.clone(), "settle without `to` swings by a fraction of the arriving move, and there is none")
                                    .hint("give `to`, or put a move before it"),
                            );
                            ok = false;
                            continue;
                        }
                    }
                };
                let n = swings as usize;
                let mut dev = first;
                for j in 1..=n {
                    let tj = l.t + (t - l.t) * j as f64 / (n + 1) as f64;
                    ok &= push(Key::new(tj, target + dev, Ease::Auto), &mut out, d);
                    dev *= -decay;
                }
                ok &= push(Key::new(t, target, Ease::Auto), &mut out, d);
            }
            KeyItem::Wobble { at: start, amp, until, swings } => {
                let base = last.map_or(rest, |l| l.v);
                if last.is_none_or(|l| start > l.t + 1e-9) {
                    ok &= push(Key::new(start, base, Ease::Auto), &mut out, d);
                }
                let n = swings as usize;
                for j in 1..=n {
                    let tj = start + (until - start) * j as f64 / (n + 1) as f64;
                    let dev = amp * 0.55f64.powi(j as i32 - 1) * if j % 2 == 1 { 1.0 } else { -1.0 };
                    ok &= push(Key::new(tj, base + dev, Ease::Auto), &mut out, d);
                }
                ok &= push(Key::new(until, base, Ease::Auto), &mut out, d);
            }
        }
    }
    ok.then_some(out)
}

/// The resolved forward tracks of one motion. `hidden` says which parts start hidden;
/// `pulsed` collects the parts a pulse shows.
/// `rest_opacity` is each part's opacity at rest (0 when hidden).
pub fn resolve(spec: &Spec, which: Which, rest_opacity: &dyn Fn(&str) -> f64, d: &mut Diagnostics) -> Vec<Track> {
    let mut done: IndexMap<Target, Track> = IndexMap::new();
    let mut follows: Vec<(Target, Follow, String)> = Vec::new();
    let add = |done: &mut IndexMap<Target, Track>, track: Track, d: &mut Diagnostics| {
        if done.contains_key(&track.target) {
            d.push(Diagnostic::error("DUPLICATE_TRACK", track.path.clone(), format!("{} is keyed twice in {}", track.target, which.id())).hint("a pulse also keys the part's opacity; remove the other track"));
            return;
        }
        done.insert(track.target.clone(), track);
    };
    for entry in &spec.motion {
        let value = match &entry.track {
            TrackSrc::Follow(f) => {
                follows.push((entry.target.clone(), f.clone(), entry.path.clone()));
                continue;
            }
            TrackSrc::PerMotion { light, strong } => match which {
                Which::Light => light.as_ref(),
                Which::Strong => strong.as_ref(),
            },
        };
        let Some((value, path)) = value else { continue };
        let rest = rest_value(entry.target.prop, rest_opacity(&entry.target.name));
        match value {
            MotionValue::Keys(items) => {
                if let Some(keys) = expand(items, rest, d) {
                    add(&mut done, Track { target: entry.target.clone(), keys, path: path.clone() }, d);
                }
            }
            MotionValue::Loop { keys, period, until } => {
                let Some(once) = expand(keys, rest, d) else { continue };
                if let Some(keys) = looped(&once, *period, until.unwrap_or(MAX_ACTIVE), path, d) {
                    add(&mut done, Track { target: entry.target.clone(), keys, path: path.clone() }, d);
                }
            }
            MotionValue::Pulse { start, dur, r, grow } => {
                let (start, dur) = (*start, *dur);
                let end = start + dur;
                if end + 0.02 > DURATION {
                    d.push(Diagnostic::error("TIMELINE", path.clone(), "the pulse ends after the timeline"));
                    continue;
                }
                let name = entry.target.name.clone();
                let opacity = vec![
                    Key::new(start, 0.0, Ease::Auto),
                    Key::new(start + 0.04_f64.min(dur / 3.0), 1.0, Ease::Out),
                    Key::new(start + dur * 0.55, 1.0, Ease::Auto),
                    Key::new(end, 0.0, Ease::In),
                ];
                add(&mut done, Track { target: Target { name: name.clone(), prop: Some(Prop::Opacity) }, keys: opacity, path: path.clone() }, d);
                for (prop, amount) in [(Prop::R, r), (Prop::Grow, grow)] {
                    if let Some(amount) = amount {
                        // Swell while visible, then snap back to rest while hidden.
                        let keys = vec![Key::new(start, 0.0, Ease::Auto), Key::new(end, *amount, Ease::Out), Key::new(end + 0.02, 0.0, Ease::Step)];
                        add(&mut done, Track { target: Target { name: name.clone(), prop: Some(prop) }, keys, path: path.clone() }, d);
                    }
                }
            }
            MotionValue::Follow(f) => follows.push((entry.target.clone(), f.clone(), path.clone())),
        }
    }
    // Follows, in dependency order.
    let mut pending = follows;
    while !pending.is_empty() {
        let before = pending.len();
        let mut later = Vec::new();
        for (target, f, path) in pending {
            match done.get(&f.target) {
                Some(source) => {
                    let keys: Vec<Key> = source.keys.iter().map(|k| Key::new(k.t + f.lag, f.scale * k.v + f.offset, k.ease)).collect();
                    if let Some(k) = keys.last().filter(|k| k.t > DURATION) {
                        d.push(Diagnostic::error("TIMELINE", at(&path, "lag"), format!("the follow ends at {} s, after the timeline", r4(k.t))));
                        continue;
                    }
                    add(&mut done, Track { target, keys, path }, d);
                }
                None => later.push((target, f, path)),
            }
        }
        if later.len() == before {
            // Nothing resolved: followed tracks absent from this motion stay static, unless
            // they follow each other in a cycle.
            for (target, f, path) in &later {
                if later.iter().any(|(t, _, _)| *t == f.target) {
                    d.push(Diagnostic::error("FOLLOW_CYCLE", at(path, "follow"), format!("{target} follows {} in a cycle", f.target)));
                }
            }
            break;
        }
        pending = later;
    }
    done.into_values().collect()
}

/// `keys` repeated every `period` up to `until`; a loop must end where it starts.
fn looped(once: &[Key], period: f64, until: f64, path: &str, d: &mut Diagnostics) -> Option<Vec<Key>> {
    let (first, last) = (once.first()?, once.last()?);
    if (first.v - last.v).abs() > 1e-9 {
        d.push(Diagnostic::error("LOOP_SEAM", path, "a loop must end at the value it starts with"));
        return None;
    }
    if last.t - first.t > period + 1e-9 {
        d.push(Diagnostic::error("LOOP_SEAM", path, "the loop's keys span more than its period"));
        return None;
    }
    let mut out: Vec<Key> = Vec::new();
    let mut copy = 0;
    loop {
        let shift = period * copy as f64;
        if first.t + shift > until + 1e-9 {
            break;
        }
        for k in once {
            let t = k.t + shift;
            if t > until + 1e-9 {
                break;
            }
            if out.last().is_some_and(|l: &Key| (l.t - t).abs() < 1e-9) {
                continue;
            }
            out.push(Key::new(t, k.v, k.ease));
        }
        copy += 1;
        if copy > 200 {
            break;
        }
    }
    Some(out)
}

/// Compiles authored tracks to Lottie keys.
pub fn compile_tracks(tracks: &[Track]) -> Vec<CTrack> {
    tracks.iter().map(|t| CTrack { target: t.target.clone(), keys: ease::compile(&t.keys), path: t.path.clone() }).collect()
}

/// The time of the motion's last key.
pub fn last_key(tracks: &[CTrack]) -> f64 {
    tracks.iter().filter_map(|t| t.keys.last()).map(|k| k.t).fold(0.0, f64::max)
}

/// Beats of one track: runs of keys between rests, as key index ranges.
fn beats(keys: &[Key]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    for i in 0..keys.len().saturating_sub(1) {
        let moving = (keys[i + 1].v - keys[i].v).abs() > 1e-12;
        match (moving, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                out.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push((s, keys.len() - 1));
    }
    out
}

/// The `auto` reverse of one track over `[0, end]`: its beats replay in mirrored order at
/// mirrored positions; inside a beat, key times and eases are kept and each value is
/// reflected about the beat's ends (`v' = v_start + v_end − v`).
pub fn reverse_auto(keys: &[Key], end: f64) -> Vec<Key> {
    let list = beats(keys);
    if list.is_empty() {
        return keys.first().map(|k| vec![Key::new(0.0, k.v, Ease::Auto)]).unwrap_or_default();
    }
    let mut reflected: Vec<Vec<Key>> = list
        .iter()
        .map(|&(s, e)| {
            let (ts, te, vs, ve) = (keys[s].t, keys[e].t, keys[s].v, keys[e].v);
            (s..=e).map(|i| Key::new(end - te + (keys[i].t - ts), vs + ve - keys[i].v, keys[i].ease)).collect()
        })
        .collect();
    reflected.reverse();
    let mut out: Vec<Key> = Vec::new();
    for beat in reflected {
        for k in beat {
            if out.last().is_some_and(|l| k.t <= l.t + 1e-9) {
                continue;
            }
            out.push(k);
        }
    }
    out
}

/// What builds a reverse motion.
pub struct ReverseInput<'a> {
    pub mode: ReverseMode,
    pub forward: &'a Motion,
    /// Explicit keys replacing tracks (`reverse.light|strong`).
    pub overrides: Vec<Track>,
    /// Timing override.
    pub active: Option<f64>,
    /// For `fade`: the opacity target of every part the forward motion changes, with its
    /// forward opacity at the start and the end.
    pub fading: Vec<(Target, f64, f64)>,
    /// `state.reverseName`, replacing `<id>-reverse` (or `<id>-reset`).
    pub name: Option<String>,
}

/// The reverse of a forward motion.
pub fn reverse(id: &str, input: ReverseInput<'_>, d: &mut Diagnostics) -> Motion {
    let forward = input.forward;
    let end = last_key(&forward.tracks);
    let mut raw = Vec::new();
    let mut tracks: Vec<CTrack> = match input.mode {
        ReverseMode::Auto => {
            raw = forward.raw.iter().map(|t| Track { target: t.target.clone(), keys: reverse_auto(&t.keys, end), path: t.path.clone() }).collect();
            compile_tracks(&raw)
        }
        ReverseMode::Mirror => forward.tracks.iter().map(|t| CTrack { target: t.target.clone(), keys: ease::mirror(&t.keys, end), path: t.path.clone() }).collect(),
        ReverseMode::Fade => {
            // Preview reset: changing parts fade out over 0.08 s, snap while hidden at
            // 0.12 s and fade back in by 0.25 s.
            let mut out: Vec<CTrack> = forward
                .tracks
                .iter()
                .filter(|t| t.target.prop != Some(Prop::Opacity))
                .map(|t| {
                    let (start, stop) = (t.keys.first().map_or(0.0, |k| k.v), t.keys.last().map_or(0.0, |k| k.v));
                    CTrack {
                        target: t.target.clone(),
                        keys: vec![CKey { t: 0.0, v: stop, out: Some(Handle::Hold) }, CKey { t: 0.12, v: start, out: None }],
                        path: t.path.clone(),
                    }
                })
                .collect();
            for (target, start, stop) in &input.fading {
                let keys = vec![
                    Key::new(0.0, *stop, Ease::Auto),
                    Key::new(0.08, 0.0, Ease::Out),
                    Key::new(0.15, 0.0, Ease::Auto),
                    Key::new(0.25, *start, Ease::Out),
                ];
                out.push(CTrack { target: target.clone(), keys: ease::compile(&keys), path: String::new() });
            }
            out
        }
    };
    for o in input.overrides {
        let compiled = ease::compile(&o.keys);
        match tracks.iter_mut().find(|t| t.target == o.target) {
            Some(t) => *t = CTrack { target: o.target.clone(), keys: compiled, path: o.path.clone() },
            None => tracks.push(CTrack { target: o.target.clone(), keys: compiled, path: o.path.clone() }),
        }
        match raw.iter_mut().find(|t| t.target == o.target) {
            Some(t) => *t = o,
            None => raw.push(o),
        }
    }
    let fade = input.mode == ReverseMode::Fade;
    let base = input.name.clone().unwrap_or_else(|| format!("{id}{}", if fade { "-reset" } else { "-reverse" }));
    let suffix = match forward.which {
        Which::Light => String::new(),
        Which::Strong => "-strong".to_string(),
    };
    let derived = if fade {
        if forward.which == Which::Light { 0.28 } else { 0.30 }
    } else {
        last_key(&tracks)
    };
    let active = input.active.unwrap_or(derived);
    let name = format!("{base}{suffix}");
    check_timing(&name, active, &tracks, "/timing", d);
    Motion { name, which: forward.which, reverse: true, active, rest: rest_of(active), raw, tracks }
}

/// `MOVES_AFTER_REST` and `TIMELINE` for one motion.
pub fn check_timing(name: &str, active: f64, tracks: &[CTrack], timing_path: &str, d: &mut Diagnostics) {
    if active <= 0.0 {
        d.push(Diagnostic::error("NO_MOTION", timing_path, format!("{name} has no movement")).hint("key at least one track that changes"));
        return;
    }
    if active > MAX_ACTIVE + 1e-9 {
        d.push(Diagnostic::error("TIMELINE", timing_path, format!("{name} acts for {} s; at most {MAX_ACTIVE} s fits before rest", r4(active))));
    }
    for t in tracks {
        if let Some(k) = t.keys.iter().find(|k| k.t > active + 1e-9) {
            d.push(
                Diagnostic::error("MOVES_AFTER_REST", t.path.clone(), format!("{} has a key at {} s, after {name}'s action ends at {} s", t.target, r4(k.t), r4(active)))
                    .hint("end every track by the action's end, or lengthen timing"),
            );
        }
    }
}

/// Up to `cols` sheet columns for a motion, chosen as beats rather than even steps: 0 and
/// rest, then `hit` contacts, then turns (extremes) and moves starting from rest, then other
/// authored keys. Derived keys (contact crossings) are never chosen. Few beats are padded
/// with midpoints of the longest gaps.
pub fn beat_times(m: &Motion, cols: usize) -> Vec<f64> {
    let frame = 1.0 / 60.0;
    let rest = m.rest;
    let mut candidates: Vec<(u8, f64)> = Vec::new();
    if m.raw.is_empty() {
        candidates.extend(m.tracks.iter().flat_map(|t| t.keys.iter().map(|k| (1, k.t))));
    }
    for t in &m.raw {
        let keys = &t.keys;
        for (i, k) in keys.iter().enumerate() {
            let before = i.checked_sub(1).map(|j| k.v - keys[j].v).unwrap_or(0.0);
            let after = keys.get(i + 1).map(|n| n.v - k.v).unwrap_or(0.0);
            let score = if k.ease == Ease::Hit {
                3
            } else if before * after < 0.0 || (before == 0.0 && after != 0.0) {
                2
            } else {
                1
            };
            candidates.push((score, k.t));
        }
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.total_cmp(&b.1)));
    let mut times = vec![0.0];
    for (_, t) in candidates {
        if times.len() + 1 >= cols {
            break;
        }
        if t < rest - frame / 2.0 && times.iter().all(|x: &f64| (x - t).abs() >= frame) {
            times.push(t);
        }
    }
    times.push(rest);
    times.sort_by(f64::total_cmp);
    while times.len() < cols {
        let (i, _) = times.windows(2).enumerate().map(|(i, w)| (i, w[1] - w[0])).fold((0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
        let mid = (times[i] + times[i + 1]) / 2.0;
        times.insert(i + 1, mid);
    }
    times.into_iter().map(r4).collect()
}

pub fn r4(x: f64) -> f64 {
    (x * 1e4).round() / 1e4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse;

    fn k(t: f64, v: f64) -> Key {
        Key::new(t, v, Ease::Auto)
    }

    #[test]
    fn auto_reverse_reproduces_the_toggle() {
        // v1's authored toggle-switch Strong reverse, key for key.
        let forward = vec![k(0.0, 0.0), k(0.09, -0.7), k(0.22, 10.7), k(0.36, 10.0)];
        let back = reverse_auto(&forward, 0.36);
        let got: Vec<(f64, f64)> = back.iter().map(|k| (r4(k.t), r4(k.v))).collect();
        assert_eq!(got, [(0.0, 10.0), (0.09, 10.7), (0.22, -0.7), (0.36, 0.0)]);
    }

    #[test]
    fn auto_reverse_mirrors_beat_order() {
        // Dial turns (0.06–0.46), then the door swings (0.46–1.02): the reverse shuts first.
        let dial = vec![k(0.06, 0.0), k(0.46, 180.0)];
        let back = reverse_auto(&dial, 1.02);
        assert_eq!(back.iter().map(|k| (r4(k.t), k.v)).collect::<Vec<_>>(), [(0.56, 180.0), (0.96, 0.0)]);
    }

    #[test]
    fn presets_expand() {
        let text = include_str!("../tests/fixtures/in-room-safe.icon.json");
        let spec = parse(text).unwrap();
        let mut d = Diagnostics::default();
        let tracks = resolve(&spec, Which::Strong, &|_| 1.0, &mut d);
        assert!(d.0.is_empty(), "{d:?}");
        let door = &tracks[0];
        let got: Vec<(f64, f64)> = door.keys.iter().map(|k| (r4(k.t), r4(k.v))).collect();
        assert_eq!(got, [(0.6, 0.0), (0.78, 104.0), (0.8933, 98.0), (1.0067, 101.0), (1.12, 100.0)]);
    }

    #[test]
    fn bounce_matches_elevator() {
        let items = vec![
            (KeyItem::Key { t: 0.16, v: 0.0, ease: Ease::Auto }, String::new()),
            (KeyItem::Key { t: 0.4, v: 1.0, ease: Ease::Hit }, String::new()),
            (KeyItem::Bounce { t: 0.56, amp: 0.12 }, String::new()),
        ];
        let keys = expand(&items, 0.0, &mut Diagnostics::default()).unwrap();
        assert_eq!(keys.iter().map(|k| (r4(k.t), r4(k.v))).collect::<Vec<_>>(), [(0.16, 0.0), (0.4, 1.0), (0.48, 0.88), (0.56, 1.0)]);
    }

    #[test]
    fn follow_mirrors_a_turn() {
        let text = include_str!("../tests/fixtures/alarm-clock.icon.json");
        let spec = parse(text).unwrap();
        let mut d = Diagnostics::default();
        let tracks = resolve(&spec, Which::Light, &|_| 1.0, &mut d);
        let right = tracks.iter().find(|t| t.target.name == "bell-r").unwrap();
        assert_eq!(right.keys[1].v, 6.0);
    }

    #[test]
    fn rest_matches_v1() {
        assert_eq!(rest_of(0.45) * 60.0, 30.0);
        assert_eq!(rest_of(0.7) * 60.0, 45.0);
    }
}
