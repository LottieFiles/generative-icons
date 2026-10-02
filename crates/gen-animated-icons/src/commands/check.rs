//! `check`: offline gates for a project. `--project` validates the config and lock,
//! `--source` preflights the local icon source (kit hashes, definitions, the hold and
//! pairing checks verification runs after an export, library id collisions against the
//! cached catalog) and, with no icon named, validates every spec v2 icon (the `spec` check),
//! `--descriptors` validates installed descriptors, and `--native` verifies each local icon's
//! build, v1 or v2. With no selector every check runs, and icons not built yet are skipped by
//! the native check, not failed, so `check <id>` works before the first build and verifies the
//! build after it.

use crate::Context;
use crate::cli::CheckArgs;
use crate::config::{Config, Layout};
use crate::descriptor::read_installed;
use crate::error::{CliError, ErrorCode, Result};
use crate::kit;
use crate::local_set::{BUILD_DIR, LocalSet, build_dir, changed_source, trace};
use crate::lock::Lock;
use crate::managed::{inside, write_inside};
use crate::output::Report;
use gai_check::preflight::{Code, Detail, Diagnostic, FamilyContract, check_definitions};
use gai_js::{Limits, SetRuntime, SetSource};
use serde_json::{Map, Value, json};
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The `scope` sentence of every check report.
pub const SCOPE: &str = "Source and contract checks; not a visual review";
/// The finding of a build older than its source.
const STALE_BUILD: &str = "STALE_BUILD";

/// The outcome of one check.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    /// `project`, `source`, `descriptors` or `native`.
    pub id: &'static str,
    /// `PASS`, `FAIL`, `SKIP`, or `STALE` (the native check of a build older than its
    /// source, which nothing else failed).
    pub status: &'static str,
    /// Findings, in order.
    pub diagnostics: Vec<Value>,
    /// Extra fields of this check's report.
    pub extra: Map<String, Value>,
}

impl Check {
    fn new(id: &'static str, diagnostics: Vec<Value>) -> Check {
        let failed = diagnostics.iter().any(|d| d.get("severity").and_then(Value::as_str) == Some("error"));
        Check { id, status: if failed { "FAIL" } else { "PASS" }, diagnostics, extra: Map::new() }
    }

    fn skip(id: &'static str, reason: &str) -> Check {
        Check { id, status: "SKIP", diagnostics: vec![finding("info", "SKIPPED", None, reason)], extra: Map::new() }
    }

    /// `{id, status, …extra, diagnostics}`.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("id".into(), json!(self.id));
        map.insert("status".into(), json!(self.status));
        map.extend(self.extra.clone());
        map.insert("diagnostics".into(), Value::Array(self.diagnostics.clone()));
        Value::Object(map)
    }
}

/// `{severity, code, icon?, message}`.
fn finding(severity: &str, code: &str, icon: Option<&str>, message: &str) -> Value {
    let mut map = Map::new();
    map.insert("severity".into(), json!(severity));
    map.insert("code".into(), json!(code));
    if let Some(icon) = icon {
        map.insert("icon".into(), json!(icon));
    }
    map.insert("message".into(), json!(message));
    Value::Object(map)
}

/// `{severity, code, icon?, message, path}`: a finding about one project file.
fn finding_at(severity: &str, code: &str, icon: Option<&str>, message: &str, path: &str) -> Value {
    let mut value = finding(severity, code, icon, message);
    value["path"] = json!(path);
    value
}

/// Runs `check`.
pub fn run(ctx: &Context, args: &CheckArgs) -> Result<Report> {
    let all = !(args.project || args.source || args.descriptors || args.native);
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    // Spec v2 icons named here are checked by the v2 compiler; v1 is unchanged.
    if !args.ids.is_empty() && (all || args.source) {
        let set = LocalSet::of(&config);
        let specs: Vec<bool> = args.ids.iter().map(|id| super::v2::has_spec(root, &set, id)).collect::<Result<_>>()?;
        if specs.iter().all(|&s| s) {
            return super::v2::check(ctx, &args.ids);
        }
        if specs.iter().any(|&s| s) {
            return Err(CliError::usage("Check v1 icons (batches/*.mjs) and v2 icons (*.icon.json) in separate runs"));
        }
    }
    let mut checks = Vec::new();
    if all || args.project {
        checks.push(project(root, &config)?);
    }
    let set = LocalSet::of(&config);
    let mut warnings = Vec::new();
    if all || args.source {
        // Named icons are all v1 here; with none named, every spec is checked too.
        let specs = match args.ids.is_empty() {
            true => super::v2::spec_ids(root, &set)?,
            false => Vec::new(),
        };
        let v1 = inside(root, &set.file("definitions.mjs"))?.is_file();
        // `check` never waits on the network: library ids come from the cached catalog.
        let offline = Context { offline: true, ..ctx.clone() };
        match (set.exists(root)?, v1 || specs.is_empty()) {
            (true, true) => checks.push(source(&offline, &set, &args.ids, &mut warnings)?.check),
            (false, true) => {
                checks.push(Check::skip("source", "No local icon source; gen-animated-icons create <id> starts one"))
            }
            // Only spec v2 icons: there is no v1 source to preflight.
            (_, false) => {}
        }
        if !specs.is_empty() {
            checks.push(spec_check(root, &set, &specs)?);
        }
    }
    if all || args.descriptors {
        checks.push(descriptors(root, &config, &args.ids)?);
    }
    if all || args.native {
        checks.push(native(root, &set, &args.ids, args.native)?);
    }
    let failed = checks.iter().any(|check| check.status == "FAIL");
    // Builds older than their source: they must be rebuilt before anything relies on them.
    let stale: Vec<&str> = checks
        .iter()
        .flat_map(|check| &check.diagnostics)
        .filter(|d| d.get("code").and_then(Value::as_str) == Some(STALE_BUILD))
        .filter_map(|d| d.get("icon").and_then(Value::as_str))
        .collect();
    let stale = indexmap::IndexSet::<&str>::from_iter(stale).into_iter().collect::<Vec<_>>();
    let status = match (failed, stale.is_empty()) {
        (true, _) => "FAIL",
        (false, false) => "STALE",
        (false, true) => "PASS",
    };
    let mut value =
        json!({ "status": status, "scope": SCOPE, "checks": checks.iter().map(Check::to_json).collect::<Vec<_>>() });
    if let (false, Value::Object(map)) = (stale.is_empty(), &mut value) {
        map.shift_insert(1, "stale".into(), json!(stale));
    }
    let mut text = match status {
        "STALE" => {
            let (were, them) = if stale.len() == 1 { ("it was", "it") } else { ("they were", "them") };
            format!(
                "STALE: {} changed since {were} built; rebuild {them} (gen-animated-icons build {}) before previewing or submitting. {SCOPE}\n",
                stale.join(", "),
                stale.join(" ")
            )
        }
        _ => format!("{status}: {SCOPE}\n"),
    };
    for check in &checks {
        let _ = writeln!(text, "  {:<4}  {}", check.status, check.id);
        for diagnostic in &check.diagnostics {
            let field = |key: &str| diagnostic.get(key).and_then(Value::as_str).unwrap_or("");
            let icon = diagnostic.get("icon").and_then(Value::as_str).map(|i| format!(" {i}")).unwrap_or_default();
            let _ =
                writeln!(text, "        {} {}{icon}: {}", field("severity"), field("code"), message_text(diagnostic));
            if let Some(path) = diagnostic.get("path").and_then(Value::as_str) {
                let line = diagnostic.get("line").map(|line| format!(" (at or after line {line})")).unwrap_or_default();
                let pointer = diagnostic.get("pointer").and_then(Value::as_str).unwrap_or_default();
                let pointer = if pointer.is_empty() { String::new() } else { format!("#{pointer}") };
                let _ = writeln!(text, "          in {path}{pointer}{line}");
            }
            if let Some(hint) = diagnostic.get("hint").and_then(Value::as_str) {
                let _ = writeln!(text, "          hint: {hint}");
            }
        }
    }
    Ok(Report { failed, warnings, ..Report::new(value, text) })
}

