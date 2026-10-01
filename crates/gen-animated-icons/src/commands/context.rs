//! `context`: the reference packet an AI reads before drawing an icon
//! (`generation-context.mjs` `buildPacket`), the ranked search behind it (`--search`),
//! and the design prompt that embeds it (`--prompt`, the text `claude_design.py`
//! assembled).
//!
//! In library mode (a checkout: `--library-root`, or a folder above the working one)
//! the packet reads the set manifests and loose native JSON exactly as the JS did, and
//! stdout equals the JS output except the last `inputs` entry, which names this tool
//! instead of `generation-context.mjs`. In a project the packet comes from the package
//! catalog, its `.lottie` files and the search sidecar; only `inputs` differ.

use crate::Context;
use crate::cli::ContextArgs;
use crate::embedded::SKILL_FILES;
use crate::error::{CliError, ErrorCode, Result};
use crate::js::{js_concat_string, to_js_string, truthy};
use crate::library::{LibraryIcon, LibrarySource, SearchIndex, search_index};
use crate::local_set::{BUILD_DIR, Built, strong_motion};
use crate::managed::write_inside;
use crate::output::Report;
use crate::search::{Searchable, ranked};
use crate::{config::Config, kit};
use gai_check::preflight::{Catalog, CatalogIcon};
use gai_core::sha256_hex;
use gai_package::zip::{ZipLimits, find_file, read_zip_with};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Candidates a packet or `--search` lists by default.
const CANDIDATES: usize = 6;
/// References a packet embeds at most.
const MAX_NEIGHBORS: usize = 4;
/// Largest text file `--prompt` embeds.
const MAX_CONTEXT_BYTES: usize = 300_000;

const SELECTION_NEIGHBORS: &str = "Agent-selected neighbors";
const SELECTION_LEXICAL: &str =
    "Lexical matches with listed vocabulary expansions; the current coding agent must confirm relevance";
const EVIDENCE_BOUNDARY: &str = "Current package hash matches catalog and loose JSON matches packaged animation. Geometry is initial native pose only, with local transforms; this is not a new visual or motion approval.";
const REVIEW_INSTRUCTIONS: [&str; 5] = [
    "Check whether the requested meaning already exists. Keep IDs stable; explain a new icon versus a variant. Do not copy external library geometry.",
    "Use selected neighbors for construction consistency, not a shared animation template. Inspect mechanism-specific source and failure records before design.",
    "These are text references. Initial path points do not show playback. The current coding agent reviews real native motion at equal elapsed times; self-review is not independent approval.",
    "Compare changed icons beside these neighbors at 24/32/48 px in both themes and all weights/styles. Also inspect 16/20 px in 32/36 px controls and beside text as stress cases; record limits without silently changing supported sizes.",
    "Keep optical mass/centering observations separate from hard geometry failures. A control-point hull or stroke-length estimate does not measure perceived weight.",
];
/// The fixed opening of the design prompt (`claude_design.py:59-72`).
const PROMPT_PREAMBLE: &str = "You are providing a bounded icon design proposal through an optional CLI adapter.
The developer or calling agent will review the proposal and run authoring, native
export, rendering and interaction checks. No particular supervisor model is required. Do not launch
another agent or ask for user approval. Do not claim any rendering, file edit, or test
you have not performed. You have supplied text context, no tools, and no visual image
input in this invocation. Apply the drawing and motion rules to the scope of the user
brief. This invocation is limited to a text proposal; it does not run the full
single-agent workflow. Do not delegate or claim later stages are complete.

Return a concise but drawable design: rationale; complete SVG or named vector
geometry on the 32-unit grid; parts/pivots/followers; distinct Light/Strong timings
in seconds; state/reset policy; and specific risks to verify during implementation. For a narrow
correction or critique-only brief, limit the output to that scope. SVG/code in your
response is a draft, not a change to the live gallery. Identify missing references.

";

/// The `inputs` entry naming this module, where the JS listed its own file.
fn self_input() -> Value {
    json!({
        "path": format!("gen-animated-icons@{}/context", env!("CARGO_PKG_VERSION")),
        "sha256": sha256_hex(include_str!("context.rs")),
    })
}

