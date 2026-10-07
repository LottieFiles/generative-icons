//! `create`: scaffolds a local icon. It writes the icon's batch, keeps `definitions.mjs`
//! importing every batch, starts the set (`set-config.json`, the design review record)
//! when needed, and saves a context packet for the AI that draws it. It draws nothing.
//!
//! With `--from <library icon>` the batch starts from that icon's source: the library
//! set's definitions and batches are copied beside it as `batches/_<set>*.mjs` (kit
//! imports re-pointed at the project's kit copy) and the batch re-exports the icon under
//! the new name. Without it the batch is a commented template that fails `check` until
//! parts are drawn.

use super::{context, lock_project, text_list};
use crate::Context;
use crate::cli::CreateArgs;
use crate::config::Config;
use crate::error::{CliError, ErrorCode, Result};
use crate::kit;
use crate::local_set::{
    CONTEXT_DIR, LocalSet, definitions, initial_set_config, new_id_error, resolve_relative, rewrite_imports,
};
use crate::lock::Lock;
use crate::managed::{Batch, inside, read_optional, write_inside};
use crate::output::Report;
use crate::search::{Searchable, ranked};
use gai_core::sha256_hex;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// The design review record a new set starts with (`icon-set.py:59`).
const DESIGN_REVIEW: &str = "# Design and review record\n\nRecord meaning, reference IDs, named parts and pivots, Light/Strong timing, protected gaps, observed failures and corrections. Keep source checks, native rendering, agent visual review and user approval separate. Link final evidence to current package hashes.\n";

