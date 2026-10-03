//! Spec v2 icons (`gai-icon/2`) in an app project: `gen-animated-icons/icons/<id>.icon.json`.
//!
//! - `check <id>` validates, compiles and lints a spec offline.
//! - `draft <id>` makes ONE engine call that builds every motion and renders a review sheet
//!   (motions × looks × beats) with the engine's renderer; nothing is stored or exported.
//! - `build <id>` makes ONE engine call that builds and exports every motion as one bundle,
//!   then binds slots and themes, packs the `.lottie`, verifies it in dotlottie-rs, renders
//!   the review sheets and installs it, as a v1 build does.
//! - `create <id> --spec` writes a starter spec.
//!
//! v1 icons (`batches/*.mjs`) are not touched: a command takes this path only when every icon
//! it names has a spec.

use super::add::{build_facts, local_descriptor, stage_icon, stage_settings};
use super::{lock_project, preview, text_list};
use crate::Context;
use crate::cli::{BuildArgs, CreateArgs, DraftArgs};
use crate::config::{Config, Layout};
use crate::engine;
use crate::error::{CliError, ErrorCode, Result};
use crate::local_set::{BUILD_DIR, Built, LocalSet, PREVIEW_DIR, SOURCES_FILE, initial_set_config, is_icon_name, new_id_error};
use crate::lock::Lock;
use crate::managed::{Batch, create_dir_inside, inside, read_inside, read_optional, write_inside};
use crate::output::Report;
use gai_build::post_export::{Composition, builder_manifest};
use gai_build::post_export_v2::{frame, post_export_v2};
use gai_core::sha256_hex;
use gai_engine::PlanRunner;
use gai_package::zip::ZipDate;
use gai_package::{ActionList, build_set_archives, encode_set_json};
use gai_render::{Look, Theme, render_frame};
use gai_spec::motion::{Motion, Which};
use gai_spec::spec::ReverseMode;
use gai_spec::{Compiled, Diagnostics, Level};
use serde_json::{Map, Value, json};
use std::fmt::Write;
use std::path::Path;
use std::time::Instant;

/// Folder of draft sheets.
pub const DRAFT_DIR: &str = ".gen-animated-icons/drafts";
/// The environment variable that renames the v2 engine documents (`<prefix>-1`, …).
pub const DOCUMENTS_ENV: &str = "GEN_ANIMATED_ICONS_V2_DOCUMENTS";

/// `icons/<id>.icon.json`.
pub fn spec_file(set: &LocalSet, id: &str) -> String {
    set.file(&format!("{id}{}", gai_spec::SUFFIX))
}

/// Whether `id` has a v2 spec.
pub fn has_spec(root: &Path, set: &LocalSet, id: &str) -> Result<bool> {
    Ok(is_icon_name(id) && inside(root, &spec_file(set, id))?.is_file())
}

/// Every v2 spec in the project, sorted.
pub fn spec_ids(root: &Path, set: &LocalSet) -> Result<Vec<String>> {
    let dir = inside(root, &set.icons)?;
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(CliError::io("read", &dir, &error)),
    };
    let mut ids: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(gai_spec::SUFFIX).map(str::to_string))
        .filter(|id| is_icon_name(id))
        .collect();
    ids.sort();
    Ok(ids)
}

/// The set's id and name from `set-config.json`, or the defaults of a new project set.
struct SetMeta {
    id: String,
    name: String,
}

fn set_meta(root: &Path, set: &LocalSet) -> Result<SetMeta> {
    let bytes = read_optional(&inside(root, &set.file("set-config.json"))?)?;
    let config: Value = bytes.and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(Value::Null);
    let field = |key: &str, default: &str| config.get(key).and_then(Value::as_str).unwrap_or(default).to_string();
    Ok(SetMeta { id: field("id", crate::local_set::DEFAULT_SET_ID), name: field("name", crate::local_set::DEFAULT_SET_NAME) })
}

/// A spec read and compiled.
struct Loaded {
    path: String,
    text: String,
    compiled: std::result::Result<Compiled, Diagnostics>,
    compile_ms: f64,
}

fn load(root: &Path, set: &LocalSet, id: &str) -> Result<Loaded> {
    let path = spec_file(set, id);
    let file = inside(root, &path)?;
    let bytes = std::fs::read(&file).map_err(|error| CliError::io("read", &file, &error).with_id(id))?;
    let text = String::from_utf8(bytes).map_err(|_| CliError::new(ErrorCode::SourceError, format!("{path} is not UTF-8")).with_path(path.clone()))?;
    let started = Instant::now();
    let compiled = gai_spec::load(&text);
    let compile_ms = started.elapsed().as_secs_f64() * 1000.0;
    if let Ok(c) = &compiled
        && c.spec.id != id
    {
        return Err(CliError::new(ErrorCode::SourceError, format!("{path} declares id {}, not {id}", c.spec.id)).with_path(path).with_hint("the file name and \"id\" must match"));
    }
    Ok(Loaded { path, text, compiled, compile_ms })
}

/// The findings of `id`'s spec: `(project-relative spec path, findings, whether it compiled)`.
pub(crate) fn findings(root: &Path, set: &LocalSet, id: &str) -> Result<(String, Diagnostics, bool)> {
    let loaded = load(root, set, id)?;
    Ok(match loaded.compiled {
        Ok(compiled) => (loaded.path, compiled.diagnostics, true),
        Err(diagnostics) => (loaded.path, diagnostics, false),
    })
}

/// Whether the build in `dir` is the v2 build of `id`: its `set.json` record comes from a spec.
pub(crate) fn is_v2_build(dir: &Path, id: &str) -> bool {
    let set = read_inside(dir, "set.json").ok().flatten().and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let icons = set.as_ref().and_then(|set| set.get("icons")).and_then(Value::as_array);
    let icon = icons.into_iter().flatten().find(|icon| icon.get("name").and_then(Value::as_str) == Some(id));
    icon.and_then(|icon| icon.get("source")).and_then(Value::as_str) == Some(gai_spec::FORMAT)
}

