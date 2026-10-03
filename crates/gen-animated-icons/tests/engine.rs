//! Sign-in commands and engine settings without a browser or the network, plus a live
//! build through the hosted engine (`#[ignore]`; needs `gen-animated-icons login`).

mod common;

use common::*;
use serde_json::json;
use std::process::Command;

#[test]
fn sign_in_state_and_engine_settings() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);

    let whoami = run(app, &["whoami"]);
    let error = whoami.fails(4, "NOT_LOGGED_IN");
    let hint = error["hint"].as_str().unwrap();
    assert!(hint.starts_with("Ask the user to run `gen-animated-icons login`"), "{hint}");
    let logout = run(app, &["logout"]).ok();
    assert_eq!(
        logout,
        json!({"loggedOut": true, "engine": "https://creator.mcp.lottiefiles.dev/mcp", "wasSignedIn": false, "revoked": false})
    );
    run(app, &["whoami", "--engine-url", "http://engine.example/mcp"]).fails(3, "INVALID_CONFIG");

    // GAI_ENGINE_TOKEN stands in for a sign-in at the GAI_ENGINE_URL engine.
    let env = [("GAI_ENGINE_TOKEN", "ci-secret"), ("GAI_ENGINE_URL", "https://engine.example/mcp")];
    let info = run_with(app, &["info", "--offline"], &env).ok();
    assert_eq!(
        info["auth"],
        json!({"loggedIn": true, "session": "not-verified", "engine": "https://engine.example/mcp", "reason": "offline"})
    );
    // The flag may move it to another LottieFiles engine, but not to another origin.
    let moved = [&env[..], &[("GEN_ANIMATED_ICONS_ENGINE_URL", "https://creator.mcp.lottiefiles.com/mcp")]].concat();
    let info = run_with(app, &["info", "--offline"], &moved).ok();
    assert_eq!(info["auth"]["engine"], "https://creator.mcp.lottiefiles.com/mcp");
    let elsewhere = run_with(app, &["whoami", "--engine-url", "https://other.example/mcp"], &env);
    let error = elsewhere.fails(3, "INVALID_CONFIG");
    assert!(error["message"].as_str().unwrap().contains("GAI_ENGINE_TOKEN"), "{error}");
    assert!(!elsewhere.stderr.contains("ci-secret"));

    // A project's engine.url applies when neither the flag nor a variable is set, and only
    // for a LottieFiles engine.
    run(app, &["init"]).ok();
    let original = read(app, "gen-animated-icons.json");
    let with_engine = |url: &str| original.replacen("{\n", &format!("{{\n  \"engine\": {{\"url\": \"{url}\"}},\n"), 1);
    write(app, "gen-animated-icons.json", with_engine("https://creator.mcp.lottiefiles.com/mcp"));
    let info = run(app, &["info", "--offline"]).ok();
    assert_eq!(
        info["auth"],
        json!({"loggedIn": false, "session": "signed-out", "engine": "https://creator.mcp.lottiefiles.com/mcp"})
    );
    let doctor = run(app, &["doctor", "--offline"]).ok();
    let auth = doctor["checks"].as_array().unwrap().iter().find(|c| c["id"] == "auth").unwrap().clone();
    assert_eq!(auth["message"], "Not signed in to https://creator.mcp.lottiefiles.com/mcp");
    assert!(auth["hint"].as_str().unwrap().starts_with("Ask the user to run `gen-animated-icons login`"), "{auth}");

    write(app, "gen-animated-icons.json", with_engine("https://project.example/mcp"));
    let error = run(app, &["whoami"]).fails(3, "INVALID_CONFIG");
    assert!(error["message"].as_str().unwrap().contains("https://project.example/mcp"), "{error}");
    let info = run(app, &["info", "--offline"]).ok();
    assert_eq!(info["auth"]["engine"], json!(null));
    // The user's own choice wins over the project's.
    let info = run_with(app, &["info", "--offline"], &[("GAI_ENGINE_URL", "https://engine.example/mcp")]).ok();
    assert_eq!(info["auth"]["engine"], "https://engine.example/mcp");
}

/// A cloned project must not receive the CI token or the stored sign-in: its engine.url
/// is refused before anything is sent.
#[test]
fn a_project_cannot_send_the_engine_token_elsewhere() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    run(app, &["init"]).ok();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let config = read(app, "gen-animated-icons.json").replacen(
        "{\n",
        &format!("{{\n  \"engine\": {{\"url\": \"{url}\"}},\n"),
        1,
    );
    write(app, "gen-animated-icons.json", config);

    let env = [("GAI_ENGINE_TOKEN", "ci-secret-token")];
    for args in [&["whoami"][..], &["doctor"], &["build", "--all"]] {
        let run = run_with(app, args, &env);
        assert!(!run.stderr.contains("ci-secret-token") && !run.stdout.contains("ci-secret-token"), "{run:?}");
    }
    assert!(listener.accept().is_err(), "the project's engine was contacted");
    run_with(app, &["whoami"], &env).fails(3, "INVALID_CONFIG");
}

