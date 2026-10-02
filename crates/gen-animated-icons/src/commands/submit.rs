//! `submit`: bundles local icons for review by the library maintainers and sends the
//! bundle to the submissions endpoint, or with `--dry-run` only writes it.
//!
//! The bundle is a deterministic ZIP (stored entries, 1980-01-01 timestamps, names in
//! bytewise order, mode 0644) holding the icon source, each icon's build (assets,
//! `.lottie` files, `set.json`, export provenance, the source snapshot list and evidence),
//! its review sheets, the check results and `submission.json`. Only verified builds of
//! the current source are submitted.
//!
//! The review sheets are the ones `build` writes (Light and Strong at `preview`'s default
//! columns and looks; see [`Review`]), rendered again from the verified build, so a sheet
//! that `preview` overwrote with other options, an older build's sheet, or any other PNG
//! in the previews folder never reaches the bundle. A sheet in the previews folder that is
//! missing or differs is rewritten there once nothing else refuses the submission.
//!
//! A spec v2 icon (`<id>.icon.json`) is bundled the same way, with the spec its build verified
//! (the build's copy, which its snapshot list records and the icons folder still holds) as its
//! source, `checks.json` holding the `spec` check in place of the source preflight, and
//! `"format": "gai-icon/2"` in `submission.json`. v1 and v2 icons are submitted in separate
//! runs.
//!
//! The source is what each build verified, taken from the build's own snapshot: the files
//! its icon depends on (its batch, the modules that batch imports, helpers in subfolders
//! included, and the set's config and rule files), plus a `definitions.mjs` that imports
//! only the submitted icons and the set's evidence. Other icons' batches, unrelated files
//! and edits made since the build stay out. An icon whose imports cannot be traced
//! depends on every module its build loaded, so it is refused before anything is written
//! or sent, unless `--include-untraced` asks for all of them. Nothing is read through a
//! symlink, in the build or in the icons folder: one is refused with `SYMLINK_PATH`. Every
//! entry name is a plain relative path (see [`is_portable_path`]).
//!
//! ```text
//! source/gen-animated-icons/icons/…   definitions, set config, the icons' modules (v2: the
//!                                      specs), evidence
//! built/<id>/…                         assets, .lottie files, set.json, provenance, evidence
//! previews/<id>/<action>.png           the build's review sheets
//! checks.json                          the source and native check results
//! submission.json                      { schemaVersion: 1, ids, format? (v2), cli, kitSha256,
//!                                        engine, files: { <path>: <sha256> }, license, message }
//! ```
//!
//! `submission.json` is JS-JSON indented 2. `files` covers every other entry. `kitSha256`
//! is [`crate::kit::sha256`]: sha256 over one `"<name>\0<sha256>\n"` line per kit file,
//! in bytewise name order.

use super::check::{self, verify_build};
use super::preview::Review;
use crate::Context;
use crate::cli::SubmitArgs;
use crate::config::Config;
use crate::engine;
use crate::error::{CliError, ErrorCode, Result};
use crate::kit;
use crate::local_set::{
    BUILD_DIR, Built, LocalSet, PREVIEW_DIR, SOURCES_FILE, Snapshot, changed_source, definitions,
    generated_definitions, is_icon_name, resolve_relative, scan_imports, trace,
};
use crate::managed::{create_dir_inside, inside, is_portable_path, read_inside, write_replacing};
use crate::output::Report;
use gai_core::sha256_hex;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where submissions go unless `--endpoint` says otherwise.
pub const DEFAULT_ENDPOINT: &str = "https://gen-animated-icons.lottiefiles.com/api/submissions";
/// Folder and file suffixes of source evidence that may be bundled.
const EVIDENCE_SUFFIXES: [&str; 4] = [".md", ".json", ".png", ".webp"];
/// Path components that never enter a bundle.
const PRIVATE: [&str; 3] = [".local", "wip", "unreleased"];
/// Largest submissions response read (it is `{submissionId, status, reviewUrl}`).
const MAX_RESPONSE: u64 = 1 << 20;

/// What a submission bundles before its review sheets, checks and `submission.json`.
struct Gathered {
    files: BTreeMap<String, Vec<u8>>,
    /// The verified builds, in id order.
    builds: Vec<Built>,
    /// The `checks` of `checks.json`.
    checks: Vec<Value>,
    /// `submission.json`'s `format`: `gai-icon/2` for spec v2 icons, none for v1.
    format: Option<&'static str>,
}

/// Runs `submit`.
pub fn run(ctx: &Context, args: &SubmitArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    let set = LocalSet::of(&config);
    let ids: Vec<String> = super::unique(&args.ids).into_iter().map(str::to_string).collect();
    let mut warnings = Vec::new();
    let gathered = match spec_icons(root, &set, &ids)? {
        true => gather_v2(root, &set, &ids)?,
        false => gather(ctx, &set, &ids, args, &mut warnings)?,
    };
    finish(ctx, args, &ids, gathered, warnings)
}

/// Whether `ids` are spec v2 icons (each has a spec); v1 and v2 icons go in separate runs.
fn spec_icons(root: &Path, set: &LocalSet, ids: &[String]) -> Result<bool> {
    let has_spec = |id: &String| {
        super::v2::has_spec(root, set, id).map_err(|error| symlink_error(error, &super::v2::spec_file(set, id)))
    };
    let specs: Vec<bool> = ids.iter().map(has_spec).collect::<Result<_>>()?;
    match (specs.iter().all(|&s| s), specs.iter().any(|&s| s)) {
        (true, true) => Ok(true),
        (false, true) => {
            Err(CliError::usage("Submit v1 icons (batches/*.mjs) and v2 icons (*.icon.json) in separate runs"))
        }
        _ => Ok(false),
    }
}

