//! The parts of a set's `assets/set.json` that the visual checks read.

use crate::player::Style;
use crate::RenderError;
use serde_json::Value;

/// One action (a packaged `.lottie`) as verify.html reads it from set.json.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionMeta {
    /// Action name: `.lottie` file stem and animation id.
    pub name: String,
    /// The icon (family) this action belongs to.
    pub icon: String,
    /// `light` or `strong`.
    pub motion_id: String,
    /// Package hash recorded in set.json.
    pub sha256: String,
    /// Frame count (144 in the catalog).
    pub frames: f64,
    /// Frames per second (60 in the catalog).
    pub fps: f64,
    /// Rest frame; may be fractional.
    pub rest: f64,
    /// Active motion length in seconds.
    pub active: f64,
    /// Key poses in seconds, for `poses.png`.
    pub key_poses: Vec<f64>,
    /// Styles to check, in order.
    pub styles: Vec<Style>,
    /// Continuous loader.
    pub continuous: bool,
    /// Stateful action (forward/reverse pair).
    pub stateful: bool,
    /// Preview-only reset action.
    pub preview_reset: bool,
    /// `forward` or `reverse` (absent in a few legacy actions).
    pub direction: String,
    /// Declared rotation cycle in degrees; 0 when absent.
    pub cycle_degrees: f64,
    /// `duotoneFill` as declared; absent means the duotone tint is expected.
    pub duotone_fill: Option<bool>,
}

impl ActionMeta {
    /// `previewReset && direction == "reverse"`: a reset that ends empty.
    pub fn is_reset_reverse(&self) -> bool {
        self.preview_reset && self.direction == "reverse"
    }

    /// JS truthiness of `cycleDegrees`.
    pub fn is_cyclic(&self) -> bool {
        self.cycle_degrees != 0.0 && !self.cycle_degrees.is_nan()
    }

    /// `duotoneFill !== false`.
    pub fn expects_duotone_fill(&self) -> bool {
        self.duotone_fill != Some(false)
    }
}

/// A motion of a family (`icon.motions[]`), with its reverse when stateful.
#[derive(Clone, Debug, PartialEq)]
pub struct Motion {
    /// Action name.
    pub name: String,
    /// `light` or `strong`.
    pub motion_id: String,
    /// Frame count.
    pub frames: f64,
    /// Rest frame.
    pub rest: f64,
    /// Active length in seconds.
    pub active: f64,
    /// Frames per second.
    pub fps: f64,
    /// Continuous loader.
    pub continuous: bool,
    /// The reverse action of a stateful motion.
    pub reverse: Option<Box<Motion>>,
}

/// An icon family: its forward motions (Light then Strong).
#[derive(Clone, Debug, PartialEq)]
pub struct Family {
    /// Icon name.
    pub name: String,
    /// Stateful icon.
    pub stateful: bool,
    /// Forward motions in set.json order.
    pub motions: Vec<Motion>,
}

/// A parsed set manifest.
#[derive(Clone, Debug)]
pub struct SetManifest {
    families: Vec<Family>,
    /// `(icon name, actions)` in set.json order.
    actions: Vec<(String, Vec<ActionMeta>)>,
}

fn field<'a>(record: &'a Value, key: &str, owner: &str) -> Result<&'a Value, RenderError> {
    record.get(key).ok_or_else(|| RenderError::Manifest(format!("{owner} has no `{key}`")))
}

fn number(record: &Value, key: &str, owner: &str) -> Result<f64, RenderError> {
    field(record, key, owner)?
        .as_f64()
        .ok_or_else(|| RenderError::Manifest(format!("{owner}: `{key}` is not a number")))
}

fn string(record: &Value, key: &str, owner: &str) -> Result<String, RenderError> {
    field(record, key, owner)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| RenderError::Manifest(format!("{owner}: `{key}` is not a string")))
}

/// JS truthiness of an optional field.
fn truthy(record: &Value, key: &str) -> bool {
    gai_core::js::truthy(record.get(key))
}

fn name_of(record: &Value) -> String {
    record.get("name").and_then(Value::as_str).unwrap_or("?").to_string()
}