fn message_text(diagnostic: &Value) -> String {
    match diagnostic.get("message") {
        Some(Value::String(message)) => message.clone(),
        Some(other) => gai_core::to_js_json(other),
        None => String::new(),
    }
}

/// The `spec` check: each v2 spec of `ids` validated, compiled and linted, as `check <id>`
/// does (see [`spec_findings`]).
fn spec_check(root: &Path, set: &LocalSet, ids: &[String]) -> Result<Check> {
    let results = ids
        .iter()
        .map(|id| super::v2::findings(root, set, id).map(|(path, found, compiled)| (id.clone(), path, found, compiled)))
        .collect::<Result<Vec<_>>>()?;
    Ok(spec_findings(&results))
}

/// The `spec` check of `(id, spec path, findings, compiled)` results: each finding with its
/// icon, the spec's project-relative `path`, the JSON `pointer` into it and its `hint`, and
/// `icons` with each spec's status (`FAIL` when it does not compile).
pub fn spec_findings(results: &[(String, String, gai_spec::Diagnostics, bool)]) -> Check {
    let mut diagnostics = Vec::new();
    let mut icons = Vec::new();
    for (id, path, found, compiled) in results {
        for d in &found.0 {
            let mut finding = finding_at(d.level.as_str(), d.code, Some(id), &d.message, path);
            if !d.path.is_empty() {
                finding["pointer"] = json!(d.path);
            }
            if let Some(hint) = &d.hint {
                finding["hint"] = json!(hint);
            }
            diagnostics.push(finding);
        }
        icons.push(json!({ "id": id, "status": if *compiled { "PASS" } else { "FAIL" }, "spec": path }));
    }
    let mut check = Check::new("spec", diagnostics);
    check.extra.insert("icons".into(), Value::Array(icons));
    check
}

/// Config validity (already loaded) and the lock's file states.
fn project(root: &Path, config: &Config) -> Result<Check> {
    let mut diagnostics = Vec::new();
    if config.layout == Layout::Legacy {
        diagnostics.push(finding(
            "warning",
            "LEGACY_PROJECT",
            None,
            "Set up by lottie-icons; run gen-animated-icons init --migrate",
        ));
    }
    for file in Lock::load(root, config.layout)?.status(root)? {
        if file.status != "unchanged" {
            let message = format!("{} is {}", file.file, file.status);
            diagnostics.push(finding("warning", "MANAGED_FILE", None, &message));
        }
    }
    Ok(Check::new("project", diagnostics))
}

/// A source preflight: the check, the icon names the definitions export and the modules
/// the loader read.
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    /// The `source` check.
    pub check: Check,
    /// Every icon name, in definitions order.
    pub names: Vec<String>,
    /// Every module the loader read below the icons folder, icons-relative and sorted
    /// (`definitions.mjs`, `batches/…`, helpers in subfolders). Empty when the kit failed.
    pub modules: Vec<String>,
}

