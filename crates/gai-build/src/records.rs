//! Package records (builder spec §6.8): one per animation, in the exact key order of the
//! old builders, with the provenance hashes of §6.11.
//!
//! Verbatim fields (labels, `brief`, `constraints`, `keyPoses`, …) come from the variant's
//! `JSON.stringify` projection, so `undefined` is already dropped and key order is JS's.
//! Records are built as JSON values whose numbers print identically through the JS
//! (`JSON.stringify`) and Python (`json.dumps`) writers.

use crate::post_export::{PRIMARY_DARK, PRIMARY_LIGHT, js_number};
use gai_core::js::truthy;
use crate::profile::{Continuous, Profile, SOURCE_FILE, SetConfig};
use gai_core::sha256_hex;
use gai_js::{DURATION, FPS, Icon, Variant};
use serde_json::{Map, Value, json};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Why a record could not be made.
#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("cannot read {}: {source}", path.display())]
    Read { path: PathBuf, source: std::io::Error },
    #[error("{} contains a file name that is not UTF-8", .0.display())]
    FileName(PathBuf),
    #[error("{animation}: rest is not a finite number")]
    Rest { animation: String },
}

/// The provenance hashes every record of a set shares (§6.11), each the sha256 of raw
/// file bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// `directionSources`: each kit file of [`Profile::direction_sources`].
    pub direction_sources: Map<String, Value>,
    /// `sourceSha256`: the set's `build.mjs`, or [`builder_identity`] once it is gone.
    pub source_sha256: String,
    /// The builder hash key and value, when the profile has one.
    pub builder_sha256: Option<(&'static str, String)>,
    /// `authoringSources`: `set-config.json`, then every `batches/*.mjs` in byte order.
    pub authoring_sources: Option<Map<String, Value>>,
    /// `definitionsSha256`.
    pub definitions_sha256: String,
}

impl Provenance {
    /// Hashes the set folder `root` and the kit folder `library`.
    pub fn collect(profile: &Profile, root: &Path, library: &Path) -> Result<Provenance, RecordError> {
        let direction_sources = js_object(
            profile
                .direction_sources
                .iter()
                .map(|name| Ok((name.clone(), hash(&library.join(name))?.into())))
                .collect::<Result<Vec<_>, RecordError>>()?,
        );
        let builder_sha256 = match profile.builder_hash {
            Some(builder) => Some((builder.key(), hash_or_identity(&builder.file(root, library))?)),
            None => None,
        };
        let authoring_sources = if profile.authoring { Some(authoring_sources(root)?) } else { None };
        Ok(Provenance {
            direction_sources,
            source_sha256: hash_or_identity(&root.join(SOURCE_FILE))?,
            builder_sha256,
            authoring_sources,
            definitions_sha256: hash(&root.join("definitions.mjs"))?,
        })
    }
}

/// What a set built without its own `build.mjs` records as its builder: this crate's
/// version and the JS engine that sampled it. `sourceSha256` and the builder hash are the
/// sha256 of this text ([`builder_identity_sha256`]).
pub fn builder_identity() -> String {
    format!("gai-build {}; {}; {}", env!("CARGO_PKG_VERSION"), gai_js::ENGINE, gai_js::math())
}

/// sha256 of [`builder_identity`].
pub fn builder_identity_sha256() -> String {
    sha256_hex(builder_identity())
}

fn hash(path: &Path) -> Result<String, RecordError> {
    std::fs::read(path).map(sha256_hex).map_err(|source| RecordError::Read { path: path.to_path_buf(), source })
}

/// The file's hash, or the builder identity's when the file does not exist.
fn hash_or_identity(path: &Path) -> Result<String, RecordError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(sha256_hex(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(builder_identity_sha256()),
        Err(source) => Err(RecordError::Read { path: path.to_path_buf(), source }),
    }
}

