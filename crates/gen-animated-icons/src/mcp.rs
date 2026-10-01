//! `mcp`: a stdio MCP server that gives agents this CLI's task-shaped commands:
//! `search`, `add`, `create`, `context`, `check`, `build` and `preview`.
//!
//! Each tool call runs the command in a child process of this binary with `--json`, so
//! a renderer fault or a stuck engine call ends that call, not the server, and icon code
//! never runs in the server itself: the child, like any one-shot command, ends itself
//! with `SOURCE_ERROR` when that code runs past its time limit ([`crate::protect`]). The result
//! is the command's JSON as text plus `structuredContent`, then its warnings as text;
//! `preview` also returns the rendered sheets as PNG images. `submit` and `login` are not
//! exposed: both need the user.
//!
//! A call's arguments must match the tool's input schema exactly ([`validate`]): an
//! unknown name, a missing required one or a value of the wrong type or range is refused
//! with `invalid_params` naming the arguments the tool takes, before anything runs.
//!
//! A call's `cwd` must stay inside the server's folder. When the request carries a
//! progress token, the command runs with `--verbose` and each log line it prints (one
//! per exported action during `build`) is sent as a progress notification, which keeps
//! clients that reset their timeout on progress waiting. A call is stopped after
//! [`CALL_TIMEOUT`]. A server started with `--offline` passes it to every command, so
//! `build` is refused with `NETWORK`.
//!
//! Messages are one JSON-RPC message per line ([`LineTransport`]). A line that is not one
//! is answered, and the server keeps reading: -32700 for a line that is not JSON or is
//! longer than [`MAX_LINE`] bytes, -32600 for JSON that is not a request, each with the
//! request's id when one can be read, else `null`.

use crate::Context;
use crate::cli::MAX_COLUMNS;
use crate::error::{CliError, ErrorCode, Result};
use base64::Engine as _;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, ListToolsResult,
    PaginatedRequestParams, ProgressNotificationParam, ServerCapabilities, ServerConfig, Tool, ToolAnnotations,
};
use rmcp::service::{RequestContext, RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value, json};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};

/// Sheets `preview` returns as images at most.
const MAX_IMAGES: usize = 4;

/// Longest a tool call may run before its command is stopped.
pub const CALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// Longest request line the server reads, in bytes; the rest of a longer one is dropped
/// as it arrives, and the line is answered with a parse error.
pub const MAX_LINE: usize = 1 << 20;

/// JSON-RPC: the line is not JSON.
const PARSE_ERROR: i64 = -32700;
/// JSON-RPC: the JSON is not a request.
const INVALID_REQUEST: i64 = -32600;

const INSTRUCTIONS: &str = "gen-animated-icons finds, creates and builds animated icons (dotLottie) for the project in the working folder. \
Look for an existing icon first: search, then add. To make a missing icon: context (read the contract and the nearest references), \
create (scaffold a batch, optionally --from a close library icon), edit the batch file it names, check, build, then preview and look at the images. \
Revise and rebuild until the sheets look right. Never claim a visual review you did not do.";

/// The MCP settings entry that starts this server; `init` prints it for the user to add
/// and never writes it.
pub fn client_config() -> Value {
    json!({ "mcpServers": { "gen-animated-icons": { "command": "npx", "args": ["gen-animated-icons", "mcp"] } } })
}

/// The entry that starts this very binary by its absolute path, which works before the
/// npm package is published; `None` when the executable's path cannot be told.
pub fn local_client_config() -> Option<Value> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    Some(json!({ "mcpServers": { "gen-animated-icons": { "command": exe.to_str()?, "args": ["mcp"] } } }))
}

/// Runs the server on stdin and stdout until the client disconnects.
pub fn serve(ctx: &Context) -> Result<()> {
    let exe = std::env::current_exe()
        .map_err(|error| CliError::new(ErrorCode::Internal, format!("Cannot find this executable: {error}")))?;
    let server = Server { exe, cwd: ctx.cwd.clone(), settings: settings(ctx), tools: Arc::new(tools()) };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| CliError::new(ErrorCode::Internal, format!("Cannot start the MCP server: {error}")))?;
    runtime.block_on(async move {
        let running = server
            .serve(LineTransport::new(tokio::io::stdin(), tokio::io::stdout(), MAX_LINE))
            .await
            .map_err(|error| CliError::new(ErrorCode::Internal, format!("MCP initialization failed: {error}")))?;
        running
            .waiting()
            .await
            .map_err(|error| CliError::new(ErrorCode::Internal, format!("The MCP server stopped: {error}")))?;
        Ok(())
    })
}

/// Line-delimited JSON-RPC over a reader and a writer, as `rmcp::transport::stdio` reads
/// it, except that a line that is not a message is answered rather than dropped (see the
/// module notes) and no line is held past `max` bytes.
struct LineTransport<R, W> {
    read: tokio::io::BufReader<R>,
    /// The line so far. A `receive` that is cancelled between reads keeps it.
    line: Vec<u8>,
    /// Whether the line ran past `max`; the rest of it is dropped as it arrives.
    long: bool,
    max: usize,
    /// `None` once closed. Each line is written whole under the lock.
    write: Arc<tokio::sync::Mutex<Option<W>>>,
}

impl<R: AsyncRead, W> LineTransport<R, W> {
    fn new(read: R, write: W, max: usize) -> Self {
        let write = Arc::new(tokio::sync::Mutex::new(Some(write)));
        LineTransport { read: tokio::io::BufReader::new(read), line: Vec::new(), long: false, max, write }
    }
}

/// Writes `line` whole, then flushes.
async fn write_line<W: AsyncWrite + Unpin>(write: &tokio::sync::Mutex<Option<W>>, line: &[u8]) -> std::io::Result<()> {
    let mut guard = write.lock().await;
    let writer = guard
        .as_mut()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotConnected, "the transport is closed"))?;
    writer.write_all(line).await?;
    writer.flush().await
}

