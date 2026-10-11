//! `icon-library.zip` against the committed `delivery.json`, and the asset audit against
//! `scripts/audit-assets.py`.

use crate::common::{available, repo, run};
use gai_package::{audit_assets, package_library, py_pretty};
use serde_json::Value;
use std::process::Command;

#[test]
#[ignore = "whole catalog"]
fn icon_library_matches_delivery_proof() {
    let Some(repo) = repo() else { return };
    let out = tempfile::tempdir().unwrap();
    let proof = package_library(&repo, Some(&out.path().join("icon-library.zip"))).unwrap();
    let committed: Value = serde_json::from_slice(
        &std::fs::read(repo.join("motion/icon-sets/library/evidence/delivery.json")).unwrap(),
    )
    .unwrap();
    println!(
        "icon-library.zip: {} icons, {} actions, {} bytes, {}",
        proof.icons, proof.actions, proof.bytes, proof.sha256
    );
    assert_eq!(proof.path, "icon-library.zip");
    let mut expected = committed;
    expected["path"] = Value::String(proof.path.clone());
    assert_eq!(proof.to_json(), expected);
}

#[test]
#[ignore = "whole catalog"]
fn asset_audit_matches_python() {
    let Some(repo) = repo() else { return };
    let audit = audit_assets(&repo).unwrap();
    assert_eq!((audit.sets, audit.icons, audit.actions), (59, 1161, 3472));
    if available("python3") {
        // audit-assets.py only reads the checkout.
        let printed = run(Command::new("python3").arg(repo.join("scripts/audit-assets.py")));
        assert_eq!(py_pretty(&audit.to_json()) + "\n", printed);
    }
}
