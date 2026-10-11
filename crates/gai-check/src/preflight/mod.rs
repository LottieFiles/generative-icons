//! Source preflight (`motion/icon-sets/library/preflight.mjs`) and the checked build
//! (`build-checked.mjs`).
//!
//! Preflight screens a set's trusted `definitions.mjs` before anything is exported. It
//! evaluates the definitions in gai-js (never installing them), checks names, labels, paints,
//! callbacks, Duotone and state policy and timing, then samples every part at every frame
//! and authored time for both profiles, both directions of stateful icons and each outline
//! weight: path validity (preflight's own `parseGeometry`, run in JS by `probe()`), topology
//! stability, visibility range, pose finiteness, rigid-pivot constancy and a control-hull
//! canvas margin. `checkSet` adds catalog ID collisions and a hashed input inventory. It
//! writes nothing.
//!
//! Reports serialize exactly as `JSON.stringify(report, null, 2)` did, so
//! [`Run::stdout`] is byte-identical to `node preflight.mjs --all`.
//!
//! ```no_run
//! # fn main() -> gai_check::preflight::Result<()> {
//! use gai_check::preflight::{Preflight, Status, Target};
//! let preflight = Preflight::open("/path/to/repo")?;
//! let run = preflight.run(&Target::All)?;
//! print!("{}", run.stdout());
//! std::process::exit(if run.status() == Status::Pass { 0 } else { 1 });
//! # }
//! ```

mod catalog;
mod check;
mod contract;
mod error;
mod inspect;
mod inventory;
mod paths;
mod report;

pub use catalog::{Catalog, CatalogIcon, Input};
pub use check::check_definitions;
pub use contract::FamilyContract;
pub use error::{Error, Result};
pub use report::{
    CheckedReport, Code, Detail, Diagnostic, LaunchOutcome, Report, Run, SCOPE, SKIP_REASON, SetReport, Severity,
    Status,
};

use gai_js::{Limits, ModernProfile, SetRuntime, SetSource};
use std::{
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

/// What to screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Every `motion/icon-sets/NN-*` folder; those without `definitions.mjs` are skipped.
    All,
    /// One set folder, resolved against the working directory like `path.resolve`. A
    /// folder without `definitions.mjs` is an error.
    Set(PathBuf),
}

/// Preflight for the library repo at `root`.
#[derive(Debug, Clone)]
pub struct Preflight {
    root: PathBuf,
    contract: FamilyContract,
    limits: Limits,
    jobs: usize,
}

/// A set's definitions as checked on a worker, before the catalog and inventory steps.
struct Screened {
    report: Report,
    /// `icon?.name` of each icon when a string; `None` when `icons` is not an array.
    names: Option<Vec<Option<String>>>,
}