/// Preflights the project's icon source: the kit copy, `checkDefinitions` on the whole
/// set, the modules it loads, constraint part names, the checks verification would run on
/// each sound icon's scenes ([`crate::scene_checks`]) and collisions with library ids.
/// Findings about an icon carry the `path` of its batch (and a `line` where a callback
/// threw). `ids` limits the diagnostics reported; an id whose batch exists but exports
/// nothing is a diagnostic, an id without a batch `UNKNOWN_ICON`.
pub fn source(ctx: &Context, set: &LocalSet, ids: &[String], warnings: &mut Vec<String>) -> Result<Source> {
    let root = ctx.cwd.as_path();
    set.require(root)?;
    if let Err(error) = kit::verify(root, &set.kit) {
        let mut check = Check::new("source", vec![finding("error", error.code.as_str(), None, &error.message)]);
        check.extra.insert("hint".into(), json!(error.hint));
        return Ok(Source { check, names: Vec::new(), modules: Vec::new() });
    }
    let icons_dir = set.icons_dir(root)?;
    let origin = Origin { root, set_dir: &icons_dir, label: Some(&set.icons) };
    let evaluated = preflight(&icons_dir, origin)?;
    let (modules, strays) = classify_modules(&evaluated.modules, root, &icons_dir, &set.kit_dir(root)?);
    let mut findings = Vec::new();
    for id in ids.iter().filter(|id| !evaluated.names.contains(id)) {
        let batch = set.batch(id);
        if !inside(root, &batch)?.is_file() {
            return Err(CliError::new(ErrorCode::UnknownIcon, format!("{id} is not defined in {}", set.icons))
                .with_id(id)
                .with_hint(format!("Run gen-animated-icons create {id}")));
        }
        let finding = match modules.contains(&format!("batches/{id}.mjs")) {
            true => finding_at(
                "error",
                "ICON_NOT_EXPORTED",
                Some(id),
                &format!("{batch} exports no icon named {id}; draw its parts with make('{id}', …) in its icons array"),
                &batch,
            ),
            false => {
                let definitions = set.file("definitions.mjs");
                let message = format!(
                    "{definitions} does not import {batch}; import its icons there (import {{ icons as {} }} from './batches/{id}.mjs') and spread them into icons",
                    js_identifier(id)
                );
                finding_at("error", "ICON_NOT_EXPORTED", Some(id), &message, &definitions)
            }
        };
        findings.push(finding);
    }
    for stray in &strays {
        let message = format!("{stray} is not part of the kit; keep your own modules in {}/batches", set.icons);
        findings.push(finding_at("error", "UNMANAGED_MODULE", None, &message, stray));
    }
    findings.extend(untraced_imports(set, &icons_dir, &evaluated.names, ids, &modules));
    // The file each icon is drawn in: its own batch, else the definitions.
    let icon_path = |icon: &str| -> Result<String> {
        let batch = set.batch(icon);
        Ok(match inside(root, &batch)?.is_file() {
            true => batch,
            false => set.file("definitions.mjs"),
        })
    };
    for facts in &evaluated.facts {
        for part in &facts.unknown_constraint_parts {
            let message = format!(
                "constraints part {part} is not a part of {}; its parts are {} (rename constraints along with the parts)",
                facts.name,
                facts.parts.join(", ")
            );
            findings.push(finding_at(
                "error",
                "UNKNOWN_CONSTRAINT_PART",
                Some(&facts.name),
                &message,
                &icon_path(&facts.name)?,
            ));
        }
    }
    let report = evaluated.report;
    // What verification would find after an export, for each selected icon whose source is
    // sound, while nothing set-wide is wrong.
    let errors: Vec<Option<String>> = report
        .diagnostics
        .iter()
        .filter(|d| d.severity().as_str() == "error")
        .map(|d| d.icon.as_ref().map(crate::js::to_js_string))
        .chain(
            findings
                .iter()
                .filter(|f| f.get("severity").and_then(Value::as_str) == Some("error"))
                .map(|f| f.get("icon").and_then(Value::as_str).map(str::to_string)),
        )
        .collect();
    if !errors.contains(&None) {
        let selected: Vec<&String> = evaluated
            .names
            .iter()
            .filter(|name| (ids.is_empty() || ids.contains(name)) && !errors.contains(&Some(name.to_string())))
            .collect();
        match scene_findings(&icons_dir, origin, &selected) {
            Ok(found) => {
                for (icon, finding) in found {
                    findings.push(finding_at("error", finding.code, Some(&icon), &finding.message, &icon_path(&icon)?));
                }
            }
            // The whole set is evaluated, so another icon's error can stop it; that error is
            // reported already. Otherwise it is what `build` would stop at.
            Err(_) if !errors.is_empty() => {}
            Err(error) => {
                let path = error.path.clone().unwrap_or_else(|| set.file("definitions.mjs"));
                findings.push(finding_at("error", error.code.as_str(), None, &error.message, &path));
            }
        }
    }
    let mut diagnostics = report.diagnostics;
    match ctx.library().and_then(|library| library.catalog()) {
        Ok(catalog) => {
            for name in &evaluated.names {
                if let Some(icon) = catalog.get(name) {
                    let owner = icon.descriptor.pointer("/source/set").map(crate::js::to_js_string).unwrap_or_default();
                    diagnostics.push(Diagnostic {
                        code: Code::CatalogIdCollision,
                        icon: Some(json!(name)),
                        part: None,
                        message: Some(json!(format!(
                            "ID already belongs to {owner}; revise there or choose a distinct concept/ID"
                        ))),
                        detail: Detail::None,
                    });
                }
            }
        }
        Err(error) if ctx.offline && error.code == ErrorCode::Network => warnings.push(
            "Library ids were not checked for collisions: no library catalog is cached yet (check stays offline; build checks them online)"
                .into(),
        ),
        Err(error) => warnings.push(format!("Library ids were not checked for collisions: {}", error.message)),
    }
    let selected = |d: &Value| {
        ids.is_empty() || d.get("icon").and_then(Value::as_str).is_none_or(|icon| ids.iter().any(|id| id == icon))
    };
    let mut diagnostics: Vec<Value> = diagnostics.iter().map(Diagnostic::to_json).collect();
    for (index, error) in &evaluated.thrown {
        if let (Some(diagnostic), Some(frame)) = (diagnostics.get_mut(*index), thrown_at(error, Some(origin))) {
            diagnostic["path"] = json!(frame.file);
            diagnostic["line"] = json!(frame.line);
        }
    }
    for diagnostic in &mut diagnostics {
        let icon = diagnostic.get("icon").and_then(Value::as_str).map(str::to_string);
        if diagnostic.get("code").and_then(Value::as_str) == Some("DUOTONE_POLICY")
            && let Some(facts) = evaluated.facts.iter().find(|facts| Some(&facts.name) == icon.as_ref())
        {
            diagnostic["message"] = json!(duotone_policy(facts));
        }
        if let Some(icon) = icon.filter(|_| diagnostic.get("path").is_none()) {
            diagnostic["path"] = json!(icon_path(&icon)?);
        }
    }
    let diagnostics: Vec<Value> = diagnostics.into_iter().chain(findings).filter(|d| selected(d)).collect();
    let mut check = Check::new("source", diagnostics);
    for (key, value) in [("icons", report.icons), ("actions", report.actions), ("samples", report.samples)] {
        check.extra.insert(key.into(), json!(value));
    }
    Ok(Source { check, names: evaluated.names, modules })
}

/// An `UNTRACED_IMPORTS` warning for each icon of `names` (those in `ids`, or all) drawn in
/// its own batch whose imports [`trace`] cannot follow: its build would depend on every
/// module the set loads, and `submit` would refuse it, so this is said before an export.
/// The finding names the import as written, with its file and line.
fn untraced_imports(
    set: &LocalSet,
    icons_dir: &Path,
    names: &[String],
    ids: &[String],
    modules: &[String],
) -> Vec<Value> {
    let mut findings = Vec::new();
    let selected = names.iter().filter(|name| ids.is_empty() || ids.contains(name));
    for name in super::unique(&selected.cloned().collect::<Vec<_>>()) {
        if !modules.contains(&format!("batches/{name}.mjs")) {
            continue;
        }
        let traced = trace(icons_dir, name, modules);
        let Some(untraced) = traced.untraced.filter(|_| !traced.complete) else { continue };
        let path = set.file(&untraced.module);
        let what = match &untraced.import {
            Some((import, _)) => format!(
                "The import `{import}` cannot be traced (an import of a computed name, or an escape in a specifier)"
            ),
            None => format!("{path} cannot be read to trace its imports"),
        };
        let message = format!(
            "{what}, so a build of {name} depends on every module the set loads, and submit refuses it; import each module by a plain string such as './shared/h.mjs'"
        );
        let mut finding = finding_at("warning", "UNTRACED_IMPORTS", Some(name), &message, &path);
        if let Some((_, line)) = untraced.import {
            finding["line"] = json!(line);
        }
        findings.push(finding);
    }
    findings
}

