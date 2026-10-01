//! Normalization against the committed-format natives (gitignored oracles: these tests skip
//! when `motion/icon-sets/*/native/` is absent).

mod common;

use std::path::{Path, PathBuf};

use common::{
    HOSTED_FIXTURES, hosted_fixture, library_root, native_oracle, with_hosted_path_names,
};
use gai_engine::normalize_native;
use serde_json::{Map, Value, json};

/// The top-level key order of a hosted export.
const HOSTED_ORDER: [&str; 9] = ["v", "fr", "ip", "op", "w", "h", "nm", "layers", "assets"];

/// Rebuilds the hosted form of a native file: each layer's shapes inside one group with an
/// identity transform, keys in the engine's order, written compactly by serde_json.
fn wrap_as_hosted(native: &str) -> String {
    let mut value: Value = serde_json::from_str(native).unwrap();
    for layer in value["layers"].as_array_mut().unwrap() {
        let mut items = layer["shapes"].as_array().unwrap().clone();
        items.push(json!({
            "ty": "tr", "a": {"a": 0, "k": [0.0, 0.0]}, "p": {"a": 0, "k": [0.0, 0.0]},
            "s": {"a": 0, "k": [100.0, 100.0]}, "r": {"a": 0, "k": 0.0}, "o": {"a": 0, "k": 100.0},
            "sk": {"a": 0, "k": 0.0}, "sa": {"a": 0, "k": 0.0},
        }));
        layer["shapes"] = json!([{"ty": "gr", "it": items}]);
    }
    let mut object = value.as_object().unwrap().clone();
    let mut hosted = Map::new();
    for key in HOSTED_ORDER {
        if let Some(v) = object.remove(key) {
            hosted.insert(key.to_owned(), v);
        }
    }
    hosted.extend(object);
    serde_json::to_string(&Value::Object(hosted)).unwrap()
}

fn natives(sets: Option<&[&str]>) -> Vec<PathBuf> {
    let Some(root) = library_root() else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(root.join("motion/icon-sets"))
        .min_depth(3)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| {
            let in_native = p
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|d| d == "native");
            let set = p
                .parent()
                .and_then(Path::parent)
                .and_then(Path::file_name)
                .and_then(|s| s.to_str());
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            in_native
                && name.ends_with(".json")
                && !name.ends_with(".provenance.json")
                && sets.is_none_or(|sets| set.is_some_and(|s| sets.contains(&s)))
        })
        .collect();
    files.sort();
    files
}

/// Normalizing the hosted form of each native must give back its exact bytes, so the unwrap,
/// the key sort and serde_json's number writer (`60.0`, `-0.0`, `-9.536743e-6`) all match.
fn round_trip(files: &[PathBuf]) -> usize {
    let mut failures = Vec::new();
    for file in files {
        let native = std::fs::read_to_string(file).unwrap();
        let normalized = normalize_native(&wrap_as_hosted(&native)).unwrap().text;
        if normalized != native {
            failures.push(file.display().to_string());
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} differ: {:?}",
        failures.len(),
        files.len(),
        &failures[..failures.len().min(10)]
    );
    files.len()
}

#[test]
fn committed_natives_round_trip_through_the_hosted_form() {
    let files = natives(Some(&["06-tools", "11-feedback-status"]));
    if files.is_empty() {
        eprintln!("skipped: native oracles are not present");
        return;
    }
    let count = round_trip(&files);
    eprintln!("{count} natives round-trip byte-identically");
}

#[test]
#[ignore = "whole catalog; run with --ignored"]
fn every_committed_native_round_trips_through_the_hosted_form() {
    let files = natives(None);
    if files.is_empty() {
        eprintln!("skipped: native oracles are not present");
        return;
    }
    let count = round_trip(&files);
    eprintln!("{count} natives round-trip byte-identically");
}

/// Real creator-mcp 0.2.0 exports: after normalization they equal the committed natives
/// byte for byte, apart from each path's `nm`.
#[test]
fn hosted_exports_match_the_committed_natives_except_path_names() {
    for (set, name) in HOSTED_FIXTURES {
        let normalized = normalize_native(&hosted_fixture(name)).unwrap();
        let text = &normalized.text;
        assert!(
            !text.contains("\"ty\":\"gr\"") && !text.contains("\"ty\":\"tr\""),
            "{name}"
        );
        assert!(!text.ends_with('\n'));
        let Some(native) = native_oracle(set, name) else {
            eprintln!("skipped oracle comparison for {set}/{name}: native not present");
            continue;
        };
        assert_eq!(*text, with_hosted_path_names(&native, text), "{set}/{name}");
    }
}