/// Runs `context`.
pub fn run(ctx: &Context, args: &ContextArgs) -> Result<Report> {
    let brief = args.brief.as_ref().map(|path| read_text(&ctx.cwd.join(path))).transpose()?;
    let query = query(ctx, args, brief.as_ref().map(|(_, text)| text.as_str()))?;
    if args.search {
        let limit = args.limit.map_or(CANDIDATES, |n| n as usize);
        let results = match ctx.checkout()? {
            Some(root) => library_search(&load_catalog(&root)?, &query, limit),
            None => {
                let library = ctx.library()?;
                let catalog = library.catalog()?;
                let index = library.search_index(&catalog)?;
                package_search(&catalog.icons, &index, &query, limit)
            }
        };
        let text = candidates_text(&results, &query);
        return Ok(Report::new(Value::Array(results), text));
    }
    let checkout = ctx.checkout()?;
    let packet = packet(ctx, &query, &args.neighbor)?;
    let packet_text = gai_core::to_js_json_pretty(&packet, 2) + "\n";
    let prompt = match args.prompt {
        true => Some(prompt(ctx, checkout.as_deref(), brief.as_ref(), &args.context, &packet_text)?),
        false => None,
    };
    if let Some(output) = &args.output {
        let dir = ctx.cwd.join(output);
        let exists = || {
            CliError::new(ErrorCode::Exists, format!("{} already exists", dir.display()))
                .with_path(dir.display().to_string())
                .with_hint("Choose a new --output folder")
        };
        if std::fs::symlink_metadata(&dir).is_ok() {
            return Err(exists());
        }
        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent).map_err(|error| CliError::io("create", parent, &error))?;
        }
        // Creating the folder itself fails if anything appeared there meanwhile.
        std::fs::create_dir(&dir).map_err(|error| match error.kind() {
            std::io::ErrorKind::AlreadyExists => exists(),
            _ => CliError::io("create", &dir, &error),
        })?;
        if let Some(prompt) = &prompt {
            write_inside(&dir, "prompt.md", prompt.as_bytes())?;
        }
        write_inside(&dir, "generation-context.json", packet_text.as_bytes())?;
    }
    Ok(match prompt {
        Some(prompt) => Report::new(json!({ "prompt": prompt, "packet": packet }), prompt),
        None => {
            let text = packet_summary(&packet);
            Report::new(packet, text)
        }
    })
}

/// A JSON value as text: a string as it is, anything else as JSON.
fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => gai_core::to_js_json(other),
    }
}

/// `{id, label, category, score}` as a line of `search`'s text.
fn candidate_line(candidate: &Value) -> String {
    let field = |key: &str| candidate.get(key).map(plain).unwrap_or_default();
    let category = candidate.get("category").map(plain);
    let score = candidate.get("score").and_then(Value::as_u64).and_then(|score| u32::try_from(score).ok());
    super::search::result_line(&field("id"), &field("label"), category.as_deref(), false, score)
}

/// `--search` as text: one line per candidate, as `search` prints results.
fn candidates_text(candidates: &[Value], query: &str) -> String {
    if candidates.is_empty() {
        return format!("No library icons match \"{}\". Try a synonym or the underlying object.\n", query.trim());
    }
    candidates.iter().map(|candidate| candidate_line(candidate) + "\n").collect()
}

