//! Scene assembly: `scene.json` text from a set's description and its sampled animations
//! (js-runtime §7.4, builder §4). The output is byte-identical to the old builders'
//! `JSON.stringify(scene)`: runs of equal frames are compacted, keys keep the builder's
//! order and every number is printed by `gai_core` as JS would print it.
//!
//! Nothing is written to disk; callers get the exact text per animation.

use gai_core::to_js_json;
use gai_js::{DURATION, Description, FRAMES, Kind, Limits, SetRuntime, SetSource, VariantSamples, frame_time};
use serde_json::{Map, Value, json};

/// Every node's paint colour in a scene; the paint contract rebinds it later.
const PRIMARY: &str = "#00ddb3";

/// One animation's scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    /// Index into [`Description::icons`].
    pub icon: usize,
    /// Index into that icon's variants.
    pub variant: usize,
    /// The animation name, which is also the file stem (`<name>.scene.json`).
    pub name: String,
    /// The `scene.json` text: compact, no trailing newline.
    pub json: String,
}

/// A set's description with its scenes in builder order.
#[derive(Debug, Clone)]
pub struct SetScenes {
    pub description: Description,
    pub scenes: Vec<Scene>,
}

/// Why a set's scenes could not be built.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error(transparent)]
    Js(#[from] gai_js::Error),
    /// The old builders would have written `null`; no committed scene has one.
    #[error("{animation}: {what} is not a finite number")]
    NotFinite { animation: String, what: String },
    #[error("{animation}: rest is not a number")]
    Rest { animation: String },
    #[error("no icon named {0}")]
    UnknownIcon(String),
    #[error("{animation}: {sampled} parts sampled for {described} described")]
    PartCount { animation: String, sampled: usize, described: usize },
}

/// Loads `source` in a fresh runtime, installs it and assembles every scene in builder
/// order. Sets run in parallel through [`gai_js::map_parallel`], whose threads also give
/// QuickJS its full stack limit.
pub fn set_scenes(source: &SetSource, limits: &Limits) -> Result<SetScenes, SceneError> {
    let mut runtime = SetRuntime::load(source, limits)?;
    let description = runtime.install()?;
    let scenes = build_scenes(&runtime, &description)?;
    Ok(SetScenes { description, scenes })
}

/// Like [`set_scenes`] for one icon, for a user project that builds a single icon. The
/// whole set is still evaluated, as the builder did; only this icon is sampled.
pub fn icon_scenes(source: &SetSource, limits: &Limits, icon: &str) -> Result<SetScenes, SceneError> {
    let mut runtime = SetRuntime::load(source, limits)?;
    let description = runtime.install()?;
    let index = description
        .icons
        .iter()
        .position(|candidate| candidate.name == icon)
        .ok_or_else(|| SceneError::UnknownIcon(icon.to_owned()))?;
    let scenes = build_icon_scenes(&runtime, &description, index)?;
    Ok(SetScenes { description, scenes })
}

/// Samples and assembles every animation of an installed set, in builder order.
pub fn build_scenes(runtime: &SetRuntime, description: &Description) -> Result<Vec<Scene>, SceneError> {
    let mut scenes = Vec::new();
    for icon in 0..description.icons.len() {
        scenes.extend(build_icon_scenes(runtime, description, icon)?);
    }
    Ok(scenes)
}

/// Samples and assembles the animations of one icon, in builder order.
pub fn build_icon_scenes(
    runtime: &SetRuntime,
    description: &Description,
    icon: usize,
) -> Result<Vec<Scene>, SceneError> {
    let variants = &description.icons[icon].variants;
    (0..variants.len())
        .map(|variant| {
            let samples = runtime.sample_variant(icon, variant)?;
            Ok(Scene {
                icon,
                variant,
                name: variants[variant].name.clone(),
                json: assemble_scene(description, icon, variant, &samples)?,
            })
        })
        .collect()
}