/// The import binding suggested for a batch: `bBellNudge` for `bell-nudge`.
fn js_identifier(id: &str) -> String {
    let mut name = String::from("b");
    name.extend(id.split('-').map(|word| {
        let mut chars = word.chars();
        chars.next().map(|first| first.to_uppercase().chain(chars).collect::<String>()).unwrap_or_default()
    }));
    name
}

/// Splits the loader's modules into icons-relative source paths (sorted) and
/// project-relative modules in the kit folder that are not kit files.
fn classify_modules(loaded: &[PathBuf], root: &Path, icons_dir: &Path, kit_dir: &Path) -> (Vec<String>, Vec<String>) {
    let canonical = |dir: &Path| dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let (icons_dir, kit_dir, root) = (canonical(icons_dir), canonical(kit_dir), canonical(root));
    let slashed =
        |path: &Path| path.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
    let (mut modules, mut strays) = (Vec::new(), Vec::new());
    for path in loaded {
        if let Ok(relative) = path.strip_prefix(&icons_dir) {
            modules.push(slashed(relative));
        } else if let Ok(relative) = path.strip_prefix(&kit_dir) {
            let kit_file = kit::files().iter().any(|file| Path::new(file.name) == relative);
            if !kit_file {
                strays.push(path.strip_prefix(&root).map_or_else(|_| path.display().to_string(), slashed));
            }
        }
    }
    modules.sort();
    modules.dedup();
    strays.sort();
    strays.dedup();
    (modules, strays)
}

/// What the preflight runtime found.
struct Evaluated {
    report: gai_check::preflight::Report,
    names: Vec<String>,
    modules: Vec<PathBuf>,
    facts: Vec<IconFacts>,
    /// `(index into the report's diagnostics, the error)` for sampling failures whose
    /// callback throws the same message again when called alone, which locates the throw.
    thrown: Vec<(usize, gai_js::Error)>,
}

/// Sampling failures located at most, so a broken helper shared by many parts stays quick.
const MAX_LOCATED: usize = 16;

/// `checkDefinitions` on the set in `icons_dir`, the icon names it exports, the modules it
/// loaded, facts about each icon and where its sampling failures were thrown. Checks the
/// engine's `Math` first.
fn preflight(icons_dir: &Path, origin: Origin<'_>) -> Result<Evaluated> {
    crate::math::verify()?;
    let invalid = |message: String| {
        CliError::new(ErrorCode::Internal, format!("The embedded family contract is invalid: {message}"))
    };
    let contract_json: Value =
        serde_json::from_str(kit::family_contract()).map_err(|error| invalid(error.to_string()))?;
    let contract = FamilyContract::from_json(&contract_json).map_err(invalid)?;
    let source = SetSource::user_project(icons_dir);
    let limits = Limits::default();
    let located = |error| source_error_in(error, Some(origin));
    gai_js::run_on_worker(&limits, || {
        let runtime = SetRuntime::load(&source, &limits).map_err(located)?;
        let report = check_definitions(&runtime, &contract).map_err(|error| match error {
            gai_check::preflight::Error::Js(error) => located(error),
            other => CliError::new(ErrorCode::SourceError, other.to_string()),
        })?;
        let names = runtime.with_icons(icon_names).map_err(located)?;
        let facts = runtime.with_icons(icon_facts).map_err(located)?;
        let mut thrown = Vec::new();
        for (index, diagnostic) in report.diagnostics.iter().enumerate() {
            if thrown.len() == MAX_LOCATED {
                break;
            }
            if let Some(error) = throw_again(&runtime, &contract, diagnostic) {
                thrown.push((index, error));
            }
        }
        Ok(Evaluated { report, names, modules: runtime.modules(), facts, thrown })
    })
}

/// Calls the callbacks of the part an `INVALID_GEOMETRY` sample names (`d`, `visibility`,
/// then `pose.transform`) with the sample's time, direction and width, and returns the
/// error of the first that throws the diagnostic's message: gai-js keeps its stack.
fn throw_again(runtime: &SetRuntime, contract: &FamilyContract, diagnostic: &Diagnostic) -> Option<gai_js::Error> {
    use gai_js::rquickjs::Value as Js;
    let Detail::Sample { profile, reverse, weight, time } = &diagnostic.detail else { return None };
    let (Some(icon), Some(part), Some(message)) = (
        diagnostic.icon.as_ref().and_then(Value::as_str),
        diagnostic.part.as_deref(),
        diagnostic.message.as_ref().and_then(Value::as_str),
    ) else {
        return None;
    };
    if diagnostic.code != Code::InvalidGeometry {
        return None;
    }
    let (strong, reverse, time) = (profile == "strong", *reverse, *time);
    let width = contract.outline_weights.iter().find(|(name, _)| name == weight).map(|(_, width)| *width);
    let error = runtime
        .with_icons(|scope| {
            let Some(icon) = named(scope.icons(), icon)? else { return Ok(()) };
            let Some(part) = named(&icon.get::<_, Js>("parts")?, part)? else { return Ok(()) };
            if let Some(d) = part.get::<_, Js>("d")?.into_function() {
                let width = match width {
                    Some(width) => Js::new_number(scope.ctx().clone(), width),
                    None => part.get::<_, Js>("width")?,
                };
                d.call::<_, Js>((time, strong, reverse, width))?;
            }
            if let Some(visibility) = part.get::<_, Js>("visibility")?.into_function() {
                visibility.call::<_, Js>((time, strong, reverse))?;
            }
            if let Some(pose) = part.get::<_, Js>("pose")?.into_object()
                && let Some(transform) = pose.get::<_, Js>("transform")?.into_function()
            {
                transform.call::<_, Js>((time, strong, reverse))?;
            }
            Ok(())
        })
        .err()?;
    matches!(&error, gai_js::Error::Script { message: thrown, .. } if thrown == message).then_some(error)
}

