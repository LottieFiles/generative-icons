//! `library adopt SET BUILD…`: replaces library icons with spec v2 builds. A build is the
//! folder `gen-animated-icons build <id>` writes in a project (`.gen-animated-icons/build/<id>`):
//! the spec, every action's asset, manifest and themes, and a `set.json`. Adopting copies
//! the spec to the set's `specs/<id>.icon.json` and the actions into the set's assets,
//! replaces the icon's `set.json` entry (keeping its number, category and labels), and
//! packages the set. From then on `library build` keeps the icon as adopted, and
//! `library verify` checks it against its spec. Frozen sets (01, 02) are never rebuilt,
//! so adopting there only replaces the icon's assets and entry.

use super::{resolve_set, sets_dir};
use crate::cli::LibraryAdoptArgs;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use gai_check::verify::adopted::{SOURCE, SPECS_DIR};
use gai_package::{SetPaths, encode_set_json, package_set};
use serde_json::{Map, Value, json};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The files a build holds for every action.
const ACTION_FILES: [&str; 4] = [".json", ".manifest.json", ".theme-dark.json", ".theme-light.json"];

/// One build, read and checked against the set before anything is written.
struct Adoption {
    id: String,
    dir: PathBuf,
    spec: Vec<u8>,
    entry: Value,
    actions: Vec<String>,
}

pub fn run(repo: &Path, args: &LibraryAdoptArgs) -> Result<Report> {
    let set = resolve_set(repo, &args.set)?;
    let set_dir = sets_dir(repo).join(&set);
    let paths = SetPaths::new(repo, &set).map_err(|e| CliError::new(ErrorCode::InvalidCatalog, e.to_string()))?;
    let set_json_path = paths.assets.join("set.json");
    let mut record: Value = read_json(&set_json_path)?;

    let mut adoptions = Vec::new();
    for dir in &args.builds {
        let adoption = read_build(dir, &record, &set)?;
        if adoptions.iter().any(|a: &Adoption| a.id == adoption.id) {
            return Err(CliError::usage(format!("{} is named twice", adoption.id)));
        }
        adoptions.push(adoption);
    }

    // Everything is checked; write.
    let specs = set_dir.join(SPECS_DIR);
    std::fs::create_dir_all(&specs).map_err(|e| CliError::io("create", &specs, &e))?;
    let mut text = String::new();
    let mut adopted = Vec::new();
    for a in &adoptions {
        let spec_path = specs.join(format!("{}.icon.json", a.id));
        std::fs::write(&spec_path, &a.spec).map_err(|e| CliError::io("write", &spec_path, &e))?;
        for name in &a.actions {
            for suffix in ACTION_FILES {
                let from = a.dir.join(format!("{name}{suffix}"));
                let to = paths.assets.join(format!("{name}{suffix}"));
                std::fs::copy(&from, &to).map_err(|e| CliError::io("copy", &from, &e))?;
            }
        }
        let icons = record["icons"].as_array_mut().expect("checked in read_build");
        let slot = icons.iter_mut().find(|i| i["icon"] == json!(a.id)).expect("checked in read_build");
        *slot = a.entry.clone();
        let _ = writeln!(text, "adopted {} into {set}: {}", a.id, a.actions.join(", "));
        adopted.push(json!({"icon": a.id, "spec": format!("{SPECS_DIR}/{}.icon.json", a.id), "actions": a.actions}));
    }
    std::fs::write(&set_json_path, encode_set_json(&record)).map_err(|e| CliError::io("write", &set_json_path, &e))?;
    let package = package_set(&paths).map_err(|e| CliError::new(ErrorCode::InvalidCatalog, format!("{set}: {e}")))?;
    for archive in &package.archives {
        let _ = writeln!(text, "{archive}");
    }
    let _ = writeln!(text, "Next: gen-animated-icons library verify {set} && gen-animated-icons library verify-visual {set}");
    Ok(Report::new(json!({"set": set, "adopted": adopted, "archives": package.archives.len()}), text))
}

