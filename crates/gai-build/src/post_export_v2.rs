//! Post-export for spec v2 (`gai-icon/2`) exports: the same contract as [`post_export`]
//! (slots, themes, exact markers, `meta`), bound by paint name instead of layer name.
//!
//! v2 layers are rigs (no shapes), painted part layers (`<part>__color.primary__<style>`, a
//! paint per style named `color.primary__<style>`, possibly inside groups: compound paths and
//! the Solid fill-plus-rim) and matte sources (`td: 1`). Each named paint gets `c.sid` and
//! `o.sid`, and Regular strokes `w.sid`; matte sources stay unbound, since a source bound to
//! a style slot vanishes whenever that style is off. v1's `bind_paint` is not used and does
//! not change.
//!
//! [`post_export`]: crate::post_export::post_export

use crate::post_export::{
    Asset, COLOR_SLOT, Composition, PRIMARY_DARK, PostExportError, STROKE_WIDTH_SLOT, STYLE_SLOTS, THEMES, js_number, rgb, theme,
};
use gai_core::to_js_json;
use serde_json::{Map, Value, json};

/// `meta.generator` of a v2 asset.
pub const GENERATOR: &str = "Engine MCP native export (gai-icon/2)";
/// The Regular weight, the stroke-width slot's default.
const REGULAR_WEIGHT: u32 = 2;

/// Post-processes one normalized v2 native.
pub fn post_export_v2(native_text: &str, composition: &Composition<'_>) -> Result<Asset, PostExportError> {
    for (what, value) in [("active", composition.active), ("rest", composition.rest)] {
        if !value.is_finite() {
            return Err(PostExportError::NotFinite(what));
        }
    }
    let fps = 60.0;
    let mut native: Value = serde_json::from_str(native_text)?;
    let map = native.as_object_mut().ok_or(PostExportError::NotAnObject)?;
    map.insert("nm".into(), composition.name.into());
    bind_by_name(map)?;
    // Rest is whole frames by construction; snap away float noise (0.8833… × 60).
    let rest_frame = frame(composition.rest * fps);
    map.insert(
        "markers".into(),
        json!([
            {"cm": "action", "tm": 0, "dr": js_number(frame(composition.active * fps))},
            {"cm": "rest", "tm": js_number(rest_frame), "dr": 1},
            {"cm": "showcase", "tm": 0, "dr": js_number(2.4 * fps)},
        ]),
    );
    map.insert("meta".into(), json!({"generator": GENERATOR, "set": composition.set_id, "restFrame": js_number(rest_frame)}));
    let layers = map.get("layers").and_then(Value::as_array).map_or(0, Vec::len);
    let slots_map = map.get("slots").and_then(Value::as_object).cloned().unwrap_or_default();
    let themes = THEMES.map(|(_, hex)| theme(hex, &slots_map));
    Ok(Asset { json: to_js_json(&native), themes, layers, slots: slots_map.keys().cloned().collect() })
}

/// `x` snapped to the nearest whole frame when it is within 1e-6 of it, else rounded to 6 places.
pub fn frame(x: f64) -> f64 {
    let whole = x.round();
    if (x - whole).abs() < 1e-6 { whole } else { (x * 1e6).round() / 1e6 }
}

/// Binds every contract paint to its slots, strips the contract from painted layers' names,
/// and writes the slots. A painted layer with a paint outside the contract fails.
pub fn bind_by_name(native: &mut Map<String, Value>) -> Result<(), PostExportError> {
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
        if layer.get("td").and_then(Value::as_u64) == Some(1) {
            continue;
        }
        let mut paints = 0;
        if let Some(shapes) = layer.get_mut("shapes").and_then(Value::as_array_mut) {
            paints = bind_items(shapes, &slots).map_err(|what| fail(&format!("{nm}: {what}")))?;
        }
        if let Some((part, style)) = nm.split_once("__") {
            let style = style.strip_prefix(&format!("{COLOR_SLOT}__")).ok_or_else(|| PostExportError::PaintContract(nm.clone()))?;
            if !slots.contains_key(&format!("style.{style}")) {
                return Err(PostExportError::PaintContract(nm.clone()));
            }
            layer.insert("nm".into(), part.into());
        } else if paints > 0 {
            return Err(PostExportError::PaintContract(nm));
        }
    }
    native.insert("slots".into(), Value::Object(slots));
    Ok(())
}

