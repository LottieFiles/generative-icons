//! The catalog and contract preflight reads, and byte parity with Node's `preflight.mjs`
//! (the `npm --prefix motion run icons:preflight` script, run with `node` directly so npm's
//! banner stays out of stdout). The Node comparisons skip when `node` or
//! `motion/node_modules` is missing.

use gai_check::preflight::{Catalog, FamilyContract, Preflight, Status, Target};
use gai_core::sha256_hex;
use serde_json::Value as Json;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime},
};

/// The repo root (`GAI_LIBRARY_ROOT`, else two folders above this crate), canonical as
/// Node's `import.meta.url` is.
fn root() -> PathBuf {
    let root = std::env::var_os("GAI_LIBRARY_ROOT")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."), PathBuf::from);
    fs::canonicalize(root).unwrap()
}

/// `node motion/icon-sets/library/preflight.mjs <arg>`: stdout and exit code.
fn node_preflight(root: &Path, arg: &Path) -> Option<(String, Option<i32>)> {
    if !root.join("motion/node_modules/svgpath").exists() {
        eprintln!("skipped: motion/node_modules is not installed");
        return None;
    }
    let script = root.join("motion/icon-sets/library/preflight.mjs");
    match Command::new("node").arg(script).arg(arg).current_dir(root).output() {
        Ok(output) => Some((String::from_utf8(output.stdout).unwrap(), output.status.code())),
        Err(error) => {
            eprintln!("skipped: cannot run node: {error}");
            None
        }
    }
}

/// `generation contract agrees with every existing catalog icon`.
#[test]
fn contract_agrees_with_every_catalog_icon() {
    let root = root();
    let contract = FamilyContract::load(&root).unwrap();
    let document: Json = serde_json::from_slice(&fs::read(root.join(FamilyContract::FILE)).unwrap()).unwrap();
    let presets: Vec<Json> = contract.outline_weights.iter().map(|(_, width)| Json::from(*width)).collect();
    let catalog = Catalog::load(&root).unwrap();
    assert!(!catalog.icons.is_empty());
    for icon in &catalog.icons {
        let (record, id) = (&icon.record, &icon.id);
        assert_eq!(record["width"].as_f64(), Some(contract.grid), "{id:?}");
        assert_eq!(record["height"].as_f64(), Some(contract.grid), "{id:?}");
        let got: Vec<f64> =
            record["strokeWidth"]["presets"].as_array().unwrap().iter().filter_map(Json::as_f64).collect();
        assert_eq!(got, presets.iter().filter_map(Json::as_f64).collect::<Vec<_>>(), "{id:?}");
        assert_eq!(record["styles"], document["presentationStyles"], "{id:?}");
        assert_eq!(record["fps"].as_f64(), Some(contract.fps), "{id:?}");
        assert_eq!(record["duration"].as_f64(), Some(contract.duration_seconds), "{id:?}");
    }
}

/// `loadCatalog`: sets in `catalog.sets` order (unsorted), `catalog.json` hashed first.
#[test]
fn catalog_keeps_catalog_order_and_hashes_what_it_read() {
    let root = root();
    let catalog = Catalog::load(&root).unwrap();
    let document: Json = serde_json::from_slice(&fs::read(root.join(Catalog::FILE)).unwrap()).unwrap();
    let sets: Vec<&str> = document["sets"].as_array().unwrap().iter().map(|set| set.as_str().unwrap()).collect();
    let paths: Vec<String> = catalog.inputs.iter().map(|input| input.path.clone()).collect();
    let expected: Vec<String> = std::iter::once(Catalog::FILE.to_owned())
        .chain(sets.iter().map(|set| format!("prototype/icons/{set}")))
        .map(|relative| format!("{}/{relative}", root.display()))
        .collect();
    assert_eq!(paths, expected);
    for input in &catalog.inputs {
        assert_eq!(input.sha256, sha256_hex(fs::read(&input.path).unwrap()));
    }
    let heart = catalog.find("heart").unwrap();
    assert_eq!(
        (heart.set.as_ref(), heart.manifest.as_str()),
        (Some(&Json::from("01-heart-bell")), "prototype/icons/01/assets/set.json")
    );
    assert_eq!(heart.record["assetDirectory"], "prototype/icons/01/assets");
    assert!(catalog.find("no-such-icon").is_none());
}

/// A single set's run is byte-identical to Node's.
#[test]
fn one_set_matches_node() {
    let root = root();
    let set = root.join("motion/icon-sets/16-voice");
    let Some((node, code)) = node_preflight(&root, &set) else { return };
    let run = Preflight::open(&root).unwrap().run(&Target::Set(set)).unwrap();
    assert_eq!(run.status(), Status::Pass);
    assert_eq!(code, Some(0));
    assert_eq!(run.stdout(), node);
}

/// `preflight --all` is byte-identical to `node preflight.mjs --all`, and neither run
/// writes under `motion/icon-sets`.
#[test]
#[ignore = "whole catalog: about a minute of Rust and a minute of Node"]
fn all_sets_match_node_and_write_nothing() {
    let root = root();
    let sets = root.join("motion/icon-sets");
    let before = snapshot(&sets);
    let start = Instant::now();
    let run = Preflight::open(&root).unwrap().run(&Target::All).unwrap();
    let rust = start.elapsed();
    let stdout = run.stdout();
    let start = Instant::now();
    let Some((node, code)) = node_preflight(&root, Path::new("--all")) else { return };
    eprintln!("rust {rust:.1?}, node {:.1?}, {} bytes", start.elapsed(), stdout.len());
    assert_eq!(code, Some(if run.status() == Status::Pass { 0 } else { 1 }));
    if stdout != node {
        let at = stdout.bytes().zip(node.bytes()).take_while(|(a, b)| a == b).count();
        panic!(
            "first difference at byte {at}: rust {:?} node {:?}",
            &stdout[at.saturating_sub(200)..(at + 200).min(stdout.len())],
            &node[at.saturating_sub(200)..(at + 200).min(node.len())]
        );
    }
    assert_eq!(snapshot(&sets), before, "a run wrote under motion/icon-sets");
    let skipped: Vec<_> = run.skipped.iter().map(String::as_str).collect();
    assert_eq!(skipped, ["01-heart-bell", "02-search"]);
}

/// Every file below `dir` with its size and modification time.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, (u64, SystemTime)> {
    let mut files = BTreeMap::new();
    let mut pending = vec![dir.to_owned()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.is_dir() {
                pending.push(entry.path());
            } else {
                files.insert(entry.path(), (meta.len(), meta.modified().unwrap()));
            }
        }
    }
    files
}
