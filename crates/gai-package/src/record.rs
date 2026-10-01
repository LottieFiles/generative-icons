//! `set.json` finalization (`package-set.py:20-57`, builder spec §6.10): build every
//! `.lottie`, then record each archive's `bytes` and `sha256` on the animation, on the
//! matching motion and its `reverse`, and on the icon (from its first motion).
//!
//! Assignments follow Python dict semantics: an existing key keeps its position and a new
//! key is appended, `bytes` before `sha256`. The icon-level `reverse` is never touched.

use crate::dotlottie::{Archive, build_icon_archives};
use crate::error::{Result, invalid};
use crate::io::{array_field, field, object, py_pretty, str_field};
use crate::zip::ZipDate;
use gai_core::parallel::parallel_map;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;

/// Which icon list names a set's actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionList {
    /// `icon.animations`: every set except the frozen prototypes.
    Animations,
    /// `icon.motions`: the frozen sets 01 and 02, which have no `animations`.
    Motions,
}

impl ActionList {
    fn key(self) -> &'static str {
        match self {
            ActionList::Animations => "animations",
            ActionList::Motions => "motions",
        }
    }
}

/// Builds every `.lottie` of the set `record` from the asset folder `assets` and writes the
/// resulting `bytes` and `sha256` into `record`. Returns the archives in record order.
pub fn build_set_archives(
    record: &mut Value,
    assets: &Path,
    list: ActionList,
    date: ZipDate,
) -> Result<Vec<Archive>> {
    let icons = array_field(record, "icons", "set.json")?;
    let names: Vec<Vec<String>> = icons
        .iter()
        .map(|icon| action_names(icon, list))
        .collect::<Result<_>>()?;
    let built = parallel_map(names, |actions| build_icon_archives(assets, &actions, date));
    let per_icon: Vec<Vec<Archive>> = built.into_iter().collect::<Result<_>>()?;

    let icons = field_mut(record, "icons")?
        .as_array_mut()
        .ok_or_else(|| invalid("set.json: \"icons\" is not an array"))?;
    for (icon, archives) in icons.iter_mut().zip(&per_icon) {
        let name = icon
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string();
        let context = format!("icon {name}");
        let items = field_mut(icon, list.key())?
            .as_array_mut()
            .ok_or_else(|| invalid(format!("{context}: actions are not an array")))?;
        for (item, archive) in items.iter_mut().zip(archives) {
            set_hash(
                object(item, &context)?,
                archive.lottie.len() as u64,
                &archive.sha256,
            );
        }
        if list == ActionList::Animations {
            let lookup: HashMap<&str, &Archive> =
                archives.iter().map(|a| (a.name.as_str(), a)).collect();
            let hashes = |name: &str| {
                lookup
                    .get(name)
                    .map(|a| (a.lottie.len() as u64, a.sha256.clone()))
                    .ok_or_else(|| invalid(format!("{context}: no animation named {name}")))
            };
            let motions = field_mut(icon, "motions")?
                .as_array_mut()
                .ok_or_else(|| invalid(format!("{context}: motions are not an array")))?;
            for motion in motions.iter_mut() {
                let (bytes, sha256) = hashes(str_field(motion, "name", &context)?)?;
                set_hash(object(motion, &context)?, bytes, &sha256);
                if let Some(reverse) = motion.get_mut("reverse") {
                    let (bytes, sha256) = hashes(str_field(reverse, "name", &context)?)?;
                    set_hash(object(reverse, &context)?, bytes, &sha256);
                }
            }
        }
        let first = array_field(icon, "motions", &context)?
            .first()
            .ok_or_else(|| invalid(format!("{context}: no motions")))?;
        let (bytes, sha256) = (
            field(first, "bytes", &context)?.clone(),
            field(first, "sha256", &context)?.clone(),
        );
        let icon = object(icon, &context)?;
        icon.insert("bytes".into(), bytes);
        icon.insert("sha256".into(), sha256);
    }
    Ok(per_icon.into_iter().flatten().collect())
}

/// `json.dumps(record, indent=2) + "\n"`: the bytes of `set.json` and `evidence/build.json`.
pub fn encode_set_json(record: &Value) -> String {
    py_pretty(record) + "\n"
}

/// Every action record of every icon (`animations`, or `motions` for 01/02), in record order.
pub fn record_actions(record: &Value, list: ActionList) -> Result<Vec<&Value>> {
    let mut all = Vec::new();
    for icon in array_field(record, "icons", "set.json")? {
        all.extend(array_field(icon, list.key(), "icon")?);
    }
    Ok(all)
}

fn action_names(icon: &Value, list: ActionList) -> Result<Vec<String>> {
    let context = format!(
        "icon {}",
        icon.get("name").and_then(Value::as_str).unwrap_or("?")
    );
    array_field(icon, list.key(), &context)?
        .iter()
        .map(|item| str_field(item, "name", &context).map(str::to_string))
        .collect()
}

fn field_mut<'a>(value: &'a mut Value, key: &str) -> Result<&'a mut Value> {
    value
        .get_mut(key)
        .ok_or_else(|| invalid(format!("missing \"{key}\"")))
}

fn set_hash(map: &mut Map<String, Value>, bytes: u64, sha256: &str) {
    map.insert("bytes".into(), bytes.into());
    map.insert("sha256".into(), sha256.into());
}
