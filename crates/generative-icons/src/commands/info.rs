//! `info`: what this folder is and what the tool manages in it. Outside a project the
//! project fields are `null`; only an invalid config or lock is an error.
//!
//! `localIcons` lists every local icon: installed, built or only created. `local` says
//! for each where its spec (or v1 batch) is, whether its build is current, stale or
//! missing, and whether it is installed, so a new session finds work in progress.

use crate::Context;
use crate::config::{Config, Layout};
use crate::error::Result;
use crate::local_set::{LocalSet, build_dir, changed_source};
use crate::lock::Lock;
use crate::managed::inside;
use crate::output::Report;
use crate::project::detect;
use gai_engine::{EngineConfig, SessionState};
use serde_json::{Value, json};
use std::fmt::Write;

/// Runs `info`.
pub fn run(ctx: &Context) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let mut warnings = Vec::new();
    let config = Config::load(root, ctx.platform)?;
    let detection = match detect(root, config.as_ref().map(|c| c.platform).or(ctx.platform), None) {
        Ok(detection) => Some(detection),
        Err(error) => {
            warnings.push(error.message);
            None
        }
    };
    let layout = Layout::find(root);
    let lock = layout.map(|layout| Lock::load(root, layout)).transpose()?;
    let files = match &lock {
        Some(lock) => lock.status(root)?,
        None => Vec::new(),
    };
    let installed: Vec<&str> = lock
        .iter()
        .flat_map(|lock| lock.icons())
        .filter(|(_, record)| record.get("origin").and_then(Value::as_str) == Some("local"))
        .map(|(id, _)| id)
        .collect();
    let local = config.as_ref().map(|config| local_icons(root, &LocalSet::of(config), &installed, &mut warnings));
    let local_icons: Vec<&str> = match &local {
        Some(local) => local.iter().map(|icon| icon.id.as_str()).collect(),
        None => installed.clone(),
    };
    let library = ctx.library().and_then(|source| Ok((source.catalog()?.version, source.location())));
    let (library_version, library_source) = match library {
        Ok((version, location)) => (Some(version), Some(location)),
        Err(error) => {
            warnings.push(format!("Library unavailable: {}", error.message));
            (None, None)
        }
    };

    let (auth, signed_in) = match crate::engine::config(ctx) {
        Ok(config) => sign_in(&config, !ctx.offline),
        Err(error) => (json!({ "loggedIn": false, "engine": null, "error": error.message }), "not signed in".into()),
    };
    let field = |get: fn(&crate::project::Detection) -> &str| detection.as_ref().map(get);
    let value = json!({
        "cli": env!("CARGO_PKG_VERSION"),
        "libraryVersion": library_version,
        "librarySource": library_source,
        "platform": config.as_ref().map(|c| c.platform).or(detection.as_ref().map(|d| d.platform)).map(|p| p.as_str()),
        "framework": field(|d| &d.framework),
        "packageManager": field(|d| &d.package_manager),
        "sourceDirectory": field(|d| &d.source_directory),
        "config": config.as_ref().map(|c| Value::Object(c.raw.clone())),
        "files": files.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
        "localIcons": local_icons,
        "local": config.as_ref().zip(local.as_ref()).map(|(config, local)| json!({
            "source": config.local_source,
            "icons": local.iter().map(LocalIcon::to_json).collect::<Vec<_>>(),
        })),
        "auth": auth,
        "renderer": gai_render::RENDERER,
    });

    let mut text = String::new();
    let _ = writeln!(text, "generative-icons {}", env!("CARGO_PKG_VERSION"));
    let _ = match &detection {
        Some(d) => writeln!(
            text,
            "Project: {} ({}, {}, sources in {})",
            d.platform.as_str(),
            d.framework,
            d.package_manager,
            d.source_directory
        ),
        None => writeln!(text, "Project: none detected"),
    };
    let _ = match &config {
        Some(config) => writeln!(text, "Config: {}", config.layout.config_file()),
        None => writeln!(text, "Config: none (run generative-icons init)"),
    };
    let _ = match (&value["libraryVersion"], &value["librarySource"]) {
        (Value::String(version), Value::String(source)) => writeln!(text, "Library: {version} from {source}"),
        _ => writeln!(text, "Library: unavailable"),
    };
    let modified = files.iter().filter(|s| s.status == "modified").count();
    let missing = files.iter().filter(|s| s.status == "missing").count();
    let _ = writeln!(text, "Managed files: {} ({modified} modified, {missing} missing)", files.len());
    let described: Vec<String> = match &local {
        Some(local) => local.iter().map(LocalIcon::describe).collect(),
        None => local_icons.iter().map(|id| id.to_string()).collect(),
    };
    let _ =
        writeln!(text, "Local icons: {}", if described.is_empty() { "none".to_string() } else { described.join(", ") });
    let _ = writeln!(text, "Engine: {} ({signed_in})", value["auth"]["engine"].as_str().unwrap_or("unknown"));
    let _ = writeln!(text, "Renderer: {}", gai_render::RENDERER);
    Ok(Report { warnings, ..Report::new(value, text) })
}

