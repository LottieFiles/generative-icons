//! The generic checks every set gets (verify.md §3, §4): provenance, the format and paint
//! contract, packaging fidelity, per-node scene checks, Light/Strong and reverse pairing,
//! design constraints, native transforms and loop seams, and the directed report.
//!
//! Four frozen variants exist ([`Generic`]); new sets always get the template.

use super::geom::{Pick, pick};
use super::js::{
    self, deep_equal, deep_equal_opt, is_number, same_numbers, same_value, strict_equals,
};
use super::manifest::{Record, array};
use super::{SetContext, VerifyError, ensure, legacy, read, read_json};
use crate::contracts::geom::Sampler;
use crate::contracts::jsnum::{hypot2, max_of, max2, min_of, sum};
use crate::contracts::scene::{Scene, SceneNode, Track};
use crate::paint;
use gai_core::parallel::ordered_map;
use serde_json::{Map, Value, json};

/// The frozen generic-check variants (verify.md §3.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generic {
    /// 01 Heart/Bell: fixed legacy checks over `motions`, no evidence.
    HeartBell,
    /// 02 Search: fixed legacy checks over `motions`, no evidence.
    Search,
    /// `library/verify-directed.mjs` (03–11, 13–15): shared-builder hash, unconditional
    /// holds, the legacy fixed-part table and per-icon block, no loop seams.
    Directed,
    /// 12 loading indicators: continuous-builder hash, no holds, the legacy block without
    /// the upload recoil, loop seams on every action.
    Continuous,
    /// The template (16 onward and every new set).
    Template {
        /// Check the filled `more-options` ellipsis (25 onward) instead of skipping it (16–24).
        ellipsis: bool,
        /// The message of a stale `setBuilderSha256` (per-file text in 16–34).
        builder_message: &'static str,
    },
}

/// How Phase A hashes the builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Builder {
    /// Only `sourceSha256` (01/02).
    None,
    /// `setBuilderSha256` against `build.mjs`.
    Set(&'static str),
    /// `sharedBuilderSha256` against `library/build-set.mjs`, when recorded.
    Shared,
    /// `continuousBuilderSha256` against `build.mjs`.
    Continuous,
}

/// Which scene-node checks Phase A runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeChecks {
    /// Unconditional reset, topology and finiteness (01/02).
    Legacy,
    /// Guarded reset, rest hold and topology.
    Guarded(Hold),
}

/// When the "no motion after rest" checks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hold {
    /// Unless the action is continuous (template).
    UnlessContinuous,
    /// Always (verify-directed).
    Always,
    /// Never (12).
    Never,
}

/// The Phase A knobs of a variant.
#[derive(Debug, Clone, Copy)]
pub(super) struct ActionVariant {
    pub builder: Builder,
    pub definitions: bool,
    pub sources: bool,
    pub ellipsis: bool,
    pub every_stroke_width: bool,
    pub normalize_ks: bool,
    pub nodes: NodeChecks,
}

impl Generic {
    fn action_variant(self) -> ActionVariant {
        let (builder, ellipsis, hold) = match self {
            Generic::HeartBell | Generic::Search => {
                return ActionVariant {
                    builder: Builder::None,
                    definitions: false,
                    sources: false,
                    ellipsis: false,
                    every_stroke_width: true,
                    normalize_ks: false,
                    nodes: NodeChecks::Legacy,
                };
            }
            Generic::Directed => (Builder::Shared, false, Hold::Always),
            Generic::Continuous => (Builder::Continuous, false, Hold::Never),
            Generic::Template {
                ellipsis,
                builder_message,
            } => (
                Builder::Set(builder_message),
                ellipsis,
                Hold::UnlessContinuous,
            ),
        };
        ActionVariant {
            builder,
            definitions: true,
            sources: true,
            ellipsis,
            every_stroke_width: false,
            normalize_ks: true,
            nodes: NodeChecks::Guarded(hold),
        }
    }
}

/// The directed report (`evidence/directed-source-verification.json`) before extras.
#[derive(Debug, Clone)]
pub(crate) struct DirectedReport {
    /// `"FAIL"` only when a collect-mode stage marks it (50).
    pub failed: bool,
    /// `set.icons.length`.
    pub icons: usize,
    /// Number of actions.
    pub actions: usize,
    /// Scene nodes with a native rigid transform, checked against the packaged layer.
    pub native_transform_nodes: u64,
    /// Largest `|packaged world point - scene point|` over every transform.
    pub native_transform_max_coordinate_error: f64,
    /// One entry per forward motion.
    pub checks: Vec<Value>,
    /// Keys inserted before `loopChecks` (53 `drawingChecks`).
    pub before_loop_checks: Vec<(String, Value)>,
    /// Loop seams; `None` for verify-directed, which has no such key.
    pub loop_checks: Option<Vec<Value>>,
    /// Keys appended at the end (49 and 50 `clearance`, 54–63 `drawingChecks`).
    pub extra: Vec<(String, Value)>,
}

impl DirectedReport {
    /// The report object, keys in output order.
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert(
            "result".into(),
            Value::from(if self.failed { "FAIL" } else { "PASS" }),
        );
        m.insert("icons".into(), Value::from(self.icons));
        m.insert("actions".into(), Value::from(self.actions));
        m.insert(
            "nativeTransformNodes".into(),
            Value::from(self.native_transform_nodes),
        );
        m.insert(
            "nativeTransformMaxCoordinateError".into(),
            Value::from(self.native_transform_max_coordinate_error),
        );
        m.insert("checks".into(), Value::Array(self.checks.clone()));
        m.extend(self.before_loop_checks.iter().cloned());
        if let Some(loops) = &self.loop_checks {
            m.insert("loopChecks".into(), Value::Array(loops.clone()));
        }
        m.extend(self.extra.iter().cloned());
        Value::Object(m)
    }

    /// The PASS line the scripts print.
    pub fn summary(&self, set_id: &str) -> String {
        format!(
            "PASS {set_id}: {} icons, {} native actions; provenance, styles, weights, topology, state boundaries, holds and physical constraints.",
            self.icons, self.actions
        )
    }
}