/// Verifies the v2 build of `id` as `build` did, against the spec it was built from: its own
/// copy, which must be the one its snapshot list records.
pub(crate) fn verify_build(root: &Path, id: &str) -> super::check::Verdict {
    let failed = |error: String| super::check::Verdict { summary: Vec::new(), error: Some(error) };
    let built = match Built::read(root, id) {
        Ok(Some(built)) => built,
        Ok(None) => return failed(format!("{id} is not built")),
        Err(error) => return failed(error.message),
    };
    if let Some(entry) = super::check::foreign_entry(&built.dir) {
        let shown = entry.strip_prefix(&built.dir).unwrap_or(&entry).display().to_string();
        return failed(format!("{shown} in the build is not a regular file or folder, so the build was not read"));
    }
    let spec = format!("{id}{}", gai_spec::SUFFIX);
    let Some(bytes) = read_inside(&built.dir, &spec).ok().flatten() else {
        return failed(format!("the build has no copy of its spec ({spec})"));
    };
    let listed = crate::local_set::Snapshot::read(&built.dir);
    if listed.as_ref().and_then(|listed| listed.sha256(&spec)) != Some(sha256_hex(&bytes).as_str()) {
        return failed(format!("{spec} in the build differs from the build's snapshot list"));
    }
    match std::str::from_utf8(&bytes).map(gai_spec::load) {
        Ok(Ok(compiled)) => {
            let verdict = verify(&built, &compiled);
            super::check::Verdict { summary: verdict.summary, error: verdict.error }
        }
        Ok(Err(d)) => failed(format!("{spec} in the build does not compile: {}", d.errors().next().map(ToString::to_string).unwrap_or_default())),
        Err(_) => failed(format!("{spec} in the build is not UTF-8")),
    }
}

fn diagnostics_text(text: &mut String, path: &str, d: &Diagnostics) {
    for x in &d.0 {
        let _ = writeln!(text, "  {} {} {}{}: {}", x.level.as_str(), x.code, path, if x.path.is_empty() { String::new() } else { format!("#{}", x.path) }, x.message);
        if let Some(hint) = &x.hint {
            let _ = writeln!(text, "      hint: {hint}");
        }
    }
}

/// The report of a spec that does not compile: nothing reaches the engine.
fn failed(id: &str, path: &str, d: &Diagnostics, stage: &str) -> Report {
    let errors = d.0.iter().filter(|x| x.level == Level::Error).count();
    let mut text = format!("FAIL {id}: {errors} error(s) in {path}; nothing was sent to the engine.\n");
    diagnostics_text(&mut text, path, d);
    let value = json!({"status": "FAIL", "stage": stage, "icon": id, "spec": path, "diagnostics": d.to_json()});
    Report { failed: true, ..Report::new(value, text) }
}

fn runner(ctx: &Context, command: &str) -> Result<(PlanRunner, String)> {
    engine::require_online(ctx, command, "Drop --offline; gen-animated-icons check runs the offline checks")?;
    let mut config = engine::config(ctx)?;
    config.documents = 1;
    engine::require_session(&config)?;
    let prefix = std::env::var(DOCUMENTS_ENV).ok().filter(|p| !p.trim().is_empty()).unwrap_or_else(|| gai_engine::plan::DOCUMENT_PREFIX.to_string());
    let runner = PlanRunner::new(&config, &prefix).map_err(engine::error)?;
    Ok((runner, config.url.to_string()))
}

/// `check` of v2 specs: validation, compilation and lints, offline.
pub fn check(ctx: &Context, ids: &[String]) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    let set = LocalSet::of(&config);
    let mut icons = Vec::new();
    let mut text = String::new();
    let mut failed_any = false;
    for id in ids {
        let loaded = load(root, &set, id)?;
        match &loaded.compiled {
            Ok(c) => {
                let warnings = c.diagnostics.0.len();
                let _ = writeln!(text, "PASS {id}: {} motions, {} layers, plan {} B, compiled in {:.1} ms{}", c.motions.len(), c.plan.layer_count(), c.plan.to_json().to_string().len(), loaded.compile_ms, if warnings > 0 { format!(", {warnings} warning(s)") } else { String::new() });
                diagnostics_text(&mut text, &loaded.path, &c.diagnostics);
                icons.push(json!({"id": id, "status": "PASS", "spec": loaded.path, "motions": motions_json(c), "layers": c.plan.layer_count(), "planBytes": c.plan.to_json().to_string().len(), "compileMs": round1(loaded.compile_ms), "diagnostics": c.diagnostics.to_json()}));
            }
            Err(d) => {
                failed_any = true;
                let _ = writeln!(text, "FAIL {id}: {} error(s)", d.errors().count());
                diagnostics_text(&mut text, &loaded.path, d);
                icons.push(json!({"id": id, "status": "FAIL", "spec": loaded.path, "diagnostics": d.to_json()}));
            }
        }
    }
    let value = json!({"status": if failed_any { "FAIL" } else { "PASS" }, "scope": "Spec v2 validation, compilation and lints; not a visual review", "icons": icons});
    Ok(Report { failed: failed_any, ..Report::new(value, text) })
}

fn motions_json(c: &Compiled) -> Value {
    Value::Array(
        c.motions
            .iter()
            .map(|m| json!({"name": m.name, "motion": m.which.id(), "reverse": m.reverse, "active": m.active, "rest": frame(m.rest * 60.0), "tracks": m.tracks.iter().map(|t| t.target.to_string()).collect::<Vec<_>>()}))
            .collect(),
    )
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// The round trip split at the script's own `timing.startedAt` and `totalMs`: before the
/// script (request, queue, document load), in it, and after it (commit, response). The two
/// clocks differ by their offset, which moves time between before and after.
fn phases(run: &gai_engine::Run) -> Value {
    let sent = run.sent.duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64() * 1000.0);
    let timing = run.result.get("timing");
    let started = timing.and_then(|t| t.get("startedAt")).and_then(Value::as_f64);
    let script = timing.and_then(|t| t.get("totalMs")).and_then(Value::as_f64);
    let total = run.round_trip.as_secs_f64() * 1000.0;
    match (started, script) {
        (Some(started), Some(script)) => {
            let before = (started - sent).round();
            json!({"roundTripMs": total.round(), "beforeScriptMs": before, "scriptMs": script, "afterScriptMs": (total - before - script).round()})
        }
        _ => Value::Null,
    }
}

