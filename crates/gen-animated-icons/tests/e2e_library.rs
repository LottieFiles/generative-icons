//! Maintainer commands end to end on a temporary copy of this checkout: `library build
//! --reuse-native` (preflight, build, package), `library package` and `library verify`
//! must regenerate every committed asset and evidence file byte for byte. Needs the
//! gitignored natives and scenes (restore them from
//! `.local/parity-oracles/scenes-native-098688f.tgz`); skips without them.

mod common;

use common::*;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Outputs a rebuild must reproduce: committed assets and gate evidence. Review sheets
/// and set zips are gitignored products, not oracles.
fn is_oracle(path: &Path) -> bool {
    let name = path.to_string_lossy();
    !(name.ends_with(".png") || name.ends_with(".zip") || name.ends_with(".webp"))
}

/// Evidence the build, package and verify stages write. The browser evidence is
/// `verify-visual`'s; delivery, bundle and id checks belong to other tools.
fn is_gate_evidence(name: &str) -> bool {
    matches!(
        name,
        "build.json"
            | "directed-source-verification.json"
            | "clearance-verification.json"
            | "geometry-verification.json"
            | "attachment-verification.json"
            | "approval-clearance-verification.json"
    )
}

fn has_oracles(repo: &Path, set: &str) -> bool {
    let dir = repo.join("motion/icon-sets").join(set);
    dir.join("native").is_dir() && dir.join("scenes").is_dir()
}

/// Removes the outputs of `set` from the copy: each action's asset, manifest, themes
/// and `.lottie` that set.json names, and the gate evidence. set.json stays, since the
/// preflight catalog reads it, but is dated 1970. Files no action names (set 50's stale
/// `do-not-disturb*`) stay, as the builders never delete them. Returns what was removed.
fn clear_outputs(copy: &Path, set: &str) -> Vec<PathBuf> {
    let assets = copy.join("prototype/icons").join(&set[..2]).join("assets");
    let evidence = copy.join("motion/icon-sets").join(set).join("evidence");
    let manifest: Value = serde_json::from_slice(&std::fs::read(assets.join("set.json")).unwrap()).unwrap();
    std::fs::File::options().write(true).open(assets.join("set.json")).unwrap().set_modified(UNIX_EPOCH).unwrap();
    let mut removed = Vec::new();
    for icon in manifest["icons"].as_array().unwrap() {
        let actions = icon.get("animations").or_else(|| icon.get("motions")).and_then(Value::as_array).unwrap();
        for name in actions.iter().map(|action| action["name"].as_str().unwrap()) {
            for suffix in [".json", ".lottie", ".manifest.json", ".theme-dark.json", ".theme-light.json"] {
                let file = assets.join(format!("{name}{suffix}"));
                std::fs::remove_file(&file).unwrap();
                removed.push(file);
            }
        }
    }
    for file in files_below(&evidence) {
        if file.parent() == Some(Path::new("")) && is_gate_evidence(&file.to_string_lossy()) {
            std::fs::remove_file(evidence.join(&file)).unwrap();
            removed.push(evidence.join(&file));
        }
    }
    removed
}

/// Asserts every oracle file of `set` in `repo` is byte-identical in `copy`; returns the
/// number compared.
fn assert_identical(repo: &Path, copy: &Path, set: &str) -> usize {
    let mut compared = 0;
    for folder in [
        PathBuf::from("prototype/icons").join(&set[..2]).join("assets"),
        PathBuf::from("motion/icon-sets").join(set).join("evidence"),
    ] {
        for file in files_below(&repo.join(&folder)).into_iter().filter(|file| is_oracle(file)) {
            let (original, rebuilt) = (repo.join(&folder).join(&file), copy.join(&folder).join(&file));
            let rebuilt_bytes = std::fs::read(&rebuilt).unwrap_or_else(|e| panic!("{}: {e}", rebuilt.display()));
            assert!(std::fs::read(&original).unwrap() == rebuilt_bytes, "{} differs", folder.join(&file).display());
            compared += 1;
        }
    }
    let set_json = copy.join("prototype/icons").join(&set[..2]).join("assets/set.json");
    assert!(std::fs::metadata(set_json).unwrap().modified().unwrap() > UNIX_EPOCH, "{set}: set.json was not rewritten");
    compared
}

