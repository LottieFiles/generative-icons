//! A project's own icon set, laid out like a library set so it can move into
//! `motion/icon-sets/NN-slug/` unchanged:
//!
//! ```text
//! gen-animated-icons/icons/set-config.json      id, name, sharedSources (user-owned)
//! gen-animated-icons/icons/definitions.mjs      one import per icon batch (managed until edited)
//! gen-animated-icons/icons/batches/<id>.mjs     one icon batch each (user-owned)
//! gen-animated-icons/icons/batches/_*.mjs       helpers and copied reference sets
//! gen-animated-icons/library/                   the managed kit copy
//! .gen-animated-icons/build/<id>/               one self-contained build per icon
//! ```
//!
//! A build folder holds what a library set keeps in `motion/icon-sets/NN` and
//! `prototype/icons/NN/assets` together: scenes, natives, assets, `.lottie` files,
//! `set.json`, a snapshot of the sources it was built from, and its evidence.
//!
//! The snapshot holds every module the loader read below the icons folder (helpers in
//! subfolders included), the top-level batches the records hash, and the set's config
//! and rule files. [`SOURCES_FILE`] lists each with its sha256, and the files this icon
//! depends on: `{"files": {"<icons-relative path>": "<sha256>"}, "depends": [<path>…]}`,
//! JS-JSON indented 2. A build is stale only when one of its `depends` changed (see
//! [`changed_source`]), so editing or creating one icon leaves the others current.

use crate::config::Config;
use crate::error::{CliError, ErrorCode, Result};
use crate::managed::{inside, is_portable_path, is_windows_reserved, read_inside, write_inside};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Folder of per-icon builds.
pub const BUILD_DIR: &str = ".gen-animated-icons/build";
/// Folder of preview sheets.
pub const PREVIEW_DIR: &str = ".gen-animated-icons/previews";
/// Folder of saved context packets.
pub const CONTEXT_DIR: &str = ".gen-animated-icons/context";
/// Set id of a new project set; `NN-slug` like a library set, outside the catalog range.
pub const DEFAULT_SET_ID: &str = "00-local";
/// Set name of a new project set, shown in composition names.
pub const DEFAULT_SET_NAME: &str = "Local icons";
/// Source files a build snapshots, besides `batches/*.mjs` and the modules the loader read.
const SNAPSHOT_FILES: [&str; 4] = ["definitions.mjs", "set-config.json", "verify.rules.json", "geometry-contract.json"];
/// The build's list of snapshotted sources. Its name cannot be an action file's.
pub const SOURCES_FILE: &str = "sources.snapshot.json";

/// `[a-z][a-z0-9]*(?:-[a-z0-9]+)*`: a kebab-case icon name.
pub fn is_icon_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
        && name.as_bytes()[0].is_ascii_lowercase()
}

/// Why `id` cannot name a new local icon (`USAGE`), or `None` when it can: it must be
/// kebab-case ([`is_icon_name`]), and every file `submit` bundles under it must have a
/// portable name ([`is_portable_path`]), which a Windows device name such as `con` is not.
pub fn new_id_error(id: &str) -> Option<CliError> {
    if !is_icon_name(id) {
        return Some(CliError::usage(format!("Use a kebab-case icon id such as payment-sent, not {id}")));
    }
    let bundled = [format!("batches/{id}.mjs"), format!("built/{id}/{id}.lottie"), format!("previews/{id}/{id}.png")];
    if bundled.iter().all(|path| is_portable_path(path)) {
        return None;
    }
    let why = match is_windows_reserved(id) {
        true => "Windows reserves it as a device name (con, prn, aux, nul, com1 to com9, lpt1 to lpt9)",
        false => "its files would not have portable names",
    };
    Some(
        CliError::usage(format!("{id} cannot be an icon id: {why}, so submit could never bundle its files"))
            .with_id(id)
            .with_hint(format!("Choose another id, such as {id}-icon")),
    )
}

/// The project's icon set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSet {
    /// Project-relative source folder (`gen-animated-icons/icons`).
    pub icons: String,
    /// Project-relative kit folder (`gen-animated-icons/library`).
    pub kit: String,
}

impl LocalSet {
    /// The set of a project config.
    pub fn of(config: &Config) -> LocalSet {
        LocalSet { icons: config.local_source.clone(), kit: config.kit_dir() }
    }

    /// `icons/<relative>`.
    pub fn file(&self, relative: &str) -> String {
        format!("{}/{relative}", self.icons)
    }

    /// The icon's batch file.
    pub fn batch(&self, id: &str) -> String {
        self.file(&format!("batches/{id}.mjs"))
    }

    /// Absolute source folder.
    pub fn icons_dir(&self, root: &Path) -> Result<PathBuf> {
        inside(root, &self.icons)
    }

    /// Absolute kit folder.
    pub fn kit_dir(&self, root: &Path) -> Result<PathBuf> {
        inside(root, &self.kit)
    }

    /// Whether the set has been started (`create` wrote its `set-config.json`).
    pub fn exists(&self, root: &Path) -> Result<bool> {
        Ok(inside(root, &self.file("set-config.json"))?.is_file())
    }

    /// Fails with a hint to `create` when the set has not been started.
    pub fn require(&self, root: &Path) -> Result<()> {
        if self.exists(root)? && inside(root, &self.file("definitions.mjs"))?.is_file() {
            return Ok(());
        }
        Err(CliError::new(ErrorCode::ProjectNotFound, format!("No local icons in {}", self.icons))
            .with_path(self.icons.clone())
            .with_hint("Run gen-animated-icons create <id> to start one"))
    }

    /// Icon batches (`batches/<name>.mjs` with a kebab-case name), sorted bytewise.
    pub fn batch_ids(&self, root: &Path) -> Result<Vec<String>> {
        let dir = inside(root, &self.file("batches"))?;
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(CliError::io("read", &dir, &error)),
        };
        let mut ids: Vec<String> = entries
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| name.strip_suffix(".mjs").map(str::to_string))
            .filter(|stem| is_icon_name(stem))
            .collect();
        ids.sort();
        Ok(ids)
    }
}

/// `set-config.json` of a new project set (the library's shape; PY-JSON indent 2).
pub fn initial_set_config() -> String {
    let config = json!({
        "id": DEFAULT_SET_ID,
        "name": DEFAULT_SET_NAME,
        "category": DEFAULT_SET_NAME,
        "sharedSources": ["drawing.mjs", "direction.mjs", "ai-authoring.mjs"],
    });
    gai_core::to_py_json(&config, Some(2), true) + "\n"
}

/// `definitions.mjs` importing each icon batch in order.
pub fn definitions(ids: &[String]) -> String {
    let mut text = String::new();
    for (index, id) in ids.iter().enumerate() {
        text.push_str(&format!("import {{ icons as b{index} }} from './batches/{id}.mjs';\n"));
    }
    let spread: Vec<String> = (0..ids.len()).map(|index| format!("...b{index}")).collect();
    text.push_str(&format!("export const icons = [{}];\n", spread.join(", ")));
    text
}