/// Runs the generic checks. `None` for the 01/02 variants, which write no report.
pub(super) fn run(
    set: &SetContext<'_>,
    generic: &Generic,
) -> Result<Option<DirectedReport>, VerifyError> {
    match generic {
        Generic::HeartBell | Generic::Search => {
            legacy::run_set(set, *generic, &generic.action_variant()).map(|()| None)
        }
        _ => directed(set, *generic).map(Some),
    }
}

fn directed(set: &SetContext<'_>, generic: Generic) -> Result<DirectedReport, VerifyError> {
    let variant = generic.action_variant();
    let manifest = &set.manifest;
    let icons = manifest.icons()?;

    // Phase A: every action.
    let actions = manifest.animations()?;
    ordered_map(actions.len(), |i| check_action(set, &variant, actions[i]))?;

    // Phase B: Light/Strong pairing and reverses.
    for icon in &icons {
        pair_icon(set, generic, *icon)?;
    }

    // Phase C: every forward motion.
    let motions = manifest.motions()?;
    let results = ordered_map(motions.len(), |i| check_motion(set, generic, motions[i]))?;
    let mut report = DirectedReport {
        failed: false,
        icons: icons.len(),
        actions: icons
            .iter()
            .map(|i| i.list("animations").map_or(0, |a| a.len()))
            .sum(),
        native_transform_nodes: 0,
        native_transform_max_coordinate_error: 0.0,
        checks: Vec::with_capacity(results.len()),
        before_loop_checks: Vec::new(),
        loop_checks: None,
        extra: Vec::new(),
    };
    for r in results {
        report.native_transform_nodes += r.transform_nodes;
        report.native_transform_max_coordinate_error =
            max2(report.native_transform_max_coordinate_error, r.max_error);
        report.checks.push(r.entry);
    }

    // Phase D: loop seams.
    let looped: Vec<Record<'_>> = match generic {
        Generic::Directed => Vec::new(),
        Generic::Continuous => actions.clone(),
        _ => {
            let mut v = Vec::new();
            for icon in icons.iter().filter(|i| i.flag("continuous")) {
                v.extend(icon.list("animations")?);
            }
            v
        }
    };
    if generic != Generic::Directed {
        report.loop_checks = Some(ordered_map(looped.len(), |i| loop_seams(set, looped[i]))?);
    }
    Ok(report)
}

/// `assert.equal(hash, expected, message)` for a sha256.
fn expect_hash(
    actual: &str,
    expected: Option<&Value>,
    what: impl FnOnce() -> String,
) -> Result<(), VerifyError> {
    ensure(expected.and_then(Value::as_str) == Some(actual), || {
        format!(
            "{}: sha256 {actual} != {}",
            what(),
            js::to_js_string(expected)
        )
    })
}

/// A property read that throws like JS on `undefined` or `null`.
pub(super) fn prop<'v>(v: Option<&'v Value>, key: &str) -> Result<Option<&'v Value>, VerifyError> {
    match v {
        None | Some(Value::Null) => Err(VerifyError::Assertion(format!(
            "Cannot read properties of undefined (reading '{key}')"
        ))),
        Some(v) => Ok(v.get(key)),
    }
}

/// Phase A for one action (verify.md §3.1–3.7).
pub(super) fn check_action(
    set: &SetContext<'_>,
    v: &ActionVariant,
    a: Record<'_>,
) -> Result<(), VerifyError> {
    let name = a.name()?;
    let (dir, out) = (&set.paths.set_dir, &set.paths.assets_dir);

    // Provenance.
    let proof = read_json(&dir.join(format!("native/{name}.json.provenance.json")))?;
    let native_path = dir.join(format!("native/{name}.json"));
    let native = read(&native_path)?;
    expect_hash(&gai_core::sha256_hex(&native), proof.get("sha256"), || {
        format!("native/{name}.json")
    })?;
    let engine = read(&dir.join(format!("scenes/{name}.engine.js")))?;
    expect_hash(
        &gai_core::sha256_hex(engine),
        proof.get("scriptSha256"),
        || format!("scenes/{name}.engine.js"),
    )?;
    let scene_bytes = read(&set.scene_path(name))?;
    expect_hash(
        &gai_core::sha256_hex(&scene_bytes),
        proof.get("sceneSha256"),
        || format!("scenes/{name}.scene.json"),
    )?;
    let build = set.builder_hash(&dir.join("build.mjs"))?;
    expect_hash(&build, a.get("sourceSha256"), || {
        format!("{name} sourceSha256")
    })?;
    if v.definitions {
        let definitions = set.hash(&dir.join("definitions.mjs"))?;
        expect_hash(&definitions, a.get("definitionsSha256"), || {
            format!("{name} definitionsSha256")
        })?;
    }
    match v.builder {
        Builder::None => {}
        Builder::Set(message) => {
            expect_hash(&build, a.get("setBuilderSha256"), || message.to_string())?
        }
        Builder::Continuous => expect_hash(&build, a.get("continuousBuilderSha256"), || {
            "Stale continuous builder".into()
        })?,
        Builder::Shared if a.flag("sharedBuilderSha256") => {
            let shared = set.builder_hash(&set.paths.library_dir.join("build-set.mjs"))?;
            expect_hash(&shared, a.get("sharedBuilderSha256"), || {
                "Stale shared builder".into()
            })?;
        }
        Builder::Shared => {}
    }
    if v.sources {
        for (rel, expected) in entries(a.get("authoringSources")) {
            expect_hash(&set.hash(&dir.join(rel))?, Some(expected), || {
                format!("Stale authoring source {rel}")
            })?;
        }
    }
    expect_hash(
        &set.hash(&out.join(format!("{name}.lottie")))?,
        a.get("sha256"),
        || format!("{name}.lottie"),
    )?;

    let raw: Value = serde_json::from_slice(&native).map_err(|source| VerifyError::Json {
        path: native_path,
        source,
    })?;
    let data = read_json(&out.join(format!("{name}.json")))?;
    check_format(&data, a)?;
    check_paint(&data, a, v)?;
    check_themes(set, &data, name)?;
    check_packaging(&data, &raw, v.normalize_ks)?;
    let rest = a.get("rest");
    let markers = array(data.get("markers"), "data.markers")?;
    ensure(
        markers
            .iter()
            .any(|m| m.get("cm") == Some(&json!("rest")) && strict_equals(m.get("tm"), rest)),
        || format!("{name}: no rest marker at {}", js::to_js_string(rest)),
    )?;
    ensure(
        markers
            .iter()
            .any(|m| m.get("cm") == Some(&json!("action"))),
        || format!("{name}: no action marker"),
    )?;

    let scene = set.cache_scene(name, &scene_bytes)?;
    for node in &scene.nodes {
        check_node(node, a, v.nodes)?;
    }
    if v.sources {
        for (file, expected) in entries(a.get("directionSources")) {
            let h = set.hash(&set.paths.library_dir.join(file))?;
            expect_hash(&h, Some(expected), || {
                format!("Stale direction source {file}")
            })?;
        }
    }
    Ok(())
}

