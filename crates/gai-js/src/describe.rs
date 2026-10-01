//! What `describe()` reports about an installed set.

use crate::runtime::Fail;
use serde_json::Value as Json;

/// Frames per second of every scene.
pub const FPS: f64 = 60.0;
/// Scene duration in seconds.
pub const DURATION: f64 = 2.4;
/// Frames sampled per animation: `Math.round(DURATION * FPS) + 1`.
pub const FRAMES: usize = 145;

/// The time of frame `frame` in seconds, as the builders compute it (`frame / 60`, a
/// division: `frame * (1 / 60)` differs on 10 frames).
pub fn frame_time(frame: usize) -> f64 {
    frame as f64 / FPS
}

/// Which builder family sampled the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Sets 03 onward and user projects: rest ceiled to whole frames, visibility, rigid transforms.
    Modern,
    /// Sets 01 and 02: raw rest, no visibility, no transforms, slot-based node names.
    Legacy,
}

/// An installed set: its icons in definitions order.
#[derive(Debug, Clone, PartialEq)]
pub struct Description {
    pub kind: Kind,
    pub icons: Vec<Icon>,
}

/// One icon of the set, with its parts and animations.
#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    /// `icon.name`.
    pub name: String,
    /// The icon without `parts`, as `JSON.stringify` sees it (key order kept, functions and
    /// `undefined` dropped, numbers exact).
    pub meta: Json,
    /// The authored parts, before the weight expansion.
    pub source_parts: Vec<SourcePart>,
    /// The parts after the weight expansion, in scene node order.
    pub parts: Vec<Part>,
    /// The animations in builder order: light, light reverse, strong, strong reverse.
    pub variants: Vec<Variant>,
}

/// An authored part.
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePart {
    pub callbacks: Callbacks,
    /// The part without `d`, `pose` and `visibility`.
    pub meta: Json,
}

/// A part after the weight expansion: one scene node.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// Index into [`Icon::source_parts`] of the part this one expands.
    pub source_index: usize,
    pub callbacks: Callbacks,
    /// The part without `d`, `pose` and `visibility`.
    pub meta: Json,
    pub node: Node,
}

/// Scene node fields computed in JS, so JS truthiness and string conversion apply.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// `${name}__color.primary__${style}` (legacy: `${name}__${slot}__${style}`).
    pub name: String,
    /// `part.type || 'fill'`; `None` when `JSON.stringify` drops it (a function).
    pub r#type: Option<Json>,
    /// `part.style === 'outline.regular' ? 1 : 0`.
    pub opacity: f64,
    /// `part.width || 1`; `Null` where `JSON.stringify` writes `null` in an array.
    pub width: Json,
}

/// The JS types of a part's callbacks (`typeof`), which decide how it is sampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Callbacks {
    pub d: JsType,
    pub pose: JsType,
    pub visibility: JsType,
    /// `Boolean(part.pose && part.pose.transform)`.
    pub transform: bool,
}

/// A `typeof` result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsType {
    Undefined,
    /// Objects and `null`.
    Object,
    Boolean,
    Number,
    BigInt,
    String,
    Symbol,
    Function,
}

impl JsType {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "undefined" => JsType::Undefined,
            "object" => JsType::Object,
            "boolean" => JsType::Boolean,
            "number" => JsType::Number,
            "bigint" => JsType::BigInt,
            "string" => JsType::String,
            "symbol" => JsType::Symbol,
            "function" => JsType::Function,
            _ => return None,
        })
    }
}

/// One animation of an icon.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    /// The animation name, for example `alarm-clock-reverse-strong`.
    pub name: String,
    pub strong: bool,
    pub reverse: bool,
    /// The builder's variant object without `parts`: the base icon, the timing and
    /// `icon, motionId, motionLabel, direction, name` (legacy: strong overrides instead of
    /// timing and no `direction`). `rest` is already ceiled to whole frames for modern sets.
    pub meta: Json,
}

impl Variant {
    /// `rest` in seconds as the scene carries it, when it is a finite number.
    pub fn rest(&self) -> Option<f64> {
        self.meta.get("rest")?.as_f64()
    }