/// The build folder of one icon.
pub fn build_dir(root: &Path, id: &str) -> Result<PathBuf> {
    inside(root, &format!("{BUILD_DIR}/{id}"))
}

/// A finished build: `set.json` and the Strong archive.
#[derive(Debug, Clone)]
pub struct Built {
    /// The build folder.
    pub dir: PathBuf,
    /// The packaged `set.json` (one icon).
    pub set: Value,
    /// Its bytes.
    pub set_bytes: Vec<u8>,
    /// The icon record inside it.
    pub icon: Value,
    /// Name of the Strong action.
    pub strong: String,
    /// The Strong `.lottie`.
    pub lottie: Vec<u8>,
}

impl Built {
    /// Reads the build of `id`; `None` when it was never built. Its files are read only as
    /// regular files inside the build folder (see [`read_inside`]).
    pub fn read(root: &Path, id: &str) -> Result<Option<Built>> {
        let dir = build_dir(root, id)?;
        let rebuild = format!("Run gen-animated-icons build {id}");
        let read = |relative: &str| {
            read_inside(&dir, relative).map_err(|error| {
                let shown = format!("{BUILD_DIR}/{id}/{relative}");
                CliError { message: format!("Cannot read the build of {id}: {}", error.message), ..error }
                    .with_path(shown)
                    .with_id(id)
                    .with_hint(&rebuild)
            })
        };
        let Some(set_bytes) = read("set.json")? else { return Ok(None) };
        let invalid = |detail: &str| {
            CliError::new(ErrorCode::InvalidConfig, format!("The build of {id} is incomplete: {detail}"))
                .with_id(id)
                .with_hint(&rebuild)
        };
        let set: Value = serde_json::from_slice(&set_bytes).map_err(|error| invalid(&error.to_string()))?;
        let icon = set
            .get("icons")
            .and_then(Value::as_array)
            .and_then(|icons| icons.iter().find(|icon| icon.get("name").and_then(Value::as_str) == Some(id)))
            .cloned()
            .ok_or_else(|| invalid("set.json lacks the icon"))?;
        let strong = strong_motion(&icon).ok_or_else(|| invalid("no Strong motion"))?;
        let name = strong.get("name").and_then(Value::as_str).ok_or_else(|| invalid("unnamed Strong motion"))?;
        let lottie = read(&format!("{name}.lottie"))?.ok_or_else(|| invalid("no Strong .lottie"))?;
        let strong = name.to_string();
        Ok(Some(Built { dir, set, set_bytes, icon, strong, lottie }))
    }
}

/// The Strong entry of an icon's `motions`.
pub fn strong_motion(icon: &Value) -> Option<&Value> {
    icon.get("motions")?.as_array()?.iter().find(|m| m.get("motionId").and_then(Value::as_str) == Some("strong"))
}

/// The sources a build of `icons_dir` snapshots: the set's config and rule files that
/// exist, its top-level `batches/*.mjs` (which the records hash) and `modules`, the
/// icons-relative paths of every module the loader read. Sorted bytewise.
pub fn snapshot_files(icons_dir: &Path, modules: &[String]) -> Result<Vec<String>> {
    let mut files: std::collections::BTreeSet<String> =
        SNAPSHOT_FILES.iter().filter(|name| icons_dir.join(name).is_file()).map(|name| name.to_string()).collect();
    files.extend(top_level_batches(icons_dir)?);
    files.extend(modules.iter().cloned());
    Ok(files.into_iter().collect())
}

/// `batches/<name>.mjs` for each regular file directly in `batches/`.
fn top_level_batches(icons_dir: &Path) -> Result<Vec<String>> {
    let dir = icons_dir.join("batches");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(CliError::io("read", &dir, &error)),
    };
    Ok(entries
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".mjs"))
        .map(|name| format!("batches/{name}"))
        .collect())
}

/// Copies the sources the build of icon `id` was made from into its folder `build` (see
/// [`snapshot_files`]) and lists them in [`SOURCES_FILE`] with the files the icon depends
/// on (see [`dependencies`]), so verification reads exactly the sources the records name
/// and later runs can tell when the icon's own sources changed. `build` must be a fresh
/// folder of the build's own.
pub fn snapshot(icons_dir: &Path, build: &Path, id: &str, modules: &[String]) -> Result<()> {
    let mut listed = serde_json::Map::new();
    for relative in snapshot_files(icons_dir, modules)? {
        let from = icons_dir.join(&relative);
        let bytes = std::fs::read(&from).map_err(|error| CliError::io("read", &from, &error))?;
        write_inside(build, &relative, &bytes)?;
        listed.insert(relative, json!(gai_core::sha256_hex(&bytes)));
    }
    let depends = dependencies(icons_dir, id, modules);
    let text = gai_core::to_js_json_pretty(&json!({ "files": listed, "depends": depends }), 2) + "\n";
    write_inside(build, SOURCES_FILE, text.as_bytes()).map(drop)
}

/// The icons-relative files whose change makes the build of `id` stale (see [`trace`]).
pub fn dependencies(icons_dir: &Path, id: &str, modules: &[String]) -> Vec<String> {
    trace(icons_dir, id, modules).depends
}

/// What [`trace`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    /// The files whose change makes the build stale, sorted bytewise.
    pub depends: Vec<String>,
    /// Whether every import was traced. When not, `depends` holds every module.
    pub complete: bool,
    /// Where tracing stopped, when it is not complete.
    pub untraced: Option<Untraced>,
}

/// Where [`trace`] stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untraced {
    /// The icons-relative module: the icon's batch, or a module it reaches.
    pub module: String,
    /// The import the scan could not follow, as written, and its 1-based line (see
    /// [`import_at`]); `None` when the module could not be read, or the icon's batch is not
    /// among the modules.
    pub import: Option<(String, usize)>,
}

