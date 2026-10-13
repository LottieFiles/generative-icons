//! Spec v2 plan runs against the mock engine: one `creator_exec` per icon.

mod common;

use std::time::Duration;

use common::{FakeBrowser, MockEngine, TINY_PNG};
use gai_engine::{EngineConfig, LoginMode, LoginOptions, PlanRunner, login};
use serde_json::{Value, json};

const BUNDLE: &str = r#"{"v":"5.7.0","fr":60.0,"ip":0.0,"op":144.0,"w":32,"h":32,"nm":"x@bundle","layers":[
  {"ty":0,"nm":"x-strong","refId":"comp_0","ind":1},{"ty":0,"nm":"x","refId":"comp_1","ind":2}],
  "assets":[{"id":"comp_0","layers":[{"ty":4,"nm":"rig-a","ind":1,"ks":{},"shapes":[]}]},
            {"id":"comp_1","layers":[{"ty":4,"nm":"rig-a","ind":1,"ks":{},"shapes":[]},{"ty":4,"nm":"x-a-regular__color.primary__outline.regular","ind":2,"parent":1,"ks":{},"shapes":[{"ty":"gr","it":[{"ty":"sh","nm":"x-a-regular"},{"ty":"st","nm":"color.primary__outline.regular"},{"ty":"tr","a":{"a":0,"k":[0.0,0.0]},"p":{"a":0,"k":[0.0,0.0]},"s":{"a":0,"k":[100.0,100.0]},"r":{"a":0,"k":0.0},"o":{"a":0,"k":100.0}}]}]}]}]}"#;

fn signed_in(engine: &MockEngine, home: &std::path::Path) -> EngineConfig {
    let config = engine.config(home);
    let options = LoginOptions { mode: LoginMode::Paste, open_browser: false, timeout: Duration::from_secs(20) };
    login(&config, &options, &mut FakeBrowser::new(engine.state.clone(), true)).unwrap();
    config
}

#[test]
fn draft_returns_the_attached_sheets_and_stores_nothing() {
    let engine = MockEngine::start(BUNDLE);
    let home = tempfile::tempdir().unwrap();
    let runner = PlanRunner::new(&signed_in(&engine, home.path()), "en-test-v2").unwrap();
    let run = runner.draft("// v2 script", json!({"plan": {"id": "x"}, "sheet": {"rows": []}})).unwrap();
    assert_eq!(run.images.len(), 2);
    assert_eq!(run.images[0].bytes, TINY_PNG);
    assert_eq!(run.images[0].mime, "image/png");
    assert_eq!(run.document, "en-test-v2-1");
    assert_eq!(run.result["timing"]["totalMs"], 42);
    let state = engine.state.lock();
    assert_eq!(state.tool_calls, ["creator_exec"]);
    assert!(state.files.is_empty());
}

#[test]
fn a_bundle_export_is_one_call_one_file_split_per_motion() {
    let engine = MockEngine::start(BUNDLE);
    let home = tempfile::tempdir().unwrap();
    let runner = PlanRunner::new(&signed_in(&engine, home.path()), "en-test-v2").unwrap();
    let motions = ["x".to_string(), "x-strong".to_string()];
    let bundle = runner.export_bundle("// v2 script", json!({"plan": {"id": "x"}}), "x", &motions, "planhash").unwrap();
    assert_eq!(bundle.animations.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), ["x", "x-strong"]);
    let x: Value = serde_json::from_str(&bundle.animations[0].native.text).unwrap();
    assert_eq!(x["nm"], "x");
    assert_eq!(x["layers"].as_array().unwrap().len(), 2);
    // normalize_native unwrapped the identity part group, as for v1.
    assert_eq!(x["layers"][1]["shapes"][0]["ty"], "sh");
    let p = &bundle.animations[1].provenance;
    assert_eq!(p["source"], "gai-icon/2");
    assert_eq!(p["planSha256"], "planhash");
    assert_eq!(p["scriptSha256"], gai_core::sha256_hex(b"// v2 script"));
    assert_eq!(p["documentId"], "en-test-v2-1");
    assert_eq!(p["bundleBytes"], BUNDLE.len());
    assert_eq!(bundle.hosted_bytes, BUNDLE.len());
    let state = engine.state.lock();
    assert_eq!(state.tool_calls, ["creator_exec", "creator_download_file", "creator_delete_file"]);
    assert_eq!(state.deleted.len(), 1);
    assert!(state.deleted[0].starts_with("x-") && state.deleted[0].ends_with(".json"));
}

#[test]
fn a_document_the_engine_can_no_longer_save_is_left_for_a_fresh_one() {
    let engine = MockEngine::start(BUNDLE);
    let home = tempfile::tempdir().unwrap();
    let runner = PlanRunner::new(&signed_in(&engine, home.path()), "en-test-v2").unwrap();
    engine.state.lock().unsavable.insert("en-test-v2-1".into());
    // The export runs, downloads and deletes on the fresh document.
    let motions = ["x".to_string(), "x-strong".to_string()];
    let bundle = runner.export_bundle("// v2 script", json!({"plan": {"id": "x"}}), "x", &motions, "planhash").unwrap();
    let fresh = bundle.run.document.clone();
    assert!(fresh.starts_with("en-test-v2-1-") && fresh.len() == "en-test-v2-1-".len() + 8, "{fresh}");
    assert_eq!(bundle.animations[0].provenance["documentId"], fresh);
    let state = engine.state.lock();
    assert_eq!(state.tool_calls, ["creator_exec", "creator_exec", "creator_download_file", "creator_delete_file"]);
    assert!(state.files.keys().all(|(doc, _)| *doc == fresh));
    drop(state);
    // When the fresh one fails too, the error is reported; it moves only once.
    let mut state = engine.state.lock();
    state.all_unsavable = true;
    state.tool_calls.clear();
    drop(state);
    let error = runner.draft("// v2 script", json!({"plan": {"id": "x"}, "sheet": {"rows": []}})).unwrap_err();
    assert!(error.to_string().contains("snapshot serialization failed"), "{error}");
    assert_eq!(engine.state.lock().tool_calls, ["creator_exec", "creator_exec"]);
}

#[test]
fn a_bundle_missing_a_motion_fails() {
    let engine = MockEngine::start(BUNDLE);
    let home = tempfile::tempdir().unwrap();
    let runner = PlanRunner::new(&signed_in(&engine, home.path()), "en-test-v2").unwrap();
    let error = runner.export_bundle("// v2", json!({"plan": {}}), "x", &["x-reverse".to_string()], "h").unwrap_err();
    assert!(error.to_string().contains("no motion x-reverse"), "{error}");
}
