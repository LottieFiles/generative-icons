//! Set verification against the committed evidence: every file `verify` writes must be
//! byte-identical to `motion/icon-sets/<set>/evidence/*`. The inputs (`scenes/`, `native/`)
//! are gitignored; tests skip when they are missing. Nothing is written to the repository.

use gai_check::verify::{SetPaths, VerifyOptions, verify_set};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn set_dir(id: &str) -> PathBuf {
    repo().join("motion/icon-sets").join(id)
}

/// Whether a set's gitignored intermediates are present.
fn has_intermediates(id: &str) -> bool {
    let dir = set_dir(id);
    ["scenes", "native"]
        .iter()
        .all(|d| std::fs::read_dir(dir.join(d)).is_ok_and(|mut e| e.next().is_some()))
}

/// Verifies a set and compares every produced evidence file with the committed one.
/// Returns the produced paths.
fn check(id: &str) -> Vec<String> {
    let paths = SetPaths::catalog(set_dir(id));
    let outcome = verify_set(&paths, &VerifyOptions::default());
    if let Some(error) = &outcome.error {
        panic!("{id}: {error}");
    }
    let dir = set_dir(id);
    let mut produced = Vec::new();
    for e in &outcome.evidence {
        let committed = std::fs::read_to_string(dir.join(&e.path))
            .unwrap_or_else(|err| panic!("{id}/{}: {err}", e.path));
        if committed != e.contents {
            let line = committed
                .lines()
                .zip(e.contents.lines())
                .position(|(a, b)| a != b)
                .unwrap_or(0);
            panic!(
                "{id}/{} differs at line {}:\n  committed: {:?}\n  produced:  {:?}",
                e.path,
                line + 1,
                committed.lines().nth(line),
                e.contents.lines().nth(line)
            );
        }
        produced.push(format!("{id}/{}", e.path));
    }
    produced
}

/// The committed evidence files `verify` reproduces, by set (verify.md §10.2).
fn expected(id: &str) -> Vec<&'static str> {
    let n: u32 = id[..2].parse().unwrap();
    let mut files = Vec::new();
    if n >= 3 {
        files.push("evidence/directed-source-verification.json");
    }
    if (12..=39).contains(&n) || [42, 43, 44, 45, 49, 58, 61].contains(&n) {
        files.push("evidence/clearance-verification.json");
    }
    match n {
        10 => files.push("evidence/approval-clearance-verification.json"),
        11 => files.push("evidence/attachment-verification.json"),
        62 => files.push("evidence/geometry-verification.json"),
        _ => {}
    }
    files
}

fn catalog_sets() -> Vec<String> {
    let mut sets: Vec<String> = std::fs::read_dir(repo().join("motion/icon-sets"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .collect();
    sets.sort();
    sets
}

/// One set of each variant and rule style: legacy 01, verify-directed with its one-off (11),
/// continuous (12), template with contract clearance (35), rule-major tail (50), style
/// dimension (53) and the geometry file written before the directed report (62).
#[test]
fn representative_sets_match_committed_evidence() {
    for id in [
        "01-heart-bell",
        "11-feedback-status",
        "12-loading-indicators",
        "35-weather",
        "50-preferences",
        "53-video-editing",
        "62-vector-editing",
    ] {
        if !has_intermediates(id) {
            eprintln!("skipping {id}: no scenes/native intermediates");
            continue;
        }
        let mut produced = check(id);
        produced.sort();
        let mut want: Vec<String> = expected(id).iter().map(|f| format!("{id}/{f}")).collect();
        want.sort();
        assert_eq!(produced, want, "{id}");
    }
}

/// Every set of the catalog: all 94 per-set evidence files (the paint audit is the 95th).
#[test]
#[ignore = "whole catalog; run with --ignored"]
fn whole_catalog_matches_committed_evidence() {
    let sets = catalog_sets();
    let missing: Vec<&String> = sets.iter().filter(|s| !has_intermediates(s)).collect();
    if !missing.is_empty() {
        eprintln!("skipping: no intermediates for {missing:?}");
        return;
    }
    let started = Instant::now();
    let mut total = 0;
    for id in &sets {
        let t = Instant::now();
        let produced = check(id);
        assert_eq!(
            produced.len(),
            expected(id).len(),
            "{id}: produced {produced:?}"
        );
        total += produced.len();
        eprintln!(
            "{id}: {} files identical ({:.1} s)",
            produced.len(),
            t.elapsed().as_secs_f64()
        );
    }
    eprintln!(
        "{} sets, {total} evidence files identical in {:.1} s",
        sets.len(),
        started.elapsed().as_secs_f64()
    );
    assert_eq!(sets.len(), 58);
    assert_eq!(total, 94);
}
