//! `build`: turns local icon source into installed assets. For each icon it samples the
//! definitions, exports every action through the hosted engine, post-processes the
//! natives, packages the `.lottie` files, snapshots the sources, verifies the build,
//! renders preview sheets and installs the icon where `add` puts library icons.
//!
//! Each icon builds into a fresh folder that replaces `.gen-animated-icons/build/<id>/`
//! only when export and packaging succeeded. An icon whose verification fails is not
//! installed. When an icon cannot be built (an engine or source error), the icons built
//! and verified before it are still installed, then the error is returned.

use super::add::{build_facts, local_descriptor, stage_icon, stage_settings};
use super::check::Origin;
use super::check::{self, verify_build};
use super::preview;
use super::{lock_project, text_list};
use crate::Context;
use crate::cli::BuildArgs;
use crate::config::{Config, Layout};
use crate::engine;
use crate::error::{CliError, ErrorCode, Result};
use crate::kit;
use crate::local_set::{BUILD_DIR, Built, LocalSet, PREVIEW_DIR, snapshot};
use crate::lock::Lock;
use crate::managed::{Batch, create_dir_inside, inside, write_inside};
use crate::output::Report;
use gai_build::build::{AnimationReport, BuildError, BuildOptions, Mode, Target, build};
use gai_core::export::Exporter;
use gai_core::sha256_hex;
use gai_engine::HostedExporter;
use gai_engine::config::MAX_DOCUMENTS;
use gai_engine::export::EXPORT_SCRIPT;
use gai_package::zip::ZipDate;
use gai_package::{ActionList, build_set_archives, encode_set_json};
use serde_json::{Map, Value, json};
use std::fmt::Write;
use std::path::Path;

/// Runs `build` against the hosted engine; `--offline` refuses before anything runs.
pub fn run(ctx: &Context, args: &BuildArgs) -> Result<Report> {
    engine::require_online(ctx, "build", "Drop --offline to build; gen-animated-icons check runs the offline checks")?;
    let mut config = engine::config(ctx)?;
    config.documents = args.jobs.clamp(1, MAX_DOCUMENTS);
    engine::require_session(&config)?;
    let exporter = HostedExporter::new(&config).map_err(engine::error)?;
    build_with(ctx, args, &exporter, Some(config.url.as_str()))
}

/// Runs `build` with any exporter whose provenance records [`EXPORT_SCRIPT`].
pub fn build_with(
    ctx: &Context,
    args: &BuildArgs,
    exporter: &dyn Exporter,
    engine_url: Option<&str>,
) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    if config.layout == Layout::Legacy {
        return Err(CliError::new(ErrorCode::InvalidConfig, "This project was set up by lottie-icons")
            .with_hint("Run gen-animated-icons init --migrate first"));
    }
    let _guard = lock_project(ctx, config.layout)?;
    let set = LocalSet::of(&config);
    set.require(root)?;
    // Builds record `kitSha256`, so they evaluate only the kit exactly as shipped.
    kit::verify(root, &set.kit)?;
    let requested: Vec<String> = super::unique(&args.ids).into_iter().map(str::to_string).collect();
    let mut warnings = Vec::new();
    let source = check::source(ctx, &set, &requested, &mut warnings)?;
    if source.check.status == "FAIL" {
        let value = json!({ "status": "FAIL", "stage": "source", "checks": [source.check.to_json()] });
        let mut text = String::from("FAIL: the source preflight found errors; nothing was exported.\n");
        for diagnostic in &source.check.diagnostics {
            let _ = writeln!(text, "  {}", gai_core::to_js_json(diagnostic));
        }
        return Ok(Report { failed: true, warnings, ..Report::new(value, text) });
    }
    // What `submit` would refuse later is said before the export.
    for diagnostic in source.check.diagnostics.iter().filter(|d| d.get("code") == Some(&json!("UNTRACED_IMPORTS"))) {
        let field = |key: &str| diagnostic.get(key).and_then(Value::as_str).unwrap_or_default();
        let line = diagnostic.get("line").map(|line| format!(":{line}")).unwrap_or_default();
        warnings.push(format!("{}{line}: {}", field("path"), field("message")));
    }
    let ids = if requested.is_empty() || args.all { source.names.clone() } else { requested };
    if ids.is_empty() {
        return Err(CliError::usage("The definitions export no icons").with_hint("Run gen-animated-icons create <id>"));
    }

    let icons_dir = set.icons_dir(root)?;
    let kit_dir = set.kit_dir(root)?;
    let origin = Origin { root, set_dir: &icons_dir, label: Some(&set.icons) };
    // Forward and reset actions share a label, so each line names its action.
    let progress = |report: &AnimationReport| ctx.out.log(&format!("{}: {report}", report.name));
    let mut results = Vec::new();
    let mut failure = None;
    for (index, id) in ids.iter().enumerate() {
        ctx.out.log(&format!("building {id} ({} of {})", index + 1, ids.len()));
        let finished = build_icon(root, &icons_dir, &source.modules, id, exporter, args.jobs, &progress, origin)
            .and_then(|built| {
                let verdict = verify_build(&built.dir, &kit_dir);
                let out = create_dir_inside(root, &format!("{PREVIEW_DIR}/{id}"))?;
                let sheets = preview::Review::of(&built.icon).render(&built.lottie, &out)?;
                Ok(IconBuild { id: id.clone(), built, verdict, sheets })
            });
        match finished {
            Ok(result) => results.push(result),
            Err(error) => {
                failure = Some((id.clone(), error));
                break;
            }
        }
    }

    let mut changed = Vec::new();
    if !args.no_install && (failure.is_none() || !results.is_empty()) {
        changed = install(root, &config, results.iter().filter(|r| r.verdict.error.is_none()))?;
    }
    if let Some((id, error)) = failure {
        return Err(unfinished(error, &id, &results, !args.no_install));
    }
    let passed = results.iter().all(|r| r.verdict.error.is_none());
    let server_version = results.first().and_then(|r| server_version(&r.built));
    let value = json!({
        "status": if passed { "PASS" } else { "FAIL" },
        "engine": { "url": engine_url, "serverVersion": server_version },
        "icons": results.iter().map(|r| r.to_json(root, &config, !args.no_install)).collect::<Vec<_>>(),
        "changed": changed,
    });
    let mut text = String::new();
    for result in &results {
        let status = if result.verdict.error.is_none() { "PASS" } else { "FAIL" };
        let _ = writeln!(
            text,
            "{status} {}: {} actions built in {BUILD_DIR}/{}",
            result.id,
            result.actions().len(),
            result.id
        );
        if let Some(error) = &result.verdict.error {
            let _ = writeln!(text, "  verification: {error}");
        }
        for sheet in &result.sheets {
            let _ = writeln!(text, "  preview: {}", preview::relative(root, &sheet.path));
        }
    }
    text_list(&mut text, "Changed", &changed);
    Ok(Report { failed: !passed, warnings, ..Report::new(value, text) })
}