/// The icons-relative files whose change makes the build of `id` stale: the set's config
/// and rule files, `definitions.mjs`, and `batches/<id>.mjs` with every module of
/// `modules` (the modules the loader read) it reaches through the imports in their text.
/// gai-js reports which modules it loaded but not which module imported which, so the
/// edges come from [`scan_imports`], resolved as the loader resolves them (symlinks
/// followed). Modules are read only as regular files inside `icons_dir`. When the icon has
/// no batch of its own among `modules`, or a module it reaches cannot be read or scanned
/// completely, the trace is incomplete and every module counts.
pub fn trace(icons_dir: &Path, id: &str, modules: &[String]) -> Trace {
    let mut depends: std::collections::BTreeSet<String> =
        SNAPSHOT_FILES.iter().filter(|name| icons_dir.join(name).is_file()).map(|name| name.to_string()).collect();
    let entry = format!("batches/{id}.mjs");
    let canonical_icons = icons_dir.canonicalize().ok();
    let mut queue = vec![entry.clone()];
    let mut reached = std::collections::BTreeSet::new();
    let mut traced = modules.contains(&entry) && canonical_icons.is_some();
    let mut untraced = (!traced).then(|| Untraced { module: entry.clone(), import: None });
    while let Some(module) = queue.pop().filter(|_| traced) {
        if !reached.insert(module.clone()) {
            continue;
        }
        let Ok(Some(bytes)) = read_inside(icons_dir, &module) else {
            traced = false;
            untraced = Some(Untraced { module, import: None });
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let imports = scan_imports(&text);
        traced = imports.complete;
        if let Some(at) = imports.lost {
            untraced = Some(Untraced { module: module.clone(), import: Some(import_at(&text, at)) });
        }
        let folder = icons_dir.join(&module);
        let folder = folder.parent().unwrap_or(icons_dir);
        for literal in imports.literals {
            let specifier = &text[literal];
            if !(specifier.starts_with("./") || specifier.starts_with("../")) {
                continue;
            }
            // The module the loader read for it: canonical, below the icons folder.
            let loaded = folder.join(specifier).canonicalize().ok().and_then(|path| {
                let relative = path.strip_prefix(canonical_icons.as_deref()?).ok()?;
                let parts: Option<Vec<&str>> = relative.components().map(|c| c.as_os_str().to_str()).collect();
                Some(parts?.join("/"))
            });
            if let Some(loaded) = loaded.filter(|path| modules.contains(path)) {
                queue.push(loaded);
            }
        }
    }
    match traced {
        true => depends.extend(reached),
        false => depends.extend(modules.iter().cloned()),
    }
    Trace { depends: depends.into_iter().collect(), complete: traced, untraced }
}

/// The import specifiers found in module text, and whether they are all of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imports {
    /// Byte ranges of the specifiers (inside their quotes) of `import … from '…'`,
    /// `export … from '…'`, `import '…'` and `import('…')`, in order.
    pub literals: Vec<std::ops::Range<usize>>,
    /// False when the text may import a module no range names: an `import(…)` of anything
    /// but a lone string literal, a quote that never closes, a specifier holding an escape
    /// (which names another module than its text, or hides where the literal ends), or a
    /// found specifier (or the comments before it) holding an `import` or `from` keyword,
    /// which happens when the scan read a comment or string as an import and may have
    /// skipped a real one.
    pub complete: bool,
    /// Where the scan first lost track, when it did: the byte offset of the `import` or
    /// `from` keyword it could not follow (0 for a text too long to scan).
    pub lost: Option<usize>,
}

/// Scans module text for import specifiers. The scan reads every `from` and `import`
/// word, including those in comments and strings: such a match adds a module at worst,
/// and one that could hide a real import makes the result incomplete. It takes time
/// linear in the text's length: where each gap of whitespace and comments ends is worked
/// out once ([`Gaps`]), and every other step moves forward.
pub fn scan_imports(text: &str) -> Imports {
    let bytes = text.as_bytes();
    let mut imports = Imports { literals: Vec::new(), complete: true, lost: None };
    let Some(gaps) = Gaps::of(text) else {
        imports.lose(0);
        return imports;
    };
    let mut at = 0;
    while at < bytes.len() {
        let keyword = ["from", "import"].into_iter().find(|keyword| bytes[at..].starts_with(keyword.as_bytes()));
        let starts_word = at == 0 || !is_identifier(bytes[at - 1]);
        let Some(keyword) = keyword.filter(|_| starts_word) else {
            at += 1;
            continue;
        };
        let after = at + keyword.len();
        if bytes.get(after).is_some_and(|&b| is_identifier(b)) {
            at = after;
            continue;
        }
        let mut quote = gaps.end(after);
        let dynamic = keyword == "import" && bytes.get(quote) == Some(&b'(');
        if dynamic {
            quote = gaps.end(quote + 1);
        }
        let Some(&delimiter) = bytes.get(quote).filter(|b| matches!(b, b'\'' | b'"')) else {
            // `import(expression)` names its module only when it runs.
            if dynamic {
                imports.lose(at);
            }
            at = after;
            continue;
        };
        let start = quote + 1;
        let Some(length) = text[start..].find(delimiter as char) else {
            imports.lose(at);
            break;
        };
        let end = start + length;
        if has_keyword(&text[after..quote]) || has_keyword(&text[start..end]) || text[start..end].contains('\\') {
            imports.lose(at);
        }
        if dynamic && !matches!(bytes.get(gaps.end(end + 1)), Some(b')' | b',')) {
            imports.lose(at);
        }
        imports.literals.push(start..end);
        at = end + 1;
    }
    imports
}

impl Imports {
    /// Marks the scan incomplete, at `at` unless it lost track earlier.
    fn lose(&mut self, at: usize) {
        self.complete = false;
        self.lost.get_or_insert(at);
    }
}

/// The import the scan lost track of at byte `at` of module text (an `import` or `from`
/// keyword), as written: from the statement's `import` or `export` (for a `from`) to the
/// end of the line `at` is on, whitespace collapsed, without a final `;`, at most 120
/// characters. Also the 1-based line it starts on.
pub fn import_at(text: &str, at: usize) -> (String, usize) {
    let at = (0..=at.min(text.len())).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0);
    let before = &text[..at];
    let keyword = |word: &str| {
        let starts_word = |i: usize| i == 0 || !is_identifier(before.as_bytes()[i - 1]);
        before.rmatch_indices(word).map(|(i, _)| i).find(|&i| starts_word(i))
    };
    let start = match text[at..].starts_with("from") {
        true => {
            let statement = [keyword("import"), keyword("export")].into_iter().flatten().max();
            statement.filter(|&start| !before[start..].contains(';')).unwrap_or(at)
        }
        false => at,
    };
    let end = at + text[at..].find(['\n', '\r']).unwrap_or(text.len() - at);
    let written = text[start..end].split_whitespace().collect::<Vec<_>>().join(" ");
    let written = written.strip_suffix(';').unwrap_or(&written).trim_end();
    let mut shown: String = written.chars().take(120).collect();
    if shown.len() < written.len() {
        shown.push('…');
    }
    (shown, text[..start].matches('\n').count() + 1)
}

/// Where each gap of whitespace and comments in a text ends.
struct Gaps(Vec<u32>);

impl Gaps {
    /// Works out [`Gaps::end`] for every character of `text` in one pass from its end;
    /// `None` for a text of 4 GiB or more.
    fn of(text: &str) -> Option<Gaps> {
        let len = u32::try_from(text.len()).ok().filter(|&len| len < u32::MAX)?;
        let bytes = text.as_bytes();
        let mut ends = vec![0u32; text.len() + 1];
        ends[text.len()] = len;
        // The first line terminator at or after the character just past this one, and
        // the first `*/` there and one character further on.
        let (mut newline, mut close, mut close_after) = (None, None, None);
        for (at, c) in text.char_indices().rev() {
            let next = at + c.len_utf8();
            let pair = |second: u8| bytes.get(at + 1) == Some(&second);
            ends[at] = if c.is_whitespace() || c == '\u{feff}' {
                ends[next]
            } else if c == '/' && pair(b'/') {
                // From the line's terminator, itself whitespace; the next one is past `//`.
                newline.map_or(len, |end: usize| ends[end])
            } else if c == '/' && pair(b'*') {
                // `*/` from two characters on, so `/*/` does not close itself.
                close_after.map_or(len, |end: usize| ends[end + 2])
            } else {
                at as u32
            };
            if matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                newline = Some(at);
            }
            close_after = close;
            if c == '*' && pair(b'/') {
                close = Some(at);
            }
        }
        Some(Gaps(ends))
    }

    /// The index of the first character at or after `from` (a character boundary) that is
    /// not whitespace or part of a comment; an unclosed block comment runs to the end.
    fn end(&self, from: usize) -> usize {
        self.0[from] as usize
    }
}

