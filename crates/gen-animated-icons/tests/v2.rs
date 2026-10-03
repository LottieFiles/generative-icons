//! Spec v2 icons (`icons/<id>.icon.json`): create, check, draft and build, offline from a bundle
//! the hosted engine exported, plus a live draft and build (`#[ignore]`; needs
//! `gen-animated-icons login`).

mod common;

use common::*;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    repo_root().join("crates/gai-spec/tests/fixtures")
}

/// A web app with the CLI set up and the alarm-clock spec in place.
fn app() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    run(&app, &["init", "--agents", "none", "--offline"]).ok();
    let spec = std::fs::read(fixtures().join("alarm-clock.icon.json")).unwrap();
    write(&app, "gen-animated-icons/icons/alarm-clock.icon.json", spec);
    (dir, app)
}

#[test]
fn create_spec_writes_a_starter_that_checks_clean() {
    let (_dir, app) = app();
    let created = run(&app, &["create", "bell-ring", "--spec"]).ok();
    assert_eq!(created["spec"], "gen-animated-icons/icons/bell-ring.icon.json");
    assert_eq!(created["format"], "gai-icon/2");
    let checked = run(&app, &["check", "bell-ring"]).ok();
    assert_eq!(checked["status"], "PASS");
    assert_eq!(checked["icons"][0]["diagnostics"], json!([]));
    run(&app, &["create", "bell-ring", "--spec"]).fails(3, "EXISTS");
}

#[test]
fn create_spec_from_a_library_spec_keeps_its_craft() {
    let (_dir, app) = app();
    let root = repo_root().to_string_lossy().into_owned();
    let created = run(&app, &["create", "paper-plane", "--spec", "--from", "send", "--library-root", &root]).ok();
    assert_eq!(created["from"]["id"], "send");
    let path = repo_root().join(format!("motion/icon-sets/{}/specs/send.icon.json", created["from"]["set"].as_str().unwrap()));
    let library: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let copy: Value = serde_json::from_slice(&std::fs::read(app.join("gen-animated-icons/icons/paper-plane.icon.json")).unwrap()).unwrap();
    assert_eq!((copy["id"].as_str(), copy["label"].as_str()), (Some("paper-plane"), Some("Paper plane")));
    for key in ["intent", "parts", "rigs", "motion", "state", "reverse"] {
        assert_eq!(copy.get(key), library.get(key), "{key}");
    }
    assert_eq!(run(&app, &["check", "paper-plane"]).ok()["status"], "PASS");
    // In a checkout, search says which icons have a spec to start from.
    let found = run(&app, &["search", "send", "--library-only", "--library-root", &root]).ok();
    assert_eq!(found.as_array().unwrap().iter().find(|r| r["id"] == "send").unwrap()["spec"], true);
    // A reanimated icon's set source is the motion its spec replaced: a JS batch refuses it.
    let refused = run(&app, &["create", "paper-plane-2", "--batch", "--from", "send", "--library-root", &root]).fails(3, "UNSUPPORTED_REFERENCE");
    assert!(refused["hint"].as_str().unwrap().contains("create paper-plane-2 --from send"), "{refused}");
    // An icon kept as it was has no spec; the hint names icons that have one.
    let none = run(&app, &["create", "spinner-2", "--spec", "--from", "rotate-tool", "--library-root", &root]).fails(3, "UNSUPPORTED_REFERENCE");
    let hint = none["hint"].as_str().unwrap();
    assert!(hint.starts_with("Start from an icon with a spec, such as ") && hint.ends_with("--batch --from rotate-tool"), "{none}");
    run(&app, &["create", "x-2", "--spec", "--from", "no-such-icon", "--library-root", &root]).fails(3, "UNKNOWN_ICON");
}

