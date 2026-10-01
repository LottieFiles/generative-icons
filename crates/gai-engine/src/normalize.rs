//! Turns the hosted export into the native form the builder expects (builder spec §5).
//!
//! The hosted script puts each part in a composition-level group; the exporter hoists the
//! group's transform and opacity into the layer, leaving an identity `ty:"tr"`. Unwrapping that
//! group, sorting keys recursively and writing compact JSON with serde_json's float writer
//! reproduces the old local exporter byte for byte, except the `nm` of each `ty:"sh"` shape
//! (the node name here, `native-export-<id>` before).

use serde_json::{Map, Value};

use crate::error::{Error, Result};

/// A validated, normalized export.
#[derive(Debug, Clone)]
pub struct NativeLottie {
    /// Compact JSON with sorted keys and no trailing newline: the bytes of `native/<name>.json`.
    pub text: String,
    /// `w`.
    pub width: Value,
    /// `h`.
    pub height: Value,
    /// `fr`.
    pub frame_rate: f64,
    /// `(op - ip) / fr`, in seconds.
    pub duration: f64,
    /// Number of layers.
    pub layers: usize,
}

/// Normalizes hosted Lottie text; see the module docs. Already-flat layers pass through, so
/// normalizing a committed native reproduces it.
pub fn normalize_native(hosted: &str) -> Result<NativeLottie> {
    let mut root: Value =
        serde_json::from_str(hosted).map_err(|e| Error::InvalidExport(format!("not JSON: {e}")))?;
    let object = root
        .as_object_mut()
        .ok_or_else(|| Error::InvalidExport("the top level is not an object".into()))?;
    let (frame_rate, duration) = timing(object)?;
    let layers = object
        .get_mut("layers")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| Error::InvalidExport("`layers` is not an array".into()))?;
    for (index, layer) in layers.iter_mut().enumerate() {
        unwrap_part_group(layer)
            .map_err(|why| Error::InvalidExport(format!("layer {index}: {why}")))?;
    }
    let layers = layers.len();
    let width = object.get("w").cloned().unwrap_or(Value::Null);
    let height = object.get("h").cloned().unwrap_or(Value::Null);
    root.sort_all_objects();
    let text = serde_json::to_string(&root).map_err(|e| Error::InvalidExport(e.to_string()))?;
    Ok(NativeLottie {
        text,
        width,
        height,
        frame_rate,
        duration,
        layers,
    })
}

/// Checks `fr > 0` and `op > ip`; returns the frame rate and the duration in seconds.
fn timing(root: &Map<String, Value>) -> Result<(f64, f64)> {
    let number = |key: &str| {
        root.get(key)
            .and_then(Value::as_f64)
            .ok_or_else(|| Error::InvalidExport(format!("`{key}` is not a number")))
    };
    let (fr, ip, op) = (number("fr")?, number("ip")?, number("op")?);
    if fr <= 0.0 || op <= ip {
        return Err(Error::InvalidExport(format!(
            "bad timing: fr {fr}, ip {ip}, op {op}"
        )));
    }
    Ok((fr, (op - ip) / fr))
}

/// Replaces a layer's single `ty:"gr"` shape by the group's items, minus its identity `ty:"tr"`.
fn unwrap_part_group(layer: &mut Value) -> std::result::Result<(), String> {
    let Some(shapes) = layer.get_mut("shapes").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    let [group] = shapes.as_mut_slice() else {
        return Ok(());
    };
    if group.get("ty").and_then(Value::as_str) != Some("gr") {
        return Ok(());
    }
    let items = match group.get_mut("it").map(Value::take) {
        Some(Value::Array(items)) => items,
        _ => return Err("group without an `it` array".into()),
    };
    let mut flat = Vec::with_capacity(items.len());
    for item in items {
        if item.get("ty").and_then(Value::as_str) == Some("tr") {
            if !is_identity_transform(&item) {
                return Err(format!("the part group carries a transform: {item}"));
            }
        } else {
            flat.push(item);
        }
    }
    *shapes = flat;
    Ok(())
}

