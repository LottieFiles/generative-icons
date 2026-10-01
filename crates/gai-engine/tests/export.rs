//! MCP calls and the exporter against the mock engine.

mod common;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use common::{FakeBrowser, Fault, MockEngine, hosted_fixture};
use gai_core::export::Exporter;
use gai_engine::{
    EngineConfig, Error, HostedExporter, LoginMode, LoginOptions, McpClient, login,
    normalize_native, whoami,
};

const SCENE: &str = r#"{"name":"filter-strong","width":32,"height":32,"fps":60,"duration":2.4,"rest":0.26666666666666666,"nodes":[]}"#;

/// A signed-in config for `engine`.
fn signed_in(engine: &MockEngine, home: &std::path::Path) -> EngineConfig {
    let config = engine.config(home);
    let options = LoginOptions {
        mode: LoginMode::Paste,
        open_browser: false,
        timeout: Duration::from_secs(20),
    };
    login(
        &config,
        &options,
        &mut FakeBrowser::new(engine.state.clone(), true),
    )
    .unwrap();
    config
}

#[test]
fn export_downloads_normalizes_deletes_and_records_provenance() {
    let hosted = hosted_fixture("filter-strong");
    let engine = MockEngine::start(&hosted);
    let home = tempfile::tempdir().unwrap();
    let exporter = HostedExporter::new(&signed_in(&engine, home.path())).unwrap();

    let native = exporter.export("filter-strong", SCENE).unwrap();
    assert_eq!(native.text, normalize_native(&hosted).unwrap().text);
    let p = &native.provenance;
    assert_eq!(
        p["server"],
        serde_json::json!({"name": "creator-mcp", "version": "0.2.0"})
    );
    assert_eq!(p["protocolVersion"], "2025-11-25");
    assert_eq!(p["documentId"], "gen-animated-icons-export-1");
    assert_eq!(p["documentRevision"], "7");
    assert_eq!(
        p["scriptSha256"],
        gai_core::sha256_hex(gai_engine::export::EXPORT_SCRIPT.as_bytes())
    );
    assert_eq!(p["sceneSha256"], gai_core::sha256_hex(SCENE.as_bytes()));
    assert_eq!(p["sha256"], gai_core::sha256_hex(native.text.as_bytes()));
    assert_eq!(p["bytes"], native.text.len());
    assert_eq!(p["hostedBytes"], hosted.len());
    assert_eq!(
        (
            p["width"].as_u64(),
            p["height"].as_u64(),
            p["layers"].as_u64()
        ),
        (Some(32), Some(32), Some(12))
    );
    assert_eq!(
        (p["fps"].as_f64(), p["duration"].as_f64()),
        (Some(60.0), Some(2.4))
    );
    assert!(p["completedAt"].as_str().unwrap().ends_with('Z'));

    let state = engine.state.lock();
    assert_eq!(
        state.tool_calls,
        [
            "creator_exec",
            "creator_download_file",
            "creator_delete_file"
        ]
    );
    assert_eq!(state.deleted.len(), 1);
    let file = &state.deleted[0];
    assert!(
        file.starts_with(&format!("filter-strong-{}-", std::process::id()))
            && file.ends_with(".json"),
        "{file}"
    );
    assert!(state.files.is_empty());
    assert_eq!((state.signed_url_hits, state.resource_reads), (1, 0));
}

#[test]
fn transient_failures_expired_tokens_and_lost_sessions_are_recovered() {
    let engine = MockEngine::start(&hosted_fixture("filter-strong"));
    let home = tempfile::tempdir().unwrap();
    let exporter = HostedExporter::new(&signed_in(&engine, home.path())).unwrap();
    {
        let mut state = engine.state.lock();
        state.sse = true;
        state.faults.extend([
            Fault::Status(429),
            Fault::Status(503),
            Fault::Unauthorized,
            Fault::DropSession,
        ]);
    }
    exporter.export("filter-strong", SCENE).unwrap();
    let state = engine.state.lock();
    assert!(state.faults.is_empty());
    assert_eq!(state.refreshes, 1, "the 401 is answered by one refresh");
    assert_eq!(state.refresh_reuses, 0);
    assert_eq!(
        state.sessions_started, 2,
        "the lost session is re-initialized"
    );
    assert_eq!(
        state.tool_calls,
        [
            "creator_exec",
            "creator_download_file",
            "creator_delete_file"
        ]
    );
}

#[test]
fn a_failing_signed_url_falls_back_to_resources_read() {
    let hosted = hosted_fixture("cancelled-reset");
    let engine = MockEngine::start(&hosted);
    let home = tempfile::tempdir().unwrap();
    let exporter = HostedExporter::new(&signed_in(&engine, home.path())).unwrap();
    engine.state.lock().signed_url_status = Some(503);
    let native = exporter.export("cancelled-reset", SCENE).unwrap();
    assert_eq!(native.text, normalize_native(&hosted).unwrap().text);
    let state = engine.state.lock();
    assert_eq!(
        state.signed_url_hits, 4,
        "the signed URL is retried before falling back"
    );
    assert_eq!(state.resource_reads, 1);
}

