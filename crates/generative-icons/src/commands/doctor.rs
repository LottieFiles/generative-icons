//! `doctor`: checks the binary and CPU, the project and its config, the installed
//! skills and kit, the lock, the icon library, the JS engine, the renderer, the sign-in
//! and the hosted engine. Exits 1 when any check fails (an ended session included);
//! skipped checks, such as a session not verified offline, do not count.

use crate::Context;
use crate::config::{CONFIG_FILE, Config};
use crate::embedded::{SKILL_FILES, SKILL_NAME};
use crate::engine;
use crate::error::{ErrorCode, Result};
use crate::kit;
use crate::lock::Lock;
use crate::managed::{inside, read_optional};
use crate::output::Report;
use crate::project::detect;
use gai_core::sha256_hex;
use gai_engine::SessionState;
use serde_json::{Map, Value, json};
use std::fmt::Write;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Pass,
    Fail,
    Skip,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Skip => "skip",
        }
    }
}

struct Check {
    id: &'static str,
    status: Status,
    message: String,
    hint: Option<String>,
}

impl Check {
    fn new(id: &'static str, status: Status, message: impl Into<String>) -> Self {
        Self { id, status, message: message.into(), hint: None }
    }

    fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    fn to_json(&self) -> Value {
        let mut check = Map::new();
        check.insert("id".into(), json!(self.id));
        check.insert("status".into(), json!(self.status.as_str()));
        check.insert("message".into(), json!(self.message));
        if let Some(hint) = &self.hint {
            check.insert("hint".into(), json!(hint));
        }
        Value::Object(check)
    }
}

/// Runs `doctor`.
pub fn run(ctx: &Context) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let binary = Check::new(
        "binary",
        Status::Pass,
        format!(
            "generative-icons {} ({} {})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
    );
    let not_set_up = |id| Check::new(id, Status::Skip, "The project is not set up").hint("Run generative-icons init");
    let mut checks = vec![binary, cpu_check()];
    match Config::load(root, ctx.platform) {
        Err(error) => {
            let check = Check::new("project", Status::Fail, error.message);
            checks.push(match error.hint {
                Some(hint) => check.hint(hint),
                None => check,
            });
            checks.extend(["skills", "kit", "lock"].map(not_set_up));
        }
        Ok(None) => {
            checks.push(match detect(root, ctx.platform, None) {
                Ok(found) => Check::new(
                    "project",
                    Status::Fail,
                    format!("A {} app without {CONFIG_FILE}", found.platform.as_str()),
                )
                .hint("Run generative-icons init"),
                Err(error) => Check::new("project", Status::Fail, error.message)
                    .hint(error.hint.unwrap_or_else(|| "Run this in an app folder, or pass --cwd <app-folder>".into())),
            });
            checks.extend(["skills", "kit", "lock"].map(not_set_up));
        }
        Ok(Some(config)) => {
            let described = match detect(root, Some(config.platform), None) {
                Ok(d) => format!("{} ({}, {})", d.platform.as_str(), d.framework, d.package_manager),
                Err(_) => config.platform.as_str().to_string(),
            };
            checks.push(Check::new("project", Status::Pass, format!("{described}, configured by {CONFIG_FILE}")));
            checks.push(managed_check(root, &config, "skills"));
            checks.push(managed_check(root, &config, "kit"));
            checks.push(lock_check(root));
        }
    }

    checks.push(library_check(ctx));
    checks.push(quickjs_check());
    checks.push(renderer_check());
    checks.extend(engine_checks(ctx));

    let failed = checks.iter().any(|check| check.status == Status::Fail);
    let status = if failed { "FAIL" } else { "PASS" };
    let value = json!({ "status": status, "checks": checks.iter().map(Check::to_json).collect::<Vec<_>>() });
    let mut text = format!("{status}\n");
    for check in &checks {
        let _ = writeln!(text, "  {:<4}  {:<8}  {}", check.status.as_str(), check.id, check.message);
        if let Some(hint) = &check.hint {
            let _ = writeln!(text, "  {:<4}  {:<8}  hint: {hint}", "", "");
        }
    }
    Ok(Report { failed, ..Report::new(value, text) })
}

/// What to do when the library cannot be read.
const LIBRARY_HINT: &str = "Pass --library <dir> with a package dist folder or a library checkout to every command (or set GEN_ANIMATED_ICONS_LIBRARY); --offline uses the cached catalog";