impl Preflight {
    /// Reads the family contract of the repo at `root` (the folder holding
    /// `motion/icon-sets`). Relative roots are resolved against the working directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let cwd = std::env::current_dir().map_err(Error::io("."))?;
        let root = paths::resolve(&cwd, root.as_ref());
        let contract = FamilyContract::load(&root)?;
        let jobs = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        Ok(Preflight { root, contract, limits: Limits::default(), jobs })
    }

    /// Runtime limits for each set.
    pub fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }

    /// Sets screened at once by [`run`](Self::run) (default: the available parallelism).
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs.max(1);
        self
    }

    /// The repo root, absolute and folded.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The family contract read from the repo.
    pub fn contract(&self) -> &FamilyContract {
        &self.contract
    }

    /// `motion/icon-sets/library`: the kit, whose sources every report lists as inputs.
    pub fn library_dir(&self) -> PathBuf {
        paths::join(&self.root, "motion/icon-sets/library")
    }

    /// `checkSet(dir)`: the report of one set folder, on a worker thread sized for its runtime.
    pub fn check_set(&self, dir: &Path) -> Result<SetReport> {
        let dir = paths::normalize(dir);
        if !paths::join(&dir, "definitions.mjs").exists() {
            return Err(Error::Legacy(dir.to_string_lossy().into_owned()));
        }
        let screened = gai_js::run_on_worker(&self.limits, || self.screen(&dir))?;
        self.finish(&dir, screened, &Catalog::load(&self.root)?)
    }

    /// `preflight.mjs <dir> | --all`. The first error in set order aborts the run, as in JS;
    /// sets are screened in parallel.
    pub fn run(&self, target: &Target) -> Result<Run> {
        let (dirs, skipped) = match target {
            Target::Set(dir) => {
                let cwd = std::env::current_dir().map_err(Error::io("."))?;
                let dir = paths::resolve(&cwd, dir);
                if !paths::join(&dir, "definitions.mjs").exists() {
                    return Err(Error::Legacy(dir.to_string_lossy().into_owned()));
                }
                (vec![dir], Vec::new())
            }
            Target::All => self.library_sets()?,
        };
        let screened = self.screen_all(&dirs);
        let mut catalog = None;
        let mut reports = Vec::with_capacity(dirs.len());
        for (dir, screened) in dirs.iter().zip(screened) {
            let screened = screened?;
            let catalog = match catalog {
                Some(ref catalog) => catalog,
                None => catalog.insert(Catalog::load(&self.root)?),
            };
            reports.push(self.finish(dir, screened, catalog)?);
        }
        Ok(Run { reports, skipped })
    }

    /// `buildChecked(dir, launch)`: preflight, then `launch(dir)` only when it passed. The
    /// launcher builds the set folder it is given.
    pub fn build_checked(&self, dir: &Path, launch: impl FnOnce(&Path) -> LaunchOutcome) -> Result<CheckedReport> {
        let report = self.check_set(dir)?;
        if report.report.status != Status::Pass {
            return Ok(CheckedReport { report, launch: None });
        }
        let launch = launch(&paths::normalize(dir));
        Ok(CheckedReport { report, launch: Some(launch) })
    }

    /// The `NN-*` folders of `motion/icon-sets` in JS sort order, split into those with
    /// `definitions.mjs` and the names of those without.
    fn library_sets(&self) -> Result<(Vec<PathBuf>, Vec<String>)> {
        let sets_dir = paths::join(&self.root, "motion/icon-sets");
        let mut names = Vec::new();
        for entry in fs::read_dir(&sets_dir).map_err(Error::io(&sets_dir))? {
            let name = entry.map_err(Error::io(&sets_dir))?.file_name().to_string_lossy().into_owned();
            // `/^\d\d-/`; JS `\d` without the `u` flag is ASCII.
            if let [a, b, b'-', ..] = name.as_bytes()
                && a.is_ascii_digit()
                && b.is_ascii_digit()
            {
                names.push(name);
            }
        }
        names.sort_by(|a, b| paths::js_cmp(a, b));
        let (mut dirs, mut skipped) = (Vec::new(), Vec::new());
        for name in names {
            let dir = paths::join(&sets_dir, &name);
            if paths::join(&dir, "definitions.mjs").exists() {
                dirs.push(dir);
            } else {
                skipped.push(name);
            }
        }
        Ok((dirs, skipped))
    }

    /// Screens `dirs` in parallel, largest sources first so the slowest sets do not start
    /// last, and returns the results in `dirs` order.
    fn screen_all(&self, dirs: &[PathBuf]) -> Vec<Result<Screened>> {
        let mut queue: Vec<(usize, &PathBuf)> = dirs.iter().enumerate().collect();
        queue.sort_by_cached_key(|(_, dir)| std::cmp::Reverse(source_bytes(dir)));
        let order: Vec<usize> = queue.iter().map(|(index, _)| *index).collect();
        let screened = gai_js::map_parallel(queue, self.jobs, &self.limits, |(_, dir)| self.screen(dir));
        let mut results: Vec<Option<Result<Screened>>> = dirs.iter().map(|_| None).collect();
        for (index, result) in order.into_iter().zip(screened) {
            results[index] = Some(result);
        }
        results.into_iter().flatten().collect()
    }

    /// Loads the set's definitions (relative imports may reach the set and its sibling
    /// `library`) and checks them. Runs on a gai-js worker.
    fn screen(&self, dir: &Path) -> Result<Screened> {
        let source = SetSource::Definitions {
            set_dir: dir.to_owned(),
            library_dir: paths::join(dir, "../library"),
            profile: ModernProfile::SHARED,
        };
        let runtime = SetRuntime::load(&source, &self.limits)?;
        let report = check_definitions(&runtime, &self.contract)?;
        let names = runtime.with_icons(inspect::icon_names)?;
        Ok(Screened { report, names })
    }

    /// Catalog collisions (`preflight.mjs:135-143`), then the input inventory.
    fn finish(&self, dir: &Path, screened: Screened, catalog: &Catalog) -> Result<SetReport> {
        let Screened { mut report, names } = screened;
        // A set made only of specs (`library adopt --add`) defines no icons; `build` and
        // `library verify` check them instead.
        if report.diagnostics.iter().any(|d| d.code == Code::EmptySet) && self.spec_only(dir) {
            report = Report::new(0, 0, 0, Vec::new());
        }
        let set = paths::basename(dir);
        // "16-voice" → "16"; manifests are "prototype/icons/NN/assets/set.json".
        let number = set.split('-').next().unwrap_or_default();
        for name in names.iter().flatten().flatten() {
            let Some(existing) = catalog.find(name) else { continue };
            if existing.manifest.split('/').nth(2) != Some(number) {
                let owner = gai_core::js::to_js_string(existing.set.as_ref());
                report.diagnostics.push(Diagnostic {
                    code: Code::CatalogIdCollision,
                    icon: Some(name.as_str().into()),
                    part: None,
                    message: Some(
                        format!("ID already belongs to {owner}; revise there or choose a distinct concept/ID").into(),
                    ),
                    detail: Detail::None,
                });
                report.status = Status::Fail;
            }
        }
        let mut inputs = inventory::sources(dir, &self.library_dir())?;
        inputs.extend(catalog.inputs.iter().cloned());
        Ok(SetReport { set, report, inputs })
    }

    /// Whether the set's committed manifest lists icons and every one is adopted from its
    /// `specs/<id>.icon.json`.
    fn spec_only(&self, dir: &Path) -> bool {
        let set = paths::basename(dir);
        let number = set.split('-').next().unwrap_or_default();
        let manifest = paths::join(&self.root, format!("prototype/icons/{number}/assets/set.json"));
        let Some(doc) = fs::read(&manifest).ok().and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        else {
            return false;
        };
        let icons = doc.get("icons").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
        !icons.is_empty()
            && icons.iter().all(|icon| {
                crate::verify::adopted::is_adopted(icon)
                    && icon.get("icon").and_then(serde_json::Value::as_str).is_some_and(|id| {
                        paths::join(dir, format!("{}/{id}.icon.json", crate::verify::adopted::SPECS_DIR)).is_file()
                    })
            })
    }
}

/// Bytes of a set's `definitions.mjs` and of the files in `batches/`: a rough measure of its
/// sampling cost, for scheduling only.
fn source_bytes(dir: &Path) -> u64 {
    let size = |path: &Path| fs::metadata(path).map_or(0, |meta| meta.len());
    let batches = fs::read_dir(paths::join(dir, "batches")).into_iter().flatten().flatten();
    size(&paths::join(dir, "definitions.mjs")) + batches.map(|entry| size(&entry.path())).sum::<u64>()
}