/// The build of `id`, refused when it holds anything a build never writes, which
/// verification would follow.
fn read_build(root: &Path, id: &str) -> Result<Built> {
    let built = Built::read(root, id)?.ok_or_else(|| {
        CliError::new(ErrorCode::UnknownIcon, format!("{id} is not built"))
            .with_id(id)
            .with_hint(format!("Run gen-animated-icons build {id}"))
    })?;
    if let Some(entry) = check::foreign_entry(&built.dir) {
        let shown = super::preview::relative(root, &entry);
        if std::fs::symlink_metadata(&entry).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(symlink_refused(&shown));
        }
        return Err(CliError::new(ErrorCode::InvalidPath, format!("{shown} is not a regular file or folder"))
            .with_path(shown)
            .with_id(id)
            .with_hint(format!("Run gen-animated-icons build {id}, then submit again")));
    }
    Ok(built)
}

/// `CHECK_FAILED` naming what failed: only verified icons can be submitted.
fn unverified(failures: &[String]) -> CliError {
    CliError::new(
        ErrorCode::CheckFailed,
        format!("Only verified icons can be submitted; failed: {}", failures.join("; ")),
    )
    .with_hint("Fix the icons, run gen-animated-icons build, then submit again")
}

/// v1 icons: the source preflight, each verified build and the source it verified.
fn gather(
    ctx: &Context,
    set: &LocalSet,
    ids: &[String],
    args: &SubmitArgs,
    warnings: &mut Vec<String>,
) -> Result<Gathered> {
    let root = ctx.cwd.as_path();
    set.require(root)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let kit_dir = set.kit_dir(root)?;
    let source = check::source(ctx, set, ids, warnings)?;
    let mut failures = Vec::new();
    if source.check.status == "FAIL" {
        failures.push("source preflight".to_string());
    }
    let mut native = Vec::new();
    let icons_dir = set.icons_dir(root)?;
    let mut builds = Vec::new();
    for id in ids {
        let built = read_build(root, id)?;
        let verdict = verify_build(&built.dir, &kit_dir);
        if let Some(error) = &verdict.error {
            failures.push(format!("{id}: {error}"));
        }
        if let Some(stale) = changed_source(&icons_dir, &built.dir) {
            if stale.refused == Some(ErrorCode::SymlinkPath) {
                return Err(symlink_refused(&format!("{}/{}", set.icons, stale.file)));
            }
            failures.push(format!("{id}: the source changed since it was built ({})", stale.describe()));
        }
        native.push(json!({ "id": id, "status": if verdict.error.is_none() { "PASS" } else { "FAIL" }, "error": verdict.error, "summary": verdict.summary }));
        add_build(&mut files, root, id, &built.dir)?;
        builds.push(built);
    }
    if !failures.is_empty() {
        return Err(unverified(&failures));
    }
    let origin = SourceOrigin { root, icons_dir: &icons_dir, icons: &set.icons };
    add_source(&mut files, origin, &builds, args.include_untraced, warnings)?;
    let checks = vec![source.check.to_json(), json!({ "id": "native", "status": "PASS", "icons": native })];
    Ok(Gathered { files, builds, checks, format: None })
}

/// Spec v2 icons: each verified build, bundled as v1's, and as its source the spec it was built
/// from (the build's own copy, which must be the one its snapshot list records and the one in
/// the icons folder now), plus the set's evidence. The spec is read only as a regular file,
/// never through a symlink, in the build or in the icons folder.
fn gather_v2(root: &Path, set: &LocalSet, ids: &[String]) -> Result<Gathered> {
    let prefix = "source/gen-animated-icons/icons";
    let icons_dir = set.icons_dir(root)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let (mut failures, mut specs, mut native, mut builds) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for id in ids {
        let built = read_build(root, id)?;
        let spec = format!("{id}{}", gai_spec::SUFFIX);
        let built_path = format!("{BUILD_DIR}/{id}/{spec}");
        let rebuild = format!("Run gen-animated-icons build {id}, then submit again");
        let snapshot = Snapshot::read(&built.dir).ok_or_else(|| {
            CliError::new(ErrorCode::CheckFailed, format!("{id}: its build lists no sources ({SOURCES_FILE})"))
                .with_id(id)
                .with_hint(&rebuild)
        })?;
        let bytes = read_regular(&built.dir, &spec, &built_path)?;
        if snapshot.sha256(&spec) != Some(sha256_hex(&bytes).as_str()) {
            return Err(CliError::new(
                ErrorCode::HashMismatch,
                format!("{built_path} differs from the build's snapshot list"),
            )
            .with_id(id)
            .with_path(built_path)
            .with_hint(rebuild));
        }
        no_symlink(&icons_dir, &spec, &format!("{}/{spec}", set.icons))?;
        if let Some(stale) = changed_source(&icons_dir, &built.dir) {
            if stale.refused == Some(ErrorCode::SymlinkPath) {
                return Err(symlink_refused(&format!("{}/{}", set.icons, stale.file)));
            }
            failures.push(format!("{id}: the source changed since it was built ({})", stale.describe()));
        }
        let compiled = std::str::from_utf8(&bytes).ok().map(gai_spec::load);
        let found = match &compiled {
            Some(Ok(compiled)) => {
                let verdict = super::v2::verify(&built, compiled);
                if let Some(error) = &verdict.error {
                    failures.push(format!("{id}: {error}"));
                }
                native.push(json!({ "id": id, "status": if verdict.error.is_none() { "PASS" } else { "FAIL" }, "error": verdict.error, "summary": verdict.summary }));
                compiled.diagnostics.clone()
            }
            Some(Err(diagnostics)) => {
                failures.push(format!("{id}: its spec has errors (gen-animated-icons check {id})"));
                diagnostics.clone()
            }
            None => {
                failures.push(format!("{id}: its spec is not UTF-8"));
                gai_spec::Diagnostics::default()
            }
        };
        specs.push((id.clone(), set.file(&spec), found, matches!(compiled, Some(Ok(_)))));
        add_build(&mut files, root, id, &built.dir)?;
        files.insert(format!("{prefix}/{spec}"), bytes);
        builds.push(built);
    }
    if !failures.is_empty() {
        return Err(unverified(&failures));
    }
    add_evidence(&mut files, SourceOrigin { root, icons_dir: &icons_dir, icons: &set.icons })?;
    let checks = vec![
        check::spec_findings(&specs).to_json(),
        json!({ "id": "native", "status": "PASS", "icons": native }),
    ];
    Ok(Gathered { files, builds, checks, format: Some(gai_spec::FORMAT) })
}

