//! Build orchestration (builder spec §7): evaluate a set, write its scenes, export them,
//! post-process each native, and write the records into `set.json`.
//!
//! Files are written in the old builders' order, one animation after another: `.engine.js`,
//! `.scene.json`, the native and its provenance, the asset, its manifest and its two themes,
//! then `set.json` once every animation is done. With `jobs > 1` up to that many exports
//! overlap; each animation's files and its progress line still follow builder order.
//!
//! By default a failure leaves the files written so far, as the old builders did. With
//! [`BuildOptions::staging`] nothing is written until every animation succeeded; each file
//! is then replaced through a temporary file and a rename.
//!
//! Packaging (the `.lottie` archives and the final `set.json`) is `gai_package`'s job.

use crate::post_export::{self, Composition, PostExportError};
use crate::profile::{Profile, ProfileError, SetConfig};
use crate::records::{self, NativeFacts, Provenance, RecordError};
use crate::scene::{self, Scene, SceneError};
use crate::set_json;
use crate::template::ENGINE_SCRIPT;
use gai_core::export::{Exporter, NativeExport};
use gai_core::js::to_js_string;
use gai_core::{sha256_hex, to_js_json_pretty};
use gai_js::{Description, Limits, SetRuntime, SetSource};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{ErrorKind, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError, mpsc};
use std::{fs, thread};

/// Where a build reads its sources and writes its outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The set folder: `definitions.mjs`, `batches/`, `set-config.json` (and, for library
    /// sets, `build.mjs`).
    pub root: PathBuf,
    /// The kit folder the definitions import (`root/../library`).
    pub library: PathBuf,
    /// `<name>.engine.js` and `<name>.scene.json`.
    pub scenes: PathBuf,
    /// `<name>.json` natives and their provenance.
    pub native: PathBuf,
    /// Assets, manifests, themes and `set.json`.
    pub out: PathBuf,
}

impl Layout {
    /// The library paths of builder spec §2.1: scenes and natives inside the set folder,
    /// outputs in `<repo>/prototype/icons/<NN>/assets` where `NN` starts the set id.
    pub fn library_set(root: &Path, profile: &Profile) -> Result<Layout, BuildError> {
        let repo = root
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| BuildError::NotInLibrary(root.to_path_buf()))?;
        Ok(Layout {
            root: root.to_path_buf(),
            library: library_of(root),
            scenes: root.join("scenes"),
            native: root.join("native"),
            out: repo.join("prototype/icons").join(profile.number()).join("assets"),
        })
    }

    /// A user project's paths: sources in `icons_dir` with the kit copy beside it, and every
    /// output under `build_dir` (scenes and natives in subfolders, assets at its top).
    pub fn project(icons_dir: &Path, build_dir: &Path) -> Layout {
        Layout {
            root: icons_dir.to_path_buf(),
            library: library_of(icons_dir),
            scenes: build_dir.join("scenes"),
            native: build_dir.join("native"),
            out: build_dir.to_path_buf(),
        }
    }
}

/// `root/../library`, lexically.
fn library_of(root: &Path) -> PathBuf {
    root.parent().map_or_else(|| root.join("..").join("library"), |parent| parent.join("library"))
}

/// A set to build: its profile and its paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub profile: Profile,
    pub layout: Layout,
}

impl Target {
    /// A library set folder (`motion/icon-sets/NN-slug`), resolved to its canonical path.
    pub fn library_set(set_dir: impl AsRef<Path>) -> Result<Target, BuildError> {
        let root = canonical(set_dir.as_ref())?;
        let profile = Profile::library_set(&root)?;
        let layout = Layout::library_set(&root, &profile)?;
        Ok(Target { profile, layout })
    }

    /// A user project: `icons_dir` holds `definitions.mjs`, `batches/` and `set-config.json`
    /// and sits next to the kit copy (`../library`); outputs go under `build_dir`.
    pub fn project(icons_dir: impl AsRef<Path>, build_dir: impl AsRef<Path>) -> Result<Target, BuildError> {
        let root = canonical(icons_dir.as_ref())?;
        let profile = Profile::project(&SetConfig::load(&root)?);
        Ok(Target { profile, layout: Layout::project(&root, build_dir.as_ref()) })
    }

