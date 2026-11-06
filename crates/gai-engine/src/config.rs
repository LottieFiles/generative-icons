//! Where the engine lives and how the client talks to it.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::error::{Error, Result};

/// The hosted engine used when nothing else is configured: Creator MCP, LottieFiles' production
/// server.
pub const DEFAULT_ENGINE_URL: &str = "https://creator.mcp.lottiefiles.com/mcp";
/// Hosts of LottieFiles' engines (https on the default port), the only engines a project
/// config may choose and the ones [`ENGINE_TOKEN_ENV`] may always go to: Creator MCP's
/// production server.
pub const LOTTIEFILES_ENGINE_HOSTS: [&str; 1] = ["creator.mcp.lottiefiles.com"];
/// Overrides the engine URL.
pub const ENGINE_URL_ENV: &str = "GAI_ENGINE_URL";
/// A bearer token used as-is instead of the stored sign-in (for CI). The engine has no API keys
/// yet, so this only helps with a short-lived access token obtained elsewhere. It is sent only
/// to the engine of [`ENGINE_URL_ENV`] (default [`DEFAULT_ENGINE_URL`]) or a LottieFiles engine.
pub const ENGINE_TOKEN_ENV: &str = "GAI_ENGINE_TOKEN";
/// Overrides the directory that holds the credentials file and the refresh lock.
pub const HOME_ENV: &str = "GEN_ANIMATED_ICONS_HOME";
/// The engine allows four active documents per user.
pub const MAX_DOCUMENTS: usize = 4;
/// Documents the exporter uses by default, leaving one for the user's own sessions.
pub const DEFAULT_DOCUMENTS: usize = 3;

/// Where the refresh token is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialStoreKind {
    /// The OS keychain, or the credentials file when no keychain is available.
    Auto,
    /// Only the credentials file (mode 0600).
    File,
}

/// How often and how long to retry 429, 5xx and network failures.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Attempts per request, the first included.
    pub attempts: u32,
    /// Delay before the first retry; it doubles on each further retry.
    pub base_delay: Duration,
    /// Upper bound for one delay, `Retry-After` included.
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            attempts: 5,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// How long a command waits for the sign-in lock while another process holds it (that process
/// is refreshing, signing in, or waiting on a keychain prompt).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockWait {
    /// When to say so ([`Notice::WaitingForSignInLock`]).
    pub notice_after: Duration,
    /// When to give up with [`Error::SignInBusy`](crate::Error::SignInBusy).
    pub give_up_after: Duration,
}

impl Default for LockWait {
    fn default() -> Self {
        LockWait {
            notice_after: Duration::from_secs(3),
            give_up_after: Duration::from_secs(60),
        }
    }
}

/// What the person running a command should hear while it waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Notice {
    /// Another process has held the sign-in lock for [`LockWait::notice_after`]; this one
    /// keeps waiting.
    WaitingForSignInLock,
    /// The refresh token is about to be read from the OS keychain, which may ask the user to
    /// allow it (macOS does for a binary it has not seen).
    ReadingKeychain,
    /// A keychain call has not answered for a few seconds: a prompt is likely waiting for the
    /// user to allow this program.
    WaitingForKeychain,
}

impl Notice {
    /// The line to show.
    pub fn message(self) -> &'static str {
        match self {
            Notice::WaitingForSignInLock => {
                "Waiting for another generative-icons process that holds the sign-in lock…"
            }
            Notice::ReadingKeychain if cfg!(target_os = "macos") => {
                "Reading the sign-in from the macOS Keychain; macOS may ask you to allow access"
            }
            Notice::ReadingKeychain => {
                "Reading the sign-in from the system keychain, which may ask you to allow access"
            }
            Notice::WaitingForKeychain if cfg!(target_os = "macos") => {
                "Waiting for the macOS Keychain: a prompt asks to allow this program to use its sign-in. Choose Always Allow (a new or rebuilt binary asks once), or Deny to stop."
            }
            Notice::WaitingForKeychain => {
                "Waiting for the system keychain, which may be asking to allow this program to use its sign-in"
            }
        }
    }

    /// Whether it is a detail, shown only on request, rather than a wait the user should know
    /// about.
    pub fn is_detail(self) -> bool {
        matches!(self, Notice::ReadingKeychain)
    }
}

/// Receives [`Notice`]s; see [`EngineConfig::notices`].
pub type NoticeSink = Arc<dyn Fn(Notice) + Send + Sync>;

/// Sends `notice` to `sink`; without one, a wait goes to stderr and a detail nowhere.
pub(crate) fn notify(sink: Option<&NoticeSink>, notice: Notice) {
    match sink {
        Some(sink) => sink(notice),
        None if !notice.is_detail() => eprintln!("{}", notice.message()),
        None => {}
    }
}

