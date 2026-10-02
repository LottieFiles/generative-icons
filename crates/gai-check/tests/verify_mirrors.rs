//! Set verification on writable copies in a temp folder: the failure modes of verify.md
//! §10.3 (which checks fail, which evidence is still written, exit codes) and rule
//! documents supplied as `SET/verify.rules.json`. Tests skip when the gitignored
//! `scenes/`/`native/` intermediates are missing.

use gai_check::verify::{
    DIRECTED_FILE, SetPaths, Verification, VerifyError, VerifyOptions, verify_set,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn source(id: &str) -> PathBuf {
    repo().join("motion/icon-sets").join(id)
}

fn copy_files(from: &Path, to: &Path, keep: impl Fn(&str) -> bool) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().unwrap().is_file() && keep(&name) {
            std::fs::copy(entry.path(), to.join(&name)).unwrap();
        }
    }
}

/// A writable copy of a set: its top-level files, `batches/`, `scenes/`, native JSON and
/// provenance, and the packaged JSON and `.lottie` files. The kit stays the repository's.
struct Mirror {
    _dir: tempfile::TempDir,
    paths: SetPaths,
}

fn mirror(id: &str) -> Option<Mirror> {
    let src = source(id);
    if std::fs::read_dir(src.join("scenes")).is_err()
        || std::fs::read_dir(src.join("native")).is_err()
    {
        eprintln!("skipping: no intermediates for {id}");
        return None;
    }
    let dir = tempfile::tempdir().unwrap();
    let set_dir = dir.path().join(id);
    copy_files(&src, &set_dir, |_| true);
    copy_files(&src.join("scenes"), &set_dir.join("scenes"), |_| true);
    if src.join("batches").is_dir() {
        copy_files(&src.join("batches"), &set_dir.join("batches"), |_| true);
    }
    // Adopted spec icons are verified against their specs.
    if src.join("specs").is_dir() {
        copy_files(&src.join("specs"), &set_dir.join("specs"), |_| true);
    }
    copy_files(&src.join("native"), &set_dir.join("native"), |n| {
        n.ends_with(".json")
    });
    let assets_dir = dir.path().join("assets");
    copy_files(
        &repo().join("prototype/icons").join(&id[..2]).join("assets"),
        &assets_dir,
        |n| n.ends_with(".json") || n.ends_with(".lottie"),
    );
    let paths = SetPaths {
        set_dir,
        assets_dir,
        library_dir: repo().join("motion/icon-sets/library"),
    };
    Some(Mirror { _dir: dir, paths })
}

