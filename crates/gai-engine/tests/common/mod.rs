//! A mock of the hosted engine (OAuth server, MCP endpoint, signed downloads) and oracle helpers.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use gai_engine::export::EXPORT_SCRIPT;
use gai_engine::{CredentialStoreKind, EngineConfig, LoginInteraction, RetryPolicy};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::runtime::Runtime;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// A 1×1 PNG, the images a mock draft attaches.
pub const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41,
    0x54, 0x78, 0x9c, 0x63, 0x60, 0x60, 0x60, 0xf8, 0x0f, 0x00, 0x01, 0x04, 0x01, 0x00, 0x5f, 0xe5, 0xc3, 0x4b, 0x00, 0x00,
    0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// The code the fake consent page hands back.
pub const CODE: &str = "the-code";

/// A fault injected into the next `tools/call`.
#[derive(Debug, Clone, Copy)]
pub enum Fault {
    Status(u16),
    Unauthorized,
    DropSession,
}

#[derive(Default)]
pub struct EngineState {
    pub base: String,
    pub registrations: Vec<Value>,
    pub challenge: Option<String>,
    pub invalid_clients: HashSet<String>,
    pub valid_refresh: HashSet<String>,
    pub valid_access: HashSet<String>,
    pub tokens_issued: usize,
    pub refreshes: usize,
    pub refresh_reuses: usize,
    pub revoked: Vec<String>,
    /// Live MCP sessions (one per client).
    pub sessions: HashSet<String>,
    pub sessions_started: usize,
    pub faults: VecDeque<Fault>,
    pub sse: bool,
    /// The status the signed URL answers with instead of the file.
    pub signed_url_status: Option<u16>,
    /// `creator_download_file` omits the `resources/read` fallback.
    pub no_uri: bool,
    pub signed_url_hits: usize,
    pub resource_reads: usize,
    pub tool_calls: Vec<String>,
    pub files: HashMap<(String, String), String>,
    pub documents_used: HashSet<String>,
    pub document_overlaps: usize,
    pub deleted: Vec<String>,
    pub hosted_body: String,
}

#[derive(Clone)]
pub struct Shared(pub Arc<Mutex<EngineState>>);

impl Shared {
    pub fn lock(&self) -> MutexGuard<'_, EngineState> {
        self.0.lock().unwrap()
    }
}

/// The mock engine; the runtime keeps the server alive.
pub struct MockEngine {
    _rt: Runtime,
    pub server: MockServer,
    pub state: Shared,
}

impl MockEngine {
    pub fn start(hosted_body: &str) -> Self {
        let rt = Runtime::new().unwrap();
        let server = rt.block_on(MockServer::start());
        let base = server.uri();
        let state = Shared(Arc::new(Mutex::new(EngineState {
            base: base.clone(),
            hosted_body: hosted_body.to_owned(),
            ..EngineState::default()
        })));
        let mounts = vec![
            Mock::given(method("GET"))
                .and(path("/.well-known/oauth-protected-resource/mcp"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "resource": format!("{base}/mcp"),
                    "authorization_servers": [base],
                    "scopes_supported": ["mcp:full"],
                    "bearer_methods_supported": ["header"],
                }))),
            Mock::given(method("GET"))
                .and(path("/.well-known/oauth-authorization-server"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "issuer": base,
                    "authorization_endpoint": format!("{base}/authorize"),
                    "token_endpoint": format!("{base}/token"),
                    "registration_endpoint": format!("{base}/register"),
                    "revocation_endpoint": format!("{base}/revoke"),
                    "code_challenge_methods_supported": ["S256"],
                    "grant_types_supported": ["authorization_code", "refresh_token"],
                    "token_endpoint_auth_methods_supported": ["none"],
                    "scopes_supported": ["mcp:full"],
                }))),
            Mock::given(method("POST"))
                .and(path("/register"))
                .respond_with(Register(state.clone())),
            Mock::given(method("POST"))
                .and(path("/token"))
                .respond_with(Token(state.clone())),
            Mock::given(method("POST"))
                .and(path("/revoke"))
                .respond_with(Revoke(state.clone())),
            Mock::given(method("POST"))
                .and(path("/mcp"))
                .respond_with(Mcp(state.clone())),
            Mock::given(method("GET"))
                .and(path_regex("^/r2/"))
                .respond_with(SignedUrl(state.clone())),
        ];
        for mock in mounts {
            rt.block_on(mock.mount(&server));
        }
        MockEngine {
            _rt: rt,
            server,
            state,
        }
    }

    /// Settings for this engine with file credentials under `home` and fast retries.
    pub fn config(&self, home: &Path) -> EngineConfig {
        let mut config = EngineConfig::new(&format!("{}/mcp", self.server.uri()), home).unwrap();
        config.credential_store = CredentialStoreKind::File;
        config.retry = RetryPolicy {
            attempts: 4,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(20),
        };
        config
    }
}