impl<R, W> rmcp::transport::Transport<RoleServer> for LineTransport<R, W>
where
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send + 'static,
{
    type Error = std::io::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleServer>,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static {
        let write = self.write.clone();
        let line = serde_json::to_vec(&item).map(|mut line| {
            line.push(b'\n');
            line
        });
        async move { write_line(&write, &line?).await }
    }

    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleServer>> {
        loop {
            // A cancelled `fill_buf` loses nothing, and no other wait comes before the line
            // it completes is taken.
            let buffer = self.read.fill_buf().await.ok()?;
            if buffer.is_empty() {
                return None;
            }
            let end = buffer.iter().position(|&byte| byte == b'\n');
            let chunk = &buffer[..end.unwrap_or(buffer.len())];
            let room = self.max.saturating_sub(self.line.len());
            self.long |= chunk.len() > room;
            self.line.extend_from_slice(&chunk[..chunk.len().min(room)]);
            let used = end.map_or(buffer.len(), |end| end + 1);
            self.read.consume(used);
            if end.is_none() {
                continue;
            }
            let line = std::mem::take(&mut self.line);
            let long = std::mem::take(&mut self.long).then_some(self.max);
            match read_line(&line, long) {
                Line::Message(message) => return Some(*message),
                Line::Skip => {}
                Line::Answer(answer) => {
                    // Written apart from this call, which may be cancelled at any wait, and
                    // given its turn now so it goes out before what later lines bring.
                    let write = self.write.clone();
                    tokio::spawn(async move { write_line(&write, &answer).await });
                    tokio::task::yield_now().await;
                }
            }
        }
    }

    async fn close(&mut self) -> std::io::Result<()> {
        self.write.lock().await.take();
        Ok(())
    }
}

/// What one line holds.
enum Line {
    Message(Box<RxJsonRpcMessage<RoleServer>>),
    /// A reply to write (with its `\n`).
    Answer(Vec<u8>),
    /// Nothing to answer: a blank line, or a notification or response that cannot be read.
    Skip,
}

/// Reads one line, without its `\n`; `long` is the limit it ran past, when it did.
fn read_line(line: &[u8], long: Option<usize>) -> Line {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let line = line.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(line);
    if let Some(max) = long {
        let message = format!("Parse error: the line is longer than {max} bytes");
        return Line::Answer(failure(PARSE_ERROR, &message, recovered_id(line)));
    }
    if line.iter().all(u8::is_ascii_whitespace) {
        return Line::Skip;
    }
    let error = match serde_json::from_slice(line) {
        Ok(message) => return Line::Message(message),
        Err(error) => error,
    };
    let Ok(value) = serde_json::from_slice::<Value>(line) else {
        return Line::Answer(failure(PARSE_ERROR, &format!("Parse error: {error}"), recovered_id(line)));
    };
    let has = |key: &str| value.get(key).is_some();
    // Neither a notification nor a response is ever answered.
    if (has("method") && !has("id")) || (!has("method") && (has("result") || has("error"))) {
        return Line::Skip;
    }
    let id = value.get("id").filter(|id| id.is_string() || id.is_number()).cloned().unwrap_or(Value::Null);
    Line::Answer(failure(INVALID_REQUEST, &format!("Invalid request: {error}"), id))
}

/// A JSON-RPC error response line.
fn failure(code: i64, message: &str, id: Value) -> Vec<u8> {
    let response = json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } });
    format!("{response}\n").into_bytes()
}

/// The `id` of a top-level object read up to the first fault in `line`: a string, or a
/// number the parser read past (so not one cut short); `null` when there is none.
fn recovered_id(line: &[u8]) -> Value {
    struct Id<'a>(&'a mut Value);
    impl<'de> serde::de::Visitor<'de> for Id<'_> {
        type Value = ();
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a request object")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
            let mut number = None;
            while let Some(key) = map.next_key::<String>()? {
                if let Some(id) = number.take() {
                    *self.0 = id;
                }
                if key != "id" || !self.0.is_null() {
                    map.next_value::<serde::de::IgnoredAny>()?;
                    continue;
                }
                match map.next_value::<Value>()? {
                    id @ Value::String(_) => *self.0 = id,
                    id @ Value::Number(_) => number = Some(id),
                    _ => {}
                }
            }
            if let Some(id) = number {
                *self.0 = id;
            }
            Ok(())
        }
    }
    let mut id = Value::Null;
    let _ = serde::Deserializer::deserialize_map(&mut serde_json::Deserializer::from_slice(line), Id(&mut id));
    id
}

#[derive(Clone)]
struct Server {
    exe: PathBuf,
    cwd: PathBuf,
    /// Global options the server was started with, repeated for every command.
    settings: Vec<OsString>,
    tools: Arc<Vec<ToolSpec>>,
}

/// The global options of `ctx` as command-line arguments. Local paths are made absolute,
/// since each call may run in another folder (its `cwd` argument).
fn settings(ctx: &Context) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    let mut option = |name: &str, value: Option<OsString>| {
        if let Some(value) = value {
            args.extend([name.into(), value]);
        }
    };
    let library = ctx.library.as_ref().map(|location| {
        match location.starts_with("http://") || location.starts_with("https://") {
            true => location.into(),
            false => ctx.cwd.join(location).into_os_string(),
        }
    });
    option("--library", library);
    option("--library-root", ctx.library_root.as_ref().map(|root| ctx.cwd.join(root).into_os_string()));
    option("--engine-url", ctx.engine_url.clone().map(Into::into));
    option("--platform", ctx.platform.map(|platform| platform.as_str().into()));
    if ctx.offline {
        args.push("--offline".into());
    }
    args
}

