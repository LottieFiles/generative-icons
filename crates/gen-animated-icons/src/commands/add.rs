//! `add`: installs library icons, and local icons this project built. Web output is
//! byte-identical to `lottie-icons add` apart from the tool name in headers and the
//! paths (exactly legacy in a legacy project); iOS and Android get the `.lottie` plus a
//! JSON descriptor. A local icon installs its build's Strong `.lottie`, as the package
//! ships each library icon's Strong archive.

use super::{lock_project, text_list, unique};
use crate::Context;
use crate::cli::AddArgs;
use crate::config::Config;
use crate::error::{CliError, ErrorCode, Result};
use crate::js::json_file;
use crate::kit;
use crate::local_set::{BUILD_DIR, Built, SOURCES_FILE};
use crate::lock::Lock;
use crate::managed::Batch;
use crate::output::Report;
use crate::project::Platform;
use crate::weight::rewrite_stroke_width;
use gai_core::sha256_hex;
use gai_package::{DescriptorSource, icon_descriptor};
use serde_json::{Map, Value, json};
use std::fmt::Write;

/// Outline weights the icons are drawn at.
const AUTHORED_WEIGHTS: [f64; 3] = [1.5, 2.0, 2.5];

const SETTINGS_TYPES: &str = "import type { PaintOptions } from \"@lottiefiles/generative-icons-runtime\";\nexport declare const defaults: PaintOptions;\n";
const ICON_TYPES: &str = "import type { IconDescriptor } from \"@lottiefiles/generative-icons-runtime\";\nexport declare const icon: IconDescriptor;\nexport declare const src: string;\nexport default icon;\n";

/// Runs `add`.
pub fn run(ctx: &Context, args: &AddArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    if let Some(weight) = args.bake_weight {
        check_bake_weight(config.platform, weight)?;
    }
    let _guard = lock_project(ctx, config.layout)?;
    let lock = Lock::load(root, config.layout)?;

    // Requested ids, and the weight baked into each.
    let requested: Vec<(String, Option<f64>)> = if args.installed {
        lock.icons()
            .filter(|(_, record)| record.get("origin").and_then(Value::as_str).is_none_or(|o| o == "library"))
            .map(|(id, record)| {
                let kept = record.pointer("/variant/weight").and_then(Value::as_f64);
                (id.to_string(), args.bake_weight.or(kept))
            })
            .collect()
    } else {
        args.ids.iter().map(|id| (id.clone(), args.bake_weight)).collect()
    };
    if requested.is_empty() {
        return Err(CliError::usage("No installed library icons to re-add"));
    }
    let ids: Vec<String> = requested.iter().map(|(id, _)| id.clone()).collect();

    let library = ctx.library()?;
    let catalog = library.catalog()?;
    let mut next = lock.clone();
    let mut batch = Batch::new();
    stage_settings(&mut batch, &config);
    for id in unique(&ids) {
        let weight = requested.iter().find(|(requested, _)| requested == id).and_then(|(_, weight)| *weight);
        let mut record = Map::new();
        let (bytes, descriptor, built) = match catalog.get(id) {
            Some(icon) => {
                record.insert("version".into(), json!(catalog.version));
                (library.asset(icon)?, icon.descriptor.clone(), None)
            }
            None => {
                let built = Built::read(root, id)?.ok_or_else(|| {
                    CliError::new(
                        ErrorCode::UnknownIcon,
                        format!("Unknown icon: {id}. Run gen-animated-icons search <meaning>."),
                    )
                    .with_id(id)
                })?;
                record.insert("version".into(), json!("local"));
                record.insert("origin".into(), json!("local"));
                (built.lottie.clone(), local_descriptor(&config, id, &built)?, Some(built))
            }
        };
        let bytes = match weight {
            Some(weight) => rewrite_stroke_width(&bytes, weight).map_err(|error| error.with_id(id))?,
            None => bytes,
        };
        stage_icon(&mut batch, &config, id, bytes, &descriptor, &mut record);
        if let Some(built) = &built {
            record.insert("build".into(), build_facts(built));
        }
        if let Some(weight) = weight {
            record.insert("variant".into(), json!({ "weight": weight }));
        }
        next.object_mut("icons").insert(id.to_string(), Value::Object(record));
    }
    next.stamp(config.layout, Some(&catalog.version));
    let digests: Vec<(String, String)> =
        batch.writes().map(|(path, bytes)| (path.to_string(), sha256_hex(bytes))).collect();
    let files = next.object_mut("files");
    for (path, sha) in digests {
        files.insert(path, json!(sha));
    }
    batch.write(next.path, next.to_bytes());

    let plan = batch.plan(root, &lock.expected())?;
    let changed = if args.dry_run { plan.pending() } else { plan.apply()? };

    let mut warnings = Vec::new();
    let baked = requested.iter().any(|(_, weight)| weight.is_some());
    if config.platform.is_native() && !baked && !AUTHORED_WEIGHTS.contains(&config.weight) {
        warnings.push(format!(
            "defaults.weight {} is not an authored weight; iOS and Android players show {}. Add with --bake-weight {} to bake it into the files",
            gai_core::js_number_to_string(config.weight),
            gai_core::js_number_to_string(nearest_authored(config.weight)),
            gai_core::js_number_to_string(config.weight)
        ));
    }

    let mut value =
        json!({ "changed": changed, "icons": ids, "version": catalog.version, "platform": config.platform.as_str() });
    if args.dry_run {
        value["dryRun"] = json!(true);
    }
    let mut text = String::new();
    let verb = if args.dry_run { "Would add" } else { "Added" };
    let _ = writeln!(text, "{verb} {} from library {}.", unique(&ids).join(", "), catalog.version);
    text_list(&mut text, if args.dry_run { "Would change" } else { "Changed" }, &changed);
    Ok(Report { warnings, ..Report::new(value, text) })
}

