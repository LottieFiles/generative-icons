//! The embedded kit is all a user project needs: a library set copied into a user project
//! whose `library/` holds only the kit describes and samples exactly as it does in the
//! library. Skips when `motion/icon-sets` is missing.

use gai_js::{
    Description, Limits, ModernProfile, SetRuntime, SetSource, VariantSamples, kit, map_parallel, run_on_worker,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn icon_sets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets")
}

/// Every `.mjs` file below `from`, copied to the same relative path below `to`.
fn copy_modules(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_modules(&path, &target);
        } else if path.extension().is_some_and(|ext| ext == "mjs") {
            fs::create_dir_all(to).unwrap();
            fs::copy(&path, target).unwrap();
        }
    }
}

/// `set` as a user project: `icons/` holds its modules, `library/` exactly the kit.
fn user_project(set: &str) -> TempDir {
    let project = tempfile::tempdir().unwrap();
    copy_modules(&icon_sets().join(set), &project.path().join("icons"));
    fs::create_dir(project.path().join("library")).unwrap();
    for file in kit::files() {
        fs::write(project.path().join("library").join(file.name), file.text).unwrap();
    }
    project
}

/// The description and the first icon's samples.
fn evaluate(source: &SetSource) -> (Description, Vec<VariantSamples>) {
    let mut runtime = SetRuntime::load(source, &Limits::default()).unwrap_or_else(|error| panic!("{error}"));
    let description = runtime.install().unwrap();
    let samples = (0..description.icons[0].variants.len()).map(|j| runtime.sample_variant(0, j).unwrap()).collect();
    (description, samples)
}

/// Whether `set` evaluates identically as a user project.
fn same_as_user_project(set: &str) -> bool {
    let project = user_project(set);
    evaluate(&SetSource::library_set(icon_sets().join(set)).unwrap())
        == evaluate(&SetSource::user_project(project.path().join("icons")))
}

#[test]
fn files_hash_their_text() {
    let names: Vec<&str> = kit::files().iter().map(|file| file.name).collect();
    assert_eq!(names, ["ai-authoring.mjs", "direction.mjs", "drawing.mjs", "family-contract.json"]);
    for file in kit::files() {
        assert_eq!(file.sha256, gai_core::sha256_hex(file.text), "{}", file.name);
    }
}

#[test]
fn a_library_set_runs_unchanged_on_the_kit() {
    if icon_sets().join("63-voice-agents").is_dir() {
        assert!(run_on_worker(&Limits::default(), || same_as_user_project("63-voice-agents")));
    }
}

#[test]
#[ignore = "whole catalog: run with --release -- --ignored"]
fn every_shared_set_runs_unchanged_on_the_kit() {
    let mut sets: Vec<String> = fs::read_dir(icon_sets())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| {
            matches!(
                SetSource::library_set(icon_sets().join(name)),
                Ok(SetSource::Definitions { profile: ModernProfile::SHARED, .. })
            ) && icon_sets().join(name).join("definitions.mjs").is_file()
        })
        .collect();
    sets.sort();
    assert!(sets.len() >= 50, "{} sets", sets.len());
    let workers = std::thread::available_parallelism().map_or(4, usize::from);
    let same = map_parallel(sets.clone(), workers, &Limits::default(), |set| same_as_user_project(&set));
    let differ: Vec<&String> = sets.iter().zip(same).filter(|(_, same)| !same).map(|(set, _)| set).collect();
    assert!(differ.is_empty(), "evaluate differently as user projects: {differ:?}");
}
