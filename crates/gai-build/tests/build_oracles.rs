//! T3–T6 (builder spec §10.2): post-export, records and `set.json` against the committed
//! outputs, with packaging finished by `gai_package` exactly as `package-set.py` did.
//!
//! - [`Mode::ReuseNative`] reads each set's committed `native/*.json` and writes into a
//!   temporary folder: assets and themes (T3, T4) must equal `prototype/icons/NN/assets`,
//!   and after packaging the manifests (T5), `.lottie` files, `set.json` and
//!   `evidence/build.json` (T6) must too.
//! - [`Mode::Full`] with a replay exporter that answers each scene with its committed native
//!   also checks the scenes, `.engine.js` files and provenance it writes on the way.
//!
//! Natives and scenes are gitignored (a snapshot is in `.local/parity-oracles/`); tests skip
//! when they are missing. `GAI_LIBRARY_ROOT` points at another checkout. Nothing is written
//! inside the repository. The whole catalog is `#[ignore]`d:
//! `cargo test --release -p gai-build --test build_oracles -- --ignored --nocapture`.

use gai_build::build::{BuildOptions, Layout, Mode, Target, build};
use gai_core::export::{Exporter, NativeExport};
use gai_js::{Limits, map_parallel};
use gai_package::zip::ZipDate;
use gai_package::{ActionList, build_set_archives, encode_set_json};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn repo() -> PathBuf {
    std::env::var_os("GAI_LIBRARY_ROOT")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."), PathBuf::from)
}

fn set_dir(set: &str) -> PathBuf {
    repo().join("motion/icon-sets").join(set)
}

/// Sets with definitions whose committed natives are present, in name order.
fn native_sets() -> Vec<String> {
    let Ok(entries) = fs::read_dir(repo().join("motion/icon-sets")) else { return Vec::new() };
    let mut sets: Vec<String> = entries
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .filter(|name| set_dir(name).join("definitions.mjs").is_file() && set_dir(name).join("native").is_dir())
        .collect();
    sets.sort();
    sets
}

/// Answers each scene with the committed native and provenance, after checking that the
/// scene is the committed one.
struct Replay {
    native: PathBuf,
    scenes: PathBuf,
    scene_mismatches: AtomicUsize,
}

impl Exporter for Replay {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
        if fs::read(self.scenes.join(format!("{name}.scene.json")))? != scene_json.as_bytes() {
            self.scene_mismatches.fetch_add(1, Ordering::Relaxed);
        }
        let text = fs::read_to_string(self.native.join(format!("{name}.json")))?;
        let provenance = fs::read(self.native.join(format!("{name}.json.provenance.json")))?;
        Ok(NativeExport { text, provenance: serde_json::from_slice(&provenance)? })
    }
}

/// Byte comparison of one category of outputs.
#[derive(Debug, Default)]
struct Tally {
    identical: usize,
    different: Vec<String>,
}

impl Tally {
    fn check(&mut self, label: &str, built: &[u8], oracle: &Path) {
        match fs::read(oracle) {
            Ok(expected) if expected == built => self.identical += 1,
            _ => self.different.push(label.to_owned()),
        }
    }
}

#[derive(Debug, Default)]
struct Outcome {
    animations: usize,
    lines: usize,
    tallies: BTreeMap<&'static str, Tally>,
    elapsed: Duration,
}

impl Outcome {
    fn tally(&mut self, category: &'static str) -> &mut Tally {
        self.tallies.entry(category).or_default()
    }
}