/// `Object.entries(v || {})` for the source-hash maps.
fn entries(v: Option<&Value>) -> impl Iterator<Item = (&String, &Value)> {
    v.and_then(Value::as_object).into_iter().flatten()
}

/// §3.2: canvas, frame rate, shape layers, styles and presets.
pub(super) fn check_format(data: &Value, a: Record<'_>) -> Result<(), VerifyError> {
    let name = a.name()?;
    for (key, want) in [("w", 32.0), ("h", 32.0), ("fr", 60.0), ("op", 144.0)] {
        ensure(is_number(data.get(key), want), || {
            format!("{name}: data.{key} is not {want}")
        })?;
    }
    let layers = array(data.get("layers"), "data.layers")?;
    ensure(
        layers
            .iter()
            .all(|l| strict_equals(l.get("ty"), Some(&json!(4)))),
        || format!("{name}: non-shape layer"),
    )?;
    ensure(
        deep_equal_opt(
            a.get("styles"),
            Some(&json!(["outline", "solid", "duotone"])),
        ),
        || format!("{name}: styles"),
    )?;
    let presets = prop(a.get("strokeWidth"), "presets")?;
    ensure(deep_equal_opt(presets, Some(&json!([1.5, 2, 2.5]))), || {
        format!("{name}: stroke width presets")
    })
}

/// §3.3: every top-level stroke and fill is slot-bound; outline strokes use the three weights.
pub(super) fn check_paint(data: &Value, a: Record<'_>, v: &ActionVariant) -> Result<(), VerifyError> {
    let name = a.name()?;
    let slots = data.get("slots");
    let bound = |sid: Option<&Value>| -> Result<bool, VerifyError> {
        Ok(js::truthy(prop(slots, &js::to_js_string(sid))?))
    };
    let mut widths = Vec::new();
    for layer in array(data.get("layers"), "data.layers")? {
        for paint in array(layer.get("shapes"), "layer.shapes")? {
            let ty = paint.get("ty").and_then(Value::as_str);
            if !matches!(ty, Some("st" | "fl")) {
                continue;
            }
            ensure(bound(prop(paint.get("c"), "sid")?)?, || {
                "Unbound color".into()
            })?;
            let style = prop(paint.get("o"), "sid")?;
            ensure(bound(style)?, || "Unbound style".into())?;
            if ty == Some("st") {
                let counted = v.every_stroke_width
                    || style
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            VerifyError::Assertion(
                                "paint.o.sid.startsWith is not a function".into(),
                            )
                        })?
                        .starts_with("style.outline");
                if counted {
                    widths.push(prop(paint.get("w"), "k")?.cloned());
                }
                ensure(is_number(paint.get("lc"), 2.0), || {
                    format!("{name}: stroke cap")
                })?;
                ensure(is_number(paint.get("lj"), 2.0), || {
                    format!("{name}: stroke join")
                })?;
            }
        }
    }
    let widths = js::same_value_zero_set(widths);
    let filled = a.str("icon").is_some_and(|icon| {
        paint::PaintContract::builtin()
            .filled_ellipsis
            .iter()
            .any(|i| i == icon)
    });
    if v.ellipsis && filled {
        ensure(widths.is_empty(), || {
            format!("{name}: ellipsis has outline strokes")
        })?;
        return paint::verify_filled_ellipsis(data, name)
            .map_err(|e| VerifyError::Assertion(e.to_string()));
    }
    let mut sorted = widths;
    js::default_sort(&mut sorted);
    let want = [1.5, 2.0, 2.5].map(|w| Some(json!(w)));
    ensure(
        sorted.len() == 3
            && sorted
                .iter()
                .zip(&want)
                .all(|(a, b)| deep_equal_opt(a.as_ref(), b.as_ref())),
        || format!("{name}: outline stroke widths"),
    )
}