/// A tool: its MCP definition and how its arguments become a command line. Values go
/// as `--option=value` and positionals after `--`, so no argument is read as a flag.
struct ToolSpec {
    tool: Tool,
    argv: fn(&Map<String, Value>) -> Parsed<Vec<String>>,
    images: bool,
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("gen-animated-icons", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools.iter().map(|spec| spec.tool.clone()).collect()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        let Some(spec) = self.tools.iter().find(|spec| spec.tool.name == request.name) else {
            return Err(ErrorData::invalid_params(format!("Unknown tool {}", request.name), None));
        };
        let arguments = request.arguments.unwrap_or_default();
        validate(&spec.tool, &arguments).map_err(|message| ErrorData::invalid_params(message, None))?;
        let argv = (spec.argv)(&arguments).map_err(|message| ErrorData::invalid_params(message, None))?;
        let cwd = call_folder(&self.cwd, arguments.get("cwd").and_then(Value::as_str))
            .map_err(|message| ErrorData::invalid_params(message, None))?;
        let progress = context.meta.get_progress_token();
        let mut command = tokio::process::Command::new(&self.exe);
        command.arg("--json").arg("--cwd").arg(&cwd);
        if progress.is_some() {
            command.arg("--verbose");
        }
        command.args(&self.settings).args(&argv).stdin(std::process::Stdio::null()).kill_on_drop(true);
        let notify = |count: u32, message: String| {
            let peer = context.peer.clone();
            let token = progress.clone();
            async move {
                if let Some(token) = token {
                    let note = ProgressNotificationParam::new(token, f64::from(count)).with_message(message);
                    let _ = peer.notify_progress(note).await;
                }
            }
        };
        let run = run_command(command, notify);
        let (code, stdout, stderr) = match tokio::time::timeout(CALL_TIMEOUT, run).await {
            Ok(result) => result.map_err(|error| {
                ErrorData::internal_error(format!("Cannot run {}: {error}", self.exe.display()), None)
            })?,
            Err(_) => {
                let minutes = CALL_TIMEOUT.as_secs() / 60;
                let error = json!({ "error": { "code": "INTERNAL", "message": format!("{} did not finish within {minutes} minutes and was stopped", spec.tool.name) } });
                let mut result = CallToolResult::error(vec![ContentBlock::text(gai_core::to_js_json(&error))]);
                result.structured_content = Some(error);
                return Ok(result.into());
            }
        };
        // Exit 0 is success and exit 1 a gate that failed with its report on stdout;
        // anything else is an error line on stderr.
        let result = match (code, serde_json::from_str::<Value>(&stdout)) {
            (Some(0 | 1), Ok(value)) => {
                let mut content = vec![ContentBlock::text(stdout.trim_end())];
                let warnings = warnings(&stderr);
                if !warnings.is_empty() {
                    content.push(ContentBlock::text(format!("Warnings:\n- {}", warnings.join("\n- "))));
                }
                if spec.images {
                    content.extend(images(&cwd, &value));
                }
                let mut result = CallToolResult::success(content);
                result.structured_content = Some(structured(value));
                result
            }
            _ => {
                let error = stderr
                    .lines()
                    .rev()
                    .find_map(|line| serde_json::from_str::<Value>(line).ok().filter(|v| v.get("error").is_some()))
                    .unwrap_or_else(|| json!({ "error": { "code": "INTERNAL", "message": format!("exit {code:?}: {}", stderr.trim()) } }));
                let mut result = CallToolResult::error(vec![ContentBlock::text(gai_core::to_js_json(&error))]);
                result.structured_content = Some(error);
                result
            }
        };
        Ok(result.into())
    }
}

/// Runs `command`, sending each `{"note": …}` line it prints on stderr through `notify`
/// (numbered from 1) as it arrives. Returns the exit code, stdout and stderr.
async fn run_command<F, N>(
    mut command: tokio::process::Command,
    notify: N,
) -> std::io::Result<(Option<i32>, String, String)>
where
    N: Fn(u32, String) -> F,
    F: std::future::Future<Output = ()>,
{
    use tokio::io::{AsyncBufReadExt, AsyncReadExt};
    command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let mut child = command.spawn()?;
    let mut stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("no stdout"))?;
    let reader = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).await.map(|_| bytes)
    });
    let mut lines =
        tokio::io::BufReader::new(child.stderr.take().ok_or_else(|| std::io::Error::other("no stderr"))?).lines();
    let (mut stderr, mut count) = (String::new(), 0);
    while let Some(line) = lines.next_line().await? {
        if let Some(note) =
            serde_json::from_str::<Value>(&line).ok().and_then(|v| v.get("note")?.as_str().map(str::to_string))
        {
            count += 1;
            notify(count, note).await;
        }
        stderr.push_str(&line);
        stderr.push('\n');
    }
    let status = child.wait().await?;
    let stdout = reader.await.map_err(std::io::Error::other)??;
    Ok((status.code(), String::from_utf8_lossy(&stdout).into_owned(), stderr))
}

/// The folder a call runs in: the server's folder, or `cwd` relative to it. Absolute
/// paths and folders outside the server's (after resolving `..` and symlinks) are refused.
fn call_folder(server: &Path, cwd: Option<&str>) -> std::result::Result<PathBuf, String> {
    let Some(cwd) = cwd else { return Ok(server.to_path_buf()) };
    let relative = Path::new(cwd);
    if relative.is_absolute() || relative.has_root() || cwd.contains(':') {
        return Err(format!("cwd must be a folder inside the server's folder, relative to it: {cwd}"));
    }
    let base = server.canonicalize().map_err(|error| format!("Cannot open the server's folder: {error}"))?;
    let folder = base.join(relative).canonicalize().map_err(|error| format!("Cannot open {cwd}: {error}"))?;
    if !folder.starts_with(&base) {
        return Err(format!("cwd must stay inside the server's folder: {cwd}"));
    }
    Ok(folder)
}

