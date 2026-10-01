//! Post-export (builder spec §6.1–§6.7): from a normalized native Lottie to the final asset,
//! its two theme files and the builder's dotLottie manifest.
//!
//! The native is parsed into an order-preserving tree and re-serialized the way the old
//! builders' `JSON.stringify(native)` did: keys keep the engine's (sorted) order, new keys
//! (`slots`, `markers`, `meta`, each `sid`) are appended, an assignment to an existing key
//! keeps its position, and every number is reprinted as JS prints it (`60.0` → `60`,
//! `-0.0` → `0`, `-9.536743e-6` → `-0.000009536743`).

use gai_core::js::truthy;
use gai_core::to_js_json;
use serde_json::{Map, Value, json};

/// The dark theme's primary colour, which is also the colour every scene node is drawn in.
pub const PRIMARY_DARK: &str = "#00ddb3";
/// The light theme's primary colour.
pub const PRIMARY_LIGHT: &str = "#008c76";
/// The only colour slot (`library/paint-tokens.mjs`).
pub const COLOR_SLOT: &str = "color.primary";
/// The stroke-width token bound to Regular outlines.
pub const STROKE_WIDTH_SLOT: &str = "style.outline.width";
/// The style opacity slots and their defaults: Regular outline visible.
pub const STYLE_SLOTS: [(&str, u32); 5] = [
    ("style.outline.light", 0),
    ("style.outline.regular", 100),
    ("style.outline.bold", 0),
    ("style.solid", 0),
    ("style.tint", 0),
];
/// The default of [`STROKE_WIDTH_SLOT`]: the Regular weight.
const REGULAR_WEIGHT: u32 = 2;
/// The themes, in the order their files are written, with their primary colour.
pub const THEMES: [(&str, &str); 2] = [("dark", PRIMARY_DARK), ("light", PRIMARY_LIGHT)];
/// Frames per second, as the markers count them.
const FPS: f64 = gai_js::FPS;

/// What the composition is named and marked with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Composition<'a> {
    /// The composition `nm`, `Motion / <set name> / <label>`.
    pub name: &'a str,
    /// The set id, written to `meta.set`.
    pub set_id: &'a str,
    /// The authored action length in seconds (`icon.active`).
    pub active: f64,
    /// The rest time in seconds, already ceiled to whole frames.
    pub rest: f64,
}

/// One animation's outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// `<name>.json`: compact, no trailing newline.
    pub json: String,
    /// `<name>.theme-dark.json` and `<name>.theme-light.json`, in [`THEMES`] order.
    pub themes: [String; 2],
    /// The layer count, for the record's `layers`.
    pub layers: usize,
    /// The slot ids in order, for the record's `slots`.
    pub slots: Vec<String>,
}

