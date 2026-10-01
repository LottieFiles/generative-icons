//! Helpers shared by the parity tests.

use gai_package::ActionList;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The library checkout this crate lives in, if its catalog is present.
pub fn repo() -> Option<PathBuf> {
    let root = std::env::var_os("GAI_LIBRARY_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let root = root.canonicalize().ok()?;
    root.join("prototype/icons/catalog.json")
        .exists()
        .then_some(root)
}

/// The set ids of the catalog, in catalog order (`motion/icon-sets/<id>`).
pub fn catalog_set_ids(repo: &Path) -> Vec<String> {
    let catalog = gai_package::Catalog::load(repo).unwrap();
    catalog
        .sets
        .iter()
        .map(|relative| set_id_of(repo, relative))
        .collect()
}

fn set_id_of(repo: &Path, relative: &str) -> String {
    let record: Value = serde_json::from_slice(
        &std::fs::read(repo.join("prototype/icons").join(relative)).unwrap(),
    )
    .unwrap();
    record["id"].as_str().unwrap().to_string()
}

/// How the committed record of `set_id` lists its actions.
pub fn action_list(set_id: &str) -> ActionList {
    if matches!(set_id, "01-heart-bell" | "02-search") {
        ActionList::Motions
    } else {
        ActionList::Animations
    }
}

/// The `.lottie` date of `set_id`: 2026-09-06 for 01–05, otherwise 2026-09-08.
pub fn lottie_date(set_id: &str) -> gai_package::zip::ZipDate {
    if &set_id[..2] < "06" {
        gai_package::zip::ZipDate::LEGACY_SETS
    } else {
        gai_package::zip::ZipDate::SETS
    }
}

/// The committed `set.json` with every packager-added `bytes`/`sha256` removed. They are
/// always the trailing keys, so this is the record as the build step left it.
pub fn strip_hashes(record: &mut Value) {
    let strip = |v: &mut Value| {
        if let Some(map) = v.as_object_mut() {
            map.shift_remove("bytes");
            map.shift_remove("sha256");
        }
    };
    for icon in record["icons"].as_array_mut().unwrap() {
        for key in ["animations", "motions"] {
            for item in icon
                .get_mut(key)
                .and_then(Value::as_array_mut)
                .into_iter()
                .flatten()
            {
                strip(item);
                if let Some(reverse) = item.get_mut("reverse") {
                    strip(reverse);
                }
            }
        }
        strip(icon);
    }
}

/// Copies `from` into `to` recursively (following symlinks), leaving out `.zip` files:
/// they are packaging outputs, never inputs.
pub fn copy_tree(from: &Path, to: &Path) {
    if from.is_dir() {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        }
    } else if from.extension().is_none_or(|e| e != "zip") {
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(from, to).unwrap();
    }
}

/// Copies each repo-relative path that exists from `repo` into `fixture`.
pub fn copy_parts(repo: &Path, fixture: &Path, parts: &[String]) {
    for part in parts {
        let from = repo.join(part);
        if from.exists() {
            copy_tree(&from, &fixture.join(part));
        }
    }
}

/// Every repo-relative path a set's packaging reads: the set folder, the shared library,
/// the gallery files, docs, notices and the workflow skill.
pub fn packaging_inputs(set_ids: &[String]) -> Vec<String> {
    let mut parts: Vec<String> = [
        "LICENSE",
        "THIRD_PARTY_NOTICES.md",
        "docs",
        ".agents",
        "motion/icon-sets/library",
        "motion/src",
        "motion/package.json",
        "motion/package-lock.json",
        "prototype/icons/catalog.json",
        "prototype/icons/01",
        "prototype/icons/03",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for id in set_ids {
        parts.push(format!("motion/icon-sets/{id}"));
        parts.push(format!("prototype/icons/{}", &id[..2]));
    }
    parts.sort();
    parts.dedup();
    parts
}

/// Runs a command and panics with its stderr when it fails.
pub fn run(command: &mut std::process::Command) -> String {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Whether `program --version` runs.
pub fn available(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}
