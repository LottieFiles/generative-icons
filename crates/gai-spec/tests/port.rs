//! Porting v1 library icons (their Light animation at frame 0) to specs: every icon not
//! yet reanimated ports to a spec that compiles, with no motion yet.

use gai_spec::port::{Icon, port};
use serde_json::Value;
use std::path::Path;

#[test]
fn every_v1_library_icon_ports_to_a_spec_that_compiles() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let catalog: Value = serde_json::from_slice(&std::fs::read(repo.join("prototype/icons/catalog.json")).unwrap()).unwrap();
    let (mut ported, mut failed) = (0, Vec::new());
    for relative in catalog["sets"].as_array().unwrap().iter().filter_map(Value::as_str) {
        let manifest = repo.join("prototype/icons").join(relative);
        let set: Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
        let specs = repo.join("motion/icon-sets").join(set["id"].as_str().unwrap()).join("specs");
        for entry in set["icons"].as_array().unwrap() {
            let id = entry.get("icon").or(entry.get("name")).and_then(Value::as_str).unwrap();
            if specs.join(format!("{id}.icon.json")).exists() {
                continue;
            }
            let light = entry["motions"].as_array().unwrap().iter().find(|m| m["motionId"] == "light").unwrap();
            let file = manifest.parent().unwrap().join(format!("{}.json", light["name"].as_str().unwrap()));
            let lottie: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
            let state = entry["stateful"].as_bool().unwrap_or(false).then(|| {
                let s = |k: &str| entry[k].as_str().unwrap_or("rest").to_string();
                (s("startState"), s("endState"), ["Go".to_string(), "Back".to_string()])
            });
            let icon = Icon {
                id: id.to_string(),
                label: entry["label"].as_str().unwrap_or(id).to_string(),
                intent: entry["intent"].as_str().unwrap_or("x").to_string(),
                state,
                preview_reset: entry["previewReset"].as_bool().unwrap_or(false),
            };
            // The port has no motion yet; one fade of a visible part lets every part compile.
            let result = port(&lottie, &icon).map_err(|e| vec![e]).and_then(|p| {
                let mut spec = p.spec;
                let (part, rest) = spec["parts"].as_object().unwrap().iter().find(|(_, v)| v.get("hidden").is_none()).map(|(k, v)| (k.clone(), v["opacity"].as_f64().unwrap_or(1.0))).unwrap();
                spec["motion"] = serde_json::json!({format!("{part}.opacity"): {"light": [[0, rest], [0.2, 0.6 * rest], [0.4, rest]], "strong": [[0, rest], [0.2, 0.4 * rest], [0.4, rest]]}});
                gai_spec::load(&spec.to_string()).map(|_| ()).map_err(|d| d.errors().map(|e| format!("{} {}: {}", e.code, e.path, e.message)).collect())
            });
            match result {
                Ok(()) => ported += 1,
                Err(errors) => failed.push(format!("{id}: {errors:?}")),
            }
        }
    }
    assert!(failed.is_empty(), "{} failed:\n{}", failed.len(), failed.iter().take(80).cloned().collect::<Vec<_>>().join("\n"));
    assert!(ported > 500, "{ported} ported");
}