/// Why a native could not be post-processed.
#[derive(Debug, thiserror::Error)]
pub enum PostExportError {
    #[error("native is not JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("native is not a JSON object")]
    NotAnObject,
    #[error("native has no layers array")]
    Layers,
    /// A layer name that is not `<part>__color.primary__<style>` with a known style.
    #[error("Missing paint contract: {0}")]
    PaintContract(String),
    #[error("layer {layer}: {what}")]
    Layer { layer: usize, what: String },
    #[error("{0} is not a finite number")]
    NotFinite(&'static str),
}

/// Post-processes the native text of one animation (`build-set.mjs:139-187`).
pub fn post_export(native_text: &str, composition: &Composition<'_>) -> Result<Asset, PostExportError> {
    for (what, value) in [("active", composition.active), ("rest", composition.rest)] {
        if !value.is_finite() {
            return Err(PostExportError::NotFinite(what));
        }
    }
    let mut native: Value = serde_json::from_str(native_text)?;
    let map = native.as_object_mut().ok_or(PostExportError::NotAnObject)?;
    map.insert("nm".into(), composition.name.into());
    bind_paint(map)?;
    map.insert(
        "markers".into(),
        json!([
            {"cm": "action", "tm": 0, "dr": js_number(composition.active * FPS)},
            {"cm": "rest", "tm": js_number(composition.rest * FPS), "dr": 1},
            {"cm": "showcase", "tm": 0, "dr": js_number(gai_js::DURATION * FPS)},
        ]),
    );
    map.insert(
        "meta".into(),
        json!({
            "generator": "Engine MCP native export",
            "set": composition.set_id,
            "restFrame": js_number(composition.rest * FPS),
        }),
    );
    let layers = map.get("layers").and_then(Value::as_array).map_or(0, Vec::len);
    let slots_map = map.get("slots").and_then(Value::as_object).cloned().unwrap_or_default();
    let themes = THEMES.map(|(_, hex)| theme(hex, &slots_map));
    Ok(Asset { json: to_js_json(&native), themes, layers, slots: slots_map.keys().cloned().collect() })
}

/// `bindPaint(native, palettes)` (`library/paint-tokens.mjs:20-38`): replaces `slots` with
/// the colour, style and stroke-width slots, binds each top-level fill and stroke to them
/// through `sid`, and strips the paint contract from layer names.
pub fn bind_paint(native: &mut Map<String, Value>) -> Result<(), PostExportError> {
    let mut slots = Map::new();
    slots.insert(COLOR_SLOT.into(), json!({"p": {"a": 0, "k": rgb(PRIMARY_DARK)}}));
    for (id, k) in STYLE_SLOTS {
        slots.insert(id.into(), json!({"p": {"a": 0, "k": k}}));
    }
    slots.insert(STROKE_WIDTH_SLOT.into(), json!({"p": {"a": 0, "k": REGULAR_WEIGHT}}));
    let layers = native.get_mut("layers").and_then(Value::as_array_mut).ok_or(PostExportError::Layers)?;
    for (index, layer) in layers.iter_mut().enumerate() {
        let fail = |what: &str| PostExportError::Layer { layer: index, what: what.to_owned() };
        let layer = layer.as_object_mut().ok_or_else(|| fail("not an object"))?;
        let nm = layer.get("nm").and_then(Value::as_str).ok_or_else(|| fail("nm is not a string"))?.to_owned();
        let mut fields = nm.split("__");
        let name = fields.next().unwrap_or_default().to_owned();
        let slot = fields.next().unwrap_or_default();
        let style = format!("style.{}", fields.next().unwrap_or("undefined"));
        // The palette's own keys only: JS would also accept inherited names such as "constructor".
        if slot != COLOR_SLOT || !slots.contains_key(&style) {
            return Err(PostExportError::PaintContract(nm));
        }
        let regular = style == "style.outline.regular";
        match layer.get_mut("shapes") {
            // `layer.shapes || []`
            shapes if !truthy(shapes.as_deref()) => {}
            Some(Value::Array(shapes)) => {
                for shape in shapes {
                    let shape = shape.as_object_mut().ok_or_else(|| fail("a shape is not an object"))?;
                    let stroke = match shape.get("ty").and_then(Value::as_str) {
                        Some("fl") => false,
                        Some("st") => true,
                        _ => continue,
                    };
                    set_sid(shape, "c", slot).ok_or_else(|| fail("a paint has no c object"))?;
                    set_sid(shape, "o", &style).ok_or_else(|| fail("a paint has no o object"))?;
                    if stroke && regular {
                        set_sid(shape, "w", STROKE_WIDTH_SLOT).ok_or_else(|| fail("a stroke has no w object"))?;
                    }
                }
            }
            _ => return Err(fail("shapes is not an array")),
        }
        layer.insert("nm".into(), name.into());
    }
    native.insert("slots".into(), Value::Object(slots));
    Ok(())
}

/// `shape[property].sid = sid`; `None` when the property is not an object.
fn set_sid(shape: &mut Map<String, Value>, property: &str, sid: &str) -> Option<()> {
    shape.get_mut(property)?.as_object_mut()?.insert("sid".into(), sid.into());
    Some(())
}

/// A theme file: the palette's primary colour, then every other slot's default
/// (`build-set.mjs:178-187`).
pub fn theme(primary: &str, slots: &Map<String, Value>) -> String {
    let mut rules = vec![json!({"id": COLOR_SLOT, "type": "Color", "value": rgb(primary)})];
    for (id, slot) in slots.iter().filter(|(id, _)| *id != COLOR_SLOT) {
        let mut rule = Map::new();
        rule.insert("id".into(), id.as_str().into());
        rule.insert("type".into(), "Scalar".into());
        // `slot.p.k`; JSON.stringify drops an undefined value.
        if let Some(value) = slot.pointer("/p/k") {
            rule.insert("value".into(), value.clone());
        }
        rules.push(Value::Object(rule));
    }
    to_js_json(&json!({ "rules": rules }))
}

/// The dotLottie manifest the builder writes for one animation (`build-set.mjs:165-177`).
/// Packaging rewrites it to list every animation of the icon.
pub fn builder_manifest(name: &str) -> String {
    to_js_json(&json!({
        "version": "2",
        "generator": "LottieFiles Motion / Engine MCP",
        "initial": {"animation": name},
        "animations": [{"id": name, "initialTheme": "dark", "themes": ["dark", "light"]}],
        "themes": [{"id": "dark"}, {"id": "light"}],
    }))
}

/// `[r, g, b]` of a `#rrggbb` colour as fractions: `parseInt(hex.slice(i, i + 2), 16) / 255`.
pub fn rgb(hex: &str) -> Value {
    let channel = |at: usize| {
        let value = hex.get(at..at + 2).and_then(|digits| u8::from_str_radix(digits, 16).ok());
        value.map_or(Value::Null, |v| js_number(f64::from(v) / 255.0))
    };
    json!([channel(1), channel(3), channel(5)])
}

/// The JSON value that reading V8's text for `x` gives back: integers V8 prints as plain
/// digits (below 1e21) become JSON integers when they fit, so both the JS and the Python
/// writers print them without a fraction. Non-finite values become `null`, as in
/// `JSON.stringify`.
pub(crate) fn js_number(x: f64) -> Value {
    const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;
    if x.is_finite() && x.fract() == 0.0 && x.abs() < 1e21 {
        if x == 0.0 {
            return Value::from(0u64);
        }
        if x < 0.0 && x >= i64::MIN as f64 {
            return Value::from(x as i64);
        }
        if x > 0.0 && x < TWO_POW_64 {
            return Value::from(x as u64);
        }
    }
    serde_json::Number::from_f64(x).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE: &str = r#"{"assets":[],"fr":60.0,"layers":[{"ind":1,"nm":"dot-bold__color.primary__outline.regular","shapes":[{"ks":{"a":0,"k":{"c":true}},"nm":"native-export-7","ty":"sh"},{"c":{"a":0,"k":[0.0,0.8666667,0.7019608,1.0]},"o":{"a":0,"k":-0.0},"ty":"st","w":{"a":0,"k":2.0}}],"ty":4},{"ind":2,"nm":"dot__color.primary__tint","shapes":[{"c":{"a":0,"k":[0.0,0.0,0.0,1.0]},"o":{"a":1,"k":[{"s":[-9.536743e-6],"t":0.0}]},"ty":"fl"}],"ty":4}],"nm":"engine","op":145.0,"w":32}"#;

    #[test]
    fn asset_follows_json_stringify() {
        let composition = Composition { name: "Motion / Set 04 / Dot", set_id: "04-actions", active: 0.24, rest: 0.55 };
        let asset = post_export(NATIVE, &composition).unwrap();
        assert_eq!(
            asset.json,
            concat!(
                r#"{"assets":[],"fr":60,"layers":[{"ind":1,"nm":"dot-bold","shapes":[{"ks":{"a":0,"k":{"c":true}},"nm":"native-export-7","ty":"sh"},"#,
                r#"{"c":{"a":0,"k":[0,0.8666667,0.7019608,1],"sid":"color.primary"},"o":{"a":0,"k":0,"sid":"style.outline.regular"},"ty":"st","w":{"a":0,"k":2,"sid":"style.outline.width"}}],"ty":4},"#,
                r#"{"ind":2,"nm":"dot","shapes":[{"c":{"a":0,"k":[0,0,0,1],"sid":"color.primary"},"o":{"a":1,"k":[{"s":[-0.000009536743],"t":0}],"sid":"style.tint"},"ty":"fl"}],"ty":4}],"#,
                r#""nm":"Motion / Set 04 / Dot","op":145,"w":32,"#,
                r#""slots":{"color.primary":{"p":{"a":0,"k":[0,0.8666666666666667,0.7019607843137254]}},"style.outline.light":{"p":{"a":0,"k":0}},"style.outline.regular":{"p":{"a":0,"k":100}},"style.outline.bold":{"p":{"a":0,"k":0}},"style.solid":{"p":{"a":0,"k":0}},"style.tint":{"p":{"a":0,"k":0}},"style.outline.width":{"p":{"a":0,"k":2}}},"#,
                r#""markers":[{"cm":"action","tm":0,"dr":14.399999999999999},{"cm":"rest","tm":33,"dr":1},{"cm":"showcase","tm":0,"dr":144}],"#,
                r#""meta":{"generator":"Engine MCP native export","set":"04-actions","restFrame":33}}"#,
            )
        );
        assert_eq!(asset.layers, 2);
        assert_eq!(asset.slots.len(), 7);
        assert_eq!(
            asset.themes[1],
            r#"{"rules":[{"id":"color.primary","type":"Color","value":[0,0.5490196078431373,0.4627450980392157]},{"id":"style.outline.light","type":"Scalar","value":0},{"id":"style.outline.regular","type":"Scalar","value":100},{"id":"style.outline.bold","type":"Scalar","value":0},{"id":"style.solid","type":"Scalar","value":0},{"id":"style.tint","type":"Scalar","value":0},{"id":"style.outline.width","type":"Scalar","value":2}]}"#
        );
    }

    #[test]
    fn paint_contract_is_enforced() {
        for nm in ["dot", "dot__color.accent__tint", "dot__color.primary__glow", "dot__color.primary"] {
            let native = format!(r#"{{"layers":[{{"nm":"{nm}","shapes":[]}}]}}"#);
            let composition = Composition { name: "n", set_id: "s", active: 1.0, rest: 1.0 };
            let error = post_export(&native, &composition).unwrap_err();
            assert_eq!(error.to_string(), format!("Missing paint contract: {nm}"));
        }
    }

    #[test]
    fn manifest_matches_the_builder() {
        assert_eq!(
            builder_manifest("bell"),
            r#"{"version":"2","generator":"LottieFiles Motion / Engine MCP","initial":{"animation":"bell"},"animations":[{"id":"bell","initialTheme":"dark","themes":["dark","light"]}],"themes":[{"id":"dark"},{"id":"light"}]}"#
        );
    }

    #[test]
    fn js_values() {
        assert_eq!(js_number(-0.0), json!(0));
        assert_eq!(js_number(144.0), json!(144));
        assert_eq!(js_number(31.000000000000004), json!(31.000000000000004));
        assert_eq!(js_number(f64::NAN), Value::Null);
    }
}