/// The packet as text: the query, the contract's numbers, the candidates, each reference's
/// intent, brief and timing, and the review instructions. The geometry stays in the JSON.
fn packet_summary(packet: &Value) -> String {
    let number = |value: &Value| value.as_f64().map(|n| gai_core::js_number_to_string(gai_core::round5(n)));
    let mut text = String::new();
    let query = packet.get("query").and_then(Value::as_str).unwrap_or_default().trim();
    let first = query.lines().next().unwrap_or_default();
    let more = if first.len() < query.len() { "…" } else { "" };
    let selection = packet.get("selection").map(plain).unwrap_or_default();
    text.push_str(&format!("Context for \"{first}{more}\" ({selection})\n"));
    let contract = packet.get("contract").unwrap_or(&Value::Null);
    let mut terms = Vec::new();
    if let Some(grid) = contract.get("grid").and_then(number) {
        terms.push(format!("{grid}-unit grid"));
    }
    if let Some(weights) = contract.get("outlineWeights").and_then(Value::as_object) {
        let values: Vec<String> = weights.values().filter_map(number).collect();
        let names: Vec<&str> = weights.keys().map(String::as_str).collect();
        let range = contract.get("outlineWeightRange").and_then(Value::as_array).and_then(|range| {
            Some(format!("; {} to {}", number(range.first()?)?, number(range.last()?)?))
        });
        terms.push(format!("outline weights {} ({}{})", values.join(", "), names.join(", "), range.unwrap_or_default()));
    }
    if let Some(fps) = contract.get("fps").and_then(number) {
        terms.push(format!("{fps} fps"));
    }
    if let Some(duration) = contract.get("durationSeconds").and_then(number) {
        terms.push(format!("{duration} s timeline"));
    }
    if !terms.is_empty() {
        text.push_str(&format!("Contract: {}\n", terms.join(", ")));
    }
    let count = packet.get("libraryCount").map(plain).unwrap_or_default();
    text.push_str(&format!("\nCandidates (of {count} library icons):\n"));
    let candidates = packet.get("candidates").and_then(Value::as_array).cloned().unwrap_or_default();
    if candidates.is_empty() {
        text.push_str("  none\n");
    }
    for candidate in &candidates {
        text.push_str(&format!("  {}\n", candidate_line(candidate)));
    }
    let references = packet.get("references").and_then(Value::as_array).cloned().unwrap_or_default();
    text.push_str(if references.is_empty() { "\nReferences: none\n" } else { "\nReferences:\n" });
    for reference in &references {
        let field = |key: &str| reference.get(key).map(plain);
        let mut line = super::search::result_line(
            &field("id").unwrap_or_default(),
            &field("label").unwrap_or_default(),
            field("category").as_deref(),
            false,
            None,
        );
        if let Some(set) = field("set") {
            line.push_str(&format!(", set {set}"));
        }
        text.push_str(&format!("  {line}\n"));
        let intent = field("intent").filter(|intent| !intent.is_empty());
        let brief = field("brief").filter(|brief| !brief.is_empty() && Some(brief) != intent.as_ref());
        for (key, value) in [("intent", intent), ("brief", brief)] {
            if let Some(value) = value {
                text.push_str(&format!("    {key}: {value}\n"));
            }
        }
        for profile in reference.get("profiles").and_then(Value::as_array).into_iter().flatten() {
            let get = |key: &str| profile.get(key);
            let mut parts = Vec::new();
            if let Some(active) = get("activeSeconds").and_then(number) {
                parts.push(format!("active {active} s"));
            }
            if let Some(rest) = get("restFrame").and_then(number) {
                parts.push(format!("rest frame {rest}"));
            }
            if get("continuous").and_then(Value::as_bool) == Some(true) {
                parts.push("continuous".into());
            }
            if let (Some(start), Some(end)) = (get("startState"), get("endState")) {
                parts.push(format!("states {} to {}", plain(start), plain(end)));
            }
            if let Some(reverse) = get("reverse") {
                parts.push(format!("reverse {}", plain(reverse)));
            }
            let name = get("profile").map(plain).unwrap_or_else(|| "motion".into());
            text.push_str(&format!("    {name}: {}\n", parts.join(", ")));
        }
        let layers = reference.get("initialOutlineGeometry").and_then(Value::as_array).map_or(0, Vec::len);
        let plural = if layers == 1 { "" } else { "s" };
        text.push_str(&format!("    initial outline geometry: {layers} layer{plural}\n"));
    }
    let instructions = packet.get("reviewInstructions").and_then(Value::as_array).cloned().unwrap_or_default();
    if !instructions.is_empty() {
        text.push_str("\nReview:\n");
        for instruction in &instructions {
            text.push_str(&format!("  - {}\n", plain(instruction)));
        }
    }
    text.push_str(
        "\nThe whole packet (the contract, and each reference's initial outline geometry) is JSON: add --json.\n",
    );
    text
}

/// The query: words, `--query-file` or the `--brief` text, decoded like Node's
/// `readFileSync(…, 'utf8')` (invalid bytes replaced, a BOM kept).
fn query(ctx: &Context, args: &ContextArgs, brief: Option<&str>) -> Result<String> {
    let words = args.words.join(" ");
    let file = args.query_file.as_ref().map(|path| read_lossy(&ctx.cwd.join(path))).transpose()?;
    match (words.is_empty(), file) {
        (false, Some(_)) => Err(CliError::usage("Give the query as words or --query-file, not both")),
        (false, None) => Ok(words),
        (true, Some(text)) => Ok(text),
        (true, None) => brief.map(str::to_string).ok_or_else(|| {
            CliError::usage("Describe the icon: gen-animated-icons context \"<meaning>\", --query-file brief.md or --brief brief.md")
        }),
    }
}