/// §3.4: each theme defines exactly the slot registry.
pub(super) fn check_themes(set: &SetContext<'_>, data: &Value, name: &str) -> Result<(), VerifyError> {
    let slots = data
        .get("slots")
        .ok_or_else(|| VerifyError::Assertion("Cannot convert undefined to object".into()))?;
    let mut keys: Vec<Option<Value>> = slots
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, _)| Some(json!(k)))
        .collect();
    js::default_sort(&mut keys);
    for theme in ["dark", "light"] {
        let file = read_json(
            &set.paths
                .assets_dir
                .join(format!("{name}.theme-{theme}.json")),
        )?;
        let mut ids = Vec::new();
        for rule in array(file.get("rules"), "theme.rules")? {
            ids.push(prop(Some(rule), "id")?.cloned());
        }
        js::default_sort(&mut ids);
        let same = ids.len() == keys.len()
            && ids
                .iter()
                .zip(&keys)
                .all(|(a, b)| deep_equal_opt(a.as_ref(), b.as_ref()));
        ensure(same, || "Theme drops token registry".into())?;
    }
    Ok(())
}

/// §3.5: packaging kept every layer transform (up to `-0`) and every path shape.
fn check_packaging(data: &Value, raw: &Value, normalize_ks: bool) -> Result<(), VerifyError> {
    let (layers, raw_layers) = (
        array(data.get("layers"), "data.layers")?,
        array(raw.get("layers"), "raw.layers")?,
    );
    let ks_same = layers.len() == raw_layers.len()
        && layers.iter().zip(raw_layers).all(|(l, r)| {
            match (l.get("ks"), r.get("ks")) {
                (Some(ks), Some(raw_ks)) if normalize_ks => js::deep_equal_round_trip(ks, raw_ks),
                // JSON.stringify writes an undefined array item as null.
                (ks, None) if normalize_ks => ks == Some(&Value::Null),
                (ks, raw_ks) => deep_equal_opt(ks, raw_ks),
            }
        });
    ensure(ks_same, || "Packaging changed motion".into())?;
    let mut same = layers.len() == raw_layers.len();
    for (l, r) in layers.iter().zip(raw_layers) {
        let (a, b) = (path_shapes(l)?, path_shapes(r)?);
        same = same && a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| deep_equal(x, y));
    }
    ensure(same, || "Packaging changed path animation".into())
}

/// `layer.shapes.filter(s => s.ty === "sh")`.
fn path_shapes(layer: &Value) -> Result<Vec<&Value>, VerifyError> {
    Ok(array(layer.get("shapes"), "layer.shapes")?
        .iter()
        .filter(|s| s.get("ty") == Some(&json!("sh")))
        .collect())
}

/// A node's visibility track, which the checks read as `node.visibility[…]`.
pub(super) fn visibility(node: &SceneNode) -> Result<&Track<f64>, VerifyError> {
    node.visibility
        .as_ref()
        .ok_or_else(|| VerifyError::Assertion(format!("{} has no visibility track", node.name)))
}

fn first_last<T>(track: &Track<T>) -> (&T, &T) {
    (&track[0].1, &track[track.len() - 1].1)
}

/// What to do about a part that moves after rest.
const HOLD_HINT: &str = "hint: every visible part must hold its final pose from rest to the end; \
                         in reverse and reset actions, poses get r = true";

/// §3.7: reset, rest hold, topology and finiteness of one scene node. Failures name the
/// part and the time (the legacy scripts named only the action).
fn check_node(node: &SceneNode, a: Record<'_>, checks: NodeChecks) -> Result<(), VerifyError> {
    let name = a.name()?;
    let part = node.base();
    let points = &node.points;
    let (first, last) = first_last(points);
    match checks {
        NodeChecks::Legacy => ensure(same_numbers(first, last), || {
            format!("Reset changes geometry: {part} ends away from its first pose")
        })?,
        NodeChecks::Guarded(hold) => {
            if !a.flag("stateful") && !a.flag("cycleDegrees") {
                let (v0, v1) = first_last(visibility(node)?);
                if *v0 > 0.0 && *v1 > 0.0 {
                    ensure(same_numbers(first, last), || {
                        format!(
                            "{name}: reset changes visible geometry: {part} ends away from its first pose"
                        )
                    })?;
                }
            }
            let held = match hold {
                Hold::UnlessContinuous => !a.flag("continuous"),
                Hold::Always => true,
                Hold::Never => false,
            };
            if held {
                let rest = a.number("rest") / 60.0;
                let vis = visibility(node)?;
                let vis_last = *first_last(vis).1;
                if vis_last > 0.0 {
                    for (t, p) in points {
                        if *t >= rest {
                            ensure(same_numbers(p, last), || {
                                format!(
                                    "{name}: moving after rest: {part} moves at {t:.3} s, \
                                     after rest at {rest:.3} s; {HOLD_HINT}"
                                )
                            })?;
                        }
                    }
                }
                for (t, v) in vis {
                    if *t >= rest {
                        ensure(same_value(*v, vis_last), || {
                            let v = gai_core::js_number_to_string(*v);
                            let end = gai_core::js_number_to_string(vis_last);
                            format!(
                                "{name}: visibility moving after rest: {part} opacity {v} at \
                                 {t:.3} s, {end} at the end, after rest at {rest:.3} s; \
                                 {HOLD_HINT}"
                            )
                        })?;
                    }
                }
            }
        }
    }
    if let Some((t, p)) = points.iter().find(|(_, p)| p.len() != first.len()) {
        return Err(VerifyError::Assertion(format!(
            "{name}: Morph topology changed: {part} has {} coordinates at {t:.3} s, {} at the start",
            p.len(),
            first.len()
        )));
    }
    match points.iter().find(|(_, p)| !p.iter().all(|c| c.is_finite())) {
        Some((t, _)) => Err(VerifyError::Assertion(format!(
            "{name}: non-finite coordinate: {part} at {t:.3} s"
        ))),
        None => Ok(()),
    }
}