fn parse_action(record: &Value) -> Result<ActionMeta, RenderError> {
    let owner = format!("action {}", name_of(record));
    let styles = field(record, "styles", &owner)?
        .as_array()
        .ok_or_else(|| RenderError::Manifest(format!("{owner}: `styles` is not a list")))?
        .iter()
        .map(|s| {
            s.as_str()
                .and_then(Style::parse)
                .ok_or_else(|| RenderError::Manifest(format!("{owner}: unknown style {s}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let key_poses = field(record, "keyPoses", &owner)?
        .as_array()
        .ok_or_else(|| RenderError::Manifest(format!("{owner}: `keyPoses` is not a list")))?
        .iter()
        .map(|t| t.as_f64().ok_or_else(|| RenderError::Manifest(format!("{owner}: key pose {t} is not a number"))))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ActionMeta {
        name: string(record, "name", &owner)?,
        icon: string(record, "icon", &owner)?,
        motion_id: string(record, "motionId", &owner)?,
        sha256: string(record, "sha256", &owner)?,
        frames: number(record, "frames", &owner)?,
        fps: number(record, "fps", &owner)?,
        rest: number(record, "rest", &owner)?,
        active: number(record, "active", &owner)?,
        key_poses,
        styles,
        continuous: truthy(record, "continuous"),
        stateful: truthy(record, "stateful"),
        preview_reset: truthy(record, "previewReset"),
        direction: record.get("direction").and_then(Value::as_str).unwrap_or_default().to_string(),
        cycle_degrees: record.get("cycleDegrees").and_then(Value::as_f64).unwrap_or(0.0),
        duotone_fill: record.get("duotoneFill").and_then(Value::as_bool),
    })
}

fn parse_motion(record: &Value) -> Result<Motion, RenderError> {
    let owner = format!("motion {}", name_of(record));
    Ok(Motion {
        name: string(record, "name", &owner)?,
        motion_id: string(record, "motionId", &owner)?,
        frames: number(record, "frames", &owner)?,
        rest: number(record, "rest", &owner)?,
        active: number(record, "active", &owner)?,
        fps: number(record, "fps", &owner)?,
        continuous: truthy(record, "continuous"),
        reverse: record.get("reverse").map(parse_motion).transpose()?.map(Box::new),
    })
}

impl SetManifest {
    /// Parses set.json bytes.
    pub fn from_slice(bytes: &[u8]) -> Result<SetManifest, RenderError> {
        let value: Value = serde_json::from_slice(bytes).map_err(|e| RenderError::Manifest(e.to_string()))?;
        SetManifest::from_value(&value)
    }

    /// Reads the set's icons. Legacy icons without `animations` use their `motions`.
    pub fn from_value(value: &Value) -> Result<SetManifest, RenderError> {
        let icons = value
            .get("icons")
            .and_then(Value::as_array)
            .ok_or_else(|| RenderError::Manifest("set.json has no `icons` list".into()))?;
        let mut families = Vec::new();
        let mut actions = Vec::new();
        for icon in icons {
            let name = string(icon, "name", "icon")?;
            let owner = format!("icon {name}");
            let motions = field(icon, "motions", &owner)?
                .as_array()
                .ok_or_else(|| RenderError::Manifest(format!("{owner}: `motions` is not a list")))?;
            let records = match icon.get("animations") {
                Some(list) => list
                    .as_array()
                    .ok_or_else(|| RenderError::Manifest(format!("{owner}: `animations` is not a list")))?,
                None => motions,
            };
            actions.push((name.clone(), records.iter().map(parse_action).collect::<Result<Vec<_>, _>>()?));
            families.push(Family {
                name,
                stateful: truthy(icon, "stateful"),
                motions: motions.iter().map(parse_motion).collect::<Result<Vec<_>, _>>()?,
            });
        }
        Ok(SetManifest { families, actions })
    }

    /// Actions in set.json order, optionally filtered by icon and action names
    /// (verify.html's `icons=` and `actions=` query parameters).
    pub fn actions(&self, icons: Option<&[String]>, names: Option<&[String]>) -> Vec<&ActionMeta> {
        self.actions
            .iter()
            .filter(|(icon, _)| icons.is_none_or(|only| only.contains(icon)))
            .flat_map(|(_, list)| list)
            .filter(|a| names.is_none_or(|only| only.contains(&a.name)))
            .collect()
    }

    /// Every action in set.json order.
    pub fn all_actions(&self) -> Vec<&ActionMeta> {
        self.actions(None, None)
    }

    /// The family record for an icon name.
    pub fn family(&self, icon: &str) -> Option<&Family> {
        self.families.iter().find(|f| f.name == icon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn action(name: &str, icon: &str) -> Value {
        json!({"name": name, "icon": icon, "motionId": "light", "sha256": "00", "frames": 144, "fps": 60,
               "rest": 31.200000000000003, "active": 0.5, "keyPoses": [0, 0.25], "styles": ["outline", "solid", "duotone"],
               "cycleDegrees": 0, "previewReset": false, "direction": "forward"})
    }

    #[test]
    fn reads_actions_and_legacy_motions() {
        let set = json!({"icons": [
            {"name": "heart", "motions": [action("heart", "heart"), action("heart-strong", "heart")]},
            {"name": "bell", "stateful": true,
             "motions": [{"name": "bell", "motionId": "light", "frames": 144, "rest": 20, "active": 0.4, "fps": 60,
                          "reverse": {"name": "bell-reset", "motionId": "light", "frames": 144, "rest": 20, "active": 0.4, "fps": 60}}],
             "animations": [action("bell", "bell"), action("bell-reset", "bell")]},
        ]});
        let manifest = SetManifest::from_value(&set).unwrap();
        let names: Vec<&str> = manifest.all_actions().iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["heart", "heart-strong", "bell", "bell-reset"]);
        let only = manifest.actions(Some(&["bell".into()]), Some(&["bell-reset".into()]));
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].rest, 31.200000000000003);
        assert!(!only[0].is_cyclic() && only[0].expects_duotone_fill());
        let bell = manifest.family("bell").unwrap();
        assert!(bell.stateful);
        assert_eq!(bell.motions[0].reverse.as_ref().unwrap().name, "bell-reset");
    }

    #[test]
    fn rejects_unknown_styles() {
        let mut bad = action("x", "x");
        bad["styles"] = json!(["outline", "neon"]);
        let set = json!({"icons": [{"name": "x", "motions": [], "animations": [bad]}]});
        assert!(SetManifest::from_value(&set).is_err());
    }
}