/// Rebuilds, packages and verifies `sets` in one copy of the checkout that holds only
/// those sets, or with `whole_checkout` every set (frozen 01 and 02 included), which
/// one `library build --all` rebuilds several at a time.
fn rebuild(sets: &[&str], whole_checkout: bool) -> Option<(PathBuf, tempfile::TempDir)> {
    let repo = repo_root();
    if let Some(missing) = sets.iter().find(|set| !has_oracles(&repo, set)) {
        eprintln!("skipped: {missing} has no scenes/native oracles");
        return None;
    }
    let dir = tempfile::tempdir().unwrap();
    copy_checkout(&repo, dir.path(), (!whole_checkout).then_some(sets));
    let root = dir.path().display().to_string();
    let removed: Vec<Vec<PathBuf>> = sets
        .iter()
        .map(|set| {
            let removed = clear_outputs(dir.path(), set);
            assert!(!removed.is_empty());
            removed
        })
        .collect();
    let started = SystemTime::now();
    let built: Vec<Value> = match whole_checkout {
        true => {
            let built = run(dir.path(), &["--library-root", &root, "library", "build", "--all", "--reuse-native"]).ok();
            built["sets"].as_array().unwrap().clone()
        }
        false => sets
            .iter()
            .map(|set| {
                let built = run(dir.path(), &["--library-root", &root, "library", "build", set, "--reuse-native"]).ok();
                built["sets"][0].clone()
            })
            .collect(),
    };
    eprintln!("library build: {} sets in {:.1} s", sets.len(), started.elapsed().unwrap().as_secs_f64());
    assert_eq!(built.len(), sets.len());
    for ((set, removed), report) in sets.iter().zip(&removed).zip(&built) {
        assert_eq!(report["set"], *set, "reports keep set order");
        assert_eq!((report["status"].as_str(), report["built"].as_bool()), (Some("PASS"), Some(true)), "{report}");
        let manifest: Value = serde_json::from_slice(
            &std::fs::read(repo.join("prototype/icons").join(&set[..2]).join("assets/set.json")).unwrap(),
        )
        .unwrap();
        let animations: usize =
            manifest["icons"].as_array().unwrap().iter().map(|icon| icon["animations"].as_array().unwrap().len()).sum();
        assert_eq!(report["actions"].as_array().unwrap().len(), animations, "{set}");
        run(dir.path(), &["--library-root", &root, "library", "package", set]).ok();
        let verified = run(dir.path(), &["--library-root", &root, "library", "verify", set]).ok();
        assert_eq!(verified["sets"][0]["status"], "PASS", "{verified}");
        for file in removed {
            assert!(file.is_file(), "{} was not regenerated", file.display());
        }
        let compared = assert_identical(&repo, dir.path(), set);
        eprintln!("{set}: {} outputs regenerated, {compared} files identical", removed.len());
    }
    eprintln!("{} sets in {:.1} s", sets.len(), started.elapsed().unwrap().as_secs_f64());
    Some((repo, dir))
}