/// Runs `create`.
pub fn run(ctx: &Context, args: &CreateArgs) -> Result<Report> {
    if !args.batch {
        return super::v2::create(ctx, args);
    }
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    if !config.layout.is_current() {
        return Err(CliError::new(ErrorCode::InvalidConfig, format!("This project was set up by {}", config.layout.tool_name()))
            .with_hint("Run generative-icons init --migrate first"));
    }
    let id = args.id.as_str();
    if let Some(error) = new_id_error(id) {
        return Err(error);
    }
    let _guard = lock_project(ctx, config.layout)?;
    let set = LocalSet::of(&config);
    kit::verify(root, &set.kit)?;
    let batch_path = set.batch(id);
    if inside(root, &batch_path)?.exists() {
        return Err(CliError::new(ErrorCode::Exists, format!("{id} already exists: {batch_path}"))
            .with_path(batch_path)
            .with_id(id)
            .with_hint(format!("Edit it, then run generative-icons check {id}")));
    }
    let mut warnings = Vec::new();
    let library = ctx.library().and_then(|library| Ok((library.catalog()?, library)));
    match &library {
        Ok((catalog, _)) if catalog.get(id).is_some() => {
            return Err(CliError::new(ErrorCode::IdCollision, format!("{id} is already a library icon"))
                .with_id(id)
                .with_hint(format!("Install it with generative-icons add {id}, or choose a distinct id")));
        }
        Ok(_) => {}
        Err(error) => warnings
            .push(format!("Library unavailable, so {id} was not checked against library ids: {}", error.message)),
    }
    let label = args.label.clone().unwrap_or_else(|| humanize(id));

    let mut batch = Batch::new();
    let mut from = None;
    let stub = match &args.from {
        Some(reference) => {
            let checkout = match &library {
                Ok((_, library)) => library.checkout().map(Path::to_path_buf),
                Err(_) => None,
            };
            let checkout = ctx.checkout()?.or(checkout).ok_or_else(|| {
                CliError::usage("--from copies a library icon's source, which needs a library checkout")
                    .with_hint("Pass --library <checkout> (or --library-root <checkout>); the published library has no sources, so without a checkout leave out --from")
            })?;
            // A reanimated icon's set source is the motion its spec replaced.
            if crate::library::checkout_specs(&checkout).contains_key(reference.as_str()) {
                return Err(CliError::new(
                    ErrorCode::UnsupportedReference,
                    format!("{reference} was reanimated as a spec; its set source is the motion the spec replaced"),
                )
                .with_id(reference)
                .with_hint(format!("Start from its spec: generative-icons create {id} --from {reference}")));
            }
            let copy = reference_copy(&checkout, reference).map_err(|error| match &library {
                Ok((catalog, _)) if error.code == ErrorCode::UnsupportedReference => {
                    let others = eligible_references(catalog, reference);
                    match others.is_empty() {
                        true => error,
                        false => error.with_hint(format!(
                            "Start from a nearby icon of sets 07 onward instead, such as {}",
                            others.join(", ")
                        )),
                    }
                }
                _ => error,
            })?;
            for (name, text) in &copy.files {
                let path = set.file(&format!("batches/{name}"));
                match read_optional(&inside(root, &path)?)? {
                    Some(existing) if existing != text.as_bytes() => {
                        warnings.push(format!("Kept your edited {path}; it differs from the library copy"));
                    }
                    Some(_) => {}
                    None => batch.write(path, text.clone()),
                }
            }
            from = Some(json!({ "id": reference, "set": copy.set_id }));
            from_stub(id, &label, args.intent.as_deref(), reference, &copy)
        }
        None => template_stub(id, &label, args.intent.as_deref()),
    };
    batch.write(batch_path.clone(), stub);
    for (relative, content) in
        [("set-config.json", initial_set_config()), ("evidence/design-review.md", DESIGN_REVIEW.to_string())]
    {
        let path = set.file(relative);
        if !inside(root, &path)?.exists() {
            batch.write(path, content);
        }
    }

    // definitions.mjs is regenerated while it still holds what create last wrote.
    let lock = Lock::load(root, config.layout)?;
    let mut next = lock.clone();
    let definitions_path = set.file("definitions.mjs");
    let mut ids = set.batch_ids(root)?;
    ids.push(id.to_string());
    ids.sort();
    ids.dedup();
    let current = read_optional(&inside(root, &definitions_path)?)?;
    let recorded = lock
        .files()
        .any(|(path, sha)| path == definitions_path && current.as_ref().is_some_and(|bytes| sha256_hex(bytes) == sha));
    let mut next_steps = Vec::new();
    if current.is_none() || recorded {
        let text = definitions(&ids);
        next.object_mut("files").insert(definitions_path.clone(), json!(sha256_hex(&text)));
        batch.write(definitions_path.clone(), text);
    } else {
        let index = ids.len() - 1;
        next_steps.push(format!(
            "add to {definitions_path}: import {{ icons as b{index} }} from './batches/{id}.mjs'; and spread ...b{index} into icons"
        ));
    }
    batch.write(next.path, next.to_bytes());

    let packet = make_packet(ctx, &label, args, args.from.as_deref(), &mut warnings)?;
    let changed = batch.plan(root, &lock.expected())?.apply()?;
    let saved = save_packet(root, id, packet, args, args.from.as_deref())?;
    let (context, references, nearest) = (&saved.path, saved.references.clone(), saved.nearest.clone());
    next_steps.extend([
        format!("edit {batch_path}"),
        format!("generative-icons check {id}"),
        format!("generative-icons build {id}"),
        format!("generative-icons preview {id}"),
    ]);
    let mut value = json!({
        "id": id,
        "label": label,
        "from": from,
        "files": changed,
        "context": context,
        "references": references,
        "next": next_steps,
    });
    if let (Some(nearest), Value::Object(map)) = (nearest, &mut value) {
        map.shift_insert(6, "nearest".into(), Value::Array(nearest));
    }
    let mut text = String::new();
    let _ = writeln!(text, "Created {id} in {batch_path}.");
    if let Some(reference) = &args.from {
        let _ =
            writeln!(text, "It starts from library icon {reference}; its set source sits beside it as _*.mjs files.");
    }
    saved.describe(&mut text);
    text_list(&mut text, "Changed", &changed);
    text_list(&mut text, "Next", &next_steps);
    Ok(Report { warnings, ..Report::new(value, text) })
}

/// The context packet `create` saves for a new icon: made before anything is written, so
/// an unknown `--neighbor` stops create, and `None` (with a warning) when the library
/// cannot be read.
/// `from` is the icon the new one starts from (`--from`, or the icon `--replace` reworks).
pub(super) fn make_packet(ctx: &Context, label: &str, args: &CreateArgs, from: Option<&str>, warnings: &mut Vec<String>) -> Result<Option<Value>> {
    let query = nearest_query(label, args.intent.as_deref());
    let neighbors: Vec<String> = from.map(str::to_string).into_iter().chain(args.neighbor.iter().cloned()).collect();
    match context::packet(ctx, &query, &super::unique(&neighbors).into_iter().map(str::to_string).collect::<Vec<_>>()) {
        Ok(packet) => Ok(Some(packet)),
        Err(error) if !args.neighbor.is_empty() && matches!(error.code, ErrorCode::UnknownIcon | ErrorCode::Usage) => {
            Err(error.with_hint("Choose up to four --neighbor ids from generative-icons search or context --search"))
        }
        Err(error) => {
            warnings.push(format!("No context packet was saved: {}", error.message));
            Ok(None)
        }
    }
}