/// Engine client settings.
#[derive(Clone)]
pub struct EngineConfig {
    /// The engine's MCP endpoint.
    pub url: Url,
    /// Directory for `credentials.json` and `refresh.lock`.
    pub home: PathBuf,
    /// Where the refresh token is stored.
    pub credential_store: CredentialStoreKind,
    /// Bearer token that replaces the stored sign-in (see [`ENGINE_TOKEN_ENV`]).
    pub token_override: Option<String>,
    /// Engine documents the exporter uses in parallel, `1..=MAX_DOCUMENTS`.
    pub documents: usize,
    /// Retry behaviour for engine requests.
    pub retry: RetryPolicy,
    /// How long to wait for the sign-in lock.
    pub lock_wait: LockWait,
    /// Where [`Notice`]s go; `None` prints waits on stderr and drops details.
    pub notices: Option<NoticeSink>,
    /// The keychain an `Auto` store uses in unit tests, which never touch the OS keychain.
    #[cfg(test)]
    pub(crate) test_keychain: Option<std::sync::Arc<dyn crate::store::Keychain>>,
}

impl EngineConfig {
    /// Settings for `url` with credentials under `home`, ignoring the environment.
    pub fn new(url: &str, home: impl Into<PathBuf>) -> Result<Self> {
        Ok(EngineConfig {
            url: parse_engine_url(url)?,
            home: home.into(),
            credential_store: CredentialStoreKind::Auto,
            token_override: None,
            documents: DEFAULT_DOCUMENTS,
            retry: RetryPolicy::default(),
            lock_wait: LockWait::default(),
            notices: None,
            #[cfg(test)]
            test_keychain: None,
        })
    }

    /// Settings from the environment: [`ENGINE_URL_ENV`] (default [`DEFAULT_ENGINE_URL`]),
    /// [`HOME_ENV`] (default `<config dir>/generative-icons`) and [`ENGINE_TOKEN_ENV`].
    pub fn from_env() -> Result<Self> {
        let url = non_empty_env(ENGINE_URL_ENV).unwrap_or_else(|| DEFAULT_ENGINE_URL.to_owned());
        let home = match non_empty_env(HOME_ENV) {
            Some(dir) => PathBuf::from(dir),
            None => default_home()?,
        };
        let mut config = EngineConfig::new(&url, home)?;
        config.token_override = non_empty_env(ENGINE_TOKEN_ENV);
        Ok(config)
    }

    /// These settings for the engine at `url`. A [`token_override`](Self::token_override) is
    /// bound to the current engine: it is kept when `url` has the same origin or is a
    /// LottieFiles engine, and otherwise this fails rather than send it to another origin.
    pub fn with_url(self, url: &str) -> Result<Self> {
        let parsed = parse_engine_url(url)?;
        let same_origin = parsed.origin() == self.url.origin();
        if self.token_override.is_some() && !same_origin && !is_lottiefiles_engine(&parsed) {
            return Err(Error::Config(format!(
                "{ENGINE_TOKEN_ENV} is only sent to {} or a LottieFiles engine, not {}; set {ENGINE_URL_ENV} to the engine it belongs to, or unset {ENGINE_TOKEN_ENV}",
                self.origin(),
                parsed.origin().ascii_serialization()
            )));
        }
        Ok(EngineConfig {
            url: parsed,
            ..self
        })
    }

    /// The engine origin (`scheme://host[:port]`), the key credentials are stored under.
    pub fn origin(&self) -> String {
        self.url.origin().ascii_serialization()
    }

    /// The fixed document ids the exporter rotates through.
    pub fn document_ids(&self) -> Vec<String> {
        (1..=self.documents.clamp(1, MAX_DOCUMENTS))
            .map(|i| format!("gen-animated-icons-export-{i}"))
            .collect()
    }
}

impl std::fmt::Debug for EngineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineConfig")
            .field("url", &self.url.as_str())
            .field("home", &self.home)
            .field("credential_store", &self.credential_store)
            .field(
                "token_override",
                &self.token_override.as_ref().map(|_| "[redacted]"),
            )
            .field("documents", &self.documents)
            .field("retry", &self.retry)
            .field("lock_wait", &self.lock_wait)
            .field("notices", &self.notices.is_some())
            .finish()
    }
}

/// Whether `url` is one of LottieFiles' engines: https, default port, a
/// [`LOTTIEFILES_ENGINE_HOSTS`] host.
pub fn is_lottiefiles_engine(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url
            .host_str()
            .is_some_and(|host| LOTTIEFILES_ENGINE_HOSTS.contains(&host))
}

/// https, or http to a loopback host (`localhost`, `127.0.0.0/8`, `[::1]`): the URLs credentials
/// and pinned downloads may travel over. Judges the parsed URL, so pass it what the client sends.
pub fn secure_or_loopback(url: &Url) -> bool {
    let loopback = match url.host() {
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };
    url.scheme() == "https" || (url.scheme() == "http" && loopback)
}