/// Phase B for one icon (§3.8, §3.9).
fn pair_icon(set: &SetContext<'_>, generic: Generic, icon: Record<'_>) -> Result<(), VerifyError> {
    let icon_name = js::to_js_string(icon.get("name"));
    let motions = icon.list("motions")?;
    let ids: Vec<Option<&Value>> = motions.iter().map(|m| m.get("motionId")).collect();
    ensure(
        ids.len() == 2 && ids[0] == Some(&json!("light")) && ids[1] == Some(&json!("strong")),
        || format!("{icon_name}: motions are not light, strong"),
    )?;
    let paired = match generic {
        Generic::Directed => !icon.flag("cycleDegrees"),
        _ => !icon.flag("continuous") && !icon.flag("cycleDegrees"),
    };
    let (light, strong) = (
        set.scene(motions[0].name()?)?,
        set.scene(motions[1].name()?)?,
    );
    if paired {
        pair_light_strong(&icon_name, &light, &strong)?;
    }
    for motion in &motions {
        let forward = set.scene(motion.name()?)?;
        if !icon.flag("stateful") {
            continue;
        }
        let reverse = Record(
            motion
                .get("reverse")
                .filter(|r| js::truthy(Some(r)))
                .ok_or_else(|| VerifyError::Assertion("Missing explicit reverse".into()))?,
        );
        let reverse = set.scene(reverse.name()?)?;
        pair_reverse(&icon_name, &forward, &reverse)?;
    }
    Ok(())
}

/// §3.8: Light and Strong share their visible start and end, node by node (positionally).
fn pair_light_strong(icon_name: &str, light: &Scene, strong: &Scene) -> Result<(), VerifyError> {
    for (i, a) in light.nodes.iter().enumerate() {
        let b = strong.nodes.get(i).ok_or_else(|| {
            VerifyError::Assertion(format!("{icon_name}: Strong has fewer nodes"))
        })?;
        let part = a.base();
        let differs = |what: &str| {
            format!(
                "{icon_name} {what}: {part} differs between Light and Strong; hint: both \
                 motions must start and end on the same pose"
            )
        };
        let ((a0, a1), (b0, b1)) = (first_last(visibility(a)?), first_last(visibility(b)?));
        ensure(same_value(*a0, *b0), || differs("initial visibility"))?;
        ensure(same_value(*a1, *b1), || differs("final visibility"))?;
        let ((p0, p1), (q0, q1)) = (first_last(&a.points), first_last(&b.points));
        if *a0 > 0.0 {
            ensure(same_numbers(p0, q0), || differs("initial drawing"))?;
        }
        if *a1 > 0.0 {
            ensure(same_numbers(p1, q1), || differs("final drawing"))?;
        }
    }
    Ok(())
}

/// §3.9: a reverse action starts where its forward action ends and ends where it starts.
fn pair_reverse(icon_name: &str, forward: &Scene, reverse: &Scene) -> Result<(), VerifyError> {
    for (i, f) in forward.nodes.iter().enumerate() {
        let r = reverse.nodes.get(i).ok_or_else(|| {
            VerifyError::Assertion(format!("{icon_name}: reverse has fewer nodes"))
        })?;
        let part = f.base();
        let (fv, rv) = (first_last(visibility(f)?), first_last(visibility(r)?));
        let (fp, rp) = (first_last(&f.points), first_last(&r.points));
        if *fv.0 > 0.0 {
            ensure(same_numbers(fp.0, rp.1), || {
                format!(
                    "{icon_name} reverse final geometry: {part} in {} does not end where {} starts",
                    reverse.name, forward.name
                )
            })?;
        }
        if *fv.1 > 0.0 {
            ensure(same_numbers(fp.1, rp.0), || {
                format!(
                    "{icon_name} reverse initial geometry: {part} in {} does not start where {} ends",
                    reverse.name, forward.name
                )
            })?;
        }
        ensure(same_value(*fv.0, *rv.1), || {
            format!(
                "Reverse visibility mismatch: {part} in {} does not end as visible as {} starts",
                reverse.name, forward.name
            )
        })?;
    }
    Ok(())
}

/// What Phase C contributes per motion.
struct MotionResult {
    entry: Value,
    transform_nodes: u64,
    max_error: f64,
}

/// `close(a, b, label, tol)`: `|a - b| < tol`.
pub(super) fn close(
    a: f64,
    b: f64,
    tol: f64,
    label: impl FnOnce() -> String,
) -> Result<(), VerifyError> {
    ensure((a - b).abs() < tol, || {
        format!(
            "{}: {} != {}",
            label(),
            gai_core::js_number_to_string(a),
            gai_core::js_number_to_string(b)
        )
    })
}

/// `pick(scene, name)` of the template: `PICK_PREFIX`, failing with `Missing <scene>/<name>`.
pub(super) fn pick_prefix<'s>(scene: &'s Scene, name: &str) -> Result<&'s SceneNode, VerifyError> {
    pick(scene, Pick::Prefix, name, &[])
        .ok_or_else(|| VerifyError::Assertion(format!("Missing {}/{name}", scene.name)))
}

/// `still(scene, part)`: every key equals the first.
pub(super) fn still(scene: &Scene, part: &str) -> Result<(), VerifyError> {
    let n = pick_prefix(scene, part)?;
    let first = &n.points[0].1;
    ensure(n.points.iter().all(|(_, p)| same_numbers(p, first)), || {
        format!("{} fixed {part}", scene.name)
    })
}

