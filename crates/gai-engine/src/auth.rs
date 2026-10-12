//! Sign-in, sign-out, session status and the access tokens every engine request carries.

use std::io::{BufRead, Write};
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::Serialize;

use crate::config::EngineConfig;
use crate::error::ErrorKind;
use crate::error::{Error, Result};
use crate::loopback::{CallbackListener, parse_pasted_redirect};
use crate::oauth::{self, Pkce, RegisteredClient, now};
use crate::store::{CredentialStore, EngineRecord, RefreshLock, Session};
use crate::{http, mcp};

/// Refresh when the access token has less than this many seconds left.
const MIN_VALIDITY: u64 = 60;
/// Registered clients expire after 30 days; register a new one a day early.
const CLIENT_MAX_AGE: u64 = 29 * 24 * 3600;
/// How long [`check_session`] waits for the engine.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

/// How the authorization code gets back to the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    /// The browser redirects to a listener on `127.0.0.1`.
    Loopback,
    /// The user pastes the address the browser was redirected to (for machines whose browser
    /// cannot reach this one, such as over SSH).
    Paste,
}

/// Sign-in settings.
#[derive(Debug, Clone)]
pub struct LoginOptions {
    /// How the code comes back.
    pub mode: LoginMode,
    /// Open the system browser on the sign-in page.
    pub open_browser: bool,
    /// How long [`LoginMode::Loopback`] waits for the redirect.
    pub timeout: Duration,
}

impl Default for LoginOptions {
    fn default() -> Self {
        LoginOptions {
            mode: LoginMode::Loopback,
            open_browser: true,
            timeout: Duration::from_secs(300),
        }
    }
}

/// The user-facing half of sign-in.
pub trait LoginInteraction {
    /// Shows the sign-in URL; `browser_opened` says whether a browser was launched with it.
    fn show_authorization_url(&mut self, url: &str, browser_opened: bool);
    /// Returns the address the browser was redirected to ([`LoginMode::Paste`] only).
    fn read_redirect_url(&mut self) -> std::io::Result<String>;
}

/// Prompts on stderr and reads the pasted address from stdin.
#[derive(Debug, Default)]
pub struct StdioInteraction;

impl LoginInteraction for StdioInteraction {
    fn show_authorization_url(&mut self, url: &str, browser_opened: bool) {
        let mut err = std::io::stderr();
        let lead = if browser_opened {
            "Opened your browser to sign in. If it did not open, visit:"
        } else {
            "Open this address in a browser to sign in:"
        };
        let _ = writeln!(err, "{lead}\n\n  {url}\n");
    }

    fn read_redirect_url(&mut self) -> std::io::Result<String> {
        let mut err = std::io::stderr();
        let _ = write!(
            err,
            "After you approve access, the browser opens a 127.0.0.1 address that does not load.\nCopy that address from the address bar and paste it here: "
        );
        let _ = err.flush();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        Ok(line)
    }
}

/// The signed-in state, as shown by `whoami`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The engine URL.
    pub engine: String,
    /// The account from the token claims (`email`, else `sub`), when the token is a JWT.
    pub account: Option<String>,
    /// The registered OAuth client.
    pub client_id: Option<String>,
    /// RFC 3339.
    pub access_token_expires_at: Option<String>,
    /// RFC 3339, when the server states it.
    pub refresh_token_expires_at: Option<String>,
    /// `"keychain"`, `"file"` or `"environment"` (a `GAI_ENGINE_TOKEN` override).
    pub credential_store: String,
}

/// [`Status`] plus the engine's `serverInfo`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Whoami {
    /// The stored session, after any refresh.
    #[serde(flatten)]
    pub status: Status,
    /// `serverInfo.name` from `initialize`.
    pub server_name: String,
    /// `serverInfo.version` from `initialize`.
    pub server_version: String,
    /// The negotiated MCP protocol version.
    pub protocol_version: String,
}

/// How far [`check_session`] could confirm the sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    /// The engine accepted the session just now: a due token was refreshed, or the current one
    /// was accepted.
    Verified,
    /// A session is stored but was not checked: offline, or the engine could not be reached.
    NotVerified,
    /// A session is stored but cannot be used or renewed; only a new sign-in helps.
    Ended,
    /// No session is stored.
    SignedOut,
}

/// What [`check_session`] found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCheck {
    /// How far the session was confirmed.
    pub state: SessionState,
    /// The stored session, after any refresh; `None` when signed out.
    pub status: Option<Status>,
    /// Why the session was not verified, or why it ended.
    pub reason: Option<String>,
}

/// What [`logout`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutOutcome {
    /// A session was stored and has been removed.
    pub was_signed_in: bool,
    /// The server confirmed the refresh token's revocation.
    pub revoked: bool,
}