/// `draft <id>`.
pub fn draft(ctx: &Context, args: &DraftArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    let set = LocalSet::of(&config);
    let id = args.id.as_str();
    if !has_spec(root, &set, id)? {
        let hint = if inside(root, &set.batch(id))?.is_file() { format!("{id} is a v1 icon; use gen-animated-icons preview after a build") } else { format!("Run gen-animated-icons create {id} --spec") };
        return Err(CliError::new(ErrorCode::UnknownIcon, format!("No spec {}", spec_file(&set, id))).with_id(id).with_hint(hint));
    }
    let loaded = load(root, &set, id)?;
    let compiled = match &loaded.compiled {
        Ok(c) => c,
        Err(d) => return Ok(failed(id, &loaded.path, d, "spec")),
    };
    let params = gai_spec::draft_params(compiled);
    if let Some(file) = &args.emit_call {
        return emit_call(ctx, id, file, params);
    }
    let (runner, engine_url) = runner(ctx, "draft")?;
    let run = runner.draft(gai_spec::SCRIPT, params).map_err(engine::error)?;
    let out = create_dir_inside(root, DRAFT_DIR)?;
    let rows = &compiled.sheet_rows;
    let mut written = Vec::new();
    for (index, image) in run.images.iter().enumerate() {
        let (scale, name) = if index == 0 { (4u32, format!("{id}.png")) } else { (3u32, format!("{id}-24px.png")) };
        let labelled = label_sheet(&image.bytes, rows, if index == 0 { 1 } else { 4 }, scale).unwrap_or_else(|_| image.bytes.clone());
        let path = write_inside(&out, &name, &labelled)?;
        written.push(preview::relative(root, &path));
    }
    let timing = run.result.get("timing").cloned().unwrap_or(Value::Null);
    let mut text = format!("Draft of {id}: {} rows × 8 beats in one engine call ({:.2} s round trip, {} ms in the script); nothing stored.\n", rows.len(), run.round_trip.as_secs_f64(), timing.get("totalMs").and_then(Value::as_f64).unwrap_or(0.0));
    for path in &written {
        let _ = writeln!(text, "  sheet: {path}");
    }
    let _ = writeln!(text, "  rows (top to bottom):");
    for row in rows {
        let times: Vec<String> = row["times"].as_array().into_iter().flatten().map(|t| format!("{}", t)).collect();
        let _ = writeln!(text, "    {} · {}  at {}", row["motion"].as_str().unwrap_or_default(), row["look"].as_str().unwrap_or_default(), times.join(", "));
    }
    diagnostics_text(&mut text, &loaded.path, &compiled.diagnostics);
    let _ = writeln!(text, "The engine's renderer is not the player: build {id} for dotlottie-rs sheets of every style and theme.");
    let value = json!({
        "status": "PASS",
        "icon": id,
        "sheets": written,
        "rows": rows,
        "diagnostics": compiled.diagnostics.to_json(),
        "engine": {"url": engine_url, "document": run.document, "roundTripMs": run.round_trip.as_millis() as u64, "script": timing, "phases": phases(&run)},
        "compileMs": round1(loaded.compile_ms),
        "planBytes": compiled.plan.to_json().to_string().len(),
    });
    Ok(Report::new(value, text))
}

/// Labels an engine sheet: row titles on the left, each cell's time above it. `upscale`
/// enlarges the image first (nearest neighbour), so a 24 px sheet's pixels stay visible;
/// `unit` is the pixel size of one sheet unit after that.
fn label_sheet(png_bytes: &[u8], rows: &[Value], upscale: u32, unit: u32) -> std::result::Result<Vec<u8>, String> {
    use gai_render::sheets::{Image, LABEL};
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let (w, h) = (info.width, info.height);
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf[..(w * h * 4) as usize].to_vec(),
        png::ColorType::Rgb => buf[..(w * h * 3) as usize].chunks(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        other => return Err(format!("unsupported PNG colour type {other:?}")),
    };
    let (uw, uh) = (w * upscale, h * upscale);
    let mut big = vec![0u8; (uw * uh * 4) as usize];
    for y in 0..uh {
        for x in 0..uw {
            let s = (((y / upscale) * w + x / upscale) * 4) as usize;
            let d = ((y * uw + x) * 4) as usize;
            big[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
        }
    }
    let label_w = 200;
    let mut out = Image::filled(uw + label_w, uh, [16, 23, 29]);
    out.draw(&big, uw, label_w, 0);
    let (cell, gap) = (32 * unit, 4 * unit);
    let font = if unit >= 4 { 13.0 } else { 11.0 };
    for (r, row) in rows.iter().enumerate() {
        let y = gap + r as u32 * (cell + gap);
        let motion = row["motion"].as_str().unwrap_or_default();
        let look = row["look"].as_str().unwrap_or_default();
        out.draw_text(motion, font, 8.0, (y + cell / 2) as f32, LABEL);
        out.draw_text(look, font, 8.0, (y + cell / 2) as f32 + font + 4.0, LABEL);
        for (c, t) in row["times"].as_array().into_iter().flatten().enumerate() {
            let x = label_w + gap + c as u32 * (cell + gap);
            let t = t.as_f64().unwrap_or(0.0);
            out.draw_text(&format!("{t:.3}s"), (gap as f32 - 2.0).min(font), x as f32 + 2.0, y as f32 - 2.0, LABEL);
        }
    }
    out.encode_png().map_err(|e| e.to_string())
}

/// Whether `build` should take the v2 path: every named icon has a spec, or none is named and
/// the project has specs but no v1 definitions.
pub fn wants_build(root: &Path, set: &LocalSet, args: &BuildArgs) -> Result<Option<Vec<String>>> {
    if args.ids.is_empty() {
        let specs = spec_ids(root, set)?;
        let v1 = inside(root, &set.file("definitions.mjs"))?.is_file();
        return Ok((!specs.is_empty() && !v1).then_some(specs));
    }
    let mut v2 = Vec::new();
    for id in &args.ids {
        if has_spec(root, set, id)? {
            v2.push(id.clone());
        }
    }
    match (v2.len(), args.ids.len()) {
        (0, _) => Ok(None),
        (a, b) if a == b => Ok(Some(v2)),
        _ => Err(CliError::usage("Build v1 icons (batches/*.mjs) and v2 icons (*.icon.json) in separate runs").with_hint(format!("v2 here: {}", v2.join(", ")))),
    }
}

/// `build` of v2 icons.
pub fn build(ctx: &Context, args: &BuildArgs, ids: &[String]) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    if config.layout == Layout::Legacy {
        return Err(CliError::new(ErrorCode::InvalidConfig, "This project was set up by lottie-icons").with_hint("Run gen-animated-icons init --migrate first"));
    }
    let set = LocalSet::of(&config);
    let meta = set_meta(root, &set)?;
    // Everything validates before the engine sees anything.
    let mut loaded = Vec::new();
    for id in ids {
        let l = load(root, &set, id)?;
        if let Err(d) = &l.compiled {
            return Ok(failed(id, &l.path, d, "spec"));
        }
        loaded.push((id.clone(), l));
    }
    if (args.emit_call.is_some() || args.from_bundle.is_some()) && loaded.len() != 1 {
        return Err(CliError::usage("--emit-call and --from-bundle take one icon"));
    }
    if let Some(file) = &args.emit_call {
        let (id, l) = &loaded[0];
        return emit_call(ctx, id, file, gai_spec::build_params(l.compiled.as_ref().expect("checked"), &format!("{id}.json")));
    }
    let _guard = lock_project(ctx, config.layout)?;
    let (source, engine_url) = match &args.from_bundle {
        Some(file) => {
            let path = ctx.cwd.join(file);
            let text = std::fs::read_to_string(&path).map_err(|error| CliError::io("read", &path, &error))?;
            (Export::File { text, path: path.display().to_string() }, None)
        }
        None => {
            let (runner, url) = runner(ctx, "build")?;
            (Export::Engine(Box::new(runner)), Some(url))
        }
    };
    let mut results = Vec::new();
    for (id, l) in &loaded {
        let compiled = l.compiled.as_ref().expect("checked");
        ctx.out.log(&format!("building {id} (v2: one engine call)"));
        let built = build_icon(root, &set, &meta, id, &l.text, compiled, &source).map_err(|e| if e.id.is_none() { e.with_id(id) } else { e })?;
        let verdict = verify(&built.built, compiled);
        let out = create_dir_inside(root, &format!("{PREVIEW_DIR}/{id}"))?;
        let sheets = preview::Review::of(&built.built.icon).render(&built.built.lottie, &out)?;
        results.push((id.clone(), built, verdict, sheets));
    }
    let mut changed = Vec::new();
    if !args.no_install {
        changed = install(root, &config, results.iter().filter(|r| r.2.error.is_none()).map(|r| (r.0.as_str(), &r.1.built)))?;
    }
    let passed = results.iter().all(|r| r.2.error.is_none());
    let mut text = String::new();
    let mut icons = Vec::new();
    for (id, b, verdict, sheets) in &results {
        let status = if verdict.error.is_none() { "PASS" } else { "FAIL" };
        let _ = writeln!(text, "{status} {id}: {} actions in {BUILD_DIR}/{id} (v2, one engine call: {:.2} s exec, {:.2} s download, {:.2} s in all; .lottie {} B)", b.actions.len(), b.exec_s, b.download_s, b.engine_s, b.built.lottie.len());
        for line in &verdict.summary {
            let _ = writeln!(text, "  {line}");
        }
        if let Some(error) = &verdict.error {
            let _ = writeln!(text, "  verification: {error}");
        }
        for sheet in sheets {
            let _ = writeln!(text, "  preview: {}", preview::relative(root, &sheet.path));
        }
        icons.push(json!({
            "id": id,
            "source": "gai-icon/2",
            "status": status,
            "actions": b.actions,
            "lottie": {"sha256": sha256_hex(&b.built.lottie), "bytes": b.built.lottie.len()},
            "checks": verdict.summary,
            "checkError": verdict.error,
            "build": format!("{BUILD_DIR}/{id}"),
            "preview": sheets.iter().map(|s| s.to_json(root)).collect::<Vec<_>>(),
            "engine": {"seconds": b.engine_s, "execSeconds": b.exec_s, "downloadSeconds": b.download_s, "script": b.script_timing, "phases": b.phases, "bundleBytes": b.bundle_bytes, "planBytes": b.plan_bytes},
        }));
    }
    text_list(&mut text, "Changed", &changed);
    let value = json!({"status": if passed { "PASS" } else { "FAIL" }, "engine": {"url": engine_url}, "icons": icons, "changed": changed});
    Ok(Report { failed: !passed, ..Report::new(value, text) })
}