/// The library `search`, `add` and `context` use (as they resolve it) and whether its
/// catalog loads: fetched online, the cached one offline.
fn library_check(ctx: &Context) -> Check {
    let failed = |error: crate::error::CliError| {
        let hint = error.hint.map_or(LIBRARY_HINT.to_string(), |hint| format!("{hint}; or {LIBRARY_HINT}"));
        Check::new("library", Status::Fail, error.message).hint(hint)
    };
    let source = match ctx.library() {
        Ok(source) => source,
        Err(error) => return failed(error),
    };
    let location = source.location();
    match source.catalog() {
        Ok(catalog) => {
            let cached = if ctx.offline && location.contains("://") { ", cached" } else { "" };
            let message = format!("{} ({} icons{cached}) from {location}", catalog.version, catalog.icons.len());
            Check::new("library", Status::Pass, message)
        }
        Err(error) if ctx.offline && error.code == ErrorCode::Network => {
            Check::new("library", Status::Skip, format!("Offline, and no catalog of {location} is cached"))
                .hint(LIBRARY_HINT)
        }
        Err(error) => failed(error),
    }
}

/// QuickJS's Math is V8's fdlibm built with FMA contraction; x86_64 needs FMA. The
/// built-in V8 vectors show whether this binary computes it that way.
fn cpu_check() -> Check {
    let arch = std::env::consts::ARCH;
    if let Err(error) = gai_js::check_cpu() {
        let error = crate::commands::check::cpu_error(error);
        return Check::new("cpu", Status::Fail, error.message).hint(error.hint.unwrap_or_default());
    }
    match crate::math::self_test() {
        Ok(count) => Check::new("cpu", Status::Pass, format!("{arch}: icon math matches V8 ({count} vectors)")),
        Err(mismatch) => Check::new("cpu", Status::Fail, format!("{arch}: icon math differs from V8: {mismatch}"))
            .hint("Reinstall generative-icons from the official release; this binary was built wrongly"),
    }
}

/// The JS engine loads the embedded kit the way a build evaluates it.
fn quickjs_check() -> Check {
    let load = || -> std::result::Result<(), String> {
        let dir = tempfile::tempdir().map_err(|error| error.to_string())?;
        let (icons, library) = (dir.path().join("icons"), dir.path().join("library"));
        for folder in [&icons, &library] {
            std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
        }
        for file in kit::files() {
            std::fs::write(library.join(file.name), file.text).map_err(|error| error.to_string())?;
        }
        let definitions = "import {make} from '../library/ai-authoring.mjs';\nexport const icons = [];\n";
        std::fs::write(icons.join("definitions.mjs"), definitions).map_err(|error| error.to_string())?;
        let limits = gai_js::Limits::default();
        gai_js::run_on_worker(&limits, || {
            gai_js::SetRuntime::load(&gai_js::SetSource::user_project(&icons), &limits).map(drop)
        })
        .map_err(|error| error.to_string())
    };
    match load() {
        Ok(()) => Check::new(
            "quickjs",
            Status::Pass,
            format!("{} evaluates the authoring kit ({})", gai_js::ENGINE, gai_js::math()),
        ),
        Err(error) => Check::new("quickjs", Status::Fail, format!("The authoring kit does not load: {error}")),
    }
}

/// The renderer reproduces the browser player's pixels.
fn renderer_check() -> Check {
    let slots = match gai_render::STROKE_WIDTH_SLOT_SUPPORTED {
        true => "in-between weights use the stroke-width slot",
        false => "in-between weights are rewritten into the file",
    };
    match gai_render::probe::renderer_matches_browser() {
        true => Check::new(
            "renderer",
            Status::Pass,
            format!("{} matches the browser player; {slots}", gai_render::RENDERER),
        ),
        false => Check::new(
            "renderer",
            Status::Fail,
            format!("{} was compiled with FMA contraction and differs from the browser player", gai_render::RENDERER),
        )
        .hint("Rebuild with CXXFLAGS=-ffp-contract=off"),
    }
}

/// Sign-in and the hosted engine. The session is checked with the engine (a refresh when
/// due, else `initialize`) unless `--offline`, when it is reported as not verified.
fn engine_checks(ctx: &Context) -> [Check; 2] {
    let config = match engine::config(ctx) {
        Ok(config) => config,
        Err(error) => {
            let failed = |id| Check::new(id, Status::Fail, error.message.clone());
            return [failed("auth"), failed("engine")];
        }
    };
    let url = config.url.to_string();
    let check = match gai_engine::check_session(&config, !ctx.offline) {
        Ok(check) => check,
        Err(error) => {
            return [
                Check::new("auth", Status::Fail, engine::error(error).message),
                Check::new("engine", Status::Skip, "No usable session"),
            ];
        }
    };
    let who = check.status.as_ref().and_then(|status| status.account.clone());
    let who = who.unwrap_or_else(|| "an unnamed account".into());
    let reason = check.reason.unwrap_or_default();
    let auth = match check.state {
        SessionState::Verified => Check::new("auth", Status::Pass, format!("Signed in to {url} as {who}")),
        SessionState::NotVerified => {
            Check::new("auth", Status::Skip, format!("Signed in to {url} as {who}; not verified ({reason})"))
        }
        SessionState::Ended => Check::new("auth", Status::Fail, format!("The session for {url} has ended ({reason})"))
            .hint(engine::LOGIN_HINT),
        SessionState::SignedOut => {
            Check::new("auth", Status::Skip, format!("Not signed in to {url}")).hint(engine::LOGIN_HINT)
        }
    };
    if ctx.offline {
        return [auth, Check::new("engine", Status::Skip, "Offline")];
    }
    let engine = match check.state {
        SessionState::Verified => match gai_engine::McpClient::new(&config).and_then(|c| c.server_info()) {
            Ok(info) => Check::new(
                "engine",
                Status::Pass,
                format!("{} {} answers (MCP {})", info.name, info.version, info.protocol_version),
            ),
            Err(error) => {
                let error = engine::error(error);
                Check::new("engine", Status::Fail, error.message)
                    .hint(error.hint.unwrap_or_else(|| "Retry later".into()))
            }
        },
        SessionState::NotVerified => Check::new("engine", Status::Fail, format!("Cannot check the session: {reason}"))
            .hint("Check the connection or --engine-url, then retry"),
        SessionState::Ended | SessionState::SignedOut => reachable(&url),
    };
    [auth, engine]
}