/// `xy(p, axis)`: every other coordinate.
pub(super) fn xy(p: &[f64], axis: usize) -> impl Iterator<Item = f64> + Clone + '_ {
    p.iter().skip(axis).step_by(2).copied()
}

/// `extent(p, axis)`: `Math.max(...xy) - Math.min(...xy)`.
pub(super) fn extent(p: &[f64], axis: usize) -> f64 {
    max_of(xy(p, axis)) - min_of(xy(p, axis))
}

/// `increasing(keys, measure, label, sign)`: the signed measure never drops by 3e-5.
pub(super) fn increasing<'k>(
    keys: impl IntoIterator<Item = &'k [f64]>,
    measure: impl Fn(&[f64]) -> f64,
    sign: f64,
    label: &str,
) -> Result<(), VerifyError> {
    let mut previous = f64::NEG_INFINITY;
    for p in keys {
        let v = sign * measure(p);
        ensure(v >= previous - 3e-5, || format!("{label} runs backward"))?;
        previous = v;
    }
    Ok(())
}

/// Phase C for one forward motion (§3.10–3.12, §4.2).
fn check_motion(
    set: &SetContext<'_>,
    generic: Generic,
    meta: Record<'_>,
) -> Result<MotionResult, VerifyError> {
    let name = meta.name()?;
    let scene = set.scene(name)?;
    let constraints: Vec<Value> = match meta.get("constraints") {
        c if !js::truthy(c) => Vec::new(),
        Some(Value::Array(c)) => c.clone(),
        _ => {
            return Err(VerifyError::Assertion(format!(
                "{name}: constraints are not an array"
            )));
        }
    };
    let fixed: Vec<Value> = match generic {
        Generic::Directed | Generic::Continuous => {
            let parts = legacy::fixed_parts(meta, generic == Generic::Directed);
            for part in &parts {
                still(&scene, part)?;
            }
            legacy::directed_block(set, &scene, meta, generic == Generic::Directed)?;
            parts.into_iter().map(Value::from).collect()
        }
        _ => constraints
            .iter()
            .filter(|r| r.get("type") == Some(&json!("fixed")))
            .map(|r| r.get("part").cloned().unwrap_or(Value::Null))
            .collect(),
    };
    for rule in &constraints {
        check_constraint(&scene, meta, rule)?;
    }
    let data = read_json(&set.paths.assets_dir.join(format!("{name}.json")))?;
    let (transform_nodes, max_error) = native_transforms(&scene, &data)?;
    let mut entry = Map::new();
    entry.insert("name".into(), Value::from(name));
    entry.insert("fixedParts".into(), Value::Array(fixed));
    entry.insert("topology".into(), Value::Bool(true));
    entry.insert("stateBoundaries".into(), Value::Bool(true));
    match generic {
        Generic::Directed => {
            entry.insert("hold".into(), Value::Bool(true));
        }
        Generic::Continuous => {
            entry.insert("continuous".into(), Value::Bool(true));
        }
        _ => {
            entry.insert("hold".into(), Value::Bool(!meta.flag("continuous")));
            entry.insert("continuous".into(), Value::Bool(meta.flag("continuous")));
        }
    }
    entry.insert("designConstraints".into(), Value::Array(constraints));
    Ok(MotionResult {
        entry: Value::Object(entry),
        transform_nodes,
        max_error,
    })
}

/// §3.10: one design constraint recorded in `set.json`.
fn check_constraint(scene: &Scene, meta: Record<'_>, rule: &Value) -> Result<(), VerifyError> {
    let profile = rule.get("profile");
    if js::truthy(profile) && !strict_equals(profile, meta.get("motionId")) {
        return Ok(());
    }
    let part = js::to_js_string(rule.get("part"));
    let node = pick_prefix(scene, &part)?;
    let kind = rule.get("type").and_then(Value::as_str);
    let label = format!(
        "{}: {} {part}",
        meta.name()?,
        js::to_js_string(rule.get("type"))
    );
    let num = |key: &str| js::to_number(rule.get(key));
    match kind {
        Some("fixed") => still(scene, &part),
        Some("opaque") => {
            for (_, v) in visibility(node)? {
                ensure(same_value(*v, 1.0), || format!("{label} fades"))?;
            }
            Ok(())
        }
        Some("hidden") => {
            let (from, to) = (num("from"), num("to"));
            for f in 0..=144u32 {
                let t = f64::from(f) / 60.0;
                if t >= from && t <= to {
                    let v = Sampler::Ge.scalar(visibility(node)?, t);
                    ensure(same_value(v, 0.0), || {
                        format!("{label} leaves a visible cap")
                    })?;
                }
            }
            Ok(())
        }
        Some("rigid") => {
            let at = |p: &[f64], i: usize| p.get(i).copied().unwrap_or(f64::NAN);
            let base = &node.points[0].1;
            let distances: Vec<f64> = (2..base.len())
                .step_by(2)
                .map(|i| hypot2(base[i] - base[0], at(base, i + 1) - base[1]))
                .collect();
            for (_, p) in &node.points {
                for i in (2..p.len()).step_by(2) {
                    let d = hypot2(p[i] - p[0], at(p, i + 1) - p[1]);
                    close(
                        d,
                        distances.get(i / 2 - 1).copied().unwrap_or(f64::NAN),
                        5e-5,
                        || label.clone(),
                    )?;
                }
            }
            Ok(())
        }
        Some("monotonic") => {
            let sign = if js::truthy(rule.get("direction")) {
                num("direction")
            } else {
                1.0
            };
            let measure = rule.get("measure").and_then(Value::as_str);
            if measure == Some("alpha") {
                let values: Vec<[f64; 1]> = visibility(node)?.iter().map(|(_, v)| [*v]).collect();
                return increasing(values.iter().map(|v| &v[..]), |p| p[0], sign, &label);
            }
            let keys = node.points.iter().map(|(_, p)| p.as_slice());
            let mean = |p: &[f64], axis: usize| sum(xy(p, axis)) / (p.len() as f64 / 2.0);
            match measure {
                Some("width") => increasing(keys, |p| extent(p, 0), sign, &label),
                Some("height") => increasing(keys, |p| extent(p, 1), sign, &label),
                Some("x") => increasing(keys, |p| mean(p, 0), sign, &label),
                Some("y") => increasing(keys, |p| mean(p, 1), sign, &label),
                _ => Err(VerifyError::Assertion(format!("{label} unknown measure"))),
            }
        }
        Some("inside") => {
            let b = rule.get("box");
            let edge = |i: usize| js::to_number(b.and_then(|b| b.get(i)));
            let (left, top, right, bottom) = (edge(0), edge(1), edge(2), edge(3));
            for f in 0..=144u32 {
                let t = f64::from(f) / 60.0;
                if Sampler::Ge.scalar(visibility(node)?, t) > 0.0 {
                    let p = Sampler::Ge.coords(&node.points, t);
                    let ok = min_of(xy(&p, 0)) >= left - 3e-5
                        && max_of(xy(&p, 0)) <= right + 3e-5
                        && min_of(xy(&p, 1)) >= top - 3e-5
                        && max_of(xy(&p, 1)) <= bottom + 3e-5;
                    ensure(ok, || format!("{label} crosses its content region"))?;
                }
            }
            Ok(())
        }
        _ => Err(VerifyError::Assertion(format!(
            "Unknown design constraint {}",
            js::to_js_string(rule.get("type"))
        ))),
    }
}

