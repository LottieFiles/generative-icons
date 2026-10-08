//! `export` against the built library: library icons become animated SVGs with the
//! default trigger for their kind, files are written without following symlinks, and
//! options that don't fit the icon are usage errors. Needs this checkout's built packages.

mod common;

use common::*;
use serde_json::Value;

#[test]
fn library_icons_export_with_the_trigger_their_kind_needs() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let library = dist.to_str().unwrap();
    let catalog: Value = serde_json::from_str(&read(&dist, "catalog.json")).unwrap();
    let icons = catalog["icons"].as_array().unwrap();
    let stateful = icons.iter().find(|icon| icon["stateful"] == true).unwrap()["id"].as_str().unwrap();
    let dir = tempfile::tempdir().unwrap();

    let bell = run(dir.path(), &["export", "bell", "--library", library]).ok();
    assert_eq!((bell["trigger"].as_str(), bell["source"].as_str()), (Some("hover"), Some("library asset")));
    let svg = bell["svg"].as_str().unwrap();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\""), "{svg}");
    assert!(svg.contains("begin=\"bell_h.mouseenter\"") && svg.contains("stroke=\"currentColor\""));
    assert_eq!(bell["bytes"].as_u64().unwrap() as usize, svg.len());

    let toggle = run(dir.path(), &["export", stateful, "--library", library]).ok();
    assert_eq!(toggle["trigger"], "toggle");
    let wrong = run(dir.path(), &["export", "bell", "--trigger", "toggle", "--library", library]);
    wrong.fails(2, "USAGE");
    run(dir.path(), &["export", "bell", "--weight", "3.5", "--library", library]).fails(2, "USAGE");
    run(dir.path(), &["export", "no-such-icon", "--library", library]).fails(3, "UNKNOWN_ICON");
}

#[cfg(unix)]
#[test]
fn out_replaces_a_planted_symlink_instead_of_writing_through_it() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim.txt");
    std::fs::write(&victim, "keep me").unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir_all(app.join("icons")).unwrap();
    std::os::unix::fs::symlink(&victim, app.join("icons/bell.svg")).unwrap();

    let args = ["export", "bell", "--style", "solid", "--out", "icons/bell.svg", "--library", dist.to_str().unwrap()];
    let report = run(&app, &args).ok();
    assert_eq!(report["path"], "icons/bell.svg");
    assert!(report.get("svg").is_none());
    assert_eq!(std::fs::read_to_string(&victim).unwrap(), "keep me");
    let written = app.join("icons/bell.svg");
    assert!(!std::fs::symlink_metadata(&written).unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read(&written).unwrap().len() as u64, report["bytes"].as_u64().unwrap());
}
