//! Per-set maintainer commands: `library preflight`, `library build` (the checked
//! build: preflight, the pipeline, then the set package), `library verify` and
//! `library package`.
//!
//! `library build` preflights every named set first, then builds the ones that passed;
//! both stages run several sets at once (one at a time when exporting, since exports
//! share the engine's documents). Each set writes only its own folders, and every set
//! is preflighted before any is rewritten, so no preflight reads a half-written
//! `set.json`. Reports keep set order.

use super::{resolve_set, select, sets_dir};
use crate::Context;
use crate::cli::{BuildMode, LibraryBuildArgs, LibraryPackageArgs, LibraryPreflightArgs, LibraryVerifyArgs};
use crate::commands::build::build_error;
use crate::engine;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use gai_build::build::{AnimationReport, BuildOptions, BuildReport, Mode, build_set};
use gai_check::preflight::{CheckedReport, LaunchOutcome, Preflight, Status, Target};
use gai_core::export::Exporter;
use gai_engine::HostedExporter;
use gai_engine::config::MAX_DOCUMENTS;
use gai_engine::export::EXPORT_SCRIPT;
use gai_package::{PackageReport, SetPaths, package_library, package_set};
use serde_json::{Value, json};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

/// Sets preflighted or built at once, at most. Each set build holds its scenes, natives
/// and archives in memory, so the pool stays below the core count of large machines.
const MAX_SET_WORKERS: usize = 8;

/// Native stack of a set worker, as the main thread has (zip and JSON work run on it).
const SET_WORKER_STACK: usize = 8 << 20;

/// Applies `f` to every item on up to `workers` threads; results keep input order.
fn in_parallel<T: Sync, R: Send>(items: &[T], workers: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
    let work = || {
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(item) = items.get(index) else { break };
            let result = f(item);
            *results[index].lock().unwrap_or_else(PoisonError::into_inner) = Some(result);
        }
    };
    std::thread::scope(|scope| {
        let work = &work;
        let handles: Vec<_> = (0..workers.clamp(1, items.len().max(1)))
            .map(|n| {
                std::thread::Builder::new()
                    .name(format!("gai-set-{n}"))
                    .stack_size(SET_WORKER_STACK)
                    .spawn_scoped(scope, work)
                    .expect("spawn a set worker")
            })
            .collect();
        for handle in handles {
            if let Err(panic) = handle.join() {
                std::panic::resume_unwind(panic);
            }
        }
    });
    results
        .into_iter()
        .map(|slot| slot.into_inner().unwrap_or_else(PoisonError::into_inner).expect("every item ran"))
        .collect()
}

/// The CLI error of a preflight that could not run.
fn preflight_error(error: gai_check::preflight::Error) -> CliError {
    match error {
        gai_check::preflight::Error::Js(error) => crate::commands::check::source_error(error),
        gai_check::preflight::Error::Legacy(message) => CliError::new(ErrorCode::SourceError, message),
        other => CliError::new(ErrorCode::SourceError, other.to_string()),
    }
}

fn package_error(set: &str, error: gai_package::PackageError) -> CliError {
    CliError::new(ErrorCode::Io, format!("{set}: packaging failed: {error}"))
}

/// `library preflight <set> | --all`: `preflight.mjs`, byte for byte.
pub fn preflight(repo: &Path, args: &LibraryPreflightArgs) -> Result<Report> {
    let preflight = Preflight::open(repo).map_err(preflight_error)?;
    let target = match (&args.set, args.all) {
        (Some(_), true) => return Err(CliError::usage("Name a set or pass --all, not both")),
        (Some(set), false) => Target::Set(sets_dir(repo).join(resolve_set(repo, set)?)),
        (None, true) => Target::All,
        (None, false) => return Err(CliError::usage("Name a set, or pass --all")),
    };
    let run = preflight.run(&target).map_err(preflight_error)?;
    Ok(Report { failed: run.status() != Status::Pass, ..Report::new(run.to_json(), run.stdout()) })
}

/// What `library build` did to one set.
struct SetBuild {
    report: Option<BuildReport>,
    package: Option<PackageReport>,
    error: Option<CliError>,
}