#[test]
fn modern_continuous_and_legacy_sets_rebuild_byte_for_byte() {
    let Some((repo, dir)) = rebuild(&["63-voice-agents", "12-loading-indicators", "04-actions"], false) else { return };
    let root = dir.path().display().to_string();

    // Registering a registered set leaves the catalog byte for byte, mode included.
    let catalog = dir.path().join("prototype/icons/catalog.json");
    let before = std::fs::read(&catalog).unwrap();
    let mode = std::fs::metadata(&catalog).unwrap().permissions();
    let registered = run(dir.path(), &["--library-root", &root, "library", "register", "63-voice-agents"]).ok();
    assert_eq!(registered["catalogChanged"], false);
    assert_eq!(registered["category"]["label"], "Voice agents");
    assert_eq!(std::fs::read(&catalog).unwrap(), before);
    assert_eq!(std::fs::read(repo.join("prototype/icons/catalog.json")).unwrap(), before);
    assert_eq!(std::fs::metadata(&catalog).unwrap().permissions(), mode);

    // Preflight output is the preflight.mjs report.
    let preflight = run(dir.path(), &["--library-root", &root, "library", "preflight", "04"]).ok();
    assert_eq!(
        (preflight["status"].as_str(), preflight["reports"][0]["set"].as_str()),
        (Some("PASS"), Some("04-actions"))
    );
    // Frozen sets (01 and 02 keep their authoring in build.mjs) cannot be built.
    std::fs::create_dir_all(dir.path().join("motion/icon-sets/01-heart-bell")).unwrap();
    let frozen = run(dir.path(), &["--library-root", &root, "library", "build", "01", "--reuse-native"]);
    assert_eq!(
        frozen.fails(2, "USAGE")["message"],
        "01-heart-bell has no definitions.mjs; frozen sets are verified as they are, never rebuilt"
    );
}

#[test]
#[ignore = "whole catalog; run with --release -- --ignored"]
fn every_set_rebuilds_byte_for_byte() {
    let repo = repo_root();
    let mut sets: Vec<String> = std::fs::read_dir(repo.join("motion/icon-sets"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.as_bytes()[0].is_ascii_digit()
                && repo.join("motion/icon-sets").join(name).join("definitions.mjs").is_file()
        })
        .collect();
    sets.sort();
    let sets: Vec<&str> = sets.iter().map(String::as_str).collect();
    let Some((_, dir)) = rebuild(&sets, true) else { return };
    let root = dir.path().display().to_string();
    let paint = dir.path().join("motion/icon-sets/library/evidence/paint-verification.json");
    std::fs::remove_file(&paint).unwrap();
    let all = run(dir.path(), &["--library-root", &root, "library", "verify", "--all"]).ok();
    assert_eq!(all["status"], "PASS");
    let verified: Vec<&Value> = all["sets"].as_array().unwrap().iter().collect();
    assert_eq!(verified.len(), 58);
    assert_eq!(
        std::fs::read(&paint).unwrap(),
        std::fs::read(repo.join("motion/icon-sets/library/evidence/paint-verification.json")).unwrap()
    );
}

#[test]
fn visual_checks_run_each_action_in_a_child_process() {
    let repo = repo_root();
    let set = "63-voice-agents";
    let committed = repo.join("motion/icon-sets").join(set).join("evidence/browser-verification.json");
    require!(committed.is_file().then_some(()), "set 63 browser evidence");
    let dir = tempfile::tempdir().unwrap();
    copy_checkout(&repo, dir.path(), Some(&[set]));
    let root = dir.path().display().to_string();
    let result = run(
        dir.path(),
        &["--library-root", &root, "library", "verify-visual", "63", "--icons", "turn-taking", "--jobs", "4"],
    )
    .ok();
    assert_eq!((result["result"].as_str(), result["actions"].as_u64()), (Some("PASS"), Some(4)), "{result}");
    // The four re-checked records join the other 56 kept from the committed report.
    assert_eq!(result["assets"], 60);

    // Each record equals the browser's, apart from the frame events a real display
    // clock produced.
    let records = |path: &Path| -> Vec<Value> {
        let report: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut assets = report["assets"].as_array().unwrap().clone();
        for asset in &mut assets {
            asset["playback"].as_object_mut().unwrap().remove("observedFrameEvents");
        }
        assets
    };
    let rewritten = dir.path().join("motion/icon-sets").join(set).join("evidence/browser-verification.json");
    assert_eq!(records(&rewritten), records(&committed));
}