/// The `{"warning": …}` lines a command printed on stderr.
fn warnings(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|line| line.get("warning").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// `structuredContent` must be an object: arrays are wrapped as `{results}`.
fn structured(value: Value) -> Value {
    match value {
        Value::Object(_) => value,
        other => json!({ "results": other }),
    }
}

/// The PNG sheets a `preview` result lists, as image content.
fn images(cwd: &Path, value: &Value) -> Vec<ContentBlock> {
    let files = value.get("files").and_then(Value::as_array).into_iter().flatten();
    files
        .filter_map(|file| file.get("path").and_then(Value::as_str))
        .filter_map(|path| std::fs::read(cwd.join(path)).ok())
        .take(MAX_IMAGES)
        .map(|png| ContentBlock::image(base64::engine::general_purpose::STANDARD.encode(png), "image/png"))
        .collect()
}

fn tool(
    name: &'static str,
    title: &str,
    description: &'static str,
    properties: Value,
    required: &[&str],
    annotations: ToolAnnotations,
) -> Tool {
    let mut properties = properties;
    if let Value::Object(map) = &mut properties {
        map.insert(
            "cwd".into(),
            json!({ "type": "string", "description": "Project folder inside the server's folder, relative to it. Default: the server's folder." }),
        );
    }
    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), properties);
    schema.insert("required".into(), json!(required));
    schema.insert("additionalProperties".into(), json!(false));
    Tool::new(name, description, schema).with_title(title).annotate(annotations)
}

fn annotations(read_only: bool, destructive: bool, idempotent: bool, open_world: bool) -> ToolAnnotations {
    ToolAnnotations::new().read_only(read_only).destructive(destructive).idempotent(idempotent).open_world(open_world)
}