/// A packaged layer property at integer frame `f`: keyframes sampled with S_GE, or the
/// static value. Scalars read as one-element arrays.
fn property(p: Option<&Value>, f: f64) -> Result<Vec<f64>, VerifyError> {
    let numbers = |v: Option<&Value>| match v {
        Some(Value::Array(items)) => items.iter().map(|x| js::to_number(Some(x))).collect(),
        other => vec![js::to_number(other)],
    };
    if js::truthy(prop(p, "a")?) {
        let keys: Track<Vec<f64>> = array(prop(p, "k")?, "k")?
            .iter()
            .map(|k| {
                (
                    js::to_number(k.get("t")),
                    k.get("s").map_or_else(Vec::new, |s| numbers(Some(s))),
                )
            })
            .collect();
        if keys.is_empty() {
            return Err(VerifyError::Assertion(
                "keyframed property without keys".into(),
            ));
        }
        Ok(Sampler::Ge.coords(&keys, f).into_owned())
    } else {
        Ok(numbers(prop(p, "k")?))
    }
}

/// §3.11: every rigid scene transform matches the packaged layer's `ks` in world space.
fn native_transforms(scene: &Scene, data: &Value) -> Result<(u64, f64), VerifyError> {
    let layers = array(data.get("layers"), "data.layers")?;
    let (mut count, mut max_error) = (0u64, 0.0f64);
    for node in scene.nodes.iter().filter(|n| n.transform) {
        let layer = layers
            .iter()
            .find(|l| l.get("nm").and_then(Value::as_str) == Some(node.base()));
        count += 1;
        let layer = layer
            .ok_or_else(|| VerifyError::Assertion(format!("No packaged layer {}", node.base())))?;
        let resampled = array(layer.get("shapes"), "layer.shapes")?
            .iter()
            .filter(|s| s.get("ty") == Some(&json!("sh")))
            .any(|s| {
                !s.get("ks")
                    .is_some_and(|ks| strict_equals(ks.get("a"), Some(&json!(0))))
            });
        ensure(!resampled, || "Rigid path was resampled".into())?;
        let ks = layer.get("ks");
        let base = node.base_points.as_deref().ok_or_else(|| {
            VerifyError::Assertion(format!("{} has a transform but no basePoints", node.name))
        })?;
        let at = |p: &[f64], i: usize| p.get(i).copied().unwrap_or(f64::NAN);
        for f in 0..=144u32 {
            let frame = f64::from(f);
            let r = at(&property(prop(ks, "r")?, frame)?, 0) * std::f64::consts::PI / 180.0;
            let pos = property(prop(ks, "p")?, frame)?;
            let anchor = property(prop(ks, "a")?, frame)?;
            let expected = Sampler::Ge.coords(&node.points, frame / 60.0);
            let (cos, sin) = (libm::cos(r), libm::sin(r));
            for i in (0..base.len()).step_by(2) {
                let (x, y) = (base[i] - at(&anchor, 0), at(base, i + 1) - at(&anchor, 1));
                let actual = [
                    at(&pos, 0) + x * cos - y * sin,
                    at(&pos, 1) + x * sin + y * cos,
                ];
                for (j, &value) in actual.iter().enumerate() {
                    let want = at(&expected, i + j);
                    max_error = max2(max_error, (value - want).abs());
                    close(value, want, 3e-5, || "Native world geometry".into())?;
                }
            }
        }
    }
    Ok((count, max_error))
}