    /// `motionId`: `light` or `strong`.
    pub fn motion_id(&self) -> &str {
        if self.strong { "strong" } else { "light" }
    }
}

impl Description {
    pub(crate) fn from_json(json: &Json) -> Result<Self, Fail> {
        let kind = match json.get("kind").and_then(Json::as_str) {
            Some("modern") => Kind::Modern,
            Some("legacy") => Kind::Legacy,
            other => return Err(bad(format!("kind {other:?}"))),
        };
        if field(json, "frames")?.as_u64() != Some(FRAMES as u64) {
            return Err(bad("frame count".into()));
        }
        let icons = array(json, "icons")?
            .iter()
            .enumerate()
            .map(|(i, icon)| icon_from_json(i, icon))
            .collect::<Result<_, _>>()?;
        Ok(Description { kind, icons })
    }
}

fn icon_from_json(index: usize, json: &Json) -> Result<Icon, Fail> {
    let meta = field(json, "meta")?.clone();
    let name = meta
        .get("name")
        .and_then(Json::as_str)
        .ok_or_else(|| Fail::Protocol(format!("icon {index} has no string name")))?
        .to_owned();
    let source_parts = array(json, "sourceParts")?
        .iter()
        .map(|part| {
            Ok(SourcePart { callbacks: callbacks(field(part, "callbacks")?)?, meta: field(part, "meta")?.clone() })
        })
        .collect::<Result<_, Fail>>()?;
    let parts = array(json, "parts")?
        .iter()
        .map(|part| {
            let node = field(part, "node")?;
            Ok(Part {
                source_index: usize_field(part, "sourceIndex")?,
                callbacks: callbacks(field(part, "callbacks")?)?,
                meta: field(part, "meta")?.clone(),
                node: Node {
                    name: str_field(node, "name")?.to_owned(),
                    r#type: node.get("type").cloned(),
                    opacity: field(node, "opacity")?.as_f64().ok_or_else(|| bad("node opacity".into()))?,
                    width: node.get("width").cloned().unwrap_or(Json::Null),
                },
            })
        })
        .collect::<Result<_, Fail>>()?;
    let variants = array(json, "variants")?
        .iter()
        .map(|variant| {
            Ok(Variant {
                name: str_field(variant, "name")?.to_owned(),
                strong: bool_field(variant, "strong")?,
                reverse: bool_field(variant, "reverse")?,
                meta: field(variant, "meta")?.clone(),
            })
        })
        .collect::<Result<_, Fail>>()?;
    Ok(Icon { name, meta, source_parts, parts, variants })
}

/// Reads the host's `callbacks(part)` object.
pub(crate) fn callbacks(callbacks: &Json) -> Result<Callbacks, Fail> {
    let js_type = |key: &str| JsType::parse(str_field(callbacks, key)?).ok_or_else(|| bad(format!("typeof {key}")));
    Ok(Callbacks {
        d: js_type("d")?,
        pose: js_type("pose")?,
        visibility: js_type("visibility")?,
        transform: bool_field(callbacks, "transform")?,
    })
}

fn bad(what: String) -> Fail {
    Fail::Protocol(format!("describe(): unexpected {what}"))
}

fn field<'a>(json: &'a Json, key: &str) -> Result<&'a Json, Fail> {
    json.get(key).ok_or_else(|| bad(format!("missing {key}")))
}

fn array<'a>(json: &'a Json, key: &str) -> Result<&'a Vec<Json>, Fail> {
    field(json, key)?.as_array().ok_or_else(|| bad(format!("{key} (not an array)")))
}

fn str_field<'a>(json: &'a Json, key: &str) -> Result<&'a str, Fail> {
    field(json, key)?.as_str().ok_or_else(|| bad(format!("{key} (not a string)")))
}

fn bool_field(json: &Json, key: &str) -> Result<bool, Fail> {
    field(json, key)?.as_bool().ok_or_else(|| bad(format!("{key} (not a boolean)")))
}

fn usize_field(json: &Json, key: &str) -> Result<usize, Fail> {
    field(json, key)?.as_u64().map(|n| n as usize).ok_or_else(|| bad(format!("{key} (not an index)")))
}