/// Stages the shared web `settings` module (web projects only).
pub(crate) fn stage_settings(batch: &mut Batch, config: &Config) {
    if config.platform != Platform::Web {
        return;
    }
    let settings = format!(
        "// Shared project defaults. Edit {}, then run add to update.\nexport const defaults = {};\n",
        config.layout.config_file(),
        gai_core::to_js_json_pretty(&config.defaults, 2)
    );
    batch.write(format!("{}/settings.mjs", config.descriptors), settings);
    batch.write(format!("{}/settings.d.mts", config.descriptors), SETTINGS_TYPES);
}

/// Stages one icon's `.lottie` and descriptor files, and adds `sha256`, `source` and
/// `asset` to its lock record.
pub(crate) fn stage_icon(
    batch: &mut Batch,
    config: &Config,
    id: &str,
    bytes: Vec<u8>,
    descriptor: &Value,
    record: &mut Map<String, Value>,
) {
    let tool = config.layout.tool_name();
    let asset = format!("{}/{id}.lottie", config.assets);
    record.insert("sha256".into(), json!(sha256_hex(&bytes)));
    batch.write(asset.clone(), bytes);
    // Web projects (the only ones with a public path) get an ES module and types.
    let source = match &config.public_path {
        Some(public_path) => {
            let source = format!("{}/{id}.mjs", config.descriptors);
            let public_path = public_path.strip_suffix('/').unwrap_or(public_path);
            let module = format!(
                "// Generated by {tool}. Review local edits before updating.\nexport const icon = {};\nexport const src = {};\nexport default icon;\n",
                gai_core::to_js_json_pretty(descriptor, 2),
                gai_core::to_js_json(&json!(format!("{public_path}/{id}.lottie")))
            );
            batch.write(source.clone(), module);
            batch.write(format!("{}/{id}.d.mts", config.descriptors), ICON_TYPES);
            source
        }
        None => {
            let source = format!("{}/{id}.json", config.descriptors);
            batch.write(source.clone(), json_file(descriptor));
            source
        }
    };
    record.insert("source".into(), json!(source));
    record.insert("asset".into(), json!(asset));
}