struct IconBuild {
    built: Built,
    actions: Vec<Value>,
    phases: Value,
    engine_s: f64,
    exec_s: f64,
    download_s: f64,
    script_timing: Value,
    bundle_bytes: usize,
    plan_bytes: usize,
}

/// Records order: Light, its reverse, Strong, its reverse (v1's order).
fn record_order(c: &Compiled) -> Vec<&Motion> {
    let mut out = Vec::new();
    for which in [Which::Light, Which::Strong] {
        out.extend(c.motions.iter().filter(|m| m.which == which && !m.reverse));
        out.extend(c.motions.iter().filter(|m| m.which == which && m.reverse));
    }
    out
}

/// Where a build's bundle comes from.
enum Export {
    /// One `creator_exec` on the hosted engine.
    Engine(Box<PlanRunner>),
    /// A bundle exported elsewhere (`--from-bundle`).
    File { text: String, path: String },
}

/// What the export produced.
struct Exported {
    animations: Vec<gai_engine::BundleAnimation>,
    phases: Value,
    engine_s: f64,
    /// The `creator_exec` round trip alone.
    exec_s: f64,
    download_s: f64,
    timing: Value,
    bundle_bytes: usize,
}

fn export(source: &Export, id: &str, c: &Compiled, plan_sha: &str) -> Result<Exported> {
    let names: Vec<String> = c.motions.iter().map(|m| m.name.clone()).collect();
    match source {
        Export::Engine(runner) => {
            let started = Instant::now();
            let bundle = runner.export_bundle(gai_spec::SCRIPT, gai_spec::build_params(c, ""), id, &names, plan_sha).map_err(engine::error)?;
            Ok(Exported {
                phases: phases(&bundle.run),
                engine_s: started.elapsed().as_secs_f64(),
                exec_s: bundle.run.round_trip.as_secs_f64(),
                download_s: bundle.download.as_secs_f64(),
                timing: bundle.run.result.get("timing").cloned().unwrap_or(Value::Null),
                bundle_bytes: bundle.hosted_bytes,
                animations: bundle.animations,
            })
        }
        Export::File { text, path } => {
            let origin = json!({"engine": null, "bundleFile": path});
            let animations = gai_engine::animations_from_bundle(text, &names, gai_spec::SCRIPT, plan_sha, origin).map_err(engine::error)?;
            Ok(Exported { animations, phases: Value::Null, engine_s: 0.0, exec_s: 0.0, download_s: 0.0, timing: Value::Null, bundle_bytes: text.len() })
        }
    }
}

/// `--emit-call`: the `creator_exec` arguments, written instead of sent.
fn emit_call(ctx: &Context, id: &str, file: &Path, params: Value) -> Result<Report> {
    let path = ctx.cwd.join(file);
    let call = json!({"code": gai_spec::SCRIPT, "params": params, "timeoutMs": 30000});
    let text = call.to_string();
    std::fs::write(&path, &text).map_err(|error| CliError::io("write", &path, &error))?;
    let value = json!({"icon": id, "call": path.display().to_string(), "bytes": text.len(), "codeSha256": sha256_hex(gai_spec::SCRIPT.as_bytes())});
    Ok(Report::new(value, format!("Wrote the engine call for {id} to {} ({} B); nothing was sent.\n", path.display(), text.len())))
}