fn parse_engine_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).map_err(|e| Error::Config(format!("engine URL {raw:?}: {e}")))?;
    if !secure_or_loopback(&url) {
        return Err(Error::Config(format!(
            "engine URL {raw:?} must use https (http is allowed for loopback only)"
        )));
    }
    Ok(url)
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn default_home() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("generative-icons"))
        .ok_or_else(|| Error::Config(format!("no home directory; set {HOME_ENV}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_url_must_be_https_or_loopback() {
        assert!(EngineConfig::new("https://creator.mcp.lottiefiles.com/mcp", "/tmp").is_ok());
        assert!(EngineConfig::new("http://127.0.0.1:9000/mcp", "/tmp").is_ok());
        assert!(EngineConfig::new("http://example.com/mcp", "/tmp").is_err());
        assert!(EngineConfig::new("not a url", "/tmp").is_err());
    }

    #[test]
    fn loopback_is_judged_on_the_parsed_host() {
        let secure = |raw: &str| secure_or_loopback(&Url::parse(raw).unwrap());
        for good in [
            "https://example.com/",
            "http://localhost:9/",
            "http://LOCALHOST/",
            "http://127.0.0.2/",
            "http://[::1]:9/",
            // Userinfo: the request still goes to localhost.
            "http://evil.example%5C@localhost/",
        ] {
            assert!(secure(good), "{good}");
        }
        for bad in [
            "http://192.168.45.216:18902\\@localhost/lib",
            "http://localhost@evil.example/",
            "http://127.0.0.1.evil.example/",
            "http://[::ffff:10.0.0.1]/",
            "ftp://localhost/",
        ] {
            assert!(!secure(bad), "{bad}");
        }
    }

    #[test]
    fn only_lottiefiles_engines_are_on_the_allowlist() {
        let lottiefiles = |raw: &str| is_lottiefiles_engine(&Url::parse(raw).unwrap());
        assert!(lottiefiles(DEFAULT_ENGINE_URL));
        assert!(lottiefiles("https://creator.mcp.lottiefiles.com/mcp"));
        assert!(lottiefiles("https://creator.mcp.lottiefiles.com:443/mcp"));
        assert!(!lottiefiles("http://creator.mcp.lottiefiles.com/mcp"));
        assert!(!lottiefiles("https://creator.mcp.lottiefiles.com:8443/mcp"));
        assert!(!lottiefiles(
            "https://creator.mcp.lottiefiles.com.evil.example/mcp"
        ));
        assert!(!lottiefiles(
            "https://evil.example/creator.mcp.lottiefiles.com"
        ));
        // The production host Creator MCP left on 2 Nov 2026 is not on the list.
        assert!(!lottiefiles("https://creator-mcp.lottiefiles.com/mcp"));
    }

    #[test]
    fn the_token_override_never_follows_the_url_to_another_origin() {
        let mut config = EngineConfig::new("https://ci-engine.example/mcp", "/tmp").unwrap();
        config.token_override = Some("ci-secret".into());
        let moved = config
            .clone()
            .with_url("https://ci-engine.example/other")
            .unwrap();
        assert_eq!(moved.token_override.as_deref(), Some("ci-secret"));
        let moved = config.clone().with_url(DEFAULT_ENGINE_URL).unwrap();
        assert_eq!(moved.url.as_str(), DEFAULT_ENGINE_URL);
        assert_eq!(moved.token_override.as_deref(), Some("ci-secret"));
        let err = config
            .clone()
            .with_url("https://evil.example/mcp")
            .unwrap_err();
        assert!(
            matches!(&err, Error::Config(m) if m.contains(ENGINE_TOKEN_ENV)),
            "{err}"
        );
        assert!(!err.to_string().contains("ci-secret"));

        // Without a token any https engine is fine.
        config.token_override = None;
        let moved = config.with_url("https://evil.example/mcp").unwrap();
        assert_eq!(moved.origin(), "https://evil.example");
        assert!(moved.with_url("http://evil.example/mcp").is_err());
    }

    #[test]
    fn debug_output_redacts_the_token() {
        let mut config = EngineConfig::new(DEFAULT_ENGINE_URL, "/tmp").unwrap();
        config.token_override = Some("ci-secret".into());
        let debug = format!("{config:?}");
        assert!(
            !debug.contains("ci-secret") && debug.contains("[redacted]"),
            "{debug}"
        );
    }

    #[test]
    fn origin_and_document_ids() {
        let mut config = EngineConfig::new(DEFAULT_ENGINE_URL, "/tmp").unwrap();
        assert_eq!(config.origin(), "https://creator.mcp.lottiefiles.com");
        assert_eq!(config.document_ids().len(), DEFAULT_DOCUMENTS);
        config.documents = 9;
        assert_eq!(
            config.document_ids().last().unwrap(),
            "gen-animated-icons-export-4"
        );
    }
}