/// Builds `set` into a temporary folder in `mode`, packages the result and compares it.
fn check_set(set: &str, full: bool) -> Outcome {
    let started = Instant::now();
    let dir = set_dir(set);
    let temp = tempfile::tempdir().expect("temporary folder");
    let mut target = Target::library_set(&dir).unwrap_or_else(|error| panic!("{set}: {error}"));
    let oracle_assets = target.layout.out.clone();
    let oracle_native = target.layout.native.clone();
    let oracle_scenes = target.layout.scenes.clone();
    // Reuse-native only reads the committed natives; a full build writes its own.
    target.layout = Layout {
        scenes: temp.path().join("scenes"),
        native: if full { temp.path().join("native") } else { oracle_native.clone() },
        out: temp.path().join("assets"),
        ..target.layout
    };
    let replay =
        Replay { native: oracle_native.clone(), scenes: oracle_scenes.clone(), scene_mismatches: AtomicUsize::new(0) };
    let lines = AtomicUsize::new(0);
    let progress = |_: &gai_build::build::AnimationReport| {
        lines.fetch_add(1, Ordering::Relaxed);
    };
    let options = BuildOptions { jobs: 3, progress: Some(&progress), ..BuildOptions::default() };
    let mode = if full { Mode::Full(&replay) } else { Mode::ReuseNative };
    let report = build(&target, mode, &options).unwrap_or_else(|error| panic!("{set}: {error}"));

    let mut outcome = Outcome { animations: report.animations.len(), lines: lines.into_inner(), ..Outcome::default() };
    let assets = &target.layout.out;
    for animation in &report.animations {
        let name = &animation.name;
        let read = |path: PathBuf| fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let file = format!("{name}.json");
        outcome.tally("assets").check(name, &read(assets.join(&file)), &oracle_assets.join(&file));
        for theme in ["dark", "light"] {
            let file = format!("{name}.theme-{theme}.json");
            outcome.tally("themes").check(&file, &read(assets.join(&file)), &oracle_assets.join(&file));
        }
        if full {
            for (category, file) in
                [("scenes", format!("{name}.scene.json")), ("engine.js", format!("{name}.engine.js"))]
            {
                outcome.tally(category).check(
                    &file,
                    &read(target.layout.scenes.join(&file)),
                    &oracle_scenes.join(&file),
                );
            }
            for (category, file) in [("natives", file.clone()), ("provenance", format!("{file}.provenance.json"))] {
                outcome.tally(category).check(
                    &file,
                    &read(target.layout.native.join(&file)),
                    &oracle_native.join(&file),
                );
            }
        }
    }
    if full {
        let mismatches = replay.scene_mismatches.load(Ordering::Relaxed);
        assert_eq!(mismatches, 0, "{set}: {mismatches} scenes sent to the exporter differ from the oracle");
    }

    // Packaging: what package-set.py (and 03–06's package.py) did with these outputs.
    let text = fs::read(assets.join("set.json")).expect("set.json written");
    let mut record: Value = serde_json::from_slice(&text).expect("set.json is JSON");
    let date = if &set[..2] < "06" { ZipDate::LEGACY_SETS } else { ZipDate::SETS };
    let archives = build_set_archives(&mut record, assets, ActionList::Animations, date)
        .unwrap_or_else(|error| panic!("{set}: packaging: {error}"));
    for archive in &archives {
        let file = format!("{}.manifest.json", archive.name);
        outcome.tally("manifests").check(&file, archive.manifest.as_bytes(), &oracle_assets.join(&file));
        let file = format!("{}.lottie", archive.name);
        outcome.tally("lottie").check(&file, &archive.lottie, &oracle_assets.join(&file));
    }
    let encoded = encode_set_json(&record);
    outcome.tally("set.json").check("set.json", encoded.as_bytes(), &oracle_assets.join("set.json"));
    outcome.tally("build.json").check("build.json", encoded.as_bytes(), &dir.join("evidence/build.json"));
    outcome.elapsed = started.elapsed();
    outcome
}

fn check_sets(sets: &[String], full: bool) -> Vec<(String, Outcome)> {
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let outcomes = map_parallel(sets.to_vec(), workers, &Limits::default(), |set| {
        let outcome = check_set(&set, full);
        (set, outcome)
    });
    for (set, outcome) in &outcomes {
        assert_eq!(outcome.lines, outcome.animations, "{set}: one progress line per animation");
        for (category, tally) in &outcome.tallies {
            assert!(tally.different.is_empty(), "{set}: {category} differ: {:?}", tally.different);
        }
    }
    outcomes
}