/// Every tool, in the order an agent uses them.
fn tools() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            tool: tool(
                "search",
                "Search icons",
                "Find existing animated icons by meaning before making one. Ranks the released library (and this project's own built icons) by the words of `query`, with synonyms such as bell/notification or loader/spinner. Returns [{id, label, category, stateful, origin, score, matchedTerms}], best first. Use a returned id with `add`. An empty result means nothing close exists; consider `context` and `create`.",
                json!({
                    "query": { "type": "string", "description": "What the icon should mean, e.g. \"payment received\"" },
                    "limit": { "type": "integer", "minimum": 1, "description": "Most results (default 20)" },
                    "localOnly": { "type": "boolean", "description": "Only this project's own icons" },
                    "libraryOnly": { "type": "boolean", "description": "Only the released library" }
                }),
                &["query"],
                annotations(true, false, true, true),
            ),
            argv: |args| {
                let query = string(args, "query")?.ok_or("query is required")?;
                let mut options = Vec::new();
                push_value(&mut options, "--limit", integer(args, "limit")?);
                push_flag(&mut options, "--local-only", flag(args, "localOnly"));
                push_flag(&mut options, "--library-only", flag(args, "libraryOnly"));
                Ok(command_line("search", options, vec![query]))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "add",
                "Add icons to the project",
                "Install icons into the project: the .lottie asset plus a typed descriptor (web: an ES module in the descriptors folder; iOS/Android: JSON). Accepts library ids from `search` and local icons already built with `build`. Never overwrites files the user edited (fails with MODIFIED_FILE instead). Returns {changed, icons, version, platform}. Needs a project set up with `gen-animated-icons init`.",
                json!({
                    "ids": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Icon ids" },
                    "bakeWeight": { "type": "number", "minimum": 1, "maximum": 3, "description": "iOS/Android only: bake this outline weight into the files" },
                    "dryRun": { "type": "boolean", "description": "Report what would change without writing" }
                }),
                &["ids"],
                annotations(false, false, true, true),
            ),
            argv: |args| {
                let mut options = Vec::new();
                push_value(&mut options, "--bake-weight", number(args, "bakeWeight")?);
                push_flag(&mut options, "--dry-run", flag(args, "dryRun"));
                Ok(command_line("add", options, strings(args, "ids")?))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "context",
                "Get the icon design context",
                "Read this before drawing a new icon. Returns the reference packet: the family contract (32-unit grid, outline weights, timing limits), the library icons nearest to `query` (candidates), and for up to four references their Light/Strong timing profiles, the Strong motion's intent and brief, and their initial outline geometry. Pass `neighbors` to choose references yourself. Set `search` for the ranked candidate list only.",
                json!({
                    "query": { "type": "string", "description": "The icon's meaning and motion, e.g. \"card payment approved with a check\"" },
                    "neighbors": { "type": "array", "items": { "type": "string" }, "maxItems": 4, "description": "Library or built local icon ids to use as references" },
                    "search": { "type": "boolean", "description": "Return only the ranked candidates" },
                    "limit": { "type": "integer", "minimum": 1, "description": "Candidates with search (default 6)" }
                }),
                &["query"],
                annotations(true, false, true, true),
            ),
            argv: |args| {
                let query = string(args, "query")?.ok_or("query is required")?;
                let mut options = Vec::new();
                for neighbor in strings(args, "neighbors")? {
                    push_value(&mut options, "--neighbor", Some(neighbor));
                }
                push_flag(&mut options, "--search", flag(args, "search"));
                push_value(&mut options, "--limit", integer(args, "limit")?);
                Ok(command_line("context", options, vec![query]))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "create",
                "Create a local icon",
                "Scaffold a new icon in the project's own icon set (gen-animated-icons/icons): a batch file to edit, definitions.mjs updated to import it, and a saved context packet. It draws nothing. With `from`, the batch starts from that library icon's source (a working drawing and motion to adapt; needs a library checkout); otherwise it is a commented template that fails `check` until you draw parts. Returns {id, files, context, references, nearest, next}: `references` are the icons the saved packet embeds, and `nearest` lists the closest library icons only when you chose no references (no `from` or `neighbors`). Edit the batch path in `next`, then call `check`.",
                json!({
                    "id": { "type": "string", "pattern": ICON_ID_PATTERN, "description": "New kebab-case id; must not be a library id or a Windows device name (con, prn, aux, nul, com1 to com9, lpt1 to lpt9)" },
                    "from": { "type": "string", "description": "Library icon to start from" },
                    "label": { "type": "string", "description": "Display label (default: from the id)" },
                    "intent": { "type": "string", "description": "What the motion communicates" },
                    "neighbors": { "type": "array", "items": { "type": "string" }, "maxItems": 4, "description": "Reference icons for the saved context packet, as chosen with `context` (default: `from`, else the nearest by label and intent)" }
                }),
                &["id"],
                annotations(false, false, false, true),
            ),
            argv: |args| {
                let id = string(args, "id")?.ok_or("id is required")?;
                let mut options = Vec::new();
                push_value(&mut options, "--from", string(args, "from")?);
                push_value(&mut options, "--label", string(args, "label")?);
                push_value(&mut options, "--intent", string(args, "intent")?);
                for neighbor in strings(args, "neighbors")? {
                    push_value(&mut options, "--neighbor", Some(neighbor));
                }
                Ok(command_line("create", options, vec![id]))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "check",
                "Check the project and local icons",
                "Offline gates. `source` runs the source preflight on local icons (paths valid at every frame and weight, stable topology, timing, paint styles, no clash with library ids); `native` verifies built icons; `descriptors` validates installed descriptors; `project` checks the config and managed files. With no selector every check runs. Returns {status: PASS|STALE|FAIL, stale?, checks:[{id, status, diagnostics}]}; fix each error diagnostic, then check again. STALE means nothing failed but the builds listed in `stale` are older than their source: build them again before you preview or rely on them. PASS is not a visual review: build and preview to look at the icon.",
                json!({
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "Limit to these local icons (default: every one)" },
                    "source": { "type": "boolean", "description": "Run the source preflight" },
                    "native": { "type": "boolean", "description": "Verify the builds (a requested icon that is not built yet fails)" },
                    "descriptors": { "type": "boolean", "description": "Validate the installed descriptors" },
                    "project": { "type": "boolean", "description": "Check the config and managed files" }
                }),
                &[],
                annotations(false, false, true, false),
            ),
            argv: |args| {
                let mut options = Vec::new();
                for (key, option) in [
                    ("source", "--source"),
                    ("native", "--native"),
                    ("descriptors", "--descriptors"),
                    ("project", "--project"),
                ] {
                    push_flag(&mut options, option, flag(args, key));
                }
                Ok(command_line("check", options, strings(args, "ids")?))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "build",
                "Build local icons",
                "Export local icons through the hosted engine and install them: preflight, one export per action (Light, Strong and reverse), packaging into .lottie, verification, preview sheets, then install like `add`. Needs a signed-in engine session (the user runs `gen-animated-icons login`; NOT_LOGGED_IN means ask them to) and the network: a server started with --offline refuses it (NETWORK). Takes about 15 s per icon and more under load, so build one icon per call; pass a progress token to get a notification per exported action. Returns {status, icons:[{id, actions, asset, descriptor, checks, preview}], changed}; then call `preview` or look at the listed sheets.",
                json!({
                    "ids": { "type": "array", "items": { "type": "string" }, "description": "Icons to build (default: every local icon)" },
                    "jobs": { "type": "integer", "minimum": 1, "maximum": 4, "description": "Parallel engine exports (default 2)" },
                    "noInstall": { "type": "boolean", "description": "Build and verify without installing" }
                }),
                &[],
                annotations(false, false, true, true),
            ),
            argv: |args| {
                let mut options = Vec::new();
                push_value(&mut options, "--jobs", integer(args, "jobs")?);
                push_flag(&mut options, "--no-install", flag(args, "noInstall"));
                Ok(command_line("build", options, strings(args, "ids")?))
            },
            images: false,
        },
        ToolSpec {
            tool: tool(
                "preview",
                "Preview an icon",
                "Render an icon's motions with the players' own renderer and return the contact sheets as images: one PNG per motion (Light, Strong), rows are looks (outline weights, solid, duotone, dark and light themes), columns are the same elapsed times for both motions. Look at them to judge the drawing and the motion before and after each change. Works for built local icons, installed icons and library icons. Returns {id, source, renderer, files:[{path, …}]} plus the images.",
                json!({
                    "id": { "type": "string", "description": "Icon id" },
                    "motion": { "type": "string", "enum": ["light", "strong", "both", "reverse", "all"], "description": "Default both; reverse renders a stateful icon's reverse (reset) actions" },
                    "animations": { "type": "array", "items": { "type": "string" }, "description": "Render these actions by name instead of `motion`, e.g. the one a verify failure names" },
                    "times": { "type": "array", "items": { "type": "number", "minimum": 0 }, "maxItems": MAX_COLUMNS, "description": "Column times in seconds" },
                    "frames": { "type": "integer", "minimum": 1, "maximum": MAX_COLUMNS, "description": "Evenly spaced columns from the start to the rest frame (reverse actions get their own)" }
                }),
                &["id"],
                annotations(false, false, true, true),
            ),
            argv: |args| {
                let id = string(args, "id")?.ok_or("id is required")?;
                let mut options = Vec::new();
                push_value(&mut options, "--motion", string(args, "motion")?);
                let animations = strings(args, "animations")?;
                if animations.iter().any(|name| name.contains(',')) {
                    return Err("animations must be action names".into());
                }
                push_value(&mut options, "--animation", (!animations.is_empty()).then(|| animations.join(",")));
                let times = numbers(args, "times")?;
                push_value(&mut options, "--times", (!times.is_empty()).then(|| times.join(",")));
                push_value(&mut options, "--frames", integer(args, "frames")?);
                Ok(command_line("preview", options, vec![id]))
            },
            images: true,
        },
    ]
}

type Parsed<T> = std::result::Result<T, String>;

/// Checks a call's arguments against the tool's input schema, as `deny_unknown_fields`
/// would: every name must be one of its `properties`, every `required` one present, and
/// each value of its property's `type`, within its `enum`, bounds, item counts and
/// `pattern`. A `null` value counts as left out. The error names what the tool takes.
fn validate(tool: &Tool, args: &Map<String, Value>) -> Parsed<()> {
    let name = &tool.name;
    let schema = &tool.input_schema;
    let properties = schema.get("properties").and_then(Value::as_object).cloned().unwrap_or_default();
    let names: Vec<&str> = properties.keys().map(String::as_str).collect();
    let takes = || {
        let list: Vec<String> = names.iter().map(|key| format!("`{key}`")).collect();
        format!("{name} takes {}", list.join(", "))
    };
    for key in args.keys() {
        if !properties.contains_key(key) {
            let guess = similar(key, &names).map(|known| format!("; did you mean `{known}`?")).unwrap_or_default();
            return Err(format!("{name} has no argument `{key}`{guess} ({})", takes()));
        }
    }
    let required = schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str);
    for key in required {
        if args.get(key).is_none_or(Value::is_null) {
            return Err(format!("{name} needs `{key}` ({})", takes()));
        }
    }
    for (key, value) in args.iter().filter(|(_, value)| !value.is_null()) {
        check_value(key, value, &properties[key])?;
    }
    Ok(())
}

