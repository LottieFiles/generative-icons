//! Validation of evidence written by the legacy verify.html variants (sets 01–34).
//!
//! Those pages record older schemas (no loop-seam metrics before set 16, no
//! `styleProbeFrame` before 21, set 12's loader record, …). Re-verifying them
//! with the canonical checks plus [`SetRules`] produces the canonical schema.
//! This module checks that the canonical run reproduces every value the old
//! evidence recorded, on the keys both records share.

use crate::catalog::SetManifest;
use crate::evidence::Report;
use crate::probe::require_browser_parity;
use crate::rules::SetRules;
use crate::verify::{ActionOptions, Outcome, verify_action};
use crate::RenderError;
use gai_core::to_py_json;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Keys that depend on the browser's clock, never compared.
const WALL_CLOCK: [&str; 1] = ["/playback/observedFrameEvents"];

/// Differences between an old record and a new one on the keys both share,
/// recursively, as `"/json/pointer: old != new"`. Arrays must have the same
/// length; numbers compare by value (`2` equals `2.0`).
pub fn compare_shared_keys(old: &Value, new: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk(old, new, String::new(), &mut out);
    out
}

fn walk(old: &Value, new: &Value, path: String, out: &mut Vec<String>) {
    if WALL_CLOCK.contains(&path.as_str()) {
        return;
    }
    match (old, new) {
        (Value::Object(a), Value::Object(b)) => {
            for (key, value) in a {
                if let Some(other) = b.get(key) {
                    walk(value, other, format!("{path}/{key}"), out);
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                walk(x, y, format!("{path}/{i}"), out);
            }
        }
        (Value::Number(a), Value::Number(b)) if a.as_f64() == b.as_f64() => {}
        (a, b) if a == b => {}
        (a, b) => out.push(format!("{path}: {} != {}", compact(a), compact(b))),
    }
}

fn compact(value: &Value) -> String {
    let text = value.to_string();
    if text.len() > 80 { format!("{}…", &text[..text.char_indices().nth(80).map_or(text.len(), |(i, _)| i)]) } else { text }
}

/// The outcome of re-verifying one recorded action.
#[derive(Clone, Debug, PartialEq)]
pub enum ActionValidation {
    /// Every shared key matched. `identical` when the records are also the same
    /// JSON (same keys in the same order), the wall-clock field aside.
    Matched {
        /// The canonical record reproduces the old one byte for byte.
        identical: bool,
    },
    /// The canonical run passed but recorded different values.
    Differs(Vec<String>),
    /// The canonical run failed a check the old page did not have or passed.
    Failed(String),
    /// The recorded package hash no longer matches set.json (not re-verified).
    Stale,
    /// The action is recorded but no longer in set.json.
    Missing,
}

/// Per-action validation of one set's evidence, in evidence order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvidenceValidation {
    /// `(action name, outcome)`.
    pub actions: Vec<(String, ActionValidation)>,
}

impl EvidenceValidation {
    /// Actions whose values all matched.
    pub fn matched(&self) -> usize {
        self.actions.iter().filter(|(_, v)| matches!(v, ActionValidation::Matched { .. })).count()
    }

    /// Actions whose canonical record is byte-identical to the old one.
    pub fn identical(&self) -> usize {
        self.actions.iter().filter(|(_, v)| *v == ActionValidation::Matched { identical: true }).count()
    }

    /// True when every re-verified action matched and none was stale or missing.
    pub fn all_matched(&self) -> bool {
        self.actions.iter().all(|(_, v)| matches!(v, ActionValidation::Matched { .. }))
    }
}

/// Re-verifies every action recorded in `evidence` with the canonical checks
/// plus `SetRules::for_set(set_id)` and compares the records.
///
/// `only` limits the run to the named actions. Runs in process; see the crate
/// docs for crash isolation.
pub fn validate_evidence(
    assets_dir: &Path,
    set_id: &str,
    evidence: &Report,
    only: Option<&[String]>,
) -> Result<EvidenceValidation, RenderError> {
    require_browser_parity()?;
    let path = assets_dir.join("set.json");
    let bytes = fs::read(&path).map_err(|source| RenderError::Io { path, source })?;
    let set = SetManifest::from_slice(&bytes)?;
    let rules = SetRules::for_set(set_id);
    let actions = set.all_actions();
    let mut validation = EvidenceValidation::default();
    for record in &evidence.assets {
        let name = record.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
        if only.is_some_and(|names| !names.contains(&name)) {
            continue;
        }
        let Some(meta) = actions.iter().find(|a| a.name == name) else {
            validation.actions.push((name, ActionValidation::Missing));
            continue;
        };
        if record.get("sha256").and_then(Value::as_str) != Some(meta.sha256.as_str()) {
            validation.actions.push((name, ActionValidation::Stale));
            continue;
        }
        let lottie_path = assets_dir.join(format!("{name}.lottie"));
        let lottie = fs::read(&lottie_path).map_err(|source| RenderError::Io { path: lottie_path, source })?;
        let run = verify_action(&set, meta, &lottie, &rules, ActionOptions { sheets: false });
        let outcome = match run.outcome {
            Outcome::Passed(new) => {
                let old = Value::Object(record.clone());
                let mut new = Value::Object(new);
                let differences = compare_shared_keys(&old, &new);
                if differences.is_empty() {
                    if let (Some(events), Some(slot)) = (old.pointer(WALL_CLOCK[0]), new.pointer_mut(WALL_CLOCK[0])) {
                        *slot = events.clone();
                    }
                    let identical = to_py_json(&old, Some(2), true) == to_py_json(&new, Some(2), true);
                    ActionValidation::Matched { identical }
                } else {
                    ActionValidation::Differs(differences)
                }
            }
            Outcome::Failed(message) => ActionValidation::Failed(message),
        };
        validation.actions.push((name, outcome));
    }
    Ok(validation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compares_only_shared_keys() {
        let old = json!({"name": "a", "sharedRest": true, "playback": {"completed": true, "observedFrameEvents": 81, "lastFrame": 28},
                         "sizes": [{"themes": [{"width": 2, "restHash": 1}]}]});
        let new = json!({"name": "a", "styleProbeFrame": 30, "playback": {"completed": true, "speed": 1, "observedFrameEvents": 28, "lastFrame": 28},
                         "sizes": [{"themes": [{"width": 2.0, "restHash": 1}]}]});
        assert!(compare_shared_keys(&old, &new).is_empty());
        let changed = json!({"name": "a", "sizes": [{"themes": [{"width": 2, "restHash": 2}]}]});
        assert_eq!(compare_shared_keys(&old, &changed), ["/sizes/0/themes/0/restHash: 1 != 2"]);
        let shorter = json!({"sizes": []});
        assert_eq!(compare_shared_keys(&old, &shorter).len(), 1);
    }
}