    /// The definitions this target evaluates, with the profile's sampling rules.
    pub fn source(&self) -> SetSource {
        SetSource::Definitions {
            set_dir: self.layout.root.clone(),
            library_dir: self.layout.library.clone(),
            profile: self.profile.sampling,
        }
    }
}

fn canonical(path: &Path) -> Result<PathBuf, BuildError> {
    fs::canonicalize(path).map_err(|source| BuildError::Read { path: path.to_path_buf(), source })
}

/// What a build does.
#[derive(Clone, Copy)]
pub enum Mode<'a> {
    /// Sample and write scenes only (`.engine.js` and `.scene.json`); no engine, no assets.
    OfflineScenes,
    /// Skip sampling and the engine: read each existing `native/<name>.json`, then write the
    /// assets, themes, manifests and `set.json`. Makes everything after the engine
    /// reproducible offline.
    ReuseNative,
    /// Everything: scenes, one export per animation, assets and `set.json`.
    Full(&'a dyn Exporter),
}

impl fmt::Debug for Mode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Mode::OfflineScenes => "OfflineScenes",
            Mode::ReuseNative => "ReuseNative",
            Mode::Full(_) => "Full",
        })
    }
}

/// How a build runs.
#[derive(Clone)]
pub struct BuildOptions<'a> {
    /// Build only this icon. The whole set is still evaluated; `set.json` is not written,
    /// since a partial one would drop the other icons.
    pub icon: Option<String>,
    /// Exports in flight at once (at least 1).
    pub jobs: usize,
    /// Write nothing until every animation succeeded, then replace each file atomically.
    pub staging: bool,
    /// Limits for the JS runtime.
    pub limits: Limits,
    /// The script written as each `<name>.engine.js`. It must be the script the exporter
    /// runs: a provenance `scriptSha256` that differs from it fails the build, since verify
    /// compares the two. Default: the legacy template ([`ENGINE_SCRIPT`]).
    pub engine_script: &'a [u8],
    /// Called after each animation, in builder order, on the calling thread.
    pub progress: Option<&'a (dyn Fn(&AnimationReport) + Sync)>,
}

impl Default for BuildOptions<'_> {
    fn default() -> Self {
        BuildOptions {
            icon: None,
            jobs: 1,
            staging: false,
            limits: Limits::default(),
            engine_script: ENGINE_SCRIPT,
            progress: None,
        }
    }
}

impl fmt::Debug for BuildOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuildOptions")
            .field("icon", &self.icon)
            .field("jobs", &self.jobs)
            .field("staging", &self.staging)
            .field("limits", &self.limits)
            .field("engine_script", &sha256_hex(self.engine_script))
            .field("progress", &self.progress.is_some())
            .finish()
    }
}

/// One finished animation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationReport {
    /// The animation name.
    pub name: String,
    /// `icon.label`, as JS would interpolate it.
    pub label: String,
    /// `Light motion` or `Strong motion`.
    pub motion_label: String,
    /// Native layers, once post-export ran (`None` in [`Mode::OfflineScenes`]).
    pub layers: Option<usize>,
}

impl fmt::Display for AnimationReport {
    /// The old builders' line, e.g. `Alarm clock / Strong motion: 31 native layers exported`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.layers {
            Some(layers) => write!(f, "{} / {}: {layers} native layers exported", self.label, self.motion_label),
            None => write!(f, "{} / {}: scene written", self.label, self.motion_label),
        }
    }
}

/// What a build did.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildReport {
    /// The set id.
    pub set_id: String,
    /// Every animation built, in builder order.
    pub animations: Vec<AnimationReport>,
    /// The builder's `set.json` value, when records were made. It holds only the icons that
    /// were built, and is written to `out/set.json` only for a whole-set build.
    pub set_json: Option<Value>,
    /// Every file written, in order.
    pub files: Vec<PathBuf>,
}

