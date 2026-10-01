//! A minimal MCP client over streamable HTTP (JSON-RPC 2.0 POSTs whose answer is JSON or an
//! SSE stream). It covers what the exporter needs: `initialize`, `tools/call` and
//! `resources/read`, with the session header, bearer auth, a token refresh on 401, and
//! backoff on 429, 5xx and network failures.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{Value, json};
use url::Url;

use crate::auth::Auth;
use crate::config::{EngineConfig, RetryPolicy};
use crate::error::{Error, Result};
use crate::http;

/// The newest protocol revision that still has the `initialize` handshake. The engine also
/// speaks the stateless 2026-07-28 revision, which has no sessions.
pub const PROTOCOL_VERSION: &str = "2025-11-25";
const SESSION_HEADER: &str = "mcp-session-id";
const PROTOCOL_HEADER: &str = "mcp-protocol-version";

/// Who answered `initialize`; recorded in export provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// `serverInfo.name`, e.g. `creator-mcp`.
    pub name: String,
    /// `serverInfo.version`, e.g. `0.2.0`.
    pub version: String,
    /// The protocol version the server chose.
    pub protocol_version: String,
}

#[derive(Debug, Clone)]
struct Session {
    id: Option<String>,
    info: ServerInfo,
}

/// A blocking MCP client for the hosted engine. Cheap to share across threads.
pub struct McpClient {
    url: Url,
    http: Client,
    auth: Auth,
    retry: RetryPolicy,
    next_id: AtomicU64,
    session: Mutex<Option<Session>>,
}

/// How one POST ended, before JSON-RPC parsing.
enum Posted {
    Answer(Response),
    /// 404 on a request that carried a session id: the server dropped the session.
    SessionGone,
}

impl McpClient {
    /// A client for `config.url`. Nothing is sent until the first call.
    pub fn new(config: &EngineConfig) -> Result<Self> {
        Ok(Self::with_client(config, http::client()?))
    }

    /// A client for `config.url` that sends through `http`.
    pub(crate) fn with_client(config: &EngineConfig, http: Client) -> Self {
        McpClient {
            url: config.url.clone(),
            auth: Auth::new(config, http.clone()),
            http,
            retry: config.retry,
            next_id: AtomicU64::new(1),
            session: Mutex::new(None),
        }
    }

    /// The server's identity, from `initialize` (run on first use).
    pub fn server_info(&self) -> Result<ServerInfo> {
        Ok(self.session()?.info)
    }