/// One local icon: its spec or batch, its build's state and whether it is installed.
struct LocalIcon {
    id: String,
    /// Project-relative spec file, when it exists.
    spec: Option<String>,
    /// Project-relative batch file, when it exists.
    batch: Option<String>,
    /// `current`, `stale` or `not-built`.
    build: &'static str,
    installed: bool,
}

impl LocalIcon {
    fn to_json(&self) -> Value {
        json!({ "id": self.id, "spec": self.spec, "batch": self.batch, "build": self.build, "installed": self.installed })
    }

    /// `id (not built)`, `id (stale build, installed)`, …
    fn describe(&self) -> String {
        let build = match self.build {
            "current" => None,
            "stale" => Some("stale build"),
            _ => Some("not built"),
        };
        let notes: Vec<&str> = build.into_iter().chain(self.installed.then_some("installed")).collect();
        match notes.is_empty() {
            true => self.id.clone(),
            false => format!("{} ({})", self.id, notes.join(", ")),
        }
    }
}

/// Every local icon of the project, by id: those with a spec, a batch, a build or an install.
fn local_icons(
    root: &std::path::Path,
    set: &LocalSet,
    installed: &[&str],
    warnings: &mut Vec<String>,
) -> Vec<LocalIcon> {
    let mut unreadable =
        |error: crate::error::CliError| warnings.push(format!("Cannot read the local icons: {}", error.message));
    let batches = set.batch_ids(root).unwrap_or_else(|error| {
        unreadable(error);
        Vec::new()
    });
    let specs = super::v2::spec_ids(root, set).unwrap_or_else(|error| {
        unreadable(error);
        Vec::new()
    });
    let built = crate::commands::check::built_ids(root).unwrap_or_default();
    let mut ids: Vec<String> = specs
        .iter()
        .chain(&batches)
        .chain(&built)
        .cloned()
        .chain(installed.iter().map(|id| id.to_string()))
        .collect();
    ids.sort();
    ids.dedup();
    let icons_dir = set.icons_dir(root).map_err(&mut unreadable).ok();
    ids.into_iter()
        .map(|id| {
            let build = match built.contains(&id) {
                false => "not-built",
                true => match build_dir(root, &id)
                    .ok()
                    .zip(icons_dir.as_ref())
                    .and_then(|(dir, icons)| changed_source(icons, &dir))
                {
                    Some(_) => "stale",
                    None => "current",
                },
            };
            let batch = set.batch(&id);
            let batch = inside(root, &batch).is_ok_and(|path| path.is_file()).then_some(batch);
            let spec = specs.contains(&id).then(|| super::v2::spec_file(set, &id));
            LocalIcon { installed: installed.contains(&id.as_str()), id, spec, batch, build }
        })
        .collect()
}

/// The sign-in state, checked with the engine unless offline (see
/// [`gai_engine::check_session`]), as JSON and as the text after the engine URL. `loggedIn`
/// is true while the session is usable or was not checked; `session` says which.
fn sign_in(config: &EngineConfig, online: bool) -> (Value, String) {
    let engine = config.url.as_str();
    let check = match gai_engine::check_session(config, online) {
        Ok(check) => check,
        Err(error) => {
            let message = crate::engine::error(error).message;
            let stored = gai_engine::status(config).ok().flatten().is_some();
            let auth = json!({ "loggedIn": stored, "session": SessionState::NotVerified, "engine": engine, "reason": message });
            return (auth, format!("sign-in not verified: {message}"));
        }
    };
    let logged_in = matches!(check.state, SessionState::Verified | SessionState::NotVerified);
    let mut auth = json!({ "loggedIn": logged_in, "session": check.state, "engine": engine });
    if let Some(reason) = &check.reason {
        auth["reason"] = json!(reason);
    }
    let who = check.status.as_ref().and_then(|status| status.account.as_deref()).unwrap_or("an unnamed account");
    let reason = check.reason.as_deref().unwrap_or_default();
    let text = match check.state {
        SessionState::Verified => format!("signed in as {who}"),
        SessionState::NotVerified => format!("signed in as {who}; not verified: {reason}"),
        SessionState::Ended => format!("session ended: {reason}; ask the user to run `generative-icons login`"),
        SessionState::SignedOut => "not signed in".into(),
    };
    (auth, text)
}