/// Without a session the engine can only say that it is there (401).
fn reachable(url: &str) -> Check {
    let status = gai_engine::Downloader::new(Duration::from_secs(10)).and_then(|client| client.status(url));
    match status {
        Ok(status) => {
            Check::new("engine", Status::Pass, format!("{url} is reachable (HTTP {status} without a session)"))
                .hint(engine::LOGIN_HINT)
        }
        Err(error) => Check::new("engine", Status::Fail, format!("Cannot reach {url}: {error}"))
            .hint("Check the connection or --engine-url"),
    }
}

/// Compares installed skill or kit files with the embedded ones and the lock.
fn managed_check(root: &std::path::Path, config: &Config, id: &'static str) -> Check {
    let expected: Vec<(String, &str)> = match id {
        "skills" => config
            .skills
            .iter()
            .flat_map(|skill_root| {
                SKILL_FILES.iter().map(move |(rel, content)| (format!("{skill_root}/{SKILL_NAME}/{rel}"), *content))
            })
            .collect(),
        _ => kit::files().iter().map(|file| (format!("{}/{}", config.kit_dir(), file.name), file.text)).collect(),
    };
    if expected.is_empty() {
        return Check::new(id, Status::Skip, "No skill folders configured");
    }
    let lock = match Lock::load(root) {
        Ok(lock) => lock,
        Err(error) => return Check::new(id, Status::Fail, error.message),
    };
    let recorded: std::collections::HashMap<&str, &str> = lock.files().collect();
    let (mut missing, mut outdated, mut edited) = (Vec::new(), Vec::new(), Vec::new());
    for (path, content) in &expected {
        let bytes = match inside(root, path).and_then(|file| read_optional(&file)) {
            Ok(bytes) => bytes,
            Err(error) => return Check::new(id, Status::Fail, error.message),
        };
        match bytes {
            None => missing.push(path.clone()),
            Some(bytes) if bytes == content.as_bytes() => {}
            Some(bytes) if recorded.get(path.as_str()) == Some(&sha256_hex(&bytes).as_str()) => {
                outdated.push(path.clone())
            }
            Some(_) => edited.push(path.clone()),
        }
    }
    let where_ = match id {
        "skills" => config.skills.join(", "),
        _ => config.kit_dir(),
    };
    if !edited.is_empty() {
        let hint = match id {
            "kit" => "Move your changes into your own files, then run generative-icons init --update-kit",
            _ => "Restore or delete the edited files, then run generative-icons init",
        };
        return Check::new(id, Status::Fail, format!("Edited: {}", edited.join(", "))).hint(hint);
    }
    if !missing.is_empty() || !outdated.is_empty() {
        let mut parts = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("missing: {}", missing.join(", ")));
        }
        if !outdated.is_empty() {
            parts.push(format!("from an older version: {}", outdated.join(", ")));
        }
        return Check::new(id, Status::Fail, parts.join("; ")).hint("Run generative-icons init");
    }
    Check::new(id, Status::Pass, format!("Current in {where_}"))
}

/// Lock integrity: parseable, with every managed file's state summarized.
fn lock_check(root: &std::path::Path) -> Check {
    let status = Lock::load(root).and_then(|lock| lock.status(root));
    match status {
        Err(error) => Check::new("lock", Status::Fail, error.message),
        Ok(files) => {
            let count = |state| files.iter().filter(|s| s.status == state).count();
            let (modified, missing) = (count("modified"), count("missing"));
            let check = Check::new(
                "lock",
                Status::Pass,
                format!(
                    "{} managed files: {} unchanged, {modified} modified, {missing} missing",
                    files.len(),
                    count("unchanged")
                ),
            );
            if modified + missing > 0 { check.hint("Run generative-icons diff to see which") } else { check }
        }
    }
}