fn build_icon(root: &Path, set: &LocalSet, meta: &SetMeta, id: &str, spec_text: &str, c: &Compiled, source: &Export) -> Result<IconBuild> {
    let plan = c.plan.to_json();
    let plan_text = plan.to_string();
    let plan_sha = sha256_hex(plan_text.as_bytes());
    let bundle = export(source, id, c, &plan_sha)?;
    let engine_s = bundle.engine_s;
    let builds = create_dir_inside(root, BUILD_DIR)?;
    let staging = tempfile::Builder::new().prefix(&format!(".{id}-")).tempdir_in(&builds).map_err(|error| CliError::io("create", &builds, &error))?;
    let dir = staging.path();
    let spec_sha = sha256_hex(spec_text.as_bytes());
    let script_sha = sha256_hex(gai_spec::SCRIPT.as_bytes());
    let comp_name = format!("Motion / {} / {}", meta.name, c.spec.label);
    let write = |name: &str, bytes: &[u8]| write_inside(dir, name, bytes).map(drop);
    let mut records: Vec<(String, Value)> = Vec::new();
    for m in record_order(c) {
        let a = bundle.animations.iter().find(|a| a.name == m.name).ok_or_else(|| CliError::new(ErrorCode::EngineError, format!("the bundle lacks {}", m.name)))?;
        let composition = Composition { name: &comp_name, set_id: &meta.id, active: m.active, rest: m.rest };
        let asset = post_export_v2(&a.native.text, &composition).map_err(|e| CliError::new(ErrorCode::EngineError, format!("{}: {e}", m.name)))?;
        write(&format!("{}.json", m.name), asset.json.as_bytes())?;
        for (theme, text) in gai_build::post_export::THEMES.iter().zip(&asset.themes) {
            write(&format!("{}.theme-{}.json", m.name, theme.0), text.as_bytes())?;
        }
        write(&format!("{}.manifest.json", m.name), builder_manifest(&m.name).as_bytes())?;
        write(&format!("native/{}.json", m.name), a.native.text.as_bytes())?;
        let mut provenance = a.provenance.clone();
        provenance["specSha256"] = json!(spec_sha);
        write(&format!("native/{}.json.provenance.json", m.name), (gai_core::to_js_json_pretty(&provenance, 2) + "\n").as_bytes())?;
        let native_sha = sha256_hex(a.native.text.as_bytes());
        records.push((m.name.clone(), record(c, m, meta, asset.layers, &asset.slots, &native_sha, &plan_sha, &script_sha, &spec_sha)));
    }
    let entry = icon_entry(c, &records);
    let mut set_record = json!({
        "name": meta.name, "id": meta.id, "category": meta.name, "version": "1.0", "defaultStyle": "outline",
        "defaultMotion": "strong", "runtime": gai_build::set_json::RUNTIME, "icons": [entry],
    });
    let archives = build_set_archives(&mut set_record, dir, ActionList::Animations, ZipDate::SETS)
        .map_err(|error| CliError::new(ErrorCode::Internal, format!("{id}: packaging failed: {error}")))?;
    for archive in &archives {
        write(&format!("{}.lottie", archive.name), &archive.lottie)?;
        write(&format!("{}.manifest.json", archive.name), archive.manifest.as_bytes())?;
    }
    write("set.json", encode_set_json(&set_record).as_bytes())?;
    write(&format!("{id}{}", gai_spec::SUFFIX), spec_text.as_bytes())?;
    write("plan.json", (plan_text.clone() + "\n").as_bytes())?;
    let spec_rel = format!("{id}{}", gai_spec::SUFFIX);
    let sources = json!({"files": {spec_rel.clone(): spec_sha}, "depends": [spec_rel]});
    write(SOURCES_FILE, (gai_core::to_js_json_pretty(&sources, 2) + "\n").as_bytes())?;
    let _ = set;
    let final_dir = inside(root, &format!("{BUILD_DIR}/{id}"))?;
    if final_dir.exists() {
        std::fs::remove_dir_all(&final_dir).map_err(|error| CliError::io("remove", &final_dir, &error))?;
    }
    let staged = staging.keep();
    std::fs::rename(&staged, &final_dir).map_err(|error| CliError::io("move", &staged, &error))?;
    let built = Built::read(root, id)?.ok_or_else(|| CliError::new(ErrorCode::Internal, format!("{id}: the build folder is incomplete")))?;
    let actions = built
        .icon
        .get("animations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|a| json!({"name": a.get("name"), "sha256": a.get("sha256"), "bytes": a.get("bytes"), "layers": a.get("layers")}))
        .collect();
    Ok(IconBuild { built, actions, phases: bundle.phases, engine_s, exec_s: bundle.exec_s, download_s: bundle.download_s, script_timing: bundle.timing, bundle_bytes: bundle.bundle_bytes, plan_bytes: plan_text.len() })
}

/// One animation's record, in v1's key order where v1 has the key.
#[allow(clippy::too_many_arguments)]
fn record(c: &Compiled, m: &Motion, meta: &SetMeta, layers: usize, slots: &[String], native: &str, plan: &str, script: &str, spec_sha: &str) -> Value {
    let spec = &c.spec;
    let fade = spec.state.as_ref().is_some_and(|s| s.reverse == ReverseMode::Fade);
    let mut r = Map::new();
    let mut put = |k: &str, v: Value| {
        r.insert(k.to_string(), v);
    };
    put("name", json!(m.name));
    put("icon", json!(spec.id));
    put("motionId", json!(m.which.id()));
    put("continuous", json!(false));
    put("previewReset", json!(fade));
    put("cycleDegrees", num(spec.cycle.unwrap_or(0.0)));
    put("source", json!(gai_spec::FORMAT));
    put("motionLabel", json!(if m.which == Which::Light { "Light motion" } else { "Strong motion" }));
    put("direction", json!(if m.reverse { "reverse" } else { "forward" }));
    match &spec.state {
        Some(state) => {
            put("stateful", json!(true));
            let (from, to) = if m.reverse { (&state.to, &state.from) } else { (&state.from, &state.to) };
            put("startState", json!(from));
            put("endState", json!(to));
        }
        None => {
            put("startState", json!("rest"));
            put("endState", json!("rest"));
        }
    }
    put("heading", json!(spec.label));
    put("description", json!(spec.intent));
    put("label", json!(spec.label));
    put("intent", json!(spec.intent));
    put("width", json!(32));
    put("height", json!(32));
    put("fps", json!(60));
    put("frames", json!(144));
    put("duration", json!(2.4));
    put("active", num(m.active));
    put("rest", num(frame(m.rest * 60.0)));
    let a = m.active;
    put("keyPoses", json!([0, a * 0.2, a * 0.4, a * 0.6, a * 0.8, a]));
    put("layers", json!(layers));
    put("slots", json!(slots));
    put("nativeSha256", json!(native));
    put("sourceSha256", json!(spec_sha));
    put("planSha256", json!(plan));
    put("scriptSha256", json!(script));
    put("brief", json!(spec.intent));
    put("constraints", json!([]));
    put("category", json!(meta.name));
    put("definitionsSha256", json!(spec_sha));
    put("primary", json!({"dark": gai_build::post_export::PRIMARY_DARK, "light": gai_build::post_export::PRIMARY_LIGHT}));
    put("duotoneFill", json!(c.duotone_fill));
    put("styles", json!(["outline", "solid", "duotone"]));
    put("strokeWidth", json!({"min": 1.5, "default": 2, "max": 2.5, "presets": [1.5, 2, 2.5], "units": "32-unit construction grid"}));
    Value::Object(r)
}

/// A number as JS prints it: whole values as integers.
fn num(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 1e15 { json!(x as i64) } else { json!(x) }
}

/// The icon's `set.json` entry: its first forward record, labels, motions (forward records,
/// each with its reverse) and every record.
fn icon_entry(c: &Compiled, records: &[(String, Value)]) -> Value {
    let find = |name: &str| records.iter().find(|(n, _)| n == name).map(|(_, r)| r.clone());
    let forward: Vec<&Motion> = c.motions.iter().filter(|m| !m.reverse).collect();
    let motions: Vec<Value> = forward
        .iter()
        .filter_map(|m| {
            let mut record = find(&m.name)?;
            if c.stateful()
                && let Some(rev) = c.motions.iter().find(|r| r.reverse && r.which == m.which).and_then(|r| find(&r.name))
            {
                record["reverse"] = rev;
            }
            Some(record)
        })
        .collect();
    let mut entry = motions.first().cloned().unwrap_or_else(|| json!({}));
    if let Some(state) = &c.spec.state {
        entry["forwardLabel"] = json!(state.labels[0]);
        entry["reverseLabel"] = json!(state.labels[1]);
    }
    entry["motions"] = Value::Array(motions);
    entry["animations"] = Value::Array(records.iter().map(|(_, r)| r.clone()).collect());
    entry
}

/// A v2 build's verification.
pub struct Verdict {
    pub summary: Vec<String>,
    pub error: Option<String>,
}

/// Paint contract, format and plan structure, and dotlottie-rs renders of every action.
pub(crate) fn verify(built: &Built, c: &Compiled) -> Verdict {
    let mut summary = Vec::new();
    let mut error = None;
    let mut fail = |e: String| {
        if error.is_none() {
            error = Some(e);
        }
    };
    match gai_check::paint::audit(&[built.dir.join("set.json")], gai_check::paint::PaintContract::builtin()) {
        Ok(report) => summary.push(format!("paint: {}", report.summary())),
        Err(e) => fail(format!("paint: {e}")),
    }
    let expected_layers = c.plan.layer_count();
    let names: Vec<String> = c.plan.created_layers().iter().map(|l| l.name.clone()).chain(c.plan.rigs.iter().map(|r| r.name.clone())).collect();
    let mut keyframes = 0;
    for m in &c.motions {
        let path = built.dir.join(format!("{}.json", m.name));
        let data: Value = match std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()) {
            Some(d) => d,
            None => {
                fail(format!("{} is missing or not JSON", m.name));
                continue;
            }
        };
        let num = |k: &str| data.get(k).and_then(Value::as_f64).unwrap_or(-1.0);
        if (num("w"), num("h"), num("fr")) != (32.0, 32.0, 60.0) || num("op") < 144.0 {
            fail(format!("{}: not a 32×32, 60 fps, 2.4 s animation", m.name));
        }
        let markers: Vec<&str> = data.get("markers").and_then(Value::as_array).into_iter().flatten().filter_map(|x| x.get("cm").and_then(Value::as_str)).collect();
        if markers != ["action", "rest", "showcase"] {
            fail(format!("{}: markers {markers:?}", m.name));
        }
        let layers = data.get("layers").and_then(Value::as_array).cloned().unwrap_or_default();
        if layers.len() != expected_layers {
            fail(format!("{}: {} layers, the plan has {expected_layers}", m.name, layers.len()));
        }
        for layer in &layers {
            let nm = layer.get("nm").and_then(Value::as_str).unwrap_or_default();
            if !names.iter().any(|n| n == nm) {
                fail(format!("{}: unplanned layer {nm}", m.name));
            }
            if nm.ends_with("-matte") && layer.get("td").and_then(Value::as_u64) != Some(1) {
                fail(format!("{}: matte source {nm} is not td:1", m.name));
            }
            keyframes += count_keys(layer);
        }
    }
    summary.push(format!("format: {} actions, 32×32, 60 fps, markers, {expected_layers} layers each as planned, {keyframes} keyframes in all", c.motions.len()));
    // dotlottie-rs: every action loads and renders; a gesture ends where it starts; a
    // reverse ends where its forward starts.
    let look = Look::regular(Theme::Dark);
    let frame_at = |name: &str, f: f64| render_frame(&built.lottie, Some(name), f, 64, &look).map(|fr| fr.rgba);
    let mut rendered = 0;
    for m in &c.motions {
        let rest_frame = frame(m.rest * 60.0);
        let (first, rest, last) = (frame_at(&m.name, 0.0), frame_at(&m.name, rest_frame), frame_at(&m.name, 143.0));
        match (first, rest, last) {
            (Ok(first), Ok(rest), Ok(last)) => {
                rendered += 3;
                if gai_render::raster::raster_difference(&rest, &last) > 1e-9 {
                    fail(format!("{}: it still moves after rest", m.name));
                }
                if !c.stateful() && !gai_render::raster::equivalent(&first, &rest, false) {
                    fail(format!("{}: the rest frame differs from the first frame, so the gesture does not end where it starts", m.name));
                }
                if m.reverse {
                    let forward = c.motions.iter().find(|f| !f.reverse && f.which == m.which).map(|f| f.name.clone()).unwrap_or_default();
                    if let Ok(start) = frame_at(&forward, 0.0)
                        && !gai_render::raster::equivalent(&start, &rest, false)
                    {
                        fail(format!("{}: the reverse does not end where {forward} starts", m.name));
                    }
                }
            }
            (a, b, d) => {
                let e = [a.err(), b.err(), d.err()].into_iter().flatten().next().map(|e| e.to_string()).unwrap_or_default();
                fail(format!("{}: dotlottie-rs cannot render it: {e}", m.name));
            }
        }
    }
    summary.push(format!("player: {rendered} dotlottie-rs frames (first, rest, last of each action)"));
    // The library's visual check (`library verify-visual`) on every action: every size,
    // style, weight and theme, clipping, visibility and holds included, so an adoptable
    // build cannot fail it later.
    match std::fs::read(built.dir.join("set.json")).ok().and_then(|bytes| gai_render::SetManifest::from_slice(&bytes).ok()) {
        Some(set) => {
            let mut checked = 0;
            for meta in set.all_actions() {
                let Ok(lottie) = std::fs::read(built.dir.join(format!("{}.lottie", meta.name))) else {
                    fail(format!("{}: no .lottie to check", meta.name));
                    continue;
                };
                let run = gai_render::verify_action(&set, meta, &lottie, &gai_render::SetRules::CANONICAL, gai_render::ActionOptions { sheets: false });
                match run.outcome {
                    gai_render::Outcome::Passed(_) => checked += 1,
                    gai_render::Outcome::Failed(message) => fail(format!("{}: library visual check: {message}", meta.name)),
                }
            }
            summary.push(format!("visual: {checked} actions pass the library's renderer checks"));
        }
        None => fail("set.json cannot be read for the visual check".into()),
    }
    Verdict { summary, error }
}