/// One icon's finished build.
struct IconBuild {
    id: String,
    built: Built,
    verdict: check::Verdict,
    sheets: Vec<preview::Sheet>,
}

impl IconBuild {
    /// `{name, sha256, bytes}` of each action's `.lottie`, from the packaged set.json.
    fn actions(&self) -> Vec<Value> {
        let animations = self.built.icon.get("animations").and_then(Value::as_array).into_iter().flatten();
        animations
            .map(|a| json!({ "name": a.get("name"), "sha256": a.get("sha256"), "bytes": a.get("bytes") }))
            .collect()
    }

    fn to_json(&self, root: &Path, config: &Config, install: bool) -> Value {
        let passed = self.verdict.error.is_none();
        let mut icon = Map::new();
        icon.insert("id".into(), json!(self.id));
        icon.insert("actions".into(), Value::Array(self.actions()));
        let installed = install && passed;
        let asset = format!("{}/{}.lottie", config.assets, self.id);
        icon.insert(
            "asset".into(),
            json!({ "path": installed.then_some(asset), "sha256": sha256_hex(&self.built.lottie), "bytes": self.built.lottie.len() }),
        );
        let descriptor = match &config.public_path {
            Some(_) => format!("{}/{}.mjs", config.descriptors, self.id),
            None => format!("{}/{}.json", config.descriptors, self.id),
        };
        icon.insert("descriptor".into(), json!(installed.then_some(descriptor)));
        icon.insert("checks".into(), json!(if passed { "PASS" } else { "FAIL" }));
        if let Some(error) = &self.verdict.error {
            icon.insert("checkError".into(), json!(error));
        }
        icon.insert("build".into(), json!(format!("{BUILD_DIR}/{}", self.id)));
        icon.insert("preview".into(), Value::Array(self.sheets.iter().map(|s| s.to_json(root)).collect()));
        Value::Object(icon)
    }
}

/// The engine version recorded in the Strong action's provenance.
fn server_version(built: &Built) -> Option<Value> {
    let proof = std::fs::read(built.dir.join(format!("native/{}.json.provenance.json", built.strong))).ok()?;
    let proof: Value = serde_json::from_slice(&proof).ok()?;
    proof.pointer("/server/version").cloned()
}

/// The error of an icon that could not be built, naming the icons of the same run that
/// were built (and installed) before it.
fn unfinished(error: CliError, id: &str, done: &[IconBuild], installed: bool) -> CliError {
    let error = if error.id.is_none() { error.with_id(id) } else { error };
    if done.is_empty() {
        return error;
    }
    let (passed, failed): (Vec<&IconBuild>, Vec<&IconBuild>) = done.iter().partition(|r| r.verdict.error.is_none());
    let names = |list: &[&IconBuild]| list.iter().map(|r| r.id.as_str()).collect::<Vec<_>>().join(", ");
    let mut notes = Vec::new();
    if !passed.is_empty() {
        let verb = if installed { "built and installed" } else { "built" };
        notes.push(format!("{} {verb} before it", names(&passed)));
    }
    if !failed.is_empty() {
        notes.push(format!("{} built but failed verification", names(&failed)));
    }
    CliError { message: format!("{} ({})", error.message, notes.join("; ")), ..error }
}

