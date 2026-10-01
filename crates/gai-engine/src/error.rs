//! Errors from the engine client.

/// Result type of this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong talking to the hosted engine.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// No credentials are stored for this engine.
    #[error("not signed in to {engine}")]
    NotLoggedIn { engine: String },
    /// The stored session can no longer be used (refresh rejected, client expired, token revoked).
    #[error("the engine session has ended ({0})")]
    SessionExpired(String),
    /// Sign-in did not complete: the user refused, closed the page or the wait timed out.
    #[error("sign-in did not complete: {0}")]
    LoginCancelled(String),
    /// An OAuth endpoint answered with an error response (RFC 6749 §5.2).
    #[error("{endpoint} refused the request: {error}{}", description.as_deref().map(|d| format!(" ({d})")).unwrap_or_default())]
    OAuth {
        endpoint: String,
        error: String,
        description: Option<String>,
    },
    /// The engine kept answering 429 after every retry.
    #[error("the engine is rate limiting requests; try again in a minute")]
    RateLimited,
    /// The engine kept answering 5xx after every retry.
    #[error("the engine is unavailable (HTTP {status})")]
    Unavailable { status: u16 },
    /// An HTTP status the client does not handle.
    #[error("unexpected HTTP {status} from {url}: {body}")]
    Http {
        status: u16,
        url: String,
        body: String,
    },
    /// Connection, TLS or timeout failure. The URL it names has no query (signed URLs carry
    /// their credential there).
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),
    /// The server's answer did not follow MCP or OAuth.
    #[error("unexpected engine response: {0}")]
    Protocol(String),
    /// A JSON-RPC error object.
    #[error("{method} failed with JSON-RPC error {code}: {message}")]
    Rpc {
        method: String,
        code: i64,
        message: String,
    },
    /// A tool call returned `isError` (for `creator_exec`, the script threw or the engine failed).
    #[error("{tool} failed ({code}): {message}")]
    Tool {
        tool: String,
        code: String,
        message: String,
        retryable: bool,
    },
    /// The scene handed to the exporter is not JSON.
    #[error("invalid scene: {0}")]
    InvalidScene(#[source] serde_json::Error),
    /// The exported Lottie failed validation or normalization.
    #[error("invalid export: {0}")]
    InvalidExport(String),
    /// The keychain or credentials file failed.
    #[error("credential store: {0}")]
    Store(String),
    /// Bad engine URL or settings.
    #[error("invalid engine configuration: {0}")]
    Config(String),
    /// Local I/O (credentials file, lock file, loopback socket, stdin).
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        let redacted = error.url().map(|url| {
            let mut url = url.clone();
            url.set_query(None);
            url.set_fragment(None);
            url
        });
        Error::Network(match redacted {
            Some(url) => error.with_url(url),
            None => error,
        })
    }
}

/// What kind of failure an [`Error`] is. Exhaustive on purpose: a new kind does not compile
/// until every caller that maps kinds handles it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NotLoggedIn,
    SessionExpired,
    LoginCancelled,
    /// An OAuth endpoint refused a request.
    Auth,
    RateLimited,
    /// The engine kept answering 5xx.
    Unavailable,
    /// A request or an engine script timed out.
    Timeout,
    Network,
    /// The engine failed or answered outside the protocol.
    Engine,
    /// The scene handed to the exporter is not JSON.
    InvalidScene,
    CredentialStore,
    Io,
    InvalidConfig,
}

impl ErrorKind {
    /// The stable error code the CLI reports (its exit codes 4 and 5 group these).
    pub fn code(self) -> &'static str {
        match self {
            ErrorKind::NotLoggedIn => "NOT_LOGGED_IN",
            ErrorKind::SessionExpired => "SESSION_EXPIRED",
            ErrorKind::LoginCancelled => "LOGIN_CANCELLED",
            ErrorKind::Auth => "AUTH_ERROR",
            ErrorKind::RateLimited => "RATE_LIMITED",
            ErrorKind::Unavailable => "ENGINE_UNAVAILABLE",
            ErrorKind::Timeout => "ENGINE_TIMEOUT",
            ErrorKind::Network => "NETWORK",
            ErrorKind::Engine => "ENGINE_ERROR",
            ErrorKind::InvalidScene => "INVALID_SCENE",
            ErrorKind::CredentialStore => "CREDENTIAL_STORE",
            ErrorKind::Io => "IO",
            ErrorKind::InvalidConfig => "INVALID_CONFIG",
        }
    }
}

impl Error {
    /// The kind of failure.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::NotLoggedIn { .. } => ErrorKind::NotLoggedIn,
            Error::SessionExpired(_) => ErrorKind::SessionExpired,
            Error::LoginCancelled(_) => ErrorKind::LoginCancelled,
            Error::OAuth { .. } => ErrorKind::Auth,
            Error::RateLimited => ErrorKind::RateLimited,
            Error::Unavailable { .. } => ErrorKind::Unavailable,
            Error::Network(e) if e.is_timeout() => ErrorKind::Timeout,
            Error::Network(_) => ErrorKind::Network,
            Error::Tool { code, .. } if code.contains("timeout") => ErrorKind::Timeout,
            Error::Tool { .. } | Error::Rpc { .. } => ErrorKind::Engine,
            Error::Http { .. } | Error::Protocol(_) | Error::InvalidExport(_) => ErrorKind::Engine,
            Error::InvalidScene(_) => ErrorKind::InvalidScene,
            Error::Store(_) => ErrorKind::CredentialStore,
            Error::Io(_) => ErrorKind::Io,
            Error::Config(_) => ErrorKind::InvalidConfig,
        }
    }

    /// The stable error code the CLI reports; see [`ErrorKind::code`].
    pub fn code(&self) -> &'static str {
        self.kind().code()
    }

    /// Whether the user has to sign in (again) to get past this error.
    pub fn needs_login(&self) -> bool {
        matches!(self, Error::NotLoggedIn { .. } | Error::SessionExpired(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_errors_drop_the_url_query() {
        // Nothing listens on port 1, so the request fails before any response.
        let client = crate::http::client().unwrap();
        let sent = client
            .get("http://127.0.0.1:1/r2/doc/a.json?X-Amz-Signature=secret#frag")
            .send()
            .unwrap_err();
        assert!(
            sent.to_string().contains("secret"),
            "reqwest names the full URL"
        );
        let error = Error::from(sent);
        let shown = format!("{error} {error:?}");
        assert!(
            !shown.contains("secret") && !shown.contains("frag"),
            "{shown}"
        );
        assert!(shown.contains("/r2/doc/a.json"), "{shown}");
        assert_eq!(error.kind(), ErrorKind::Network);
    }

    #[test]
    fn messages_name_no_cli_commands() {
        let not_logged_in = Error::NotLoggedIn {
            engine: "https://e.test/mcp".into(),
        };
        assert_eq!(
            not_logged_in.to_string(),
            "not signed in to https://e.test/mcp"
        );
        assert_eq!(not_logged_in.code(), "NOT_LOGGED_IN");
        let expired = Error::SessionExpired("revoked".into());
        assert!(!expired.to_string().contains("gen-animated-icons"));
        assert_eq!(expired.kind(), ErrorKind::SessionExpired);
        let scene = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        assert_eq!(Error::InvalidScene(scene).code(), "INVALID_SCENE");
    }
}