/// The review sheets, `checks.json` and `submission.json`, then the bundle: written, sent, or
/// both.
fn finish(
    ctx: &Context,
    args: &SubmitArgs,
    ids: &[String],
    gathered: Gathered,
    warnings: Vec<String>,
) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let Gathered { mut files, builds, checks, format } = gathered;
    let mut stale_sheets = Vec::new();
    for (id, built) in ids.iter().zip(&builds) {
        for (name, png) in Review::of(&built.icon).pngs(&built.lottie)? {
            let shown = format!("{PREVIEW_DIR}/{id}/{name}");
            // A sheet that is not a regular file is replaced too, never read through.
            if read_inside(root, &shown).ok().flatten().as_deref() != Some(png.as_slice()) {
                stale_sheets.push((id.as_str(), name.clone(), png.clone()));
            }
            files.insert(format!("previews/{id}/{name}"), png);
        }
    }
    let checks = json!({ "status": "PASS", "scope": check::SCOPE, "checks": checks });
    files.insert("checks.json".into(), crate::js::json_file(&checks));

    // From the bundled provenance, never a file read again.
    let engine_label = builds.first().and_then(|built| {
        let proof = files.get(&format!("built/{}/native/{}.json.provenance.json", ids[0], built.strong))?;
        serde_json::from_slice::<Value>(proof).ok().map(|proof| super::add::engine_label(&proof))
    });
    let digests: Map<String, Value> =
        files.iter().map(|(path, bytes)| (path.clone(), json!(sha256_hex(bytes)))).collect();
    let mut submission = json!({
        "schemaVersion": 1,
        "ids": ids,
        "cli": format!("gen-animated-icons@{}", env!("CARGO_PKG_VERSION")),
        "kitSha256": kit::sha256(),
        "engine": engine_label,
        "files": digests,
        "license": args.license.as_str(),
        "message": args.message,
    });
    if let (Some(format), Value::Object(map)) = (format, &mut submission) {
        map.shift_insert(2, "format".into(), json!(format));
    }
    files.insert("submission.json".into(), crate::js::json_file(&submission));
    if let Some(name) = files.keys().find(|name| !is_bundle_name(name)) {
        return Err(unbundlable(name));
    }
    let bundle = write_zip(&files);
    let sha = sha256_hex(&bundle);
    let mut regenerated = Vec::new();
    for (id, name, png) in stale_sheets {
        let dir = create_dir_inside(root, &format!("{PREVIEW_DIR}/{id}"))?;
        write_replacing(&dir.join(&name), &png)?;
        regenerated.push(format!("{PREVIEW_DIR}/{id}/{name}"));
    }

    let output = match &args.output {
        Some(output) => Some(write_new(&ctx.cwd.join(output), &bundle)?),
        None if args.dry_run => {
            let path = inside(root, &format!(".gen-animated-icons/submissions/submission-{}.zip", &sha[..12]))?;
            if !path.exists() {
                write_new(&path, &bundle)?;
            }
            Some(path)
        }
        None => None,
    };
    let output_text = output.as_ref().map(|path| super::preview::relative(root, path));
    let mut value = json!({ "ids": ids, "dryRun": args.dry_run, "bundle": output_text, "sha256": sha, "bytes": bundle.len(), "files": files.len(), "license": args.license.as_str(), "regenerated": regenerated });
    let mut text = String::new();
    if !regenerated.is_empty() {
        let _ = writeln!(text, "Rendered review sheets again from the verified builds: {}", regenerated.join(", "));
    }
    if args.dry_run {
        let _ = writeln!(
            text,
            "Wrote the review bundle for {} ({} files, {} bytes); nothing was sent.",
            ids.join(", "),
            files.len(),
            bundle.len()
        );
    } else {
        let received = send(ctx, args, &bundle, &sha)?;
        let _ = writeln!(
            text,
            "Submitted {} for review: {}.",
            ids.join(", "),
            received.get("reviewUrl").and_then(Value::as_str).unwrap_or("received")
        );
        if let Value::Object(map) = &mut value {
            for (key, field) in received.as_object().into_iter().flatten() {
                map.insert(key.clone(), field.clone());
            }
        }
    }
    if let Some(path) = &output_text {
        let _ = writeln!(text, "Bundle: {path}");
    }
    Ok(Report { warnings, ..Report::new(value, text) })
}