/// Builds and packages one icon into a staging folder, snapshots its sources (`modules`,
/// the icons-relative modules the loader read), then moves it to
/// `.gen-animated-icons/build/<id>`.
#[allow(clippy::too_many_arguments)]
fn build_icon(
    root: &Path,
    icons_dir: &Path,
    modules: &[String],
    id: &str,
    exporter: &dyn Exporter,
    jobs: usize,
    progress: &(dyn Fn(&AnimationReport) + Sync),
    origin: Origin<'_>,
) -> Result<Built> {
    let builds = create_dir_inside(root, BUILD_DIR)?;
    let staging = tempfile::Builder::new()
        .prefix(&format!(".{id}-"))
        .tempdir_in(&builds)
        .map_err(|error| CliError::io("create", &builds, &error))?;
    let target = Target::project(icons_dir, staging.path()).map_err(|error| build_error_in(error, Some(origin)))?;
    let options = BuildOptions {
        icon: Some(id.to_string()),
        jobs,
        engine_script: EXPORT_SCRIPT.as_bytes(),
        progress: Some(progress),
        ..BuildOptions::default()
    };
    let report = build(&target, Mode::Full(exporter), &options).map_err(|error| build_error_in(error, Some(origin)))?;
    let mut record = report
        .set_json
        .ok_or_else(|| CliError::new(ErrorCode::Internal, format!("{id}: the build made no records")))?;
    let package = |error: gai_package::PackageError| {
        CliError::new(ErrorCode::Internal, format!("{id}: packaging failed: {error}"))
    };
    let archives =
        build_set_archives(&mut record, staging.path(), ActionList::Animations, ZipDate::SETS).map_err(package)?;
    let write = |name: String, bytes: &[u8]| write_inside(staging.path(), &name, bytes).map(drop);
    for archive in &archives {
        write(format!("{}.lottie", archive.name), &archive.lottie)?;
        write(format!("{}.manifest.json", archive.name), archive.manifest.as_bytes())?;
    }
    write("set.json".into(), encode_set_json(&record).as_bytes())?;
    snapshot(icons_dir, staging.path(), id, modules)?;

    let final_dir = inside(root, &format!("{BUILD_DIR}/{id}"))?;
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir).map_err(|error| CliError::io("remove", &final_dir, &error))?;
    }
    let staged = staging.keep();
    std::fs::rename(&staged, &final_dir).map_err(|error| CliError::io("move", &staged, &error))?;
    Built::read(root, id)?
        .ok_or_else(|| CliError::new(ErrorCode::Internal, format!("{id}: the build folder is incomplete")))
}

/// The CLI error of a failed build.
pub fn build_error(error: BuildError) -> CliError {
    build_error_in(error, None)
}

/// The CLI error of a failed build of the set at `origin` (see [`check::source_error_in`]).
pub fn build_error_in(error: BuildError, origin: Option<Origin<'_>>) -> CliError {
    match error {
        BuildError::Export { animation, error } => engine::export_error(&animation, &error),
        BuildError::Js(error) => check::source_error_in(error, origin),
        BuildError::UnknownIcon(name) => {
            CliError::new(ErrorCode::UnknownIcon, format!("No icon named {name} in the definitions")).with_id(name)
        }
        BuildError::Profile(error) => CliError::new(ErrorCode::InvalidConfig, error.to_string()),
        BuildError::Read { path, source } => CliError::io("read", &path, &source),
        BuildError::Write { path, source } => CliError::io("write", &path, &source),
        error
        @ (BuildError::Scene(_) | BuildError::InvalidName(_) | BuildError::Scope(_) | BuildError::Json { .. }) => {
            CliError::new(ErrorCode::SourceError, error.to_string())
        }
        error @ BuildError::PostExport { .. } => CliError::new(ErrorCode::EngineError, error.to_string()),
        error => CliError::new(ErrorCode::Internal, error.to_string()),
    }
}

/// Installs every verified icon as `add` installs a local icon, in one batch.
fn install<'a>(root: &Path, config: &Config, icons: impl Iterator<Item = &'a IconBuild>) -> Result<Vec<String>> {
    let lock = Lock::load(root, config.layout)?;
    let mut next = lock.clone();
    let mut batch = Batch::new();
    stage_settings(&mut batch, config);
    for result in icons {
        let descriptor = local_descriptor(config, &result.id, &result.built)?;
        let mut record = Map::new();
        record.insert("version".into(), json!("local"));
        record.insert("origin".into(), json!("local"));
        stage_icon(&mut batch, config, &result.id, result.built.lottie.clone(), &descriptor, &mut record);
        record.insert("build".into(), build_facts(&result.built));
        next.object_mut("icons").insert(result.id.clone(), Value::Object(record));
    }
    next.stamp(config.layout, None);
    let digests: Vec<(String, String)> =
        batch.writes().map(|(path, bytes)| (path.to_string(), sha256_hex(bytes))).collect();
    let files = next.object_mut("files");
    for (path, sha) in digests {
        files.insert(path, json!(sha));
    }
    batch.write(next.path, next.to_bytes());
    batch.plan(root, &lock.expected())?.apply()
}