fn form(request: &Request) -> HashMap<String, String> {
    url::form_urlencoded::parse(&request.body)
        .into_owned()
        .collect()
}

fn oauth_error(status: u16, error: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({"error": error}))
}

struct Register(Shared);

impl Respond for Register {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        let mut state = self.0.lock();
        state.registrations.push(body.clone());
        let client_id = format!("client-{}", state.registrations.len());
        ResponseTemplate::new(201).set_body_json(json!({
            "client_id": client_id,
            "client_id_issued_at": 1_790_000_000u64,
            "redirect_uris": body["redirect_uris"],
            "token_endpoint_auth_method": "none",
        }))
    }
}

struct Token(Shared);

impl Respond for Token {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let form = form(request);
        let mut state = self.0.lock();
        let client = form.get("client_id").cloned().unwrap_or_default();
        if state.invalid_clients.contains(&client) {
            return oauth_error(401, "invalid_client");
        }
        if form.get("resource").map(String::as_str) != Some(&format!("{}/mcp", state.base)) {
            return oauth_error(400, "invalid_target");
        }
        match form.get("grant_type").map(String::as_str) {
            Some("authorization_code") => {
                let verifier = form.get("code_verifier").cloned().unwrap_or_default();
                let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
                if form.get("code").map(String::as_str) != Some(CODE)
                    || state.challenge.as_deref() != Some(&challenge)
                {
                    return oauth_error(400, "invalid_grant");
                }
            }
            Some("refresh_token") => {
                let presented = form.get("refresh_token").cloned().unwrap_or_default();
                if !state.valid_refresh.remove(&presented) {
                    state.refresh_reuses += 1;
                    state.valid_refresh.clear();
                    return oauth_error(400, "invalid_grant");
                }
                state.refreshes += 1;
            }
            _ => return oauth_error(400, "unsupported_grant_type"),
        }
        state.tokens_issued += 1;
        let n = state.tokens_issued;
        let claims = URL_SAFE_NO_PAD.encode(format!(
            r#"{{"sub":"user-1","email":"designer@example.com","n":{n}}}"#
        ));
        let access = format!("eyJhbGciOiJub25lIn0.{claims}.sig{n}");
        let refresh = format!("rt-{n}");
        state.valid_access.insert(access.clone());
        state.valid_refresh.insert(refresh.clone());
        ResponseTemplate::new(200).set_body_json(json!({
            "access_token": access, "token_type": "bearer", "expires_in": 300,
            "refresh_token": refresh, "scope": "mcp:full",
        }))
    }
}

struct Revoke(Shared);

impl Respond for Revoke {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let token = form(request).remove("token").unwrap_or_default();
        let mut state = self.0.lock();
        state.valid_refresh.remove(&token);
        state.revoked.push(token);
        ResponseTemplate::new(200)
    }
}

struct Mcp(Shared);

impl Mcp {
    fn reply(&self, state: &EngineState, id: &Value, result: Value) -> ResponseTemplate {
        let message = json!({"jsonrpc": "2.0", "id": id, "result": result});
        if state.sse {
            let body = format!(
                "event: message\ndata: {}\n\n",
                json!({"jsonrpc": "2.0", "method": "notifications/message", "params": {"level": "info"}})
            ) + &format!("event: message\ndata: {message}\n\n");
            ResponseTemplate::new(200).set_body_raw(body, "text/event-stream")
        } else {
            ResponseTemplate::new(200).set_body_json(message)
        }
    }