/// `(icon, finding)` for what set verification would report about each of `names` after
/// an export ([`crate::scene_checks`]), from the scenes `build` would write: the set is
/// evaluated afresh, as `build` does, with the project profile.
fn scene_findings(
    icons_dir: &Path,
    origin: Origin<'_>,
    names: &[&String],
) -> Result<Vec<(String, crate::scene_checks::Finding)>> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let config = gai_build::profile::SetConfig::load(icons_dir)
        .map_err(|error| CliError::new(ErrorCode::InvalidConfig, error.to_string()))?;
    let continuous = gai_build::profile::Profile::project(&config).continuous;
    // The Duotone declaration only refuses a set (the preflight reports it per icon), so it
    // is left out here: another icon's mismatch must not hide this one's findings.
    let source = match SetSource::user_project(icons_dir) {
        SetSource::Definitions { set_dir, library_dir, profile } => SetSource::Definitions {
            set_dir,
            library_dir,
            profile: gai_js::ModernProfile { check_duotone: false, ..profile },
        },
        legacy => legacy,
    };
    let limits = Limits::default();
    let scene_error = |error: gai_build::scene::SceneError| match error {
        gai_build::scene::SceneError::Js(error) => source_error_in(error, Some(origin)),
        other => CliError::new(ErrorCode::SourceError, other.to_string()),
    };
    gai_js::run_on_worker(&limits, || {
        let mut runtime = SetRuntime::load(&source, &limits).map_err(|error| source_error_in(error, Some(origin)))?;
        let description = runtime.install().map_err(|error| source_error_in(error, Some(origin)))?;
        let mut findings = Vec::new();
        for (index, icon) in description.icons.iter().enumerate().filter(|(_, icon)| names.contains(&&icon.name)) {
            let scenes = gai_build::scene::build_icon_scenes(&runtime, &description, index).map_err(scene_error)?;
            let found = crate::scene_checks::check_icon(&description, index, &scenes, continuous)
                .map_err(|error| CliError::new(ErrorCode::Internal, format!("{}: {error}", icon.name)))?;
            findings.extend(found.into_iter().map(|finding| (icon.name.clone(), finding)));
        }
        Ok(findings)
    })
}

/// The string `name` of each exported icon.
fn icon_names(scope: &gai_js::JsScope<'_>) -> gai_js::rquickjs::Result<Vec<String>> {
    let mut names = Vec::new();
    if let Some(icons) = scope.icons().as_array() {
        for icon in icons.iter::<gai_js::rquickjs::Value>() {
            let name = match icon?.as_object() {
                Some(object) => object.get::<_, gai_js::rquickjs::Value>("name")?,
                None => continue,
            };
            if let Some(name) = name.as_string() {
                names.push(name.to_string()?);
            }
        }
    }
    Ok(names)
}

/// The object in the JS array `list` whose `name` is `name`.
fn named<'js>(
    list: &gai_js::rquickjs::Value<'js>,
    name: &str,
) -> gai_js::rquickjs::Result<Option<gai_js::rquickjs::Object<'js>>> {
    use gai_js::rquickjs::Value as Js;
    let Some(items) = list.as_array() else { return Ok(None) };
    for item in items.iter::<Js>() {
        if let Some(item) = item?.into_object()
            && item.get::<_, Js>("name")?.as_string().map(|s| s.to_string()).transpose()?.as_deref() == Some(name)
        {
            return Ok(Some(item));
        }
    }
    Ok(None)
}

/// What `check` reports about one exported icon beyond the preflight.
struct IconFacts {
    name: String,
    /// The authored part names.
    parts: Vec<String>,
    /// String `constraints[].part` values that name none of the parts. Verification looks
    /// constraint parts up by name after export, so a stale one fails there with no
    /// pointer back to the source.
    unknown_constraint_parts: Vec<String>,
    /// The first part with style `tint`.
    tint: Option<String>,
    /// `duotoneFill`, when it is a boolean.
    duotone_fill: Option<bool>,
}

/// [`IconFacts`] of each exported icon with a string name.
fn icon_facts(scope: &gai_js::JsScope<'_>) -> gai_js::rquickjs::Result<Vec<IconFacts>> {
    use gai_js::rquickjs::{Object, Value as Js};
    let strings = |object: &Object<'_>, list: &str, key: &str| -> gai_js::rquickjs::Result<Vec<String>> {
        let mut out = Vec::new();
        let Some(items) = object.get::<_, Js>(list)?.into_array() else { return Ok(out) };
        for item in items.iter::<Js>() {
            if let Some(value) = item?.into_object().map(|item| item.get::<_, Js>(key)).transpose()?
                && let Some(text) = value.as_string()
            {
                out.push(text.to_string()?);
            }
        }
        Ok(out)
    };
    let mut facts = Vec::new();
    let Some(icons) = scope.icons().as_array() else { return Ok(facts) };
    for icon in icons.iter::<Js>() {
        let Some(icon) = icon?.into_object() else { continue };
        let Some(name) = icon.get::<_, Js>("name")?.as_string().map(|s| s.to_string()).transpose()? else { continue };
        let parts = strings(&icon, "parts", "name")?;
        let unknown_constraint_parts =
            strings(&icon, "constraints", "part")?.into_iter().filter(|part| !parts.contains(part)).collect();
        let mut tint = None;
        if let Some(items) = icon.get::<_, Js>("parts")?.into_array() {
            for part in items.iter::<Js>() {
                let Some(part) = part?.into_object() else { continue };
                let style = part.get::<_, Js>("style")?.as_string().map(|s| s.to_string()).transpose()?;
                if style.as_deref() == Some("tint") {
                    tint = part.get::<_, Js>("name")?.as_string().map(|s| s.to_string()).transpose()?;
                    break;
                }
            }
        }
        let duotone_fill = icon.get::<_, Js>("duotoneFill")?.as_bool();
        facts.push(IconFacts { name, parts, unknown_constraint_parts, tint, duotone_fill });
    }
    Ok(facts)
}

