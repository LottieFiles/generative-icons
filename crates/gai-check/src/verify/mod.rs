//! Set verification (verify.md): the source-side gate that runs after a set is built and
//! packaged. It proves that the shipped assets are the ones the recorded sources produced,
//! that packaging did not change motion, and that the exported world geometry obeys the
//! generic and drawing-specific contracts, then writes the evidence.
//!
//! One call replaces a set's `verify.mjs` and every stage it chains:
//!
//! 1. the AI scope gate of `run-ai.mjs` for the sets the kit's scope lists, and set-level
//!    asserts (icon counts, category) from the set's rules;
//! 2. the generic checks of the set's frozen variant ([`Generic`]): the template for new
//!    sets, `verify-directed.mjs` for 03–11 and 13–15, the 12 continuous variant, or the
//!    fixed 01/02 checks;
//! 3. drawing rules ([`rules`]) and clearance ([`crate::contracts`]) placed before or after
//!    the `directed-source-verification.json` write, exactly as the legacy scripts did;
//! 4. the fixed one-offs of 10 and 11 ([`OneOff`]).
//!
//! Every evidence file is `JSON.stringify(report, null, 2) + "\n"` and byte-identical to
//! the JS output. A failure keeps whatever evidence the JS would already have written.
//!
//! ```no_run
//! # use gai_check::verify::{SetPaths, VerifyOptions, verify_set};
//! let paths = SetPaths::catalog("motion/icon-sets/35-weather");
//! let outcome = verify_set(&paths, &VerifyOptions::default());
//! outcome.write_evidence(&paths.set_dir)?;
//! std::process::exit(outcome.exit_code());
//! # Ok::<(), gai_check::verify::VerifyError>(())
//! ```

pub mod adopted;
mod generic;
pub mod geom;
pub mod js;
mod legacy;
mod manifest;
mod oneoff;
pub mod profile;
pub mod rules;
mod scope;

pub use generic::Generic;
pub use oneoff::OneOff;
pub use profile::{Profile, Stage};
pub use scope::SCOPE_FILE;

use manifest::SetManifest;

use crate::contracts::{self, Clearance, ContractError, Scene, SceneSource};
use serde_json::Value;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Where the directed report lives, relative to the set folder.
pub const DIRECTED_FILE: &str = "evidence/directed-source-verification.json";

/// Verification failures. Everything but [`VerifyError::Io`] and [`VerifyError::Rules`]
/// is a failed check (exit 1); those two are exit 2.
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    /// A file could not be read or written.
    #[error("cannot read {}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error,
    },
    /// A file is not valid JSON of the expected shape.
    #[error("cannot parse {}: {source}", path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: serde_json::Error,
    },
    /// A check failed (`assert.*` in the legacy scripts); the message names it.
    #[error("{0}")]
    Assertion(String),
    /// Scene geometry the checks cannot evaluate (an unknown command, a cubic before a move).
    #[error("{0}")]
    Geometry(String),
    /// Collected drawing-rule failures; their evidence was written with `result: "FAIL"`.
    #[error("{} failed checks; first: {}", errors.len(), errors.first().map_or("", String::as_str))]
    Collected {
        /// Every failure message, in check order.
        errors: Vec<String>,
    },
    /// A rules document is malformed.
    #[error("invalid verify rules: {0}")]
    Rules(String),
    /// The clearance stage failed or aborted.
    #[error(transparent)]
    Clearance(#[from] ContractError),
}

impl VerifyError {
    /// The process exit code for this failure: 2 for I/O and malformed rules, 1 otherwise.
    pub fn exit_code(&self) -> i32 {
        match self {
            VerifyError::Io { .. } | VerifyError::Rules(_) => 2,
            VerifyError::Clearance(ContractError::Io { .. } | ContractError::Rules(_)) => 2,
            _ => 1,
        }
    }
}

/// Folders one set's verification reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPaths {
    /// The source set, `motion/icon-sets/NN-slug`: provenance, scenes, native exports,
    /// definitions and evidence.
    pub set_dir: PathBuf,
    /// The packaged assets, `prototype/icons/NN/assets`: `set.json`, actions, themes, `.lottie`.
    pub assets_dir: PathBuf,
    /// The shared kit, `motion/icon-sets/library`: `build-set.mjs` and direction sources.
    pub library_dir: PathBuf,
}