fn read_lossy(file: &Path) -> Result<String> {
    let bytes = std::fs::read(file).map_err(|error| CliError::io("read", file, &error))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// A text file for the prompt: `(resolved path, contents)`.
fn read_text(file: &Path) -> Result<(PathBuf, String)> {
    let cannot = |detail: String| CliError::usage(format!("Cannot read text context {}: {detail}", file.display()));
    let bytes = std::fs::read(file).map_err(|error| cannot(error.to_string()))?;
    let text = String::from_utf8(bytes).map_err(|error| cannot(error.to_string()))?;
    if text.len() > MAX_CONTEXT_BYTES {
        return Err(CliError::usage(format!("Context is too large: {}; provide a relevant excerpt", file.display())));
    }
    Ok((file.to_path_buf(), text))
}

/// The packet for `query` and `neighbors`: library mode when a checkout is available,
/// otherwise the package library plus the project's own built icons.
pub fn packet(ctx: &Context, query: &str, neighbors: &[String]) -> Result<Value> {
    match ctx.checkout()? {
        Some(root) => library_packet(&root, query, neighbors),
        None => project_packet(ctx, query, neighbors),
    }
}

/// `loadCatalog`, with its errors mapped to CLI codes.
fn load_catalog(root: &Path) -> Result<Catalog> {
    Catalog::load(root).map_err(|error| match error {
        gai_check::preflight::Error::DuplicateCatalogId(id) => {
            CliError::new(ErrorCode::InvalidCatalog, format!("Duplicate catalog ID {id}"))
        }
        other => CliError::new(ErrorCode::InvalidCatalog, other.to_string()),
    })
}

/// `searchIcons` over checkout records.
fn library_search(catalog: &Catalog, query: &str, limit: usize) -> Vec<Value> {
    let texts: Vec<SearchText> = catalog.icons.iter().map(|icon| SearchText::of_record(&icon.record)).collect();
    let searchables: Vec<Searchable<'_>> = texts.iter().map(SearchText::searchable).collect();
    ranked(&searchables, query, limit)
        .into_iter()
        .map(|hit| {
            let record = &catalog.icons[hit.index].record;
            candidate(record.get("id"), record.get("label"), record.get("category"), hit.score, &hit.matched_terms)
        })
        .collect()
}

/// `searchIcons` over package descriptors, with sidecar briefs.
fn package_search(icons: &[LibraryIcon], index: &SearchIndex, query: &str, limit: usize) -> Vec<Value> {
    let searchables: Vec<Searchable<'_>> = icons
        .iter()
        .map(|icon| Searchable {
            id: &icon.id,
            label: &icon.label,
            category: icon.category.as_deref(),
            intent: icon.intent.as_deref(),
            brief: index.brief(&icon.id),
            aliases: &icon.aliases,
        })
        .collect();
    ranked(&searchables, query, limit)
        .into_iter()
        .map(|hit| {
            let icon = &icons[hit.index];
            let (label, category) = (json!(icon.label), icon.category.as_ref().map(|c| json!(c)));
            candidate(Some(&json!(icon.id)), Some(&label), category.as_ref(), hit.score, &hit.matched_terms)
        })
        .collect()
}

/// `{id, label, category, score, matchedTerms}`, `undefined` members omitted.
fn candidate(
    id: Option<&Value>,
    label: Option<&Value>,
    category: Option<&Value>,
    score: u32,
    terms: &[String],
) -> Value {
    let mut item = Map::new();
    for (key, value) in [("id", id), ("label", label), ("category", category)] {
        if let Some(value) = value {
            item.insert(key.into(), value.clone());
        }
    }
    item.insert("score".into(), json!(score));
    item.insert("matchedTerms".into(), json!(terms));
    Value::Object(item)
}

/// The strings `searchIcons` tokenizes for one record: `id + " " + label` and the
/// joined `[category, intent, brief, ...keywords]`.
struct SearchText {
    id: String,
    label: String,
    category: String,
    intent: String,
    brief: String,
    keywords: Vec<String>,
}

impl SearchText {
    fn of_record(record: &Map<String, Value>) -> SearchText {
        let joined = |key: &str| record.get(key).map(to_js_string).unwrap_or_default();
        let keywords = match record.get("keywords").filter(|k| truthy(k)) {
            Some(Value::Array(items)) => items.iter().map(to_js_string).collect(),
            _ => Vec::new(),
        };
        SearchText {
            id: record.get("id").map(to_js_string).unwrap_or_default(),
            label: js_concat_string(record.get("label")),
            category: joined("category"),
            intent: joined("intent"),
            brief: joined("brief"),
            keywords,
        }
    }

    fn searchable(&self) -> Searchable<'_> {
        Searchable {
            id: &self.id,
            label: &self.label,
            category: Some(&self.category),
            intent: Some(&self.intent),
            brief: Some(&self.brief),
            aliases: &self.keywords,
        }
    }
}

/// The packet from a checkout (`buildPacket`).
fn library_packet(root: &Path, query: &str, neighbors: &[String]) -> Result<Value> {
    let catalog = load_catalog(root)?;
    let mut inputs: Vec<Value> = catalog.inputs.iter().map(|input| input.to_json()).collect();
    let candidates = library_search(&catalog, query, CANDIDATES);
    let by_id: HashMap<String, &CatalogIcon> = catalog
        .icons
        .iter()
        .filter_map(|icon| icon.id.as_ref().and_then(Value::as_str).map(|id| (id.to_string(), icon)))
        .collect();
    let selected = select(neighbors, &candidates, |id| by_id.contains_key(id))?;
    let contract_file = root.join("motion/icon-sets/library/family-contract.json");
    let contract_bytes = read_reference(&contract_file)?;
    inputs.push(input(&contract_file, &contract_bytes));
    let contract = parse_json(&contract_bytes, &contract_file)?;
    let mut references = Vec::new();
    for id in &selected {
        let record = Value::Object(by_id[id.as_str()].record.clone());
        let directory = record.get("assetDirectory").and_then(Value::as_str).unwrap_or_default();
        let (reference, files) = record_reference(id, &record, &root.join(directory))?;
        references.push(reference);
        inputs.extend(files);
    }
    inputs.push(self_input());
    Ok(assemble(query, neighbors, catalog.icons.len(), contract, candidates, references, inputs))
}