/// The `DUOTONE_POLICY` message with what disagrees and how to fix it.
fn duotone_policy(facts: &IconFacts) -> String {
    let base = "Declared Duotone policy disagrees with tint parts";
    match (&facts.tint, facts.duotone_fill) {
        (Some(part), _) => format!(
            "{base}: {part} has style tint, but duotoneFill is not true; hint: set duotoneFill: true when a tint part fills a real enclosed surface, else draw no tint part"
        ),
        (None, Some(true)) => format!(
            "{base}: duotoneFill is true, but no part has style tint; hint: set duotoneFill: false, or tint the enclosed surface (framed() draws one)"
        ),
        (None, _) => format!(
            "{base}: duotoneFill must be true or false; hint: set duotoneFill: false, as no part has style tint"
        ),
    }
}

/// Where a set's authoring sources live, for locating the errors they raise.
#[derive(Debug, Clone, Copy)]
pub struct Origin<'a> {
    /// Paths in messages are made relative to this folder.
    pub root: &'a Path,
    /// The set's own sources: a stack frame here is reported before one in the kit.
    pub set_dir: &'a Path,
    /// The set's name in messages, in place of the folder name gai-js uses.
    pub label: Option<&'a str>,
}

/// The CLI error of a definitions module that fails to load or run.
pub fn source_error(error: gai_js::Error) -> CliError {
    source_error_in(error, None)
}

/// Where authoring code threw: the project-relative file of the innermost stack frame in
/// the set's own sources (else under the origin's root, else the innermost), its line, and
/// whether it is the parser's position (a syntax error) rather than a function's frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thrown {
    /// `/`-separated, relative to the origin's root when inside it.
    pub file: String,
    /// QuickJS columns are unreliable. A syntax error has the right line; a thrown error
    /// names where its statement or call starts, at or before it.
    pub line: u32,
    pub parse: bool,
}

/// The [`Thrown`] place of a `Script` error with a stack, or of a call stopped at its time
/// limit where it stopped.
pub fn thrown_at(error: &gai_js::Error, origin: Option<Origin<'_>>) -> Option<Thrown> {
    let stack = match error {
        gai_js::Error::Script { stack: Some(stack), .. }
        | gai_js::Error::Timeout { stack: Some(stack), .. }
        | gai_js::Error::Budget { stack: Some(stack), .. } => stack,
        _ => return None,
    };
    let canonical = |dir: &Path| dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let root = origin.map(|origin| canonical(origin.root));
    let set_dir = origin.map(|origin| canonical(origin.set_dir));
    let frames = stack_frames(stack);
    let frame = frames
        .iter()
        .find(|frame| set_dir.as_deref().is_some_and(|dir| frame.file.starts_with(dir)))
        .or_else(|| frames.iter().find(|frame| root.as_deref().is_some_and(|root| frame.file.starts_with(root))))
        .or(frames.first())?;
    Some(Thrown { file: relative_to(root.as_deref(), &frame.file), line: frame.line, parse: frame.parse })
}

/// `path` relative to `root` when inside it, `/`-separated.
fn relative_to(root: Option<&Path>, path: &Path) -> String {
    match root.and_then(|root| path.strip_prefix(root).ok()) {
        Some(inner) => inner.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"),
        None => path.display().to_string(),
    }
}

/// The CLI error of authoring code that fails to load or run. With an `origin`, the
/// message names the file and line where it threw or was stopped ([`thrown_at`]),
/// relative to the origin's root, and `path` is that file. A call stopped where no stack
/// was taken names the batch of the icon it was running, when the set has one.
pub fn source_error_in(error: gai_js::Error, origin: Option<Origin<'_>>) -> CliError {
    if matches!(error, gai_js::Error::UnsupportedCpu) {
        return cpu_error(error);
    }
    let root = origin.map(|origin| origin.root.canonicalize().unwrap_or_else(|_| origin.root.to_path_buf()));
    let set_label = match &error {
        gai_js::Error::Script { set, .. }
        | gai_js::Error::Timeout { set, .. }
        | gai_js::Error::Budget { set, .. }
        | gai_js::Error::Module { set, .. }
        | gai_js::Error::Engine { set, .. } => Some(set.clone()),
        _ => None,
    };
    let mut message = error.to_string();
    if let (Some(set), Some(label)) = (&set_label, origin.and_then(|origin| origin.label))
        && message.starts_with(set.as_str())
    {
        message = format!("{label}{}", &message[set.len()..]);
    }
    if let Some(root) = &root {
        let prefix = format!("{}{}", root.display(), std::path::MAIN_SEPARATOR);
        message = message.replace(&prefix, "");
    }
    let mut path = None;
    match (&error, thrown_at(&error, origin)) {
        (gai_js::Error::Script { message: thrown, location, .. }, Some(Thrown { file, line, parse })) => {
            message = match parse {
                true => format!("{file}:{line}: {thrown}{location}"),
                false => format!("{file}: {thrown}{location} (at or after line {line})"),
            };
            path = Some(file);
        }
        (
            gai_js::Error::Timeout { set, location, .. } | gai_js::Error::Budget { set, location, .. },
            Some(Thrown { file, line, .. }),
        ) => {
            let full = error.to_string();
            let stopped = full.strip_prefix(&format!("{set}{location}: ")).unwrap_or(&full);
            message = format!("{file}: {stopped}{location} (running at or after line {line})");
            path = Some(file);
        }
        (gai_js::Error::Timeout { location, .. } | gai_js::Error::Budget { location, .. }, None) => {
            let batch = origin.zip(location.icon.as_deref()).and_then(|(origin, icon)| {
                let batch = origin.set_dir.join("batches").join(format!("{icon}.mjs"));
                (crate::local_set::is_icon_name(icon) && batch.is_file()).then_some(batch)
            });
            path = batch.map(|batch| {
                let batch = batch.canonicalize().unwrap_or(batch);
                relative_to(root.as_deref(), &batch)
            });
        }
        (gai_js::Error::Module { importer, .. }, _) if !importer.is_empty() => {
            path = Some(relative_to(root.as_deref(), Path::new(importer)));
        }
        _ => {}
    }
    let hint = match error {
        gai_js::Error::Timeout { .. } | gai_js::Error::Budget { .. } => {
            "The icon code ran too long: look for a loop that never ends or work that grows without bound in the icon source, then run gen-animated-icons check"
        }
        _ => "Fix the icon source, then run gen-animated-icons check",
    };
    let error = CliError::new(ErrorCode::SourceError, message).with_hint(hint);
    match path {
        Some(path) => error.with_path(path),
        None => error,
    }
}

/// The CLI error of a CPU that cannot run the engine's `Math`.
pub fn cpu_error(error: gai_js::Error) -> CliError {
    CliError::new(ErrorCode::Internal, error.to_string()).with_hint(
        "Run on a CPU with FMA (x86-64-v3 or any arm64); on Apple silicon, use the arm64 build, not one under Rosetta",
    )
}