/// `authoringSources`: `set-config.json`, then each `batches/<n>` whose name ends in `.mjs`,
/// sorted byte-wise as Node's `readdirSync` returns them. A missing `batches/` is an error,
/// as it was in JS.
fn authoring_sources(root: &Path) -> Result<Map<String, Value>, RecordError> {
    let batches = root.join("batches");
    let entries = std::fs::read_dir(&batches).map_err(|source| RecordError::Read { path: batches.clone(), source })?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| RecordError::Read { path: batches.clone(), source })?;
        let name = entry.file_name().into_string().map_err(|_| RecordError::FileName(batches.clone()))?;
        if name.ends_with(".mjs") {
            names.push(format!("batches/{name}"));
        }
    }
    names.sort();
    let mut files = vec![SetConfig::FILE.to_owned()];
    files.extend(names);
    let entries = files
        .into_iter()
        .map(|name| Ok((name.clone(), hash(&root.join(&name))?.into())))
        .collect::<Result<Vec<_>, RecordError>>()?;
    Ok(js_object(entries))
}

/// `Object.fromEntries(entries)`: a repeated key keeps its first position and takes the last
/// value, and array-index keys (`"0"`, `"1"`, …) come first in ascending order, as in every
/// JS object.
fn js_object(entries: Vec<(String, Value)>) -> Map<String, Value> {
    let mut map = Map::new();
    for (key, value) in entries {
        map.insert(key, value);
    }
    let mut indices: Vec<(u32, String)> =
        map.keys().filter_map(|key| array_index(key).map(|index| (index, key.clone()))).collect();
    if indices.is_empty() {
        return map;
    }
    indices.sort();
    let mut ordered = Map::new();
    for (_, key) in &indices {
        let value = map.shift_remove(key).unwrap_or_default();
        ordered.insert(key.clone(), value);
    }
    ordered.extend(map);
    ordered
}

/// The value of a canonical array-index key: digits without a leading zero, below 2³² − 1.
fn array_index(key: &str) -> Option<u32> {
    let canonical = !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()) && (key == "0" || !key.starts_with('0'));
    canonical.then(|| key.parse::<u32>().ok()).flatten().filter(|&index| index != u32::MAX)
}

/// What post-export found in one native, for its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeFacts<'a> {
    /// `native.layers.length`.
    pub layers: usize,
    /// `Object.keys(native.slots)`.
    pub slots: &'a [String],
    /// sha256 of the bytes of `native/<name>.json`.
    pub sha256: &'a str,
}

