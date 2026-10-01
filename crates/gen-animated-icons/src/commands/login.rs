//! `login`, `logout` and `whoami`: the hosted engine session (OAuth through the browser,
//! or a pasted redirect address where no browser can reach this machine).
//!
//! `--offline` never uses the network: `login` and `logout` refuse with `NETWORK`, and
//! `whoami` reports the stored session without checking it (`session: "not-verified"`).

use crate::Context;
use crate::cli::LoginArgs;
use crate::engine;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use gai_engine::{EngineConfig, LoginMode, LoginOptions, SessionState, Status, StdioInteraction};
use serde_json::{Map, Value, json};
use std::fmt::Write;

/// Runs `login`.
pub fn login(ctx: &Context, args: &LoginArgs) -> Result<Report> {
    engine::refuse_offline(
        ctx,
        "login signs in through the hosted engine",
        "Run gen-animated-icons login without --offline",
    )?;
    let config = engine::config(ctx)?;
    let paste = args.no_browser || headless();
    let options = LoginOptions {
        mode: if paste { LoginMode::Paste } else { LoginMode::Loopback },
        open_browser: !paste,
        ..LoginOptions::default()
    };
    let status = gai_engine::login(&config, &options, &mut StdioInteraction).map_err(engine::error)?;
    let mut value = status_json(&status);
    value.shift_insert(0, "loggedIn".into(), json!(true));
    let text =
        format!("Signed in to {} as {}.\n", status.engine, status.account.as_deref().unwrap_or("an unnamed account"));
    Ok(Report::new(Value::Object(value), text))
}

/// Runs `logout`.
pub fn logout(ctx: &Context) -> Result<Report> {
    engine::refuse_offline(
        ctx,
        "logout revokes the session with the hosted engine",
        "Run gen-animated-icons logout without --offline",
    )?;
    let config = engine::config(ctx)?;
    let outcome = gai_engine::logout(&config).map_err(engine::error)?;
    let value = json!({ "loggedOut": true, "engine": config.url.as_str(), "wasSignedIn": outcome.was_signed_in, "revoked": outcome.revoked });
    let text = match outcome.was_signed_in {
        true => format!("Signed out of {}.\n", config.url),
        false => format!("Not signed in to {}; nothing to do.\n", config.url),
    };
    Ok(Report::new(value, text))
}

/// Runs `whoami`: checks the session end to end (refresh and `initialize`), or under
/// `--offline` reports the stored session without checking it.
pub fn whoami(ctx: &Context) -> Result<Report> {
    let config = engine::config(ctx)?;
    if ctx.offline {
        return stored_session(&config);
    }
    engine::require_session(&config)?;
    let who = gai_engine::whoami(&config).map_err(engine::error)?;
    let server = [
        ("serverVersion", json!(who.server_version)),
        ("serverName", json!(who.server_name)),
        ("protocolVersion", json!(who.protocol_version)),
    ];
    let value = session_json(&who.status, SessionState::Verified, None, server);
    let mut text = String::new();
    let _ = writeln!(text, "Signed in to {} ({} {}).", who.status.engine, who.server_name, who.server_version);
    let _ = writeln!(text, "Account: {}", who.status.account.as_deref().unwrap_or("unknown"));
    if let Some(expires) = &who.status.access_token_expires_at {
        let _ = writeln!(text, "Access token valid until {expires}.");
    }
    Ok(Report::new(value, text))
}

/// `whoami --offline`: the session as the credentials store holds it, never sent to the
/// engine. Signed out is `NOT_LOGGED_IN` and an ended session `SESSION_EXPIRED`, as online.
fn stored_session(config: &EngineConfig) -> Result<Report> {
    let check = gai_engine::check_session(config, false).map_err(engine::error)?;
    let reason = check.reason.clone().unwrap_or_default();
    let status = match (check.state, check.status) {
        (SessionState::SignedOut, _) | (_, None) => return Err(engine::not_logged_in(config)),
        (SessionState::Ended, _) => {
            return Err(CliError::new(
                ErrorCode::SessionExpired,
                format!("The session with {} has ended: {reason}", config.url),
            )
            .with_hint(engine::LOGIN_HINT));
        }
        (_, Some(status)) => status,
    };
    let server = [("serverVersion", Value::Null), ("serverName", Value::Null), ("protocolVersion", Value::Null)];
    let value = session_json(&status, SessionState::NotVerified, Some(&reason), server);
    let mut text = String::new();
    let _ = writeln!(
        text,
        "Signed in to {} as {}; not verified: {reason} (run whoami without --offline to check the session with the engine).",
        status.engine,
        status.account.as_deref().unwrap_or("an unnamed account")
    );
    if let Some(expires) = &status.access_token_expires_at {
        let _ = writeln!(text, "Access token valid until {expires}.");
    }
    Ok(Report::new(value, text))
}

/// `whoami`'s JSON: `{engine, session, reason?, serverVersion, account, clientId,
/// accessTokenExpiresAt, refreshTokenExpiresAt, credentialStore, serverName,
/// protocolVersion}`, the server fields from `server`.
fn session_json(
    status: &Status,
    session: SessionState,
    reason: Option<&str>,
    server: [(&str, Value); 3],
) -> Value {
    let stored = status_json(status);
    let [(version_key, version), rest @ ..] = server;
    let mut value = Map::new();
    value.insert("engine".into(), json!(status.engine));
    value.insert("session".into(), json!(session));
    if let Some(reason) = reason {
        value.insert("reason".into(), json!(reason));
    }
    value.insert(version_key.into(), version);
    for key in ["account", "clientId", "accessTokenExpiresAt", "refreshTokenExpiresAt", "credentialStore"] {
        value.insert(key.into(), stored.get(key).cloned().unwrap_or(Value::Null));
    }
    for (key, field) in rest {
        value.insert(key.into(), field);
    }
    Value::Object(value)
}

/// The session in camelCase, as gai-engine serializes it.
fn status_json(status: &Status) -> Map<String, Value> {
    match serde_json::to_value(status) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

/// CI and SSH sessions cannot receive the browser's loopback redirect.
fn headless() -> bool {
    ["CI", "SSH_CONNECTION", "SSH_TTY"].iter().any(|name| std::env::var_os(name).is_some_and(|v| !v.is_empty()))
}