#[test]
fn create_writes_a_spec_by_default_with_a_context_packet() {
    let (_dir, app) = app();
    let root = repo_root().to_string_lossy().into_owned();
    let created = run(&app, &["create", "alarm-nudge", "--neighbor", "menu", "--neighbor", "heart", "--library-root", &root]).ok();
    assert_eq!((created["format"].as_str(), created["spec"].as_str()), (Some("gai-icon/2"), Some("gen-animated-icons/icons/alarm-nudge.icon.json")));
    assert_eq!(created["context"], ".gen-animated-icons/context/alarm-nudge.json");
    assert_eq!(created["references"], json!(["menu", "heart"]));
    assert!(created.get("nearest").is_none());
    assert!(app.join(".gen-animated-icons/context/alarm-nudge.json").is_file());
    assert!(!app.join("gen-animated-icons/icons/batches/alarm-nudge.mjs").exists());
    assert_eq!(run(&app, &["check", "alarm-nudge"]).ok()["status"], "PASS");
    // A new session finds it as work in progress.
    let info = run(&app, &["info", "--offline"]).ok();
    let mine = info["local"]["icons"].as_array().unwrap().iter().find(|i| i["id"] == "alarm-nudge").unwrap().clone();
    assert_eq!(mine, json!({"id": "alarm-nudge", "spec": "gen-animated-icons/icons/alarm-nudge.icon.json", "batch": null, "build": "not-built", "installed": false}));
    // Without references, the closest library icons are listed.
    let lexical = run(&app, &["create", "bell-swing", "--intent", "The bell swings.", "--library-root", &root]).ok();
    assert!(!lexical["nearest"].as_array().unwrap().is_empty(), "{lexical}");
    // A library id is taken, and --spec (the old opt-in) still works.
    run(&app, &["create", "bell", "--library-root", &root]).fails(3, "ID_COLLISION");
    assert_eq!(run(&app, &["create", "bell-twice", "--spec", "--library-root", &root]).ok()["format"], "gai-icon/2");
    run(&app, &["create", "x-3", "--spec", "--batch"]).fails(2, "USAGE");
}

#[test]
fn check_reports_pointers_codes_and_hints() {
    let (_dir, app) = app();
    write(&app, "gen-animated-icons/icons/broken.icon.json", r#"{"format": "gai-icon/2", "id": "broken", "label": "Broken", "intent": "Light: one swing. Strong: a bigger one.",
      "parts": {"feet": {"shape": {"path": "M10 23 L10 26 M21 23 L21 26"}}, "dot": {"shape": {"cirle": [[16, 16], 2]}}},
      "motion": {"feet.y": {"light": [[0, 0], [0.2, 1, "hti"], [0.4, 0]]}}}"#);
    let checked = run(&app, &["check", "broken"]);
    assert_eq!(checked.code, 1, "{}", checked.stderr);
    let report = checked.json();
    let diagnostics = report["icons"][0]["diagnostics"].as_array().unwrap().clone();
    let found: Vec<(&str, &str)> = diagnostics.iter().map(|d| (d["code"].as_str().unwrap(), d["path"].as_str().unwrap())).collect();
    assert_eq!(found, [("ONE_SUBPATH", "/parts/feet/shape/path"), ("V2_SCHEMA", "/parts/dot/shape/cirle"), ("V2_SCHEMA", "/motion/feet.y/light/1/2")]);
    assert_eq!(diagnostics[1]["hint"], "did you mean `circle`?");
    assert_eq!(diagnostics[2]["hint"], "did you mean `hit`?");
    // A draft of a spec with errors stops before the engine, signed in or not.
    let drafted = run(&app, &["draft", "broken"]);
    assert_eq!(drafted.code, 1);
    assert_eq!(drafted.json()["stage"], "spec");
}

#[test]
fn draft_and_build_need_a_session_and_can_emit_their_call() {
    let (_dir, app) = app();
    run(&app, &["draft", "alarm-clock"]).fails(4, "NOT_LOGGED_IN");
    run(&app, &["build", "alarm-clock"]).fails(4, "NOT_LOGGED_IN");
    let emitted = run(&app, &["draft", "alarm-clock", "--emit-call", "draft.json"]).ok();
    assert_eq!(emitted["icon"], "alarm-clock");
    let call: Value = serde_json::from_str(&read(&app, "draft.json")).unwrap();
    assert_eq!(call["code"].as_str(), Some(gai_spec::SCRIPT));
    assert_eq!(call["timeoutMs"], 30000);
    assert_eq!(call["params"]["sheet"]["rows"].as_array().unwrap().len(), 5);
    assert!(call["params"].get("file").is_none());
    run(&app, &["build", "alarm-clock", "--emit-call", "build.json"]).ok();
    let call: Value = serde_json::from_str(&read(&app, "build.json")).unwrap();
    assert_eq!(call["params"]["file"], "alarm-clock.json");
    assert_eq!(call["params"]["plan"]["motions"].as_array().unwrap().len(), 2);
    // v1 and v2 icons build in separate runs.
    write(&app, "gen-animated-icons/icons/batches/old.mjs", "export const icons = [];\n");
    run(&app, &["build", "alarm-clock", "old"]).fails(2, "USAGE");
}

