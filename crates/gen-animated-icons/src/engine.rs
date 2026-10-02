//! The hosted engine: which endpoint and credentials a run uses, and the CLI codes of
//! its errors.
//!
//! The URL comes from `--engine-url` (or `GEN_ANIMATED_ICONS_ENGINE_URL`), then
//! `GAI_ENGINE_URL`, then the project config's `engine.url`, then the default. The sign-in
//! goes to that engine, so a project config, which arrives with the code, may only name a
//! LottieFiles engine. Credentials live under `GEN_ANIMATED_ICONS_HOME`; `GAI_ENGINE_TOKEN`
//! replaces the stored sign-in and goes only to the `GAI_ENGINE_URL` engine (default the
//! LottieFiles one) or another LottieFiles engine.

use crate::Context;
use crate::config::CONFIG_FILE;
use crate::error::{CliError, ErrorCode, Result};
use gai_engine::config::{ENGINE_URL_ENV, is_lottiefiles_engine};
use gai_engine::{EngineConfig, ErrorKind, Notice};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

/// What to do on `NOT_LOGGED_IN` and `SESSION_EXPIRED`. Sign-in opens a browser, so it is
/// the user's step, never an agent's.
pub const LOGIN_HINT: &str = "Ask the user to run `gen-animated-icons login` in their terminal (it opens a browser; add --no-browser over SSH), then retry";

/// What to do on `SIGN_IN_BUSY`.
pub const SIGN_IN_BUSY_HINT: &str = "Another gen-animated-icons process is signing in or waiting on a Keychain prompt: approve the prompt, or end that process, then retry";

/// Engine settings for this run. Its notices go to stderr: a wait for the sign-in lock as a
/// warning, a keychain read as a `--verbose` note.
pub fn config(ctx: &Context) -> Result<EngineConfig> {
    let mut config = resolve(ctx)?;
    let out = ctx.out.clone();
    config.notices = Some(Arc::new(move |notice: Notice| match notice.is_detail() {
        true => out.log(notice.message()),
        false => out.warning(notice.message()),
    }));
    Ok(config)
}

/// The engine URL and credentials home for this run.
fn resolve(ctx: &Context) -> Result<EngineConfig> {
    let base = EngineConfig::from_env().map_err(error)?;
    if let Some(url) = ctx.engine_url.as_deref().filter(|url| !url.trim().is_empty()) {
        return base.with_url(url).map_err(error);
    }
    if std::env::var(ENGINE_URL_ENV).is_ok_and(|url| !url.trim().is_empty()) {
        return Ok(base);
    }
    let Some(url) = project_url(&ctx.cwd) else { return Ok(base) };
    let named = EngineConfig::new(&url, &base.home).map_err(error)?;
    if !is_lottiefiles_engine(&named.url) {
        return Err(CliError::new(
            ErrorCode::InvalidConfig,
            format!("{CONFIG_FILE} names the engine {url}, but a project may only name a LottieFiles engine"),
        )
        .with_hint(
            "Remove engine.url, or choose the engine yourself with --engine-url or GEN_ANIMATED_ICONS_ENGINE_URL",
        ));
    }
    base.with_url(&url).map_err(error)
}

/// `engine.url` from the project config, when the config is readable.
fn project_url(root: &Path) -> Option<String> {
    let bytes = std::fs::read(root.join(CONFIG_FILE)).ok()?;
    let config: Value = serde_json::from_slice(&bytes).ok()?;
    config.pointer("/engine/url")?.as_str().map(str::to_string)
}

/// Fails with `NETWORK` under `--offline`, which never uses the network: `command` exports
/// through the hosted engine. `hint` says what to do instead.
pub fn require_online(ctx: &Context, command: &str, hint: &str) -> Result<()> {
    refuse_offline(ctx, &format!("{command} exports through the hosted engine"), hint)
}

/// Fails with `NETWORK` under `--offline`: `needs` says what reaches the network, as the
/// start of a sentence, and `hint` what to do instead.
pub fn refuse_offline(ctx: &Context, needs: &str, hint: &str) -> Result<()> {
    match ctx.offline {
        true => Err(CliError::new(ErrorCode::Network, format!("{needs}, and --offline never uses the network"))
            .with_hint(hint)),
        false => Ok(()),
    }
}

/// Fails with `NOT_LOGGED_IN` unless a session (or a token override) is available.
pub fn require_session(config: &EngineConfig) -> Result<gai_engine::Status> {
    gai_engine::status(config).map_err(error)?.ok_or_else(|| not_logged_in(config))
}

/// `NOT_LOGGED_IN` for the engine of `config`.
pub fn not_logged_in(config: &EngineConfig) -> CliError {
    CliError::new(ErrorCode::NotLoggedIn, format!("Not signed in to {}", config.url)).with_hint(LOGIN_HINT)
}

