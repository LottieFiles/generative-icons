//! T-scenes and T2: scenes and the `.engine.js` template against the old builders'
//! outputs in `motion/icon-sets/*/scenes/` (gitignored; a snapshot is in
//! `.local/parity-oracles/`). Tests skip when the oracle files are missing. Set
//! `GAI_ICON_SETS` to read another copy of `motion/icon-sets`.
//!
//! The whole catalog is `#[ignore]`d: `cargo test --release -p gai-build --test
//! scene_oracles -- --ignored --nocapture`.

use gai_build::{scene, template};
use gai_js::{Limits, SetSource, map_parallel};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

fn icon_sets() -> PathBuf {
    std::env::var_os("GAI_ICON_SETS")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets"), PathBuf::from)
}

/// Set folders (`NN-slug`) that have oracle scenes, in name order.
fn oracle_sets() -> Vec<String> {
    let Ok(entries) = fs::read_dir(icon_sets()) else { return Vec::new() };
    let mut sets: Vec<String> = entries
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .filter(|name| icon_sets().join(name).join("scenes").is_dir())
        .collect();
    sets.sort();
    sets
}

#[derive(Debug, Default)]
struct Outcome {
    identical: usize,
    different: Vec<String>,
    /// Built scenes without an oracle file.
    missing: Vec<String>,
    /// Oracle files no current definition produces (stale).
    stale: Vec<String>,
    elapsed: Duration,
}

fn check_set(set: &str, limits: &Limits) -> Outcome {
    let started = Instant::now();
    let dir = icon_sets().join(set);
    let source = SetSource::library_set(&dir).expect("set folder");
    let built = scene::set_scenes(&source, limits).unwrap_or_else(|error| panic!("{set}: {error}"));
    let mut outcome = Outcome::default();
    for scene in &built.scenes {
        match fs::read(dir.join("scenes").join(format!("{}.scene.json", scene.name))) {
            Ok(oracle) if oracle == scene.json.as_bytes() => outcome.identical += 1,
            Ok(oracle) => {
                let at = oracle.iter().zip(scene.json.as_bytes()).position(|(a, b)| a != b).unwrap_or(oracle.len());
                let context = |bytes: &[u8]| {
                    String::from_utf8_lossy(&bytes[at.saturating_sub(40)..(at + 40).min(bytes.len())]).into_owned()
                };
                eprintln!(
                    "{set}/{}: differs at byte {at}\n  oracle {}\n  built  {}",
                    scene.name,
                    context(&oracle),
                    context(scene.json.as_bytes())
                );
                outcome.different.push(scene.name.clone());
            }
            Err(_) => outcome.missing.push(scene.name.clone()),
        }
    }
    if let Ok(entries) = fs::read_dir(dir.join("scenes")) {
        for entry in entries.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            if let Some(name) = file.strip_suffix(".scene.json")
                && !built.scenes.iter().any(|scene| scene.name == name)
            {
                outcome.stale.push(name.to_owned());
            }
        }
    }
    outcome.stale.sort();
    outcome.elapsed = started.elapsed();
    outcome
}

fn check_sets(sets: &[String]) -> Vec<(String, Outcome)> {
    let limits = Limits::default();
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let outcomes = map_parallel(sets.to_vec(), workers, &limits, |set| {
        let outcome = check_set(&set, &limits);
        (set, outcome)
    });
    for (set, outcome) in &outcomes {
        assert!(
            outcome.different.is_empty(),
            "{set}: {} scenes differ: {:?}",
            outcome.different.len(),
            outcome.different
        );
        assert!(outcome.missing.is_empty(), "{set}: no oracle for {:?}", outcome.missing);
    }
    outcomes
}

/// A few sets that cover each builder family: both legacy shims, 03's hardcoded reverse
/// names and rigid transforms, and a shared-builder set.
#[test]
fn subset_matches_the_oracles() {
    let wanted = ["01-heart-bell", "02-search", "03-essentials", "23-get-help"];
    let sets: Vec<String> = oracle_sets().into_iter().filter(|set| wanted.contains(&set.as_str())).collect();
    if sets.is_empty() {
        eprintln!("skipped: no scene oracles under {}", icon_sets().display());
        return;
    }
    let outcomes = check_sets(&sets);
    let identical: usize = outcomes.iter().map(|(_, outcome)| outcome.identical).sum();
    assert!(identical > 0);
    for (set, outcome) in &outcomes {
        assert!(outcome.stale.is_empty(), "{set}: stale oracles {:?}", outcome.stale);
    }
}

#[test]
#[ignore = "whole catalog: run with --release -- --ignored"]
fn every_scene_matches_its_oracle() {
    let mut sets = oracle_sets();
    if sets.is_empty() {
        eprintln!("skipped: no scene oracles under {}", icon_sets().display());
        return;
    }
    // Slowest sets first, so the parallel run ends together.
    let slow = ["46-files-folders", "62-vector-editing", "22-teamwork", "07-ai-conversation", "10-ai-agents"];
    sets.sort_by_key(|set| !slow.contains(&set.as_str()));
    let started = Instant::now();
    let outcomes = check_sets(&sets);
    let wall = started.elapsed();
    let identical: usize = outcomes.iter().map(|(_, outcome)| outcome.identical).sum();
    let busy: Duration = outcomes.iter().map(|(_, outcome)| outcome.elapsed).sum();
    let stale: Vec<String> = outcomes
        .iter()
        .flat_map(|(set, outcome)| outcome.stale.iter().map(move |name| format!("{set}/{name}")))
        .collect();
    eprintln!(
        "{identical} scenes byte-identical across {} sets; wall {wall:.1?}, summed per-set time {busy:.1?}; stale oracles {stale:?}",
        sets.len()
    );
    // The only stale oracles are the four do-not-disturb scenes set 50 no longer defines.
    let expected_stale: Vec<String> =
        ["do-not-disturb", "do-not-disturb-reverse", "do-not-disturb-reverse-strong", "do-not-disturb-strong"]
            .iter()
            .map(|name| format!("50-preferences/{name}"))
            .collect();
    assert_eq!(stale, expected_stale);
}

/// T2: every `.engine.js` the old builders wrote for sets 03–63 is the embedded template.
#[test]
fn engine_scripts_match_the_template() {
    let mut checked = 0;
    for set in oracle_sets().iter().filter(|set| !set.starts_with("01-") && !set.starts_with("02-")) {
        let Ok(entries) = fs::read_dir(icon_sets().join(set).join("scenes")) else { continue };
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(".engine.js") {
                let script = fs::read(entry.path()).expect("read engine script");
                assert!(script == template::ENGINE_SCRIPT, "{}", entry.path().display());
                checked += 1;
            }
        }
    }
    eprintln!("{checked} engine scripts match the template");
}