/// The descriptor of a built local icon: the package descriptor of its set.json record,
/// with the project's license.
pub(crate) fn local_descriptor(config: &Config, id: &str, built: &Built) -> Result<Value> {
    let set_id = built.set.get("id").cloned().unwrap_or(Value::Null);
    let label = built.set.get("category").or_else(|| built.set.get("name")).and_then(Value::as_str).unwrap_or(id);
    let category_id = set_id.as_str().and_then(|set| set.get(3..)).unwrap_or(id).to_string();
    let manifest = format!("{BUILD_DIR}/{id}/set.json");
    let source = DescriptorSource {
        category_id: &category_id,
        category_label: label,
        set_id: &set_id,
        manifest: &manifest,
        manifest_sha256: &sha256_hex(&built.set_bytes),
    };
    let asset_path = built.dir.join(format!("{}.lottie", built.strong));
    let mut descriptor = icon_descriptor(id, &built.icon, &built.lottie, &asset_path, &source).map_err(|error| {
        CliError::new(ErrorCode::CheckFailed, format!("The build of {id} makes an invalid descriptor: {error}"))
            .with_id(id)
    })?;
    let license = config.raw.get("local").and_then(|local| local.get("license")).and_then(Value::as_str);
    descriptor["license"] = json!(license.unwrap_or("UNLICENSED"));
    Ok(descriptor)
}

/// `build`: the sources, kit and engine a local icon came from, and each action's
/// asset JSON digest. Nothing time-dependent, so the lock stays deterministic.
pub(crate) fn build_facts(built: &Built) -> Value {
    let mut actions = Map::new();
    for animation in built.icon.get("animations").and_then(Value::as_array).into_iter().flatten() {
        if let Some(name) = animation.get("name").and_then(Value::as_str)
            && let Ok(bytes) = std::fs::read(built.dir.join(format!("{name}.json")))
        {
            actions.insert(name.into(), json!(sha256_hex(bytes)));
        }
    }
    let engine = std::fs::read(built.dir.join(format!("native/{}.json.provenance.json", built.strong)))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .map(|proof| engine_label(&proof));
    let sources = std::fs::read(built.dir.join(SOURCES_FILE)).ok().map(sha256_hex);
    json!({
        "definitionsSha256": built.icon.get("definitionsSha256").cloned().unwrap_or(Value::Null),
        "sourcesSha256": sources,
        "kitSha256": kit::sha256(),
        "engine": engine,
        "actions": actions,
    })
}

/// `creator-mcp@0.2.0`, from an export's provenance.
pub(crate) fn engine_label(provenance: &Value) -> Value {
    match (
        provenance.pointer("/server/name").and_then(Value::as_str),
        provenance.pointer("/server/version").and_then(Value::as_str),
    ) {
        (Some(name), Some(version)) => json!(format!("{name}@{version}")),
        _ => provenance.get("engine").cloned().unwrap_or(Value::Null),
    }
}

/// `--bake-weight` is for native players, which cannot apply in-between weights.
fn check_bake_weight(platform: Platform, weight: f64) -> Result<()> {
    if !platform.is_native() {
        return Err(CliError::usage(
            "--bake-weight is for iOS and Android; the web runtime applies any weight from 1 to 3",
        ));
    }
    if !(1.0..=3.0).contains(&weight) || AUTHORED_WEIGHTS.contains(&weight) {
        return Err(CliError::usage("--bake-weight takes a weight from 1 to 3 other than the authored 1.5, 2 and 2.5"));
    }
    Ok(())
}

/// The authored weight native players snap to; the lighter one wins a tie.
fn nearest_authored(weight: f64) -> f64 {
    AUTHORED_WEIGHTS
        .into_iter()
        .fold(AUTHORED_WEIGHTS[0], |best, w| if (w - weight).abs() < (best - weight).abs() { w } else { best })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_like_the_native_snippets() {
        assert_eq!(nearest_authored(1.0), 1.5);
        assert_eq!(nearest_authored(1.75), 1.5);
        assert_eq!(nearest_authored(1.8), 2.0);
        assert_eq!(nearest_authored(3.0), 2.5);
    }

    #[test]
    fn bake_weight_is_native_only_and_in_between() {
        assert!(check_bake_weight(Platform::Web, 1.75).is_err());
        assert!(check_bake_weight(Platform::Ios, 2.0).is_err());
        assert!(check_bake_weight(Platform::Android, 3.5).is_err());
        assert!(check_bake_weight(Platform::Android, 1.0).is_ok());
    }
}