fn count_keys(v: &Value) -> usize {
    match v {
        Value::Object(map) => {
            let own = match (map.get("a").and_then(Value::as_u64), map.get("k")) {
                (Some(1), Some(Value::Array(k))) => k.len(),
                _ => 0,
            };
            own + map.values().map(count_keys).sum::<usize>()
        }
        Value::Array(items) => items.iter().map(count_keys).sum(),
        _ => 0,
    }
}

/// Installs verified v2 builds as `add` installs a local icon, in one batch.
fn install<'a>(root: &Path, config: &Config, icons: impl Iterator<Item = (&'a str, &'a Built)>) -> Result<Vec<String>> {
    let lock = Lock::load(root, config.layout)?;
    let mut next = lock.clone();
    let mut batch = Batch::new();
    stage_settings(&mut batch, config);
    for (id, built) in icons {
        let descriptor = local_descriptor(config, id, built)?;
        let mut record = Map::new();
        record.insert("version".into(), json!("local"));
        record.insert("origin".into(), json!("local"));
        record.insert("format".into(), json!(gai_spec::FORMAT));
        stage_icon(&mut batch, config, id, built.lottie.clone(), &descriptor, &mut record);
        record.insert("build".into(), build_facts(built));
        next.object_mut("icons").insert(id.to_string(), Value::Object(record));
    }
    next.stamp(config.layout, None);
    let digests: Vec<(String, String)> = batch.writes().map(|(path, bytes)| (path.to_string(), sha256_hex(bytes))).collect();
    let files = next.object_mut("files");
    for (path, sha) in digests {
        files.insert(path, json!(sha));
    }
    batch.write(next.path, next.to_bytes());
    batch.plan(root, &lock.expected())?.apply()
}