/// The packet from the package library, plus built local icons as neighbors.
fn project_packet(ctx: &Context, query: &str, neighbors: &[String]) -> Result<Value> {
    let library = ctx.library()?;
    let catalog = library.catalog()?;
    let index = library.search_index(&catalog)?;
    let location = library.location();
    let mut inputs = vec![json!({ "path": join_location(&location, "catalog.json"), "sha256": catalog.sha256 })];
    let candidates = package_search(&catalog.icons, &index, query, CANDIDATES);
    let in_project = Config::load(&ctx.cwd, ctx.platform)?.is_some();
    let local = |id: &str| -> Result<Option<Built>> { if in_project { Built::read(&ctx.cwd, id) } else { Ok(None) } };
    for id in neighbors {
        if catalog.get(id).is_none() && local(id)?.is_none() {
            return Err(unknown_neighbor(id));
        }
    }
    let selected = select(neighbors, &candidates, |_| true)?;
    inputs.push(json!({
        "path": format!("gen-animated-icons@{}/kit/family-contract.json", env!("CARGO_PKG_VERSION")),
        "sha256": sha256_hex(kit::family_contract()),
    }));
    let contract: Value = serde_json::from_str(kit::family_contract()).map_err(|error| {
        CliError::new(ErrorCode::Internal, format!("The embedded family contract is invalid: {error}"))
    })?;
    let mut references = Vec::new();
    for id in &selected {
        match catalog.get(id) {
            Some(icon) => {
                let (reference, file) = package_reference(library.as_ref(), icon, index.entry(id), &location)?;
                references.push(reference);
                inputs.push(file);
            }
            None => {
                let built = local(id)?.ok_or_else(|| unknown_neighbor(id))?;
                // What loadCatalog adds to a record: its set and manifest.
                let mut record = built.icon.clone();
                if let Value::Object(map) = &mut record {
                    map.insert("set".into(), built.set.get("id").cloned().unwrap_or(Value::Null));
                    map.insert("manifest".into(), json!(format!("{BUILD_DIR}/{id}/set.json")));
                }
                let (reference, files) = record_reference(id, &record, &built.dir)?;
                references.push(reference);
                inputs.extend(files);
            }
        }
    }
    inputs.push(self_input());
    Ok(assemble(query, neighbors, catalog.icons.len(), contract, candidates, references, inputs))
}

fn unknown_neighbor(id: &str) -> CliError {
    CliError::new(ErrorCode::UnknownIcon, format!("Unknown neighbor {id}; choose an existing catalog ID")).with_id(id)
}

/// The neighbors to embed: the unique requested ones, else the first two candidates.
/// Unknown ids fail before the count check, as in the JS.
fn select(neighbors: &[String], candidates: &[Value], known: impl Fn(&str) -> bool) -> Result<Vec<String>> {
    if let Some(unknown) = neighbors.iter().find(|id| !known(id)) {
        return Err(unknown_neighbor(unknown));
    }
    let selected: Vec<String> = match neighbors.is_empty() {
        false => indexmap::IndexSet::<&String>::from_iter(neighbors).into_iter().cloned().collect(),
        true => {
            candidates.iter().take(2).filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_string)).collect()
        }
    };
    if selected.len() > MAX_NEIGHBORS {
        return Err(CliError::usage("Use at most four focused neighbors; split a large batch into briefs"));
    }
    Ok(selected)
}

fn assemble(
    query: &str,
    neighbors: &[String],
    library_count: usize,
    contract: Value,
    candidates: Vec<Value>,
    references: Vec<Value>,
    inputs: Vec<Value>,
) -> Value {
    json!({
        "version": 1,
        "query": query,
        "selection": if neighbors.is_empty() { SELECTION_LEXICAL } else { SELECTION_NEIGHBORS },
        "libraryCount": library_count,
        "contract": contract,
        "candidates": candidates,
        "references": references,
        "inputs": inputs,
        "reviewInstructions": REVIEW_INSTRUCTIONS,
    })
}

