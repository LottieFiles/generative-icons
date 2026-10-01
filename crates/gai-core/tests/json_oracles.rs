//! Re-serializes every committed JSON asset and set.json and requires byte identity
//! with the files that `JSON.stringify` wrote. Skips when run outside the library repo.
//! Manifests and some set.json files were rewritten by Python `json.dumps`.
use gai_core::{to_js_json, to_js_json_pretty, to_py_json};
use std::path::Path;

#[test]
fn committed_json_round_trips_byte_identical() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototype/icons");
    if !assets.exists() {
        return;
    }
    let (mut checked, mut failures) = (0, Vec::new());
    for entry in walkdir::WalkDir::new(&assets).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if !name.ends_with(".json") || !path.to_string_lossy().contains("/assets/") {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        // Python packaging rewrote manifests and some set.json files with json.dumps.
        let candidates = if name == "set.json" {
            vec![to_js_json_pretty(&value, 2) + "\n", to_py_json(&value, Some(2), true) + "\n", to_py_json(&value, Some(2), true)]
        } else if name.ends_with(".manifest.json") {
            vec![to_py_json(&value, None, true), to_js_json(&value)]
        } else {
            vec![to_js_json(&value)]
        };
        checked += 1;
        if !candidates.contains(&text) {
            failures.push(path.display().to_string());
        }
    }
    assert!(checked > 3000, "only {checked} files found");
    assert!(failures.is_empty(), "{} of {checked} differ, e.g. {:?}", failures.len(), &failures[..failures.len().min(5)]);
}