/// `library build <set>… | --all`: preflight, then the build, then the set package. An
/// export through the hosted engine refuses `--offline` before anything runs.
pub fn build(ctx: &Context, repo: &Path, args: &LibraryBuildArgs) -> Result<Report> {
    if args.mode() == BuildMode::Full {
        engine::require_online(ctx, "library build", "Drop --offline, or pass --reuse-native or --offline-scenes")?;
    }
    let sets = select(repo, &args.sets, args.all, |dir| dir.join("definitions.mjs").is_file())?;
    if let Some(frozen) = sets.iter().find(|set| !sets_dir(repo).join(set).join("definitions.mjs").is_file()) {
        return Err(CliError::usage(format!(
            "{frozen} has no definitions.mjs; frozen sets are verified as they are, never rebuilt"
        )));
    }
    let hosted = match args.mode() {
        BuildMode::Full => {
            let mut config = engine::config(ctx)?;
            config.documents = args.jobs.clamp(1, MAX_DOCUMENTS);
            engine::require_session(&config)?;
            Some(HostedExporter::new(&config).map_err(engine::error)?)
        }
        _ => None,
    };
    let preflight = Preflight::open(repo).map_err(preflight_error)?;
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get).min(MAX_SET_WORKERS);
    // Exports share the engine's documents, so sets export one at a time.
    let build_workers = if hosted.is_some() { 1 } else { workers };
    let dirs: Vec<PathBuf> = sets.iter().map(|set| sets_dir(repo).join(set)).collect();
    let checks = in_parallel(&dirs, workers, |dir| preflight.check_set(dir));
    // As one set at a time would: the sets before the first preflight that could not
    // run are built, then its error is returned.
    let stop = checks.iter().position(Result::is_err).unwrap_or(checks.len());
    let launch = |index: &usize| {
        let (set, dir) = (&sets[*index], &dirs[*index]);
        let mut outcome = SetBuild { report: None, package: None, error: None };
        let mode = match (&hosted, args.mode()) {
            (Some(exporter), _) => Mode::Full(exporter as &dyn Exporter),
            (None, BuildMode::ReuseNative) => Mode::ReuseNative,
            (None, _) => Mode::OfflineScenes,
        };
        let engine_script = match hosted {
            Some(_) => EXPORT_SCRIPT.as_bytes(),
            None => gai_build::template::ENGINE_SCRIPT,
        };
        let progress = |report: &AnimationReport| ctx.out.log(&format!("{set}: {}: {report}", report.name));
        let options =
            BuildOptions { jobs: args.jobs, engine_script, progress: Some(&progress), ..BuildOptions::default() };
        let launched = match build_set(dir, mode, &options) {
            Ok(report) => {
                outcome.report = Some(report);
                match matches!(mode, Mode::OfflineScenes) {
                    true => LaunchOutcome::built(),
                    false => match SetPaths::new(repo, set).and_then(|paths| package_set(&paths)) {
                        Ok(package) => {
                            outcome.package = Some(package);
                            LaunchOutcome::built()
                        }
                        Err(error) => {
                            outcome.error = Some(package_error(set, error));
                            LaunchOutcome::failed()
                        }
                    },
                }
            }
            Err(error) => {
                outcome.error = Some(build_error(error));
                LaunchOutcome::failed()
            }
        };
        (launched, outcome)
    };
    let passed: Vec<usize> = (0..stop)
        .filter(|&index| checks[index].as_ref().is_ok_and(|report| report.report.status == Status::Pass))
        .collect();
    let mut launched: Vec<Option<(LaunchOutcome, SetBuild)>> = (0..stop).map(|_| None).collect();
    for (index, result) in passed.iter().zip(in_parallel(&passed, build_workers, launch)) {
        launched[*index] = Some(result);
    }
    let mut checks = checks.into_iter();
    let mut results = Vec::new();
    let mut failed = false;
    for build in launched {
        let report = checks.next().expect("one check per set").map_err(preflight_error)?;
        let (launch, outcome) = match build {
            Some((launch, outcome)) => (Some(launch), outcome),
            None => (None, SetBuild { report: None, package: None, error: None }),
        };
        let checked = CheckedReport { report, launch };
        failed |= checked.status() != Status::Pass;
        let mut value = checked.to_json();
        if let Value::Object(map) = &mut value {
            if let Some(report) = &outcome.report {
                let actions: Vec<Value> = report
                    .animations
                    .iter()
                    .map(|a| json!({ "name": a.name, "label": a.label, "motionLabel": a.motion_label, "layers": a.layers }))
                    .collect();
                map.insert("actions".into(), Value::Array(actions));
            }
            if let Some(package) = &outcome.package {
                map.insert("package".into(), package_json(package));
            }
            if let Some(error) = &outcome.error {
                map.insert("buildError".into(), error.to_json()["error"].clone());
            }
        }
        results.push(value);
    }
    if let Some(error) = checks.next() {
        return Err(preflight_error(error.expect_err("the first check that could not run")));
    }
    let status = if failed { "FAIL" } else { "PASS" };
    let mut text = String::new();
    for result in &results {
        let field = |key: &str| result.get(key).map(crate::js::to_js_string).unwrap_or_default();
        let _ = writeln!(
            text,
            "{} {}: {} actions",
            field("status"),
            field("set"),
            result.get("actions").and_then(Value::as_array).map_or(0, Vec::len)
        );
        if let Some(error) = result.pointer("/buildError/message").and_then(Value::as_str) {
            let _ = writeln!(text, "  {error}");
        }
    }
    Ok(Report { failed, ..Report::new(json!({ "status": status, "sets": results }), text) })
}