/// A reference from a set.json record whose loose JSON and `.lottie` sit in `folder`,
/// and the inputs it read.
fn record_reference(id: &str, record: &Value, folder: &Path) -> Result<(Value, Vec<Value>)> {
    let motions: Vec<&Value> = match record.get("motions").filter(|m| truthy(m)) {
        Some(Value::Array(motions)) => motions.iter().collect(),
        _ => vec![record],
    };
    let strong =
        motions.iter().find(|m| m.get("motionId").and_then(Value::as_str) == Some("strong")).ok_or_else(|| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Missing Strong reference for {id}")).with_id(id)
        })?;
    let name = strong.get("name").map(to_js_string).unwrap_or_else(|| "undefined".into());
    let json_file = folder.join(format!("{name}.json"));
    let package_file = folder.join(format!("{name}.lottie"));
    let json_bytes = read_reference(&json_file)?;
    let package_bytes = read_reference(&package_file)?;
    if strong.get("sha256").and_then(Value::as_str) != Some(sha256_hex(&package_bytes).as_str()) {
        return Err(stale(id));
    }
    let loose = parse_json(&json_bytes, &json_file)?;
    let packed = packed_animation(&package_bytes, &package_file, &name)?;
    if gai_core::to_js_json(&packed) != gai_core::to_js_json(&loose) {
        return Err(CliError::new(
            ErrorCode::StaleReference,
            format!("Loose native JSON differs from package for {id}; refresh the reference first"),
        )
        .with_id(id));
    }
    let mut reference = Map::new();
    reference.insert("id".into(), json!(id));
    for key in ["label", "category", "set"] {
        if let Some(value) = record.get(key) {
            reference.insert(key.into(), value.clone());
        }
    }
    for key in ["intent", "brief"] {
        let value = if strong.get(key).is_some_and(truthy) { strong.get(key) } else { record.get(key) };
        if let Some(value) = value {
            reference.insert(key.into(), value.clone());
        }
    }
    if let Some(manifest) = record.get("manifest") {
        reference.insert("sourceManifest".into(), manifest.clone());
    }
    let fill = record.get("duotoneFill").filter(|fill| !fill.is_null()).cloned();
    reference.insert("duotoneFill".into(), fill.unwrap_or_else(|| json!("undeclared in legacy metadata")));
    let profiles: Vec<Value> = motions.iter().map(|m| Value::Object(search_index::profile(m))).collect();
    reference.insert("profiles".into(), Value::Array(profiles));
    reference.insert("initialOutlineGeometry".into(), reference_geometry(&loose));
    reference.insert("evidenceBoundary".into(), json!(EVIDENCE_BOUNDARY));
    Ok((Value::Object(reference), vec![input(&json_file, &json_bytes), input(&package_file, &package_bytes)]))
}

/// A reference from a package descriptor, its `.lottie` and its sidecar entry.
fn package_reference(
    library: &dyn LibrarySource,
    icon: &LibraryIcon,
    entry: Option<&Map<String, Value>>,
    location: &str,
) -> Result<(Value, Value)> {
    let id = icon.id.as_str();
    let descriptor = &icon.descriptor;
    let strong = strong_motion(descriptor).ok_or_else(|| {
        CliError::new(ErrorCode::InvalidCatalog, format!("Missing Strong reference for {id}")).with_id(id)
    })?;
    let name = strong.get("name").and_then(Value::as_str).unwrap_or_default();
    let bytes = library.asset(icon).map_err(|error| match error.code {
        ErrorCode::HashMismatch => stale(id),
        _ => error,
    })?;
    let package_path = join_location(location, &icon.asset_path);
    let loose = packed_animation(&bytes, Path::new(&package_path), name)?;
    let sidecar = entry.and_then(|entry| entry.get("reference")).and_then(Value::as_object);
    let mut reference = Map::new();
    reference.insert("id".into(), json!(id));
    reference.insert("label".into(), json!(icon.label));
    if let Some(category) = &icon.category {
        reference.insert("category".into(), json!(category));
    }
    if let Some(set) = descriptor.pointer("/source/set") {
        reference.insert("set".into(), set.clone());
    }
    match sidecar {
        Some(sidecar) => {
            for key in ["intent", "brief"] {
                if let Some(value) = sidecar.get(key) {
                    reference.insert(key.into(), value.clone());
                }
            }
        }
        None => {
            if let Some(intent) = &icon.intent {
                reference.insert("intent".into(), json!(intent));
            }
        }
    }
    if let Some(manifest) = descriptor.pointer("/source/manifest").and_then(Value::as_str) {
        reference.insert("sourceManifest".into(), json!(format!("prototype/icons/{manifest}")));
    }
    let fill = entry.and_then(|entry| entry.get("duotoneFill")).cloned();
    reference.insert("duotoneFill".into(), fill.unwrap_or_else(|| json!("undeclared in legacy metadata")));
    let profiles = match sidecar.and_then(|s| s.get("profiles")) {
        Some(profiles) => profiles.clone(),
        None => {
            let motions = descriptor.get("motions").and_then(Value::as_array).into_iter().flatten();
            Value::Array(motions.map(|m| Value::Object(search_index::profile(m))).collect())
        }
    };
    reference.insert("profiles".into(), profiles);
    reference.insert("initialOutlineGeometry".into(), reference_geometry(&loose));
    reference.insert("evidenceBoundary".into(), json!(EVIDENCE_BOUNDARY));
    Ok((Value::Object(reference), json!({ "path": package_path, "sha256": sha256_hex(&bytes) })))
}