/// POSTs the bundle; the response must be `{submissionId, status, reviewUrl}`.
fn send(ctx: &Context, args: &SubmitArgs, bundle: &[u8], sha: &str) -> Result<Value> {
    if ctx.offline {
        return Err(CliError::new(ErrorCode::Network, "Submitting needs the network")
            .with_hint("Drop --offline, or use --dry-run"));
    }
    let endpoint = args.endpoint.as_deref().unwrap_or(DEFAULT_ENDPOINT);
    crate::library::require_secure_url(endpoint, "submissions")?;
    let config = engine::config(ctx)?;
    engine::require_session(&config)?;
    let network = |detail: String| {
        CliError::new(ErrorCode::Network, format!("Cannot submit to {endpoint}: {detail}"))
            .with_hint("Retry later, or keep the bundle with --dry-run --output bundle.zip")
    };
    // The engine client's TLS stack; it never sends to (or redirects to) plain http elsewhere
    // than loopback, nor from https to http.
    let body = gai_engine::Downloader::new(Duration::from_secs(120))
        .and_then(|client| {
            client.post(endpoint, "application/zip", &[("X-Bundle-Sha256", sha)], bundle.to_vec(), MAX_RESPONSE)
        })
        .map_err(|error| network(error.to_string()))?;
    let received: Value =
        serde_json::from_slice(&body).map_err(|error| network(format!("unexpected response: {error}")))?;
    let mut result = Map::new();
    for key in ["submissionId", "status", "reviewUrl"] {
        result.insert(key.into(), received.get(key).cloned().unwrap_or(Value::Null));
    }
    Ok(Value::Object(result))
}

/// Writes a new file; an existing one is `EXISTS`.
fn write_new(path: &Path, bytes: &[u8]) -> Result<std::path::PathBuf> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|error| CliError::io("create", dir, &error))?;
    let mut temp = tempfile::NamedTempFile::new_in(dir).map_err(|error| CliError::io("create", dir, &error))?;
    temp.write_all(bytes).map_err(|error| CliError::io("write", path, &error))?;
    temp.persist_noclobber(path).map_err(|error| match error.error.kind() {
        std::io::ErrorKind::AlreadyExists => {
            CliError::new(ErrorCode::Exists, format!("{} already exists", path.display()))
                .with_path(path.display().to_string())
                .with_hint("Choose a new --output path")
        }
        _ => CliError::io("write", path, &error.error),
    })?;
    Ok(path.to_path_buf())
}

/// Where a submission's source comes from.
#[derive(Clone, Copy)]
struct SourceOrigin<'a> {
    root: &'a Path,
    /// The project's icons folder, checked by [`inside`].
    icons_dir: &'a Path,
    /// Its project-relative path, for messages.
    icons: &'a str,
}