#[test]
fn build_from_an_engine_bundle_meets_the_runtime_contract() {
    let (_dir, app) = app();
    let bundle = fixtures().join("alarm-clock.bundle.json");
    let built = run(&app, &["build", "alarm-clock", "--from-bundle", bundle.to_str().unwrap()]).ok();
    assert_eq!(built["status"], "PASS", "{built}");
    let icon = &built["icons"][0];
    assert_eq!(icon["source"], "gai-icon/2");
    let names: Vec<&str> = icon["actions"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["alarm-clock", "alarm-clock-strong"]);
    assert!(icon["checks"][0].as_str().unwrap().starts_with("paint: PASS"), "{icon}");

    // The contract: 32×32, 60 fps, markers, slots, layer names, weights, themes.
    let build = app.join(".gen-animated-icons/build/alarm-clock");
    let strong: Value = serde_json::from_str(&std::fs::read_to_string(build.join("alarm-clock-strong.json")).unwrap()).unwrap();
    assert_eq!((strong["w"].as_u64(), strong["h"].as_u64(), strong["fr"].as_f64()), (Some(32), Some(32), Some(60.0)));
    assert_eq!(strong["markers"], json!([{"cm": "action", "tm": 0, "dr": 42}, {"cm": "rest", "tm": 45, "dr": 1}, {"cm": "showcase", "tm": 0, "dr": 144}]));
    let slots: Vec<&str> = strong["slots"].as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(slots, ["color.primary", "style.outline.light", "style.outline.regular", "style.outline.bold", "style.solid", "style.tint", "style.outline.width"]);
    let layers: Vec<&str> = strong["layers"].as_array().unwrap().iter().map(|l| l["nm"].as_str().unwrap()).collect();
    assert!(layers.contains(&"alarm-clock-body-regular") && layers.contains(&"rig-clock-a"), "{layers:?}");
    assert!(layers.iter().all(|l| !l.contains("__")));
    for (theme, color) in [("dark", "0.8666666666666667"), ("light", "0.5490196078431373")] {
        let text = std::fs::read_to_string(build.join(format!("alarm-clock-strong.theme-{theme}.json"))).unwrap();
        assert!(text.contains(color), "{theme}: {text}");
    }
    let set: Value = serde_json::from_str(&std::fs::read_to_string(build.join("set.json")).unwrap()).unwrap();
    let record = &set["icons"][0]["animations"][1];
    assert_eq!((record["motionId"].as_str(), record["active"].as_f64(), record["rest"].as_u64()), (Some("strong"), Some(0.7), Some(45)));
    assert_eq!(record["source"], "gai-icon/2");
    assert_eq!(record["duotoneFill"], true);

    // One .lottie holds Light and Strong with both themes, and it is installed.
    let lottie = std::fs::read(app.join("public/gen-animated-icons/alarm-clock.lottie")).unwrap();
    let files = gai_package::zip::read_zip(&lottie).unwrap();
    let entries: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(entries, ["manifest.json", "a/alarm-clock.json", "a/alarm-clock-strong.json", "t/dark.json", "t/light.json"]);
    let lock: Value = serde_json::from_str(&read(&app, ".gen-animated-icons/lock.json")).unwrap();
    assert_eq!(lock["icons"]["alarm-clock"]["format"], "gai-icon/2");
    assert!(app.join(".gen-animated-icons/previews/alarm-clock/alarm-clock-strong.png").is_file());
}