    fn tool(&self, state: &mut EngineState, name: &str, args: &Value) -> (bool, Value) {
        state.tool_calls.push(name.to_owned());
        let doc = args["documentId"].as_str().unwrap_or("default").to_owned();
        let file = args["file"]
            .as_str()
            .or(args.pointer("/params/file").and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned();
        match name {
            // Spec v2: one plan script per icon; `file` stores the bundle, `sheet` attaches PNGs.
            "creator_exec" if args.pointer("/params/plan").is_some() => {
                assert_eq!(args["timeoutMs"], 30000);
                state.documents_used.insert(doc.clone());
                let mut stored = Vec::new();
                if !file.is_empty() {
                    let bytes = state.hosted_body.len();
                    state.files.insert((doc.clone(), file.clone()), state.hosted_body.clone());
                    stored.push(json!({"bytes": bytes, "file": file, "format": "lottie", "kind": "export", "status": "ready"}));
                }
                let sheets = if args.pointer("/params/sheet").is_some() { 2 } else { 0 };
                (
                    false,
                    json!({
                        "documentId": doc, "durability": "durable", "logs": [], "revision": "3",
                        "result": {"stored": stored, "sheets": vec![json!({"tiles": 1}); sheets], "timing": {"totalMs": 42}},
                    }),
                )
            }
            "creator_exec" => {
                assert_eq!(args["code"].as_str(), Some(EXPORT_SCRIPT));
                assert!(
                    args.pointer("/params/scene/nodes")
                        .is_some_and(Value::is_array)
                );
                assert_eq!(args["timeoutMs"], 30000);
                if state.files.keys().any(|(d, _)| *d == doc) {
                    state.document_overlaps += 1;
                }
                state.documents_used.insert(doc.clone());
                let bytes = state.hosted_body.len();
                state
                    .files
                    .insert((doc.clone(), file.clone()), state.hosted_body.clone());
                (
                    false,
                    json!({
                        "documentId": doc, "durability": "durable", "logs": [], "revision": "7",
                        "outputs": [{"file": file, "status": "ready", "bytes": bytes}],
                        "result": {"bytes": bytes, "file": file, "format": "lottie", "height": 32, "kind": "export", "status": "ready", "width": 32},
                    }),
                )
            }
            "creator_download_file" => match state.files.get(&(doc.clone(), file.clone())) {
                Some(body) => {
                    let mut located = json!({
                        "bytes": body.len(), "documentId": doc, "file": file, "status": "ready",
                        "downloadUrl": format!("{}/r2/{doc}/{file}?X-Amz-Signature=abc", state.base),
                        "uri": format!("creator-mcp://artifacts/{doc}/{file}"),
                    });
                    if state.no_uri {
                        located.as_object_mut().unwrap().remove("uri");
                    }
                    (false, located)
                }
                None => (
                    true,
                    Value::String(format!("no output file named {file} in document {doc}")),
                ),
            },
            "creator_delete_file" => {
                state.files.remove(&(doc, file.clone()));
                state.deleted.push(file.clone());
                (false, json!({"ok": true, "file": file}))
            }
            other => (true, Value::String(format!("unknown tool {other}"))),
        }
    }
}

impl Respond for Mcp {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        let mut state = self.0.lock();
        let header = |name: &str| {
            request
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let bearer = header("authorization").unwrap_or_default();
        if !state
            .valid_access
            .contains(bearer.trim_start_matches("Bearer "))
        {
            return ResponseTemplate::new(401)
                .insert_header("www-authenticate", "Bearer error=\"invalid_token\"");
        }
        let method = body["method"].as_str().unwrap_or_default().to_owned();
        let id = body["id"].clone();
        if method == "initialize" {
            assert_eq!(
                body.pointer("/params/protocolVersion"),
                Some(&json!("2025-11-25"))
            );
            state.sessions_started += 1;
            let session = format!("session-{}", state.sessions_started);
            state.sessions.insert(session.clone());
            let result = json!({
                "protocolVersion": "2025-11-25", "capabilities": {"tools": {}, "resources": {}},
                "serverInfo": {"name": "creator-mcp", "version": "0.2.0"},
            });
            return self
                .reply(&state, &id, result)
                .insert_header("mcp-session-id", session.as_str());
        }
        if !header("mcp-session-id").is_some_and(|id| state.sessions.contains(&id)) {
            return ResponseTemplate::new(404);
        }
        assert_eq!(
            header("mcp-protocol-version").as_deref(),
            Some("2025-11-25")
        );
        if method == "notifications/initialized" {
            return ResponseTemplate::new(202);
        }
        if method == "tools/call" {
            match state.faults.pop_front() {
                Some(Fault::Status(status)) => {
                    return ResponseTemplate::new(status).insert_header("retry-after", "0");
                }
                Some(Fault::Unauthorized) => {
                    state
                        .valid_access
                        .remove(bearer.trim_start_matches("Bearer "));
                    return ResponseTemplate::new(401);
                }
                Some(Fault::DropSession) => {
                    state.sessions.clear();
                    return ResponseTemplate::new(404);
                }
                None => {}
            }
            let name = body
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let args = body
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or(Value::Null);
            let (is_error, payload) = self.tool(&mut state, &name, &args);
            let text = match &payload {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            let mut content = vec![json!({"type": "text", "text": text})];
            if name == "creator_exec" && args.pointer("/params/sheet").is_some() {
                use base64::Engine as _;
                for _ in 0..2 {
                    let png = base64::engine::general_purpose::STANDARD.encode(TINY_PNG);
                    content.push(json!({"type": "image", "data": png, "mimeType": "image/png"}));
                }
            }
            let result = json!({"content": content, "isError": is_error});
            return self.reply(&state, &id, result);
        }
        if method == "resources/read" {
            state.resource_reads += 1;
            let uri = body
                .pointer("/params/uri")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let key = uri
                .trim_start_matches("creator-mcp://artifacts/")
                .split_once('/')
                .map(|(d, f)| (d.to_owned(), f.to_owned()));
            let text = key
                .and_then(|k| state.files.get(&k).cloned())
                .unwrap_or_default();
            let result =
                json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]});
            return self.reply(&state, &id, result);
        }
        ResponseTemplate::new(200).set_body_json(json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "method not found"}}))
    }
}