fn stale(id: &str) -> CliError {
    CliError::new(
        ErrorCode::StaleReference,
        format!("Stale package hash for {id}; rebuild/verify before using as a reference"),
    )
    .with_id(id)
}

/// Reads a reference file; a missing one fails with Node's `ENOENT` text.
fn read_reference(file: &Path) -> Result<Vec<u8>> {
    std::fs::read(file).map_err(|error| {
        let message = match error.kind() {
            std::io::ErrorKind::NotFound => format!("ENOENT: no such file or directory, open '{}'", file.display()),
            _ => format!("Cannot read {}: {error}", file.display()),
        };
        CliError::new(ErrorCode::StaleReference, message).with_path(file.display().to_string())
    })
}

fn parse_json(bytes: &[u8], file: &Path) -> Result<Value> {
    serde_json::from_slice(bytes)
        .map_err(|error| CliError::new(ErrorCode::CorruptArchive, format!("Cannot parse {}: {error}", file.display())))
}

/// `a/<name>.json` inside a `.lottie`, parsed.
fn packed_animation(bytes: &[u8], file: &Path, name: &str) -> Result<Value> {
    let cannot = |detail: String| {
        CliError::new(
            ErrorCode::CorruptArchive,
            format!("Cannot read native reference from {}: {detail}", file.display()),
        )
    };
    // Built references come from the project, so they are read within a player's limits.
    let files = read_zip_with(bytes, &ZipLimits::LOTTIE).map_err(|error| cannot(error.to_string()))?;
    let entry = format!("a/{name}.json");
    let data = find_file(&files, &entry).ok_or_else(|| cannot(format!("no {entry}")))?;
    serde_json::from_slice(data).map_err(|error| cannot(error.to_string()))
}

fn input(file: &Path, bytes: &[u8]) -> Value {
    json!({ "path": file.display().to_string(), "sha256": sha256_hex(bytes) })
}

/// `base` + `relative`, for folders and URLs alike.
fn join_location(base: &str, relative: &str) -> String {
    match base.ends_with('/') {
        true => format!("{base}{relative}"),
        false => format!("{base}/{relative}"),
    }
}

/// `referenceGeometry(data)`: the initial pose of every layer painted with the Regular
/// outline, from top-level layers and shapes only.
pub fn reference_geometry(data: &Value) -> Value {
    let layers = data.get("layers").and_then(Value::as_array).into_iter().flatten();
    let shapes_of = |layer: &Value| {
        layer.get("shapes").filter(|s| truthy(s)).and_then(Value::as_array).cloned().unwrap_or_default()
    };
    let is_paint = |shape: &Value| matches!(shape.get("ty").and_then(Value::as_str), Some("st" | "fl"));
    let regular = |shape: &Value| shape.pointer("/o/sid").and_then(Value::as_str) == Some("style.outline.regular");
    let geometry = layers
        .filter(|layer| shapes_of(layer).iter().any(|s| is_paint(s) && regular(s)))
        .map(|layer| {
            let shapes = shapes_of(layer);
            let mut entry = Map::new();
            if let Some(name) = layer.get("nm") {
                entry.insert("name".into(), name.clone());
            }
            let mut transform = Map::new();
            for (key, value) in layer.get("ks").and_then(Value::as_object).into_iter().flatten() {
                if let Some(initial) = initial(Some(value)) {
                    transform.insert(key.clone(), initial);
                }
            }
            entry.insert("transform".into(), Value::Object(transform));
            let paths = shapes
                .iter()
                .filter(|s| s.get("ty").and_then(Value::as_str) == Some("sh"))
                .map(|s| {
                    let ks = s.get("ks");
                    let first = match ks.and_then(|k| k.get("a")).and_then(Value::as_f64) == Some(1.0) {
                        true => initial(ks).and_then(|k| k.get(0).cloned()),
                        false => initial(ks),
                    };
                    first.unwrap_or(Value::Null)
                })
                .collect();
            entry.insert("paths".into(), Value::Array(paths));
            let paint = shapes
                .iter()
                .filter(|s| is_paint(s))
                .map(|s| {
                    let mut paint = Map::new();
                    for (key, value) in [
                        ("type", s.get("ty").cloned()),
                        ("width", initial(s.get("w"))),
                        ("cap", s.get("lc").cloned()),
                        ("join", s.get("lj").cloned()),
                    ] {
                        if let Some(value) = value {
                            paint.insert(key.into(), value);
                        }
                    }
                    Value::Object(paint)
                })
                .collect();
            entry.insert("paint".into(), Value::Array(paint));
            Value::Object(entry)
        })
        .collect();
    Value::Array(geometry)
}

/// `p?.a === 1 ? p.k[0]?.s : p?.k`.
fn initial(property: Option<&Value>) -> Option<Value> {
    let property = property?;
    match property.get("a").and_then(Value::as_f64) == Some(1.0) {
        true => property.get("k")?.get(0)?.get("s").cloned(),
        false => property.get("k").cloned(),
    }
}

