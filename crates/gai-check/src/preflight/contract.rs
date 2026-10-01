//! The numeric family contract (`motion/icon-sets/library/family-contract.json`). It is read
//! from disk on every run, as `preflight.mjs:10-11` did, so contract edits flow through.

use super::error::{Error, Result};
use gai_core::js_round;
use serde_json::{Map, Value as Json};
use std::{fs, path::Path};

/// The fields preflight reads.
#[derive(Debug, Clone, PartialEq)]
pub struct FamilyContract {
    /// `grid`: the canvas edge, the hull check's upper bound.
    pub grid: f64,
    /// `outlineWeights` in JS property order: document order, except that integer-like
    /// keys would come first. Outline parts are sampled at each width in this order.
    pub outline_weights: Vec<(String, f64)>,
    /// `sourceStyles`: the paint styles a part may use.
    pub source_styles: Vec<String>,
    /// `profiles`: the motion profiles sampled, in order (`light`, `strong`).
    pub profiles: Vec<String>,
    /// Frames per second of the sampled timeline.
    pub fps: f64,
    /// The timeline length; action and rest times must lie inside it.
    pub duration_seconds: f64,
}

impl FamilyContract {
    /// The contract's path below the repo root.
    pub const FILE: &'static str = "motion/icon-sets/library/family-contract.json";

    /// Reads `<root>/motion/icon-sets/library/family-contract.json`.
    pub fn load(root: &Path) -> Result<Self> {
        let path = super::paths::join(root, Self::FILE);
        let bytes = fs::read(&path).map_err(Error::io(&path))?;
        let json: Json = serde_json::from_slice(&bytes).map_err(|source| Error::Json { path: path.clone(), source })?;
        Self::from_json(&json).map_err(|message| Error::Invalid { path, message })
    }

    /// The contract from its parsed JSON (parsed with key order preserved).
    pub fn from_json(json: &Json) -> Result<Self, String> {
        let number = |key: &str| json.get(key).and_then(Json::as_f64).ok_or(format!("`{key}` must be a number"));
        let strings = |key: &str| {
            json.get(key)
                .and_then(Json::as_array)
                .and_then(|items| items.iter().map(|item| item.as_str().map(str::to_owned)).collect())
                .ok_or(format!("`{key}` must be an array of strings"))
        };
        let weights =
            json.get("outlineWeights").and_then(Json::as_object).ok_or("`outlineWeights` must be an object")?;
        let contract = FamilyContract {
            grid: number("grid")?,
            outline_weights: js_property_order(weights)
                .map(|(key, value)| value.as_f64().map(|w| (key.clone(), w)))
                .collect::<Option<_>>()
                .ok_or("`outlineWeights` values must be numbers")?,
            source_styles: strings("sourceStyles")?,
            profiles: strings("profiles")?,
            fps: number("fps")?,
            duration_seconds: number("durationSeconds")?,
        };
        let frames = contract.duration_seconds * contract.fps;
        if !(0.0..=1e6).contains(&frames) {
            return Err("`durationSeconds` × `fps` must be a frame count between 0 and 1e6".into());
        }
        Ok(contract)
    }

    /// `outlineWeights.regular`, the width an outline stroke must be authored at.
    pub fn regular_width(&self) -> Option<f64> {
        self.outline_weights.iter().find(|(name, _)| name == "regular").map(|(_, width)| *width)
    }

    /// The frame times every action is sampled at: `f / fps` for
    /// `f` in `0..=Math.round(durationSeconds * fps)` (a division, `preflight.mjs:87`).
    pub fn frame_times(&self) -> Vec<f64> {
        let count = js_round(self.duration_seconds * self.fps) as usize + 1;
        (0..count).map(|frame| frame as f64 / self.fps).collect()
    }
}

/// `Object.entries` order: array-index keys ascending, then the rest in insertion order.
fn js_property_order(map: &Map<String, Json>) -> impl Iterator<Item = (&String, &Json)> {
    let index = |key: &str| key.parse::<u32>().ok().filter(|n| *n != u32::MAX && n.to_string() == key);
    let mut indexed: Vec<_> = map.iter().filter_map(|(key, value)| index(key).map(|n| (n, key, value))).collect();
    indexed.sort_by_key(|(n, _, _)| *n);
    let named = map.iter().filter(move |(key, _)| index(key).is_none());
    indexed.into_iter().map(|(_, key, value)| (key, value)).chain(named)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn contract(weights: Json) -> FamilyContract {
        FamilyContract::from_json(&json!({
            "grid": 32, "outlineWeights": weights, "sourceStyles": ["outline", "solid", "tint"],
            "profiles": ["light", "strong"], "fps": 60, "durationSeconds": 2.4
        }))
        .unwrap()
    }

    #[test]
    fn weights_keep_document_order() {
        let c = contract(json!({"light": 1.5, "regular": 2, "bold": 2.5}));
        let names: Vec<_> = c.outline_weights.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["light", "regular", "bold"]);
        assert_eq!(c.regular_width(), Some(2.0));
        // 2.4 * 60 is exactly 144; frame 13 is 13/60, not 13 * (1/60).
        let times = c.frame_times();
        assert_eq!(times.len(), 145);
        assert_eq!(times[13], 0.21666666666666667);
        assert_eq!(times[144], 2.4);
    }

    #[test]
    fn integer_like_weight_keys_come_first_as_in_js() {
        let c = contract(json!({"light": 1.5, "10": 3, "2": 2}));
        let names: Vec<_> = c.outline_weights.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["2", "10", "light"]);
        assert_eq!(c.regular_width(), None);
    }

    #[test]
    fn wrong_shapes_are_refused() {
        assert!(FamilyContract::from_json(&json!({"grid": "32"})).is_err());
        let mut bad = json!({
            "grid": 32, "outlineWeights": {"light": "1.5"}, "sourceStyles": [], "profiles": [], "fps": 60, "durationSeconds": 2.4
        });
        assert!(FamilyContract::from_json(&bad).is_err());
        bad["outlineWeights"] = json!({});
        bad["fps"] = json!(-1);
        assert!(FamilyContract::from_json(&bad).is_err());
    }
}
