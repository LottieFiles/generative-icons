//! The search sidecar (`search-index.json` beside `catalog.json`): per-icon data the npm
//! descriptors lack but ranked search and context packets read.
//!
//! ```json
//! { "<id>": { "brief": "<String(icon.brief)>", "duotoneFill": …,
//!             "reference": { "intent": …, "brief": …, "profiles": [ … ] } } }
//! ```
//!
//! `brief` is what library-mode search joins (an object brief reads "[object Object]",
//! kept for parity). `reference` holds the fields a context packet takes from set.json
//! for the icon's Strong reference (`strong.intent || icon.intent`, `strong.brief ||
//! icon.brief` and the per-motion profiles with their package hashes), so packets made
//! from a package equal packets made from a checkout except for `inputs`.
//!
//! `library release` writes the sidecar from a checkout; a checkout read directly
//! derives the same entries from its set manifests.

use crate::error::{CliError, ErrorCode, Result};
use crate::js::{to_js_string, truthy};
use crate::managed::inside;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::Path;

/// Sidecar entries by icon id.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchIndex {
    entries: HashMap<String, Map<String, Value>>,
}

impl SearchIndex {
    /// Parses `search-index.json`.
    pub fn parse(bytes: &[u8], location: &str) -> Result<SearchIndex> {
        let value: Value = serde_json::from_slice(bytes).map_err(|error| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Invalid search index at {location}: {error}"))
        })?;
        Ok(Self::from_value(value))
    }

    fn from_value(value: Value) -> SearchIndex {
        let entries = match value {
            Value::Object(map) => map
                .into_iter()
                .filter_map(|(id, entry)| match entry {
                    Value::Object(entry) => Some((id, entry)),
                    _ => None,
                })
                .collect(),
            _ => HashMap::new(),
        };
        SearchIndex { entries }
    }

    /// Derives the sidecar of a library checkout from its registered set manifests.
    pub fn from_checkout(repo: &Path) -> Result<SearchIndex> {
        Ok(Self::from_value(build(repo)?))
    }

    /// The brief of one icon.
    pub fn brief(&self, id: &str) -> Option<&str> {
        self.entries.get(id)?.get("brief")?.as_str()
    }

    /// The whole entry of one icon.
    pub fn entry(&self, id: &str) -> Option<&Map<String, Value>> {
        self.entries.get(id)
    }
}

/// The sidecar of the checkout at `repo`, in catalog order.
pub fn build(repo: &Path) -> Result<Value> {
    let icons_dir = repo.join("prototype/icons");
    let read = |relative: &str| -> Result<Value> {
        let file = inside(&icons_dir, relative)?;
        let bytes = std::fs::read(&file).map_err(|error| CliError::io("read", &file, &error))?;
        serde_json::from_slice(&bytes).map_err(|error| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Cannot parse prototype/icons/{relative}: {error}"))
        })
    };
    let catalog = read("catalog.json")?;
    let sets = catalog.get("sets").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut index = Map::new();
    for relative in sets.iter().filter_map(Value::as_str) {
        let set = read(relative)?;
        for icon in set.get("icons").and_then(Value::as_array).into_iter().flatten() {
            // `icon.icon || icon.name`, as generation-context.mjs picks the id.
            let id = [icon.get("icon"), icon.get("name")]
                .into_iter()
                .flatten()
                .find(|value| truthy(value))
                .and_then(Value::as_str);
            if let Some(id) = id {
                index.insert(id.to_string(), Value::Object(entry(icon)));
            }
        }
    }
    Ok(Value::Object(index))
}

/// One icon's sidecar entry.
fn entry(icon: &Value) -> Map<String, Value> {
    let mut entry = Map::new();
    if let Some(brief) = icon.get("brief").filter(|brief| !brief.is_null()) {
        entry.insert("brief".into(), json!(to_js_string(brief)));
    }
    if let Some(fill) = icon.get("duotoneFill").filter(|fill| !fill.is_null()) {
        entry.insert("duotoneFill".into(), fill.clone());
    }
    if let Some(reference) = reference(icon) {
        entry.insert("reference".into(), Value::Object(reference));
    }
    entry
}