/// [`app`] as `create --spec` leaves a project (its set config written), with the alarm clock
/// built from the bundle the hosted engine exported.
fn built_app() -> (tempfile::TempDir, PathBuf) {
    let (dir, app) = app();
    run(&app, &["create", "bell-ring", "--spec"]).ok();
    let bundle = fixtures().join("alarm-clock.bundle.json");
    let built = run(&app, &["build", "alarm-clock", "--from-bundle", bundle.to_str().unwrap()]).ok();
    assert_eq!(built["status"], "PASS", "{built}");
    (dir, app)
}

const SPEC: &str = "gen-animated-icons/icons/alarm-clock.icon.json";

/// `(id, build)` of each local icon `info` lists.
fn local_builds(app: &Path) -> Vec<(String, String)> {
    let info = run(app, &["--offline", "info"]).ok();
    let icons = info["local"]["icons"].as_array().unwrap().iter();
    icons.map(|icon| (icon["id"].as_str().unwrap().to_string(), icon["build"].as_str().unwrap().to_string())).collect()
}

#[test]
fn info_calls_a_v2_build_current_until_its_spec_changes() {
    let (_dir, app) = built_app();
    // The unbuilt bell-ring spec is listed too: work in progress a new session must find.
    let builds = |alarm: &str| vec![("alarm-clock".to_string(), alarm.to_string()), ("bell-ring".to_string(), "not-built".to_string())];
    assert_eq!(local_builds(&app), builds("current"));
    // Another spec is not this build's source.
    write(&app, "gen-animated-icons/icons/bell-ring.icon.json", read(&app, "gen-animated-icons/icons/bell-ring.icon.json") + "\n");
    assert_eq!(local_builds(&app), builds("current"));

    let spec = read(&app, SPEC);
    write(&app, SPEC, spec.replacen("\"label\"", "\"label\" ", 1));
    assert_eq!(local_builds(&app), builds("stale"));
    write(&app, SPEC, &spec);
    assert_eq!(local_builds(&app), builds("current"));
}

#[test]
fn check_without_ids_checks_every_spec_and_v2_build() {
    let (_dir, app) = app();
    run(&app, &["create", "bell-ring", "--spec"]).ok();
    let checked = run(&app, &["check"]).ok();
    assert_eq!(checked["status"], "PASS", "{checked}");
    let ids: Vec<&str> = checked["checks"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["project", "spec", "descriptors", "native"]);
    let spec = &checked["checks"][1];
    assert_eq!(spec["status"], "PASS");
    assert_eq!(spec["icons"], json!([
        {"id": "alarm-clock", "status": "PASS", "spec": SPEC},
        {"id": "bell-ring", "status": "PASS", "spec": "gen-animated-icons/icons/bell-ring.icon.json"},
    ]));
    assert_eq!(checked["checks"][3]["status"], "SKIP");
    // --source is the spec check here.
    let source = run(&app, &["check", "--source"]).ok();
    assert_eq!(source["checks"].as_array().unwrap().len(), 1);
    assert_eq!(source["checks"][0]["id"], "spec");

    // Built: the native check verifies the v2 build, current.
    let bundle = fixtures().join("alarm-clock.bundle.json");
    run(&app, &["build", "alarm-clock", "--from-bundle", bundle.to_str().unwrap()]).ok();
    let checked = run(&app, &["check"]).ok();
    assert_eq!((checked["status"].as_str(), checked.get("stale")), (Some("PASS"), None), "{checked}");
    let native = &checked["checks"][3];
    assert_eq!((native["id"].as_str(), native["status"].as_str()), (Some("native"), Some("PASS")), "{native}");
    assert_eq!(native["diagnostics"], json!([]));
    assert_eq!(run(&app, &["check", "--native", "alarm-clock"]).ok()["status"], "PASS");

    // A spec edited since the build makes it stale; a spec error fails the check, with its pointer.
    let spec = read(&app, SPEC);
    write(&app, SPEC, spec.replacen("\"label\"", "\"label\" ", 1));
    let stale = run(&app, &["check"]).ok();
    assert_eq!((stale["status"].as_str(), &stale["stale"]), (Some("STALE"), &json!(["alarm-clock"])), "{stale}");
    write(&app, "gen-animated-icons/icons/broken.icon.json", r#"{"format": "gai-icon/2", "id": "broken", "label": "Broken", "intent": "Light: one. Strong: two.",
      "parts": {"dot": {"shape": {"cirle": [[16, 16], 2]}}}, "motion": {"dot.y": {"light": [[0, 0], [0.2, 1], [0.4, 0]]}}}"#);
    let failed = run(&app, &["check"]);
    assert_eq!(failed.code, 1, "{failed:?}");
    let report = failed.json();
    let finding = report["checks"][1]["diagnostics"].as_array().unwrap().iter().find(|d| d["icon"] == "broken").unwrap().clone();
    assert_eq!(
        (finding["code"].as_str(), finding["path"].as_str(), finding["pointer"].as_str(), finding["hint"].as_str()),
        (Some("V2_SCHEMA"), Some("gen-animated-icons/icons/broken.icon.json"), Some("/parts/dot/shape/cirle"), Some("did you mean `circle`?"))
    );
}

