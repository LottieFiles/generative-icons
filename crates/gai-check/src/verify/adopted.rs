//! Icons adopted from spec v2 (`gen-animated-icons library adopt`). Their source is the
//! set's `specs/<icon>.icon.json`, exported by the hosted engine in one call, so they
//! have no sampled scenes and no legacy natives. The legacy stages verify the set
//! without them; this stage checks what they ship: the spec each record names, every
//! action's archive, the format, paint and theme contracts, the markers, and the
//! Light/Strong (and reverse) actions the set's consumers expect.

use super::generic::{ActionVariant, Builder, Hold, NodeChecks, check_format, check_paint, check_themes};
use super::manifest::{Record, SetManifest, array};
use super::js::strict_equals;
use super::{SetContext, VerifyError, ensure, read_json};
use serde_json::{Value, json};

/// The `source` of an adopted icon's records.
pub const SOURCE: &str = "gai-icon/2";

/// The folder of a set that holds adopted specs.
pub const SPECS_DIR: &str = "specs";

/// Whether an icon (or action) record came from a spec.
pub fn is_adopted(record: &Value) -> bool {
    record.get("source").and_then(Value::as_str) == Some(SOURCE)
}

/// Splits a manifest into the icons the legacy stages verify and the adopted ones.
pub(super) fn split(manifest: &SetManifest) -> (SetManifest, Vec<Value>) {
    let (legacy, adopted) = split_doc(&manifest.doc);
    (SetManifest { doc: legacy }, adopted)
}

/// [`split`] on a parsed `set.json`: the document without its adopted icons, and those icons.
pub fn split_doc(doc: &Value) -> (Value, Vec<Value>) {
    let mut legacy = doc.clone();
    let mut adopted = Vec::new();
    if let Some(icons) = legacy.get_mut("icons").and_then(Value::as_array_mut) {
        let (kept, taken): (Vec<Value>, Vec<Value>) = icons.drain(..).partition(|icon| !is_adopted(icon));
        *icons = kept;
        adopted = taken;
    }
    (legacy, adopted)
}

/// The paint and format knobs for adopted actions: the template's.
const VARIANT: ActionVariant = ActionVariant {
    builder: Builder::None,
    definitions: false,
    sources: false,
    ellipsis: true,
    every_stroke_width: false,
    normalize_ks: true,
    nodes: NodeChecks::Guarded(Hold::Always),
};

/// Checks every adopted icon; returns the evidence entry (one item per icon).
pub(super) fn check(set: &SetContext<'_>, icons: &[Value]) -> Result<Value, VerifyError> {
    let mut out = Vec::with_capacity(icons.len());
    for icon in icons {
        out.push(check_icon(set, Record(icon))?);
    }
    Ok(Value::Array(out))
}