/// Signs in through the browser and stores the session.
///
/// Discovers the authorization server, reuses the registered client while it is younger than
/// 29 days (else registers one), runs PKCE with a loopback redirect or a pasted redirect URL,
/// and exchanges the code. A client the server no longer knows (`invalid_client`) is
/// registered again once. The session it replaces is revoked (best effort), so a refresh
/// token that leaked earlier does not outlive the new sign-in.
pub fn login(
    config: &EngineConfig,
    options: &LoginOptions,
    ui: &mut dyn LoginInteraction,
) -> Result<Status> {
    let client = http::client()?;
    let store = CredentialStore::open(config);
    let server = oauth::discover(&client, &config.url)?;
    let listener = CallbackListener::bind()?;
    let redirect_uri = listener.redirect_uri().to_owned();
    // In paste mode nothing listens, so a browser on this machine shows an error page whose
    // address can be copied instead of waiting on an unanswered connection.
    let listener = (options.mode == LoginMode::Loopback).then_some(listener);

    let mut registered = store
        .record()?
        .and_then(|record| record.client)
        .filter(|c| {
            c.issuer == server.issuer && now().saturating_sub(c.registered_at) < CLIENT_MAX_AGE
        });
    let mut registered_again = false;
    loop {
        let app: RegisteredClient = match registered.take() {
            Some(app) => app,
            None => oauth::register(&client, &server, &redirect_uri)?,
        };
        let pkce = Pkce::new()?;
        let state = oauth::random_token(16)?;
        let url = oauth::authorization_url(
            &server,
            &app.client_id,
            &redirect_uri,
            &pkce.challenge,
            &state,
        )?;
        let opened = options.open_browser && open::that_detached(&url).is_ok();
        ui.show_authorization_url(&url, opened);
        let code = match &listener {
            Some(listener) => listener.wait(&state, options.timeout)?,
            None => parse_pasted_redirect(&ui.read_redirect_url()?, &state)?,
        };
        match oauth::exchange_code(
            &client,
            &server,
            &app.client_id,
            &code,
            &pkce.verifier,
            &redirect_uri,
        ) {
            Ok(tokens) => {
                let record = EngineRecord {
                    engine: config.url.to_string(),
                    server,
                    client: Some(app),
                    session: Some(Session::from_tokens(tokens, None)),
                };
                replace_session(&client, &store, &record)?;
                return Ok(status_of(&record, store.backend()));
            }
            Err(Error::OAuth { error, .. }) if error == "invalid_client" && !registered_again => {
                registered_again = true;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Saves the session of a new sign-in, under the lock. The session it replaces is revoked (best
/// effort), and so is a refresh token an earlier build left under the legacy keychain account.
fn replace_session(client: &Client, store: &CredentialStore, record: &EngineRecord) -> Result<()> {
    let lock = store.lock()?;
    let previous = store.load(&lock)?;
    if let Some(previous) = &previous {
        revoke_session(client, previous);
    }
    save(
        client,
        store,
        &lock,
        previous.as_ref().unwrap_or(record),
        record,
    )
}

/// Saves `record`. The file then has the per-home format, so a legacy keychain token this home
/// still owns is revoked (best effort, as `issuer`'s client) and deleted first; nothing would
/// find it afterwards.
fn save(
    client: &Client,
    store: &CredentialStore,
    lock: &RefreshLock,
    issuer: &EngineRecord,
    record: &EngineRecord,
) -> Result<()> {
    if let Ok(Some(token)) = store.take_legacy(lock) {
        revoke_token(client, issuer, &token);
    }
    store.save(lock, record)
}

/// Revokes the refresh token when the server offers revocation, then deletes the session. A
/// refresh token an earlier build left under the legacy keychain account is revoked and deleted
/// as well. Signing out twice is fine.
pub fn logout(config: &EngineConfig) -> Result<LogoutOutcome> {
    let store = CredentialStore::open(config);
    let lock = store.lock()?;
    let record = store.load(&lock)?;
    let legacy = store.take_legacy(&lock)?;
    let session = record.as_ref().filter(|record| record.session.is_some());
    if session.is_none() && legacy.is_none() {
        return Ok(LogoutOutcome {
            was_signed_in: false,
            revoked: false,
        });
    }
    let client = http::client()?;
    let mut revoked = true;
    if let Some(token) = &legacy {
        revoked &= record
            .as_ref()
            .is_some_and(|record| revoke_token(&client, record, token));
    }
    if let Some(mut record) = session.cloned() {
        revoked &= revoke_session(&client, &record);
        record.session = None;
        store.save(&lock, &record)?;
    }
    Ok(LogoutOutcome {
        was_signed_in: true,
        revoked,
    })
}

/// A bearer token for the engine that lasts at least `secs` more seconds, refreshed (single-flight
/// across processes) when the stored one would expire sooner. It is for a LottieFiles service that
/// verifies the account with it, such as the community's submissions endpoint (`submit`).
pub fn fresh_access_token(config: &EngineConfig, secs: u64) -> Result<String> {
    Auth::new(config, http::client()?).access_token_valid_for(secs)
}

/// The stored session without touching the network; `None` when signed out.
pub fn status(config: &EngineConfig) -> Result<Option<Status>> {
    if let Some(token) = &config.token_override {
        return Ok(Some(Status {
            engine: config.url.to_string(),
            account: oauth::account_from_jwt(token),
            client_id: None,
            access_token_expires_at: None,
            refresh_token_expires_at: None,
            credential_store: "environment".into(),
        }));
    }
    let store = CredentialStore::open(config);
    Ok(store
        .record()?
        .filter(|record| record.session.is_some())
        .map(|record| status_of(&record, store.backend())))
}

/// Checks the session end to end: refreshes the token if needed and runs `initialize`.
pub fn whoami(config: &EngineConfig) -> Result<Whoami> {
    let server = mcp::McpClient::new(config)?.server_info()?;
    let status = status(config)?.ok_or_else(|| Error::NotLoggedIn {
        engine: config.url.to_string(),
    })?;
    Ok(Whoami {
        status,
        server_name: server.name,
        server_version: server.version,
        protocol_version: server.protocol_version,
    })
}

/// Checks that the session works, as cheaply as the engine allows (it has no introspection
/// endpoint). A stored session without a usable refresh token has ended. Otherwise, `online`, a
/// due access token is refreshed and a current one is presented in `initialize`; offline, or
/// when the engine cannot be reached, the session is not verified. Like every locked load, this
/// moves a refresh token an earlier build left under the legacy keychain account.
pub fn check_session(config: &EngineConfig, online: bool) -> Result<SessionCheck> {
    let check = |state, status, reason| SessionCheck {
        state,
        status,
        reason,
    };
    let mut due = false;
    let stored = match &config.token_override {
        Some(_) => status(config)?,
        None => {
            let store = CredentialStore::open(config);
            // Signed out needs neither the lock nor the keychain (nor a home to lock in).
            if store
                .record()?
                .is_none_or(|record| record.session.is_none())
            {
                return Ok(check(SessionState::SignedOut, None, None));
            }
            let record = {
                let lock = store.lock()?;
                store.load(&lock)?
            };
            let Some((record, session)) =
                record.and_then(|record| Some((record.clone(), record.session?)))
            else {
                return Ok(check(SessionState::SignedOut, None, None));
            };
            let stored = Some(status_of(&record, store.backend()));
            if let Some(reason) = unrenewable(&session) {
                return Ok(check(SessionState::Ended, stored, Some(reason)));
            }
            due = !session.valid_for(MIN_VALIDITY);
            stored
        }
    };
    if !online {
        return Ok(check(
            SessionState::NotVerified,
            stored,
            Some("offline".into()),
        ));
    }
    let mut once = config.clone();
    once.retry.attempts = 1;
    let client = http::client_with_timeout(CHECK_TIMEOUT)?;
    let checked = if due {
        Auth::new(&once, client).access_token().map(drop)
    } else {
        mcp::McpClient::with_client(&once, client)
            .server_info()
            .map(drop)
    };
    match checked {
        Ok(()) => Ok(check(
            SessionState::Verified,
            status(config)?.or(stored),
            None,
        )),
        Err(Error::SessionExpired(reason)) => Ok(check(SessionState::Ended, stored, Some(reason))),
        Err(e) if e.needs_login() => Ok(check(SessionState::Ended, stored, Some(e.to_string()))),
        Err(e) => match e.kind() {
            ErrorKind::CredentialStore | ErrorKind::Io | ErrorKind::InvalidConfig => Err(e),
            _ => Ok(check(
                SessionState::NotVerified,
                stored,
                Some(e.to_string()),
            )),
        },
    }
}

/// Why a stored session cannot be renewed, when it cannot.
fn unrenewable(session: &Session) -> Option<String> {
    if session.refresh_token.is_none() {
        return Some("no refresh token is stored".into());
    }
    session
        .refresh_token_expires_at
        .filter(|at| *at <= now())
        .map(|at| format!("the refresh token expired at {}", rfc3339(at)))
}

/// Revokes the record's refresh token when the server offers revocation; whether it confirmed.
/// The record passed [`AuthServer::check`](crate::oauth::AuthServer::check) when loaded, so the
/// token only goes to its issuer.
fn revoke_session(client: &Client, record: &EngineRecord) -> bool {
    record
        .session
        .as_ref()
        .and_then(|s| s.refresh_token.as_ref())
        .is_some_and(|token| revoke_token(client, record, token))
}

/// Revokes `token` at the record's revocation endpoint as its registered client; whether the
/// server confirmed.
fn revoke_token(client: &Client, record: &EngineRecord, token: &str) -> bool {
    match (&record.server.revocation_endpoint, &record.client) {
        (Some(endpoint), Some(app)) => {
            oauth::revoke(client, endpoint, &app.client_id, token).is_ok()
        }
        _ => false,
    }
}

fn status_of(record: &EngineRecord, backend: &str) -> Status {
    let session = record.session.as_ref();
    Status {
        engine: record.engine.clone(),
        account: session.and_then(|s| s.account.clone()),
        client_id: record.client.as_ref().map(|c| c.client_id.clone()),
        access_token_expires_at: session.map(|s| rfc3339(s.access_token_expires_at)),
        refresh_token_expires_at: session
            .and_then(|s| s.refresh_token_expires_at)
            .map(rfc3339),
        credential_store: backend.to_owned(),
    }
}

fn rfc3339(unix: u64) -> String {
    humantime::format_rfc3339_seconds(UNIX_EPOCH + Duration::from_secs(unix)).to_string()
}

/// Supplies bearer tokens, refreshing them single-flight across threads and processes.
pub(crate) struct Auth {
    engine: String,
    source: TokenSource,
}

enum TokenSource {
    /// `GAI_ENGINE_TOKEN`: used as-is, never refreshed.
    Fixed(String),
    /// The signed-in session in the credential store.
    Stored(Box<StoredTokens>),
}

struct StoredTokens {
    client: Client,
    store: CredentialStore,
    cached: Mutex<Option<Session>>,
    refreshing: Mutex<()>,
}

impl Auth {
    pub fn new(config: &EngineConfig, client: Client) -> Self {
        let source = match &config.token_override {
            Some(token) => TokenSource::Fixed(token.clone()),
            None => TokenSource::Stored(Box::new(StoredTokens {
                client,
                store: CredentialStore::open(config),
                cached: Mutex::new(None),
                refreshing: Mutex::new(()),
            })),
        };
        Auth {
            engine: config.url.to_string(),
            source,
        }
    }

    /// A token valid for at least a minute.
    pub fn access_token(&self) -> Result<String> {
        self.access_token_valid_for(MIN_VALIDITY)
    }

    /// A token valid for at least `secs` more seconds: the cached one when it lasts that
    /// long, else a refreshed one (a fixed `GAI_ENGINE_TOKEN` is used as it is).
    pub fn access_token_valid_for(&self, secs: u64) -> Result<String> {
        match &self.source {
            TokenSource::Fixed(token) => Ok(token.clone()),
            TokenSource::Stored(stored) => {
                match stored.cached().filter(|s| s.valid_for(secs)) {
                    Some(session) => Ok(session.access_token),
                    None => stored.refresh(&self.engine, None),
                }
            }
        }
    }

    /// A replacement for `rejected`, which the engine answered with 401.
    pub fn renew_rejected(&self, rejected: &str) -> Result<String> {
        match &self.source {
            TokenSource::Fixed(_) => Err(Error::SessionExpired(format!(
                "the engine rejected the token in {}",
                crate::config::ENGINE_TOKEN_ENV
            ))),
            TokenSource::Stored(stored) => stored.refresh(&self.engine, Some(rejected)),
        }
    }
}

impl StoredTokens {
    fn cached(&self) -> Option<Session> {
        self.cached
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn remember(&self, session: Option<Session>) {
        *self.cached.lock().unwrap_or_else(|e| e.into_inner()) = session;
    }

    /// Under the process mutex: a current token in the file (another process refreshed) is used
    /// as it is, without the lock or the keychain. Otherwise, under the cross-process file lock,
    /// re-read the store, use its token if another process refreshed meanwhile, else redeem the
    /// refresh token and save the rotated pair before the lock is released.
    fn refresh(&self, engine: &str, rejected: Option<&str>) -> Result<String> {
        let _in_process = self.refreshing.lock().unwrap_or_else(|e| e.into_inner());
        let usable = |session: &Session| {
            session.valid_for(MIN_VALIDITY) && rejected != Some(session.access_token.as_str())
        };
        if let Some(session) = self
            .store
            .record()?
            .and_then(|record| record.session)
            .filter(usable)
        {
            self.remember(Some(session.clone()));
            return Ok(session.access_token);
        }
        let lock = self.store.lock()?;
        let not_logged_in = || Error::NotLoggedIn {
            engine: engine.to_owned(),
        };
        let mut record = self.store.load(&lock)?.ok_or_else(not_logged_in)?;
        let session = record.session.clone().ok_or_else(not_logged_in)?;
        if usable(&session) {
            self.remember(Some(session.clone()));
            return Ok(session.access_token);
        }
        let (Some(app), Some(refresh_token)) =
            (record.client.clone(), session.refresh_token.clone())
        else {
            return Err(Error::SessionExpired("no refresh token is stored".into()));
        };
        match oauth::refresh(&self.client, &record.server, &app.client_id, &refresh_token) {
            Ok(tokens) => {
                let next = Session::from_tokens(tokens, Some(&session));
                record.session = Some(next.clone());
                if let Err(error) = save(&self.client, &self.store, &lock, &record, &record) {
                    // The server has retired the stored token, and the new one cannot be kept.
                    // Revoke the new one and drop the session, so the next run asks for a
                    // sign-in instead of replaying the old token (which revokes the chain).
                    revoke_session(&self.client, &record);
                    record.session = None;
                    let _ = self.store.save(&lock, &record);
                    self.remember(None);
                    return Err(error);
                }
                let token = next.access_token.clone();
                self.remember(Some(next));
                Ok(token)
            }
            Err(Error::OAuth {
                error, description, ..
            }) if error == "invalid_grant" || error == "invalid_client" => {
                record.session = None;
                let reason = if error == "invalid_client" {
                    record.client = None;
                    "the registered client has expired".to_owned()
                } else {
                    let detail = description.map(|d| format!(": {d}")).unwrap_or_default();
                    format!("the refresh token was rejected{detail}")
                };
                save(&self.client, &self.store, &lock, &record, &record)?;
                self.remember(None);
                Err(Error::SessionExpired(reason))
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::Path;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use serde_json::json;
    use tokio::runtime::Runtime;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::config::CredentialStoreKind;
    use crate::oauth::AuthServer;
    use crate::store::{FakeKeychain, Keychain};

    /// Token endpoint that rotates refresh tokens and revokes on reuse, like the engine's.
    #[derive(Clone, Default)]
    struct RotatingTokens {
        state: Arc<Mutex<TokenState>>,
    }

    #[derive(Default)]
    struct TokenState {
        /// The engine URL the tokens are for.
        resource: String,
        valid: HashSet<String>,
        refreshes: usize,
        reuses: usize,
        revoked: Vec<String>,
        client_expired: bool,
    }

    impl Respond for RotatingTokens {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let form: std::collections::HashMap<String, String> =
                url::form_urlencoded::parse(&request.body)
                    .into_owned()
                    .collect();
            let mut state = self.state.lock().unwrap();
            if request.url.path() == "/revoke" {
                let token = form.get("token").cloned().unwrap_or_default();
                state.valid.remove(&token);
                state.revoked.push(token);
                return ResponseTemplate::new(200);
            }
            if state.client_expired {
                return ResponseTemplate::new(401)
                    .set_body_json(json!({"error": "invalid_client"}));
            }
            let presented = form.get("refresh_token").cloned().unwrap_or_default();
            if form.get("resource") != Some(&state.resource) || !state.valid.remove(&presented) {
                state.reuses += 1;
                state.valid.clear();
                return ResponseTemplate::new(400).set_body_json(json!({"error": "invalid_grant"}));
            }
            state.refreshes += 1;
            let n = state.refreshes;
            state.valid.insert(format!("rt-{n}"));
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(100))
                .set_body_json(json!({
                    "access_token": format!("at-{n}"), "token_type": "bearer",
                    "expires_in": 300, "refresh_token": format!("rt-{n}"), "scope": "mcp:full"
                }))
        }
    }

    /// The token and revocation endpoints; the engine is `<server>/mcp`.
    fn start(tokens: &RotatingTokens) -> (Runtime, MockServer) {
        let rt = Runtime::new().unwrap();
        let server = rt.block_on(MockServer::start());
        for endpoint in ["/token", "/revoke"] {
            rt.block_on(
                Mock::given(method("POST"))
                    .and(path(endpoint))
                    .respond_with(tokens.clone())
                    .mount(&server),
            );
        }
        tokens.state.lock().unwrap().resource = format!("{}/mcp", server.uri());
        (rt, server)
    }

    fn config_for(server: &MockServer, home: &Path) -> EngineConfig {
        config(server, home)
    }

    fn config(server: &MockServer, home: &Path) -> EngineConfig {
        let mut config = EngineConfig::new(&format!("{}/mcp", server.uri()), home).unwrap();
        config.credential_store = CredentialStoreKind::File;
        config
    }

    /// Stores a session whose access token expired, with refresh token `rt-0`.
    fn seed(config: &EngineConfig) {
        seed_access(config, "at-0", now() - 1);
    }

    /// Stores a session with this access token and refresh token `rt-0`.
    fn seed_access(config: &EngineConfig, access_token: &str, expires_at: u64) {
        let store = CredentialStore::open(config);
        let base = config.origin();
        let record = EngineRecord {
            engine: config.url.to_string(),
            server: AuthServer {
                issuer: base.clone(),
                authorization_endpoint: format!("{base}/authorize"),
                token_endpoint: format!("{base}/token"),
                registration_endpoint: None,
                revocation_endpoint: Some(format!("{base}/revoke")),
                resource: config.url.to_string(),
                scope: "mcp:full".into(),
            },
            client: Some(RegisteredClient {
                client_id: "cid".into(),
                registered_at: now(),
                issuer: base,
            }),
            session: Some(Session {
                access_token: access_token.into(),
                access_token_expires_at: expires_at,
                refresh_token: Some("rt-0".into()),
                refresh_token_expires_at: None,
                scope: None,
                account: None,
            }),
        };
        let lock = store.lock().unwrap();
        store.save(&lock, &record).unwrap();
    }

    /// The stored record with its refresh token.
    fn stored(config: &EngineConfig) -> Option<EngineRecord> {
        let store = CredentialStore::open(config);
        let lock = store.lock().unwrap();
        store.load(&lock).unwrap()
    }

    /// [`config`] with refresh tokens in a fake keychain.
    fn keychain_config(server: &MockServer, home: &Path) -> (EngineConfig, Arc<FakeKeychain>) {
        let keychain = Arc::new(FakeKeychain::default());
        let mut config = config(server, home);
        config.credential_store = CredentialStoreKind::Auto;
        config.test_keychain = Some(keychain.clone());
        (config, keychain)
    }

    /// What a build before per-home accounts left: [`seed`]'s session in a version 1 file, and
    /// its refresh token in the keychain under the bare origin.
    fn seed_legacy(config: &EngineConfig, keychain: &FakeKeychain) {
        let mut file_only = config.clone();
        file_only.credential_store = CredentialStoreKind::File;
        seed(&file_only);
        let path = config.home.join("credentials.json");
        let mut file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        file["version"] = json!(1);
        let session = file["engines"][config.origin()]["session"]
            .as_object_mut()
            .unwrap();
        let token = session.remove("refreshToken").unwrap();
        std::fs::write(&path, file.to_string()).unwrap();
        keychain
            .set(&config.origin(), token.as_str().unwrap())
            .unwrap();
    }

    /// [`seed_legacy`], moved to this home's account as a first load does (the file stays at
    /// version 1), with `left_behind` under the legacy account again.
    fn seed_moved_legacy(config: &EngineConfig, keychain: &FakeKeychain, left_behind: &str) {
        seed_legacy(config, keychain);
        assert!(
            stored(config)
                .unwrap()
                .session
                .unwrap()
                .refresh_token
                .is_some()
        );
        assert_eq!(keychain.get(&config.origin()).unwrap(), None);
        keychain.set(&config.origin(), left_behind).unwrap();
    }

    /// Answers `initialize` (and the notification after it) for bearer tokens in `accepted`.
    fn mount_mcp(rt: &Runtime, server: &MockServer, accepted: &'static [&'static str]) {
        let respond = move |request: &Request| {
            let bearer = request
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or_default();
            if !accepted.contains(&bearer) {
                return ResponseTemplate::new(401);
            }
            ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1,
                "result": {"protocolVersion": "2025-11-25", "serverInfo": {"name": "mock", "version": "1"}}
            }))
        };
        rt.block_on(
            Mock::given(method("POST"))
                .and(path("/mcp"))
                .respond_with(respond)
                .mount(server),
        );
    }

    #[test]
    fn concurrent_refreshes_redeem_each_refresh_token_once() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let config = config(&server, home.path());
        seed(&config);

        // Each thread has its own Auth (own cache and mutex), like separate CLI processes;
        // only the file lock coordinates them.
        let barrier = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let config = config.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    let auth = Auth::new(&config, http::client().unwrap());
                    barrier.wait();
                    auth.access_token().unwrap()
                })
            })
            .collect();
        let issued: HashSet<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let state = tokens.state.lock().unwrap();
        assert_eq!(state.reuses, 0, "a refresh token was redeemed twice");
        assert_eq!(state.refreshes, 1);
        assert_eq!(issued, HashSet::from(["at-1".to_owned()]));
        let session = stored(&config).unwrap().session.unwrap();
        assert_eq!(session.refresh_token.as_deref(), Some("rt-1"));
    }

    #[test]
    fn a_keychain_prompt_never_holds_the_sign_in_lock() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed(&config);

        // Reading the refresh token now waits on an access prompt, as macOS asks a new binary.
        keychain.prompt.open();
        let reads = keychain.reads.load(SeqCst);
        let refreshing = {
            let config = config.clone();
            thread::spawn(move || Auth::new(&config, http::client().unwrap()).access_token())
        };
        while keychain.reads.load(SeqCst) == reads {
            thread::sleep(Duration::from_millis(2));
        }
        // Another process takes the lock while the prompt is open.
        let mut other = config.clone();
        other.credential_store = CredentialStoreKind::File;
        let (sent, taken) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let store = CredentialStore::open(&other);
            let _ = sent.send(store.lock().map(drop).is_ok());
        });
        let took = taken.recv_timeout(Duration::from_secs(5));
        keychain.prompt.answer();
        let token = refreshing.join().unwrap();
        assert_eq!(
            took,
            Ok(true),
            "the sign-in lock was held across the keychain prompt"
        );

        // The refresh still redeems rt-0 once and keeps its successor.
        assert_eq!(token.unwrap(), "at-1");
        let state = tokens.state.lock().unwrap();
        assert_eq!((state.refreshes, state.reuses), (1, 0));
        assert_eq!(keychain.entries().values().collect::<Vec<_>>(), ["rt-1"]);
    }

    #[test]
    fn slow_keychain_reads_still_redeem_each_refresh_token_once() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed(&config);
        *keychain.read_delay.lock().unwrap() = Duration::from_millis(30);

        // Every reader has read rt-0 before the first refresh rotates it.
        let barrier = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let config = config.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    let auth = Auth::new(&config, http::client().unwrap());
                    barrier.wait();
                    auth.access_token().unwrap()
                })
            })
            .collect();
        let issued: HashSet<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let state = tokens.state.lock().unwrap();
        assert_eq!(state.reuses, 0, "a refresh token was redeemed twice");
        assert_eq!(state.refreshes, 1);
        assert_eq!(issued, HashSet::from(["at-1".to_owned()]));
        assert_eq!(keychain.entries().values().collect::<Vec<_>>(), ["rt-1"]);
    }

    #[test]
    fn a_rejected_token_is_refreshed_once_then_reused() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let config = config(&server, home.path());
        seed(&config);

        let auth = Auth::new(&config, http::client().unwrap());
        assert_eq!(auth.access_token().unwrap(), "at-1");
        assert_eq!(
            auth.access_token().unwrap(),
            "at-1",
            "a fresh token is served from the cache"
        );
        assert_eq!(auth.renew_rejected("at-1").unwrap(), "at-2");
        // A second process that saw the same 401 picks up at-2 instead of refreshing again.
        let other = Auth::new(&config, http::client().unwrap());
        assert_eq!(other.renew_rejected("at-1").unwrap(), "at-2");
        assert_eq!(tokens.state.lock().unwrap().refreshes, 2);
    }

    #[test]
    fn refresh_rejections_end_the_session() {
        let tokens = RotatingTokens::default();
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let config = config(&server, home.path());
        seed(&config);

        // rt-0 is unknown to the server: invalid_grant.
        let err = Auth::new(&config, http::client().unwrap())
            .access_token()
            .unwrap_err();
        assert!(matches!(err, Error::SessionExpired(_)), "{err}");
        let record = stored(&config).unwrap();
        assert!(record.session.is_none() && record.client.is_some());
        let err = Auth::new(&config, http::client().unwrap())
            .access_token()
            .unwrap_err();
        assert!(matches!(err, Error::NotLoggedIn { .. }));

        // An expired client: the registration is dropped too, so the next login registers anew.
        seed(&config);
        tokens.state.lock().unwrap().client_expired = true;
        let err = Auth::new(&config, http::client().unwrap())
            .access_token()
            .unwrap_err();
        assert!(
            matches!(&err, Error::SessionExpired(m) if m.contains("client")),
            "{err}"
        );
        let record = stored(&config).unwrap();
        assert!(record.session.is_none() && record.client.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_save_after_rotation_revokes_the_new_token() {
        use std::os::unix::fs::PermissionsExt;
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let config = config(&server, home.path());
        seed(&config);

        // A read-only home: the rotated pair cannot be saved.
        let set_mode = |mode| {
            std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(mode)).unwrap()
        };
        set_mode(0o500);
        let auth = Auth::new(&config, http::client().unwrap());
        let failed = auth.access_token();
        set_mode(0o700);
        let err = failed.unwrap_err();
        assert!(matches!(err, Error::Io(_)), "{err}");
        let state = tokens.state.lock().unwrap();
        assert_eq!(
            (state.refreshes, state.revoked.as_slice()),
            (1, &["rt-1".to_owned()][..])
        );
        assert!(
            state.valid.is_empty(),
            "no usable refresh token is left behind"
        );
    }

    #[test]
    fn token_override_is_used_verbatim() {
        let tokens = RotatingTokens::default();
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let mut config = config(&server, home.path());
        config.token_override = Some("ci-token".into());
        let auth = Auth::new(&config, http::client().unwrap());
        assert_eq!(auth.access_token().unwrap(), "ci-token");
        assert!(matches!(
            auth.renew_rejected("ci-token"),
            Err(Error::SessionExpired(_))
        ));
        assert_eq!(
            status(&config).unwrap().unwrap().credential_store,
            "environment"
        );
    }

    #[test]
    fn a_sign_in_from_an_earlier_build_keeps_working() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_legacy(&config, &keychain);
        assert_eq!(
            keychain.entries().keys().collect::<Vec<_>>(),
            [&config.origin()]
        );

        // The first refresh redeems the legacy token, and the rotated one goes to this home's
        // account; the legacy entry is gone.
        let auth = Auth::new(&config, http::client().unwrap());
        assert_eq!(auth.access_token().unwrap(), "at-1");
        let entries = keychain.entries();
        assert_eq!(entries.len(), 1, "{:?}", entries.keys());
        let (account, token) = entries.iter().next().unwrap();
        assert!(
            account.starts_with(&format!("{} ", config.origin())),
            "{account}"
        );
        assert_eq!(token, "rt-1");
        let state = tokens.state.lock().unwrap();
        assert_eq!((state.refreshes, state.reuses), (1, 0));
    }

    #[test]
    fn logout_revokes_and_deletes_legacy_tokens() {
        let tokens = RotatingTokens::default();
        let (_rt, server) = start(&tokens);

        // Moved on the way: the session's token is the legacy one.
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_legacy(&config, &keychain);
        let outcome = logout(&config).unwrap();
        assert!(outcome.was_signed_in && outcome.revoked, "{outcome:?}");
        assert!(keychain.entries().is_empty());
        assert_eq!(tokens.state.lock().unwrap().revoked, ["rt-0"]);
        assert_eq!(status(&config).unwrap(), None);

        // Left behind next to the moved one: both are revoked.
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_moved_legacy(&config, &keychain, "rt-old");
        let outcome = logout(&config).unwrap();
        assert!(outcome.was_signed_in && outcome.revoked, "{outcome:?}");
        assert!(keychain.entries().is_empty());
        assert_eq!(
            tokens.state.lock().unwrap().revoked,
            ["rt-0", "rt-old", "rt-0"]
        );

        // Without a session of its own, logout still removes it.
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_legacy(&config, &keychain);
        let path = config.home.join("credentials.json");
        let mut file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        file["engines"][config.origin()]["session"] = serde_json::Value::Null;
        std::fs::write(&path, file.to_string()).unwrap();
        keychain.set(&config.origin(), "rt-orphan").unwrap();
        let outcome = logout(&config).unwrap();
        assert!(outcome.was_signed_in && outcome.revoked, "{outcome:?}");
        assert!(keychain.entries().is_empty());
        assert_eq!(
            tokens.state.lock().unwrap().revoked.last().unwrap(),
            "rt-orphan"
        );
        let again = logout(&config).unwrap();
        assert!(!again.was_signed_in && !again.revoked);
    }

    #[test]
    fn homes_that_do_not_own_the_legacy_token_never_touch_it() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let dir = tempfile::tempdir().unwrap();
        let keychain = Arc::new(FakeKeychain::default());
        let legacy = |config: &EngineConfig| keychain.get(&config.origin()).unwrap();
        let mut owner = config(&server, &dir.path().join("owner"));
        owner.credential_store = CredentialStoreKind::Auto;
        owner.test_keychain = Some(keychain.clone());
        seed_legacy(&owner, &keychain);
        keychain.set(&owner.origin(), "rt-owner").unwrap();
        tokens.state.lock().unwrap().valid.insert("rt-owner".into());

        // A home that never signed in (an isolated test home, say) signs out.
        let mut fresh = owner.clone();
        fresh.home = dir.path().join("fresh");
        let outcome = logout(&fresh).unwrap();
        assert!(!outcome.was_signed_in, "{outcome:?}");
        assert_eq!(legacy(&owner).as_deref(), Some("rt-owner"));
        assert!(!fresh.home.join("credentials.json").exists());

        // A home this build signed in to signs in again and signs out.
        let mut current = owner.clone();
        current.home = dir.path().join("current");
        seed(&current);
        let mut record = stored(&current).unwrap();
        record.session.as_mut().unwrap().refresh_token = Some("rt-new".into());
        let store = CredentialStore::open(&current);
        replace_session(&http::client().unwrap(), &store, &record).unwrap();
        logout(&current).unwrap();
        assert_eq!(legacy(&owner).as_deref(), Some("rt-owner"));
        assert!(
            !tokens
                .state
                .lock()
                .unwrap()
                .revoked
                .contains(&"rt-owner".to_owned())
        );

        // The owner still refreshes with it.
        assert_eq!(
            Auth::new(&owner, http::client().unwrap())
                .access_token()
                .unwrap(),
            "at-1"
        );
        assert_eq!(legacy(&owner), None);
    }

    #[test]
    fn a_left_behind_legacy_token_is_revoked_before_the_file_moves_on() {
        let tokens = RotatingTokens::default();
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        // This home's account has the current token; the legacy one is left behind.
        seed_moved_legacy(&config, &keychain, "rt-old");

        // The first refresh saves a file of the new format, so it retires the legacy token
        // first; nothing could find it afterwards.
        let auth = Auth::new(&config, http::client().unwrap());
        assert_eq!(auth.access_token().unwrap(), "at-1");
        assert_eq!(keychain.get(&config.origin()).unwrap(), None);
        assert_eq!(tokens.state.lock().unwrap().revoked, ["rt-old"]);
        assert_eq!(keychain.entries().values().collect::<Vec<_>>(), ["rt-1"]);
    }

    #[test]
    fn a_new_sign_in_revokes_legacy_tokens() {
        let tokens = RotatingTokens::default();
        let (_rt, server) = start(&tokens);
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_moved_legacy(&config, &keychain, "rt-old");

        let mut record = stored(&config).unwrap();
        record.session.as_mut().unwrap().refresh_token = Some("rt-new".into());
        let store = CredentialStore::open(&config);
        replace_session(&http::client().unwrap(), &store, &record).unwrap();
        assert_eq!(tokens.state.lock().unwrap().revoked, ["rt-0", "rt-old"]);
        let entries = keychain.entries();
        assert_eq!(entries.values().collect::<Vec<_>>(), ["rt-new"]);
        assert!(!entries.contains_key(&config.origin()));
    }

    #[test]
    fn session_checks_report_what_they_could_confirm() {
        let tokens = RotatingTokens::default();
        let (rt, server) = start(&tokens);
        mount_mcp(&rt, &server, &["at-live", "at-1", "ci-token"]);
        let state = |config: &EngineConfig, online| check_session(config, online).unwrap();

        // Signed out.
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        let signed_out = state(&config, true);
        assert_eq!(
            (signed_out.state, signed_out.status),
            (SessionState::SignedOut, None)
        );
        let missing = home.path().join("never-signed-in");
        let (elsewhere, _) = keychain_config(&server, &missing);
        assert_eq!(state(&elsewhere, false).state, SessionState::SignedOut);
        assert!(!missing.exists(), "checking created the home");

        // A session whose refresh token is not stored has ended, online or not.
        seed(&config);
        keychain.entries.lock().unwrap().clear();
        for online in [false, true] {
            let ended = state(&config, online);
            assert_eq!(ended.state, SessionState::Ended);
            assert_eq!(ended.reason.as_deref(), Some("no refresh token is stored"));
            assert!(ended.status.is_some());
        }
        assert_eq!(tokens.state.lock().unwrap().refreshes, 0);

        // A legacy sign-in is found; offline it is not verified.
        let home = tempfile::tempdir().unwrap();
        let (config, keychain) = keychain_config(&server, home.path());
        seed_legacy(&config, &keychain);
        let offline = state(&config, false);
        assert_eq!(offline.state, SessionState::NotVerified);
        assert_eq!(offline.reason.as_deref(), Some("offline"));

        // Online, the due token is refreshed (and nothing else is sent).
        tokens.state.lock().unwrap().valid.insert("rt-0".into());
        let verified = state(&config, true);
        assert_eq!(
            (verified.state, verified.reason),
            (SessionState::Verified, None)
        );
        assert_eq!(tokens.state.lock().unwrap().refreshes, 1);
        let expires = verified.status.unwrap().access_token_expires_at.unwrap();
        assert_eq!(
            expires,
            rfc3339(
                stored(&config)
                    .unwrap()
                    .session
                    .unwrap()
                    .access_token_expires_at
            )
        );

        // A current token is presented to the engine instead.
        let home = tempfile::tempdir().unwrap();
        let config = config_for(&server, home.path());
        seed_access(&config, "at-live", now() + 240);
        assert_eq!(state(&config, true).state, SessionState::Verified);
        assert_eq!(tokens.state.lock().unwrap().refreshes, 1);

        // The engine rejects it and the refresh token is dead: ended.
        let home = tempfile::tempdir().unwrap();
        let config = config_for(&server, home.path());
        seed_access(&config, "at-revoked", now() + 240);
        let ended = state(&config, true);
        assert_eq!(ended.state, SessionState::Ended, "{ended:?}");
        assert!(
            ended
                .reason
                .unwrap()
                .starts_with("the refresh token was rejected")
        );

        // The engine cannot be reached: not verified, with the reason.
        let home = tempfile::tempdir().unwrap();
        let mut config = EngineConfig::new("http://127.0.0.1:1/mcp", home.path()).unwrap();
        config.credential_store = CredentialStoreKind::File;
        seed_access(&config, "at-live", now() + 240);
        let unreachable = state(&config, true);
        assert_eq!(unreachable.state, SessionState::NotVerified);
        assert!(unreachable.reason.unwrap().starts_with("network error"));

        // A token override is checked against the engine as well.
        let home = tempfile::tempdir().unwrap();
        let mut config = config_for(&server, home.path());
        config.token_override = Some("ci-token".into());
        assert_eq!(state(&config, false).state, SessionState::NotVerified);
        assert_eq!(state(&config, true).state, SessionState::Verified);
        config.token_override = Some("ci-expired".into());
        let rejected = state(&config, true);
        assert_eq!(rejected.state, SessionState::Ended);
        assert_eq!(
            rejected.reason.as_deref(),
            Some("the engine rejected the token in GAI_ENGINE_TOKEN")
        );
    }
}