#[test]
fn submit_bundles_a_verified_v2_build() {
    let (dir, app) = app();
    run(&app, &["create", "bell-ring", "--spec"]).ok();
    run(&app, &["submit", "alarm-clock", "--dry-run"]).fails(3, "UNKNOWN_ICON");
    let bundle = fixtures().join("alarm-clock.bundle.json");
    run(&app, &["build", "alarm-clock", "--from-bundle", bundle.to_str().unwrap()]).ok();
    write(&app, "gen-animated-icons/icons/evidence/design-review.md", "# notes\n");
    // A sheet that preview overwrote is rendered again from the verified build.
    let sheet = ".gen-animated-icons/previews/alarm-clock/alarm-clock.png";
    let review = sha(&app, sheet);
    write(&app, sheet, b"not the review sheet");

    let one = run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "one.zip"]).ok();
    let two = run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "two.zip"]).ok();
    assert_eq!(one["sha256"], two["sha256"]);
    assert_eq!((&one["regenerated"], &two["regenerated"]), (&json!([sheet]), &json!([])));
    assert_eq!(sha(&app, sheet), review);
    run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "one.zip"]).fails(3, "EXISTS");

    let entries = gen_animated_icons::weight::read_zip(&std::fs::read(app.join("one.zip")).unwrap()).unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let built = [
        "alarm-clock-strong.json", "alarm-clock-strong.lottie", "alarm-clock-strong.manifest.json",
        "alarm-clock-strong.theme-dark.json", "alarm-clock-strong.theme-light.json", "alarm-clock.icon.json",
        "alarm-clock.json", "alarm-clock.lottie", "alarm-clock.manifest.json", "alarm-clock.theme-dark.json",
        "alarm-clock.theme-light.json", "native/alarm-clock-strong.json.provenance.json",
        "native/alarm-clock.json.provenance.json", "plan.json", "set.json", "sources.snapshot.json",
    ];
    let mut expected: Vec<String> = built.iter().map(|name| format!("built/alarm-clock/{name}")).collect();
    expected.extend(["checks.json", "previews/alarm-clock/alarm-clock-strong.png", "previews/alarm-clock/alarm-clock.png"].map(String::from));
    expected.extend(["source/gen-animated-icons/icons/alarm-clock.icon.json", "source/gen-animated-icons/icons/evidence/design-review.md", "submission.json"].map(String::from));
    expected.sort();
    assert_eq!(names, expected);
    let entry = |name: &str| entries.iter().find(|e| e.name == name).unwrap().data.clone();
    assert_eq!(entry("source/gen-animated-icons/icons/alarm-clock.icon.json"), read(&app, SPEC).into_bytes());
    assert_eq!(sha256_of(&entry("previews/alarm-clock/alarm-clock.png")), review);
    let submission: Value = serde_json::from_slice(&entry("submission.json")).unwrap();
    assert_eq!((submission["ids"].clone(), submission["format"].as_str()), (json!(["alarm-clock"]), Some("gai-icon/2")));
    let files = submission["files"].as_object().unwrap();
    assert_eq!(files.len(), entries.len() - 1);
    for e in entries.iter().filter(|e| e.name != "submission.json") {
        assert_eq!(files[&e.name].as_str(), Some(sha256_of(&e.data).as_str()), "{}", e.name);
    }
    let checks: Value = serde_json::from_slice(&entry("checks.json")).unwrap();
    assert_eq!(checks["status"], "PASS");
    assert_eq!(checks["checks"][0]["icons"], json!([{"id": "alarm-clock", "status": "PASS", "spec": SPEC}]));
    assert_eq!((checks["checks"][1]["id"].as_str(), checks["checks"][1]["icons"][0]["status"].as_str()), (Some("native"), Some("PASS")));

    // Only the verified build of the current spec: an edited spec is refused until rebuilt.
    let spec = read(&app, SPEC);
    write(&app, SPEC, spec.replacen("\"label\"", "\"label\" ", 1));
    let error = run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "stale.zip"]).fails(1, "CHECK_FAILED");
    assert!(error["message"].as_str().unwrap().contains("alarm-clock: the source changed since it was built (alarm-clock.icon.json)"), "{error}");
    write(&app, SPEC, &spec);
    // A build copy that differs from its snapshot list is refused.
    let copy = app.join(".gen-animated-icons/build/alarm-clock/alarm-clock.icon.json");
    std::fs::write(&copy, spec.replacen("\"label\"", "\"label\" ", 1)).unwrap();
    run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "edited.zip"]).fails(6, "HASH_MISMATCH");
    std::fs::write(&copy, &spec).unwrap();
    // Nothing is read through a symlink: the spec, or a file in the build.
    #[cfg(unix)]
    {
        let real = dir.path().join("alarm-clock.icon.json");
        std::fs::write(&real, &spec).unwrap();
        std::fs::remove_file(app.join(SPEC)).unwrap();
        std::os::unix::fs::symlink(&real, app.join(SPEC)).unwrap();
        let error = run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "link.zip"]).fails(3, "SYMLINK_PATH");
        assert_eq!(error["path"], SPEC);
        assert!(error["message"].as_str().unwrap().contains("never follows symlinks"), "{error}");
        std::fs::remove_file(app.join(SPEC)).unwrap();
        write(&app, SPEC, &spec);
        std::fs::remove_file(&copy).unwrap();
        std::os::unix::fs::symlink(&real, &copy).unwrap();
        run(&app, &["submit", "alarm-clock", "--dry-run", "--output", "link.zip"]).fails(3, "SYMLINK_PATH");
        assert!(!app.join("link.zip").exists());
    }
    // v1 and v2 icons are submitted in separate runs.
    write(&app, "gen-animated-icons/icons/batches/old.mjs", "export const icons = [];\n");
    run(&app, &["submit", "alarm-clock", "old", "--dry-run"]).fails(2, "USAGE");
}