/// `source/gen-animated-icons/icons/…`: what each build in `builds` verified, read from its
/// snapshot (see [`Snapshot`]): the files its icon depends on, checked against the
/// snapshot list, then a `definitions.mjs` importing only the submitted icons, then the
/// set's evidence. A file is refused when it, or its counterpart in the icons folder, is
/// or runs through a symlink, and so is a bundled module importing through one. A build
/// whose imports cannot be traced again from its snapshot depends on every module it
/// loaded; it is refused unless `untraced` allows it.
fn add_source(
    files: &mut BTreeMap<String, Vec<u8>>,
    origin: SourceOrigin<'_>,
    builds: &[Built],
    untraced: bool,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let prefix = "source/gen-animated-icons/icons";
    let mut sources: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut ids = Vec::new();
    for built in builds {
        let id = built.icon.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
        let rebuild = |message: String| {
            CliError::new(ErrorCode::CheckFailed, message)
                .with_id(&id)
                .with_hint(format!("Run gen-animated-icons build {id}, then submit again"))
        };
        let snapshot = Snapshot::read(&built.dir)
            .ok_or_else(|| rebuild(format!("{id}: its build lists no sources ({SOURCES_FILE})")))?;
        let depends = snapshot.depends.as_ref().ok_or_else(|| {
            rebuild(format!("{id} was built by an earlier version, which did not record the files the icon depends on"))
        })?;
        let own = format!("batches/{id}.mjs");
        if !depends.contains(&own) {
            return Err(CliError::new(
                ErrorCode::CheckFailed,
                format!("{id} is not drawn in its own batch ({}/{own}), so it cannot be submitted alone", origin.icons),
            )
            .with_id(&id)
            .with_hint(format!(
                "Move {id} into {}/{own}, import it in definitions.mjs, rebuild, then submit",
                origin.icons
            )));
        }
        let built_path = |relative: &str| format!("{BUILD_DIR}/{id}/{relative}");
        for relative in depends {
            let bytes = read_regular(&built.dir, relative, &built_path(relative))?;
            if relative == "definitions.mjs" {
                // Replaced below, so it must have been create's form.
                if generated_definitions(&String::from_utf8_lossy(&bytes)).is_none() {
                    return Err(CliError::new(
                        ErrorCode::CheckFailed,
                        format!("{id} was built from a definitions.mjs edited by hand; a submission imports only the submitted icons, so it needs one import per batch as create writes it"),
                    )
                    .with_id(&id)
                    .with_path(format!("{}/definitions.mjs", origin.icons))
                    .with_hint("Restore definitions.mjs to one import line per batch, rebuild, then submit"));
                }
                continue;
            }
            if snapshot.sha256(relative) != Some(sha256_hex(&bytes).as_str()) {
                return Err(CliError::new(
                    ErrorCode::HashMismatch,
                    format!("{} differs from the build's snapshot list", built_path(relative)),
                )
                .with_id(&id)
                .with_path(built_path(relative))
                .with_hint(format!("Run gen-animated-icons build {id}, then submit again")));
            }
            // The source must not reach it through a symlink either.
            no_symlink(origin.icons_dir, relative, &format!("{}/{relative}", origin.icons))?;
            match sources.get(relative) {
                Some(other) if *other != bytes => {
                    return Err(rebuild(format!(
                        "{}/{relative} differs between the builds of the submitted icons",
                        origin.icons
                    )));
                }
                _ => {
                    sources.insert(relative.clone(), bytes);
                }
            }
        }
        let others: Vec<&str> = depends
            .iter()
            .filter(|path| {
                path.strip_prefix("batches/")
                    .and_then(|name| name.strip_suffix(".mjs"))
                    .is_some_and(|name| is_icon_name(name) && name != id)
            })
            .map(String::as_str)
            .collect();
        let including = match others.is_empty() {
            true => String::new(),
            false => format!(", other icons' batches included: {}", others.join(", ")),
        };
        // The build fell back to every module it loaded when it could not trace the icon's
        // imports; tracing the copies it verified again tells.
        if !trace(&built.dir, &id, depends).complete {
            let message = format!(
                "{id}: its imports could not be traced (an import of a computed name, or an escape in a specifier), so its bundle would hold every module its build loaded{including}"
            );
            if !untraced {
                return Err(CliError::new(ErrorCode::CheckFailed, message).with_id(&id).with_hint(format!(
                    "Import each module by a plain string such as './shared/h.mjs', rebuild {id} and submit again; or pass --include-untraced to send every module its build loaded"
                )));
            }
            warnings.push(message);
        } else if !others.is_empty() {
            warnings.push(format!(
                "{id} imports other icons' batches, so the bundle holds them too: {}",
                others.join(", ")
            ));
        }
        ids.push(id);
    }
    for (relative, bytes) in &sources {
        if !relative.ends_with(".mjs") {
            continue;
        }
        let text = String::from_utf8_lossy(bytes);
        let folder = relative.rsplit_once('/').map_or("", |(folder, _)| folder);
        for literal in scan_imports(&text).literals {
            let specifier = &text[literal];
            if !(specifier.starts_with("./") || specifier.starts_with("../")) {
                continue;
            }
            // Outside the icons folder (the kit) or bundled already.
            let Some(target) = resolve_relative(folder, specifier).filter(|target| !sources.contains_key(target))
            else {
                continue;
            };
            match no_symlink(origin.icons_dir, &target, &format!("{}/{target}", origin.icons)) {
                Err(error) if error.code == ErrorCode::SymlinkPath => return Err(error),
                _ => {}
            }
        }
    }
    for (relative, bytes) in sources {
        files.insert(format!("{prefix}/{relative}"), bytes);
    }
    ids.sort();
    files.insert(format!("{prefix}/definitions.mjs"), definitions(&ids).into_bytes());
    add_evidence(files, origin)
}

/// `source/gen-animated-icons/icons/evidence/…`: the set's evidence notes and images.
fn add_evidence(files: &mut BTreeMap<String, Vec<u8>>, origin: SourceOrigin<'_>) -> Result<()> {
    let evidence = inside(origin.icons_dir, "evidence")
        .map_err(|error| symlink_error(error, &format!("{}/evidence", origin.icons)))?;
    add_tree(files, origin.root, &evidence, "source/gen-animated-icons/icons/evidence", |name| {
        EVIDENCE_SUFFIXES.iter().any(|s| name.ends_with(s))
    })
}

/// `SYMLINK_PATH` for `shown`, a path that is or runs through a symlink.
fn symlink_refused(shown: &str) -> CliError {
    CliError::new(
        ErrorCode::SymlinkPath,
        format!("A submission never follows symlinks, and {shown} is or runs through one"),
    )
    .with_path(shown)
    .with_hint("Replace the symlink with the file it points to, rebuild, then submit")
}

/// [`symlink_refused`] for a `SYMLINK_PATH` error of [`inside`]; other errors pass unchanged.
fn symlink_error(error: CliError, shown: &str) -> CliError {
    match error.code {
        ErrorCode::SymlinkPath => symlink_refused(shown),
        _ => error,
    }
}

/// Fails with `SYMLINK_PATH` when `relative` below `dir` is or runs through a symlink.
fn no_symlink(dir: &Path, relative: &str, shown: &str) -> Result<PathBuf> {
    inside(dir, relative).map_err(|error| symlink_error(error, shown))
}