/// `intent`, `brief` and `profiles` of the packet reference (`generation-context.mjs:79-96`);
/// `None` without a Strong motion.
fn reference(icon: &Value) -> Option<Map<String, Value>> {
    let motions: Vec<&Value> = match icon.get("motions").filter(|m| truthy(m)) {
        Some(Value::Array(motions)) => motions.iter().collect(),
        _ => vec![icon],
    };
    let strong = motions.iter().find(|m| m.get("motionId").and_then(Value::as_str) == Some("strong"))?;
    let mut reference = Map::new();
    for key in ["intent", "brief"] {
        if let Some(value) = js_or(strong.get(key), icon.get(key)) {
            reference.insert(key.into(), value.clone());
        }
    }
    let profiles = motions.iter().map(|motion| Value::Object(profile(motion))).collect();
    reference.insert("profiles".into(), Value::Array(profiles));
    Some(reference)
}

/// One motion's packet profile; `undefined` members are omitted, as `JSON.stringify` does.
pub fn profile(motion: &Value) -> Map<String, Value> {
    let mut profile = Map::new();
    let mut put = |key: &str, value: Option<&Value>| {
        if let Some(value) = value {
            profile.insert(key.into(), value.clone());
        }
    };
    put("profile", motion.get("motionId"));
    put("activeSeconds", motion.get("active"));
    put("restFrame", motion.get("rest"));
    put("continuous", Some(&json!(motion.get("continuous").is_some_and(truthy))));
    put("startState", motion.get("startState"));
    put("endState", motion.get("endState"));
    let reverse = motion.get("reverse").filter(|r| !r.is_null()).and_then(|r| r.get("name"));
    put("reverse", reverse);
    put("packageSha256", motion.get("sha256"));
    profile
}

/// JS `a || b` over possibly `undefined` values.
fn js_or<'a>(a: Option<&'a Value>, b: Option<&'a Value>) -> Option<&'a Value> {
    if a.is_some_and(truthy) { a } else { b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_briefs() {
        let index = SearchIndex::parse(br#"{"heart":{"brief":"Beat."},"bell":{"duotoneFill":true}}"#, "x").unwrap();
        assert_eq!(index.brief("heart"), Some("Beat."));
        assert_eq!(index.brief("bell"), None);
        assert_eq!(index.entry("bell").unwrap()["duotoneFill"], true);
    }

    #[test]
    fn checkout_entries_follow_generation_context() {
        let dir = tempfile::tempdir().unwrap();
        let icons = dir.path().join("prototype/icons");
        std::fs::create_dir_all(icons.join("01/assets")).unwrap();
        std::fs::write(icons.join("catalog.json"), r#"{"sets":["01/assets/set.json"]}"#).unwrap();
        std::fs::write(
            icons.join("01/assets/set.json"),
            r#"{"icons":[
              {"icon":"","name":"bell","brief":"Rings.","intent":"Ring.","motions":[
                {"motionId":"light","name":"bell","active":0.5,"rest":33,"sha256":"a"},
                {"motionId":"strong","name":"bell-strong","active":0.4,"rest":27,"intent":"Ring hard.","reverse":{"name":"bell-back"},"sha256":"b"}]},
              {"icon":"heart","name":"heart-x"},
              {"icon":"chat","brief":{"meaning":"x"}}]}"#,
        )
        .unwrap();
        let index = SearchIndex::from_checkout(dir.path()).unwrap();
        assert_eq!(index.brief("bell"), Some("Rings."));
        assert_eq!(index.brief("heart"), None);
        assert_eq!(index.brief("chat"), Some("[object Object]"));
        let reference = &index.entry("bell").unwrap()["reference"];
        assert_eq!(
            reference,
            &json!({"intent": "Ring hard.", "brief": "Rings.", "profiles": [
                {"profile": "light", "activeSeconds": 0.5, "restFrame": 33, "continuous": false, "packageSha256": "a"},
                {"profile": "strong", "activeSeconds": 0.4, "restFrame": 27, "continuous": false, "reverse": "bell-back", "packageSha256": "b"}]})
        );
        assert!(index.entry("heart").unwrap().get("reference").is_none());
    }
}