fn summary(outcomes: &[(String, Outcome)]) -> String {
    let mut totals: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, outcome) in outcomes {
        for (category, tally) in &outcome.tallies {
            *totals.entry(category).or_default() += tally.identical;
        }
    }
    let animations: usize = outcomes.iter().map(|(_, outcome)| outcome.animations).sum();
    let busy: Duration = outcomes.iter().map(|(_, outcome)| outcome.elapsed).sum();
    format!(
        "{} sets, {animations} animations; byte-identical: {totals:?}; summed per-set time {busy:.1?}",
        outcomes.len()
    )
}

/// Sets covering each builder family: 03 (hard-coded reverse names, no duotoneFill), 04
/// (directed), 11 (shared builder, patched category), 12 (continuous), 23 and 34 (canonical).
const SUBSET: [&str; 6] =
    ["03-essentials", "04-actions", "11-feedback-status", "12-loading-indicators", "23-get-help", "34-scheduling-time"];

fn subset() -> Vec<String> {
    native_sets().into_iter().filter(|set| SUBSET.contains(&set.as_str())).collect()
}

#[test]
fn reuse_native_subset_matches_the_committed_outputs() {
    let sets = subset();
    if sets.is_empty() {
        eprintln!("skipped: no committed natives under {}", repo().display());
        return;
    }
    let outcomes = check_sets(&sets, false);
    eprintln!("{}", summary(&outcomes));
}

#[test]
fn full_build_with_replayed_exports_matches_the_committed_outputs() {
    let sets: Vec<String> = subset().into_iter().filter(|set| set == "23-get-help").collect();
    if sets.is_empty() || !set_dir("23-get-help").join("scenes").is_dir() {
        eprintln!("skipped: no scene and native oracles under {}", repo().display());
        return;
    }
    let outcomes = check_sets(&sets, true);
    for category in ["scenes", "engine.js", "natives", "provenance"] {
        assert!(outcomes[0].1.tallies[category].identical > 0, "{category} checked");
    }
    eprintln!("{}", summary(&outcomes));
}

/// Largest sets first, so the parallel run ends together.
fn catalog() -> Vec<String> {
    let mut sets = native_sets();
    let slow = ["12-loading-indicators", "22-teamwork", "46-files-folders", "62-vector-editing", "07-ai-conversation"];
    sets.sort_by_key(|set| !slow.contains(&set.as_str()));
    sets
}

#[test]
#[ignore = "whole catalog: run with --release -- --ignored"]
fn every_set_reuse_native() {
    let sets = catalog();
    if sets.is_empty() {
        eprintln!("skipped: no committed natives under {}", repo().display());
        return;
    }
    let started = Instant::now();
    let outcomes = check_sets(&sets, false);
    eprintln!("reuse-native: {}; wall {:.1?}", summary(&outcomes), started.elapsed());
    assert_eq!(outcomes.len(), 56);
}

#[test]
#[ignore = "whole catalog: run with --release -- --ignored"]
fn every_set_full_with_replayed_exports() {
    let sets = catalog();
    if sets.is_empty() || !set_dir("34-scheduling-time").join("scenes").is_dir() {
        eprintln!("skipped: no scene and native oracles under {}", repo().display());
        return;
    }
    let started = Instant::now();
    let outcomes = check_sets(&sets, true);
    eprintln!("full (replayed exports): {}; wall {:.1?}", summary(&outcomes), started.elapsed());
    assert_eq!(outcomes.len(), 56);
    // A set whose icons are all adopted from specs builds nothing and has no tally.
    assert!(outcomes
        .iter()
        .all(|(_, outcome)| outcome.tallies.get("engine.js").map_or(0, |t| t.identical) == outcome.animations));
}