impl SetPaths {
    /// The catalog layout around a set folder: `<set>/../library` and
    /// `<set>/../../../prototype/icons/<first two characters of the folder>/assets`.
    pub fn catalog(set_dir: impl Into<PathBuf>) -> SetPaths {
        let set_dir = set_dir.into();
        let folder = set_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let number: String = folder.chars().take(2).collect();
        SetPaths {
            assets_dir: set_dir
                .join("../../../prototype/icons")
                .join(number)
                .join("assets"),
            library_dir: set_dir.join("../library"),
            set_dir,
        }
    }
}

/// Options for [`verify_set`].
#[derive(Debug, Clone, Default)]
pub struct VerifyOptions {
    /// The sha256 recorded as `sourceSha256` and builder hash by sets built without a
    /// `build.mjs` (gai-build's builder identity). Used whenever `build.mjs` or
    /// `library/build-set.mjs` does not exist; without it a missing builder is an I/O error.
    pub builder_identity_sha256: Option<String>,
}

/// One evidence file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// Path relative to the set folder, `/`-separated.
    pub path: String,
    /// The exact file contents.
    pub contents: String,
}

/// The outcome of verifying one set.
#[derive(Debug)]
pub struct Verification {
    /// `set.json` `id`, or the folder name when the manifest could not be read.
    pub set_id: String,
    /// Evidence in write order: everything the legacy scripts would have written, including
    /// FAIL evidence and files written before a later stage failed.
    pub evidence: Vec<Evidence>,
    /// The one-line PASS summary of each completed stage.
    pub summary: Vec<String>,
    /// The failure, if any.
    pub error: Option<VerifyError>,
}

impl Verification {
    /// Whether every stage passed.
    pub fn passed(&self) -> bool {
        self.error.is_none()
    }

    /// 0 on PASS, otherwise [`VerifyError::exit_code`].
    pub fn exit_code(&self) -> i32 {
        self.error.as_ref().map_or(0, VerifyError::exit_code)
    }

    /// Writes the evidence under `root` (normally the set folder), each file atomically.
    pub fn write_evidence(&self, root: &Path) -> Result<(), VerifyError> {
        for e in &self.evidence {
            write_atomic(&root.join(&e.path), e.contents.as_bytes())?;
        }
        Ok(())
    }
}

/// Writes `bytes` to a sibling temp file, then renames it over `path`.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), VerifyError> {
    let io = |source| VerifyError::Io {
        path: path.to_path_buf(),
        source,
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&temp, bytes).map_err(io)?;
    std::fs::rename(&temp, path).map_err(|source| {
        let _ = std::fs::remove_file(&temp);
        io(source)
    })
}

/// `JSON.stringify(value, null, 2) + "\n"`.
pub fn evidence_json(value: &Value) -> String {
    contracts::evidence::evidence_json(value)
}