fn run(m: &Mirror) -> Verification {
    verify_set(&m.paths, &VerifyOptions::default())
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn write_json(path: &Path, v: &Value) {
    std::fs::write(path, serde_json::to_string(v).unwrap()).unwrap();
}

fn first_action(m: &Mirror) -> String {
    read_json(&m.paths.assets_dir.join("set.json"))["icons"][0]["animations"][0]["name"]
        .as_str()
        .unwrap()
        .to_string()
}

fn written(v: &Verification) -> Vec<&str> {
    v.evidence.iter().map(|e| e.path.as_str()).collect()
}

fn message(v: &Verification) -> String {
    v.error
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default()
}

/// Replaces a set's built-in rules with an edited copy in `SET/verify.rules.json`.
fn edit_rules(m: &Mirror, id: &str, edit: impl FnOnce(&mut Value)) {
    let builtin = gai_check::verify::profile::builtin_rules(id).unwrap();
    let mut doc: Value = serde_json::from_str(builtin).unwrap();
    edit(&mut doc);
    write_json(&m.paths.set_dir.join("verify.rules.json"), &doc);
}

/// F1: a changed native export fails provenance; nothing is written.
#[test]
fn stale_native_export_fails_before_any_evidence() {
    let Some(m) = mirror("40-notifications") else {
        return;
    };
    let native = m
        .paths
        .set_dir
        .join(format!("native/{}.json", first_action(&m)));
    let mut bytes = std::fs::read(&native).unwrap();
    bytes.push(b' ');
    std::fs::write(&native, bytes).unwrap();
    let v = run(&m);
    assert_eq!(v.exit_code(), 1);
    assert!(message(&v).contains("sha256"), "{}", message(&v));
    assert!(v.evidence.is_empty());
}

/// F2: a `-0` rotation in the packaged action is a motion change (Object.is); F4: a theme
/// that drops a slot fails the token registry.
#[test]
fn packaging_fidelity_and_theme_registry() {
    let Some(m) = mirror("40-notifications") else {
        return;
    };
    let name = first_action(&m);
    let packaged = m.paths.assets_dir.join(format!("{name}.json"));
    let original = std::fs::read(&packaged).unwrap();
    let mut data = read_json(&packaged);
    let layer = data["layers"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|l| l["ks"]["r"]["a"] == 0)
        .unwrap();
    layer["ks"]["r"]["k"] = serde_json::from_str("-0.0").unwrap();
    write_json(&packaged, &data);
    let v = run(&m);
    assert!(
        message(&v).contains("Packaging changed motion"),
        "{}",
        message(&v)
    );

    std::fs::write(&packaged, original).unwrap();
    let theme = m.paths.assets_dir.join(format!("{name}.theme-dark.json"));
    let mut rules = read_json(&theme);
    rules["rules"].as_array_mut().unwrap().pop();
    write_json(&theme, &rules);
    assert!(message(&run(&m)).contains("Theme drops token registry"));
}

/// F5: a visible node that does not return to its start fails the reset check once the
/// scene hash is re-recorded; F7: stroke presets must be in order.
#[test]
fn reset_geometry_and_preset_order() {
    let Some(m) = mirror("40-notifications") else {
        return;
    };
    let set = read_json(&m.paths.assets_dir.join("set.json"));
    let action = set["icons"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|i| i["animations"].as_array().unwrap())
        .find(|a| a["stateful"] != true && a["cycleDegrees"].as_f64().unwrap_or(0.0) == 0.0)
        .unwrap();
    let name = action["name"].as_str().unwrap();
    let scene_path = m.paths.set_dir.join(format!("scenes/{name}.scene.json"));
    let mut scene = read_json(&scene_path);
    let node = scene["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["visibility"][0][1].as_f64() > Some(0.0))
        .unwrap();
    let last = node["points"].as_array_mut().unwrap().last_mut().unwrap();
    last[1][0] = Value::from(last[1][0].as_f64().unwrap() + 1.0);
    write_json(&scene_path, &scene);
    let proof_path = m
        .paths
        .set_dir
        .join(format!("native/{name}.json.provenance.json"));
    let mut proof = read_json(&proof_path);
    proof["sceneSha256"] = Value::from(gai_core::sha256_hex(std::fs::read(&scene_path).unwrap()));
    write_json(&proof_path, &proof);
    assert!(
        message(&run(&m)).contains("reset changes visible geometry"),
        "{}",
        message(&run(&m))
    );

    let set_path = m.paths.assets_dir.join("set.json");
    let mut set = read_json(&set_path);
    set["icons"][0]["animations"][0]["strokeWidth"]["presets"] = serde_json::json!([2, 1.5, 2.5]);
    write_json(&set_path, &set);
    assert_eq!(run(&m).exit_code(), 1);
}

/// F8: 42's drawing rules run after the directed write and keep FAIL evidence.
#[test]
fn collect_mode_rules_after_the_directed_report_write_fail_evidence() {
    let Some(m) = mirror("42-maps-navigation") else {
        return;
    };
    edit_rules(&m, "42-maps-navigation", |doc| {
        doc["icons"]["traffic-light"][0]["min"] = Value::from(100)
    });
    let v = run(&m);
    assert!(
        matches!(v.error, Some(VerifyError::Collected { .. })),
        "{:?}",
        v.error
    );
    assert_eq!(
        written(&v),
        [DIRECTED_FILE, "evidence/clearance-verification.json"]
    );
    let clearance: Value = serde_json::from_str(&v.evidence[1].contents).unwrap();
    assert_eq!(clearance["result"], "FAIL");
    let first = clearance["errors"][0].as_str().unwrap();
    assert!(
        first.contains("traffic-light gap lamp-0/housing-frame frame") && first.ends_with(" < 100"),
        "{first}"
    );
}

/// F9: 49 writes its clearance, then the directed report with the clearance embedded and
/// the top-level result still PASS.
#[test]
fn embedded_clearance_failure_keeps_the_directed_report() {
    let Some(m) = mirror("49-onboarding-tips") else {
        return;
    };
    edit_rules(&m, "49-onboarding-tips", |doc| {
        doc["icons"]["hotspot"][0]["min"] = Value::from(100)
    });
    let v = run(&m);
    assert_eq!(v.exit_code(), 1);
    assert_eq!(
        written(&v),
        ["evidence/clearance-verification.json", DIRECTED_FILE]
    );
    let directed: Value = serde_json::from_str(&v.evidence[1].contents).unwrap();
    assert_eq!(
        (&directed["result"], &directed["clearance"]["result"]),
        (&Value::from("PASS"), &Value::from("FAIL"))
    );
}

/// F10: 50 marks the directed report FAIL and writes no clearance file.
#[test]
fn rule_major_failure_marks_the_directed_report() {
    let Some(m) = mirror("50-preferences") else {
        return;
    };
    edit_rules(&m, "50-preferences", |doc| {
        doc["icons"]["auto-rotate"][0]["min"] = Value::from(100)
    });
    let v = run(&m);
    assert_eq!(v.exit_code(), 1);
    assert_eq!(written(&v), [DIRECTED_FILE]);
    let directed: Value = serde_json::from_str(&v.evidence[0].contents).unwrap();
    assert_eq!(directed["result"], "FAIL");
    assert!(
        directed["clearance"]["errors"][0]
            .as_str()
            .unwrap()
            .contains("phone / arrow-a frame")
    );
}

/// F11: an assert-mode stage before the directed write leaves nothing written.
#[test]
fn assert_mode_failure_before_the_directed_report_writes_nothing() {
    let Some(m) = mirror("62-vector-editing") else {
        return;
    };
    // An icon still built from its scene: adopted icons skip the drawing rules.
    edit_rules(&m, "62-vector-editing", |doc| {
        doc["icons"]["auto-layout"][0]["limit"] = Value::from(100)
    });
    let v = run(&m);
    assert!(
        matches!(v.error, Some(VerifyError::Assertion(ref msg)) if msg.starts_with("layout child padding: ")),
        "{:?}",
        v.error
    );
    assert!(v.evidence.is_empty());
}

/// Clearance from a geometry contract runs after the directed write and keeps FAIL evidence.
#[test]
fn contract_clearance_failure_after_the_directed_report() {
    let Some(m) = mirror("35-weather") else {
        return;
    };
    let contract = m.paths.set_dir.join("geometry-contract.json");
    let mut doc = read_json(&contract);
    let icons = doc["icons"].as_object_mut().unwrap();
    // A rule of an icon still built from its scene: adopted icons are checked against their specs.
    let specs = m.paths.set_dir.join("specs");
    let rule = icons
        .iter_mut()
        .filter(|(id, _)| !specs.join(format!("{id}.icon.json")).exists())
        .flat_map(|(_, r)| r.as_array_mut().unwrap().iter_mut())
        .find(|r| r["kind"] == "gap")
        .unwrap();
    rule["min"] = Value::from(100);
    write_json(&contract, &doc);
    let v = run(&m);
    assert_eq!(v.exit_code(), 1);
    assert_eq!(
        written(&v),
        [DIRECTED_FILE, "evidence/clearance-verification.json"]
    );
}

/// F13: verification is deterministic; a set built without `build.mjs` verifies against
/// the builder identity, and without one the missing file is an I/O error (exit 2).
#[test]
fn deterministic_and_builder_identity() {
    let Some(m) = mirror("40-notifications") else {
        return;
    };
    let (a, b) = (run(&m), run(&m));
    assert!(a.passed() && a.evidence == b.evidence);
    let build = m.paths.set_dir.join("build.mjs");
    let identity = gai_core::sha256_hex(std::fs::read(&build).unwrap());
    std::fs::remove_file(&build).unwrap();
    assert_eq!(run(&m).exit_code(), 2);
    let with_identity = verify_set(
        &m.paths,
        &VerifyOptions {
            builder_identity_sha256: Some(identity),
        },
    );
    assert!(with_identity.passed(), "{:?}", with_identity.error);
    assert_eq!(with_identity.evidence, a.evidence);

    let dir = tempfile::tempdir().unwrap();
    with_identity.write_evidence(dir.path()).unwrap();
    let written = std::fs::read_to_string(dir.path().join(DIRECTED_FILE)).unwrap();
    assert_eq!(
        written,
        std::fs::read_to_string(source("40-notifications").join(DIRECTED_FILE)).unwrap()
    );
}

/// Runs a contracts rules document unchanged (given an evidence file) as the set's verify
/// rules: its evidence and the contracts clearance stage's must both equal the committed file.
fn contracts_document_runs_unchanged(id: &str) {
    let Some(m) = mirror(id) else { return };
    let doc = repo().join(format!("crates/gai-check/rules/{id}.rules.json"));
    let mut doc = read_json(&doc);
    doc["evidence"]["file"] = Value::from("evidence/clearance-verification.json");
    write_json(&m.paths.set_dir.join("verify.rules.json"), &doc);
    let v = run(&m);
    assert!(v.passed(), "{id}: {:?}", v.error);
    let clearance = "evidence/clearance-verification.json";
    assert_eq!(written(&v), [DIRECTED_FILE, clearance, clearance], "{id}");
    let committed = std::fs::read_to_string(source(id).join(clearance)).unwrap();
    assert!(
        v.evidence[1].contents == committed && v.evidence[2].contents == committed,
        "{id}: clearance differs"
    );
}

/// The shared format: 16 (assert-mode margin and compare) and 18 (gates, per-rule slack and
/// messages) run on the verify engine exactly as on the contracts engine.
#[test]
fn contracts_rules_documents_run_unchanged() {
    for id in ["16-voice", "18-motion-design"] {
        contracts_document_runs_unchanged(id);
    }
}

/// Every built-in contracts rules document (16–20).
#[test]
#[ignore = "five sets; run with --ignored"]
fn every_contracts_rules_document_runs_unchanged() {
    for id in [
        "16-voice",
        "17-media-playback",
        "18-motion-design",
        "19-agent-operations",
        "20-arrows",
    ] {
        contracts_document_runs_unchanged(id);
    }
}

/// The AI scope gate of `run-ai.mjs`: the definitions (run in QuickJS) must list the scope's
/// icons in order. The kit is mirrored next to the set so its relative imports resolve.
#[test]
fn ai_scope_gate_runs_the_definitions() {
    let Some(mut m) = mirror("07-ai-conversation") else {
        return;
    };
    let kit = m.paths.set_dir.parent().unwrap().join("library");
    let repo_kit = repo().join("motion/icon-sets/library");
    copy_files(&repo_kit, &kit, |_| true);
    let scope_dir = kit.join("design-reviews/ai-category");
    copy_files(
        &repo_kit.join("design-reviews/ai-category"),
        &scope_dir,
        |_| true,
    );
    m.paths.library_dir = kit;
    assert!(run(&m).passed());

    let scope_path = scope_dir.join("scope.json");
    let mut scope = read_json(&scope_path);
    let listed = scope["sets"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"] == "07-ai-conversation")
        .unwrap();
    listed["icons"].as_array_mut().unwrap().swap(0, 1);
    write_json(&scope_path, &scope);
    let v = run(&m);
    assert_eq!(
        message(&v),
        "Incomplete or misordered AI scope: 07-ai-conversation"
    );
    assert!(v.evidence.is_empty());
}