/// Whether `text` holds `import` or `from` as a keyword could stand: not inside a longer
/// name, and followed by what can follow it in an import (or nothing).
fn has_keyword(text: &str) -> bool {
    let bytes = text.as_bytes();
    ["import", "from"].into_iter().any(|keyword| {
        text.match_indices(keyword).any(|(at, _)| {
            let starts_word = at == 0 || !is_identifier(bytes[at - 1]);
            let next = text[at + keyword.len()..].chars().next();
            let ends_word = next.is_none_or(|c| c.is_whitespace() || matches!(c, '/' | '\'' | '"' | '{' | '*' | '('));
            starts_word && ends_word
        })
    })
}

/// A build's [`SOURCES_FILE`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Every snapshotted file, icons-relative, with its sha256.
    pub files: serde_json::Map<String, Value>,
    /// The files the icon depends on; `None` for a build made before builds tracked them.
    pub depends: Option<Vec<String>>,
}

impl Snapshot {
    /// The snapshot list of the build in `build`; `None` when it is missing, unreadable or
    /// not a regular file (see [`read_inside`]).
    pub fn read(build: &Path) -> Option<Snapshot> {
        let value: Value = serde_json::from_slice(&read_inside(build, SOURCES_FILE).ok()??).ok()?;
        let files = value.get("files")?.as_object()?.clone();
        let depends = value
            .get("depends")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect());
        Some(Snapshot { files, depends })
    }

    /// The sha256 recorded for `relative`.
    pub fn sha256(&self, relative: &str) -> Option<&str> {
        self.files.get(relative).and_then(Value::as_str)
    }
}

/// Why a build is stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// The first source that differs from the build's snapshot.
    pub file: String,
    /// The build lists no `depends` (or no snapshot at all): an earlier version built it,
    /// so any change to the sources makes it stale until it is rebuilt once.
    pub untracked: bool,
    /// Why `file` was not read: it is or runs through a symlink (`SYMLINK_PATH`), or it is
    /// not a regular file inside the icons folder (`INVALID_PATH`).
    pub refused: Option<ErrorCode>,
}

impl Stale {
    /// `<file>`, with why it was not read or why an untracked build went stale, for "…
    /// since it was built (…)".
    pub fn describe(&self) -> String {
        let file = match self.refused {
            None => self.file.clone(),
            Some(ErrorCode::SymlinkPath) => format!(
                "{} is or runs through a symlink, which is never followed; replace it with the file it points to",
                self.file
            ),
            Some(_) => format!("{} is not a regular file inside the icons folder", self.file),
        };
        match self.untracked {
            false => file,
            true => format!(
                "{file}; an earlier version built it and tracked every source file, so any change makes it stale until it is rebuilt once"
            ),
        }
    }
}

/// The first source that differs from the snapshot of the build in `build`: a file the
/// icon depends on that changed or is gone, or a config or rule file added since. A
/// `definitions.mjs` that `create` regenerated (it still only imports the batches, the
/// icon's among them) is not a change. `None` when the build is current. A build without
/// a snapshot list counts as changed; one without `depends` (an earlier format) depends
/// on every listed file and every top-level batch. A spec v2 build depends on its spec alone
/// (`<id>.icon.json`): it is current while the spec's sha256 is the one recorded at build time.
/// Sources are read only as regular files inside `icons_dir` (see [`read_inside`]): a listed
/// path that is not one is refused unread and counts as changed, so a planted entry can
/// neither leave the folder nor block.
pub fn changed_source(icons_dir: &Path, build: &Path) -> Option<Stale> {
    let stale =
        |file: &str, untracked: bool, refused: Option<ErrorCode>| Stale { file: file.to_string(), untracked, refused };
    let Some(snapshot) = Snapshot::read(build) else { return Some(stale(SOURCES_FILE, true, None)) };
    let listed = &snapshot.files;
    // Its sha256 now (`None` when gone), or the code it was refused with.
    let current = |relative: &str| match read_inside(icons_dir, relative) {
        Ok(bytes) => Ok(bytes.map(gai_core::sha256_hex)),
        Err(error) => Err(error.code),
    };
    let Some(depends) = &snapshot.depends else {
        for (relative, sha) in listed {
            match current(relative) {
                Ok(now) if now.as_deref() == sha.as_str() => {}
                Ok(_) => return Some(stale(relative, true, None)),
                Err(code) => return Some(stale(relative, true, Some(code))),
            }
        }
        let now = snapshot_files(icons_dir, &[]).unwrap_or_default();
        return now.into_iter().find(|relative| !listed.contains_key(relative)).map(|file| stale(&file, true, None));
    };
    let depends: Vec<&str> = depends.iter().map(String::as_str).collect();
    for relative in &depends {
        match current(relative) {
            Ok(now) if now.as_deref() == snapshot.sha256(relative) => continue,
            Ok(_) => {}
            Err(code) => return Some(stale(relative, false, Some(code))),
        }
        if *relative == "definitions.mjs" && regenerated_definitions(icons_dir, build, &depends) {
            continue;
        }
        return Some(stale(relative, false, None));
    }
    // A v2 build reads none of the set's v1 config and rule files.
    if depends.iter().any(|relative| relative.ends_with(gai_spec::SUFFIX)) {
        return None;
    }
    let added = SNAPSHOT_FILES.iter().filter(|name| **name != "definitions.mjs");
    added
        .filter(|name| icons_dir.join(name).is_file() && !listed.contains_key(**name))
        .map(|name| stale(name, false, None))
        .next()
}

/// The icon ids a `definitions.mjs` text imports when it is [`definitions`] output.
pub fn generated_definitions(text: &str) -> Option<Vec<String>> {
    let ids: Vec<String> = text
        .lines()
        .filter_map(|line| line.split_once(" from './batches/")?.1.strip_suffix(".mjs';").map(str::to_string))
        .collect();
    (definitions(&ids) == text).then_some(ids)
}

/// Whether `definitions.mjs` changed only as `create` rewrites it: the build's copy and the
/// current file are both [`definitions`] output, and every batch the icon depends on that
/// the build's copy imported is still imported.
fn regenerated_definitions(icons_dir: &Path, build: &Path, depends: &[&str]) -> bool {
    let generated = |dir: &Path| {
        let bytes = read_inside(dir, "definitions.mjs").ok()??;
        generated_definitions(std::str::from_utf8(&bytes).ok()?)
    };
    let (Some(before), Some(now)) = (generated(build), generated(icons_dir)) else { return false };
    before.iter().filter(|id| depends.contains(&format!("batches/{id}.mjs").as_str())).all(|id| now.contains(id))
}