/// Why a build failed.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Profile(#[from] ProfileError),
    #[error(transparent)]
    Js(#[from] gai_js::Error),
    #[error(transparent)]
    Scene(#[from] SceneError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error("{} is not inside a library checkout (motion/icon-sets/<set>)", .0.display())]
    NotInLibrary(PathBuf),
    #[error("no icon named {0}")]
    UnknownIcon(String),
    #[error("animation name {0:?} cannot be used as a file name")]
    InvalidName(String),
    /// `library/run-ai.mjs`: an AI set must define exactly its scope's icons, in order.
    #[error("Incomplete or misordered AI scope: {0}")]
    Scope(String),
    #[error("{animation}: export failed: {error:#}")]
    Export { animation: String, error: anyhow::Error },
    /// The exporter ran a different script from the `.engine.js` written for it.
    #[error("{animation}: the exporter recorded script sha256 {recorded}, but {animation}.engine.js has {written}")]
    Script { animation: String, recorded: String, written: String },
    #[error("{animation}: {source}")]
    PostExport { animation: String, source: PostExportError },
    #[error("cannot read {}: {source}", path.display())]
    Read { path: PathBuf, source: std::io::Error },
    #[error("{}: {source}", path.display())]
    Json { path: PathBuf, source: serde_json::Error },
    #[error("cannot write {}: {source}", path.display())]
    Write { path: PathBuf, source: std::io::Error },
}

/// Builds the library set at `set_dir` (`motion/icon-sets/NN-slug`).
pub fn build_set(set_dir: &Path, mode: Mode<'_>, options: &BuildOptions<'_>) -> Result<BuildReport, BuildError> {
    build(&Target::library_set(set_dir)?, mode, options)
}

/// Builds `target`.
pub fn build(target: &Target, mode: Mode<'_>, options: &BuildOptions<'_>) -> Result<BuildReport, BuildError> {
    let (profile, layout) = (&target.profile, &target.layout);
    let source = target.source();
    let icon = options.icon.as_deref();
    let sample = !matches!(mode, Mode::ReuseNative);
    let (description, scenes) = gai_js::run_on_worker(&options.limits, || evaluate(&source, options, sample))?;
    let selected: Vec<usize> = match icon {
        Some(name) => vec![icon_index(&description, name)?],
        None => {
            check_scope(layout, profile, &description)?;
            (0..description.icons.len()).collect()
        }
    };
    for &index in &selected {
        for variant in &description.icons[index].variants {
            check_name(&variant.name)?;
        }
    }

    let mut run = Run {
        target,
        options,
        description: &description,
        provenance: None,
        output: Output::new(options.staging),
        animations: Vec::new(),
        records: vec![Vec::new(); description.icons.len()],
    };
    match mode {
        Mode::OfflineScenes => {
            run.output.directory(&layout.scenes)?;
            for scene in &scenes {
                run.write_scene(scene)?;
                run.report(scene.icon, scene.variant, None);
            }
        }
        Mode::ReuseNative => {
            run.provenance = Some(Provenance::collect(profile, &layout.root, &layout.library)?);
            run.output.directory(&layout.out)?;
            for &index in &selected {
                for (variant, animation) in description.icons[index].variants.iter().enumerate() {
                    let path = layout.native.join(format!("{}.json", animation.name));
                    let native = fs::read(&path).map_err(|source| BuildError::Read { path: path.clone(), source })?;
                    let text = String::from_utf8(native).map_err(|error| BuildError::Read {
                        path,
                        source: std::io::Error::new(ErrorKind::InvalidData, error),
                    })?;
                    run.finish(index, variant, &text)?;
                }
            }
        }
        Mode::Full(exporter) => {
            run.provenance = Some(Provenance::collect(profile, &layout.root, &layout.library)?);
            for directory in [&layout.scenes, &layout.native, &layout.out] {
                run.output.directory(directory)?;
            }
            let script_sha256 = sha256_hex(options.engine_script);
            export_in_order(exporter, options.jobs, &scenes, |step| match step {
                Step::Scene(index) => run.write_scene(&scenes[index]),
                Step::Native(index, export) => run.write_native(&scenes[index], export, &script_sha256),
            })?;
        }
    }

    let set_json = run.provenance.is_some().then(|| {
        let icons = selected.iter().map(|&i| set_json::icon_entry(&description.icons[i], &run.records[i])).collect();
        set_json::assemble(profile, icons)
    });
    if let (Some(set), None) = (&set_json, icon) {
        run.output.write(layout.out.join("set.json"), set_json::encode(set).into_bytes())?;
    }
    let Run { output, animations, .. } = run;
    Ok(BuildReport { set_id: profile.id.clone(), animations, set_json, files: output.commit()? })
}

/// Loads and installs the whole set and, when `sample` is set, samples the selected icons.
/// Runs on a JS worker thread.
fn evaluate(
    source: &SetSource,
    options: &BuildOptions<'_>,
    sample: bool,
) -> Result<(Description, Vec<Scene>), BuildError> {
    let mut runtime = SetRuntime::load(source, &options.limits)?;
    let description = runtime.install()?;
    let scenes = match (sample, options.icon.as_deref()) {
        (false, _) => Vec::new(),
        (true, Some(icon)) => scene::build_icon_scenes(&runtime, &description, icon_index(&description, icon)?)?,
        (true, None) => scene::build_scenes(&runtime, &description)?,
    };
    Ok((description, scenes))
}

fn icon_index(description: &Description, name: &str) -> Result<usize, BuildError> {
    description.icons.iter().position(|icon| icon.name == name).ok_or_else(|| BuildError::UnknownIcon(name.to_owned()))
}

/// `library/run-ai.mjs:10-15`: a set listed in the kit's AI scope must define exactly the
/// scope's icons, in order. Sets outside the scope, and kits without it, pass.
fn check_scope(layout: &Layout, profile: &Profile, description: &Description) -> Result<(), BuildError> {
    let path = layout.library.join("design-reviews/ai-category/scope.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(BuildError::Read { path, source }),
    };
    let scope: Value = serde_json::from_slice(&bytes).map_err(|source| BuildError::Json { path, source })?;
    let names: Vec<Value> = description.icons.iter().map(|icon| Value::from(icon.name.as_str())).collect();
    let listed = scope.get("sets").and_then(Value::as_array).into_iter().flatten();
    for set in listed.filter(|set| set.get("id").and_then(Value::as_str) == Some(profile.id.as_str())) {
        if set.get("icons").and_then(Value::as_array) != Some(&names) {
            return Err(BuildError::Scope(profile.id.clone()));
        }
    }
    Ok(())
}

/// Animation names become file stems, so they must stay inside their folder on every OS.
/// On Windows a `:` starts a drive prefix (`C:x` replaces the folder when joined) or an
/// alternate data stream, and a device name (`NUL`, `com1.x`) is not a file at all.
fn check_name(name: &str) -> Result<(), BuildError> {
    let stem = name.split('.').next().unwrap_or_default().trim_end_matches(' ');
    let device = matches!(stem.to_ascii_uppercase().as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem.len() == 4
            && [b"COM", b"LPT"].iter().any(|d| stem.as_bytes()[..3].eq_ignore_ascii_case(*d))
            && stem.as_bytes()[3].is_ascii_digit();
    let safe = !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':', '\0'])
        && !device;
    if safe { Ok(()) } else { Err(BuildError::InvalidName(name.to_owned())) }
}

