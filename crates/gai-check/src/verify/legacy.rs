//! Frozen checks tied to the frozen sets: the `verify-directed.mjs` fixed-part table and
//! per-icon block (03–11, 12–15; verify.md §4.2) and the 01/02 verifiers (§5). They are
//! literal transcriptions and are never extended; new sets use constraints and rules.

use super::generic::{
    ActionVariant, Generic, check_action, close, extent, increasing, pick_prefix, still,
    visibility, xy,
};
use super::js::{deep_equal, same_numbers, to_number};
use super::manifest::Record;
use super::{SetContext, VerifyError, ensure, read, read_json};
use crate::contracts::geom::Sampler;
use crate::contracts::jsnum::{at_index, hypot2, max_of, min_of, slice_range};
use crate::contracts::scene::{Scene, SceneNode, Track};
use serde::Deserialize;
use serde_json::{Value, json};

/// The hard-coded fixed parts of the directed sets (`verify-directed.mjs:129`). In 12 the
/// upload tray is fixed in both profiles; verify-directed exempts Strong, which recoils.
pub(super) fn fixed_parts(meta: Record<'_>, directed: bool) -> Vec<&'static str> {
    let strong = meta.str("motionId") == Some("strong");
    match meta.str("icon").unwrap_or("undefined") {
        "folder" => vec!["folder-back-frame"],
        "upload" if directed && strong => vec![],
        "upload" => vec!["upload-tray"],
        "copy" => vec!["copy-source"],
        "home" => vec!["home-shell", "home-doorway"],
        "calendar" => vec!["calendar-frame-frame", "calendar-header"],
        "clock" => vec!["clock-frame"],
        "camera" => vec!["camera-body", "camera-lens"],
        "image" => vec!["image-frame"],
        "message" => vec!["message-body"],
        "microphone" => vec!["microphone-stand", "microphone-foot"],
        "volume" => vec!["volume-speaker-frame"],
        "wifi" => vec!["wifi-source"],
        "sun" => vec!["sun-disc-frame"],
        "compass" => vec!["compass-frame"],
        "users" => vec!["users-front-shoulders-frame"],
        "battery" => vec!["battery-shell-frame", "battery-terminal"],
        "bluetooth" => vec!["bluetooth-upper", "bluetooth-lower"],
        "monitor" => vec!["monitor-screen-frame", "monitor-stand", "monitor-foot"],
        "smartphone" => vec!["smartphone-frame-frame", "smartphone-speaker"],
        "laptop" => vec!["laptop-base", "laptop-hinge"],
        "printer" => vec!["printer-housing"],
        "document" => vec!["document-page", "document-fold"],
        "clipboard" => vec!["clipboard-board"],
        "book" => vec!["book-left-page", "book-right-page", "book-spine"],
        "credit-card" => vec!["credit-card-reader"],
        "wallet" => vec!["wallet-body"],
        "gift" => vec!["gift-box", "gift-ribbon", "gift-solid-box"],
        "flag" => vec!["flag-pole"],
        "filter" => vec!["filter-tier-0"],
        _ => vec![],
    }
}

fn frames() -> impl Iterator<Item = f64> {
    (0..=144u32).map(|f| f64::from(f) / 60.0)
}

fn at(p: &[f64], i: i64) -> f64 {
    at_index(p.len(), i).map_or(f64::NAN, |k| p[k])
}

fn slice(p: &[f64], start: i64, end: i64) -> &[f64] {
    &p[slice_range(p.len(), start, Some(end))]
}

fn keys(node: &SceneNode) -> impl Iterator<Item = &[f64]> {
    node.points.iter().map(|(_, p)| p.as_slice())
}

/// The per-icon mechanics of `verify-directed.mjs:131-185` (and 12's copy without the
/// Strong upload recoil), keyed by the motion's public icon id.
pub(super) fn directed_block(
    set: &SetContext<'_>,
    sc: &Scene,
    meta: Record<'_>,
    directed: bool,
) -> Result<(), VerifyError> {
    let n = meta.str("icon").unwrap_or("undefined");
    let pick = |name: &str| pick_prefix(sc, name);
    let sample = |node: &SceneNode, t: f64| Sampler::Ge.coords(&node.points, t).into_owned();
    match n {
        "upload" if directed && meta.str("motionId") == Some("strong") => {
            let tray = pick("upload-tray")?;
            let base = &tray.points[0].1;
            for p in keys(tray) {
                let dy = at(p, 1) - at(base, 1);
                ensure((-0.12003..=0.45003).contains(&dy), || {
                    "Upload tray recoil range".into()
                })?;
                for i in (0..p.len()).step_by(2) {
                    close(p[i], at(base, i as i64), 3e-5, || "Tray fixed x".into())?;
                    close(
                        at(p, i as i64 + 1) - at(base, i as i64 + 1),
                        dy,
                        3e-5,
                        || "Tray rigid recoil".into(),
                    )?;
                }
            }
            ensure(
                same_numbers(&tray.points[tray.points.len() - 1].1, base),
                || "Tray returns after launch".into(),
            )
        }
        "scan" => increasing(keys(pick("scan-bar")?), |p| at(p, 1), 1.0, "scan"),
        "battery" => increasing(
            keys(pick("battery-charge")?),
            |p| extent(p, 0),
            1.0,
            "charge",
        ),
        "clock" => clock(set, sc, meta),
        "plug" => {
            for t in frames() {
                let a = sample(pick("plug-body-frame")?, t);
                let b = sample(pick("plug-cable")?, t);
                close(at(&b, 0), 16.0, 3e-5, || "Cable attachment x".into())?;
                close(at(&b, 1), max_of(xy(&a, 1)), 3e-5, || {
                    "Cable attachment y".into()
                })?;
                close(at(&b, -2), 16.0, 3e-5, || "Cable end x".into())?;
                close(at(&b, -1), 27.0, 3e-5, || "Cable end y".into())?;
            }
            Ok(())
        }
        "laptop" => {
            for p in keys(pick("laptop-lid-frame")?) {
                close(max_of(xy(p, 1)), 23.0, 3e-5, || "Laptop hinge".into())?;
            }
            Ok(())
        }
        "printer" => {
            for t in frames() {
                let a = sample(pick("printer-sheet")?, t);
                let b = sample(pick("printer-line")?, t);
                close(min_of(xy(&a, 1)), 18.0, 3e-5, || "Printer slot".into())?;
                close(max_of(xy(&a, 1)) - at(&b, 1), 2.7, 3e-5, || {
                    "Ink attached to page".into()
                })?;
            }
            Ok(())
        }
        "gift" => {
            for t in frames() {
                let a = sample(pick("gift-lid-frame")?, t);
                for side in ["left", "right"] {
                    let b = sample(pick(&format!("gift-bow-{side}"))?, t);
                    close(at(&b, 0), 16.0, 3e-5, || "Bow knot x".into())?;
                    close(at(&b, 1), min_of(xy(&a, 1)), 3e-5, || "Bow knot y".into())?;
                }
            }
            Ok(())
        }
        "book" => {
            let leaf = pick("book-turning-leaf")?;
            ensure(keys(leaf).any(|p| at(p, 6) < 16.0), || {
                "Leaf never crosses spine".into()
            })?;
            for p in keys(leaf) {
                close(at(p, 0), 16.0, 3e-5, || "Page spine x".into())?;
                close(at(p, 1), 8.0, 3e-5, || "Page spine y".into())?;
            }
            Ok(())
        }
        "chart" => {
            for i in 0..3 {
                let bar = pick(&format!("chart-bar-{i}-frame"))?;
                increasing(keys(bar), |p| extent(p, 1), 1.0, "Chart target")?;
                for p in keys(bar) {
                    close(max_of(xy(p, 1)), 26.0, 3e-5, || "Chart baseline".into())?;
                    close(extent(p, 0), 0.0, 3e-5, || "Single bar centerline".into())?;
                    close(at(p, 0), f64::from(8 + i * 8), 3e-5, || {
                        "Fixed bar column".into()
                    })?;
                }
            }
            Ok(())
        }
        "filter" => {
            for i in 1..3 {
                increasing(
                    keys(pick(&format!("filter-tier-{i}"))?),
                    |p| extent(p, 0),
                    -1.0,
                    "Filter order",
                )?;
            }
            Ok(())
        }
        "keyframes" => keyframes(sc),
        "image-upscale" => {
            for t in frames() {
                let frame = sample(pick("image-upscale-frame")?, t);
                let arrow = sample(pick("image-upscale-arrow")?, t);
                ensure(
                    min_of(xy(&arrow, 0)) - max_of(xy(&frame, 0)) >= 1.749,
                    || "Upscale arrow crosses growing corner in x".into(),
                )?;
                ensure(
                    min_of(xy(&frame, 1)) - max_of(xy(&arrow, 1)) >= 1.749,
                    || "Upscale arrow crosses growing corner in y".into(),
                )?;
            }
            Ok(())
        }
        "flag" => {
            for p in keys(pick("flag-cloth-frame")?) {
                close(at(p, 0), 8.0, 3e-5, || "Flag hoist x".into())?;
                close(at(p, 1), 6.0, 3e-5, || "Flag hoist y".into())?;
            }
            Ok(())
        }
        "smartphone" => {
            for node in sc.nodes.iter().filter(|n| n.name.contains("tile")) {
                for t in frames() {
                    if Sampler::Ge.scalar(visibility(node)?, t) > 0.0 {
                        let p = sample(node, t);
                        ensure(
                            min_of(xy(&p, 1)) >= 10.6 - 3e-5 && max_of(xy(&p, 1)) <= 24.31,
                            || "Phone clipping".into(),
                        )?;
                    }
                }
            }
            Ok(())
        }
        "image-caption" | "code-generate" if meta.str("direction") == Some("forward") => {
            let parts: [(&str, usize); 2] = if n == "image-caption" {
                [("l1", 7), ("l2", 6)]
            } else {
                [("l1", 6), ("l2", 5)]
            };
            for (part, count) in parts {
                let node = pick(&format!("{n}-{part}"))?;
                let widths = super::js::same_value_zero_set(
                    keys(node).map(|p| Some(json!(gai_core::js_round(extent(p, 0) * 1e4)))),
                );
                ensure(widths.len() == count, || {
                    format!("{n} typed character states {part}")
                })?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The keyframes playhead: fixed keys, per-weight rail envelopes and junctions.
fn keyframes(sc: &Scene) -> Result<(), VerifyError> {
    for part in [
        "keyframes-key-old-frame",
        "keyframes-key-new-frame",
        "keyframes-rail-left",
    ] {
        still(sc, part)?;
    }
    let is_band = |name: &str| {
        name.strip_prefix("keyframes-rail-stem-")
            .is_some_and(|rest| {
                ["light__", "regular__", "bold__", "solid__"]
                    .iter()
                    .any(|w| rest.starts_with(w))
            })
    };
    for band in sc.nodes.iter().filter(|n| is_band(&n.name)) {
        let w = if band.name.contains("-light__") {
            1.5
        } else if band.name.contains("-bold__") {
            2.5
        } else if band.name.contains("-solid__") {
            2.6
        } else {
            2.0
        };
        for t in frames() {
            let p = Sampler::Ge.coords(&band.points, t);
            let (mid, right, stem) = (slice(&p, 0, 36), slice(&p, 36, 72), slice(&p, 74, 110));
            close(extent(mid, 1), w, 3e-5, || {
                "Keyframes middle rail weight".into()
            })?;
            close(extent(right, 1), w, 3e-5, || {
                "Keyframes right rail weight".into()
            })?;
            close(extent(stem, 0), w, 3e-5, || "Keyframes stem weight".into())?;
            close(
                (min_of(xy(mid, 1)) + max_of(xy(mid, 1))) / 2.0,
                20.0,
                3e-5,
                || "Keyframes rail baseline".into(),
            )?;
            let tab = Sampler::Ge.coords(&pick_prefix(sc, "keyframes-tab")?.points, t);
            close(
                min_of(xy(stem, 1)) + w / 2.0,
                max_of(xy(&tab, 1)),
                3e-5,
                || "Playhead attachment".into(),
            )?;
            let key = pick_prefix(sc, "keyframes-key-new-frame")?;
            if Sampler::Ge.scalar(visibility(key)?, t) > 0.0 && w < 2.6 {
                ensure(max_of(xy(mid, 0)) <= 18.319, || {
                    "Rail enters new diamond".into()
                })?;
                ensure(min_of(xy(right, 0)) >= 24.681, || {
                    "Right rail enters new diamond".into()
                })?;
                ensure(max_of(xy(stem, 1)) <= 16.819, || {
                    "Stem enters new diamond".into()
                })?;
            }
        }
    }
    Ok(())
}

/// The scene fields only the clock check reads: each node's transform rotation track.
#[derive(Deserialize)]
struct RotationScene {
    nodes: Vec<RotationNode>,
}

#[derive(Deserialize)]
struct RotationNode {
    #[serde(default)]
    transform: Option<RotationTrack>,
}

#[derive(Deserialize)]
struct RotationTrack {
    rotation: Track<f64>,
}

/// The clock hands turn 12:1, the minute hand clockwise.
fn clock(set: &SetContext<'_>, sc: &Scene, meta: Record<'_>) -> Result<(), VerifyError> {
    let path = set.scene_path(meta.name()?);
    let rotations: RotationScene = serde_json::from_slice(&read(&path)?)
        .map_err(|source| VerifyError::Json { path, source })?;
    let rotation = |name: &str| -> Result<Option<&Track<f64>>, VerifyError> {
        let node = pick_prefix(sc, name)?;
        let index = sc
            .nodes
            .iter()
            .position(|n| std::ptr::eq(n, node))
            .unwrap_or_default();
        Ok(rotations
            .nodes
            .get(index)
            .and_then(|n| n.transform.as_ref())
            .map(|t| &t.rotation))
    };
    let (minute, hour) = (rotation("clock-minute")?, rotation("clock-hour")?);
    let (Some(minute), Some(hour)) = (minute, hour) else {
        return Err(VerifyError::Assertion(
            "clock hands have no transforms".into(),
        ));
    };
    if minute.is_empty() || hour.is_empty() {
        return Err(VerifyError::Assertion(
            "clock hand rotation has no keys".into(),
        ));
    }
    for t in frames() {
        close(
            Sampler::Ge.scalar(minute, t),
            Sampler::Ge.scalar(hour, t) * 12.0,
            3e-5,
            || "Clock 12:1 ratio".into(),
        )?;
    }
    let angles: Vec<[f64; 1]> = minute.iter().map(|(_, v)| [*v]).collect();
    increasing(
        angles.iter().map(|a| &a[..]),
        |p| p[0],
        -1.0,
        "clock direction",
    )
}

/// 01 Heart/Bell and 02 Search (verify.md §5): Phase A over `motions`, the mechanics of
/// each drawing, the research baselines and, for 01, the Light hash baseline.
pub(super) fn run_set(
    set: &SetContext<'_>,
    generic: Generic,
    variant: &ActionVariant,
) -> Result<(), VerifyError> {
    let icons = set.manifest.icons()?;
    // The set's icon list includes icons adopted from specs, which the adopted stage checks.
    let names: Vec<Value> = set.full_names.clone();
    let heart_bell = generic == Generic::HeartBell;
    let expected = if heart_bell {
        json!(["heart", "bell"])
    } else {
        json!(["search"])
    };
    ensure(deep_equal(&Value::Array(names), &expected), || {
        format!("set icons are not {expected}")
    })?;
    for motion in set.manifest.motions()? {
        check_action(set, variant, motion)?;
        let scene = set.scene(motion.name()?)?;
        match (heart_bell, motion.str("icon")) {
            (true, Some("heart")) => heart(&scene)?,
            (true, Some("bell")) => bell(&scene)?,
            (true, _) => {}
            (false, _) => search(&scene)?,
        }
    }
    for icon in &icons {
        research(set, *icon, heart_bell)?;
    }
    if heart_bell {
        let baseline = read_json(&set.paths.set_dir.join("evidence/light-baseline.json"))?;
        for name in ["heart", "bell"].into_iter().filter(|name| !set.is_adopted(name)) {
            let h = set.hash(&set.paths.assets_dir.join(format!("{name}.json")))?;
            let want = baseline.get(format!("prototype/icons/01/assets/{name}.json"));
            ensure(want.and_then(Value::as_str) == Some(h.as_str()), || {
                "Light native action changed".into()
            })?;
        }
    }
    Ok(())
}

fn starts_with<'s>(scene: &'s Scene, prefix: &str) -> Result<&'s SceneNode, VerifyError> {
    scene
        .nodes
        .iter()
        .find(|n| n.name.starts_with(prefix))
        .ok_or_else(|| VerifyError::Assertion(format!("{} has no {prefix} node", scene.name)))
}

fn heart(scene: &Scene) -> Result<(), VerifyError> {
    for p in keys(starts_with(scene, "heart-body")?) {
        ensure(same_numbers(slice(p, 0, 2), &[16.0, 26.5]), || {
            "Heart tip moves".into()
        })?;
    }
    Ok(())
}

fn bell(scene: &Scene) -> Result<(), VerifyError> {
    let body = starts_with(scene, "bell-body")?;
    let origin = &body.points[0].1;
    let pivot = |p: &[f64], i: usize| hypot2(at(p, i as i64) - 16.0, at(p, i as i64 + 1) - 5.6);
    for p in keys(body) {
        for i in (0..p.len()).step_by(2) {
            ensure((pivot(p, i) - pivot(origin, i)).abs() < 0.00003, || {
                "Bell shell deforms".into()
            })?;
        }
    }
    // A JSON.stringify comparison: -0 and 0 print alike.
    let mount = starts_with(scene, "bell-mount")?;
    let first = &mount.points[0].1;
    let same_text = |p: &[f64]| p.len() == first.len() && p.iter().zip(first).all(|(a, b)| a == b);
    ensure(keys(mount).all(same_text), || "Top mount moves".into())
}

fn search(scene: &Scene) -> Result<(), VerifyError> {
    let moving: Vec<&SceneNode> = scene
        .nodes
        .iter()
        .filter(|n| {
            n.name.starts_with("search-body-regular") || n.name.starts_with("search-handle-regular")
        })
        .collect();
    let first = moving
        .first()
        .ok_or_else(|| VerifyError::Assertion("search has no moving parts".into()))?;
    let at_key = |k: usize| -> Result<Vec<f64>, VerifyError> {
        let mut all = Vec::new();
        for n in &moving {
            let key = n
                .points
                .get(k)
                .ok_or_else(|| VerifyError::Assertion(format!("{} has fewer keys", n.name)))?;
            all.extend_from_slice(&key.1);
        }
        Ok(all)
    };
    let origin = at_key(0)?;
    let d = |p: &[f64], i: usize, j: usize| {
        hypot2(p[i] - p[j], at(p, i as i64 + 1) - at(p, j as i64 + 1))
    };
    for k in 0..first.points.len() {
        let current = at_key(k)?;
        for i in (0..current.len()).step_by(2) {
            for j in (i + 2..current.len()).step_by(2) {
                let o = if j < origin.len() {
                    d(&origin, i, j)
                } else {
                    f64::NAN
                };
                ensure((d(&current, i, j) - o).abs() < 0.00004, || {
                    "Search assembly bends or separates".into()
                })?;
            }
        }
    }
    Ok(())
}

/// The research-baseline checks of 01/02: drawings and topology unchanged against
/// `research-review/before`, shared rest shapes, and a substantially stronger Strong.
fn research(set: &SetContext<'_>, icon: Record<'_>, heart_bell: bool) -> Result<(), VerifyError> {
    let icon_name = super::js::to_js_string(icon.get("name"));
    let motions = icon.list("motions")?;
    let ids: Vec<Option<&str>> = motions.iter().map(|m| m.str("motionId")).collect();
    ensure(ids == [Some("light"), Some("strong")], || {
        format!("{icon_name}: motions are not light, strong")
    })?;
    let (light, strong) = (
        set.scene(motions[0].name()?)?,
        set.scene(motions[1].name()?)?,
    );
    let before = set.paths.set_dir.join(format!(
        "research-review/before/scenes/{icon_name}.scene.json"
    ));
    let previous = Scene::from_slice(&read(&before)?).map_err(|source| VerifyError::Json {
        path: before,
        source,
    })?;
    let short = || VerifyError::Assertion(format!("{icon_name}: a baseline scene has fewer nodes"));
    let rest_active = || to_number(motions[0].get("active")) <= 0.5;
    let compare = |a: &SceneNode, b: &SceneNode, drawing: &'static str, topology: &'static str| {
        ensure(same_numbers(&a.points[0].1, &b.points[0].1), || {
            drawing.into()
        })?;
        ensure(a.commands == b.commands, || topology.into())
    };
    let (rounded, changed) = (
        "Research revision changed the rounded drawing",
        "Research revision changed topology",
    );
    let (shapes, topology) = ("Variant rest shapes differ", "Variant topology differs");
    if heart_bell {
        for (i, n) in light.nodes.iter().enumerate() {
            compare(
                n,
                previous.nodes.get(i).ok_or_else(short)?,
                rounded,
                changed,
            )?;
        }
        if icon_name == "heart" {
            ensure(rest_active(), || {
                "Frequent feedback exceeds the chosen half-second budget".into()
            })?;
        }
        for (i, n) in light.nodes.iter().enumerate() {
            compare(n, strong.nodes.get(i).ok_or_else(short)?, shapes, topology)?;
        }
    } else {
        ensure(rest_active(), || {
            "Frequent feedback exceeds the chosen half-second budget".into()
        })?;
        for (i, n) in light.nodes.iter().enumerate() {
            compare(
                n,
                previous.nodes.get(i).ok_or_else(short)?,
                rounded,
                changed,
            )?;
            compare(n, strong.nodes.get(i).ok_or_else(short)?, shapes, topology)?;
        }
    }
    let body = format!("{icon_name}-body");
    let excursion = |scene: &Scene| -> Result<f64, VerifyError> {
        let n = starts_with(scene, &body)?;
        let origin = &n.points[0].1;
        Ok(max_of(keys(n).flat_map(|p| {
            p.iter()
                .enumerate()
                .map(|(i, x)| (x - origin.get(i).copied().unwrap_or(f64::NAN)).abs())
        })))
    };
    let (s, l) = (excursion(&strong)?, excursion(&light)?);
    ensure(s > l * 1.5, || {
        "Strong motion is not substantially stronger".into()
    })
}
