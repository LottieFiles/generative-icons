//! The icon lock (schema 1): installed icons and the sha256 of every managed file this
//! tool wrote, so later runs can tell tool output from user edits.
//!
//! ```text
//! { "schemaVersion": 1, "libraryVersion": "…", "generator": "generative-icons@…",
//!   "icons": { "<id>": { "version", "origin"?, "sha256", "source", "asset",
//!                        "variant"?: { "weight" }, "build"?: { … } } },
//!   "files": { "<managed path>": "<sha256>" } }
//! ```
//!
//! A local icon has `version: "local"`, `origin: "local"` and a `build` record:
//! `definitionsSha256` (from its `set.json`), `sourcesSha256` (sha256 of the build's
//! [`SOURCES_FILE`](crate::local_set::SOURCES_FILE), which lists every source module it
//! was built from), `kitSha256`, `engine` (`name@version` of the exporting engine) and
//! `actions` (`name` → sha256 of each action's asset JSON).
//! `kitSha256` is [`crate::kit::sha256`]: sha256 over one `"<name>\0<sha256>\n"` line per
//! kit file, in bytewise name order. Nothing time-dependent is recorded.
//!
//! An existing lock is edited in place, as the JS CLI did with `structuredClone`: key
//! order and unknown keys survive. New keys are appended, except `libraryVersion` and
//! `generator`, which keep their documented place after `schemaVersion`.

use crate::config::LOCK_FILE;
use crate::error::{CliError, ErrorCode, Result};
use crate::managed::{inside, read_optional};
use gai_core::sha256_hex;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::Path;

/// The `generator` value this binary records.
pub fn generator() -> String {
    format!("generative-icons@{}", env!("CARGO_PKG_VERSION"))
}

/// A parsed lock and the digest of its bytes on disk.
#[derive(Debug, Clone, PartialEq)]
pub struct Lock {
    /// Project-relative lock path.
    pub path: &'static str,
    /// The lock object.
    pub value: Map<String, Value>,
    /// sha256 of the lock file as read; `None` when it did not exist.
    pub disk_sha256: Option<String>,
}

impl Lock {
    /// Reads the project's lock, or starts an empty one.
    pub fn load(root: &Path) -> Result<Lock> {
        let path = LOCK_FILE;
        let file = inside(root, path)?;
        let invalid = |message: String| CliError::new(ErrorCode::InvalidConfig, message).with_path(path);
        let Some(bytes) = read_optional(&file)? else {
            let mut value = Map::new();
            value.insert("schemaVersion".into(), json!(1));
            value.insert("generator".into(), json!(generator()));
            value.insert("icons".into(), json!({}));
            value.insert("files".into(), json!({}));
            return Ok(Lock { path, value, disk_sha256: None });
        };
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|error| invalid(format!("Cannot parse {path}: {error}")))?;
        let Value::Object(value) = value else { return Err(invalid("Unsupported icon lock".into())) };
        if value.get("schemaVersion").and_then(Value::as_f64) != Some(1.0) {
            return Err(invalid("Unsupported icon lock".into()));
        }
        for key in ["icons", "files"] {
            if value.get(key).is_some_and(|v| !v.is_object()) {
                return Err(invalid(format!("The lock's {key} must be an object")));
            }
        }
        Ok(Lock { path, value, disk_sha256: Some(sha256_hex(&bytes)) })
    }

    /// `files`: managed path → sha256 last written by this tool.
    pub fn files(&self) -> impl Iterator<Item = (&str, &str)> {
        self.object("files").into_iter().flatten().filter_map(|(path, sha)| Some((path.as_str(), sha.as_str()?)))
    }

    /// `icons`: id → install record.
    pub fn icons(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.object("icons").into_iter().flatten().map(|(id, record)| (id.as_str(), record))
    }

    /// The install record of one icon.
    pub fn icon(&self, id: &str) -> Option<&Value> {
        self.object("icons").and_then(|icons| icons.get(id))
    }

    fn object(&self, key: &str) -> Option<&Map<String, Value>> {
        self.value.get(key).and_then(Value::as_object)
    }

    /// Mutable `icons` or `files`, created at the end when absent.
    pub fn object_mut(&mut self, key: &str) -> &mut Map<String, Value> {
        let entry = self.value.entry(key).or_insert_with(|| json!({}));
        if !entry.is_object() {
            *entry = json!({});
        }
        entry.as_object_mut().expect("just made an object")
    }

    /// Sets a top-level key in place. A new key goes right after the last present key
    /// of `after`, or at the end, so the documented order holds.
    pub fn set(&mut self, key: &str, value: Value, after: &[&str]) {
        if self.value.contains_key(key) {
            self.value.insert(key.to_string(), value);
            return;
        }
        let position = self.value.keys().rposition(|existing| after.contains(&existing.as_str()));
        match position {
            Some(index) => {
                self.value.shift_insert(index + 1, key.to_string(), value);
            }
            None => {
                self.value.insert(key.to_string(), value);
            }
        }
    }

    /// Records the library version and the generator.
    pub fn stamp(&mut self, library_version: Option<&str>) {
        if let Some(version) = library_version {
            self.set("libraryVersion", json!(version), &["schemaVersion"]);
        }
        self.set("generator", json!(generator()), &["schemaVersion", "libraryVersion"]);
    }

    /// Digests a managed write may replace: `files` plus the lock itself.
    pub fn expected(&self) -> HashMap<String, String> {
        let mut expected: HashMap<String, String> =
            self.files().map(|(path, sha)| (path.to_string(), sha.to_string())).collect();
        if let Some(sha) = &self.disk_sha256 {
            expected.insert(self.path.to_string(), sha.clone());
        }
        expected
    }

    /// `JSON.stringify(lock, null, 2) + "\n"`.
    pub fn to_bytes(&self) -> Vec<u8> {
        crate::js::json_file(&Value::Object(self.value.clone()))
    }

    /// Status of every recorded file, in lock order: `unchanged`, `modified` or `missing`.
    pub fn status(&self, root: &Path) -> Result<Vec<FileStatus>> {
        self.files()
            .map(|(path, sha)| {
                let status = match read_optional(&inside(root, path)?)? {
                    None => "missing",
                    Some(bytes) if sha256_hex(&bytes) == sha => "unchanged",
                    Some(_) => "modified",
                };
                Ok(FileStatus { file: path.to_string(), status })
            })
            .collect()
    }
}