/// Reads `relative` below `dir` when it is a regular file reached without a symlink;
/// `shown` names it in errors.
fn read_regular(dir: &Path, relative: &str, shown: &str) -> Result<Vec<u8>> {
    let path = no_symlink(dir, relative, shown)?;
    match std::fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() => std::fs::read(&path).map_err(|error| CliError::io("read", &path, &error)),
        Ok(_) => Err(CliError::new(ErrorCode::InvalidPath, format!("{shown} is not a file")).with_path(shown)),
        Err(error) => Err(CliError::io("read", &path, &error)),
    }
}

/// `built/<id>/…`: assets, archives, set.json, provenance and evidence of one build.
fn add_build(files: &mut BTreeMap<String, Vec<u8>>, root: &Path, id: &str, dir: &Path) -> Result<()> {
    let prefix = format!("built/{id}");
    add_tree(files, root, dir, &prefix, |name| name.ends_with(".json") || name.ends_with(".lottie"))?;
    add_tree(files, root, &dir.join("native"), &format!("{prefix}/native"), |name| name.ends_with(".provenance.json"))?;
    add_tree(files, root, &dir.join("evidence"), &format!("{prefix}/evidence"), |name| name.ends_with(".json"))
}

/// Adds the regular files directly in `dir` whose names pass `keep`, skipping hidden and
/// private names and folders. A symlink, as `dir` or as a file it would add, is refused.
fn add_tree(
    files: &mut BTreeMap<String, Vec<u8>>,
    root: &Path,
    dir: &Path,
    prefix: &str,
    keep: impl Fn(&str) -> bool,
) -> Result<()> {
    let shown = |path: &Path| super::preview::relative(root, path);
    let symlink = |path: &Path| symlink_refused(&shown(path));
    match std::fs::symlink_metadata(dir) {
        Ok(meta) if meta.file_type().is_symlink() => return Err(symlink(dir)),
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(CliError::io("read", dir, &error)),
    }
    let entries = std::fs::read_dir(dir).map_err(|error| CliError::io("read", dir, &error))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || PRIVATE.contains(&name.to_lowercase().as_str()) || !keep(&name) {
            continue;
        }
        let path = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_symlink() => return Err(symlink(&path)),
            Ok(kind) if kind.is_file() => {}
            _ => continue,
        }
        // A name that is not UTF-8 would be bundled under another one.
        let entry_name = entry.file_name().to_str().map(|name| format!("{prefix}/{name}"));
        let entry_name = entry_name.filter(|name| is_bundle_name(name)).ok_or_else(|| unbundlable(&shown(&path)))?;
        let bytes = std::fs::read(&path).map_err(|error| CliError::io("read", &path, &error))?;
        files.insert(entry_name, bytes);
    }
    Ok(())
}

/// Whether `name` can name a bundle entry: a plain relative path with no backslash, colon,
/// control character or Windows device name (see [`is_portable_path`]) that fits a ZIP
/// header.
fn is_bundle_name(name: &str) -> bool {
    is_portable_path(name) && name.len() <= usize::from(u16::MAX)
}

/// `INVALID_PATH` for `shown`, a file whose name cannot name a bundle entry.
fn unbundlable(shown: &str) -> CliError {
    CliError::new(
        ErrorCode::InvalidPath,
        format!("{shown:?} cannot be bundled: an entry name must be a plain relative path in UTF-8, without backslashes, colons, control characters or device names"),
    )
    .with_path(shown)
    .with_hint("Rename or remove the file, then submit again")
}