/// The record of one animation (`build-set.mjs:188-227` and its variants): `variant` of
/// `icon`, exported as `native`.
pub fn record(
    profile: &Profile,
    provenance: &Provenance,
    icon: &Icon,
    variant: &Variant,
    native: &NativeFacts<'_>,
) -> Result<Value, RecordError> {
    let meta = &variant.meta;
    let get = |key: &str| meta.get(key);
    let rest = variant
        .rest()
        .filter(|rest| rest.is_finite())
        .ok_or_else(|| RecordError::Rest { animation: variant.name.clone() })?;
    let stateful = truthy(get("stateful"));
    let (start, end) = if variant.reverse { ("finalState", "initialState") } else { ("initialState", "finalState") };

    let mut record = Record(Map::new());
    record.put("name", variant.name.as_str().into());
    record.set("icon", get("icon"));
    record.set("motionId", get("motionId"));
    record.set("revision", get("revision"));
    match profile.continuous {
        Continuous::Omit => {}
        Continuous::Icon => record.put("continuous", truthy(icon.meta.get("continuous")).into()),
        Continuous::Always => record.put("continuous", true.into()),
    }
    record.put("previewReset", js_or(get("previewReset"), false.into()));
    record.put("cycleDegrees", js_or(get("cycleDegrees"), 0.into()));
    record.put("directionSources", Value::Object(provenance.direction_sources.clone()));
    record.set("motionLabel", get("motionLabel"));
    record.set("direction", get("direction"));
    record.set("stateful", get("stateful"));
    if stateful {
        record.set("startState", get(start));
        record.set("endState", get(end));
    } else {
        record.put("startState", "rest".into());
        record.put("endState", "rest".into());
    }
    for key in ["heading", "description", "label", "intent"] {
        record.set(key, get(key));
    }
    record.put("width", 32.into());
    record.put("height", 32.into());
    record.put("fps", js_number(FPS));
    record.put("frames", js_number(DURATION * FPS));
    record.put("duration", js_number(DURATION));
    record.set("active", get("active"));
    record.put("rest", js_number(rest * FPS));
    record.set("keyPoses", get("keyPoses"));
    record.put("layers", native.layers.into());
    record.put("slots", native.slots.iter().map(|slot| Value::from(slot.as_str())).collect());
    record.put("nativeSha256", native.sha256.into());
    record.put("sourceSha256", provenance.source_sha256.as_str().into());
    if let Some((key, sha)) = &provenance.builder_sha256 {
        record.put(key, sha.as_str().into());
    }
    if profile.authoring {
        if let Some(sources) = &provenance.authoring_sources {
            record.put("authoringSources", Value::Object(sources.clone()));
        }
        record.set("brief", get("brief"));
        record.put("constraints", js_or(get("constraints"), json!([])));
        record.put("category", profile.name.as_str().into());
    }
    record.put("definitionsSha256", provenance.definitions_sha256.as_str().into());
    record.put("primary", json!({"dark": PRIMARY_DARK, "light": PRIMARY_LIGHT}));
    if profile.duotone_fill {
        let tint = icon.parts.iter().any(|part| part.meta.get("style").and_then(Value::as_str) == Some("tint"));
        record.put("duotoneFill", tint.into());
    }
    record.put("styles", json!(["outline", "solid", "duotone"]));
    record.put(
        "strokeWidth",
        json!({"min": 1.5, "default": 2, "max": 2.5, "presets": [1.5, 2, 2.5], "units": "32-unit construction grid"}),
    );
    Ok(Value::Object(record.0))
}

/// An object literal under construction: `set` skips `undefined` as `JSON.stringify` does.
struct Record(Map<String, Value>);

impl Record {
    fn put(&mut self, key: &str, value: Value) {
        self.0.insert(key.to_owned(), value);
    }

    fn set(&mut self, key: &str, value: Option<&Value>) {
        if let Some(value) = value {
            self.put(key, value.clone());
        }
    }
}

/// `value || fallback`.
fn js_or(value: Option<&Value>, fallback: Value) -> Value {
    match value {
        Some(value) if truthy(Some(value)) => value.clone(),
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_from_entries_orders_like_js() {
        let entries = |keys: &[&str]| keys.iter().enumerate().map(|(i, k)| (k.to_string(), json!(i))).collect();
        let map = js_object(entries(&["b.mjs", "10", "a", "2", "b.mjs", "01", "4294967295"]));
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        assert_eq!(keys, ["2", "10", "b.mjs", "a", "01", "4294967295"]);
        assert_eq!(map["b.mjs"], json!(4));
    }

    #[test]
    fn js_or_keeps_truthy_values() {
        assert_eq!(js_or(Some(&json!(90)), json!(0)), json!(90));
        assert_eq!(js_or(Some(&json!(0)), json!(0)), json!(0));
        assert_eq!(js_or(None, json!([])), json!([]));
        assert_eq!(js_or(Some(&json!([])), json!(false)), json!([]));
        assert_eq!(js_or(Some(&json!("")), json!(false)), json!(false));
    }

    #[test]
    fn identity_names_the_engine() {
        assert!(builder_identity().starts_with(concat!("gai-build ", env!("CARGO_PKG_VERSION"), "; QuickJS-NG")));
        assert_eq!(builder_identity_sha256().len(), 64);
    }
}
