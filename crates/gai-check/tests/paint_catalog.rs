//! The paint audit against the committed `paint-verification.json`. Its inputs are the
//! committed packaged assets, so these tests only skip outside a full checkout.

use gai_check::paint::{EVIDENCE_FILE, PaintContract, audit, audit_catalog};
use gai_check::verify::js::deep_equal;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn committed() -> Option<Value> {
    let text = std::fs::read_to_string(repo().join(EVIDENCE_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Legacy motions-only records (01), the filled ellipsis (25) and open icons with duotone
/// metadata (12) reproduce their committed records in order.
#[test]
fn sample_sets_match_committed_records() {
    let Some(committed) = committed() else {
        eprintln!("skipping: no committed paint evidence");
        return;
    };
    let icons = repo().join("prototype/icons");
    let sets: Vec<PathBuf> = ["01", "12", "25"]
        .iter()
        .map(|n| icons.join(n).join("assets/set.json"))
        .collect();
    let report = audit(&sets, PaintContract::builtin()).unwrap();
    let records = committed["records"].as_array().unwrap();
    let produced = report.report["records"].as_array().unwrap();
    assert!(!produced.is_empty());
    for record in produced {
        assert!(records.contains(record), "{record}");
    }
    assert!(deep_equal(
        &report.report["openIcons"],
        &committed["openIcons"]
    ));
    assert!(deep_equal(&report.report["weights"], &committed["weights"]));
}

/// A tint on an open icon fails the audit.
#[test]
fn open_icons_must_not_be_tinted() {
    let icons = repo().join("prototype/icons");
    let set = icons.join("12/assets/set.json");
    if !set.is_file() {
        eprintln!("skipping: no packaged set 12");
        return;
    }
    let mut contract = PaintContract::builtin().clone();
    // Every icon of set 12 is open; claiming a tinted icon is open must fail.
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&set).unwrap()).unwrap();
    let tinted = doc["icons"].as_array().unwrap().iter().find(|i| {
        i["animations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["duotoneFill"] == true)
    });
    let Some(tinted) = tinted else {
        eprintln!("skipping: set 12 has no tinted icon");
        return;
    };
    contract
        .open_icons
        .push(tinted["name"].as_str().unwrap().to_string());
    assert!(audit(&[set], &contract).is_err());
}

/// The whole catalog, in catalog order: byte-identical evidence.
#[test]
#[ignore = "whole catalog; run with --ignored"]
fn whole_catalog_matches_committed_evidence() {
    let Ok(expected) = std::fs::read_to_string(repo().join(EVIDENCE_FILE)) else {
        eprintln!("skipping: no committed paint evidence");
        return;
    };
    let report = audit_catalog(&repo().join("prototype/icons"), PaintContract::builtin()).unwrap();
    eprintln!("{}", report.summary());
    assert_eq!(
        (report.icons, report.actions, report.stroke_paints),
        (1005, 3094, 35718)
    );
    assert!(
        report.to_json() == expected,
        "paint-verification.json differs"
    );
}