/// Reads a build folder and checks it against the set: one spec, an icon of the set, the
/// same actions as the icon it replaces, and every action's files.
fn read_build(dir: &Path, record: &Value, set: &str) -> Result<Adoption> {
    let entries = std::fs::read_dir(dir).map_err(|e| CliError::io("read", dir, &e))?;
    let mut specs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".icon.json")))
        .collect();
    let spec_path = match specs.len() {
        1 => specs.remove(0),
        n => {
            return Err(CliError::usage(format!(
                "{} holds {n} specs; name a build folder (.gen-animated-icons/build/<id>) that holds one",
                dir.display()
            )));
        }
    };
    let id = spec_path.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".icon.json")).unwrap_or_default().to_string();
    let spec = std::fs::read(&spec_path).map_err(|e| CliError::io("read", &spec_path, &e))?;
    let parsed: Value = serde_json::from_slice(&spec).map_err(|e| CliError::new(ErrorCode::InvalidConfig, format!("{}: {e}", spec_path.display())))?;
    if parsed.get("format").and_then(Value::as_str) != Some(SOURCE) || parsed.get("id").and_then(Value::as_str) != Some(id.as_str()) {
        return Err(CliError::new(ErrorCode::InvalidConfig, format!("{}: not a {SOURCE} spec with id {id}", spec_path.display())));
    }

    let built: Value = read_json(&dir.join("set.json"))?;
    let new = built
        .get("icons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|i| i.get("icon") == Some(&json!(id)))
        .ok_or_else(|| CliError::new(ErrorCode::InvalidCatalog, format!("{}: set.json has no entry for {id}", dir.display())))?;
    let old = record
        .get("icons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|i| i.get("icon") == Some(&json!(id)))
        .ok_or_else(|| CliError::new(ErrorCode::UnknownIcon, format!("{set} has no icon {id}")).with_hint("A spec replaces a library icon of the same id"))?;

    // Frozen sets (01, 02) list their actions under `motions` only.
    let names = |entry: &Value| -> Vec<String> {
        let mut v: Vec<String> = entry
            .get("animations")
            .or_else(|| entry.get("motions"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|a| a.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        v.sort();
        v
    };
    let (want, have) = (names(old), names(new));
    if want != have {
        return Err(CliError::new(
            ErrorCode::InvalidCatalog,
            format!("{id}: the build's actions {have:?} are not the library's {want:?}"),
        )
        .with_hint("Keep the icon's state: a stateful icon needs the same reverse (or reset) actions"));
    }
    for name in &have {
        for suffix in ACTION_FILES {
            let file = dir.join(format!("{name}{suffix}"));
            if !file.is_file() {
                return Err(CliError::new(ErrorCode::InvalidPath, format!("{} is missing; rebuild {id}", file.display())));
            }
        }
    }
    let mut entry = adopt_entry(old, new);
    // Frozen sets list actions under `motions` only, and packaging hashes only those.
    if old.get("animations").is_none()
        && let Some(map) = entry.as_object_mut()
    {
        map.remove("animations");
    }
    let actions = new
        .get("animations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| a.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    Ok(Adoption { id, dir: dir.to_path_buf(), spec, entry, actions })
}

/// The library entry for a build's entry: the build's records, with the library's
/// `stateful` flag and category, and the icon's number and labels kept.
fn adopt_entry(old: &Value, new: &Value) -> Value {
    let stateful = new.get("motions").and_then(Value::as_array).into_iter().flatten().any(|m| m.get("reverse").is_some());
    let category = old.get("category").cloned();
    let mut entry = adopt_record(new, stateful, category.as_ref());
    let map = entry.as_object_mut().expect("an object");
    for key in ["motions", "animations"] {
        if let Some(Value::Array(list)) = new.get(key) {
            let records = list
                .iter()
                .map(|r| {
                    let mut record = adopt_record(r, stateful, category.as_ref());
                    if let Some(reverse) = r.get("reverse") {
                        record["reverse"] = adopt_record(reverse, stateful, category.as_ref());
                    }
                    record
                })
                .collect();
            map.insert(key.into(), Value::Array(records));
        }
    }
    // The icon's place in its set and its state labels stay the library's.
    let mut out = Map::new();
    for (key, value) in std::mem::take(map) {
        if key == "motions" {
            for kept in ["number", "forwardLabel", "reverseLabel"] {
                if let Some(v) = old.get(kept).or_else(|| new.get(kept)) {
                    out.insert(kept.into(), v.clone());
                }
            }
        }
        if matches!(key.as_str(), "number" | "forwardLabel" | "reverseLabel") {
            continue;
        }
        out.insert(key, value);
    }
    Value::Object(out)
}

/// One record: `stateful` after `direction` and the library category; no stale package
/// fields (packaging writes `bytes` and `sha256`).
fn adopt_record(record: &Value, stateful: bool, category: Option<&Value>) -> Value {
    let mut out = Map::new();
    for (key, value) in record.as_object().into_iter().flatten() {
        match key.as_str() {
            "bytes" | "sha256" | "stateful" | "motions" | "animations" | "reverse" => continue,
            // The library's category, or none where the library records none: the build's
            // own is its project's local set.
            "category" => {
                if let Some(category) = category {
                    out.insert(key.clone(), category.clone());
                }
            }
            _ => {
                out.insert(key.clone(), value.clone());
            }
        }
        if key == "direction" {
            out.insert("stateful".into(), json!(stateful));
        }
    }
    Value::Object(out)
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path).map_err(|e| CliError::io("read", path, &e))?;
    serde_json::from_slice(&bytes).map_err(|e| CliError::new(ErrorCode::InvalidCatalog, format!("{}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_keep_the_library_number_category_and_labels() {
        let old = json!({"icon": "lock", "category": "Account", "number": "7", "forwardLabel": "Lock", "reverseLabel": "Unlock", "stateful": true});
        let rec = |name: &str, dir: &str| json!({"name": name, "icon": "lock", "direction": dir, "source": SOURCE, "category": "Local", "bytes": 1, "sha256": "x"});
        let new = json!({
            "name": "lock", "icon": "lock", "direction": "forward", "source": SOURCE, "category": "Local",
            "forwardLabel": "Close", "reverseLabel": "Open",
            "motions": [ {"name": "lock", "icon": "lock", "direction": "forward", "reverse": rec("lock-reverse", "reverse")} ],
            "animations": [rec("lock", "forward"), rec("lock-reverse", "reverse")],
            "bytes": 9, "sha256": "y"
        });
        let entry = adopt_entry(&old, &new);
        let keys: Vec<&str> = entry.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["name", "icon", "direction", "stateful", "source", "category", "number", "forwardLabel", "reverseLabel", "motions", "animations"]);
        assert_eq!(entry["category"], "Account");
        assert_eq!(entry["forwardLabel"], "Lock");
        assert_eq!(entry["stateful"], true);
        assert_eq!(entry["animations"][1]["stateful"], true);
        assert_eq!(entry["animations"][1]["category"], "Account");
        assert!(entry["animations"][0].get("sha256").is_none());
        assert_eq!(entry["motions"][0]["reverse"]["name"], "lock-reverse");
        assert_eq!(entry["motions"][0]["reverse"]["category"], "Account");
    }
}
