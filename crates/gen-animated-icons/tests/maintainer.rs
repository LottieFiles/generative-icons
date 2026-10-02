//! Maintainer commands that need no build outputs: `library new-set` against the
//! `icon-set.py` goldens (cli.md §4.1) and `library packages` against the built npm
//! dist of this checkout.

mod common;

use common::*;
use serde_json::{Value, json};
use std::path::Path;

/// A root holding only the three templates `icon-set.py` copies (`test_icon_set.py:18-29`).
fn template_root(root: &Path) {
    let repo = repo_root();
    for file in [
        "motion/icon-sets/34-scheduling-time/build.mjs",
        "motion/icon-sets/34-scheduling-time/verify.mjs",
        "prototype/icons/34/verify.html",
    ] {
        write(root, file, std::fs::read(repo.join(file)).unwrap());
    }
}

#[test]
fn new_sets_match_the_icon_set_py_goldens() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    template_root(root);
    let library = |args: &[&str]| {
        let mut full = vec!["--library-root", root.to_str().unwrap(), "library", "new-set"];
        full.extend(args);
        run(root, &full)
    };
    let shared = [
        ("build.mjs", "37496c02bf774e2c5b2d3d72a317d3343137bcfbd88955333787e519ce0a224c"),
        ("verify.mjs", "53d2a3ee3ba37a8ef5648dc80003c0e1fb900ba88682e5834840cd7d7bab42d4"),
        ("definitions.mjs", "aa6967d8c82963a819d575bbe1b714e1c0469c9fb335dd64c2b781bcf5371a37"),
        ("package.py", "d5e00c30daaa09bf64a58c041e54f8b460c6605b778c2a993b9c1eab87e7c8ea"),
        ("evidence/design-review.md", "724b51754b6323ee560bfe34d7f2389adbf3972918d3225d59103769ada06670"),
        ("batches/.gitkeep", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
    ];
    let cases = [
        (
            vec!["landscaping", "--name", "Landscaping"],
            "35-landscaping",
            [
                ("set-config.json", "59d9b46931a134808efdc9eef8566759c628b23676ca07a626c47b88a3d65e5d"),
                ("README.md", "47767ead25ac93ff05fb3f07342c565c69d4b1898c7d67f39f48f6b4e4490765"),
                ("DOWNLOAD_README.txt", "bfca6cf997890b20ee5cb5b1ad0b803cd8780eb164e7ceb71dec190a65456711"),
            ],
            "eca8f1f8eef86d4d6e19e547798c3aea24fb8033df8f00a48d6d58f70b6bd51a",
        ),
        (
            vec!["food-drink"],
            "36-food-drink",
            [
                ("set-config.json", "5378229dde06a331b1040cfe767e02c73bc44c665a9fe185254e301f1e038cd5"),
                ("README.md", "31c25e97cb1a9f2f881016c39c4b210ac6c858d46db210b66ee6162f50825f18"),
                ("DOWNLOAD_README.txt", "2920edb2f0a102b7073fc6973770c3ee3aa653ab518b90d047e1f9b2fbbf2fc0"),
            ],
            "924ff915bc9d3c8c700e1fb26a7351917784110e30c6a3674ec0e77aab2437ca",
        ),
        (
            vec!["q-a", "--name", "Q&A <\"tools\">"],
            "37-q-a",
            [
                ("set-config.json", "e48c720aa679f5371008a287e98cb9d78217f137811f91c0e73d3856eb00191b"),
                ("README.md", "9751ab94822e9a1895210f587f8fdea2c13a51aefe9042402b47eb0e026a6e23"),
                ("DOWNLOAD_README.txt", "28bf2bdb7756c9db279d850d8c55e96d23067336608bbe4fac998d83ac0419b3"),
            ],
            "1c4ecfd04d10975ae6f47c4ebebaf5faa565d07a46629af2bfb9ff01f78bc487",
        ),
    ];
    for (args, set_id, own, page) in cases {
        let mut args = args;
        args.push("--legacy-templates");
        let created = library(&args).ok();
        assert_eq!(created["setId"], set_id);
        for (file, golden) in shared.iter().chain(own.iter()) {
            assert_eq!(sha(root, &format!("motion/icon-sets/{set_id}/{file}")), *golden, "{set_id}/{file}");
        }
        assert_eq!(sha(root, &format!("prototype/icons/{}/verify.html", &set_id[..2])), page, "{set_id} page");
    }

    // A slug is a slug, a set is never replaced, and edits survive a refused rerun.
    library(&["../escape"]).fails(2, "USAGE");
    write(root, "motion/icon-sets/35-landscaping/definitions.mjs", "export const icons = [1];\n");
    let again = library(&["landscaping"]).fails(3, "EXISTS");
    assert_eq!(again["message"], "A source set named landscaping already exists.");
    assert_eq!(read(root, "motion/icon-sets/35-landscaping/definitions.mjs"), "export const icons = [1];\n");

    // The default scaffold has no JS builder, verifier or page.
    let created = library(&["garden-tools"]).ok();
    assert_eq!((created["setId"].as_str(), created["page"].clone()), (Some("38-garden-tools"), Value::Null));
    let files: Vec<&str> = created["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap()).collect();
    assert_eq!(
        files,
        [
            "set-config.json",
            "definitions.mjs",
            "batches/.gitkeep",
            "DOWNLOAD_README.txt",
            "evidence/design-review.md",
            "README.md"
        ]
        .map(|file| format!("motion/icon-sets/38-garden-tools/{file}"))
    );
    let config: Value = serde_json::from_str(&read(root, "motion/icon-sets/38-garden-tools/set-config.json")).unwrap();
    assert_eq!(config["sharedSources"], json!(["drawing.mjs", "direction.mjs", "ai-authoring.mjs"]));
    assert!(!root.join("prototype/icons/38").exists());
}

#[test]
fn packages_match_the_built_dist_and_carry_the_sidecar() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let repo = repo_root();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("packages");
    let result = run(
        dir.path(),
        &["--library-root", repo.to_str().unwrap(), "library", "packages", "--output", out.to_str().unwrap()],
    )
    .ok();
    assert_eq!(result["icons"], 1005);
    let built = out.join("icons/dist");
    let mut compared = 0;
    for file in files_below(&dist) {
        if file == Path::new("release.json") {
            continue;
        }
        assert!(
            std::fs::read(dist.join(&file)).unwrap() == std::fs::read(built.join(&file)).unwrap(),
            "{} differs",
            file.display()
        );
        compared += 1;
    }
    // release.json differs only in the source state it records.
    let strip = |path: &Path| {
        let mut release: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for key in ["status", "sourceCommit", "sourceDirty"] {
            release.as_object_mut().unwrap().remove(key);
        }
        release
    };
    assert_eq!(strip(&dist.join("release.json")), strip(&built.join("release.json")));
    let sidecar: Value = serde_json::from_slice(&std::fs::read(built.join("search-index.json")).unwrap()).unwrap();
    assert_eq!(sidecar.as_object().unwrap().len(), 1005);
    assert_eq!(
        sidecar["bell"]["reference"]["intent"],
        "The bell rings. Light: the bell swings out from its mount, the clapper trails it and strikes, a ring arc flicks out on each side and fades as it spreads, and the swing decays back to rest in smaller beats. Strong: a wider swing that rings twice, once at each end of the swing, and the clapper chases the bell through a longer decay. The mount never moves."
    );
    assert!(compared > 3000, "{compared}");
}