/// The CLI error for an engine failure.
pub fn error(error: gai_engine::Error) -> CliError {
    with_hint(CliError::new(code(error.kind()), sentence(&error.to_string())), &error)
}

/// The CLI error for an exporter failure: the engine's own code when the exporter is
/// the hosted engine, otherwise `ENGINE_ERROR`.
pub fn export_error(animation: &str, error: &anyhow::Error) -> CliError {
    let message = format!("{animation}: export failed: {error:#}");
    match error.downcast_ref::<gai_engine::Error>() {
        Some(engine) => with_hint(CliError::new(code(engine.kind()), message), engine),
        None => CliError::new(ErrorCode::EngineError, message),
    }
}

/// Every kind has its code; a new kind does not compile until it gets one.
fn code(kind: ErrorKind) -> ErrorCode {
    match kind {
        ErrorKind::NotLoggedIn => ErrorCode::NotLoggedIn,
        ErrorKind::SessionExpired => ErrorCode::SessionExpired,
        ErrorKind::LoginCancelled => ErrorCode::LoginCancelled,
        ErrorKind::Auth => ErrorCode::AuthError,
        ErrorKind::RateLimited => ErrorCode::RateLimited,
        ErrorKind::Unavailable => ErrorCode::EngineUnavailable,
        ErrorKind::Timeout => ErrorCode::EngineTimeout,
        ErrorKind::Network => ErrorCode::Network,
        ErrorKind::Engine => ErrorCode::EngineError,
        // The CLI wrote the scene, so an unreadable one is a bug here.
        ErrorKind::InvalidScene => ErrorCode::Internal,
        ErrorKind::CredentialStore => ErrorCode::CredentialStore,
        ErrorKind::SignInBusy => ErrorCode::SignInBusy,
        ErrorKind::Io => ErrorCode::Io,
        ErrorKind::InvalidConfig => ErrorCode::InvalidConfig,
    }
}

/// The next step for errors that have one: sign in, or let the other process finish.
fn with_hint(cli: CliError, error: &gai_engine::Error) -> CliError {
    match error.kind() {
        _ if error.needs_login() => cli.with_hint(LOGIN_HINT),
        ErrorKind::SignInBusy => cli.with_hint(SIGN_IN_BUSY_HINT),
        _ => cli,
    }
}

/// Capitalizes the first letter, as every CLI message starts.
fn sentence(message: &str) -> String {
    let mut chars = message.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_errors_keep_their_codes() {
        let error = error(gai_engine::Error::NotLoggedIn { engine: "https://e.test/mcp".into() });
        assert_eq!(error.code, ErrorCode::NotLoggedIn);
        assert_eq!(error.code.exit_code(), 4);
        assert_eq!(error.message, "Not signed in to https://e.test/mcp");
        assert_eq!(error.hint.as_deref(), Some(LOGIN_HINT));
        let expired = super::error(gai_engine::Error::SessionExpired("revoked".into()));
        assert_eq!((expired.code, expired.hint.as_deref()), (ErrorCode::SessionExpired, Some(LOGIN_HINT)));
        let limited = export_error("x", &anyhow::Error::new(gai_engine::Error::RateLimited));
        assert_eq!((limited.code, limited.code.exit_code()), (ErrorCode::RateLimited, 5));
        assert_eq!(export_error("x", &anyhow::anyhow!("boom")).code, ErrorCode::EngineError);
        let scene = serde_json::from_str::<Value>("{").unwrap_err();
        assert_eq!(super::error(gai_engine::Error::InvalidScene(scene)).code, ErrorCode::Internal);
    }

    #[test]
    fn a_busy_sign_in_lock_says_what_holds_it() {
        let busy = || gai_engine::Error::SignInBusy {
            lock: std::path::PathBuf::from("credentials/refresh.lock"),
            waited: std::time::Duration::from_secs(60),
        };
        let error = error(busy());
        assert_eq!((error.code.as_str(), error.code.exit_code()), ("SIGN_IN_BUSY", 4));
        assert_eq!(
            error.message,
            "Another process has held the sign-in lock (credentials/refresh.lock) for 60 s"
        );
        assert_eq!(error.hint.as_deref(), Some(SIGN_IN_BUSY_HINT));
        assert!(SIGN_IN_BUSY_HINT.contains("Keychain prompt"));
        let exported = export_error("x", &anyhow::Error::new(busy()));
        assert_eq!((exported.code, exported.hint.as_deref()), (ErrorCode::SignInBusy, Some(SIGN_IN_BUSY_HINT)));
    }
}