/// `a [0,0]`, `p [0,0]`, `s [100,100]`, `r 0`, `o 100`, `sk 0`, `sa 0`, all static; a missing
/// property is the identity too.
fn is_identity_transform(tr: &Value) -> bool {
    let identity: [(&str, &[f64]); 7] = [
        ("a", &[0.0, 0.0]),
        ("p", &[0.0, 0.0]),
        ("s", &[100.0, 100.0]),
        ("r", &[0.0]),
        ("o", &[100.0]),
        ("sk", &[0.0]),
        ("sa", &[0.0]),
    ];
    identity.iter().all(|(key, expected)| match tr.get(*key) {
        None => true,
        Some(prop) => {
            let animated = prop.get("a").and_then(Value::as_f64).unwrap_or(0.0) != 0.0;
            let values: Option<Vec<f64>> = match prop.get("k") {
                Some(Value::Array(items)) => items.iter().map(Value::as_f64).collect(),
                Some(single) => single.as_f64().map(|v| vec![v]),
                None => None,
            };
            !animated && values.as_deref() == Some(*expected)
        }
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const IDENTITY: &str = r#"{"ty":"tr","p":{"a":0,"k":[0.0,0.0]},"a":{"a":0,"k":[0.0,0.0]},"s":{"a":0,"k":[100.0,100.0]},"r":{"a":0,"k":0.0},"o":{"a":0,"k":100.0},"sk":{"a":0,"k":0.0},"sa":{"a":0,"k":0.0}}"#;

    fn hosted(tr: &str) -> String {
        format!(
            r#"{{"v":"5.7.0","fr":60.0,"ip":0.0,"op":144.0,"w":32,"h":32,"nm":"x","layers":[{{"ty":4,"nm":"part","ind":1,"shapes":[{{"ty":"gr","it":[{{"ty":"sh","nm":"part","ks":{{"a":0,"k":{{"c":false,"v":[[-0.0,-9.536743e-6]]}}}}}},{{"ty":"st","w":{{"a":0,"k":2.0}},"o":{{"a":0,"k":0.8666667}}}},{tr}]}}],"ks":{{}}}}],"assets":[]}}"#
        )
    }

    #[test]
    fn unwraps_sorts_and_keeps_number_spelling() {
        let native = normalize_native(&hosted(IDENTITY)).unwrap();
        assert_eq!(
            native.text,
            r#"{"assets":[],"fr":60.0,"h":32,"ip":0.0,"layers":[{"ind":1,"ks":{},"nm":"part","shapes":[{"ks":{"a":0,"k":{"c":false,"v":[[-0.0,-9.536743e-6]]}},"nm":"part","ty":"sh"},{"o":{"a":0,"k":0.8666667},"ty":"st","w":{"a":0,"k":2.0}}],"ty":4}],"nm":"x","op":144.0,"v":"5.7.0","w":32}"#
        );
        assert_eq!(
            (native.frame_rate, native.duration, native.layers),
            (60.0, 2.4, 1)
        );
        assert_eq!(
            (native.width.clone(), native.height.clone()),
            (json!(32), json!(32))
        );
        // Idempotent: a normalized file passes through unchanged.
        assert_eq!(normalize_native(&native.text).unwrap().text, native.text);
    }

    #[test]
    fn rejects_a_group_transform_that_is_not_the_identity() {
        let moved = IDENTITY.replace(
            r#""p":{"a":0,"k":[0.0,0.0]}"#,
            r#""p":{"a":0,"k":[1.0,0.0]}"#,
        );
        let err = normalize_native(&hosted(&moved)).unwrap_err();
        assert!(err.to_string().contains("carries a transform"), "{err}");
        let keyed = IDENTITY.replace(r#""r":{"a":0,"k":0.0}"#, r#""r":{"a":1,"k":[]}"#);
        assert!(normalize_native(&hosted(&keyed)).is_err());
    }

    #[test]
    fn validates_the_document() {
        for bad in [
            r#"[]"#,
            r#"{"fr":60,"ip":0,"op":10}"#,
            r#"{"fr":0,"ip":0,"op":10,"layers":[]}"#,
            r#"{"fr":60,"ip":10,"op":10,"layers":[]}"#,
            r#"{"fr":60,"ip":0,"op":10,"layers":{}}"#,
            "not json",
        ] {
            assert!(
                matches!(normalize_native(bad), Err(Error::InvalidExport(_))),
                "{bad}"
            );
        }
    }
}