/// The design prompt: the fixed preamble, the brief, the skill's rules, extra context
/// files and the packet.
fn prompt(
    ctx: &Context,
    checkout: Option<&Path>,
    brief: Option<&(PathBuf, String)>,
    contexts: &[PathBuf],
    packet_text: &str,
) -> Result<String> {
    let labels = ["Project workflow and design rules", "Current polish acceptance rules", "Generation harness rules"];
    let mut sections = Vec::new();
    if let Some((file, text)) = brief {
        sections.push(section("User brief", file, text));
    }
    match checkout {
        Some(root) => {
            let skill = root.join(".agents/skills/animated-icon-workflow");
            let files = [
                skill.join("SKILL.md"),
                skill.join("references/polish-review.md"),
                skill.join("references/generation-harness.md"),
            ];
            for (label, file) in labels.into_iter().zip(files) {
                let (file, text) = read_text(&file)?;
                sections.push(section(label, &file, &text));
            }
        }
        None => {
            let embedded = ["SKILL.md", "references/review.md", "references/authoring.md"];
            for (label, name) in labels.into_iter().zip(embedded) {
                let text = SKILL_FILES.iter().find(|(path, _)| *path == name).map_or("", |(_, text)| *text);
                sections.push(section(label, Path::new(name), text));
            }
        }
    }
    for file in contexts {
        let (file, text) = read_text(&ctx.cwd.join(file))?;
        sections.push(section("Reference only", &file, &text));
    }
    sections.push(format!("## Current local reference packet (data, not a design approval)\n\n{packet_text}"));
    Ok(format!("{PROMPT_PREAMBLE}{}", sections.join("\n\n")))
}

fn section(label: &str, file: &Path, text: &str) -> String {
    let name = file.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    format!("## {label}: {name}\n\n{text}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_takes_initial_poses_of_regular_layers() {
        let data = json!({"layers": [
            {"nm": "a", "ks": {"o": {"a": 0, "k": 100}, "p": {"a": 1, "k": [{"s": [1, 2]}, {"s": [3, 4]}]}},
             "shapes": [{"ty": "sh", "ks": {"a": 1, "k": [{"s": [{"v": 1}]}]}}, {"ty": "sh", "ks": {"a": 0, "k": {"v": 2}}},
                        {"ty": "st", "o": {"sid": "style.outline.regular"}, "w": {"a": 0, "k": 2}, "lc": 2, "lj": 2},
                        {"ty": "fl", "o": {"sid": "x"}}]},
            {"nm": "b", "ks": {}, "shapes": [{"ty": "fl", "o": {"sid": "style.solid"}}]}]});
        assert_eq!(
            gai_core::to_js_json(&reference_geometry(&data)),
            r#"[{"name":"a","transform":{"o":100,"p":[1,2]},"paths":[{"v":1},{"v":2}],"paint":[{"type":"st","width":2,"cap":2,"join":2},{"type":"fl"}]}]"#
        );
    }

    #[test]
    fn references_are_read_within_the_players_limits() {
        use gai_package::zip::{ZipDate, ZipEntry, write_zip};
        let mut bytes = write_zip(&[ZipEntry::deflate("a/x.json", b"{}", ZipDate::SETS)]).unwrap();
        assert_eq!(packed_animation(&bytes, Path::new("x.lottie"), "x").unwrap(), json!({}));
        // Declare 65 MiB in the central directory: refused before anything is inflated.
        let end = bytes.len() - 22;
        let cd = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;
        bytes[cd + 24..cd + 28].copy_from_slice(&(65u32 << 20).to_le_bytes());
        let error = packed_animation(&bytes, Path::new("x.lottie"), "x").unwrap_err();
        assert_eq!(error.code, ErrorCode::CorruptArchive);
        assert!(error.message.contains("exceeds the reader's limits"), "{}", error.message);
    }

    #[test]
    fn neighbors_are_checked_then_deduplicated() {
        let candidates = vec![json!({"id": "a"}), json!({"id": "b"}), json!({"id": "c"})];
        assert_eq!(select(&[], &candidates, |_| true).unwrap(), ["a", "b"]);
        let five = vec!["x".to_string(); 5];
        assert_eq!(select(&five, &candidates, |_| true).unwrap(), ["x"]);
        let many: Vec<String> = ["p", "q", "r", "s", "t"].map(String::from).to_vec();
        assert_eq!(select(&many, &candidates, |_| true).unwrap_err().code, ErrorCode::Usage);
        let error = select(&["zz".into()], &candidates, |id| id != "zz").unwrap_err();
        assert_eq!(
            (error.code, error.message.as_str()),
            (ErrorCode::UnknownIcon, "Unknown neighbor zz; choose an existing catalog ID")
        );
    }
}