/// The state of one build.
struct Run<'a> {
    target: &'a Target,
    options: &'a BuildOptions<'a>,
    description: &'a Description,
    /// Collected once records are needed (every mode but [`Mode::OfflineScenes`]).
    provenance: Option<Provenance>,
    output: Output,
    animations: Vec<AnimationReport>,
    /// Each icon's records, in builder order.
    records: Vec<Vec<Value>>,
}

impl Run<'_> {
    fn write_scene(&mut self, scene: &Scene) -> Result<(), BuildError> {
        let scenes = &self.target.layout.scenes;
        self.output.write(scenes.join(format!("{}.engine.js", scene.name)), self.options.engine_script.to_vec())?;
        self.output.write(scenes.join(format!("{}.scene.json", scene.name)), scene.json.clone().into_bytes())
    }

    fn write_native(&mut self, scene: &Scene, export: NativeExport, script_sha256: &str) -> Result<(), BuildError> {
        if let Some(recorded) = export.provenance.get("scriptSha256").and_then(Value::as_str)
            && recorded != script_sha256
        {
            return Err(BuildError::Script {
                animation: scene.name.clone(),
                recorded: recorded.to_owned(),
                written: script_sha256.to_owned(),
            });
        }
        let native = &self.target.layout.native;
        let provenance = to_js_json_pretty(&export.provenance, 2) + "\n";
        self.output.write(native.join(format!("{}.json", scene.name)), export.text.clone().into_bytes())?;
        self.output.write(native.join(format!("{}.json.provenance.json", scene.name)), provenance.into_bytes())?;
        self.finish(scene.icon, scene.variant, &export.text)
    }

    /// Post-export, record and progress for one animation (`build-set.mjs:139-228`).
    fn finish(&mut self, icon: usize, variant: usize, native: &str) -> Result<(), BuildError> {
        let profile = &self.target.profile;
        let (icon_data, animation) = (&self.description.icons[icon], &self.description.icons[icon].variants[variant]);
        let name = animation.name.as_str();
        let nm = format!("{}{}", profile.composition, to_js_string(animation.meta.get("label")));
        // Not numbers: JS would write null markers; post-export refuses them.
        let active = animation.meta.get("active").and_then(Value::as_f64).unwrap_or(f64::NAN);
        let rest = animation.rest().unwrap_or(f64::NAN);
        let composition = Composition { name: &nm, set_id: &profile.id, active, rest };
        let asset = post_export::post_export(native, &composition)
            .map_err(|source| BuildError::PostExport { animation: name.to_owned(), source })?;
        let out = &self.target.layout.out;
        self.output.write(out.join(format!("{name}.json")), asset.json.into_bytes())?;
        self.output
            .write(out.join(format!("{name}.manifest.json")), post_export::builder_manifest(name).into_bytes())?;
        for ((theme, _), text) in post_export::THEMES.iter().zip(asset.themes) {
            self.output.write(out.join(format!("{name}.theme-{theme}.json")), text.into_bytes())?;
        }
        let sha256 = sha256_hex(native);
        let facts = NativeFacts { layers: asset.layers, slots: &asset.slots, sha256: &sha256 };
        let provenance = self.provenance.as_ref().expect("provenance is collected before post-export");
        let record = records::record(profile, provenance, icon_data, animation, &facts)?;
        self.records[icon].push(record);
        self.report(icon, variant, Some(asset.layers));
        Ok(())
    }

    fn report(&mut self, icon: usize, variant: usize, layers: Option<usize>) {
        let animation = &self.description.icons[icon].variants[variant];
        let report = AnimationReport {
            name: animation.name.clone(),
            label: to_js_string(animation.meta.get("label")),
            motion_label: to_js_string(animation.meta.get("motionLabel")),
            layers,
        };
        if let Some(progress) = self.options.progress {
            progress(&report);
        }
        self.animations.push(report);
    }
}

