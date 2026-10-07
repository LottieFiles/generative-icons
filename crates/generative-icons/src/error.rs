//! Structured errors: a stable code, a human message, and the process exit status.

use serde_json::{Map, Value, json};
use std::fmt;
use std::path::Path;

/// Stable, machine-readable error codes. Agents branch on these, so never rename one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Invalid arguments.
    Usage,
    /// The subcommand does not exist.
    UnknownCommand,
    /// An option was given without its value.
    MissingValue,
    /// No supported app project in the selected folder.
    ProjectNotFound,
    /// The folder is a JavaScript workspace root, not an app.
    WorkspaceRoot,
    /// More than one platform matches; `--platform` decides.
    AmbiguousPlatform,
    /// A recognized project type this version does not support (React Native, Expo).
    UnsupportedPlatform,
    /// `generative-icons.json`, `package.json` or the lock is invalid.
    InvalidConfig,
    /// A managed path is not a plain project-relative path.
    InvalidPath,
    /// A managed path runs through a symlink.
    SymlinkPath,
    /// A managed file was edited outside this tool.
    ModifiedFile,
    /// A file changed while the command was writing.
    FileChanged,
    /// The icon id is not in the library or the project.
    UnknownIcon,
    /// The target already exists.
    Exists,
    /// A filesystem operation failed.
    Io,
    /// The network is unavailable or refused the request.
    Network,
    /// Downloaded or packaged bytes do not match their recorded sha256.
    HashMismatch,
    /// A `.lottie` archive cannot be read.
    CorruptArchive,
    /// The library catalog is missing or malformed.
    InvalidCatalog,
    /// A reference icon's packaged bytes are stale or missing.
    StaleReference,
    /// A new local icon id is already a library icon.
    IdCollision,
    /// A library icon whose source a project cannot start from (`create --from`).
    UnsupportedReference,
    /// A gate (preflight, verification, registration) failed.
    CheckFailed,
    /// Icon source threw, timed out or produced unusable geometry.
    SourceError,
    /// No engine sign-in is stored.
    NotLoggedIn,
    /// The engine session ended; sign in again.
    SessionExpired,
    /// Sign-in was refused, closed or timed out.
    LoginCancelled,
    /// The authorization server refused a request.
    AuthError,
    /// The engine kept answering 429.
    RateLimited,
    /// The engine kept answering 5xx.
    EngineUnavailable,
    /// An engine call or script timed out.
    EngineTimeout,
    /// The engine answered with an error or an unusable result.
    EngineError,
    /// The keychain or credentials file failed.
    CredentialStore,
    /// Another process held the sign-in lock too long (it is signing in, or waiting on a
    /// keychain prompt).
    SignInBusy,
    /// A bug in this tool, or a binary built wrongly.
    Internal,
}