/// One managed file's state relative to the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatus {
    /// Project-relative path.
    pub file: String,
    /// `unchanged`, `modified` or `missing`.
    pub status: &'static str,
}

impl FileStatus {
    /// `{file, status}`.
    pub fn to_json(&self) -> Value {
        json!({ "file": self.file, "status": self.status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_locks_have_the_documented_shape() {
        let dir = tempfile::tempdir().unwrap();
        let mut lock = Lock::load(dir.path()).unwrap();
        let keys: Vec<&String> = lock.value.keys().collect();
        assert_eq!(keys, ["schemaVersion", "generator", "icons", "files"]);
        lock.stamp(Some("0.1.0"));
        let keys: Vec<&String> = lock.value.keys().collect();
        assert_eq!(keys, ["schemaVersion", "libraryVersion", "generator", "icons", "files"]);
        // A lock written without `generator` gains it after `libraryVersion`.
        lock.value.shift_remove("generator");
        lock.stamp(None);
        let keys: Vec<&String> = lock.value.keys().collect();
        assert_eq!(keys, ["schemaVersion", "libraryVersion", "generator", "icons", "files"]);
    }

    #[test]
    fn status_reports_each_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".generative-icons")).unwrap();
        std::fs::write(dir.path().join("same"), "a").unwrap();
        std::fs::write(dir.path().join("edited"), "b").unwrap();
        let lock = json!({"schemaVersion": 1, "icons": {}, "files": {
            "same": sha256_hex("a"), "edited": sha256_hex("x"), "gone": sha256_hex("c") }});
        std::fs::write(dir.path().join(".generative-icons/lock.json"), lock.to_string()).unwrap();
        let lock = Lock::load(dir.path()).unwrap();
        let status: Vec<(String, &str)> =
            lock.status(dir.path()).unwrap().into_iter().map(|s| (s.file, s.status)).collect();
        assert_eq!(status, [("same".into(), "unchanged"), ("edited".into(), "modified"), ("gone".into(), "missing")]);
        assert_eq!(lock.expected().len(), 4);
    }

    #[test]
    fn unsupported_locks_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".generative-icons")).unwrap();
        std::fs::write(dir.path().join(".generative-icons/lock.json"), r#"{"schemaVersion":2}"#).unwrap();
        let error = Lock::load(dir.path()).unwrap_err();
        assert_eq!((error.code, error.message.as_str()), (ErrorCode::InvalidConfig, "Unsupported icon lock"));
    }
}