/// One value against its property schema; `key` names it in errors.
fn check_value(key: &str, value: &Value, schema: &Value) -> Parsed<()> {
    let kind = schema.get("type").and_then(Value::as_str).unwrap_or_default();
    let bound = |name: &str| schema.get(name).and_then(Value::as_f64);
    let in_bounds = |number: f64| -> Parsed<()> {
        let show = gai_core::js_number_to_string;
        match (bound("minimum"), bound("maximum")) {
            (Some(least), _) if number < least => Err(format!("`{key}` must be at least {}", show(least))),
            (_, Some(most)) if number > most => Err(format!("`{key}` must be at most {}", show(most))),
            _ => Ok(()),
        }
    };
    match kind {
        "string" => {
            let text = value.as_str().ok_or(format!("`{key}` must be a string"))?;
            if let Some(options) = schema.get("enum").and_then(Value::as_array)
                && !options.iter().any(|option| option.as_str() == Some(text))
            {
                let options: Vec<&str> = options.iter().filter_map(Value::as_str).collect();
                return Err(format!("`{key}` must be one of {}", options.join(", ")));
            }
            match schema.get("pattern").and_then(Value::as_str) {
                None => Ok(()),
                Some(ICON_ID_PATTERN) if crate::local_set::is_icon_name(text) => Ok(()),
                Some(ICON_ID_PATTERN) => {
                    Err(format!("`{key}` must be a kebab-case id such as payment-sent, not {text}"))
                }
                Some(other) => Err(format!("`{key}` has a pattern this server cannot check: {other}")),
            }
        }
        "boolean" => value.as_bool().map(drop).ok_or(format!("`{key}` must be a boolean")),
        "integer" => {
            let number = value.as_i64().ok_or(format!("`{key}` must be a whole number"))?;
            in_bounds(number as f64)
        }
        "number" => in_bounds(value.as_f64().ok_or(format!("`{key}` must be a number"))?),
        "array" => {
            let items = value.as_array().ok_or_else(|| {
                let of = schema.pointer("/items/type").and_then(Value::as_str).unwrap_or("value");
                format!("`{key}` must be an array of {of}s")
            })?;
            let count = |name: &str| schema.get(name).and_then(Value::as_u64).map(|n| n as usize);
            if let Some(least) = count("minItems").filter(|least| items.len() < *least) {
                return Err(format!("`{key}` needs at least {least} item{}", if least == 1 { "" } else { "s" }));
            }
            if let Some(most) = count("maxItems").filter(|most| items.len() > *most) {
                return Err(format!("`{key}` takes at most {most} items"));
            }
            let item = schema.get("items").cloned().unwrap_or(Value::Null);
            items
                .iter()
                .enumerate()
                .try_for_each(|(index, value)| check_value(&format!("{key}[{index}]"), value, &item))
        }
        other => Err(format!("`{key}` has a type this server cannot check: {other}")),
    }
}

/// The `pattern` of an icon id.
const ICON_ID_PATTERN: &str = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

/// The argument `key` was probably meant to be: the same name in another case or
/// spelling (`local_only` for `localOnly`), its singular or plural (`id` for `ids`), or
/// one at most two edits away.
fn similar<'a>(key: &str, names: &[&'a str]) -> Option<&'a str> {
    let plain = |name: &str| name.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    let key = plain(key);
    let close = |name: &str| {
        let name = plain(name);
        let distance = edits(&key, &name);
        let plural = name.strip_suffix('s') == Some(key.as_str()) || key.strip_suffix('s') == Some(name.as_str());
        match name == key || plural {
            true => Some(0),
            false => (distance <= 2 && name.len() > distance + 2).then_some(distance),
        }
    };
    names.iter().filter_map(|name| close(name).map(|rank| (rank, *name))).min().map(|(_, name)| name)
}

/// Levenshtein distance between short ASCII-ish names.
fn edits(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(x != *y)).min(row[j] + 1).min(above + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}

fn string(args: &Map<String, Value>, key: &str) -> Parsed<Option<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

fn strings(args: &Map<String, Value>, key: &str) -> Parsed<Vec<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_str().map(str::to_string).ok_or(format!("{key} must list strings")))
            .collect(),
        Some(_) => Err(format!("{key} must be a list of strings")),
    }
}

fn integer(args: &Map<String, Value>, key: &str) -> Parsed<Option<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(|n| Some(n.to_string())).ok_or(format!("{key} must be a positive integer")),
    }
}

fn number(args: &Map<String, Value>, key: &str) -> Parsed<Option<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            value.as_f64().map(|n| Some(gai_core::js_number_to_string(n))).ok_or(format!("{key} must be a number"))
        }
    }
}

fn numbers(args: &Map<String, Value>, key: &str) -> Parsed<Vec<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_f64().map(gai_core::js_number_to_string).ok_or(format!("{key} must list numbers")))
            .collect(),
        Some(_) => Err(format!("{key} must be a list of numbers")),
    }
}

