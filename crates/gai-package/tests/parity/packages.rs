//! npm package builds: against the `packages/*/dist` trees in the checkout, and against
//! `scripts/build-packages.py` and `scripts/pack-release.mjs` run on a committed temporary
//! copy of their inputs.

use crate::common::{available, copy_tree, repo, run};
use gai_package::{
    BuildPackagesOptions, GitState, PACKAGES, PackReleaseOptions, build_packages, pack_release,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every file below `dir`, by `/`-separated relative path.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(relative, std::fs::read(&path).unwrap());
            }
        }
    }
    files
}

/// Compares `<package>/dist` and `<package>/LICENSE` of every package; returns the file count.
fn compare_packages(expected: &Path, actual: &Path) -> usize {
    let mut count = 0;
    for package in PACKAGES {
        let (want, got) = (
            tree(&expected.join(package).join("dist")),
            tree(&actual.join(package).join("dist")),
        );
        assert_eq!(
            want.keys().collect::<Vec<_>>(),
            got.keys().collect::<Vec<_>>(),
            "{package}/dist file list"
        );
        for (name, bytes) in &want {
            assert!(got[name] == *bytes, "{package}/dist/{name} differs");
        }
        let license = |root: &Path| std::fs::read(root.join(package).join("LICENSE")).unwrap();
        assert_eq!(license(expected), license(actual), "{package}/LICENSE");
        count += want.len() + 1;
    }
    count
}

#[test]
#[ignore = "whole catalog"]
fn packages_match_checkout_dist() {
    let Some(repo) = repo() else { return };
    let Ok(bytes) = std::fs::read(repo.join("packages/icons/dist/release.json")) else {
        return;
    };
    let release: Value = serde_json::from_slice(&bytes).unwrap();
    // Record the source state the checkout's dist was built from.
    let git = GitState {
        commit: release["sourceCommit"].as_str().unwrap().to_string(),
        dirty: release["sourceDirty"].as_bool().unwrap(),
    };
    let out = tempfile::tempdir().unwrap();
    let report = build_packages(
        &repo,
        &BuildPackagesOptions {
            output: Some(out.path().to_path_buf()),
            git: Some(git),
        },
    )
    .unwrap();
    let files = compare_packages(&repo.join("packages"), out.path());
    println!(
        "{} ({files} files identical to packages/*/dist)",
        report.summary()
    );
    assert_eq!(report.icons, 1161);
}

/// The inputs `build-packages.py` and `pack-release.mjs` read.
fn package_inputs(repo: &Path) -> Vec<PathBuf> {
    let mut parts: Vec<PathBuf> = [
        ".gitignore",
        "LICENSE",
        "docs",
        "schemas",
        "scripts/build-packages.py",
        "scripts/pack-release.mjs",
        "prototype/icons/catalog.json",
        "prototype/icons/03/controller.mjs",
        "prototype/icons/guide/ai-operating-guide.md",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    for package in PACKAGES {
        for entry in std::fs::read_dir(repo.join("packages").join(package)).unwrap() {
            let name = entry.unwrap().file_name();
            if name != "dist" && name != "node_modules" {
                parts.push(Path::new("packages").join(package).join(name));
            }
        }
    }
    // Specs of icons reanimated as specs ship in the icons package.
    for entry in std::fs::read_dir(repo.join("motion/icon-sets")).unwrap() {
        let specs = entry.unwrap().path().join("specs");
        if specs.is_dir() {
            parts.push(specs.strip_prefix(repo).unwrap().to_path_buf());
        }
    }
    for relative in gai_package::Catalog::load(repo).unwrap().sets {
        let folder = Path::new("prototype/icons")
            .join(&relative)
            .parent()
            .unwrap()
            .to_path_buf();
        parts.push(folder.join("set.json"));
        for entry in std::fs::read_dir(repo.join(&folder)).unwrap() {
            let name = entry.unwrap().file_name();
            if name.to_string_lossy().ends_with(".lottie") {
                parts.push(folder.join(name));
            }
        }
    }
    parts
}

#[test]
#[ignore = "whole catalog; runs build-packages.py and pack-release.mjs"]
fn packages_match_python_and_node_on_a_copy() {
    let Some(repo) = repo() else { return };
    if !["python3", "git"].iter().all(|p| available(p)) {
        return;
    }
    let copy = tempfile::tempdir().unwrap();
    let root = copy.path();
    for part in package_inputs(&repo) {
        copy_tree(&repo.join(&part), &root.join(&part));
    }
    let git = |args: &[&str]| {
        run(Command::new("git")
            .args([
                "-c",
                "user.name=parity",
                "-c",
                "user.email=parity@example.com",
            ])
            .args(args)
            .current_dir(root))
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "inputs"]);

    let printed = run(Command::new("python3").arg(root.join("scripts/build-packages.py")));
    let out = tempfile::tempdir().unwrap();
    let report = build_packages(
        root,
        &BuildPackagesOptions {
            output: Some(out.path().to_path_buf()),
            git: None,
        },
    )
    .unwrap();
    assert_eq!(report.summary() + "\n", printed);
    let files = compare_packages(&root.join("packages"), out.path());
    println!(
        "build-packages.py on a committed copy: {files} files identical; {}",
        report.summary()
    );

    if !["node", "npm"].iter().all(|p| available(p)) {
        return;
    }
    let packs = tempfile::tempdir().unwrap();
    let (js, rs) = (packs.path().join("node"), packs.path().join("rust"));
    let printed = run(Command::new("node")
        .arg(root.join("scripts/pack-release.mjs"))
        .arg(&js)
        .current_dir(root));
    let report = pack_release(
        root,
        &PackReleaseOptions {
            output: Some(rs.clone()),
            allow_dirty: false,
        },
    )
    .unwrap();
    assert_eq!(
        report
            .summary()
            .replace(&rs.display().to_string(), &js.display().to_string())
            + "\n",
        printed
    );
    assert_eq!(tree(&js), tree(&rs), "pack-release outputs differ");
    println!(
        "pack-release.mjs: {} tarballs and release.json identical ({})",
        report.packages.len(),
        report.status
    );
}