/// Rewrites the specifiers [`scan_imports`] finds in module text. `map` returns the
/// replacement, or `None` to keep one.
pub fn rewrite_imports(text: &str, mut map: impl FnMut(&str) -> Result<Option<String>>) -> Result<String> {
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    for literal in scan_imports(text).literals {
        if let Some(replacement) = map(&text[literal.clone()])? {
            out.push_str(&text[copied..literal.start]);
            out.push_str(&replacement);
            copied = literal.end;
        }
    }
    out.push_str(&text[copied..]);
    Ok(out)
}

fn is_identifier(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

/// Lexically resolves the relative `specifier` against the `/`-separated folder `base`;
/// `None` when it climbs above the first component of `base`.
pub fn resolve_relative(base: &str, specifier: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for part in specifier.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_names_are_kebab_case() {
        for good in ["a", "heart", "folder-open", "a1-b2"] {
            assert!(is_icon_name(good), "{good}");
        }
        for bad in ["", "1a", "-a", "a-", "a--b", "A", "a_b", "_ref"] {
            assert!(!is_icon_name(bad), "{bad}");
        }
    }

    #[test]
    fn new_ids_must_name_portable_files() {
        for good in ["heart", "console", "com", "com0", "lpt10", "con-icon", "nullable"] {
            assert_eq!(new_id_error(good), None, "{good}");
        }
        // Kebab-case, but a Windows device name: submit could never bundle its files.
        for reserved in ["con", "prn", "aux", "nul", "com1", "com9", "lpt1", "lpt9"] {
            let error = new_id_error(reserved).unwrap_or_else(|| panic!("{reserved}"));
            assert_eq!((error.code, error.id.as_deref()), (ErrorCode::Usage, Some(reserved)));
            assert_eq!(
                error.message,
                format!(
                    "{reserved} cannot be an icon id: Windows reserves it as a device name (con, prn, aux, nul, com1 to com9, lpt1 to lpt9), so submit could never bundle its files"
                )
            );
            assert_eq!(error.hint, Some(format!("Choose another id, such as {reserved}-icon")));
        }
        let error = new_id_error("A_b").unwrap();
        assert_eq!(error.message, "Use a kebab-case icon id such as payment-sent, not A_b");
    }

    #[test]
    fn definitions_import_each_batch() {
        assert_eq!(
            definitions(&["dot".into()]),
            "import { icons as b0 } from './batches/dot.mjs';\nexport const icons = [...b0];\n"
        );
        assert_eq!(
            definitions(&["a".into(), "b".into()]),
            "import { icons as b0 } from './batches/a.mjs';\nimport { icons as b1 } from './batches/b.mjs';\nexport const icons = [...b0, ...b1];\n"
        );
        assert_eq!(definitions(&[]), "export const icons = [];\n");
    }

    #[test]
    fn set_config_is_python_json() {
        assert_eq!(
            initial_set_config(),
            "{\n  \"id\": \"00-local\",\n  \"name\": \"Local icons\",\n  \"category\": \"Local icons\",\n  \"sharedSources\": [\n    \"drawing.mjs\",\n    \"direction.mjs\",\n    \"ai-authoring.mjs\"\n  ]\n}\n"
        );
    }

    #[test]
    fn imports_are_rewritten_in_place() {
        let text = "import {a} from './helpers.mjs';\nexport * from \"../../library/drawing.mjs\";\nimport svgpath from 'svgpath';\n// 90° turn\nconst imported = 1; import('./lazy.mjs');\nimport './side.mjs';\n";
        let out =
            rewrite_imports(text, |spec| Ok(spec.starts_with("./").then(|| format!("./_x.{}", &spec[2..])))).unwrap();
        assert_eq!(
            out,
            "import {a} from './_x.helpers.mjs';\nexport * from \"../../library/drawing.mjs\";\nimport svgpath from 'svgpath';\n// 90° turn\nconst imported = 1; import('./_x.lazy.mjs');\nimport './_x.side.mjs';\n"
        );
    }

    #[test]
    fn a_v2_build_is_stale_only_when_its_spec_changes() {
        let dir = tempfile::tempdir().unwrap();
        let (icons, build) = (dir.path().join("icons"), dir.path().join("build"));
        std::fs::create_dir_all(&icons).unwrap();
        std::fs::create_dir_all(&build).unwrap();
        let spec = "{\"format\": \"gai-icon/2\", \"id\": \"bell\"}\n";
        std::fs::write(icons.join("bell.icon.json"), spec).unwrap();
        // What a v2 build records: its spec alone.
        let listed = json!({"files": {"bell.icon.json": gai_core::sha256_hex(spec)}, "depends": ["bell.icon.json"]});
        std::fs::write(build.join(SOURCES_FILE), listed.to_string()).unwrap();
        assert_eq!(changed_source(&icons, &build), None);
        // The set's v1 config and rule files, and other specs, are not its sources.
        std::fs::write(icons.join("set-config.json"), "{}\n").unwrap();
        std::fs::write(icons.join("verify.rules.json"), "{}\n").unwrap();
        std::fs::write(icons.join("other.icon.json"), "{}\n").unwrap();
        assert_eq!(changed_source(&icons, &build), None);
        std::fs::write(icons.join("bell.icon.json"), spec.replace("bell", "bell ")).unwrap();
        let stale = changed_source(&icons, &build).unwrap();
        assert_eq!((stale.file.as_str(), stale.untracked, stale.refused), ("bell.icon.json", false, None));
        std::fs::remove_file(icons.join("bell.icon.json")).unwrap();
        assert_eq!(changed_source(&icons, &build).unwrap().file, "bell.icon.json");
    }

    #[test]
    fn snapshots_list_every_module_and_notice_changes() {
        let dir = tempfile::tempdir().unwrap();
        let (icons, build) = (dir.path().join("icons"), dir.path().join("build"));
        let write = |relative: &str, text: &str| {
            let file = icons.join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("definitions.mjs", &definitions(&["a".into(), "c".into()]));
        write("set-config.json", "{}\n");
        write("batches/a.mjs", "import {b} from './shared/b.mjs';\nimport {k} from '../../library/drawing.mjs';\n");
        write("batches/shared/b.mjs", "export const b = 1;\n");
        write("batches/c.mjs", "import {b} from './shared/b.mjs';\n");
        write("batches/unused.mjs", "export {};\n");
        std::fs::create_dir_all(&build).unwrap();
        let modules: Vec<String> =
            ["batches/a.mjs", "batches/c.mjs", "batches/shared/b.mjs", "definitions.mjs"].map(String::from).to_vec();
        snapshot(&icons, &build, "a", &modules).unwrap();
        let stale = || changed_source(&icons, &build).map(|stale| (stale.file, stale.untracked));
        let listed: Value = serde_json::from_slice(&std::fs::read(build.join(SOURCES_FILE)).unwrap()).unwrap();
        let files: Vec<&String> = listed["files"].as_object().unwrap().keys().collect();
        assert_eq!(
            files,
            [
                "batches/a.mjs",
                "batches/c.mjs",
                "batches/shared/b.mjs",
                "batches/unused.mjs",
                "definitions.mjs",
                "set-config.json"
            ]
        );
        assert_eq!(
            listed["depends"],
            json!(["batches/a.mjs", "batches/shared/b.mjs", "definitions.mjs", "set-config.json"])
        );
        assert_eq!(std::fs::read_to_string(build.join("batches/shared/b.mjs")).unwrap(), "export const b = 1;\n");
        assert_eq!(stale(), None);

        // Another icon's batch, a new batch and create's rewrite of definitions.mjs leave it current.
        write("batches/c.mjs", "import {b} from './shared/b.mjs';\nexport const c = 2;\n");
        write("batches/new.mjs", "export {};\n");
        write("definitions.mjs", &definitions(&["a".into(), "c".into(), "new".into()]));
        assert_eq!(stale(), None);
        // Its own helpers, the set's files and a hand-edited definitions.mjs do not.
        write("batches/shared/b.mjs", "export const b = 2;\n");
        assert_eq!(stale(), Some(("batches/shared/b.mjs".to_string(), false)));
        write("batches/shared/b.mjs", "export const b = 1;\n");
        write("definitions.mjs", "import {icons} from './batches/a.mjs';\nexport {icons};\n");
        assert_eq!(stale(), Some(("definitions.mjs".to_string(), false)));
        write("definitions.mjs", &definitions(&["c".into()]));
        assert_eq!(stale(), Some(("definitions.mjs".to_string(), false)));
        write("definitions.mjs", &definitions(&["a".into(), "c".into()]));
        write("verify.rules.json", "{}\n");
        assert_eq!(stale(), Some(("verify.rules.json".to_string(), false)));
        std::fs::remove_file(icons.join("verify.rules.json")).unwrap();
        assert_eq!(stale(), None);

        // A snapshot without depends (an earlier build) depends on every file.
        let mut old = listed.clone();
        old.as_object_mut().unwrap().remove("depends");
        std::fs::write(build.join(SOURCES_FILE), old.to_string()).unwrap();
        assert_eq!(stale(), Some(("batches/c.mjs".to_string(), true)));
        let described = changed_source(&icons, &build).unwrap().describe();
        assert!(
            described.starts_with("batches/c.mjs; an earlier version built it")
                && described.ends_with("until it is rebuilt once"),
            "{described}"
        );
        std::fs::remove_file(build.join(SOURCES_FILE)).unwrap();
        assert_eq!(stale(), Some((SOURCES_FILE.to_string(), true)));
    }

    #[test]
    fn an_icon_outside_its_own_batch_depends_on_every_module() {
        let dir = tempfile::tempdir().unwrap();
        let modules = ["batches/both.mjs".to_string(), "definitions.mjs".into()];
        assert_eq!(dependencies(dir.path(), "a", &modules), modules);
    }

    #[test]
    fn a_computed_import_depends_on_every_module() {
        let dir = tempfile::tempdir().unwrap();
        let write = |relative: &str, text: &str| {
            let file = dir.path().join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("batches/a.mjs", "import {h} from './shared/h.mjs';\n");
        write("batches/shared/h.mjs", "export const h = await import('./' + name);\n");
        write("batches/shared/x.mjs", "export {};\n");
        let modules: Vec<String> = ["batches/a.mjs", "batches/b.mjs", "batches/shared/h.mjs", "batches/shared/x.mjs"]
            .map(String::from)
            .to_vec();
        assert_eq!(dependencies(dir.path(), "a", &modules), modules);
        write("batches/shared/h.mjs", "export const h = await import ( './x.mjs');\nconst reimport = 1;\n");
        assert_eq!(
            dependencies(dir.path(), "a", &modules),
            ["batches/a.mjs", "batches/shared/h.mjs", "batches/shared/x.mjs"]
        );
        let complete = |text: &str| scan_imports(text).complete;
        assert!(!complete("import(`./${x}.mjs`)") && !complete("import(\"./\" + x)") && !complete("import('./a' + x)"));
        assert!(complete("import \"./a.mjs\"; reimport(x)") && complete("import ('./a.mjs', {})"));
        assert!(complete("import(/* chunk */ './a.mjs')") && complete("const meta = import.meta;"));
    }

    /// The modules each specifier the scan finds names.
    fn specifiers(text: &str) -> (Vec<&str>, bool) {
        let imports = scan_imports(text);
        (imports.literals.into_iter().map(|range| &text[range]).collect(), imports.complete)
    }

    #[test]
    fn imports_after_comments_are_found() {
        let text = "import {a} from /* the helper */ './a.mjs';\nexport * from // why\n  \"./b.mjs\";\nimport(/* lazy */ './c.mjs');\n";
        assert_eq!(specifiers(text), (vec!["./a.mjs", "./b.mjs", "./c.mjs"], true));
        assert_eq!(specifiers("import{x}from'./x.mjs';export*from\"./y.mjs\""), (vec!["./x.mjs", "./y.mjs"], true));
        // Words in prose and names stay harmless.
        let prose = "const intent = \"Moves from 'inbox' to the archive\"; Array.from(list); const important = 1;\nimport {k} from './from-k.mjs';\n";
        assert_eq!(specifiers(prose), (vec!["inbox", "./from-k.mjs"], true));
    }

    #[test]
    fn a_scan_that_may_have_skipped_an_import_is_incomplete() {
        // A comment read as an import swallows the real one after it.
        let swallowed = "/* moved from 'old */import{b}from'./b.mjs';\n";
        assert!(!specifiers(swallowed).1, "{:?}", specifiers(swallowed));
        let across = "// can't import 'em all\nimport {a} from \"./a.mjs\";\nconst s = 'x';\n";
        assert!(!specifiers(across).1, "{:?}", specifiers(across));
        let comment = "const s = \"from /*\"; import {c} from './c.mjs'; /* */ './b.mjs';\n";
        assert!(!specifiers(comment).1, "{:?}", specifiers(comment));
        assert!(!specifiers("import {a} from './a.mjs';\n// from 'unterminated\n").1);
    }

    #[test]
    fn untraced_imports_are_named_as_written() {
        let named = |text: &str| {
            let lost = scan_imports(text).lost.expect("incomplete");
            import_at(text, lost)
        };
        assert_eq!(named("import {h} from '.\\x2fh.mjs';\n"), ("import {h} from '.\\x2fh.mjs'".to_string(), 1));
        assert_eq!(
            named("// a\nexport {\n  a,\n  b,\n} from \"./a\\u002e.mjs\";\n"),
            ("export { a, b, } from \"./a\\u002e.mjs\"".to_string(), 2)
        );
        assert_eq!(named("const x = 1;\nconst lazy = name => import(name);\n"), ("import(name)".to_string(), 2));
        // A `from` in a comment, with no statement keyword before it.
        assert_eq!(named("/* moved from 'old */import{b}from'./b.mjs';"), ("from 'old */import{b}from'./b.mjs'".to_string(), 1));
        assert_eq!(scan_imports("import {a} from './a.mjs';\n").lost, None);
        // The first place it lost track, and what trace reports.
        let dir = tempfile::tempdir().unwrap();
        let write = |relative: &str, text: &str| {
            let file = dir.path().join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("batches/a.mjs", "import {h} from './shared/h.mjs';\n");
        write("batches/shared/h.mjs", "export const h = 1;\nexport const l = n => import(n);\nimport('./' + n);\n");
        let modules = ["batches/a.mjs".to_string(), "batches/shared/h.mjs".into()];
        let traced = trace(dir.path(), "a", &modules);
        assert!(!traced.complete);
        assert_eq!(
            traced.untraced,
            Some(Untraced { module: "batches/shared/h.mjs".into(), import: Some(("import(n)".into(), 2)) })
        );
        // An icon outside its own batch has nothing to name.
        let elsewhere = trace(dir.path(), "b", &modules);
        assert_eq!(elsewhere.untraced, Some(Untraced { module: "batches/b.mjs".into(), import: None }));
        write("batches/shared/h.mjs", "export const h = 1;\n");
        assert_eq!(trace(dir.path(), "a", &modules).untraced, None);
    }

    #[test]
    fn an_escaped_specifier_makes_the_scan_incomplete() {
        // The loader decodes these to './shared/h.mjs', './a\'b.mjs' and so on.
        for text in [
            "import {h} from '.\\x2fshared/h.mjs';\n",
            "import {h} from \"./shared\\u002fh.mjs\";\n",
            "export * from './shared/\\u{68}.mjs';\n",
            "import('.\\/h.mjs');\n",
            "import x from './a\\'; import y from './b.mjs';\n",
            "import './a\\\n.mjs';\n",
        ] {
            assert!(!scan_imports(text).complete, "{text:?}");
        }
        let dir = tempfile::tempdir().unwrap();
        let write = |relative: &str, text: &str| {
            let file = dir.path().join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("batches/a.mjs", "import {h} from '.\\x2fshared/h.mjs';\n");
        write("batches/shared/h.mjs", "export const h = 1;\n");
        write("batches/b.mjs", "export {};\n");
        let modules: Vec<String> =
            ["batches/a.mjs", "batches/b.mjs", "batches/shared/h.mjs"].map(String::from).to_vec();
        assert_eq!(dependencies(dir.path(), "a", &modules), modules);
    }

    /// Runs `call` in a thread and fails when it takes longer than 5 s, as a read blocked
    /// on a FIFO does; the FIFOs in `fifos` are then opened for writing to free it.
    #[cfg(unix)]
    fn returns_promptly<T: Send + 'static>(fifos: &[PathBuf], call: impl FnOnce() -> T + Send + 'static) -> T {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || send.send(call()));
        receive.recv_timeout(std::time::Duration::from_secs(5)).unwrap_or_else(|_| {
            for fifo in fifos.iter().cloned() {
                std::thread::spawn(move || std::fs::OpenOptions::new().write(true).open(fifo));
            }
            panic!("the call blocked on a planted path");
        })
    }

    #[cfg(unix)]
    fn mkfifo(path: &Path) {
        assert!(std::process::Command::new("mkfifo").arg(path).status().unwrap().success());
    }

    #[cfg(unix)]
    #[test]
    fn planted_snapshot_paths_are_never_read() {
        let dir = tempfile::tempdir().unwrap();
        let (icons, build) = (dir.path().join("icons"), dir.path().join("build"));
        let write = |relative: &str, text: &str| {
            let file = icons.join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("definitions.mjs", &definitions(&["a".into()]));
        write("set-config.json", "{}\n");
        write("batches/a.mjs", "export const icons = [];\n");
        let modules = ["batches/a.mjs".to_string(), "definitions.mjs".into()];
        let fresh = || {
            let _ = std::fs::remove_dir_all(&build);
            std::fs::create_dir_all(&build).unwrap();
            snapshot(&icons, &build, "a", &modules).unwrap();
            serde_json::from_slice::<Value>(&std::fs::read(build.join(SOURCES_FILE)).unwrap()).unwrap()
        };
        let outside = dir.path().join("outside");
        mkfifo(&outside);
        mkfifo(&icons.join("batches/fifo.mjs"));
        let stale = |fifos: &[PathBuf]| {
            let (icons, build) = (icons.clone(), build.clone());
            let stale = returns_promptly(fifos, move || changed_source(&icons, &build));
            stale.map(|stale| (stale.file, stale.refused))
        };
        let refused = |file: &str, code| Some((file.to_string(), Some(code)));
        let fifos = [outside.clone(), icons.join("batches/fifo.mjs")];

        // A depends entry outside the icons folder, climbing out of it, or a FIFO in it.
        for planted in [outside.display().to_string(), "../outside".into(), "batches/fifo.mjs".into()] {
            let mut listing = fresh();
            listing["depends"].as_array_mut().unwrap().push(json!(planted));
            listing["files"][&planted] = json!("0".repeat(64));
            std::fs::write(build.join(SOURCES_FILE), listing.to_string()).unwrap();
            assert_eq!(stale(&fifos), refused(&planted, ErrorCode::InvalidPath), "{planted}");
        }
        // A source file replaced by a symlink to a FIFO.
        fresh();
        std::fs::rename(icons.join("set-config.json"), dir.path().join("set-config.json")).unwrap();
        std::os::unix::fs::symlink(&outside, icons.join("set-config.json")).unwrap();
        assert_eq!(stale(&fifos), refused("set-config.json", ErrorCode::SymlinkPath));
        let described = changed_source(&icons, &build).unwrap().describe();
        assert_eq!(
            described,
            "set-config.json is or runs through a symlink, which is never followed; replace it with the file it points to"
        );
        std::fs::remove_file(icons.join("set-config.json")).unwrap();
        std::fs::rename(dir.path().join("set-config.json"), icons.join("set-config.json")).unwrap();
        // The build's copy of definitions.mjs, read to tell create's rewrite from an edit.
        fresh();
        std::fs::remove_file(build.join("definitions.mjs")).unwrap();
        mkfifo(&build.join("definitions.mjs"));
        write("definitions.mjs", &definitions(&["a".into(), "b".into()]));
        let fifos = [build.join("definitions.mjs")];
        assert_eq!(stale(&fifos), Some(("definitions.mjs".to_string(), None)));
        write("definitions.mjs", &definitions(&["a".into()]));
        // The snapshot list itself.
        fresh();
        std::fs::remove_file(build.join(SOURCES_FILE)).unwrap();
        mkfifo(&build.join(SOURCES_FILE));
        let fifos = [build.join(SOURCES_FILE)];
        assert_eq!(stale(&fifos), Some((SOURCES_FILE.to_string(), None)));
        assert!(returns_promptly(&fifos, move || Snapshot::read(&build)).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn a_build_is_read_only_from_regular_files_inside_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let build = root.join(BUILD_DIR).join("a");
        std::fs::create_dir_all(&build).unwrap();
        let set = |strong: &str| {
            json!({"icons": [{"name": "a", "motions": [{"motionId": "strong", "name": strong}]}]}).to_string()
        };
        let read = |fifos: &[PathBuf]| {
            let root = root.clone();
            returns_promptly(fifos, move || Built::read(&root, "a").map(|built| built.map(|b| b.lottie)))
        };
        std::fs::write(build.join("set.json"), set("a-strong")).unwrap();
        std::fs::write(build.join("a-strong.lottie"), b"zip").unwrap();
        assert_eq!(read(&[]).unwrap(), Some(b"zip".to_vec()));
        // A Strong action named to climb out of the build.
        std::fs::write(root.join("escape.lottie"), b"outside").unwrap();
        std::fs::write(build.join("set.json"), set("../../../escape")).unwrap();
        assert_eq!(read(&[]).unwrap_err().code, ErrorCode::InvalidPath);
        // A FIFO, or a symlink, as set.json or as the archive.
        std::fs::write(build.join("set.json"), set("a-strong")).unwrap();
        std::fs::remove_file(build.join("a-strong.lottie")).unwrap();
        mkfifo(&build.join("a-strong.lottie"));
        assert_eq!(read(&[build.join("a-strong.lottie")]).unwrap_err().code, ErrorCode::InvalidPath);
        std::fs::remove_file(build.join("a-strong.lottie")).unwrap();
        std::os::unix::fs::symlink(root.join("escape.lottie"), build.join("a-strong.lottie")).unwrap();
        assert_eq!(read(&[]).unwrap_err().code, ErrorCode::SymlinkPath);
        std::fs::remove_file(build.join("set.json")).unwrap();
        mkfifo(&build.join("set.json"));
        assert_eq!(read(&[build.join("set.json")]).unwrap_err().code, ErrorCode::InvalidPath);
    }

    #[test]
    fn helpers_after_comments_and_through_symlinks_are_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        let write = |relative: &str, text: &str| {
            let file = dir.path().join(relative);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        };
        write("batches/a.mjs", "import {h} from /* shared */ './shared/h.mjs';\n");
        write("batches/shared/h.mjs", "export const h = 1;\n");
        write("batches/b.mjs", "export {};\n");
        let modules: Vec<String> =
            ["batches/a.mjs", "batches/b.mjs", "batches/shared/h.mjs"].map(String::from).to_vec();
        assert_eq!(dependencies(dir.path(), "a", &modules), ["batches/a.mjs", "batches/shared/h.mjs"]);
        // A swallowed import makes the icon depend on every module.
        write("batches/a.mjs", "/* moved from 'old */import{h}from'./shared/h.mjs';\n");
        assert_eq!(dependencies(dir.path(), "a", &modules), modules);
        // The loader follows symlinks inside the icons folder; so do the edges.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("shared/h.mjs", dir.path().join("batches/link.mjs")).unwrap();
            write("batches/a.mjs", "import {h} from './link.mjs';\n");
            assert_eq!(dependencies(dir.path(), "a", &modules), ["batches/a.mjs", "batches/shared/h.mjs"]);
        }
    }

    /// Runs `scan` on `text` in a thread and fails when it takes longer than `limit`.
    fn finishes_within(text: String, limit: std::time::Duration) -> Imports {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || send.send(scan_imports(&text)));
        receive.recv_timeout(limit).expect("scan_imports took too long")
    }

    #[test]
    fn scans_of_keyword_dense_text_stay_linear() {
        let limit = std::time::Duration::from_secs(2);
        // Each pattern was quadratic: every keyword rescanned the comments after it.
        for unit in ["import /*", "from /*", "//import\n", "import(/**/", "from /**/ /**/ //\n", "import \u{2028}//"] {
            let text = unit.repeat(400_000 / unit.len());
            let scanned = finishes_within(text, limit);
            assert!(scanned.literals.is_empty(), "{unit:?}");
        }
        let text = "import x from './a.mjs';\n".repeat(20_000);
        let scanned = finishes_within(text, limit);
        assert_eq!((scanned.literals.len(), scanned.complete), (20_000, true));
    }

    /// The scan as it was before it was made linear: the reference for its results.
    fn quadratic_scan(text: &str) -> Imports {
        fn skip(text: &str, mut from: usize) -> usize {
            loop {
                let rest = &text[from..];
                let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
                from += rest.len() - trimmed.len();
                if trimmed.starts_with("//") {
                    from += trimmed.find(['\n', '\r', '\u{2028}', '\u{2029}']).unwrap_or(trimmed.len());
                } else if let Some(comment) = trimmed.strip_prefix("/*") {
                    from += comment.find("*/").map_or(trimmed.len(), |end| 2 + end + 2);
                } else {
                    return from;
                }
            }
        }
        let bytes = text.as_bytes();
        let mut imports = Imports { literals: Vec::new(), complete: true, lost: None };
        let mut at = 0;
        while at < bytes.len() {
            let keyword = ["from", "import"].into_iter().find(|keyword| bytes[at..].starts_with(keyword.as_bytes()));
            let starts_word = at == 0 || !is_identifier(bytes[at - 1]);
            let Some(keyword) = keyword.filter(|_| starts_word) else {
                at += 1;
                continue;
            };
            let after = at + keyword.len();
            if bytes.get(after).is_some_and(|&b| is_identifier(b)) {
                at = after;
                continue;
            }
            let mut quote = skip(text, after);
            let dynamic = keyword == "import" && bytes.get(quote) == Some(&b'(');
            if dynamic {
                quote = skip(text, quote + 1);
            }
            let Some(&delimiter) = bytes.get(quote).filter(|b| matches!(b, b'\'' | b'"')) else {
                if dynamic {
                    imports.lose(at);
                }
                at = after;
                continue;
            };
            let start = quote + 1;
            let Some(length) = text[start..].find(delimiter as char) else {
                imports.lose(at);
                break;
            };
            let end = start + length;
            if has_keyword(&text[after..quote]) || has_keyword(&text[start..end]) || text[start..end].contains('\\') {
                imports.lose(at);
            }
            if dynamic && !matches!(bytes.get(skip(text, end + 1)), Some(b')' | b',')) {
                imports.lose(at);
            }
            imports.literals.push(start..end);
            at = end + 1;
        }
        imports
    }

    #[test]
    fn the_linear_scan_finds_what_the_quadratic_one_did() {
        let pieces = [
            "import", "from", "export", " ", "\n", "\r", "\t", "\u{2028}", "\u{feff}", "é", "//", "/*", "*/", "/", "*",
            "'", "\"", "(", ")", ",", "{a}", "x", "./a.mjs", "imports", "_from", "$import", "`", ";", "\\",
        ];
        // A small xorshift generator, so the cases are the same on every run.
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..20_000 {
            let count = (next() % 24) as usize;
            let text: String = (0..count).map(|_| pieces[(next() % pieces.len() as u64) as usize]).collect();
            assert_eq!(scan_imports(&text), quadratic_scan(&text), "{text:?}");
        }
    }

    #[test]
    fn relative_specifiers_resolve_lexically() {
        assert_eq!(
            resolve_relative("46-files/batches", "../../library/drawing.mjs").as_deref(),
            Some("library/drawing.mjs")
        );
        assert_eq!(resolve_relative("46-files", "./batches/a.mjs").as_deref(), Some("46-files/batches/a.mjs"));
        assert_eq!(resolve_relative("46-files", "../../../x.mjs"), None);
    }
}