fn sha256_of(bytes: &[u8]) -> String {
    gai_core::sha256_hex(bytes)
}

/// The signed-in user's own credentials, so no isolated home.
fn live(app: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_gen-animated-icons"))
        .arg("--json")
        .args(args)
        .current_dir(app)
        .env("GEN_ANIMATED_ICONS_CACHE_DIR", app.join(".test-cache"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
#[ignore = "live: needs `gen-animated-icons login`"]
fn live_v2_draft_and_build_through_the_hosted_engine() {
    let signed_in = gai_engine::EngineConfig::from_env().ok().and_then(|config| gai_engine::status(&config).ok().flatten());
    if signed_in.is_none() {
        eprintln!("skipped: not signed in; run gen-animated-icons login");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    live(app, &["init", "--agents", "none"]);
    write(app, "gen-animated-icons/icons/late-checkout.icon.json", std::fs::read(fixtures().join("late-checkout.icon.json")).unwrap());
    let started = std::time::Instant::now();
    let draft = live(app, &["draft", "late-checkout"]);
    eprintln!("v2 draft: {:.2} s wall, engine {:?}", started.elapsed().as_secs_f64(), draft["engine"]);
    assert_eq!(draft["sheets"].as_array().unwrap().len(), 2);
    assert!(app.join(".gen-animated-icons/drafts/late-checkout.png").is_file());
    let started = std::time::Instant::now();
    let built = live(app, &["build", "late-checkout"]);
    eprintln!("v2 build: {:.2} s wall, engine {:?}", started.elapsed().as_secs_f64(), built["icons"][0]["engine"]);
    assert_eq!(built["status"], "PASS", "{built}");
    assert_eq!(built["icons"][0]["actions"].as_array().unwrap().len(), 2);
}
