//! `preview` against the built library: sheets never write through a planted symlink,
//! and reverse actions render too. Needs this checkout's built packages.

mod common;

use common::*;
use serde_json::Value;

#[cfg(unix)]
#[test]
fn a_symlink_at_a_sheet_path_is_replaced_not_written_through() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim.txt");
    std::fs::write(&victim, "keep me").unwrap();
    let app = dir.path().join("app");
    let sheets = app.join(".gen-animated-icons/previews/heart");
    std::fs::create_dir_all(&sheets).unwrap();
    std::os::unix::fs::symlink(&victim, sheets.join("heart-strong.png")).unwrap();

    let preview = run(&app, &["preview", "heart", "--frames", "2", "--library", dist.to_str().unwrap()]).ok();
    assert_eq!(preview["files"].as_array().unwrap().len(), 2);
    assert_eq!(std::fs::read_to_string(&victim).unwrap(), "keep me");
    let sheet = sheets.join("heart-strong.png");
    assert!(!std::fs::symlink_metadata(&sheet).unwrap().file_type().is_symlink());
    assert_eq!(&std::fs::read(&sheet).unwrap()[..4], b"\x89PNG");

    // A symlinked folder on the way is refused before anything is rendered.
    std::fs::remove_dir_all(&sheets).unwrap();
    std::os::unix::fs::symlink(dir.path(), &sheets).unwrap();
    run(&app, &["preview", "heart", "--library", dist.to_str().unwrap()]).fails(3, "SYMLINK_PATH");
    assert!(!dir.path().join("heart-strong.png").exists());
}

#[test]
fn reverse_actions_of_stateful_icons_render() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let catalog: Value = serde_json::from_str(&read(&dist, "catalog.json")).unwrap();
    let stateful = catalog["icons"].as_array().unwrap().iter().find(|icon| icon["stateful"] == true).unwrap();
    let id = stateful["id"].as_str().unwrap();
    let reverse = stateful["motions"][0]["reverse"]["name"].as_str().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let library = dist.to_str().unwrap();

    let preview = run(dir.path(), &["preview", id, "--motion", "reverse", "--frames", "2", "--library", library]).ok();
    let motions: Vec<&str> =
        preview["files"].as_array().unwrap().iter().map(|file| file["motion"].as_str().unwrap()).collect();
    assert_eq!(motions, ["light-reverse", "strong-reverse"]);
    assert_eq!(preview["files"][0]["animation"], reverse);
    // Reverse actions get their own columns, from the start to their own rest frame.
    let end = |motions: &[&Value]| {
        let end =
            |m: &Value| m["active"].as_f64().unwrap().max(m["rest"].as_f64().unwrap() / m["fps"].as_f64().unwrap());
        (motions.iter().map(|m| end(m)).fold(0.0, f64::max) * 1e5).round() / 1e5
    };
    let forward: Vec<&Value> = stateful["motions"].as_array().unwrap().iter().collect();
    let reverses: Vec<&Value> = forward.iter().map(|m| &m["reverse"]).collect();
    let all = run(dir.path(), &["preview", id, "--motion", "all", "--library", library]).ok();
    let last = |file: &Value| file["times"].as_array().unwrap().last().unwrap().as_f64().unwrap();
    let files = all["files"].as_array().unwrap();
    let ends: Vec<f64> = files.iter().map(last).collect();
    assert_eq!(ends, [end(&forward), end(&forward), end(&reverses), end(&reverses)], "{all}");
    assert_ne!(files[0]["times"], files[2]["times"]);
    let spread = run(dir.path(), &["preview", id, "--motion", "reverse", "--frames", "3", "--library", library]).ok();
    let half = (end(&reverses) / 2.0 * 1e5).round() / 1e5;
    assert_eq!(spread["files"][0]["times"], serde_json::json!([0, half, end(&reverses)]));
    let named = run(dir.path(), &["preview", id, "--animation", reverse, "--library", library]).ok();
    assert_eq!(named["files"][0]["animation"], reverse);
    run(dir.path(), &["preview", "heart", "--motion", "reverse", "--library", library]).fails(6, "INVALID_CATALOG");
}

#[test]
fn a_created_icon_that_is_not_built_says_how_to_build_it() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(&app, &["init"]).ok();
    run(&app, &["create", "watering-can", "--library", &library]).ok();
    // No library is needed to say so.
    let error = run(&app, &["preview", "watering-can", "--offline"]).fails(3, "UNKNOWN_ICON");
    assert_eq!(error["message"], "watering-can is a local icon that is not built yet, so there is nothing to render");
    assert_eq!(error["path"], "gen-animated-icons/icons/batches/watering-can.mjs");
    assert_eq!(
        error["hint"],
        "Run gen-animated-icons check watering-can, then gen-animated-icons build watering-can, then preview it"
    );
    let error = run(&app, &["preview", "no-such-icon", "--library", &library]).fails(3, "UNKNOWN_ICON");
    assert!(error["hint"].as_str().unwrap().contains("gen-animated-icons create no-such-icon"), "{error}");
}

#[test]
fn a_sheet_takes_at_most_64_columns() {
    let dir = tempfile::tempdir().unwrap();
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    let times = |count: usize| (0..count).map(|i| format!("{}", i as f64 / 100.0)).collect::<Vec<_>>().join(",");
    let error = run(dir.path(), &["preview", "heart", "--times", &times(65), "--library", &library]).fails(2, "USAGE");
    assert_eq!(error["message"], "--times takes at most 64 columns");
    let error =
        run(dir.path(), &["preview", "heart", "--times", "0,0.1", "--times", &times(63), "--library", &library]);
    error.fails(2, "USAGE");
    let error = run(dir.path(), &["preview", "heart", "--times", "0,NaN", "--library", &library]).fails(2, "USAGE");
    assert_eq!(error["message"], "--times takes seconds from 0, not NaN");
    let dist = require!(packages_dist(), "packages/icons/dist");
    let args = ["preview", "heart", "--motion", "light", "--times", &times(64), "--cell", "16", "--styles", "solid"];
    let shown = run(dir.path(), &[&args[..], &["--library", dist.to_str().unwrap()]].concat()).ok();
    assert_eq!(shown["files"][0]["times"].as_array().unwrap().len(), 64);
}
