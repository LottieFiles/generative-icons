//! The source checks `check` runs before a build find nothing in library sets that pass
//! set verification: sets 16 onward, which use the template checks project sets get.
//! Needs this checkout's `motion/icon-sets`.

mod common;

use common::*;
use gai_build::build::Target;
use gai_build::scene::build_icon_scenes;
use gai_js::{Limits, SetRuntime, run_on_worker};
use generative_icons::scene_checks::check_icon;
use std::path::Path;

/// Every finding for the icons of the library set in `dir`, as `icon: message`.
fn findings(dir: &Path) -> Vec<String> {
    let target = Target::library_set(dir).unwrap();
    let (source, continuous, limits) = (target.source(), target.profile.continuous, Limits::default());
    run_on_worker(&limits.clone(), move || {
        let mut runtime = SetRuntime::load(&source, &limits).unwrap();
        let description = runtime.install().unwrap();
        let mut found = Vec::new();
        for (index, icon) in description.icons.iter().enumerate() {
            let scenes = build_icon_scenes(&runtime, &description, index).unwrap();
            for finding in check_icon(&description, index, &scenes, continuous).unwrap() {
                found.push(format!("{}: {}", icon.name, finding.message));
            }
        }
        found
    })
}

/// Template sets (16 onward) with definitions, sorted.
fn template_sets() -> Option<Vec<std::path::PathBuf>> {
    let sets = repo_root().join("motion/icon-sets");
    let mut dirs: Vec<_> = std::fs::read_dir(&sets)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|dir| dir.join("definitions.mjs").is_file())
        .filter(|dir| {
            let name = dir.file_name().unwrap().to_string_lossy().into_owned();
            name.get(..2).and_then(|n| n.parse::<u32>().ok()).is_some_and(|n| n >= 16)
        })
        .collect();
    dirs.sort();
    (!dirs.is_empty()).then_some(dirs)
}

#[test]
fn a_verified_set_has_no_findings() {
    let sets = require!(template_sets(), "motion/icon-sets");
    // A stateful set with reverse actions.
    let set = sets.iter().find(|dir| dir.ends_with("34-scheduling-time")).unwrap_or(&sets[0]);
    assert_eq!(findings(set), Vec::<String>::new(), "{}", set.display());
}

#[test]
#[ignore = "whole catalog"]
fn every_verified_set_has_no_findings() {
    let sets = require!(template_sets(), "motion/icon-sets");
    let mut found = Vec::new();
    for set in &sets {
        found.extend(findings(set).into_iter().map(|line| format!("{}: {line}", set.display())));
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
    assert!(sets.len() >= 40, "{} sets", sets.len());
}
