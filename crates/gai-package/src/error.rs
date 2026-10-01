use crate::zip::ZipError;
use std::path::{Path, PathBuf};

/// Everything that can stop a packaging step.
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    /// A file could not be read, written or listed.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file or folder.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A JSON input did not parse.
    #[error("{}: invalid JSON: {source}", path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The parse error.
        #[source]
        source: serde_json::Error,
    },
    /// An archive could not be written or failed its integrity check.
    #[error("{}: {source}", path.display())]
    Zip {
        /// The archive being written or checked.
        path: PathBuf,
        /// The archive error.
        #[source]
        source: ZipError,
    },
    /// An input broke a packaging rule (missing key, stale hash, invalid id, …).
    #[error("{0}")]
    Invalid(String),
    /// `pack_release` refused a build from, or a tree with, uncommitted changes.
    #[error(
        "Refusing to pack: the packages were built from, or the tree now has, uncommitted changes. Commit, run npm run build:packages, then pack again. Use --allow-dirty only for local testing; the result is status prerelease-dev and must not be published."
    )]
    DirtyRelease,
    /// An external command (`git`, `npm pack`) failed.
    #[error("{command}: {detail}")]
    Command {
        /// The command line.
        command: String,
        /// Its error output, or why it could not run.
        detail: String,
    },
}

/// Result of a packaging step.
pub type Result<T> = std::result::Result<T, PackageError>;

impl PackageError {
    pub(crate) fn io(path: &Path, source: std::io::Error) -> Self {
        PackageError::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn zip(path: &Path, source: ZipError) -> Self {
        PackageError::Zip {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Shorthand for [`PackageError::Invalid`].
pub(crate) fn invalid(message: impl Into<String>) -> PackageError {
    PackageError::Invalid(message.into())
}
