//! Every committed `.lottie`, `*.manifest.json`, `set.json` and `evidence/build.json`
//! rebuilds byte-identically from the build outputs, starting from `set.json` without the
//! packager's `bytes`/`sha256`.

use crate::common::{
    action_list, catalog_set_ids, copy_parts, lottie_date, packaging_inputs, repo, strip_hashes,
};
use gai_package::{SetPaths, build_set_archives, encode_set_json, package_set};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Tally {
    lotties: usize,
    manifests: usize,
    set_json: usize,
    build_json: usize,
    failures: Vec<String>,
}

impl Tally {
    fn check(&mut self, ok: bool, count: fn(&mut Tally) -> &mut usize, what: String) {
        if ok {
            *count(self) += 1;
        } else {
            self.failures.push(what);
        }
    }
}

/// Rebuilds one set in memory from the checkout and compares with the committed files.
fn rebuild(repo: &Path, set_id: &str, tally: &mut Tally) {
    let paths = SetPaths::new(repo, set_id).unwrap();
    let committed = std::fs::read_to_string(paths.assets.join("set.json")).unwrap();
    let mut record: Value = serde_json::from_str(&committed).unwrap();
    strip_hashes(&mut record);
    let archives = build_set_archives(
        &mut record,
        &paths.assets,
        action_list(set_id),
        lottie_date(set_id),
    )
    .unwrap();
    for archive in &archives {
        let lottie = std::fs::read(paths.assets.join(format!("{}.lottie", archive.name))).unwrap();
        tally.check(
            lottie == archive.lottie,
            |t| &mut t.lotties,
            format!("{set_id}/{}.lottie", archive.name),
        );
        let manifest =
            std::fs::read_to_string(paths.assets.join(format!("{}.manifest.json", archive.name)))
                .unwrap();
        tally.check(
            manifest == archive.manifest,
            |t| &mut t.manifests,
            format!("{set_id}/{}.manifest.json", archive.name),
        );
    }
    let encoded = encode_set_json(&record);
    tally.check(
        encoded == committed,
        |t| &mut t.set_json,
        format!("{set_id}/set.json"),
    );
    let build = std::fs::read_to_string(paths.root.join("evidence/build.json")).unwrap();
    tally.check(
        encoded == build,
        |t| &mut t.build_json,
        format!("{set_id}/evidence/build.json"),
    );
}

#[test]
fn sample_sets_rebuild_byte_identical() {
    let Some(repo) = repo() else { return };
    let mut tally = Tally::default();
    for set_id in [
        "01-heart-bell",
        "03-essentials",
        "13-fintech-finance",
        "45-chat-messaging",
    ] {
        rebuild(&repo, set_id, &mut tally);
    }
    assert!(tally.failures.is_empty(), "{:?}", tally.failures);
    assert_eq!((tally.set_json, tally.build_json), (4, 4));
}

#[test]
#[ignore = "whole catalog"]
fn whole_catalog_rebuilds_byte_identical() {
    let Some(repo) = repo() else { return };
    let mut tally = Tally::default();
    let sets = catalog_set_ids(&repo);
    for set_id in &sets {
        rebuild(&repo, set_id, &mut tally);
    }
    println!(
        "{} sets: {} .lottie, {} manifests, {} set.json, {} build.json identical; {} failures",
        sets.len(),
        tally.lotties,
        tally.manifests,
        tally.set_json,
        tally.build_json,
        tally.failures.len()
    );
    assert!(
        tally.failures.is_empty(),
        "{:?}",
        &tally.failures[..tally.failures.len().min(20)]
    );
    assert_eq!(tally.lotties, 3472);
}

/// `package_set` on a copy of each set whose `set.json` is the pre-packaging record the
/// builder wrote (JS format, before the 11/13–15 patches), e.g. the builder-probe outputs
/// `<GAI_PREPACKAGE_DIR>/<set-id>.prepackage.set.json`. Sets without one are skipped.
#[test]
#[ignore = "whole catalog; needs GAI_PREPACKAGE_DIR"]
fn prepackage_records_package_to_committed_outputs() {
    let (Some(repo), Some(dir)) = (
        repo(),
        std::env::var_os("GAI_PREPACKAGE_DIR").map(PathBuf::from),
    ) else {
        return;
    };
    let mut tally = Tally::default();
    let mut packaged = 0;
    for set_id in catalog_set_ids(&repo) {
        let prepackage = dir.join(format!("{set_id}.prepackage.set.json"));
        if !prepackage.exists() {
            continue;
        }
        let fixture = tempfile::tempdir().unwrap();
        copy_parts(
            &repo,
            fixture.path(),
            &packaging_inputs(std::slice::from_ref(&set_id)),
        );
        let paths = SetPaths::new(fixture.path(), &set_id).unwrap();
        std::fs::copy(&prepackage, paths.assets.join("set.json")).unwrap();
        std::fs::remove_file(paths.root.join("evidence/build.json")).unwrap();
        let report = package_set(&paths).unwrap();
        packaged += 1;
        let committed = SetPaths::new(&repo, &set_id).unwrap();
        for archive in &report.archives {
            let name = &archive.name;
            let same = |file: String| {
                std::fs::read(paths.assets.join(&file)).unwrap()
                    == std::fs::read(committed.assets.join(&file)).unwrap()
            };
            tally.check(
                same(format!("{name}.lottie")),
                |t| &mut t.lotties,
                format!("{set_id}/{name}.lottie"),
            );
            tally.check(
                same(format!("{name}.manifest.json")),
                |t| &mut t.manifests,
                format!("{set_id}/{name}.manifest.json"),
            );
        }
        let set_json = std::fs::read(committed.assets.join("set.json")).unwrap();
        tally.check(
            std::fs::read(paths.assets.join("set.json")).unwrap() == set_json,
            |t| &mut t.set_json,
            format!("{set_id}/set.json"),
        );
        let build = std::fs::read(paths.root.join("evidence/build.json")).unwrap();
        tally.check(
            build == std::fs::read(committed.root.join("evidence/build.json")).unwrap(),
            |t| &mut t.build_json,
            format!("{set_id}/build.json"),
        );
    }
    println!(
        "{packaged} sets from pre-packaging records: {} .lottie, {} manifests, {} set.json, {} build.json identical; {} failures",
        tally.lotties,
        tally.manifests,
        tally.set_json,
        tally.build_json,
        tally.failures.len()
    );
    assert!(
        tally.failures.is_empty(),
        "{:?}",
        &tally.failures[..tally.failures.len().min(20)]
    );
}