/// One `at …` line of a QuickJS stack.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    /// Absolute path of the module.
    file: PathBuf,
    line: u32,
    /// Where the parser stopped (`at file:line:column`, a syntax error), rather than a
    /// function's frame (`at name (file:line:column)`).
    parse: bool,
}

/// The frames of a QuickJS stack, innermost first. A frame is `at name (file:line:column)`
/// or, for the parser's position, `at file:line:column`; the file is absolute and may
/// itself hold parentheses.
fn stack_frames(stack: &str) -> Vec<Frame> {
    let place = |text: &str, parse: bool| -> Option<Frame> {
        let (rest, column) = text.rsplit_once(':')?;
        let (file, line) = rest.rsplit_once(':')?;
        let file = PathBuf::from(file);
        column.parse::<u32>().ok()?;
        file.is_absolute().then_some(Frame { file, line: line.parse().ok()?, parse })
    };
    stack
        .lines()
        .filter_map(|line| {
            let at = line.trim_start().strip_prefix("at ")?;
            match at.strip_suffix(')') {
                Some(inner) => inner.match_indices('(').find_map(|(open, _)| place(&inner[open + 1..], false)),
                None => place(at, true),
            }
        })
        .collect()
}

/// `validateIcon` on every installed descriptor (or the requested ones).
fn descriptors(root: &Path, config: &Config, ids: &[String]) -> Result<Check> {
    let lock = Lock::load(root, config.layout)?;
    let mut diagnostics = Vec::new();
    let mut checked = 0;
    for (id, record) in lock.icons().filter(|(id, _)| ids.is_empty() || ids.iter().any(|i| i == id)) {
        let Some(source) = record.get("source").and_then(Value::as_str) else { continue };
        checked += 1;
        let problem = match read_installed(root, source) {
            Ok(descriptor) => gai_package::validate_descriptor(&descriptor).err().map(|error| error.to_string()),
            Err(error) => Some(error.message),
        };
        if let Some(message) = problem {
            diagnostics.push(finding("error", "INVALID_DESCRIPTOR", Some(id), &message));
        }
    }
    let mut check = Check::new("descriptors", diagnostics);
    check.extra.insert("descriptors".into(), json!(checked));
    Ok(check)
}

/// Verifies the build of each local icon (or the requested ones): a v1 build by set
/// verification, a v2 build as `build` verified it. A requested icon that is not built yet
/// is an error only when the native check was asked for by name (`--native`); otherwise it
/// is skipped, so `check <id>` also works before a build.
fn native(root: &Path, set: &LocalSet, ids: &[String], explicit: bool) -> Result<Check> {
    let built_now = built_ids(root)?;
    let (built, unbuilt): (Vec<String>, Vec<String>) = match ids.is_empty() {
        true => (built_now, Vec::new()),
        false => ids.iter().cloned().partition(|id| built_now.contains(id)),
    };
    let not_built = |id: &str| format!("{id} is not built yet; gen-animated-icons build {id} builds and verifies it");
    if built.is_empty() && (!explicit || unbuilt.is_empty()) {
        let reason = match unbuilt.as_slice() {
            [] => "Nothing is built yet; gen-animated-icons build <id> builds an icon".to_string(),
            [id] => not_built(id),
            _ => format!("{} are not built yet; gen-animated-icons build builds them", unbuilt.join(", ")),
        };
        return Ok(Check::skip("native", &reason));
    }
    let severity = if explicit { "error" } else { "info" };
    let mut diagnostics: Vec<Value> =
        unbuilt.iter().map(|id| finding(severity, "NOT_BUILT", Some(id), &not_built(id))).collect();
    let icons_dir = set.icons_dir(root)?;
    let kit_dir = set.kit_dir(root)?;
    for id in &built {
        let dir = build_dir(root, id)?;
        let verdict = match super::v2::is_v2_build(&dir, id) {
            true => super::v2::verify_build(root, id),
            false => verify_build(&dir, &kit_dir),
        };
        if let Some(error) = &verdict.error {
            diagnostics.push(finding("error", "VERIFY_FAILED", Some(id), error));
        }
        if let Some(stale) = changed_source(&icons_dir, &dir) {
            let message = format!(
                "The source changed since {id} was built ({}); run gen-animated-icons build {id}",
                stale.describe()
            );
            diagnostics.push(finding("warning", STALE_BUILD, Some(id), &message));
        }
    }
    let mut check = Check::new("native", diagnostics);
    if check.status == "PASS" && check.diagnostics.iter().any(|d| d.get("code").and_then(Value::as_str) == Some(STALE_BUILD)) {
        check.status = "STALE";
    }
    Ok(check)
}

/// Icons with a build folder, sorted.
pub fn built_ids(root: &Path) -> Result<Vec<String>> {
    let dir = inside(root, BUILD_DIR)?;
    let mut ids: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.path().join("set.json").is_file())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| crate::local_set::is_icon_name(name))
            .collect(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(CliError::io("read", &dir, &error)),
    };
    ids.sort();
    Ok(ids)
}

/// A build's verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// Each passing stage's summary line.
    pub summary: Vec<String>,
    /// The failure, if any.
    pub error: Option<String>,
}

/// Runs set verification on one build folder (its `set.json`, scenes, natives, source
/// snapshot and assets) and writes the evidence into it. A folder holding anything a build
/// never writes (a symlink, FIFO, socket or device) fails unread, as verification would
/// follow it.
pub fn verify_build(dir: &Path, kit_dir: &Path) -> Verdict {
    if let Some(entry) = foreign_entry(dir) {
        let shown = entry.strip_prefix(dir).unwrap_or(&entry).display().to_string();
        let error = format!("{shown} in the build is not a regular file or folder, so the build was not read");
        return Verdict { summary: Vec::new(), error: Some(error) };
    }
    let paths = gai_check::verify::SetPaths {
        set_dir: dir.to_path_buf(),
        assets_dir: dir.to_path_buf(),
        library_dir: kit_dir.to_path_buf(),
    };
    let options = gai_check::verify::VerifyOptions {
        builder_identity_sha256: Some(gai_build::records::builder_identity_sha256()),
    };
    let outcome = gai_check::verify::verify_set(&paths, &options);
    let written = write_evidence(&outcome, dir).err().map(|error| format!("Cannot write evidence: {}", error.message));
    Verdict { summary: outcome.summary.clone(), error: outcome.error.as_ref().map(ToString::to_string).or(written) }
}