/// The starter spec `create --spec` writes: a bell that swings on its hook, with a clapper
/// that lags behind, valid, lint-clean and buildable as written. Compact, one entry per line,
/// like the reference examples, so an agent edits it in place.
pub fn starter(id: &str, label: &str, intent: &str) -> String {
    let s = |text: &str| serde_json::Value::String(text.to_string()).to_string();
    format!(
        r#"{{
  "$schema": "https://gen-animated-icons.lottiefiles.com/schema/icon-v2.json",
  "format": "gai-icon/2",
  "id": {id},
  "label": {label},
  "intent": {intent},
  "//": "A starter: replace the parts and keys (references/authoring-v2.md), then check, draft and build.",
  "anchors": {{"hook": [16, 5]}},
  "parts": {{
    "bell":    {{"shape": {{"path": "M8 22 C8 14 10 8 16 8 C22 8 24 14 24 22 Z"}}, "role": "surface"}},
    "rim":     {{"shape": {{"line": [[7.5, 22.5], [24.5, 22.5]]}}}},
    "clapper": {{"shape": {{"circle": [[16, 25.5], 2.1]}}, "role": "fill"}}
  }},
  "rigs": {{
    "bell":    {{"pivot": "hook", "carries": ["rim", "clapper"]}},
    "clapper": {{"pivot": "hook"}}
  }},
  "motion": {{
    "bell.rotate": {{
      "light":  [[0, 0], [0.12, -8], [0.3, 6], [0.46, 0]],
      "strong": [[0, 0], [0.08, 4], [0.2, -13, "hit"], {{"settle": 0.6, "to": 0}}]
    }},
    "clapper.rotate": {{"follow": "bell.rotate", "lag": 0.04, "scale": 0.6}}
  }}
}}
"#,
        id = s(id),
        label = s(label),
        intent = s(intent)
    )
}

/// Library icons reanimated as specs in the checkout at `root`: id → (set, spec path).
pub(crate) fn library_specs(root: &Path) -> std::collections::BTreeMap<String, (String, std::path::PathBuf)> {
    let mut out = std::collections::BTreeMap::new();
    let Ok(sets) = std::fs::read_dir(root.join("motion/icon-sets")) else { return out };
    for set in sets.flatten() {
        let Ok(specs) = std::fs::read_dir(set.path().join(gai_check::verify::adopted::SPECS_DIR)) else { continue };
        for spec in specs.flatten() {
            let name = spec.file_name().to_string_lossy().into_owned();
            if let Some(id) = name.strip_suffix(gai_spec::SUFFIX) {
                out.insert(id.to_string(), (set.file_name().to_string_lossy().into_owned(), spec.path()));
            }
        }
    }
    out
}

/// A library spec as a new icon's: its id, label and (when given) intent replaced. The
/// spec's own layout is kept where those values are written plainly, so it stays as
/// compact as the library's.
fn spec_from(source: &str, id: &str, label: &str, intent: Option<&str>) -> std::result::Result<String, String> {
    let mut value: Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    let mut text = source.to_string();
    let mut plain = true;
    let mut swap = |key: &str, new: Value| {
        let old = value.get(key).cloned().unwrap_or(Value::Null);
        let needle = format!("\"{key}\": {old}");
        match text.find(&needle) {
            Some(at) => text.replace_range(at..at + needle.len(), &format!("\"{key}\": {new}")),
            None => plain = false,
        }
        value[key] = new;
    };
    swap("id", json!(id));
    swap("label", json!(label));
    if let Some(intent) = intent {
        swap("intent", json!(intent));
    }
    if plain && serde_json::from_str::<Value>(&text).ok().as_ref() == Some(&value) {
        return Ok(text);
    }
    serde_json::to_string_pretty(&value).map(|t| t + "\n").map_err(|e| e.to_string())
}

