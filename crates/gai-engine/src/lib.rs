//! Client for LottieFiles' hosted Creator MCP engine: OAuth sign-in, token storage,
//! scene export to Lottie and file downloads. Implements [`gai_core::export::Exporter`].
//!
//! - **Sign-in** ([`login`], [`logout`], [`status`], [`check_session`], [`whoami`]): MCP OAuth
//!   with discovery, dynamic client registration, PKCE S256 and a loopback redirect on
//!   `127.0.0.1`, or a pasted redirect URL on machines whose browser cannot reach the CLI. The
//!   engine has no device flow and no API keys; [`config::ENGINE_TOKEN_ENV`] passes a bearer
//!   token through for CI.
//! - **Tokens**: access tokens last five minutes; refresh tokens rotate on every use and
//!   replaying one revokes the whole chain, so refreshes run single-flight under a
//!   cross-process file lock. The refresh token lives in the OS keychain when there is one.
//! - **Engine calls** ([`McpClient`]): JSON-RPC over streamable HTTP with the session header,
//!   a token refresh on 401 and backoff on 429 and 5xx.
//! - **Export** ([`HostedExporter`]): runs [`export::EXPORT_SCRIPT`] in `creator_exec`,
//!   downloads the file and [normalizes](normalize_native) it to the committed native format.
//! - **Spec v2** ([`PlanRunner`]): one `creator_exec` per icon runs a compiled plan, for a
//!   draft sheet (attached images, nothing stored) or one bundle export split per motion.
//! - **Public endpoints** ([`Downloader`]): the same TLS stack and trust roots for the library
//!   and review submissions.
//!
//! Every call blocks. From async code, call it inside `spawn_blocking`.

pub mod auth;
pub mod config;
pub mod error;
pub mod export;
mod http;
mod loopback;
pub mod mcp;
pub mod normalize;
mod oauth;
pub mod plan;
mod store;

pub use auth::{
    LoginInteraction, LoginMode, LoginOptions, LogoutOutcome, SessionCheck, SessionState, Status,
    StdioInteraction, Whoami, check_session, login, logout, status, whoami,
};
pub use config::{CredentialStoreKind, EngineConfig, LockWait, Notice, NoticeSink, RetryPolicy};
pub use error::{Error, ErrorKind, Result};
pub use export::HostedExporter;
pub use http::Downloader;
pub use mcp::{McpClient, ServerInfo, ToolImage, ToolOutput};
pub use normalize::{NativeLottie, normalize_native};
pub use plan::{Bundle, BundleAnimation, PlanRunner, Run, animations_from_bundle, split_bundle};