/// Binds the paints of a shape list, recursing into groups; returns how many it bound.
fn bind_items(items: &mut [Value], slots: &Map<String, Value>) -> Result<usize, String> {
    let mut bound = 0;
    for item in items {
        let Some(shape) = item.as_object_mut() else { continue };
        let kind = shape.get("ty").and_then(Value::as_str).unwrap_or_default().to_owned();
        match kind.as_str() {
            "gr" => {
                if let Some(inner) = shape.get_mut("it").and_then(Value::as_array_mut) {
                    bound += bind_items(inner, slots)?;
                }
            }
            "fl" | "st" => {
                let name = shape.get("nm").and_then(Value::as_str).unwrap_or_default().to_owned();
                let style = name.strip_prefix(&format!("{COLOR_SLOT}__")).ok_or_else(|| format!("a {kind} paint is not named color.primary__<style> ({name:?})"))?;
                let style_slot = format!("style.{style}");
                if !slots.contains_key(&style_slot) {
                    return Err(format!("unknown paint style {style}"));
                }
                set_sid(shape, "c", COLOR_SLOT).ok_or("a paint has no c object")?;
                set_sid(shape, "o", &style_slot).ok_or("a paint has no o object")?;
                if kind == "st" && style == "outline.regular" {
                    set_sid(shape, "w", STROKE_WIDTH_SLOT).ok_or("a stroke has no w object")?;
                }
                bound += 1;
            }
            _ => {}
        }
    }
    Ok(bound)
}

fn set_sid(shape: &mut Map<String, Value>, property: &str, sid: &str) -> Option<()> {
    shape.get_mut(property)?.as_object_mut()?.insert("sid".into(), sid.into());
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE: &str = r#"{"assets":[],"fr":60.0,"layers":[
        {"ind":1,"nm":"x-hand-matte","td":1,"shapes":[{"ty":"sh"},{"c":{"a":0,"k":[0,1,0]},"o":{"a":0,"k":100},"ty":"st","w":{"a":0,"k":3.8}}],"ty":4},
        {"ind":2,"nm":"x-disc-solid__color.primary__solid","tt":2,"tp":1,"shapes":[{"ty":"el"},{"c":{"a":0,"k":[0,1,0]},"nm":"color.primary__solid","o":{"a":0,"k":0},"ty":"st","w":{"a":0,"k":2.6}},{"c":{"a":0,"k":[0,1,0]},"nm":"color.primary__solid","o":{"a":0,"k":0},"ty":"fl"}],"ty":4},
        {"ind":3,"nm":"x-disc-regular__color.primary__outline.regular","shapes":[{"ty":"gr","it":[{"ty":"sh"},{"c":{"a":0,"k":[0,1,0]},"nm":"color.primary__outline.regular","o":{"a":0,"k":100},"ty":"st","w":{"a":0,"k":2}}]}],"ty":4},
        {"ind":4,"nm":"rig-disc","shapes":[],"ty":4}],"op":144.0,"w":32}"#;

    #[test]
    fn binds_by_paint_name_and_leaves_matte_sources() {
        let composition = Composition { name: "Motion / Local icons / X", set_id: "00-local", active: 0.45, rest: 0.5 };
        let asset = post_export_v2(NATIVE, &composition).unwrap();
        let v: Value = serde_json::from_str(&asset.json).unwrap();
        let layers = v["layers"].as_array().unwrap();
        assert_eq!(layers[0]["nm"], "x-hand-matte");
        assert!(layers[0]["shapes"][1]["c"].get("sid").is_none());
        assert_eq!(layers[1]["nm"], "x-disc-solid");
        assert_eq!(layers[1]["shapes"][1]["o"]["sid"], "style.solid");
        assert_eq!(layers[1]["shapes"][2]["c"]["sid"], "color.primary");
        assert!(layers[1]["shapes"][1]["w"].get("sid").is_none());
        assert_eq!(layers[2]["shapes"][0]["it"][1]["w"]["sid"], "style.outline.width");
        assert_eq!(layers[3]["nm"], "rig-disc");
        assert_eq!(v["markers"], json!([{"cm": "action", "tm": 0, "dr": 27}, {"cm": "rest", "tm": 30, "dr": 1}, {"cm": "showcase", "tm": 0, "dr": 144}]));
        assert_eq!(v["meta"]["restFrame"], 30);
        assert_eq!(asset.slots.len(), 7);
        assert!(asset.json.ends_with(r#""meta":{"generator":"Engine MCP native export (gai-icon/2)","set":"00-local","restFrame":30}}"#));
    }

    #[test]
    fn an_unnamed_paint_on_a_painted_layer_fails() {
        let native = r#"{"layers":[{"nm":"x-a-light__color.primary__outline.light","shapes":[{"ty":"st","c":{"k":0},"o":{"k":0}}]}]}"#;
        let composition = Composition { name: "n", set_id: "s", active: 1.0, rest: 1.1 };
        assert!(post_export_v2(native, &composition).is_err());
    }
}