/// The first entry below `dir` that is neither a regular file nor a folder; an entry that
/// cannot be listed or typed counts as one.
pub fn foreign_entry(dir: &Path) -> Option<PathBuf> {
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else { return Some(folder) };
        for entry in entries {
            let Ok(entry) = entry else { return Some(folder) };
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => folders.push(entry.path()),
                Ok(kind) if kind.is_file() => {}
                _ => return Some(entry.path()),
            }
        }
    }
    None
}

/// Writes a verification's evidence below the set folder `dir`. Each file is renamed into
/// place and the folders below `dir` are checked first, so a symlink planted in a build or
/// set folder is replaced or refused, never written through.
pub fn write_evidence(outcome: &gai_check::verify::Verification, dir: &Path) -> Result<()> {
    for evidence in &outcome.evidence {
        write_inside(dir, &evidence.path, evidence.contents.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn a_build_holding_a_symlink_or_fifo_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let build = dir.path().join("build");
        std::fs::create_dir_all(build.join("native")).unwrap();
        std::fs::write(build.join("set.json"), "{}").unwrap();
        assert_eq!(foreign_entry(&build), None);
        std::os::unix::fs::symlink("/dev/zero", build.join("native/a.json")).unwrap();
        let verdict = verify_build(&build, dir.path());
        assert_eq!(
            verdict.error.as_deref(),
            Some("native/a.json in the build is not a regular file or folder, so the build was not read")
        );
        std::fs::remove_file(build.join("native/a.json")).unwrap();
        assert!(std::process::Command::new("mkfifo").arg(build.join("set.json.fifo")).status().unwrap().success());
        assert_eq!(foreign_entry(&build), Some(build.join("set.json.fifo")));
    }

    #[test]
    fn stack_frames_are_read_innermost_first() {
        let stack = "    at g (/p/icons/batches/shared/h.mjs:12:5)\n    at <anonymous> (/p/icons/batches/a (1).mjs:2:23)\n    at /p/icons/definitions.mjs:4:1\n    at native\n";
        let frame = |file: &str, line, parse| Frame { file: PathBuf::from(file), line, parse };
        assert_eq!(
            stack_frames(stack),
            [
                frame("/p/icons/batches/shared/h.mjs", 12, false),
                frame("/p/icons/batches/a (1).mjs", 2, false),
                frame("/p/icons/definitions.mjs", 4, true),
            ]
        );
    }

    #[test]
    fn source_errors_prefer_the_sets_own_frame() {
        let error = gai_js::Error::Script {
            set: "icons".into(),
            location: Box::new(gai_js::Location { icon: Some("bell".into()), ..Default::default() }),
            message: "boom".into(),
            stack: Some(
                "    at make (/p/library/ai-authoring.mjs:12:8)\n    at <anonymous> (/p/icons/batches/bell.mjs:3:9)\n"
                    .into(),
            ),
        };
        let origin =
            Origin { root: Path::new("/p"), set_dir: Path::new("/p/icons"), label: Some("gen-animated-icons/icons") };
        let cli = source_error_in(error, Some(origin));
        assert_eq!((cli.code, cli.path.as_deref()), (ErrorCode::SourceError, Some("icons/batches/bell.mjs")));
        assert_eq!(cli.message, "icons/batches/bell.mjs: boom (icon bell) (at or after line 3)");
        let syntax = gai_js::Error::Script {
            set: "icons".into(),
            location: Box::default(),
            message: "unexpected token in expression: ';'".into(),
            stack: Some("    at /p/icons/batches/bell.mjs:4:18\n".into()),
        };
        let cli = source_error_in(syntax, Some(origin));
        assert_eq!(cli.message, "icons/batches/bell.mjs:4: unexpected token in expression: ';'");
        let timeout = gai_js::Error::Timeout {
            set: "icons".into(),
            location: Box::default(),
            limit: std::time::Duration::from_secs(120),
            stack: None,
        };
        let cli = source_error_in(timeout, Some(origin));
        assert_eq!(cli.message, "gen-animated-icons/icons: stopped after 120s");
        assert!(cli.hint.unwrap().starts_with("The icon code ran too long"));
        let budget = gai_js::Error::Budget {
            set: "icons".into(),
            location: Box::new(gai_js::Location { icon: Some("bell".into()), ..Default::default() }),
            limit: std::time::Duration::from_secs(600),
            stack: None,
        };
        let cli = source_error_in(budget, Some(origin));
        assert_eq!(
            (cli.code, cli.message.as_str()),
            (
                ErrorCode::SourceError,
                "gen-animated-icons/icons (icon bell): stopped when the set's calls had used their 600s budget"
            )
        );
        assert!(cli.hint.unwrap().starts_with("The icon code ran too long"));
        // Stopped where no stack was taken (the watchdog): the icon's own batch, if any.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("icons/batches")).unwrap();
        std::fs::write(root.join("icons/batches/bell.mjs"), "export const icons = [];\n").unwrap();
        let origin = Origin { root: &root, set_dir: &root.join("icons"), label: Some("icons") };
        let stuck = |icon: &str| gai_js::Error::Timeout {
            set: "icons".into(),
            location: Box::new(gai_js::Location { icon: Some(icon.into()), ..Default::default() }),
            limit: std::time::Duration::from_secs(120),
            stack: None,
        };
        let cli = source_error_in(stuck("bell"), Some(origin));
        assert_eq!(
            (cli.message.as_str(), cli.path.as_deref()),
            ("icons (icon bell): stopped after 120s", Some("icons/batches/bell.mjs"))
        );
        assert_eq!(source_error_in(stuck("other"), Some(origin)).path, None);
        let cpu = source_error(gai_js::Error::UnsupportedCpu);
        assert_eq!(cpu.code, ErrorCode::Internal);
        assert!(cpu.hint.unwrap().contains("not one under Rosetta"));
    }

    #[test]
    fn a_check_fails_only_on_errors() {
        assert_eq!(Check::new("x", vec![finding("warning", "W", None, "w")]).status, "PASS");
        assert_eq!(Check::new("x", vec![finding("error", "E", Some("a"), "e")]).status, "FAIL");
        assert_eq!(
            gai_core::to_js_json(&Check::skip("native", "none").to_json()),
            r#"{"id":"native","status":"SKIP","diagnostics":[{"severity":"info","code":"SKIPPED","message":"none"}]}"#
        );
    }
}