#[test]
fn parallel_exports_never_share_a_document() {
    let engine = MockEngine::start(&hosted_fixture("empty-state-reset-strong"));
    let home = tempfile::tempdir().unwrap();
    let mut config = signed_in(&engine, home.path());
    config.documents = 2;
    let exporter = Arc::new(HostedExporter::new(&config).unwrap());
    let workers: Vec<_> = (0..8)
        .map(|i| {
            let exporter = Arc::clone(&exporter);
            thread::spawn(move || exporter.export(&format!("icon-{i}"), SCENE).unwrap())
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let state = engine.state.lock();
    assert_eq!(state.document_overlaps, 0);
    assert_eq!(state.documents_used.len(), 2);
    assert_eq!(state.deleted.len(), 8);
    assert_eq!(state.sessions_started, 1);
}

#[test]
fn processes_sharing_a_document_never_share_a_file() {
    let engine = MockEngine::start(&hosted_fixture("empty-state-reset-strong"));
    let home = tempfile::tempdir().unwrap();
    let mut config = signed_in(&engine, home.path());
    config.documents = 1;
    // Two exporters stand in for two processes: each has its own pool of the same document.
    let exporters = [
        Arc::new(HostedExporter::new(&config).unwrap()),
        Arc::new(HostedExporter::new(&config).unwrap()),
    ];
    let workers: Vec<_> = (0..8)
        .map(|i| {
            let exporter = Arc::clone(&exporters[i % 2]);
            thread::spawn(move || exporter.export("same-icon", SCENE).unwrap())
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let state = engine.state.lock();
    let names: std::collections::HashSet<_> = state.deleted.iter().collect();
    assert_eq!((state.deleted.len(), names.len()), (8, 8));
    assert_eq!(state.documents_used.len(), 1);
}

#[test]
fn download_errors_never_show_the_signed_url() {
    let engine = MockEngine::start(&hosted_fixture("filter-strong"));
    let home = tempfile::tempdir().unwrap();
    let exporter = HostedExporter::new(&signed_in(&engine, home.path())).unwrap();
    {
        let mut state = engine.state.lock();
        state.signed_url_status = Some(403);
        state.no_uri = true;
    }
    let err = exporter.export_scene("filter-strong", SCENE).unwrap_err();
    let shown = format!("{err} {err:?}");
    assert!(matches!(err, Error::Http { status: 403, .. }), "{shown}");
    assert!(!shown.contains("X-Amz-Signature"), "{shown}");
    assert!(
        shown.contains("/r2/gen-animated-icons-export-1/filter-strong-"),
        "{shown}"
    );
}

#[test]
fn engine_errors_surface_with_codes() {
    let engine = MockEngine::start("{\"layers\":[]}");
    let home = tempfile::tempdir().unwrap();
    let config = signed_in(&engine, home.path());
    let exporter = HostedExporter::new(&config).unwrap();

    // The download is not a valid Lottie document.
    let err = exporter.export_scene("bad", SCENE).unwrap_err();
    assert!(matches!(err, Error::InvalidExport(_)), "{err}");
    let deleted = engine.state.lock().deleted.clone();
    assert!(
        deleted.len() == 1 && deleted[0].starts_with("bad-"),
        "the file is deleted even when the export fails: {deleted:?}"
    );

    assert!(matches!(
        exporter.export_scene("x", "{not json"),
        Err(Error::InvalidScene(_))
    ));

    engine.state.lock().faults.extend([Fault::Status(503); 4]);
    let err = exporter.export_scene("down", SCENE).unwrap_err();
    assert!(matches!(err, Error::Unavailable { status: 503 }), "{err}");
    assert_eq!(err.code(), "ENGINE_UNAVAILABLE");

    let client = McpClient::new(&config).unwrap();
    let err = client
        .call_tool(
            "creator_download_file",
            serde_json::json!({"file": "missing.json", "documentId": "d"}),
        )
        .unwrap_err();
    assert!(
        matches!(&err, Error::Tool { message, .. } if message.contains("no output file")),
        "{err}"
    );
}

#[test]
fn whoami_reports_the_server_and_account() {
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let config = signed_in(&engine, home.path());
    let me = whoami(&config).unwrap();
    assert_eq!(
        (me.server_name.as_str(), me.server_version.as_str()),
        ("creator-mcp", "0.2.0")
    );
    assert_eq!(me.status.account.as_deref(), Some("designer@example.com"));
    let json = serde_json::to_value(&me).unwrap();
    assert_eq!(json["serverVersion"], "0.2.0");
    assert_eq!(json["clientId"], "client-1");

    let empty = tempfile::tempdir().unwrap();
    let err = whoami(&engine.config(empty.path())).unwrap_err();
    assert!(err.needs_login() && err.code() == "NOT_LOGGED_IN", "{err}");
}