/// Copies `sets` (all with a measurement when `None`) and the kit into `root`, without
/// their measurement evidence, runs `library measure`, and returns how many evidence
/// files equal the committed ones.
fn measure_like_measure_motion(root: &Path, sets: Option<&[&str]>) -> usize {
    let repo = repo_root();
    let sets_dir = repo.join("motion/icon-sets");
    let mut names: Vec<String> = match sets {
        Some(sets) => sets.iter().map(|s| s.to_string()).collect(),
        None => std::fs::read_dir(&sets_dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().into_string().unwrap())
            .filter(|name| name.as_bytes()[0].is_ascii_digit() && sets_dir.join(name).join(EVIDENCE).is_file())
            .filter(|name| ("22".."40").contains(&&name[..2]))
            .collect(),
    };
    names.sort();
    for folder in names.iter().map(String::as_str).chain(["library"]) {
        for file in files_below(&sets_dir.join(folder)) {
            if file != Path::new(EVIDENCE) {
                write(
                    root,
                    &format!("motion/icon-sets/{folder}/{}", file.display()),
                    std::fs::read(sets_dir.join(folder).join(&file)).unwrap(),
                );
            }
        }
    }
    let mut args = vec!["--library-root", root.to_str().unwrap(), "library", "measure"];
    match sets {
        Some(_) => args.extend(names.iter().map(String::as_str)),
        None => args.push("--all"),
    }
    let report = run(root, &args).ok();
    assert_eq!(report["sets"].as_array().unwrap().len(), names.len(), "{report}");
    names
        .iter()
        .filter(|set| {
            let committed = std::fs::read(sets_dir.join(set).join(EVIDENCE)).unwrap();
            std::fs::read(root.join("motion/icon-sets").join(set).join(EVIDENCE)).unwrap() == committed
        })
        .count()
}

const EVIDENCE: &str = "evidence/motion-measurements.json";

/// One raw-ratio set (with a Strong-to-Light flag) and one visible-ratio set.
#[test]
fn motion_measurements_match_the_committed_evidence() {
    let repo = repo_root();
    require!(
        repo.join("motion/icon-sets/39-food-drink").join(EVIDENCE).is_file().then_some(()),
        "measurement evidence"
    );
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(measure_like_measure_motion(dir.path(), Some(&["23-get-help", "39-food-drink"])), 2);
    let report = run(dir.path(), &["--library-root", dir.path().to_str().unwrap(), "library", "measure", "23"]).ok();
    assert_eq!(report["sets"][0]["reviewAtNativeSpeed"], json!(["callback"]));
    run(dir.path(), &["--library-root", dir.path().to_str().unwrap(), "library", "measure", "library"])
        .fails(3, "UNKNOWN_ICON");
}

#[test]
#[ignore = "whole catalog (sets 22-39)"]
fn every_motion_measurement_matches_the_committed_evidence() {
    let repo = repo_root();
    require!(repo.join("motion/icon-sets/22-teamwork").join(EVIDENCE).is_file().then_some(()), "measurement evidence");
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(measure_like_measure_motion(dir.path(), None), 18);
}