/// A saved context packet: its path, the references it embeds, and the closest library
/// icons only when no reference was chosen (no `--from` or `--neighbor`).
pub(super) struct SavedPacket {
    pub path: Option<String>,
    pub references: Vec<Value>,
    pub nearest: Option<Vec<Value>>,
}

impl SavedPacket {
    pub(super) fn describe(&self, text: &mut String) {
        if let Some(path) = &self.path {
            let ids: Vec<&str> = self.references.iter().filter_map(Value::as_str).collect();
            let listed = if ids.is_empty() { String::new() } else { format!(" (references: {})", ids.join(", ")) };
            let _ = writeln!(text, "Context packet: {path}{listed}");
        }
    }
}

/// Writes the packet to `.generative-icons/context/<id>.json`.
pub(super) fn save_packet(root: &Path, id: &str, packet: Option<Value>, args: &CreateArgs, from: Option<&str>) -> Result<SavedPacket> {
    let chose = from.is_some() || !args.neighbor.is_empty();
    let path = format!("{CONTEXT_DIR}/{id}.json");
    if let Some(packet) = &packet {
        let text = gai_core::to_js_json_pretty(packet, 2) + "\n";
        write_inside(root, &path, text.as_bytes())?;
    }
    let references: Vec<Value> = packet
        .as_ref()
        .and_then(|packet| packet.get("references").and_then(Value::as_array))
        .map(|references| references.iter().filter_map(|reference| reference.get("id").cloned()).collect())
        .unwrap_or_default();
    let nearest: Option<Vec<Value>> = (!chose).then(|| {
        packet
            .as_ref()
            .and_then(|packet| packet.get("candidates").and_then(Value::as_array))
            .map(|candidates| candidates.iter().take(3).cloned().collect())
            .unwrap_or_default()
    });
    Ok(SavedPacket { path: packet.is_some().then_some(path), references, nearest })
}