fn flag(args: &Map<String, Value>, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn push_flag(argv: &mut Vec<String>, option: &str, on: bool) {
    if on {
        argv.push(option.to_string());
    }
}

fn push_value(argv: &mut Vec<String>, option: &str, value: Option<String>) {
    if let Some(value) = value {
        argv.push(format!("{option}={value}"));
    }
}

/// `<command> <options…> -- <positionals…>`.
fn command_line(command: &str, options: Vec<String>, positionals: Vec<String>) -> Vec<String> {
    let mut argv = vec![command.to_string()];
    argv.extend(options);
    if !positionals.is_empty() {
        argv.push("--".into());
        argv.extend(positionals);
    }
    argv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(name: &str, args: Value) -> Parsed<Vec<String>> {
        let spec = tools().into_iter().find(|spec| spec.tool.name == name).unwrap();
        (spec.argv)(args.as_object().unwrap())
    }

    #[test]
    fn tool_arguments_become_command_lines() {
        assert_eq!(
            argv("search", json!({"query": "bell ring", "limit": 3})).unwrap(),
            ["search", "--limit=3", "--", "bell ring"]
        );
        assert_eq!(
            argv("preview", json!({"id": "heart", "motion": "strong", "times": [0, 0.25]})).unwrap(),
            ["preview", "--motion=strong", "--times=0,0.25", "--", "heart"]
        );
        assert_eq!(
            argv("context", json!({"query": "pay", "neighbors": ["card", "coin"]})).unwrap(),
            ["context", "--neighbor=card", "--neighbor=coin", "--", "pay"]
        );
        assert_eq!(argv("check", json!({"ids": ["a"], "source": true})).unwrap(), ["check", "--source", "--", "a"]);
        assert_eq!(argv("build", json!({})).unwrap(), ["build"]);
        // Values that look like flags stay values.
        assert_eq!(argv("add", json!({"ids": ["--installed"]})).unwrap(), ["add", "--", "--installed"]);
        assert_eq!(argv("create", json!({"id": "x", "from": "-y"})).unwrap(), ["create", "--from=-y", "--", "x"]);
        assert_eq!(
            argv("create", json!({"id": "x", "neighbors": ["a", "b"]})).unwrap(),
            ["create", "--neighbor=a", "--neighbor=b", "--", "x"]
        );
        assert_eq!(
            argv("preview", json!({"id": "x", "animations": ["x-reset", "x-reset-strong"]})).unwrap(),
            ["preview", "--animation=x-reset,x-reset-strong", "--", "x"]
        );
        assert!(argv("search", json!({})).is_err());
        assert_eq!(warnings("{\"warning\":\"a\"}\nplain\n{\"note\":\"n\"}\n{\"warning\":\"b\"}\n"), ["a", "b"]);
        assert!(argv("add", json!({"ids": [1]})).is_err());
    }

    #[test]
    fn calls_stay_inside_the_server_folder() {
        let dir = tempfile::tempdir().unwrap();
        let server = dir.path().join("server");
        std::fs::create_dir_all(server.join("apps/web")).unwrap();
        std::fs::create_dir_all(dir.path().join("outside")).unwrap();
        let base = server.canonicalize().unwrap();
        assert_eq!(call_folder(&server, None).unwrap(), server);
        assert_eq!(call_folder(&server, Some("apps/web")).unwrap(), base.join("apps/web"));
        assert_eq!(call_folder(&server, Some("apps/../apps/web")).unwrap(), base.join("apps/web"));
        let outside = dir.path().join("outside").display().to_string();
        for bad in [outside.as_str(), "../outside", "apps/../../outside", "C:x"] {
            assert!(call_folder(&server, Some(bad)).is_err(), "{bad}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("outside"), server.join("link")).unwrap();
            assert!(call_folder(&server, Some("link")).is_err());
        }
    }

    #[test]
    fn every_tool_documents_its_arguments() {
        for spec in tools() {
            let schema = &spec.tool.input_schema;
            assert_eq!(schema["type"], "object", "{}", spec.tool.name);
            assert!(schema["properties"]["cwd"].is_object());
            assert!(spec.tool.description.as_ref().is_some_and(|d| d.len() > 100));
            for (key, property) in schema["properties"].as_object().unwrap() {
                assert!(property["description"].is_string(), "{} {key}", spec.tool.name);
            }
        }
    }

    fn validated(name: &str, args: Value) -> Parsed<()> {
        let spec = tools().into_iter().find(|spec| spec.tool.name == name).unwrap();
        validate(&spec.tool, args.as_object().unwrap())
    }

    #[test]
    fn arguments_must_match_the_schema() {
        assert_eq!(
            validated("build", json!({"id": "a"})).unwrap_err(),
            "build has no argument `id`; did you mean `ids`? (build takes `ids`, `jobs`, `noInstall`, `cwd`)"
        );
        assert!(validated("preview", json!({"ids": ["a"]})).unwrap_err().contains("did you mean `id`?"));
        assert!(
            validated("search", json!({"query": "a", "library-only": true})).unwrap_err().contains("`libraryOnly`")
        );
        assert!(validated("search", json!({"querry": "a"})).unwrap_err().contains("did you mean `query`?"));
        let unrelated = validated("check", json!({"verbose": true})).unwrap_err();
        assert!(unrelated.starts_with("check has no argument `verbose` (check takes"), "{unrelated}");
        assert_eq!(validated("search", json!({})).unwrap_err().split(" (").next(), Some("search needs `query`"));
        assert!(validated("search", json!({"query": null})).is_err());
        for (name, args, error) in [
            ("check", json!({"native": 1}), "`native` must be a boolean"),
            ("search", json!({"query": "a", "limit": 0}), "`limit` must be at least 1"),
            ("search", json!({"query": "a", "limit": 2.5}), "`limit` must be a whole number"),
            ("build", json!({"jobs": 5}), "`jobs` must be at most 4"),
            ("add", json!({"ids": []}), "`ids` needs at least 1 item"),
            ("add", json!({"ids": ["a"], "bakeWeight": 3.5}), "`bakeWeight` must be at most 3"),
            ("add", json!({"ids": "a"}), "`ids` must be an array of strings"),
            ("check", json!({"ids": ["a", 2]}), "`ids[1]` must be a string"),
            ("preview", json!({"id": "a", "times": [0, -1]}), "`times[1]` must be at least 0"),
            ("preview", json!({"id": "a", "frames": 65}), "`frames` must be at most 64"),
            ("preview", json!({"id": "a", "times": vec![0; 65]}), "`times` takes at most 64 items"),
            (
                "preview",
                json!({"id": "a", "motion": "fast"}),
                "`motion` must be one of light, strong, both, reverse, all",
            ),
            ("create", json!({"id": "A_b"}), "`id` must be a kebab-case id such as payment-sent, not A_b"),
            ("create", json!({"id": "a", "neighbors": ["a", "b", "c", "d", "e"]}), "`neighbors` takes at most 4 items"),
        ] {
            assert_eq!(validated(name, args.clone()).unwrap_err(), error, "{name} {args}");
        }
        // Left-out optional values may be null.
        assert_eq!(validated("search", json!({"query": "a", "limit": null, "cwd": null})), Ok(()));
        assert_eq!(validated("preview", json!({"id": "a", "motion": "all", "times": [0, 0.5], "cwd": "app"})), Ok(()));
    }

    /// Runs `future` on a runtime like the server's.
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
    }

    #[test]
    fn lines_that_are_not_messages_are_answered_with_their_id() {
        use rmcp::transport::Transport;
        block_on(async {
            let (mut client, server_in) = tokio::io::duplex(1 << 16);
            let (server_out, client_out) = tokio::io::duplex(1 << 16);
            let mut transport = LineTransport::new(server_in, server_out, 96);
            let long = format!("{{\"id\": 9, \"params\": \"{}\"}}", "x".repeat(200));
            let lines = [
                "not json",
                "",
                "\r",
                "{\"jsonrpc\": \"2.0\", \"id\": 7, \"method\": \"ping\", \"params\": {",
                // A number at the fault may have been cut short.
                "{\"jsonrpc\": \"2.0\", \"id\": 70",
                "{\"jsonrpc\": \"2.0\", \"id\": \"s\", \"method\": \"tools/call\", \"params\": 3}",
                "{\"jsonrpc\": \"2.0\", \"method\": \"notifications/progress\", \"params\": 3}",
                "{\"jsonrpc\": \"2.0\", \"id\": 5, \"error\": 3}",
                &long,
                "[1, 2]",
                "\u{feff}{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"ping\"}\r",
            ];
            client.write_all((lines.join("\n") + "\n").as_bytes()).await.unwrap();
            let message = serde_json::to_value(transport.receive().await.unwrap()).unwrap();
            assert_eq!((message["id"].as_u64(), message["method"].as_str()), (Some(1), Some("ping")), "{message}");
            let mut answers = tokio::io::BufReader::new(client_out).lines();
            let mut got = Vec::new();
            for _ in 0..6 {
                let answer: Value = serde_json::from_str(&answers.next_line().await.unwrap().unwrap()).unwrap();
                assert_eq!(answer["jsonrpc"], "2.0");
                got.push((answer["id"].clone(), answer["error"]["code"].as_i64().unwrap()));
            }
            let expected = [
                (json!(null), PARSE_ERROR),
                (json!(7), PARSE_ERROR),
                (json!(null), PARSE_ERROR),
                (json!("s"), INVALID_REQUEST),
                (json!(9), PARSE_ERROR),
                (json!(null), INVALID_REQUEST),
            ];
            assert_eq!(got, expected);
            // Closing the input ends the session.
            drop(client);
            assert!(transport.receive().await.is_none());
        });
    }

    #[test]
    fn a_cancelled_receive_keeps_the_line_read_so_far() {
        use rmcp::transport::Transport;
        block_on(async {
            let (mut client, server_in) = tokio::io::duplex(1024);
            let (server_out, _client_out) = tokio::io::duplex(1024);
            let mut transport = LineTransport::new(server_in, server_out, MAX_LINE);
            client.write_all(b"{\"jsonrpc\": \"2.0\", ").await.unwrap();
            let waited = tokio::time::timeout(std::time::Duration::from_millis(50), transport.receive()).await;
            assert!(waited.is_err());
            client.write_all(b"\"id\": 1, \"method\": \"ping\"}\n").await.unwrap();
            let message = serde_json::to_value(transport.receive().await.unwrap()).unwrap();
            assert_eq!(message["method"], "ping", "{message}");
        });
    }

    /// The schema keywords [`validate`] enforces (`description` is documentation).
    const KEYWORDS: [&str; 9] =
        ["type", "description", "enum", "minimum", "maximum", "minItems", "maxItems", "items", "pattern"];

    /// A valid sample value of a property schema.
    fn sample(schema: &Value) -> Value {
        if let Some(options) = schema["enum"].as_array() {
            return options[0].clone();
        }
        match schema["type"].as_str().unwrap() {
            "string" => json!("x"),
            "boolean" => json!(true),
            "integer" | "number" => schema["minimum"].as_f64().map_or(json!(1), |least| json!(least.max(1.0) as u64)),
            "array" => json!([sample(&schema["items"])]),
            other => panic!("no sample for {other}"),
        }
    }

    #[test]
    fn every_schema_keyword_is_checked_and_every_argument_reaches_the_command() {
        for spec in tools() {
            let schema = &spec.tool.input_schema;
            let properties = schema["properties"].as_object().unwrap();
            let mut walk = vec![Value::Object(properties.clone())];
            while let Some(Value::Object(map)) = walk.pop() {
                for (key, property) in map {
                    for keyword in property.as_object().unwrap().keys() {
                        assert!(KEYWORDS.contains(&keyword.as_str()), "{} {key}: {keyword}", spec.tool.name);
                    }
                    if let Some(items) = property.get("items") {
                        walk.push(json!({ format!("{key}[]"): items }));
                    }
                }
            }
            let required: Map<String, Value> = schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .map(|key| (key.as_str().unwrap().to_string(), sample(&properties[key.as_str().unwrap()])))
                .collect();
            let base = (spec.argv)(&required).unwrap();
            for (key, property) in properties.iter().filter(|(key, _)| *key != "cwd" && !required.contains_key(*key)) {
                let mut args = required.clone();
                args.insert(key.clone(), sample(property));
                assert_eq!(validate(&spec.tool, &args), Ok(()), "{} {key}", spec.tool.name);
                assert_ne!((spec.argv)(&args).unwrap(), base, "{} ignores {key}", spec.tool.name);
            }
        }
    }
}
