//! `package_set` against each set's own Python `package.py`, run on two copies of the same
//! inputs: set zips (including extras), `.lottie` files, manifests, `set.json` and
//! `build.json` must match byte for byte. The on-disk set zips are gitignored and stale, so
//! they are not oracles.
//!
//! Sets 01–06 used unsorted directory listings; Rust sorts them, so their zips may differ
//! in entry order only (same entries, same bytes).

use crate::common::{available, catalog_set_ids, copy_parts, packaging_inputs, repo, run};
use gai_package::zip::read_zip;
use gai_package::{SetPaths, package_set};
use std::path::Path;
use std::process::Command;

enum ZipMatch {
    Identical,
    OrderOnly,
    Different(String),
}

fn compare_zips(python: &[u8], rust: &[u8]) -> ZipMatch {
    if python == rust {
        return ZipMatch::Identical;
    }
    let (Ok(python), Ok(rust)) = (read_zip(python), read_zip(rust)) else {
        return ZipMatch::Different("unreadable".into());
    };
    let names = |files: &[gai_package::zip::ZipFile]| {
        files.iter().map(|f| f.name.clone()).collect::<Vec<_>>()
    };
    if let Some(i) = (0..python.len().max(rust.len())).find(|&i| python.get(i) != rust.get(i)) {
        let sorted = |files: Vec<gai_package::zip::ZipFile>| {
            let mut pairs: Vec<_> = files.into_iter().map(|f| (f.name, f.data)).collect();
            pairs.sort();
            pairs
        };
        let detail = format!(
            "entry {i}: python {:?} rust {:?}",
            names(&python).get(i),
            names(&rust).get(i)
        );
        return if sorted(python) == sorted(rust) {
            ZipMatch::OrderOnly
        } else {
            ZipMatch::Different(detail)
        };
    }
    ZipMatch::Different("same entries, different bytes".into())
}

fn same_file(a: &Path, b: &Path) -> bool {
    std::fs::read(a).ok() == std::fs::read(b).ok()
}

#[test]
#[ignore = "whole catalog; runs the Python packagers"]
fn package_set_matches_python() {
    let Some(repo) = repo() else { return };
    if !available("python3") {
        return;
    }
    let sets: Vec<String> = std::env::var("GAI_SETS").map_or_else(
        |_| catalog_set_ids(&repo),
        |list| list.split(',').map(str::to_string).collect(),
    );
    let inputs = packaging_inputs(&sets);
    let (python, rust) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    copy_parts(&repo, python.path(), &inputs);
    copy_parts(&repo, rust.path(), &inputs);

    let (mut identical, mut order_only, mut files, mut failures) =
        (Vec::new(), Vec::new(), 0, Vec::new());
    for set_id in &sets {
        // Sets scaffolded by `library new-set` have no package.py; theirs would only call
        // the shared packager.
        let set_dir = python.path().join("motion/icon-sets").join(set_id);
        match set_dir.join("package.py").is_file() {
            true => run(Command::new("python3").arg(set_dir.join("package.py"))),
            false => run(Command::new("python3")
                .arg(python.path().join("motion/icon-sets/library/package-set.py"))
                .arg(&set_dir)),
        };
        let report = package_set(&SetPaths::new(rust.path(), set_id).unwrap()).unwrap();
        let (py, rs) = (
            SetPaths::new(python.path(), set_id).unwrap(),
            SetPaths::new(rust.path(), set_id).unwrap(),
        );
        let mut outputs = vec![
            py.assets.join("set.json"),
            py.root.join("evidence/build.json"),
        ];
        for archive in &report.archives {
            outputs.push(py.assets.join(format!("{}.lottie", archive.name)));
            outputs.push(py.assets.join(format!("{}.manifest.json", archive.name)));
        }
        for output in outputs {
            let counterpart = rust
                .path()
                .join(output.strip_prefix(python.path()).unwrap());
            files += 1;
            if !same_file(&output, &counterpart) {
                failures.push(format!("{set_id}: {}", counterpart.display()));
            }
        }
        let zip_name = report.zip.file_name().unwrap();
        match compare_zips(
            &std::fs::read(py.assets.join(zip_name)).unwrap(),
            &std::fs::read(rs.assets.join(zip_name)).unwrap(),
        ) {
            ZipMatch::Identical => identical.push(set_id.clone()),
            ZipMatch::OrderOnly if set_id.as_str() < "07" => order_only.push(set_id.clone()),
            ZipMatch::OrderOnly => failures.push(format!("{set_id}: zip entry order differs")),
            ZipMatch::Different(detail) => {
                failures.push(format!("{set_id}: zip differs at {detail}"))
            }
        }
    }
    println!(
        "{} sets: {} zips byte-identical, {} legacy zips equal up to entry order {:?}; {files} other outputs compared; {} failures",
        sets.len(),
        identical.len(),
        order_only.len(),
        order_only,
        failures.len()
    );
    assert!(failures.is_empty(), "{failures:#?}");
}