fn check_icon(set: &SetContext<'_>, icon: Record<'_>) -> Result<Value, VerifyError> {
    let id = icon
        .str("icon")
        .ok_or_else(|| VerifyError::Assertion("an adopted set.json record has no icon".into()))?;
    let spec_path = set.paths.set_dir.join(SPECS_DIR).join(format!("{id}.icon.json"));
    let spec_sha = set.hash(&spec_path)?;
    let spec = read_json(&spec_path)?;
    ensure(spec.get("id").and_then(Value::as_str) == Some(id), || {
        format!("{SPECS_DIR}/{id}.icon.json: id is not {id}")
    })?;

    let animations = icon.list("animations")?;
    let motions = icon.list("motions")?;
    let names: Vec<&str> = animations.iter().map(Record::name).collect::<Result<_, _>>()?;
    let want_forward = [id.to_string(), format!("{id}-strong")];
    let forward: Vec<&str> = motions.iter().map(Record::name).collect::<Result<_, _>>()?;
    ensure(forward == want_forward.iter().map(String::as_str).collect::<Vec<_>>(), || {
        format!("{id}: adopted motions are {forward:?}, not {want_forward:?}")
    })?;
    let stateful = icon.flag("stateful");
    for motion in &motions {
        let has_reverse = motion.get("reverse").is_some_and(Value::is_object);
        ensure(has_reverse == stateful, || {
            format!("{}: a stateful icon pairs each motion with its reverse, and only then", motion.name().unwrap_or(id))
        })?;
    }

    for a in &animations {
        let name = a.name()?;
        ensure(is_adopted(a.0) && a.str("icon") == Some(id), || {
            format!("{name}: an adopted icon's actions all come from its spec")
        })?;
        for key in ["sourceSha256", "definitionsSha256"] {
            ensure(a.str(key) == Some(spec_sha.as_str()), || {
                format!("{name} {key}: stale; {SPECS_DIR}/{id}.icon.json changed since it was adopted")
            })?;
        }
        ensure(
            a.str("sha256") == Some(set.hash(&set.paths.assets_dir.join(format!("{name}.lottie")))?.as_str()),
            || format!("{name}.lottie"),
        )?;
        let data = read_json(&set.paths.assets_dir.join(format!("{name}.json")))?;
        check_format(&data, *a)?;
        check_paint(&without_matte_sources(&data, name)?, *a, &VARIANT)?;
        check_themes(set, &data, name)?;
        let markers = array(data.get("markers"), "data.markers")?;
        ensure(
            markers
                .iter()
                .any(|m| m.get("cm") == Some(&json!("rest")) && strict_equals(m.get("tm"), a.get("rest"))),
            || format!("{name}: no rest marker at the recorded rest frame"),
        )?;
        ensure(markers.iter().any(|m| m.get("cm") == Some(&json!("action"))), || {
            format!("{name}: no action marker")
        })?;
    }
    Ok(json!({ "icon": id, "spec": format!("{SPECS_DIR}/{id}.icon.json"), "specSha256": spec_sha, "actions": names }))
}

/// The animation without its matte sources. Spec icons cut Solid knockouts that move,
/// draw on or fade with track mattes: a `<part>-matte` layer marked `td: 1`, used as the
/// alpha of the layer after it, is never drawn, so its paint carries no slot.
fn without_matte_sources(data: &Value, name: &str) -> Result<Value, VerifyError> {
    let layers = array(data.get("layers"), "data.layers")?;
    let mut kept = Vec::with_capacity(layers.len());
    for (i, layer) in layers.iter().enumerate() {
        if layer.get("td").and_then(Value::as_u64) != Some(1) {
            kept.push(layer.clone());
            continue;
        }
        let named = layer.get("nm").and_then(Value::as_str).is_some_and(|nm| nm.ends_with("-matte"));
        let used = layers.get(i + 1).and_then(|next| next.get("tt")).and_then(Value::as_u64).is_some_and(|tt| tt > 0);
        ensure(named && used, || format!("{name}: layer {i} is a matte source (td 1) that no layer uses"))?;
    }
    let mut out = data.clone();
    out["layers"] = Value::Array(kept);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_order_and_separates_adopted_icons() {
        let manifest = SetManifest {
            doc: json!({"id": "x", "icons": [
                {"icon": "a"}, {"icon": "b", "source": SOURCE}, {"icon": "c"}
            ]}),
        };
        let (legacy, adopted) = split(&manifest);
        let ids: Vec<&str> = legacy.doc["icons"].as_array().unwrap().iter().map(|i| i["icon"].as_str().unwrap()).collect();
        assert_eq!(ids, ["a", "c"]);
        assert_eq!(adopted, vec![json!({"icon": "b", "source": SOURCE})]);
        assert_eq!(manifest.doc["icons"].as_array().unwrap().len(), 3, "the full manifest is untouched");
    }

    #[test]
    fn matte_sources_leave_the_paint_check_only_when_a_layer_uses_them() {
        let data = json!({"layers": [
            {"nm": "a-solid-matte", "td": 1, "shapes": [{"ty": "fl"}]},
            {"nm": "a-solid", "tt": 1, "shapes": []},
            {"nm": "b", "shapes": []}
        ]});
        let kept = without_matte_sources(&data, "a").unwrap();
        let names: Vec<&str> = kept["layers"].as_array().unwrap().iter().map(|l| l["nm"].as_str().unwrap()).collect();
        assert_eq!(names, ["a-solid", "b"]);
        let orphan = json!({"layers": [{"nm": "a-solid-matte", "td": 1, "shapes": []}, {"nm": "b", "shapes": []}]});
        assert!(without_matte_sources(&orphan, "a").is_err());
    }
}