impl ErrorCode {
    /// The `UPPER_SNAKE` name printed in JSON errors.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usage => "USAGE",
            Self::UnknownCommand => "UNKNOWN_COMMAND",
            Self::MissingValue => "MISSING_VALUE",
            Self::ProjectNotFound => "PROJECT_NOT_FOUND",
            Self::WorkspaceRoot => "WORKSPACE_ROOT",
            Self::AmbiguousPlatform => "AMBIGUOUS_PLATFORM",
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
            Self::InvalidConfig => "INVALID_CONFIG",
            Self::InvalidPath => "INVALID_PATH",
            Self::SymlinkPath => "SYMLINK_PATH",
            Self::ModifiedFile => "MODIFIED_FILE",
            Self::FileChanged => "FILE_CHANGED",
            Self::UnknownIcon => "UNKNOWN_ICON",
            Self::Exists => "EXISTS",
            Self::Io => "IO",
            Self::Network => "NETWORK",
            Self::HashMismatch => "HASH_MISMATCH",
            Self::CorruptArchive => "CORRUPT_ARCHIVE",
            Self::InvalidCatalog => "INVALID_CATALOG",
            Self::StaleReference => "STALE_REFERENCE",
            Self::IdCollision => "ID_COLLISION",
            Self::UnsupportedReference => "UNSUPPORTED_REFERENCE",
            Self::CheckFailed => "CHECK_FAILED",
            Self::SourceError => "SOURCE_ERROR",
            Self::NotLoggedIn => "NOT_LOGGED_IN",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::LoginCancelled => "LOGIN_CANCELLED",
            Self::AuthError => "AUTH_ERROR",
            Self::RateLimited => "RATE_LIMITED",
            Self::EngineUnavailable => "ENGINE_UNAVAILABLE",
            Self::EngineTimeout => "ENGINE_TIMEOUT",
            Self::EngineError => "ENGINE_ERROR",
            Self::CredentialStore => "CREDENTIAL_STORE",
            Self::SignInBusy => "SIGN_IN_BUSY",
            Self::Internal => "INTERNAL",
        }
    }

    /// Process exit status: 1 a gate failed, 2 usage, 3 project or state, 4 sign-in,
    /// 5 network or engine, 6 integrity, 70 internal.
    pub fn exit_code(self) -> u8 {
        match self {
            Self::CheckFailed | Self::SourceError => 1,
            Self::Usage | Self::UnknownCommand | Self::MissingValue => 2,
            Self::ProjectNotFound
            | Self::WorkspaceRoot
            | Self::AmbiguousPlatform
            | Self::UnsupportedPlatform
            | Self::InvalidConfig
            | Self::InvalidPath
            | Self::SymlinkPath
            | Self::ModifiedFile
            | Self::FileChanged
            | Self::UnknownIcon
            | Self::Exists
            | Self::IdCollision
            | Self::UnsupportedReference
            | Self::CredentialStore
            | Self::Io => 3,
            Self::NotLoggedIn | Self::SessionExpired | Self::LoginCancelled | Self::AuthError | Self::SignInBusy => 4,
            Self::Network | Self::RateLimited | Self::EngineUnavailable | Self::EngineTimeout | Self::EngineError => 5,
            Self::HashMismatch | Self::CorruptArchive | Self::InvalidCatalog | Self::StaleReference => 6,
            Self::Internal => 70,
        }
    }
}

/// A command failure with its code and optional context for agents.
#[derive(Debug, Clone, PartialEq)]
pub struct CliError {
    /// Stable code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
    /// The project-relative (or absolute) path involved, if any.
    pub path: Option<String>,
    /// The icon id involved, if any.
    pub id: Option<String>,
    /// What to do next.
    pub hint: Option<String>,
}

/// Result type of every command step.
pub type Result<T, E = CliError> = std::result::Result<T, E>;

impl CliError {
    /// An error with a code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), path: None, id: None, hint: None }
    }

    /// A usage error.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Usage, message)
    }

    /// A filesystem error on `path`, e.g. `Cannot read <path>: <reason>`.
    pub fn io(action: &str, path: &Path, error: &std::io::Error) -> Self {
        Self::new(ErrorCode::Io, format!("Cannot {action} {}: {error}", path.display()))
            .with_path(path.display().to_string())
    }

    /// Adds the path involved.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Adds the icon id involved.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Adds a next step.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// `{"error":{"code","message","path"?,"id"?,"hint"?}}`.
    pub fn to_json(&self) -> Value {
        let mut error = Map::new();
        error.insert("code".into(), json!(self.code.as_str()));
        error.insert("message".into(), json!(self.message));
        for (key, value) in [("path", &self.path), ("id", &self.id), ("hint", &self.hint)] {
            if let Some(value) = value {
                error.insert(key.into(), json!(value));
            }
        }
        json!({ "error": error })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_omits_absent_context() {
        let error = CliError::new(ErrorCode::UnknownIcon, "Unknown icon: x").with_id("x");
        assert_eq!(
            gai_core::to_js_json(&error.to_json()),
            r#"{"error":{"code":"UNKNOWN_ICON","message":"Unknown icon: x","id":"x"}}"#
        );
        assert_eq!(error.code.exit_code(), 3);
    }
}
