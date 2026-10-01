//! `set.json` assembly (builder spec §6.9): the set header, then one entry per icon made of
//! its first forward record, its motions (forward records, each with its `reverse`) and all
//! of its records.
//!
//! The builder writes this as `JSON.stringify(set, null, 2) + "\n"`. Packaging
//! (`gai_package`) then adds each `.lottie`'s `bytes` and `sha256` and rewrites it in Python
//! `json.dumps` format; that final form is what the library commits.

use gai_core::js::truthy;
use crate::profile::Profile;
use gai_core::to_js_json_pretty;
use gai_js::Icon;
use serde_json::{Map, Value};

/// The player version every set records.
pub const RUNTIME: &str = "@lottiefiles/dotlottie-web@0.80.0";
/// The style a set shows first.
pub const DEFAULT_STYLE: &str = "outline";

/// The `set.json` entry of `icon`, from its records in builder order
/// (`build-set.mjs:231-232`).
pub fn icon_entry(icon: &Icon, records: &[Value]) -> Value {
    let stateful = truthy(icon.meta.get("stateful"));
    let is = |record: &Value, key: &str, value: &str| record.get(key).and_then(Value::as_str) == Some(value);
    let motions: Vec<Value> = records
        .iter()
        .filter(|record| is(record, "direction", "forward"))
        .map(|forward| {
            let mut motion = forward.clone();
            if stateful {
                let reverse = records.iter().find(|record| {
                    record.get("motionId") == forward.get("motionId") && is(record, "direction", "reverse")
                });
                assign(&mut motion, "reverse", reverse.cloned());
            }
            motion
        })
        .collect();
    let mut entry = motions.first().cloned().unwrap_or_else(|| Value::Object(Map::new()));
    for key in ["number", "forwardLabel", "reverseLabel"] {
        assign(&mut entry, key, icon.meta.get(key).cloned());
    }
    assign(&mut entry, "motions", Some(Value::Array(motions)));
    assign(&mut entry, "animations", Some(Value::Array(records.to_vec())));
    entry
}

/// `object[key] = value` in an object literal: `None` (`undefined`) leaves the key out of
/// the JSON, even when an earlier spread supplied it.
fn assign(object: &mut Value, key: &str, value: Option<Value>) {
    if let Some(map) = object.as_object_mut() {
        match value {
            Some(value) => {
                map.insert(key.to_owned(), value);
            }
            None => {
                map.shift_remove(key);
            }
        }
    }
}

/// The whole `set.json` value: the profile's header, then `icons`.
pub fn assemble(profile: &Profile, icons: Vec<Value>) -> Value {
    let mut set = Map::new();
    set.insert("name".into(), profile.name.as_str().into());
    set.insert("id".into(), profile.id.as_str().into());
    if let Some(category) = &profile.category {
        set.insert("category".into(), category.as_str().into());
    }
    set.insert("version".into(), profile.version.as_str().into());
    set.insert("defaultStyle".into(), DEFAULT_STYLE.into());
    set.insert("defaultMotion".into(), profile.default_motion.as_str().into());
    set.insert("runtime".into(), RUNTIME.into());
    set.insert("icons".into(), Value::Array(icons));
    Value::Object(set)
}

/// The builder's `set.json` text: `JSON.stringify(set, null, 2) + "\n"`.
pub fn encode(set: &Value) -> String {
    to_js_json_pretty(set, 2) + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use gai_js::Icon;
    use serde_json::json;

    fn icon(meta: Value) -> Icon {
        let name = meta["name"].as_str().unwrap().to_owned();
        Icon { name, meta, source_parts: Vec::new(), parts: Vec::new(), variants: Vec::new() }
    }

    fn record(name: &str, motion: &str, direction: &str) -> Value {
        json!({"name": name, "motionId": motion, "direction": direction, "strokeWidth": 2})
    }

    #[test]
    fn stateful_icons_link_their_reverse() {
        let records = [
            record("a", "light", "forward"),
            record("a-reset", "light", "reverse"),
            record("a-strong", "strong", "forward"),
            record("a-reset-strong", "strong", "reverse"),
        ];
        let meta = json!({"name": "a", "stateful": true, "number": "04", "reverseLabel": "Reset"});
        let entry = icon_entry(&icon(meta), &records);
        let keys: Vec<&str> = entry.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "name",
                "motionId",
                "direction",
                "strokeWidth",
                "reverse",
                "number",
                "reverseLabel",
                "motions",
                "animations"
            ]
        );
        assert_eq!(entry["motions"][1]["reverse"]["name"], "a-reset-strong");
        assert_eq!(entry["animations"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn stateless_icons_have_no_reverse() {
        let records = [record("b", "light", "forward"), record("b-strong", "strong", "forward")];
        let entry = icon_entry(&icon(json!({"name": "b", "number": 7})), &records);
        assert!(entry.get("reverse").is_none());
        assert_eq!(entry["number"], 7);
        assert_eq!(entry["motions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn header_order_and_encoding() {
        let profile = crate::profile::Profile::project(&crate::profile::SetConfig {
            id: "70-test".into(),
            name: "Test".into(),
            shared_sources: None,
            builder: Default::default(),
        });
        let set = assemble(&profile, Vec::new());
        assert_eq!(
            encode(&set),
            "{\n  \"name\": \"Test\",\n  \"id\": \"70-test\",\n  \"category\": \"Test\",\n  \"version\": \"1.0\",\n  \"defaultStyle\": \"outline\",\n  \"defaultMotion\": \"strong\",\n  \"runtime\": \"@lottiefiles/dotlottie-web@0.80.0\",\n  \"icons\": []\n}\n"
        );
    }
}