/// `--offline` never reaches the network, so nothing that exports runs with it: the
/// token and a loopback engine stand in for a session, and the engine hears nothing.
#[test]
fn offline_builds_refuse_before_any_export() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(app, &["init"]).ok();
    run(app, &["create", "rocket-launch", "--batch", "--library", &library]).ok();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let env = [("GAI_ENGINE_TOKEN", "ci-secret-token"), ("GAI_ENGINE_URL", url.as_str())];
    for args in [&["build", "--offline"][..], &["build", "rocket-launch", "--offline", "--library", &library]] {
        let error = run_with(app, args, &env).fails(5, "NETWORK");
        assert_eq!(error["message"], "build exports through the hosted engine, and --offline never uses the network");
        assert_eq!(error["hint"], "Drop --offline to build; gen-animated-icons check runs the offline checks");
    }
    // The maintainer build exports too, unless it reuses natives or writes scenes only.
    let repo = dir.path().join("repo");
    write(&repo, "motion/icon-sets/64-test/definitions.mjs", "export const icons = [];\n");
    let root = repo.display().to_string();
    let error =
        run_with(app, &["library", "build", "64", "--offline", "--library-root", &root], &env).fails(5, "NETWORK");
    assert_eq!(
        error["message"],
        "library build exports through the hosted engine, and --offline never uses the network"
    );
    assert_eq!(error["hint"], "Drop --offline, or pass --reuse-native or --offline-scenes");
    assert!(listener.accept().is_err(), "the engine was contacted");
}

/// `--offline` never uses the network, so the sign-in commands read only local state or
/// refuse: the token and a loopback engine stand in for a session (the home is a temporary
/// one), and the engine hears nothing.
#[test]
fn offline_sign_in_commands_never_reach_the_engine() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let env = [("GAI_ENGINE_TOKEN", "ci-secret-token"), ("GAI_ENGINE_URL", url.as_str())];

    // whoami reports the stored session without checking it.
    let whoami = run_with(app, &["whoami", "--offline"], &env);
    let value = whoami.ok();
    assert_eq!(
        value,
        json!({"engine": url, "session": "not-verified", "reason": "offline", "serverVersion": null, "account": null,
            "clientId": null, "accessTokenExpiresAt": null, "refreshTokenExpiresAt": null,
            "credentialStore": "environment", "serverName": null, "protocolVersion": null})
    );
    let text = run_with(app, &["whoami", "--offline", "--format", "text"], &env);
    assert_eq!(
        text.stdout,
        format!("Signed in to {url} as an unnamed account; not verified: offline (run whoami without --offline to check the session with the engine).\n")
    );
    // Signed out stays an error, read from the temporary home only.
    run(app, &["whoami", "--offline"]).fails(4, "NOT_LOGGED_IN");

    // login and logout need the engine.
    let error = run_with(app, &["login", "--offline"], &env).fails(5, "NETWORK");
    assert_eq!(error["message"], "login signs in through the hosted engine, and --offline never uses the network");
    assert_eq!(error["hint"], "Run gen-animated-icons login without --offline");
    let error = run_with(app, &["logout", "--offline"], &env).fails(5, "NETWORK");
    assert_eq!(
        error["message"],
        "logout revokes the session with the hosted engine, and --offline never uses the network"
    );
    assert_eq!(error["hint"], "Run gen-animated-icons logout without --offline");
    for run in [&whoami, &text] {
        assert!(!run.stdout.contains("ci-secret-token") && !run.stderr.contains("ci-secret-token"), "{run:?}");
    }
    assert!(listener.accept().is_err(), "the engine was contacted");
}

#[test]
#[ignore = "needs gen-animated-icons login and the hosted engine"]
fn live_build_through_the_hosted_engine() {
    let signed_in =
        gai_engine::EngineConfig::from_env().ok().and_then(|config| gai_engine::status(&config).ok().flatten());
    if signed_in.is_none() {
        eprintln!("skipped: not signed in; run gen-animated-icons login");
        return;
    }
    require!(packages_dist(), "packages/icons/dist");
    let repo = repo_root().display().to_string();
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    // The signed-in user's own credentials, so no isolated home.
    let live = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_gen-animated-icons"))
            .args(["--json", "--library-root", &repo])
            .args(args)
            .current_dir(app)
            .env("GEN_ANIMATED_ICONS_CACHE_DIR", app.join(".test-cache"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{args:?}: {}", String::from_utf8_lossy(&output.stderr));
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };
    live(&["init"]);
    live(&["create", "live-folder", "--batch", "--from", "folder-open"]);
    let started = std::time::Instant::now();
    let built = live(&["build", "live-folder"]);
    eprintln!("hosted build of 4 actions: {:.1} s", started.elapsed().as_secs_f64());
    assert_eq!(built["status"], "PASS", "{built}");
    assert!(built["engine"]["serverVersion"].is_string());
    let icon = &built["icons"][0];
    assert_eq!(icon["actions"].as_array().unwrap().len(), 4);
    assert!(app.join(icon["asset"]["path"].as_str().unwrap()).is_file());
}