/// The `scene.json` text of one sampled animation.
pub fn assemble_scene(
    description: &Description,
    icon: usize,
    variant: usize,
    samples: &VariantSamples,
) -> Result<String, SceneError> {
    let parts = &description.icons[icon].parts;
    let animation = &description.icons[icon].variants[variant];
    let name = animation.name.as_str();
    if samples.parts.len() != parts.len() {
        return Err(SceneError::PartCount {
            animation: name.to_owned(),
            sampled: samples.parts.len(),
            described: parts.len(),
        });
    }
    // Modern variants carry the rest already ceiled to whole frames; legacy ones keep it raw.
    let rest = animation.rest().ok_or_else(|| SceneError::Rest { animation: name.to_owned() })?;
    let legacy = description.kind == Kind::Legacy;
    let nodes = parts
        .iter()
        .zip(&samples.parts)
        .map(|(part, sampled)| {
            let finite = |what: &str, values: &[f64]| {
                values.iter().all(|v| v.is_finite()).then_some(()).ok_or_else(|| SceneError::NotFinite {
                    animation: name.to_owned(),
                    what: format!("{} {what}", part.node.name),
                })
            };
            finite("points", &sampled.points)?;
            let mut node = Map::new();
            node.insert("name".into(), json!(part.node.name));
            node.insert("commands".into(), json!(sampled.commands));
            let frames: Vec<&[f64]> = (0..FRAMES).map(|f| sampled.frame(f)).collect();
            node.insert("points".into(), track(&frames, |frame| numbers(frame)));
            if let Some(rigid) = &sampled.rigid {
                finite("anchor", &rigid.anchor)?;
                finite("basePoints", &rigid.base_points)?;
                finite("position", rigid.position.as_flattened())?;
                finite("rotation", &rigid.rotation)?;
                node.insert("basePoints".into(), numbers(&rigid.base_points));
                node.insert(
                    "transform".into(),
                    json!({
                        "anchor": numbers(&rigid.anchor),
                        "position": track(&rigid.position, |p| numbers(p)),
                        "rotation": track(&rigid.rotation, |r| number(*r)),
                    }),
                );
            }
            if let Some(kind) = &part.node.r#type {
                node.insert("type".into(), kind.clone());
            }
            node.insert("color".into(), json!(PRIMARY));
            let opacity = number(part.node.opacity);
            node.insert("opacity".into(), json!([[0, opacity], [number(DURATION), opacity]]));
            let width = &part.node.width;
            node.insert("width".into(), json!([[0, width], [number(DURATION), width]]));
            if !legacy {
                let visibility = sampled.visibility.as_deref().unwrap_or_default();
                finite("visibility", visibility)?;
                node.insert("visibility".into(), track(visibility, |v| number(*v)));
            }
            Ok(Value::Object(node))
        })
        .collect::<Result<Vec<_>, SceneError>>()?;
    let scene = json!({
        "name": name,
        "width": 32,
        "height": 32,
        "fps": 60,
        "duration": number(DURATION),
        "rest": number(rest),
        "nodes": nodes,
    });
    Ok(to_js_json(&scene))
}

/// `[[t, value], …]` over the frames `compact()` keeps (`build-set.mjs:52-61`): the first
/// and last frame, and every frame that differs from a neighbour. Values are compared as
/// `JSON.stringify` would (finite numbers, so `-0 == 0`).
fn track<V: PartialEq>(frames: &[V], value: impl Fn(&V) -> Value) -> Value {
    let last = frames.len().saturating_sub(1);
    let kept = (0..frames.len())
        .filter(|&i| i == 0 || i == last || frames[i] != frames[i - 1] || frames[i] != frames[i + 1])
        .map(|i| json!([number(frame_time(i)), value(&frames[i])]))
        .collect();
    Value::Array(kept)
}

fn numbers(values: &[f64]) -> Value {
    Value::Array(values.iter().map(|v| number(*v)).collect())
}

/// A finite f64 as a JSON number that `gai_core` prints like JS.
fn number(value: f64) -> Value {
    serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compaction_keeps_the_ends_of_each_run() {
        let values = [1.0, 1.0, 1.0, 2.0, 2.0, 3.0, -0.0, 0.0];
        let out = track(&values, |v| number(*v));
        let times: Vec<f64> = out.as_array().unwrap().iter().map(|k| k[0].as_f64().unwrap()).collect();
        let frames: Vec<f64> = [0, 2, 3, 4, 5, 6, 7].iter().map(|&f| frame_time(f)).collect();
        assert_eq!(times, frames);
    }

    #[test]
    fn frame_times_print_like_js() {
        assert_eq!(to_js_json(&number(frame_time(1))), "0.016666666666666666");
        assert_eq!(to_js_json(&number(frame_time(144))), "2.4");
        assert_eq!(to_js_json(&number(-0.0)), "0");
    }
}
