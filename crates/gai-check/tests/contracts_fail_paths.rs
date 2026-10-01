//! FAIL-path parity: the original `verify-clearance.mjs` and the Rust interpreter run on
//! the same mutated copy of a set in a temp mirror, and must agree on the evidence bytes
//! (collect mode) or on the abort message (thrown errors, assert mode).
//!
//! Skipped when `node` or the gitignored scenes are unavailable. The repo is only read.

use gai_check::contracts::{
    Clearance, ClearanceReport, ContractError, EVIDENCE_FILE, ScenesDir, SetManifest,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// A copy of one set (a subset of its icons) laid out like the repo.
struct Mirror {
    _root: tempfile::TempDir,
    set_dir: PathBuf,
    set_json: PathBuf,
}

enum Js {
    Wrote(String),
    Threw(String),
}

impl Mirror {
    /// `None` when node or the set's scenes are missing.
    fn new(set: &str, icons: &[&str]) -> Option<Mirror> {
        let src = repo().join("motion/icon-sets").join(set);
        if !node_available() || !src.join("scenes").is_dir() {
            eprintln!("skipped: node or {set} scenes unavailable");
            return None;
        }
        let root = tempfile::tempdir().unwrap();
        let set_dir = root.path().join("motion/icon-sets").join(set);
        let library = root.path().join("motion/icon-sets/library");
        let assets = root
            .path()
            .join("prototype/icons")
            .join(&set[..2])
            .join("assets");
        for dir in [
            set_dir.join("scenes"),
            set_dir.join("evidence"),
            library.clone(),
            assets.clone(),
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
        // Copies, not symlinks: node resolves a symlinked script to the repo.
        std::fs::copy(
            src.join("verify-clearance.mjs"),
            set_dir.join("verify-clearance.mjs"),
        )
        .unwrap();
        std::fs::copy(
            repo().join("motion/icon-sets/library/geometry-subpath.mjs"),
            library.join("geometry-subpath.mjs"),
        )
        .unwrap();
        let mut manifest: Value = serde_json::from_slice(
            &std::fs::read(
                repo()
                    .join("prototype/icons")
                    .join(&set[..2])
                    .join("assets/set.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let list = manifest["icons"].as_array_mut().unwrap();
        list.retain(|i| icons.contains(&i["name"].as_str().unwrap()));
        for icon in list.iter() {
            for a in icon["animations"].as_array().unwrap() {
                let name = format!("{}.scene.json", a["name"].as_str().unwrap());
                std::fs::copy(
                    src.join("scenes").join(&name),
                    set_dir.join("scenes").join(&name),
                )
                .unwrap();
            }
        }
        let set_json = assets.join("set.json");
        std::fs::write(&set_json, serde_json::to_vec(&manifest).unwrap()).unwrap();
        Some(Mirror {
            _root: root,
            set_dir,
            set_json,
        })
    }

    /// Shifts one coordinate axis of every key of the first node whose name starts with `node`.
    fn shift(&self, action: &str, node: &str, axis: usize, by: f64) {
        let path = self
            .set_dir
            .join("scenes")
            .join(format!("{action}.scene.json"));
        let mut scene: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let n = scene["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["name"].as_str().unwrap().starts_with(node))
            .unwrap();
        for key in n["points"].as_array_mut().unwrap() {
            for (i, v) in key[1].as_array_mut().unwrap().iter_mut().enumerate() {
                if i % 2 == axis {
                    *v = json!(v.as_f64().unwrap() + by);
                }
            }
        }
        std::fs::write(&path, serde_json::to_vec(&scene).unwrap()).unwrap();
    }

    fn contract(&self, contract: Value) {
        std::fs::write(
            self.set_dir.join("geometry-contract.json"),
            serde_json::to_vec_pretty(&contract).unwrap(),
        )
        .unwrap();
    }

    fn js(&self) -> Js {
        let evidence = self.set_dir.join(EVIDENCE_FILE);
        let _ = std::fs::remove_file(&evidence);
        let out = Command::new("node")
            .arg("verify-clearance.mjs")
            .current_dir(&self.set_dir)
            .output()
            .unwrap();
        match std::fs::read_to_string(&evidence) {
            Ok(text) => Js::Wrote(text),
            Err(_) => Js::Threw(String::from_utf8_lossy(&out.stderr).into_owned()),
        }
    }

    fn rust(&self) -> Result<ClearanceReport, ContractError> {
        let manifest = SetManifest::read(&self.set_json).unwrap();
        let clearance = Clearance::for_set(&self.set_dir, &manifest.id)
            .unwrap()
            .unwrap();
        clearance.run(&manifest, &ScenesDir(self.set_dir.join("scenes")))
    }

    /// Both sides write evidence, byte-identical, with the expected verdict.
    fn assert_same_evidence(&self, expect_pass: bool) -> ClearanceReport {
        let report = self.rust().unwrap_or_else(|e| panic!("rust aborted: {e}"));
        match self.js() {
            Js::Wrote(expected) => assert_eq!(report.to_json(), expected),
            Js::Threw(stderr) => panic!("js aborted: {stderr}"),
        }
        assert_eq!(report.passed(), expect_pass);
        report
    }

    /// Both sides abort without evidence, and node's message contains `needle`.
    fn assert_same_abort(&self, needle: &str) -> ContractError {
        let err = self.rust().expect_err("rust should abort");
        match self.js() {
            Js::Threw(stderr) => assert!(stderr.contains(needle), "node said: {stderr}"),
            Js::Wrote(_) => panic!("js wrote evidence"),
        }
        err
    }
}

fn coffee_rules(extra: Vec<Value>) -> Value {
    let mut rules = vec![
        json!({"kind": "gap", "a": "wisp-0", "b": "body-frame", "min": 1, "label": "wisp-0 / body-frame"}),
    ];
    rules.extend(extra);
    json!({"version": 1, "icons": {"coffee-cup": rules}})
}

#[test]
fn contract_failures_write_identical_evidence() {
    let Some(m) = Mirror::new("39-food-drink", &["coffee-cup"]) else {
        return;
    };
    m.shift("coffee-cup", "coffee-cup-body-frame-bold", 0, -6.0);
    let mut contract = coffee_rules(vec![
        json!({"kind": "gap", "a": "wisp-0", "b": "wisp-1", "min": 99, "label": "always fails"}),
        json!({"kind": "gap", "a": "wisp-1", "b": "wisp-0", "min": 99, "label": "always fails"}),
        json!({"kind": "point-joint", "a": "wisp-0", "b": "wisp-1", "ai": 2, "bi": 2, "label": "joint"}),
        json!({"kind": "attachment", "a": "handle", "b": "body-frame", "end": 0, "label": "attach"}),
        json!({"kind": "opaque", "a": "wisp-1", "label": "opaque wisp"}),
        json!({"kind": "self-gap", "a": "body-frame", "b": "body-frame", "ai": 1, "bi": 3, "label": "self"}),
        json!({"kind": "arrow-curve-gap", "a": "handle", "b": "body-frame", "curveCommands": 4, "label": "curve"}),
        json!({"kind": "gap", "a": "handle", "b": "body-frame", "aSegments": [0, 2], "bSegments": [4, 6], "aCommands": 2,
               "bCommands": -1, "profile": "strong", "from": 0.5, "to": 2, "label": "windows"}),
        json!({"kind": "inside", "a": "handle", "box": [4, 4, 20, 28], "label": "inside"}),
        json!({"kind": "gap", "a": "handle", "b": "wisp-1", "direction": "reverse", "label": "never reverse"}),
    ]);
    contract["exceptions"] = json!({"b": "x", "2": [1.0, 2.5], "a": {"10": 1, "1": 2.50}});
    m.contract(contract);
    let report = m.assert_same_evidence(false);
    assert!(report.errors.iter().any(|e| e.contains(": margin ")));
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("always fails frame 0: ") && e.ends_with(" < 99"))
    );
}

#[test]
fn thrown_errors_abort_like_js() {
    let Some(m) = Mirror::new("39-food-drink", &["coffee-cup"]) else {
        return;
    };
    m.contract(json!({"version": 1, "icons": {}}));
    assert!(matches!(
        m.assert_same_abort("coffee-cup missing supervisor geometry contract"),
        ContractError::MissingContract(_)
    ));

    m.contract(coffee_rules(vec![
        json!({"kind": "wobble", "a": "handle", "label": "w"}),
    ]));
    assert!(matches!(
        m.assert_same_abort("Unknown geometry rule wobble"),
        ContractError::UnknownRule(_)
    ));

    m.contract(coffee_rules(vec![
        json!({"kind": "gap", "a": "handle", "b": "body-frame", "bSegments": [5, 2], "label": "s"}),
    ]));
    assert!(matches!(
        m.assert_same_abort("Invalid subpath range"),
        ContractError::InvalidSubpath
    ));

    m.contract(coffee_rules(vec![
        json!({"kind": "gap", "a": "handle", "b": "body-frame", "bSegments": [3, 5], "label": "s"}),
    ]));
    assert!(matches!(
        m.assert_same_abort("Subpath must start with a point"),
        ContractError::SubpathStart
    ));

    m.contract(coffee_rules(vec![
        json!({"kind": "gap", "a": "handle", "b": "spoon", "label": "s"}),
    ]));
    assert!(
        matches!(m.assert_same_abort("coffee-cup-spoon"), ContractError::MissingPart(p) if p == "coffee-cup-spoon")
    );
}

#[test]
fn errors_stay_lazy_like_js() {
    let Some(m) = Mirror::new("39-food-drink", &["coffee-cup"]) else {
        return;
    };
    m.contract(coffee_rules(vec![
        json!({"kind": "wobble", "a": "handle", "profile": "none", "label": "never live"}),
        json!({"kind": "gap", "a": "handle", "b": "spoon", "from": 99, "label": "never live"}),
        json!({"kind": "gap", "a": "spoon", "b": "handle", "direction": "sideways", "label": "never live"}),
    ]));
    m.assert_same_evidence(true);
}

#[test]
fn converted_rules_fail_like_legacy_18() {
    let Some(m) = Mirror::new("18-motion-design", &["parent-layers", "mask-path"]) else {
        return;
    };
    m.shift("parent-layers", "parent-layers-whip-bold", 1, 0.5);
    m.shift("parent-layers", "parent-layers-child-frame-bold", 1, -4.0);
    m.shift(
        "parent-layers-strong",
        "parent-layers-parent-frame-bold",
        0,
        -8.0,
    );
    m.shift("mask-path", "mask-path-mask-bold", 0, 0.7);
    let report = m.assert_same_evidence(false);
    for needle in [
        "detached pick whip frame",
        "node off curve frame",
        ": margin ",
        ": gap ",
    ] {
        assert!(
            report.errors.iter().any(|e| e.contains(needle)),
            "no {needle:?} error"
        );
    }
}

#[test]
fn converted_rules_assert_like_legacy_16() {
    let Some(m) = Mirror::new("16-voice", &["voice-recording"]) else {
        return;
    };
    m.shift(
        "voice-recording",
        "voice-recording-capsule-frame-bold",
        0,
        -9.0,
    );
    let err = m.assert_same_abort("voice-recording voice-recording-capsule-frame-bold__color.primary__outline.bold frame0: margin ");
    let ContractError::Assertion(message) = err else {
        panic!("expected an assertion, got {err}")
    };
    assert!(message.starts_with("voice-recording voice-recording-capsule-frame-bold__color.primary__outline.bold frame0: margin "));
}