struct SignedUrl(Shared);

impl Respond for SignedUrl {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let mut state = self.0.lock();
        state.signed_url_hits += 1;
        assert!(
            request.headers.get("authorization").is_none(),
            "credentials sent to the signed URL"
        );
        if let Some(status) = state.signed_url_status {
            return ResponseTemplate::new(status);
        }
        let key = request
            .url
            .path()
            .trim_start_matches("/r2/")
            .split_once('/')
            .map(|(d, f)| (d.to_owned(), f.to_owned()));
        match key.and_then(|k| state.files.get(&k).cloned()) {
            Some(body) => ResponseTemplate::new(200).set_body_string(body),
            None => ResponseTemplate::new(404),
        }
    }
}

/// Plays the browser: records the PKCE challenge, then follows the redirect with the code
/// (loopback) or hands back the redirect address (paste).
pub struct FakeBrowser {
    pub state: Shared,
    pub paste: bool,
    pub shown: Vec<String>,
    redirect: Option<String>,
}

impl FakeBrowser {
    pub fn new(state: Shared, paste: bool) -> Self {
        FakeBrowser {
            state,
            paste,
            shown: Vec::new(),
            redirect: None,
        }
    }
}

impl LoginInteraction for FakeBrowser {
    fn show_authorization_url(&mut self, url: &str, browser_opened: bool) {
        assert!(!browser_opened);
        self.shown.push(url.to_owned());
        let url = url::Url::parse(url).unwrap();
        let q: HashMap<String, String> = url.query_pairs().into_owned().collect();
        assert_eq!(q["code_challenge_method"], "S256");
        assert_eq!(q["scope"], "mcp:full");
        self.state.lock().challenge = Some(q["code_challenge"].clone());
        let redirect = format!("{}?code={CODE}&state={}", q["redirect_uri"], q["state"]);
        if self.paste {
            self.redirect = Some(redirect);
        } else {
            std::thread::spawn(move || {
                reqwest::blocking::get(redirect).unwrap();
            });
        }
    }

    fn read_redirect_url(&mut self) -> std::io::Result<String> {
        Ok(self
            .redirect
            .take()
            .expect("paste mode shows the URL first"))
    }
}

/// The repo root, when this checkout has one (`GAI_LIBRARY_ROOT` or an ancestor holding
/// `motion/icon-sets`).
pub fn library_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("GAI_LIBRARY_ROOT") {
        return Some(PathBuf::from(root));
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|dir| dir.join("motion/icon-sets").is_dir())
        .map(Path::to_path_buf)
}

/// `motion/icon-sets/<set>/native/<name>.json`, when present (the natives are gitignored).
pub fn native_oracle(set: &str, name: &str) -> Option<String> {
    let path = library_root()?
        .join("motion/icon-sets")
        .join(set)
        .join("native")
        .join(format!("{name}.json"));
    std::fs::read_to_string(path).ok()
}

/// The hosted fixtures captured from creator-mcp 0.2.0: (set, name).
pub const HOSTED_FIXTURES: [(&str, &str); 3] = [
    ("06-tools", "filter-strong"),
    ("11-feedback-status", "cancelled-reset"),
    ("11-feedback-status", "empty-state-reset-strong"),
];

pub fn hosted_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.hosted.json"));
    std::fs::read_to_string(path).unwrap()
}

/// `native` with each `ty:"sh"` shape's `nm` replaced by the matching one in `normalized`,
/// written with serde_json: the only field the hosted exporter names differently.
pub fn with_hosted_path_names(native: &str, normalized: &str) -> String {
    let mut native: Value = serde_json::from_str(native).unwrap();
    let normalized: Value = serde_json::from_str(normalized).unwrap();
    let names: Vec<Value> = shape_names(&normalized);
    let mut names = names.into_iter();
    for layer in native["layers"].as_array_mut().unwrap() {
        for shape in layer["shapes"].as_array_mut().unwrap() {
            if shape["ty"] == "sh" {
                assert!(
                    shape["nm"].as_str().unwrap().starts_with("native-export-"),
                    "{}",
                    shape["nm"]
                );
                shape["nm"] = names.next().unwrap();
            }
        }
    }
    assert!(names.next().is_none());
    serde_json::to_string(&native).unwrap()
}

fn shape_names(lottie: &Value) -> Vec<Value> {
    lottie["layers"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|layer| layer["shapes"].as_array().unwrap().iter())
        .filter(|shape| shape["ty"] == "sh")
        .map(|shape| shape["nm"].clone())
        .collect()
}