/// `payment-sent` → `Payment sent`.
fn humanize(id: &str) -> String {
    let words = id.replace('-', " ");
    let mut chars = words.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// Up to three icons of sets 07 onward, which `--from` accepts, to start from instead of
/// `reference`: of its category, else the nearest to its label, else to its label and
/// intent. The frozen sets' categories have no icons in later sets.
fn eligible_references(catalog: &crate::library::Catalog, reference: &str) -> Vec<String> {
    let modern: Vec<&crate::library::LibraryIcon> = catalog
        .icons
        .iter()
        .filter(|icon| icon.id != reference)
        .filter(|icon| {
            let set = icon.descriptor.pointer("/source/set").and_then(Value::as_str).unwrap_or_default();
            set.get(..2).and_then(|n| n.parse::<u32>().ok()).is_some_and(|n| n >= 7)
        })
        .collect();
    let Some(icon) = catalog.get(reference) else { return Vec::new() };
    let same: Vec<String> = modern
        .iter()
        .filter(|other| other.category.is_some() && other.category == icon.category)
        .take(3)
        .map(|other| other.id.clone())
        .collect();
    if !same.is_empty() {
        return same;
    }
    let searchables: Vec<Searchable<'_>> = modern
        .iter()
        .map(|other| Searchable {
            id: &other.id,
            label: &other.label,
            category: other.category.as_deref(),
            intent: other.intent.as_deref(),
            brief: None,
            aliases: &other.aliases,
        })
        .collect();
    let described = format!("{} {}", icon.label, icon.intent.as_deref().unwrap_or_default());
    [icon.label.as_str(), described.as_str()]
        .into_iter()
        .map(|query| ranked(&searchables, query, 3))
        .find(|hits| !hits.is_empty())
        .map(|hits| hits.iter().map(|hit| modern[hit.index].id.clone()).collect())
        .unwrap_or_default()
}

/// Words every motion brief uses, which would pull the nearest references toward any icon
/// whose intent describes its timing.
const PROFILE_WORDS: [&str; 6] = ["light", "strong", "reverse", "forward", "then", "returns"];

/// The query `create` saves a context packet for: the label and intent, without
/// [`PROFILE_WORDS`].
fn nearest_query(label: &str, intent: Option<&str>) -> String {
    let text = format!("{label} {}", intent.unwrap_or_default());
    let words = text.split_whitespace().filter(|word| {
        let bare = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        !PROFILE_WORDS.contains(&bare.as_str())
    });
    words.collect::<Vec<_>>().join(" ")
}

/// A JS string literal for generated source.
fn js_string(text: &str) -> String {
    gai_core::to_js_json(&json!(text))
}

fn template_stub(id: &str, label: &str, intent: Option<&str>) -> String {
    let intent = intent.unwrap_or("Say what the motion communicates, e.g. the bell swings once and settles.");
    format!(
        "// {id}: {label}
// One icon batch on the 32-unit grid. Draw named parts with the kit (stroke, fill,
// line, tube, framed, circle, rect, …), give Light and Strong their own timing, then run
//   generative-icons check {id}
// The contract and the nearest library references are in .generative-icons/context/{id}.json.
// An empty batch fails check on purpose: scaffolding is not a design.
import {{make, stroke, fill}} from '../../library/ai-authoring.mjs';

export const icons = [
  // make({}, {}, {},
  //   [stroke('outline', 'M8 16 L24 16'), fill('solid', 'M8 15 L24 15 L24 17 L8 17 Z')],
  //   0.42, 0.36),
// duotoneFill: true once a tint part fills an enclosed surface (framed() draws one).
].map(icon => ({{...icon, duotoneFill: false}}));
",
        js_string(id),
        js_string(label),
        js_string(intent)
    )
}

fn from_stub(id: &str, label: &str, intent: Option<&str>, reference: &str, copy: &ReferenceCopy) -> String {
    let mut overrides = format!("name: {}, label: {}, heading: {}", js_string(id), js_string(label), js_string(label));
    if let Some(intent) = intent {
        let intent = js_string(intent);
        overrides.push_str(&format!(", intent: {intent}, description: {intent}, brief: {intent}"));
    }
    format!(
        "// {id}: {label}. Started from library icon {reference} ({set}); the set's source is in
// batches/{module} and the _{set_id}.*.mjs files beside it. Change the parts and motion
// here (or in the copied files), then run
//   generative-icons check {id}
import {{icons as reference}} from './{module}';

const source = reference.find(icon => icon.name === {quoted});
// Part names start with the icon id, and constraints name parts, so both are renamed.
const rename = name => typeof name === 'string' && name.startsWith({prefix}) ? {new_prefix} + name.slice({cut}) : name;
export const icons = [{{
  ...source, {overrides},
  parts: source.parts.map(part => ({{...part, name: rename(part.name)}})),
  ...(source.constraints && {{constraints: source.constraints.map(rule => ({{...rule, part: rename(rule.part)}}))}}),
  // A stateful icon's states and action labels ship in its descriptor (React `state`,
  // handle.setState): name them for {id}.
  ...(source.stateful && {{initialState: 'prepared', finalState: 'result', forwardLabel: 'Play action', reverseLabel: 'Reset preview'}}),
}}];
",
        set = copy.set_name,
        set_id = copy.set_id,
        module = copy.module,
        quoted = js_string(reference),
        prefix = js_string(&format!("{reference}-")),
        new_prefix = js_string(&format!("{id}-")),
        cut = reference.len() + 1,
    )
}

/// A library set's source, ready to sit in a project's `batches/`.
struct ReferenceCopy {
    set_id: String,
    set_name: String,
    /// The copied `definitions.mjs`, e.g. `_46-files-folders.mjs`.
    module: String,
    /// File name → rewritten text.
    files: Vec<(String, String)>,
}

/// Copies the source set of library icon `reference` from the checkout at `root`:
/// `definitions.mjs` becomes `_<set>.mjs`, `batches/<name>.mjs` becomes
/// `_<set>.<name>.mjs`, and kit imports point at the project's kit copy.
fn reference_copy(root: &Path, reference: &str) -> Result<ReferenceCopy> {
    let catalog = gai_check::preflight::Catalog::load(root)
        .map_err(|error| CliError::new(ErrorCode::InvalidCatalog, error.to_string()))?;
    let icon = catalog.find(reference).ok_or_else(|| {
        CliError::new(ErrorCode::UnknownIcon, format!("Unknown library icon: {reference}")).with_id(reference)
    })?;
    let set_id = icon.set.as_ref().and_then(Value::as_str).unwrap_or_default().to_string();
    let frozen = set_id.get(..2).and_then(|n| n.parse::<u32>().ok()).is_none_or(|n| n < 7);
    if frozen {
        return Err(CliError::new(
            ErrorCode::UnsupportedReference,
            format!(
                "{reference} comes from {set_id}, whose builder profile differs from project sets; choose a reference from sets 07 onward (search results list each icon's set)"
            ),
        )
        .with_id(reference));
    }
    let set_dir = root.join("motion/icon-sets").join(&set_id);
    let config: Value = std::fs::read(set_dir.join("set-config.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null);
    let set_name = config.get("name").and_then(Value::as_str).unwrap_or(&set_id).to_string();
    let mut sources = vec![("definitions.mjs".to_string(), format!("_{set_id}.mjs"))];
    let batches = set_dir.join("batches");
    let mut names: Vec<String> = std::fs::read_dir(&batches)
        .map_err(|error| CliError::io("read", &batches, &error))?
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(".mjs").map(str::to_string))
        .collect();
    names.sort();
    sources.extend(names.iter().map(|name| (format!("batches/{name}.mjs"), format!("_{set_id}.{name}.mjs"))));
    let renamed: HashMap<String, String> =
        sources.iter().map(|(from, to)| (format!("{set_id}/{from}"), to.clone())).collect();
    let mut files = Vec::new();
    for (from, to) in &sources {
        let path = set_dir.join(from);
        let text = std::fs::read_to_string(&path).map_err(|error| CliError::io("read", &path, &error))?;
        let base = match from.rsplit_once('/') {
            Some((dir, _)) => format!("{set_id}/{dir}"),
            None => set_id.clone(),
        };
        let text = rewrite_imports(&text, |specifier| {
            if !specifier.starts_with('.') {
                return match specifier {
                    "svgpath" => Ok(None),
                    other => Err(unsupported_import(&set_id, from, other)),
                };
            }
            let resolved =
                resolve_relative(&base, specifier).ok_or_else(|| unsupported_import(&set_id, from, specifier))?;
            if let Some(name) = renamed.get(&resolved) {
                return Ok(Some(format!("./{name}")));
            }
            match resolved.strip_prefix("library/") {
                Some(module) if kit::files().iter().any(|file| file.name == module) => {
                    Ok(Some(format!("../../library/{module}")))
                }
                _ => Err(unsupported_import(&set_id, from, specifier)),
            }
        })?;
        files.push((to.clone(), text));
    }
    Ok(ReferenceCopy { module: format!("_{set_id}.mjs"), set_id, set_name, files })
}

fn unsupported_import(set_id: &str, file: &str, specifier: &str) -> CliError {
    CliError::usage(format!(
        "{set_id}/{file} imports {specifier}, which a project cannot resolve; choose another reference icon"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_become_labels() {
        assert_eq!(humanize("payment-sent"), "Payment sent");
        assert_eq!(humanize("a"), "A");
    }

    #[test]
    fn reference_sources_are_repointed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let write = |path: &str, text: &str| {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write(
            "prototype/icons/catalog.json",
            r#"{"sets":["46/assets/set.json"],"categories":[{"id":"f","label":"F","sets":["46/assets/set.json"]}]}"#,
        );
        write(
            "prototype/icons/46/assets/set.json",
            r#"{"id":"46-files","icons":[{"icon":"folder-open","name":"folder-open"}]}"#,
        );
        write("motion/icon-sets/46-files/set-config.json", r#"{"id":"46-files","name":"Files"}"#);
        write(
            "motion/icon-sets/46-files/definitions.mjs",
            "import {icons as a} from './batches/folders.mjs';\nimport {mix} from '../library/drawing.mjs';\nexport const icons=a;\n",
        );
        write(
            "motion/icon-sets/46-files/batches/folders.mjs",
            "import {make} from './helpers.mjs';\nexport const icons=[];\n",
        );
        write(
            "motion/icon-sets/46-files/batches/helpers.mjs",
            "export * from '../../library/ai-authoring.mjs';\nimport svgpath from 'svgpath';\n",
        );
        let copy = reference_copy(root, "folder-open").unwrap();
        assert_eq!(
            (copy.set_id.as_str(), copy.set_name.as_str(), copy.module.as_str()),
            ("46-files", "Files", "_46-files.mjs")
        );
        let files: HashMap<String, String> = copy.files.into_iter().collect();
        assert_eq!(
            files["_46-files.mjs"],
            "import {icons as a} from './_46-files.folders.mjs';\nimport {mix} from '../../library/drawing.mjs';\nexport const icons=a;\n"
        );
        assert_eq!(
            files["_46-files.folders.mjs"],
            "import {make} from './_46-files.helpers.mjs';\nexport const icons=[];\n"
        );
        assert_eq!(
            files["_46-files.helpers.mjs"],
            "export * from '../../library/ai-authoring.mjs';\nimport svgpath from 'svgpath';\n"
        );
        write("motion/icon-sets/46-files/batches/bad.mjs", "import {x} from '../../library/keyframes.mjs';\n");
        assert_eq!(reference_copy(root, "folder-open").err().unwrap().code, ErrorCode::Usage);

        // Sets before 07 have their own builder profiles, so their icons are refused as such.
        write(
            "prototype/icons/catalog.json",
            r#"{"sets":["46/assets/set.json","04/assets/set.json"],"categories":[{"id":"f","label":"F","sets":["46/assets/set.json","04/assets/set.json"]}]}"#,
        );
        write("prototype/icons/04/assets/set.json", r#"{"id":"04-actions","icons":[{"icon":"trash","name":"trash"}]}"#);
        let error = reference_copy(root, "trash").err().unwrap();
        assert_eq!(
            (error.code, error.code.exit_code(), error.id.as_deref()),
            (ErrorCode::UnsupportedReference, 3, Some("trash"))
        );
        assert!(error.message.contains("choose a reference from sets 07 onward (search results list each icon's set)"));
    }

    #[test]
    fn eligible_references_share_the_category_and_a_modern_set() {
        let icon = |id: &str, set: &str, category: &str| {
            let sha = "a".repeat(64);
            format!(
                r#"{{"id":"{id}","label":"{id}","category":"{category}","stateful":false,"source":{{"set":"{set}"}},"asset":{{"path":"assets/{id}.lottie","sha256":"{sha}"}}}}"#
            )
        };
        let icons = [
            icon("trash", "04-actions", "Actions"),
            icon("photo-delete", "43-photos-camera", "Actions"),
            icon("old-bin", "05-interface", "Actions"),
            icon("folder-open", "46-files", "Files"),
            icon("delete-message", "61-chat", "Actions"),
            icon("archive", "26-account", "Actions"),
            icon("bin-four", "47-x", "Actions"),
        ];
        let catalog = format!(r#"{{"schemaVersion":1,"version":"1","icons":[{}]}}"#, icons.join(","));
        let catalog = crate::library::Catalog::parse(catalog.as_bytes(), "x").unwrap();
        assert_eq!(eligible_references(&catalog, "trash"), ["photo-delete", "delete-message", "archive"]);
        assert!(eligible_references(&catalog, "no-such-icon").is_empty());
    }

    #[test]
    fn frozen_references_get_the_nearest_modern_icons() {
        // As in the library: the frozen sets' categories have no icons in sets 07 onward.
        let icon = |id: &str, label: &str, set: &str, category: &str, intent: &str| {
            let sha = "a".repeat(64);
            format!(
                r#"{{"id":"{id}","label":"{label}","category":"{category}","intent":"{intent}","stateful":false,"source":{{"set":"{set}"}},"asset":{{"path":"assets/{id}.lottie","sha256":"{sha}"}}}}"#
            )
        };
        let icons = [
            icon("trash", "Trash", "04-actions", "Actions", "The lid snaps open and the bin rim dips."),
            icon("send", "Send", "04-actions", "Actions", "The plane launches."),
            icon("bell", "Bell", "01-heart-bell", "Essentials", "The bell rings."),
            icon("bell-ring", "Bell ring", "40-notifications", "Notifications", "The bell swings."),
            icon("message-sent", "Message sent", "45-chat", "Chat", "The message leaves."),
            icon("bin-lid", "Recycle bin", "47-files", "Files", "The bin lid opens."),
            icon("folder-open", "Folder open", "46-files", "Files", "The folder opens."),
        ];
        let catalog = format!(r#"{{"schemaVersion":1,"version":"1","icons":[{}]}}"#, icons.join(","));
        let catalog = crate::library::Catalog::parse(catalog.as_bytes(), "x").unwrap();
        assert_eq!(eligible_references(&catalog, "bell"), ["bell-ring"]);
        // No later icon is named for it, so its label and intent decide.
        assert_eq!(eligible_references(&catalog, "trash"), ["bin-lid", "folder-open"]);
        assert!(eligible_references(&catalog, "send").is_empty());
    }

    #[test]
    fn the_nearest_query_drops_timing_words() {
        assert_eq!(
            nearest_query("Watering can", Some("The can tips forward, then returns; Strong pours more than Light.")),
            "Watering can The can tips pours more than"
        );
        assert_eq!(nearest_query("Bell", None), "Bell");
    }
}
