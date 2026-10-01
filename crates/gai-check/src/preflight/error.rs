use std::path::PathBuf;

/// Why preflight could not produce a report. Each of these aborts the whole run, as a throw
/// out of `checkSet` did in `preflight.mjs`: nothing is printed on stdout.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The folder has no `definitions.mjs` (sets 01 and 02, or a mistyped path). `0` is the
    /// folder as given.
    #[error("No separate definitions.mjs in {0}; legacy inline builders require their existing checks")]
    Legacy(String),
    /// Evaluating the definitions or an author callback failed outside what preflight reports:
    /// a module error, a throw at module top level, a timeout.
    #[error(transparent)]
    Js(#[from] gai_js::Error),
    /// A callback threw `null` or `undefined`. `preflight.mjs` crashed reading `.message` of it.
    #[error("Cannot read properties of null or undefined (reading 'message'): {set}: {stage} of part {part} threw it")]
    ThrewNullish { set: String, part: String, stage: &'static str },
    /// The probe returned a sample that contradicts its own outcome.
    #[error("{set}: malformed probe sample for part {part}")]
    Probe { set: String, part: String },
    /// A file or folder could not be read.
    #[error("cannot read {}: {source}", .path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// The contract, catalog or a `set.json` is not JSON.
    #[error("{} is not valid JSON: {source}", .path.display())]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// A contract or catalog file has the wrong shape.
    #[error("{}: {message}", .path.display())]
    Invalid { path: PathBuf, message: String },
    /// Two catalog icons share an ID (`generation-context.mjs:37`).
    #[error("Duplicate catalog ID {0}")]
    DuplicateCatalogId(String),
}

/// A preflight result.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Error {
        let path = path.into();
        move |source| Error::Io { path, source }
    }
}