/// Reads a file.
pub(crate) fn read(path: &Path) -> Result<Vec<u8>, VerifyError> {
    std::fs::read(path).map_err(|source| VerifyError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads and parses a JSON file.
pub(crate) fn read_json(path: &Path) -> Result<Value, VerifyError> {
    serde_json::from_slice(&read(path)?).map_err(|source| VerifyError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// `assert.ok(ok, message)`.
pub(crate) fn ensure(ok: bool, message: impl FnOnce() -> String) -> Result<(), VerifyError> {
    if ok {
        Ok(())
    } else {
        Err(VerifyError::Assertion(message()))
    }
}

/// Everything one set's stages share: paths, the manifest, and caches of file hashes and
/// parsed scenes (pure; they never change a result).
pub(crate) struct SetContext<'a> {
    /// The folders.
    pub paths: &'a SetPaths,
    /// The parsed `set.json`.
    pub manifest: SetManifest,
    options: &'a VerifyOptions,
    hashes: Mutex<HashMap<PathBuf, String>>,
    scenes: Mutex<HashMap<String, Arc<Scene>>>,
    /// Actions of icons adopted from spec v2: they have no scenes, so a stage that names
    /// one must skip it ([`SetContext::is_adopted`]) rather than read a stale scene.
    adopted: std::collections::HashSet<String>,
}

impl<'a> SetContext<'a> {
    fn new(paths: &'a SetPaths, manifest: SetManifest, options: &'a VerifyOptions) -> Self {
        SetContext {
            paths,
            manifest,
            options,
            hashes: Mutex::default(),
            scenes: Mutex::default(),
            adopted: Default::default(),
        }
    }

    /// Whether `name` is an action of an icon adopted from spec v2.
    pub fn is_adopted(&self, name: &str) -> bool {
        self.adopted.contains(name)
    }

    /// sha256 of a file's raw bytes.
    pub fn hash(&self, path: &Path) -> Result<String, VerifyError> {
        if let Some(h) = self
            .hashes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(path)
        {
            return Ok(h.clone());
        }
        let h = gai_core::sha256_hex(read(path)?);
        self.hashes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(path.to_path_buf(), h.clone());
        Ok(h)
    }

    /// sha256 of a builder file, or the builder identity once the file is gone.
    pub fn builder_hash(&self, path: &Path) -> Result<String, VerifyError> {
        match (self.hash(path), &self.options.builder_identity_sha256) {
            (Err(VerifyError::Io { source, .. }), Some(identity))
                if source.kind() == ErrorKind::NotFound =>
            {
                Ok(identity.clone())
            }
            (result, _) => result,
        }
    }

    /// `SET/scenes/<name>.scene.json`.
    pub fn scene_path(&self, name: &str) -> PathBuf {
        self.paths
            .set_dir
            .join("scenes")
            .join(format!("{name}.scene.json"))
    }

    /// Parses scene bytes and caches the result under the action name.
    pub fn cache_scene(&self, name: &str, bytes: &[u8]) -> Result<Arc<Scene>, VerifyError> {
        let scene = Arc::new(
            Scene::from_slice(bytes).map_err(|source| VerifyError::Json {
                path: self.scene_path(name),
                source,
            })?,
        );
        self.scenes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(name.to_string(), scene.clone());
        Ok(scene)
    }

    /// `readScene(name)`: the parsed scene of an action.
    pub fn scene(&self, name: &str) -> Result<Arc<Scene>, VerifyError> {
        ensure(!self.is_adopted(name), || {
            format!("{name} is adopted from a spec and has no scene; the stage that reads it must skip adopted icons")
        })?;
        if let Some(s) = self
            .scenes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(name)
        {
            return Ok(s.clone());
        }
        let bytes = read(&self.scene_path(name))?;
        self.cache_scene(name, &bytes)
    }
}

impl SceneSource for SetContext<'_> {
    fn load(&self, name: &str) -> Result<Scene, ContractError> {
        self.scene(name).map(|s| (*s).clone()).map_err(|e| match e {
            VerifyError::Io { path, source } => ContractError::Io { path, source },
            VerifyError::Json { path, source } => ContractError::Json { path, source },
            other => ContractError::Geometry(other.to_string()),
        })
    }
}

/// Verifies one set: every stage of its profile, in the legacy order, collecting evidence.
pub fn verify_set(paths: &SetPaths, options: &VerifyOptions) -> Verification {
    let folder = paths
        .set_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut run = Verification {
        set_id: folder,
        evidence: Vec::new(),
        summary: Vec::new(),
        error: None,
    };
    if let Err(error) = run_stages(paths, options, &mut run) {
        run.error = Some(error);
    }
    run
}

fn run_stages(
    paths: &SetPaths,
    options: &VerifyOptions,
    run: &mut Verification,
) -> Result<(), VerifyError> {
    let full = SetManifest::read(&paths.assets_dir.join("set.json"))?;
    run.set_id = full.id().to_string();
    let profile = Profile::for_set(&run.set_id, &paths.set_dir)?;
    // Icons adopted from spec v2 have no scenes or legacy natives: the legacy stages
    // verify the set without them, and the adopted stage checks what they ship.
    let (manifest, adopted) = adopted::split(&full);
    let mut set = SetContext::new(paths, manifest, options);
    set.adopted = adopted
        .iter()
        .flat_map(|icon| icon.get("animations").and_then(Value::as_array).into_iter().flatten())
        .filter_map(|a| a.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    scope::check(&set)?;
    if let Some(tail) = &profile.rules {
        tail.check_set(&full)?;
    }
    let Some(mut directed) = generic::run(&set, &profile.generic)? else {
        ensure(adopted.is_empty(), || {
            format!("{}: a frozen set cannot hold adopted spec icons", run.set_id)
        })?;
        // 01/02 write no evidence.
        run.summary
            .push(format!("PASS {}: frozen legacy checks", run.set_id));
        return Ok(());
    };
    if !adopted.is_empty() {
        let checked = adopted::check(&set, &adopted)?;
        directed.icons += adopted.len();
        directed.actions += adopted
            .iter()
            .map(|icon| icon.get("animations").and_then(Value::as_array).map_or(0, Vec::len))
            .sum::<usize>();
        directed.extra.push(("adoptedSpecs".into(), checked));
        run.summary
            .push(format!("PASS {}: {} adopted spec icons", run.set_id, adopted.len()));
    }

    // Stages placed before the directed write: a failure there leaves it unwritten.
    let mut deferred = None;
    if profile.clearance == Stage::BeforeDirected {
        clearance(&set, run)?;
    }
    if let Some(tail) = profile
        .rules
        .as_ref()
        .filter(|t| t.stage() == Some(Stage::BeforeDirected))
    {
        let outcome = tail.run(&set)?;
        outcome.embed_into(&mut directed);
        run.evidence.extend(outcome.file());
        deferred = outcome.failure();
        if deferred.is_none() {
            run.summary
                .push(format!("PASS {} drawing rules", run.set_id));
        }
    }
    run.evidence.push(Evidence {
        path: DIRECTED_FILE.into(),
        contents: evidence_json(&directed.to_value()),
    });
    run.summary.push(directed.summary(&run.set_id));
    if let Some(error) = deferred {
        return Err(error);
    }

    // Stages after the directed write.
    if let Some(tail) = profile
        .rules
        .as_ref()
        .filter(|t| t.stage() == Some(Stage::AfterDirected))
    {
        let outcome = tail.run(&set)?;
        run.evidence.extend(outcome.file());
        if let Some(error) = outcome.failure() {
            return Err(error);
        }
        run.summary
            .push(format!("PASS {} drawing rules", run.set_id));
    }
    if profile.clearance == Stage::AfterDirected {
        clearance(&set, run)?;
    }
    for one_off in &profile.one_offs {
        let (evidence, summary) = one_off.run(&set)?;
        run.evidence.push(evidence);
        run.summary.push(summary);
    }
    Ok(())
}

/// The clearance stage (`verify-clearance.mjs`): collect-mode programs write FAIL
/// evidence and then fail; assert-mode programs write nothing on failure.
fn clearance(set: &SetContext<'_>, run: &mut Verification) -> Result<(), VerifyError> {
    let Some(program) = Clearance::for_set(&set.paths.set_dir, set.manifest.id())? else {
        return Ok(());
    };
    let manifest: contracts::SetManifest = serde_json::from_value(set.manifest.doc.clone())
        .map_err(|source| VerifyError::Json {
            path: set.paths.assets_dir.join("set.json"),
            source,
        })?;
    let report = program.run(&manifest, set)?;
    run.evidence.push(Evidence {
        path: contracts::EVIDENCE_FILE.into(),
        contents: report.to_json(),
    });
    if report.passed() {
        run.summary
            .push(format!("PASS {} clearance", set.manifest.id()));
        Ok(())
    } else {
        Err(VerifyError::Collected {
            errors: report.errors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_paths_follow_the_repo_layout() {
        let p = SetPaths::catalog("/r/motion/icon-sets/35-weather");
        assert_eq!(
            p.assets_dir,
            Path::new("/r/motion/icon-sets/35-weather/../../../prototype/icons/35/assets")
        );
        assert_eq!(
            p.library_dir,
            Path::new("/r/motion/icon-sets/35-weather/../library")
        );
    }

    #[test]
    fn exit_codes() {
        let io = VerifyError::Io {
            path: "x".into(),
            source: std::io::Error::other("x"),
        };
        assert_eq!(io.exit_code(), 2);
        assert_eq!(VerifyError::Assertion("x".into()).exit_code(), 1);
        assert_eq!(
            VerifyError::Collected {
                errors: vec!["a".into()]
            }
            .exit_code(),
            1
        );
    }

    #[test]
    fn atomic_writes_create_folders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evidence/x.json");
        write_atomic(&path, b"{}\n").unwrap();
        write_atomic(&path, b"[]\n").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"[]\n");
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }
}