/// A stored ZIP: entries in key order, dated 1980-01-01 00:00, mode 0644.
pub fn write_zip(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    const DATE: u16 = (1 << 5) | 1;
    const MADE_BY: u16 = (3 << 8) | 20;
    const MODE: u32 = 0o644 << 16;
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let offset = out.len() as u32;
        let (crc, size, name_len) = (crc32fast::hash(data), data.len() as u32, name.len() as u16);
        let header = |buffer: &mut Vec<u8>, signature: u32, central: bool| {
            buffer.extend_from_slice(&signature.to_le_bytes());
            if central {
                buffer.extend_from_slice(&MADE_BY.to_le_bytes());
            }
            buffer.extend_from_slice(&20u16.to_le_bytes());
            buffer.extend_from_slice(&0u16.to_le_bytes());
            buffer.extend_from_slice(&0u16.to_le_bytes());
            buffer.extend_from_slice(&0u16.to_le_bytes());
            buffer.extend_from_slice(&DATE.to_le_bytes());
            buffer.extend_from_slice(&crc.to_le_bytes());
            buffer.extend_from_slice(&size.to_le_bytes());
            buffer.extend_from_slice(&size.to_le_bytes());
            buffer.extend_from_slice(&name_len.to_le_bytes());
            buffer.extend_from_slice(&0u16.to_le_bytes());
        };
        header(&mut out, 0x0403_4b50, false);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        header(&mut central, 0x0201_4b50, true);
        // Comment length, disk number and internal attributes, then the mode.
        central.extend_from_slice(&[0; 6]);
        central.extend_from_slice(&MODE.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let (central_offset, central_size, count) = (out.len() as u32, central.len() as u32, files.len() as u16);
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&[0; 2]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_set::snapshot;

    /// A project root whose icons folder holds icon `a` (with a helper), icon `b`, and the
    /// build of `a` made from them.
    struct Project {
        dir: tempfile::TempDir,
    }

    impl Project {
        fn new() -> Project {
            let project = Project { dir: tempfile::tempdir().unwrap() };
            project.write("definitions.mjs", &definitions(&["a".into(), "b".into()]));
            project.write("set-config.json", "{}\n");
            project.write("batches/a.mjs", "import {h} from './shared/h.mjs';\nexport const icons = [];\n");
            project.write("batches/shared/h.mjs", "export const h = 1;\n");
            project.write("batches/b.mjs", "export const icons = [];\n");
            project.build();
            project
        }

        fn icons(&self) -> PathBuf {
            self.dir.path().join("icons")
        }

        fn build_dir(&self) -> PathBuf {
            self.dir.path().join(".gen-animated-icons/build/a")
        }

        fn write(&self, relative: &str, text: &str) {
            let file = self.icons().join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }

        fn build(&self) {
            let _ = std::fs::remove_dir_all(self.build_dir());
            std::fs::create_dir_all(self.build_dir()).unwrap();
            let modules =
                ["batches/a.mjs", "batches/b.mjs", "batches/shared/h.mjs", "definitions.mjs"].map(String::from);
            snapshot(&self.icons(), &self.build_dir(), "a", &modules).unwrap();
        }

        /// The source entries of a bundle of `a`, as `(icons-relative path, text)`.
        fn bundle(&self) -> Result<Vec<(String, String)>> {
            self.bundle_with(false, &mut Vec::new())
        }

        /// [`Project::bundle`], allowing an untraced build when `untraced` and keeping the
        /// warnings.
        fn bundle_with(&self, untraced: bool, warnings: &mut Vec<String>) -> Result<Vec<(String, String)>> {
            let built = Built {
                dir: self.build_dir(),
                set: json!({}),
                set_bytes: Vec::new(),
                icon: json!({"name": "a"}),
                strong: "a-strong".into(),
                lottie: Vec::new(),
            };
            let icons = self.icons();
            let origin = SourceOrigin { root: self.dir.path(), icons_dir: &icons, icons: "icons" };
            let mut files = BTreeMap::new();
            add_source(&mut files, origin, &[built], untraced, warnings)?;
            Ok(files
                .into_iter()
                .map(|(name, bytes)| {
                    let name = name.strip_prefix("source/gen-animated-icons/icons/").unwrap().to_string();
                    (name, String::from_utf8(bytes).unwrap())
                })
                .collect())
        }
    }

    #[test]
    fn a_bundle_holds_what_the_build_verified_for_its_icons_only() {
        let project = Project::new();
        // Work after the build: another icon, an unrelated file, an edit to another icon.
        project.write("batches/c.mjs", "export const icons = ['unbuilt'];\n");
        project.write("batches/_scratch.mjs", "export const scratch = 'private';\n");
        project.write("batches/b.mjs", "export const icons = ['edited'];\n");
        project.write("definitions.mjs", &definitions(&["a".into(), "b".into(), "c".into()]));
        project.write("evidence/design-review.md", "# notes\n");
        let names: Vec<(String, String)> = project.bundle().unwrap();
        assert_eq!(
            names,
            [
                ("batches/a.mjs".to_string(), "import {h} from './shared/h.mjs';\nexport const icons = [];\n".into()),
                ("batches/shared/h.mjs".into(), "export const h = 1;\n".into()),
                ("definitions.mjs".into(), definitions(&["a".into()])),
                ("evidence/design-review.md".into(), "# notes\n".into()),
                ("set-config.json".into(), "{}\n".into()),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_bundle_never_reads_through_a_symlink() {
        use std::os::unix::fs::symlink;
        let project = Project::new();
        let secret = project.dir.path().join("secret");
        std::fs::write(&secret, "export const h = 1;\n").unwrap();
        // A symlinked batch nothing imports stays out.
        symlink(&secret, project.icons().join("batches/_leak.mjs")).unwrap();
        assert!(project.bundle().unwrap().iter().all(|(name, _)| name != "batches/_leak.mjs"));

        // A file of the build's snapshot.
        let copy = project.build_dir().join("batches/shared/h.mjs");
        std::fs::remove_file(&copy).unwrap();
        symlink(&secret, &copy).unwrap();
        let error = project.bundle().unwrap_err();
        assert_eq!(
            (error.code, error.path.as_deref()),
            (ErrorCode::SymlinkPath, Some(".gen-animated-icons/build/a/batches/shared/h.mjs"))
        );

        // Its counterpart in the icons folder, or a folder on the way.
        project.build();
        std::fs::rename(project.icons().join("batches/shared"), project.icons().join("batches/real")).unwrap();
        symlink("real", project.icons().join("batches/shared")).unwrap();
        let error = project.bundle().unwrap_err();
        assert_eq!((error.code, error.path.as_deref()), (ErrorCode::SymlinkPath, Some("icons/batches/shared/h.mjs")));

        // A bundled module importing through a symlink the loader resolved.
        std::fs::remove_file(project.icons().join("batches/shared")).unwrap();
        std::fs::rename(project.icons().join("batches/real"), project.icons().join("batches/shared")).unwrap();
        symlink("shared/h.mjs", project.icons().join("batches/link.mjs")).unwrap();
        project.write("batches/a.mjs", "import {h} from './link.mjs';\nexport const icons = [];\n");
        project.build();
        let error = project.bundle().unwrap_err();
        assert_eq!((error.code, error.path.as_deref()), (ErrorCode::SymlinkPath, Some("icons/batches/link.mjs")));
        assert!(error.message.contains("never follows symlinks"), "{}", error.message);

        // An evidence folder that is a symlink.
        std::fs::remove_file(project.icons().join("batches/link.mjs")).unwrap();
        project.write("batches/a.mjs", "import {h} from './shared/h.mjs';\nexport const icons = [];\n");
        project.build();
        symlink(project.dir.path(), project.icons().join("evidence")).unwrap();
        assert_eq!(project.bundle().unwrap_err().code, ErrorCode::SymlinkPath);
    }

    #[test]
    fn a_bundle_needs_a_tracked_build_of_create_s_definitions() {
        let project = Project::new();
        let listing = project.build_dir().join(SOURCES_FILE);
        let mut snapshot: Value = serde_json::from_slice(&std::fs::read(&listing).unwrap()).unwrap();

        // A path that leaves the build, or a copy that is not the one listed.
        snapshot["depends"].as_array_mut().unwrap().push(json!("../../../secret"));
        std::fs::write(&listing, snapshot.to_string()).unwrap();
        assert_eq!(project.bundle().unwrap_err().code, ErrorCode::InvalidPath);
        snapshot["depends"].as_array_mut().unwrap().pop();
        std::fs::write(&listing, snapshot.to_string()).unwrap();
        std::fs::write(project.build_dir().join("batches/shared/h.mjs"), "export const h = 2;\n").unwrap();
        assert_eq!(project.bundle().unwrap_err().code, ErrorCode::HashMismatch);
        project.build();

        // Built by an earlier version: no depends.
        let mut old = snapshot.clone();
        old.as_object_mut().unwrap().remove("depends");
        std::fs::write(&listing, old.to_string()).unwrap();
        let error = project.bundle().unwrap_err();
        assert_eq!(error.code, ErrorCode::CheckFailed);
        assert!(error.message.contains("built by an earlier version"), "{}", error.message);

        // A definitions.mjs edited by hand cannot be narrowed to the submitted icons.
        project.write("definitions.mjs", "import {icons as a} from './batches/a.mjs';\nexport const icons = a;\n");
        project.build();
        let error = project.bundle().unwrap_err();
        assert_eq!((error.code, error.path.as_deref()), (ErrorCode::CheckFailed, Some("icons/definitions.mjs")));
    }

    #[test]
    fn an_untraced_build_is_refused() {
        let project = Project::new();
        // A computed import: the build depends on every module it loaded, b's batch included.
        project.write(
            "batches/a.mjs",
            "import {h} from './shared/h.mjs';\nconst lazy = name => import(name);\nexport const icons = [];\n",
        );
        project.build();
        let error = project.bundle().unwrap_err();
        assert_eq!((error.code, error.id.as_deref()), (ErrorCode::CheckFailed, Some("a")));
        assert_eq!(
            error.message,
            "a: its imports could not be traced (an import of a computed name, or an escape in a specifier), so its bundle would hold every module its build loaded, other icons' batches included: batches/b.mjs"
        );
        assert!(
            error
                .hint
                .as_deref()
                .unwrap()
                .ends_with("or pass --include-untraced to send every module its build loaded")
        );
        // Asked for, they are bundled with a warning.
        let mut warnings = Vec::new();
        let names: Vec<String> =
            project.bundle_with(true, &mut warnings).unwrap().into_iter().map(|(name, _)| name).collect();
        assert!(names.contains(&"batches/b.mjs".to_string()), "{names:?}");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].starts_with("a: its imports could not be traced"), "{warnings:?}");

        // An escaped specifier hides its module from the scan as well.
        project.write("batches/a.mjs", "import {h} from '.\\x2fshared/h.mjs';\nexport const icons = [];\n");
        project.build();
        assert_eq!(project.bundle().unwrap_err().code, ErrorCode::CheckFailed);
        // An icon that imports another icon's batch by name is traced; the bundle says so.
        project.write("batches/a.mjs", "import {icons as b} from './b.mjs';\nexport const icons = [...b];\n");
        project.build();
        let mut warnings = Vec::new();
        project.bundle_with(false, &mut warnings).unwrap();
        assert_eq!(warnings, ["a imports other icons' batches, so the bundle holds them too: batches/b.mjs"]);
    }

    #[cfg(unix)]
    #[test]
    fn bundle_entry_names_are_plain_paths() {
        let project = Project::new();
        // Plain file-name characters on Unix, separators or worse once extracted elsewhere.
        for name in ["a\\b.md", "line\nbreak.md", "tab\t.md", "nul\u{7f}.md"] {
            project.write(&format!("evidence/{name}"), "# notes\n");
            let error = project.bundle().unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidPath, "{name:?}");
            assert_eq!(error.path.as_deref(), Some(format!("icons/evidence/{name}").as_str()));
            std::fs::remove_file(project.icons().join("evidence").join(name)).unwrap();
        }
        project.write("evidence/notes.md", "# notes\n");
        assert!(project.bundle().is_ok());
    }

    #[test]
    fn bundles_are_stored_sorted_and_dated() {
        let files: BTreeMap<String, Vec<u8>> =
            [("b.txt".to_string(), b"bee".to_vec()), ("a/x.json".to_string(), b"{}".to_vec())].into_iter().collect();
        let zip = write_zip(&files);
        let entries = crate::weight::read_zip(&zip).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a/x.json", "b.txt"]);
        assert_eq!(entries[1].data, b"bee");
        // Local header: DOS time 0, date 1980-01-01.
        assert_eq!(&zip[10..14], &[0, 0, 0x21, 0]);
        assert_eq!(write_zip(&files), zip);
    }
}
