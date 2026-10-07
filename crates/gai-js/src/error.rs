use std::{fmt, path::PathBuf, time::Duration};

/// Where in a set an error happened. Each field is known only once the runtime reaches it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Location {
    pub icon: Option<String>,
    pub variant: Option<String>,
    pub part: Option<String>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = [("icon", &self.icon), ("variant", &self.variant), ("part", &self.part)];
        let mut first = true;
        for (label, value) in fields {
            if let Some(value) = value {
                f.write_str(if first { " (" } else { ", " })?;
                write!(f, "{label} {value}")?;
                first = false;
            }
        }
        if !first {
            f.write_str(")")?;
        }
        Ok(())
    }
}

/// Why an import was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModuleError {
    #[error("only relative imports of .mjs files and the builtin 'svgpath' are supported")]
    Unsupported,
    #[error("builtin modules cannot import relative paths")]
    FromBuiltin,
    #[error("{} does not exist", .0.display())]
    NotFound(PathBuf),
    #[error("{} is not an .mjs file", .0.display())]
    NotModule(PathBuf),
    #[error("{} is outside the set and library folders", .0.display())]
    OutsideRoots(PathBuf),
    #[error("{} is a symlink to {}, outside the set and library folders", .path.display(), .target.display())]
    Symlink { path: PathBuf, target: PathBuf },
    #[error("{} starts with a byte order mark", .0.display())]
    ByteOrderMark(PathBuf),
    #[error("{} is not UTF-8", .0.display())]
    NotUtf8(PathBuf),
    #[error("cannot read {}: {message}", .path.display())]
    Read { path: PathBuf, message: String },
}

/// Errors from loading and running a set's authoring code.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Authoring code, or the host on its behalf, threw.
    #[error("{set}{location}: {message}")]
    Script {
        set: String,
        location: Box<Location>,
        message: String,
        /// The QuickJS stack trace, when the thrown value is an Error.
        stack: Option<String>,
    },
    /// A call ran past [`Limits::call_timeout`](crate::Limits::call_timeout).
    #[error("{set}{location}: stopped after {limit:?}")]
    Timeout {
        set: String,
        location: Box<Location>,
        limit: Duration,
        /// The QuickJS stack where the interrupt stopped the call, when it could take one
        /// (not when the watchdog ended it).
        stack: Option<String>,
    },
    /// The set's calls together ran past [`Limits::budget`](crate::Limits::budget).
    #[error("{set}{location}: stopped when the set's calls had used their {limit:?} budget")]
    Budget {
        set: String,
        location: Box<Location>,
        limit: Duration,
        /// As for [`Error::Timeout`]; `None` when no call was running.
        stack: Option<String>,
    },
    /// An import could not be resolved or read.
    #[error("{set}: cannot import '{specifier}' from '{importer}': {reason}")]
    Module { set: String, specifier: String, importer: String, reason: Box<ModuleError> },
    /// A folder whose name does not start with a set number.
    #[error("{} is not a set folder named NN-slug", .0.display())]
    NotASet(PathBuf),
    /// The engine failed outside author code, or the host returned data of the wrong shape.
    #[error("{set}: {message}")]
    Engine { set: String, message: String },
    /// The CPU cannot run the V8-exact Math. The message names the remedy.
    #[error("{}", unsupported_cpu(rosetta()))]
    UnsupportedCpu,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why an x86_64 process cannot run the V8-exact Math, and what to do about it.
fn unsupported_cpu(rosetta: bool) -> &'static str {
    if rosetta {
        "this x86_64 build of generative-icons runs under Rosetta, which lacks the FMA instructions its \
         V8-exact Math needs. Use the arm64 build: reinstall with arm64 Node (`node -p process.arch` prints \
         arm64), or set GEN_ANIMATED_ICONS_BINARY to the darwin-arm64 binary"
    } else {
        "this x86_64 CPU lacks FMA, which generative-icons' V8-exact Math needs. Run it on an x86_64 CPU \
         with FMA (Intel Haswell, AMD Piledriver or later) or on arm64"
    }
}

/// Whether this process is an x86_64 binary translated by Rosetta (`sysctl.proc_translated`).
#[cfg(target_os = "macos")]
pub(crate) fn rosetta() -> bool {
    use std::ffi::{c_char, c_int, c_void};
    unsafe extern "C" {
        fn sysctlbyname(
            name: *const c_char,
            oldp: *mut c_void,
            oldlenp: *mut usize,
            newp: *mut c_void,
            newlen: usize,
        ) -> c_int;
    }
    let mut value: c_int = 0;
    let mut size = size_of::<c_int>();
    // SAFETY: `value` and `size` describe a writable c_int; the name is NUL-terminated.
    let status = unsafe {
        sysctlbyname(c"sysctl.proc_translated".as_ptr(), (&raw mut value).cast(), &mut size, std::ptr::null_mut(), 0)
    };
    status == 0 && value == 1
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn rosetta() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_missing_fma_error_names_a_remedy() {
        assert!(unsupported_cpu(true).contains("Use the arm64 build"));
        assert!(unsupported_cpu(true).contains("GEN_ANIMATED_ICONS_BINARY"));
        assert!(unsupported_cpu(false).contains("CPU with FMA"));
        #[cfg(target_arch = "aarch64")]
        assert!(!rosetta(), "a native arm64 process is not translated");
        assert_eq!(Error::UnsupportedCpu.to_string(), unsupported_cpu(rosetta()));
    }
}