    /// Calls a tool and returns its payload: `structuredContent` when present, otherwise the
    /// first text item parsed as JSON (or kept as a string). `isError` becomes [`Error::Tool`].
    pub fn call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        let result = self.request("tools/call", json!({"name": name, "arguments": arguments}))?;
        let text = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item.get("type").and_then(Value::as_str) == Some("text"))
            })
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str);
        let payload = match (
            result.get("structuredContent").filter(|v| v.is_object()),
            text,
        ) {
            (Some(structured), _) => structured.clone(),
            (None, Some(text)) => {
                serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
            }
            (None, None) => Value::Null,
        };
        if result.get("isError").and_then(Value::as_bool) == Some(true) {
            return Err(tool_error(name, &payload));
        }
        Ok(payload)
    }

    /// Reads a resource and returns its bytes (`text` as UTF-8, or `blob` decoded).
    pub fn read_resource(&self, uri: &str) -> Result<Vec<u8>> {
        let result = self.request("resources/read", json!({"uri": uri}))?;
        let item = result
            .get("contents")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .ok_or_else(|| Error::Protocol(format!("resources/read {uri} returned no contents")))?;
        if let Some(text) = item.get("text").and_then(Value::as_str) {
            return Ok(text.as_bytes().to_vec());
        }
        let blob = item.get("blob").and_then(Value::as_str).ok_or_else(|| {
            Error::Protocol(format!("resources/read {uri}: neither text nor blob"))
        })?;
        STANDARD
            .decode(blob)
            .map_err(|e| Error::Protocol(format!("resources/read {uri}: {e}")))
    }

    /// Sends a request in the current session, re-initializing once if the server dropped it.
    fn request(&self, method: &str, params: Value) -> Result<Value> {
        for _ in 0..2 {
            let session = self.session()?;
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let body = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
            match self.post(&body, Some(&session))? {
                Posted::Answer(response) => return rpc_result(method, id, response),
                Posted::SessionGone => {
                    let mut current = self.session.lock().unwrap_or_else(|e| e.into_inner());
                    if current.as_ref().map(|s| &s.id) == Some(&session.id) {
                        *current = None;
                    }
                }
            }
        }
        Err(Error::Protocol(format!(
            "{method}: the engine dropped the session twice"
        )))
    }

    /// The current session, running `initialize` and `notifications/initialized` if needed.
    fn session(&self) -> Result<Session> {
        let mut current = self.session.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(session) = current.as_ref() {
            return Ok(session.clone());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let body = json!({
            "jsonrpc": "2.0", "id": id, "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "gen-animated-icons", "version": env!("CARGO_PKG_VERSION")},
            },
        });
        let Posted::Answer(response) = self.post(&body, None)? else {
            return Err(Error::Protocol(
                "initialize: the engine reported a lost session".into(),
            ));
        };
        let session_id = response
            .headers()
            .get(SESSION_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let result = rpc_result("initialize", id, response)?;
        let server = result
            .get("serverInfo")
            .or_else(|| result.pointer("/_meta/io.modelcontextprotocol~1serverInfo"))
            .ok_or_else(|| Error::Protocol("initialize returned no serverInfo".into()))?;
        let text = |v: Option<&Value>| v.and_then(Value::as_str).unwrap_or_default().to_owned();
        let session = Session {
            id: session_id,
            info: ServerInfo {
                name: text(server.get("name")),
                version: text(server.get("version")),
                protocol_version: result
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .unwrap_or(PROTOCOL_VERSION)
                    .to_owned(),
            },
        };
        let notice = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        if let Posted::Answer(response) = self.post(&notice, Some(&session))? {
            // Only the status matters for a notification; drain the body.
            let _ = response.bytes();
        }
        *current = Some(session.clone());
        Ok(session)
    }

    /// POSTs one JSON-RPC message with auth and session headers, retrying what is worth it.
    fn post(&self, body: &Value, session: Option<&Session>) -> Result<Posted> {
        let payload = body.to_string();
        let mut token = self.auth.access_token()?;
        let mut renewed = false;
        let mut attempt = 1;
        loop {
            let mut request = self
                .http
                .post(self.url.clone())
                .bearer_auth(&token)
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json, text/event-stream")
                .body(payload.clone());
            if let Some(session) = session {
                request = request.header(PROTOCOL_HEADER, &session.info.protocol_version);
                if let Some(id) = &session.id {
                    request = request.header(SESSION_HEADER, id);
                }
            }
            let response = match request.send() {
                Ok(response) => response,
                Err(e) if attempt < self.retry.attempts && http::is_transient_error(&e) => {
                    thread::sleep(http::backoff(&self.retry, attempt, None));
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            let status = response.status().as_u16();
            match status {
                200..=299 => return Ok(Posted::Answer(response)),
                401 if !renewed => {
                    token = self.auth.renew_rejected(&token)?;
                    renewed = true;
                }
                401 => {
                    return Err(Error::SessionExpired(
                        "the engine rejected a fresh access token".into(),
                    ));
                }
                404 if session.is_some_and(|s| s.id.is_some()) => return Ok(Posted::SessionGone),
                _ if http::is_transient(status) && attempt < self.retry.attempts => {
                    thread::sleep(http::backoff(
                        &self.retry,
                        attempt,
                        Some(response.headers()),
                    ));
                    attempt += 1;
                }
                _ => return Err(http::status_error(response)),
            }
        }
    }
}

/// Finds the response to `id` in a JSON or SSE body and returns its `result`.
fn rpc_result(method: &str, id: u64, response: Response) -> Result<Value> {
    let is_sse = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/event-stream"));
    let text = response.text()?;
    let messages = if is_sse {
        sse_data(&text)
            .into_iter()
            .filter_map(|data| serde_json::from_str::<Value>(&data).ok())
            .collect()
    } else {
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Array(batch)) => batch,
            Ok(message) => vec![message],
            Err(e) => return Err(Error::Protocol(format!("{method}: {e}"))),
        }
    };
    let message = messages
        .into_iter()
        .find(|m| m.get("id").and_then(Value::as_u64) == Some(id))
        .ok_or_else(|| Error::Protocol(format!("{method}: no response with id {id}")))?;
    if let Some(error) = message.get("error") {
        return Err(Error::Rpc {
            method: method.to_owned(),
            code: error.get("code").and_then(Value::as_i64).unwrap_or(0),
            message: error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        });
    }
    message
        .get("result")
        .cloned()
        .ok_or_else(|| Error::Protocol(format!("{method}: response has neither result nor error")))
}

/// The `data` of each event in an SSE body (multi-line data joined with `\n`).
fn sse_data(body: &str) -> Vec<String> {
    let mut events = Vec::new();
    let mut data: Vec<&str> = Vec::new();
    for line in body.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if !data.is_empty() {
                events.push(data.join("\n"));
                data.clear();
            }
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    events
}

/// A tool error from the engine's payload: `{result: {error: {code, message, retryable}}}` for
/// `creator_exec`, plain text for the file tools.
fn tool_error(tool: &str, payload: &Value) -> Error {
    let error = payload
        .pointer("/result/error")
        .or_else(|| payload.get("error"))
        .filter(|e| e.is_object());
    match error {
        Some(error) => {
            let field = |key: &str| error.get(key).and_then(Value::as_str).unwrap_or_default();
            let mut message = field("message").to_owned();
            if let Some(line) = error.get("line").and_then(Value::as_u64) {
                message.push_str(&format!(" (line {line})"));
            }
            if !field("hint").is_empty() {
                message.push_str(&format!("; hint: {}", field("hint")));
            }
            Error::Tool {
                tool: tool.to_owned(),
                code: field("code").to_owned(),
                message,
                retryable: error
                    .get("retryable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            }
        }
        None => Error::Tool {
            tool: tool.to_owned(),
            code: "tool_error".into(),
            message: match payload {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            },
            retryable: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_events_are_split_and_joined() {
        let body =
            "event: message\ndata: {\"a\":1}\n\n: comment\ndata:{\"b\":\ndata: 2}\n\ndata: tail";
        assert_eq!(sse_data(body), ["{\"a\":1}", "{\"b\":\n2}", "tail"]);
    }

    #[test]
    fn tool_errors_from_exec_payloads_and_text() {
        let payload = json!({"result": {"error": {"code": "script_error", "message": "boom", "retryable": false, "line": 3}}});
        match tool_error("creator_exec", &payload) {
            Error::Tool {
                code,
                message,
                retryable,
                ..
            } => {
                assert_eq!(code, "script_error");
                assert_eq!(message, "boom (line 3)");
                assert!(!retryable);
            }
            other => panic!("{other}"),
        }
        let text = Value::String("no output file named a.json".into());
        assert!(
            matches!(tool_error("creator_delete_file", &text), Error::Tool { code, .. } if code == "tool_error")
        );
    }
}