/// Phase D for one action: position and velocity continuity across the loop seam.
fn loop_seams(set: &SetContext<'_>, meta: Record<'_>) -> Result<Value, VerifyError> {
    let name = meta.name()?;
    let scene = set.scene(name)?;
    let (mut max_seam, mut max_ratio) = (0.0f64, 0.0f64);
    for n in &scene.nodes {
        let vis = visibility(n)?;
        let hidden = [0.0, 1.0 / 60.0, 143.0 / 60.0, 2.4]
            .iter()
            .all(|&t| Sampler::Ge.scalar(vis, t) == 0.0);
        let point_samples: Vec<Vec<f64>> = if hidden {
            Vec::new()
        } else {
            (0..=144u32)
                .map(|f| {
                    Sampler::Ge
                        .coords(&n.points, f64::from(f) / 60.0)
                        .into_owned()
                })
                .collect()
        };
        let vis_samples: Vec<Vec<f64>> = (0..=144u32)
            .map(|f| vec![Sampler::Ge.scalar(vis, f64::from(f) / 60.0)])
            .collect();
        for samples in [point_samples, vis_samples] {
            let Some(first) = samples.first() else {
                continue;
            };
            for c in 0..first.len() {
                let value = |i: usize| samples[i].get(c).copied().unwrap_or(f64::NAN);
                let (a, b) = (value(0), value(144));
                max_seam = max2(max_seam, (a - b).abs());
                close(a, b, 0.00003, || format!("{name} loop position"))?;
                let velocities: Vec<f64> = (0..144).map(|i| value(i + 1) - value(i)).collect();
                let peak = max_of(velocities.iter().map(|v| v.abs()));
                let mismatch = (velocities[0] - velocities[143]).abs();
                let acceleration =
                    max_of((1..144).map(|i| (velocities[i] - velocities[i - 1]).abs()));
                ensure(mismatch <= acceleration * 1.06 + 0.0001, || {
                    format!("{name} loop acceleration spike {}", n.name)
                })?;
                if peak > 0.0001 {
                    max_ratio = max2(max_ratio, mismatch / peak);
                }
            }
        }
    }
    Ok(json!({"name": name, "maxSeamPositionError": max_seam, "maxVelocityRatio": max_ratio}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(points: Value, visibility: Value) -> SceneNode {
        serde_json::from_value(json!({
            "name": "recycle-bin-lid__stroke__outline",
            "commands": [0],
            "points": points,
            "width": [[0, 2]],
            "visibility": visibility,
        }))
        .unwrap()
    }

    fn fail(node: &SceneNode, record: &Value) -> String {
        check_node(node, Record(record), NodeChecks::Guarded(Hold::Always))
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn hold_failures_name_the_part_and_times() {
        let action = json!({"name": "recycle-bin-reset-strong", "rest": 48});
        let moving = node(
            json!([[0, [0, 0]], [0.85, [1, 0]], [1, [0, 0]]]),
            json!([[0, 1], [1, 1]]),
        );
        let message = fail(&moving, &action);
        assert!(
            message.starts_with("recycle-bin-reset-strong: moving after rest: recycle-bin-lid moves at 0.850 s, after rest at 0.800 s"),
            "{message}"
        );
        assert!(message.contains("hint:"), "{message}");

        let fading = node(json!([[0, [0, 0]], [1, [0, 0]]]), json!([[0, 1], [0.9, 0.5], [1, 1]]));
        let message = fail(&fading, &action);
        assert!(
            message.starts_with("recycle-bin-reset-strong: visibility moving after rest: recycle-bin-lid opacity 0.5 at 0.900 s, 1 at the end, after rest at 0.800 s"),
            "{message}"
        );
    }

    fn scene(name: &str, nodes: Vec<SceneNode>) -> Scene {
        Scene { name: name.into(), nodes }
    }

    #[test]
    fn pairing_failures_name_the_part() {
        let still = node(json!([[0, [0, 0]], [1, [0, 0]]]), json!([[0, 1], [1, 1]]));
        let ends_apart = node(json!([[0, [0, 0]], [1, [1, 0]]]), json!([[0, 1], [1, 1]]));
        let light = scene("recycle-bin", vec![still.clone()]);
        let message = pair_light_strong("recycle-bin", &light, &scene("recycle-bin-strong", vec![ends_apart]))
            .unwrap_err()
            .to_string();
        assert!(
            message.starts_with("recycle-bin final drawing: recycle-bin-lid differs between Light and Strong; hint:"),
            "{message}"
        );
        assert!(pair_light_strong("recycle-bin", &light, &light).is_ok());

        let forward = scene("recycle-bin", vec![node(json!([[0, [0, 0]], [1, [2, 0]]]), json!([[0, 1], [1, 1]]))]);
        assert_eq!(
            pair_reverse("recycle-bin", &forward, &scene("recycle-bin-reverse", vec![still])).unwrap_err().to_string(),
            "recycle-bin reverse initial geometry: recycle-bin-lid in recycle-bin-reverse does not start where recycle-bin ends"
        );
    }

    #[test]
    fn reset_and_coordinate_failures_name_the_part() {
        let action = json!({"name": "recycle-bin-strong", "rest": 48});
        let open = node(json!([[0, [0, 0]], [1, [1, 0]]]), json!([[0, 1], [1, 1]]));
        assert_eq!(
            fail(&open, &action),
            "recycle-bin-strong: reset changes visible geometry: recycle-bin-lid ends away from its first pose"
        );
        let mut broken = node(json!([[0, [0, 0]], [1, [0, 0]]]), json!([[0, 1], [1, 1]]));
        broken.points[0].1[1] = f64::NAN;
        broken.points[1].1[1] = f64::NAN;
        assert_eq!(
            fail(&broken, &action),
            "recycle-bin-strong: non-finite coordinate: recycle-bin-lid at 0.000 s"
        );
        let morph = node(json!([[0, [0, 0]], [0.5, [0]], [1, [0, 0]]]), json!([[0, 1], [1, 1]]));
        assert_eq!(
            fail(&morph, &json!({"name": "recycle-bin-strong", "rest": 60})),
            "recycle-bin-strong: Morph topology changed: recycle-bin-lid has 1 coordinates at 0.500 s, 2 at the start"
        );
    }
}