/// One step of an export run, taken in builder order on the calling thread.
enum Step {
    /// Write this scene before its export starts.
    Scene(usize),
    /// This scene's export finished.
    Native(usize, NativeExport),
}

/// Exports `scenes` on up to `jobs` threads. `step` sees each scene before its export is
/// dispatched and each result in scene order; the first error stops dispatching, and exports
/// already running finish before it is returned.
fn export_in_order(
    exporter: &dyn Exporter,
    jobs: usize,
    scenes: &[Scene],
    mut step: impl FnMut(Step) -> Result<(), BuildError>,
) -> Result<(), BuildError> {
    type Outcome = thread::Result<anyhow::Result<NativeExport>>;
    let count = scenes.len();
    let jobs = jobs.clamp(1, count.max(1));
    let (job_sender, jobs_queue) = mpsc::channel::<usize>();
    let jobs_queue = Mutex::new(jobs_queue);
    let (done_sender, done) = mpsc::channel::<(usize, Outcome)>();
    thread::scope(|scope| {
        // Owned by this closure, so workers stop however it returns.
        let job_sender = job_sender;
        for _ in 0..jobs {
            let (queue, done_sender) = (&jobs_queue, done_sender.clone());
            scope.spawn(move || {
                loop {
                    // A statement of its own, so the lock is released before the export.
                    let next = queue.lock().unwrap_or_else(PoisonError::into_inner).recv();
                    let Ok(index) = next else { break };
                    let scene = &scenes[index];
                    let outcome = panic::catch_unwind(AssertUnwindSafe(|| exporter.export(&scene.name, &scene.json)));
                    if done_sender.send((index, outcome)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(done_sender);
        let mut pending = BTreeMap::new();
        let (mut dispatched, mut received, mut finished) = (0, 0, 0);
        while finished < count {
            while dispatched < count && dispatched - received < jobs {
                step(Step::Scene(dispatched))?;
                job_sender.send(dispatched).expect("export workers wait for jobs while the sender lives");
                dispatched += 1;
            }
            let (index, outcome) = done.recv().expect("export workers answer every dispatched job");
            received += 1;
            pending.insert(index, outcome.unwrap_or_else(|payload| panic::resume_unwind(payload)));
            while let Some(result) = pending.remove(&finished) {
                let export =
                    result.map_err(|error| BuildError::Export { animation: scenes[finished].name.clone(), error })?;
                step(Step::Native(finished, export))?;
                finished += 1;
            }
        }
        Ok(())
    })
}

/// Where files go: straight to disk, or held until [`Output::commit`].
struct Output {
    /// `Some` when staging.
    staged: Option<Vec<(PathBuf, Vec<u8>)>>,
    directories: Vec<PathBuf>,
    written: Vec<PathBuf>,
}

impl Output {
    fn new(staging: bool) -> Self {
        Output { staged: staging.then(Vec::new), directories: Vec::new(), written: Vec::new() }
    }

    /// Creates `directory` now, or at commit when staging.
    fn directory(&mut self, directory: &Path) -> Result<(), BuildError> {
        if self.staged.is_some() {
            self.directories.push(directory.to_path_buf());
            return Ok(());
        }
        create_directory(directory)
    }

    fn write(&mut self, path: PathBuf, bytes: Vec<u8>) -> Result<(), BuildError> {
        match &mut self.staged {
            Some(staged) => staged.push((path, bytes)),
            None => {
                fs::write(&path, bytes).map_err(|source| BuildError::Write { path: path.clone(), source })?;
                self.written.push(path);
            }
        }
        Ok(())
    }

    /// Writes staged files, each through a temporary file renamed over its target, and
    /// returns every path written.
    fn commit(self) -> Result<Vec<PathBuf>, BuildError> {
        let Some(staged) = self.staged else { return Ok(self.written) };
        for directory in &self.directories {
            create_directory(directory)?;
        }
        let mut written = Vec::with_capacity(staged.len());
        for (path, bytes) in staged {
            replace(&path, &bytes).map_err(|source| BuildError::Write { path: path.clone(), source })?;
            written.push(path);
        }
        Ok(written)
    }
}

fn create_directory(directory: &Path) -> Result<(), BuildError> {
    fs::create_dir_all(directory).map_err(|source| BuildError::Write { path: directory.to_path_buf(), source })
}

/// Replaces `path` with `bytes` atomically, keeping the permissions of the file it replaces.
fn replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = tempfile::Builder::new().prefix(".gai-build-").tempfile_in(directory)?;
    temp.write_all(bytes)?;
    match fs::metadata(path) {
        Ok(existing) => temp.as_file().set_permissions(existing.permissions())?,
        Err(error) if error.kind() == ErrorKind::NotFound => new_file_permissions(temp.as_file())?,
        Err(error) => return Err(error),
    }
    temp.persist(path).map_err(|error| error.error)?;
    Ok(())
}

/// Temporary files are private (0600); a new output gets the usual 0644.
#[cfg(unix)]
fn new_file_permissions(file: &fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o644))
}

#[cfg(not(unix))]
fn new_file_permissions(_: &fs::File) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_stay_inside_their_folder() {
        for name in ["alarm-clock-reverse-strong", "console", "com", "com10", "lpt-strong", "nul-x", "a€"] {
            assert!(check_name(name).is_ok(), "{name:?}");
        }
        for name in ["", ".", "..", "a/b", "a\\b", "a\0"] {
            assert!(check_name(name).is_err(), "{name:?}");
        }
        // Windows: drive-relative prefixes, alternate data streams and device names.
        for name in ["C:evil", "C:..", "x:stream", "NUL", "con", "aux.scene", "Com1", "lpt9.x", "prn "] {
            assert!(check_name(name).is_err(), "{name:?}");
        }
    }

    #[test]
    fn report_lines_match_the_builder() {
        let mut report = AnimationReport {
            name: "alarm-clock-strong".into(),
            label: "Alarm clock".into(),
            motion_label: "Strong motion".into(),
            layers: Some(31),
        };
        assert_eq!(report.to_string(), "Alarm clock / Strong motion: 31 native layers exported");
        report.layers = None;
        assert_eq!(report.to_string(), "Alarm clock / Strong motion: scene written");
    }

    #[test]
    fn staged_output_writes_nothing_until_commit() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("a.json");
        fs::write(&existing, "old").unwrap();
        let mut output = Output::new(true);
        output.directory(&dir.path().join("sub")).unwrap();
        output.write(existing.clone(), b"new".to_vec()).unwrap();
        output.write(dir.path().join("sub/b.json"), b"b".to_vec()).unwrap();
        assert_eq!(fs::read_to_string(&existing).unwrap(), "old");
        assert!(!dir.path().join("sub").exists());
        let written = output.commit().unwrap();
        assert_eq!(written, [existing.clone(), dir.path().join("sub/b.json")]);
        assert_eq!(fs::read_to_string(&existing).unwrap(), "new");
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".gai-build-"))
            .collect();
        assert!(leftovers.is_empty());
    }

    struct Echo;

    impl Exporter for Echo {
        fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
            anyhow::ensure!(name != "fails", "engine refused {name}");
            Ok(NativeExport { text: scene_json.to_owned(), provenance: Value::Null })
        }
    }

    fn scenes(names: &[&str]) -> Vec<Scene> {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| Scene { icon: 0, variant: i, name: (*name).into(), json: format!("{i}") })
            .collect()
    }

    #[test]
    fn exports_are_taken_in_order() {
        for jobs in [1, 3, 8] {
            let scenes = scenes(&["a", "b", "c", "d", "e"]);
            let mut steps = Vec::new();
            export_in_order(&Echo, jobs, &scenes, |step| {
                steps.push(match step {
                    Step::Scene(i) => format!("scene {i}"),
                    Step::Native(i, export) => format!("native {i} {}", export.text),
                });
                Ok(())
            })
            .unwrap();
            let natives: Vec<&String> = steps.iter().filter(|s| s.starts_with("native")).collect();
            assert_eq!(natives, ["native 0 0", "native 1 1", "native 2 2", "native 3 3", "native 4 4"]);
            for i in 0..5 {
                let scene = steps.iter().position(|s| *s == format!("scene {i}")).unwrap();
                let native = steps.iter().position(|s| *s == format!("native {i} {i}")).unwrap();
                assert!(scene < native);
            }
            if jobs == 1 {
                assert_eq!(steps[..4], ["scene 0", "native 0 0", "scene 1", "native 1 1"]);
            }
        }
    }

    #[test]
    fn the_first_failure_stops_the_run() {
        let scenes = scenes(&["a", "fails", "c", "d"]);
        let mut natives = Vec::new();
        let error = export_in_order(&Echo, 2, &scenes, |step| {
            if let Step::Native(i, _) = step {
                natives.push(i);
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(natives, [0]);
        assert_eq!(error.to_string(), "fails: export failed: engine refused fails");
    }
}