fn package_json(package: &PackageReport) -> Value {
    json!({
        "archives": package.archives.len(),
        "verifiedImages": package.verified_images,
        "zip": package.zip.display().to_string(),
        "zipBytes": package.zip_bytes,
    })
}

/// `library verify <set>… | --all [--paint]`: the set gates of `verify.mjs`, writing
/// their evidence; `--paint` (implied by `--all`) also audits every catalog paint.
pub fn verify(repo: &Path, args: &LibraryVerifyArgs) -> Result<Report> {
    let paint_only = args.paint && !args.all && args.sets.is_empty();
    let sets = match paint_only {
        true => Vec::new(),
        false => select(repo, &args.sets, args.all, |dir| {
            let folder = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let number = folder.get(..2).unwrap_or_default();
            repo.join("prototype/icons").join(number).join("assets/set.json").is_file()
        })?,
    };
    let options = gai_check::verify::VerifyOptions {
        builder_identity_sha256: Some(gai_build::records::builder_identity_sha256()),
    };
    let mut results = Vec::new();
    let mut failed = false;
    let mut text = String::new();
    for set in &sets {
        let dir = sets_dir(repo).join(set);
        let outcome = gai_check::verify::verify_set(&gai_check::verify::SetPaths::catalog(&dir), &options);
        let written = crate::commands::check::write_evidence(&outcome, &dir);
        let error = outcome.error.as_ref().map(ToString::to_string).or_else(|| written.err().map(|e| e.message));
        failed |= error.is_some();
        for line in &outcome.summary {
            let _ = writeln!(text, "{line}");
        }
        if let Some(error) = &error {
            let _ = writeln!(text, "FAIL {set}: {error}");
        }
        results.push(json!({
            "set": set,
            "status": if error.is_none() { "PASS" } else { "FAIL" },
            "summary": outcome.summary,
            "evidence": outcome.evidence.iter().map(|e| format!("motion/icon-sets/{set}/{}", e.path)).collect::<Vec<_>>(),
            "error": error,
        }));
    }
    let mut value = json!({ "status": "PASS", "sets": results });
    if args.paint || args.all {
        let contract = gai_check::paint::PaintContract::builtin();
        let paint = gai_check::paint::audit_catalog(&repo.join("prototype/icons"), contract)
            .map_err(|error| CliError::new(ErrorCode::CheckFailed, format!("Paint audit failed: {error}")))?;
        crate::managed::write_inside(repo, gai_check::paint::EVIDENCE_FILE, paint.to_json().as_bytes())?;
        let _ = writeln!(text, "{}", paint.summary());
        value["paint"] = json!({ "icons": paint.icons, "actions": paint.actions, "strokePaints": paint.stroke_paints, "evidence": gai_check::paint::EVIDENCE_FILE });
    }
    value["status"] = json!(if failed { "FAIL" } else { "PASS" });
    Ok(Report { failed, ..Report::new(value, text) })
}

/// `library package <set>… | --icon-library`: set packages, or `icon-library.zip`.
pub fn package(repo: &Path, args: &LibraryPackageArgs) -> Result<Report> {
    if args.icon_library {
        if !args.sets.is_empty() {
            return Err(CliError::usage("Name sets or pass --icon-library, not both"));
        }
        let proof = package_library(repo, None).map_err(|error| package_error("library", error))?;
        let text = format!("{}\n", gai_core::to_js_json_pretty(&proof.to_json(), 2));
        return Ok(Report::new(proof.to_json(), text));
    }
    let sets = select(repo, &args.sets, false, |_| true)?;
    let mut results = Vec::new();
    let mut text = String::new();
    for set in &sets {
        let paths = SetPaths::new(repo, set).map_err(|error| package_error(set, error))?;
        let report = package_set(&paths).map_err(|error| package_error(set, error))?;
        for archive in &report.archives {
            let _ = writeln!(text, "{archive}");
        }
        let mut value = package_json(&report);
        value["set"] = json!(set);
        results.push(value);
    }
    Ok(Report::new(json!({ "sets": results }), text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_results_keep_input_order() {
        let items: Vec<u64> = (0..40).collect();
        let squares = in_parallel(&items, 6, |n| {
            // Later items finish first.
            std::thread::sleep(std::time::Duration::from_micros(400 - n * 10));
            n * n
        });
        assert_eq!(squares, items.iter().map(|n| n * n).collect::<Vec<_>>());
        assert!(in_parallel(&Vec::<u8>::new(), 4, |n| *n).is_empty());
    }
}
