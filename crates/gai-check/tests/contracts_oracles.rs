//! Byte parity of `evidence/clearance-verification.json` with the committed oracles.
//!
//! Scenes are gitignored build intermediates, so each set is skipped when its
//! `scenes/` folder is missing. Mismatching output is written under the test's
//! temp dir for diffing; the repo is never written.

use gai_check::contracts::{Clearance, EVIDENCE_FILE, ScenesDir, SetManifest};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Every set with a clearance stage: 12–39 and 45.
const SETS: &[&str] = &[
    "12-loading-indicators",
    "13-fintech-finance",
    "14-travel",
    "15-commerce-delivery",
    "16-voice",
    "17-media-playback",
    "18-motion-design",
    "19-agent-operations",
    "20-arrows",
    "21-annotation-arrows",
    "22-teamwork",
    "23-get-help",
    "24-file-transfer",
    "25-everyday-controls",
    "26-account-privacy",
    "27-forms-input",
    "28-tables-analytics",
    "29-text-document-editing",
    "30-accessibility-reading",
    "31-developer-version-control",
    "32-infrastructure-devices",
    "33-communication-social",
    "34-scheduling-time",
    "35-weather",
    "36-health-fitness",
    "37-smart-home",
    "38-education",
    "39-food-drink",
    "45-chat-messaging",
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

enum Outcome {
    Skipped,
    Identical,
    Different(PathBuf),
}

/// Runs one set's clearance and compares the evidence bytes with the committed file.
fn check(set: &str) -> Outcome {
    let root = repo();
    let set_dir = root.join("motion/icon-sets").join(set);
    let scenes = set_dir.join("scenes");
    if !scenes.is_dir() {
        return Outcome::Skipped;
    }
    let manifest = SetManifest::read(
        &root
            .join("prototype/icons")
            .join(&set[..2])
            .join("assets/set.json"),
    )
    .unwrap();
    let clearance = Clearance::for_set(&set_dir, &manifest.id)
        .unwrap()
        .unwrap_or_else(|| panic!("{set} has no clearance program"));
    let report = clearance
        .run(&manifest, &ScenesDir(scenes))
        .unwrap_or_else(|e| panic!("{set}: {e}"));
    let expected = std::fs::read_to_string(set_dir.join(EVIDENCE_FILE)).unwrap();
    let actual = report.to_json();
    if actual == expected {
        Outcome::Identical
    } else {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("clearance")
            .join(format!("{set}.json"));
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        std::fs::write(&out, actual).unwrap();
        Outcome::Different(out)
    }
}

fn assert_identical(sets: &[&str]) {
    let mut failures = Vec::new();
    for set in sets {
        let start = Instant::now();
        match check(set) {
            Outcome::Skipped => eprintln!("{set}: skipped (no scenes)"),
            Outcome::Identical => eprintln!(
                "{set}: byte-identical ({:.1}s)",
                start.elapsed().as_secs_f64()
            ),
            Outcome::Different(out) => {
                eprintln!("{set}: DIFFERENT, wrote {}", out.display());
                failures.push(*set);
            }
        }
    }
    assert!(failures.is_empty(), "evidence differs for {failures:?}");
}

/// One quick set per program kind: a v1 contract (duplicate labels, profile and time
/// gates), built-in converted rules, and a frozen legacy check.
#[test]
fn representative_sets_match_oracles() {
    assert_identical(&["33-communication-social", "16-voice", "45-chat-messaging"]);
}

#[test]
#[ignore = "whole catalogue: run with --release -- --ignored"]
fn every_clearance_evidence_matches_oracle() {
    assert_identical(SETS);
}