/// `create <id> --spec`.
pub fn create(ctx: &Context, args: &CreateArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    if config.layout == Layout::Legacy {
        return Err(CliError::new(ErrorCode::InvalidConfig, "This project was set up by lottie-icons").with_hint("Run gen-animated-icons init --migrate first"));
    }
    let id = args.id.as_str();
    if let Some(error) = new_id_error(id) {
        return Err(error);
    }
    let _guard = lock_project(ctx, config.layout)?;
    let set = LocalSet::of(&config);
    let path = spec_file(&set, id);
    for existing in [path.clone(), set.batch(id)] {
        if inside(root, &existing)?.exists() {
            return Err(CliError::new(ErrorCode::Exists, format!("{id} already exists: {existing}")).with_path(existing).with_id(id).with_hint(format!("Edit it, then run gen-animated-icons check {id}")));
        }
    }
    let label = args.label.clone().unwrap_or_else(|| {
        let mut words: Vec<String> = id.split('-').map(str::to_string).collect();
        if let Some(first) = words.first_mut() {
            *first = first[..1].to_uppercase() + &first[1..];
        }
        words.join(" ")
    });
    let (text, from) = match &args.from {
        None => {
            let intent = args.intent.clone().unwrap_or_else(|| "Light: the bell swings once on its hook. Strong: it winds up, is struck hard and rings down, the clapper lagging behind.".into());
            (starter(id, &label, &intent), None)
        }
        Some(reference) => {
            let library = ctx.library().ok();
            let checkout = ctx.checkout()?.or_else(|| library.as_ref().and_then(|l| l.checkout().map(Path::to_path_buf))).ok_or_else(|| {
                CliError::usage("--from with --spec copies a library icon's spec, which needs a library checkout")
                    .with_hint("Pass --library <checkout> (or --library-root <checkout>); the published library has no specs, so without a checkout leave out --from")
            })?;
            let specs = library_specs(&checkout);
            let Some((set_id, path)) = specs.get(reference.as_str()) else {
                let catalog = library.as_ref().and_then(|l| l.catalog().ok());
                let known = catalog.as_ref().is_none_or(|c| c.get(reference).is_some());
                if !known {
                    return Err(CliError::new(ErrorCode::UnknownIcon, format!("Unknown library icon: {reference}")).with_id(reference));
                }
                let category = catalog.as_ref().and_then(|c| c.get(reference)).and_then(|i| i.category.clone());
                let mut near: Vec<&str> = catalog
                    .iter()
                    .flat_map(|c| &c.icons)
                    .filter(|i| i.category.is_some() && i.category == category && specs.contains_key(i.id.as_str()))
                    .map(|i| i.id.as_str())
                    .take(3)
                    .collect();
                if near.is_empty() {
                    near = specs.keys().map(String::as_str).take(3).collect();
                }
                return Err(CliError::new(ErrorCode::UnsupportedReference, format!("{reference} has no spec: it was not reanimated as one"))
                    .with_id(reference)
                    .with_hint(format!("Start from an icon with a spec, such as {}, or leave out --from for the starter", near.join(", "))));
            };
            let source = std::fs::read_to_string(path).map_err(|e| CliError::io("read", path, &e))?;
            let text = spec_from(&source, id, &label, args.intent.as_deref()).map_err(|e| CliError::new(ErrorCode::InvalidCatalog, format!("{}: {e}", path.display())))?;
            (text, Some(json!({"id": reference, "set": set_id})))
        }
    };
    let mut batch = Batch::new();
    batch.write(path.clone(), text);
    let set_config = set.file("set-config.json");
    if !inside(root, &set_config)?.exists() {
        batch.write(set_config, initial_set_config());
    }
    let lock = Lock::load(root, config.layout)?;
    let changed = batch.plan(root, &lock.expected())?.apply()?;
    let mut out = format!("Created {path} (spec v2).\n");
    if let Some(from) = &from {
        let _ = writeln!(
            out,
            "It starts from library icon {} (set {}): its parts, rigs, motion, state and reverse. Redraw what the new meaning needs, rename the state and labels if it has them, and keep the motion's craft: the event, the beats, the clearance and the endings.",
            from["id"].as_str().unwrap_or_default(),
            from["set"].as_str().unwrap_or_default()
        );
    }
    out.push_str(&format!("Next:\n  gen-animated-icons check {id}\n  gen-animated-icons draft {id}   # one engine call, a review sheet, nothing stored\n  gen-animated-icons build {id}\n"));
    text_list(&mut out, "Changed", &changed);
    let value = json!({"id": id, "format": gai_spec::FORMAT, "spec": path, "from": from, "changed": changed, "next": [format!("gen-animated-icons check {id}"), format!("gen-animated-icons draft {id}"), format!("gen-animated-icons build {id}")]});
    Ok(Report::new(value, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_size(png: &[u8]) -> (u32, u32) {
        let read = |at: usize| u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]);
        (read(16), read(20))
    }

    #[test]
    fn draft_sheets_get_a_label_column_and_keep_their_cells() {
        // An engine sheet of 2 rows × 8 cells at 4× (cell 128 px, gap 16 px).
        let (w, h) = (8 * 128 + 9 * 16, 2 * 128 + 3 * 16);
        let sheet = gai_render::sheets::Image::filled(w, h, [10, 20, 30]).encode_png().unwrap();
        let rows = vec![json!({"motion": "x", "look": "regular", "times": [0.0, 0.1]}), json!({"motion": "x-strong", "look": "solid", "times": [0.0]})];
        let labelled = label_sheet(&sheet, &rows, 1, 4).unwrap();
        assert_eq!(png_size(&labelled), (w + 200, h));
        // The 24 px sheet is enlarged ×4 so its pixels stay visible.
        let small = gai_render::sheets::Image::filled(219, 81, [10, 20, 30]).encode_png().unwrap();
        assert_eq!(png_size(&label_sheet(&small, &rows, 4, 3).unwrap()), (219 * 4 + 200, 81 * 4));
    }

    #[test]
    fn the_starter_compiles_clean() {
        let text = starter("bell-ring", "Bell ring", "Light: the bell swings once. Strong: it is struck hard and rings down.");
        let compiled = gai_spec::load(&text).unwrap();
        assert!(compiled.diagnostics.0.is_empty(), "{:?}", compiled.diagnostics);
        assert_eq!(compiled.motions.len(), 2);
    }
}
