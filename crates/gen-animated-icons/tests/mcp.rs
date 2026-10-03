//! The stdio MCP server: initialize, list the tools, and call `search` and `preview`
//! (which returns the rendered sheets as image content), with this checkout's built
//! packages as the library (skipped without them); and a call stuck in icon code, which
//! runs in a process of its own while the server keeps answering.

mod common;

use base64::Engine as _;
use common::*;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// A running server and its JSON-RPC stream.
struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    next: u64,
    /// Notifications received while waiting for responses.
    notifications: Vec<Value>,
    /// Responses to other requests received while waiting.
    stray: Vec<Value>,
}

impl Session {
    fn start(dir: &std::path::Path, library: &str) -> Session {
        Session::start_with(dir, &["mcp", "--library", library], &[])
    }

    /// Starts `gen-animated-icons <args>` with extra environment variables.
    fn start_with(dir: &std::path::Path, args: &[&str], env: &[(&str, &str)]) -> Session {
        let mut command = command(dir);
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child =
            command.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let (send, lines) = channel();
        std::thread::spawn(move || {
            for line in stdout.lines().map_while(Result::ok) {
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        Session { child, stdin: Some(stdin), lines, next: 1, notifications: Vec::new(), stray: Vec::new() }
    }

    fn send(&mut self, message: Value) {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
    }

    /// Sends one raw line.
    fn send_line(&mut self, line: &str) {
        let stdin = self.stdin.as_mut().unwrap();
        stdin.write_all(line.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
    }

    /// The next message the server writes.
    fn next_message(&mut self) -> Value {
        let line = self.lines.recv_timeout(Duration::from_secs(20)).expect("the server answers");
        serde_json::from_str(&line).unwrap()
    }

    /// Initializes the session.
    fn initialize(&mut self) -> Value {
        let init = self.request(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}}),
        );
        self.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        init
    }

    /// Sends a request and waits for its response.
    fn request(&mut self, method: &str, params: Value) -> Value {
        let message = self.exchange(method, params);
        assert!(message.get("error").is_none(), "{message}");
        message["result"].clone()
    }

    /// Sends a request and returns the whole response message, keeping notifications.
    fn exchange(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let line = self.lines.recv_timeout(Duration::from_secs(120)).expect("the server answers");
            let message: Value = serde_json::from_str(&line).unwrap();
            if message["id"] == id {
                return message;
            }
            match message.get("id") {
                None => self.notifications.push(message),
                Some(_) => self.stray.push(message),
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn agents_search_and_preview_through_mcp() {
    let dist = require!(packages_dist(), "packages/icons/dist");
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(dir.path(), &dist.display().to_string());

    let init = session.initialize();
    assert_eq!(init["serverInfo"]["name"], "gen-animated-icons");
    assert!(init["capabilities"]["tools"].is_object());
    assert!(init["instructions"].as_str().unwrap().contains("preview"));

    let tools = session.request("tools/list", json!({}));
    let names: Vec<&str> = tools["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["search", "add", "context", "create", "check", "draft", "build", "preview"]);
    let search_tool = &tools["tools"][0];
    assert_eq!(search_tool["inputSchema"]["required"], json!(["query"]));
    assert_eq!(search_tool["annotations"]["readOnlyHint"], true);

    let found = session.request(
        "tools/call",
        json!({"name": "search", "arguments": {"query": "heart", "limit": 3, "libraryOnly": true}}),
    );
    assert_eq!(found["isError"], false, "{found}");
    let results = found["structuredContent"]["results"].as_array().unwrap();
    assert_eq!((results.len(), results[0]["id"].as_str()), (3, Some("heart")));
    let text: Value = serde_json::from_str(found["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(text[0]["id"], "heart");
    // Arguments that look like flags are values.
    let dashed =
        session.request("tools/call", json!({"name": "search", "arguments": {"query": "--heart", "limit": 1}}));
    assert_eq!(dashed["structuredContent"]["results"][0]["id"], "heart", "{dashed}");

    let preview = session
        .request("tools/call", json!({"name": "preview", "arguments": {"id": "heart", "motion": "both", "frames": 3}}));
    assert_eq!(preview["isError"], false, "{preview}");
    let files = preview["structuredContent"]["files"].as_array().unwrap();
    assert_eq!((preview["structuredContent"]["source"].as_str(), files.len()), (Some("library asset"), 2));
    let images: Vec<&Value> = preview["content"].as_array().unwrap().iter().filter(|c| c["type"] == "image").collect();
    assert_eq!(images.len(), 2);
    for (image, file) in images.iter().zip(files) {
        assert_eq!(image["mimeType"], "image/png");
        let png = base64::engine::general_purpose::STANDARD.decode(image["data"].as_str().unwrap()).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(std::fs::read(dir.path().join(file["path"].as_str().unwrap())).unwrap(), png);
    }

    // With a progress token, each log line of the command arrives as progress.
    let tracked = session.request(
        "tools/call",
        json!({"name": "search", "arguments": {"query": "bell", "limit": 1}, "_meta": {"progressToken": "p-1"}}),
    );
    assert_eq!(tracked["isError"], false, "{tracked}");
    let progress: Vec<&Value> =
        session.notifications.iter().filter(|n| n["method"] == "notifications/progress").collect();
    assert!(!progress.is_empty(), "{:?}", session.notifications);
    assert_eq!(
        (progress[0]["params"]["progressToken"].as_str(), progress[0]["params"]["progress"].as_f64()),
        (Some("p-1"), Some(1.0))
    );
    assert!(progress[0]["params"]["message"].as_str().unwrap().starts_with("library: "), "{}", progress[0]);

    // A call's folder stays inside the server's.
    for cwd in ["..", "/", "sub/../.."] {
        let refused = session.exchange("tools/call", json!({"name": "check", "arguments": {"cwd": cwd}}));
        assert_eq!(refused["error"]["code"], -32602, "{cwd}: {refused}");
    }

    // Errors come back as tool errors with the CLI's code.
    let unknown = session.request("tools/call", json!({"name": "preview", "arguments": {"id": "no-such-icon"}}));
    assert_eq!(unknown["isError"], true);
    assert_eq!(unknown["structuredContent"]["error"]["code"], "UNKNOWN_ICON");
}

/// Icon code that never returns: the per-call limit stops it after minutes.
const SPIN: &str = "for (;;) {}\nexport const icons = [];\n";

#[test]
fn a_call_stuck_in_icon_code_leaves_the_server_answering() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(&app, &["init"]).ok();
    run(&app, &["create", "spin", "--batch", "--library", &library]).ok();
    write(&app, "gen-animated-icons/icons/batches/spin.mjs", SPIN);
    let mut session = Session::start(&app, &library);
    session.initialize();

    // The call's process names the app folder on its command line.
    let marker = app.canonicalize().unwrap().display().to_string();
    let running = || {
        let ps = std::process::Command::new("ps").args(["-ax", "-o", "command="]).output().unwrap();
        String::from_utf8_lossy(&ps.stdout).lines().any(|line| line.contains(&format!("--cwd {marker}")))
    };

    let started = Instant::now();
    session.send(json!({"jsonrpc": "2.0", "id": 900, "method": "tools/call", "params": {"name": "check", "arguments": {"ids": ["spin"]}}}));
    // The server is not running that code: other requests are answered meanwhile.
    let tools = session.request("tools/list", json!({}));
    assert_eq!(tools["tools"].as_array().unwrap().len(), 8);
    let found = session.request("tools/call", json!({"name": "search", "arguments": {"query": "bell", "limit": 1}}));
    assert_eq!(found["isError"], false, "{found}");
    assert!(started.elapsed() < Duration::from_secs(60), "{:?}", started.elapsed());
    assert!(session.stray.is_empty(), "the stuck check answered: {:?}", session.stray);
    if cfg!(unix) {
        assert!(running(), "the check runs in a process of its own");
    }

    // Closing the connection ends the server, and with it the stuck call's process.
    drop(session.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(30);
    while session.child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "the server outlived its client");
        std::thread::sleep(Duration::from_millis(50));
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while cfg!(unix) && running() {
        assert!(Instant::now() < deadline, "the stuck check outlived the server");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn arguments_outside_a_tools_schema_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(&app, &["init"]).ok();
    for id in ["mailbox", "plant-sprout"] {
        run(&app, &["create", id, "--batch", "--library", &library]).ok();
    }
    let mut session = Session::start(&app, &library);
    session.initialize();

    // `build` takes `ids`; an `id` would otherwise be dropped and every icon built.
    let refused = session.exchange(
        "tools/call",
        json!({"name": "build", "arguments": {"id": "plant-sprout"}, "_meta": {"progressToken": "p-2"}}),
    );
    assert_eq!(refused["error"]["code"], -32602, "{refused}");
    let message = refused["error"]["message"].as_str().unwrap();
    assert!(message.contains("`id`") && message.contains("did you mean `ids`"), "{message}");
    assert!(
        session.notifications.iter().all(|n| n["method"] != "notifications/progress"),
        "{:?}",
        session.notifications
    );

    // Every tool checks names, types and bounds against its schema before running.
    for (name, arguments, expected) in [
        ("check", json!({"id": "mailbox"}), "did you mean `ids`"),
        ("preview", json!({"ids": ["mailbox"]}), "did you mean `id`"),
        ("search", json!({"query": "bell", "local_only": true}), "did you mean `localOnly`"),
        ("search", json!({"limit": 3}), "needs `query`"),
        ("check", json!({"source": "yes"}), "`source` must be a boolean"),
        ("build", json!({"jobs": 9}), "`jobs` must be at most 4"),
        ("build", json!({"ids": "mailbox"}), "`ids` must be an array"),
        ("preview", json!({"id": "mailbox", "motion": "fast"}), "`motion` must be one of"),
        ("create", json!({"id": "Not_Kebab"}), "`id` must be a kebab-case id"),
        ("context", json!({"query": "pay", "neighbors": ["a", "b", "c", "d", "e"]}), "at most 4"),
    ] {
        let refused = session.exchange("tools/call", json!({"name": name, "arguments": arguments}));
        assert_eq!(refused["error"]["code"], -32602, "{name} {arguments}: {refused}");
        let message = refused["error"]["message"].as_str().unwrap();
        assert!(message.contains(expected), "{name} {arguments}: {message}");
    }
    // Well-formed calls still run.
    let checked =
        session.request("tools/call", json!({"name": "check", "arguments": {"ids": ["mailbox"], "project": true}}));
    assert_eq!(checked["structuredContent"]["status"], "PASS", "{checked}");
}

#[test]
fn lines_that_are_not_requests_are_answered_and_the_server_keeps_serving() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(dir.path(), "unused");
    let error = |message: &Value| (message["id"].clone(), message["error"]["code"].clone());

    // Before the handshake, as after it.
    session.send_line("this is not json");
    assert_eq!(error(&session.next_message()), (json!(null), json!(-32700)));
    let init = session.initialize();
    assert_eq!(init["serverInfo"]["name"], "gen-animated-icons");

    // The id is answered when it can be read before the fault.
    session.send_line(r#"{"jsonrpc": "2.0", "id": 41, "method": "tools/list", "params": {"#);
    assert_eq!(error(&session.next_message()), (json!(41), json!(-32700)));
    session.send_line(r#"{"jsonrpc": "2.0", "id": "q-7", "method": "tools/list"} trailing"#);
    assert_eq!(error(&session.next_message()), (json!("q-7"), json!(-32700)));
    // JSON that is not a request is an invalid request; a notification is never answered.
    session.send_line(r#"{"jsonrpc": "2.0", "id": 42, "method": "tools/call", "params": "search"}"#);
    assert_eq!(error(&session.next_message()), (json!(42), json!(-32600)));
    session.send_line(r#"{"jsonrpc": "2.0", "method": "notifications/progress", "params": 3}"#);
    // A line past the limit is dropped as it arrives, then answered.
    let padding = "x".repeat(gen_animated_icons::mcp::MAX_LINE);
    session.send_line(&format!(
        r#"{{"jsonrpc": "2.0", "id": 43, "method": "tools/list", "params": {{"pad": "{padding}"}}}}"#
    ));
    let long = session.next_message();
    assert_eq!(error(&long), (json!(43), json!(-32700)));
    assert!(long["error"]["message"].as_str().unwrap().contains("longer than"), "{long}");

    let tools = session.request("tools/list", json!({}));
    assert_eq!(tools["tools"].as_array().unwrap().len(), 8);
    assert!(
        session.stray.is_empty() && session.notifications.is_empty(),
        "{:?} {:?}",
        session.stray,
        session.notifications
    );
}

#[test]
fn an_offline_server_never_builds() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(&app, &["init"]).ok();
    run(&app, &["create", "rocket-launch", "--batch", "--library", &library]).ok();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let env = [("GAI_ENGINE_TOKEN", "ci-secret-token"), ("GAI_ENGINE_URL", url.as_str())];
    let mut session = Session::start_with(&app, &["--offline", "mcp", "--library", &library], &env);
    session.initialize();
    let built = session.request("tools/call", json!({"name": "build", "arguments": {"ids": ["rocket-launch"]}}));
    assert_eq!(built["isError"], true, "{built}");
    assert_eq!(built["structuredContent"]["error"]["code"], "NETWORK", "{built}");
    assert!(listener.accept().is_err(), "the engine was contacted");
}
