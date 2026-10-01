//! The inputs clearance reads: scene JSON (world geometry per action) and `set.json`.

use super::ContractError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A keyframed track, JSON `[[t, value], …]`, in file order. Samplers need at least one key.
pub type Track<T> = Vec<(f64, T)>;

/// Path command codes in scene `commands`.
pub mod cmd {
    /// Move to (2 coordinates).
    pub const MOVE: u8 = 0;
    /// Line to (2 coordinates).
    pub const LINE: u8 = 1;
    /// Cubic to (6 coordinates: two controls and the end).
    pub const CUBIC: u8 = 2;
    /// Close path (no coordinates).
    pub const CLOSE: u8 = 3;

    /// Coordinates a command consumes (`width(c)` in the JS).
    pub fn coords(c: u8) -> usize {
        match c {
            CUBIC => 6,
            MOVE | LINE => 2,
            _ => 0,
        }
    }
}

/// One `SET/scenes/<action>.scene.json`: every point already posed and rounded to 1e-5.
#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    /// The action name (file stem).
    pub name: String,
    /// Drawing nodes in paint order.
    pub nodes: Vec<SceneNode>,
}

/// One painted path of a scene.
#[derive(Debug, Clone, Deserialize)]
pub struct SceneNode {
    /// `<icon>-<part>[-<weight>]__<colorToken>__<styleToken>`.
    pub name: String,
    /// Path commands (see [`cmd`]); unknown codes fail when flattened.
    pub commands: Vec<u8>,
    /// Flat coordinates per key.
    pub points: Track<Vec<f64>>,
    /// Local rest geometry, present on rigid (transformed) nodes.
    #[serde(rename = "basePoints", default)]
    pub base_points: Option<Vec<f64>>,
    /// Whether the node carries a rigid `transform` block.
    #[serde(default, deserialize_with = "present")]
    pub transform: bool,
    /// `"stroke"` or `"fill"`. Only strokes have a paint radius.
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Stroke width track.
    pub width: Track<f64>,
    /// Node opacity track. Absent only in the 01/02 scenes.
    #[serde(default)]
    pub visibility: Option<Track<f64>>,
}

fn present<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(!value.is_null())
}

impl SceneNode {
    /// The name up to the first `__` (the packaged layer name).
    pub fn base(&self) -> &str {
        self.name.split("__").next().unwrap_or(&self.name)
    }

    /// Whether the node is painted as a stroke.
    pub fn is_stroke(&self) -> bool {
        self.kind.as_deref() == Some("stroke")
    }

    /// The visibility track, or an error for scenes without one.
    pub fn visibility(&self) -> Result<&Track<f64>, ContractError> {
        self.visibility.as_ref().ok_or_else(|| {
            ContractError::Geometry(format!("{} has no visibility track", self.name))
        })
    }
}

impl Scene {
    /// Parses scene JSON. Tracks must be non-empty, since every sampler reads their first key.
    pub fn from_slice(bytes: &[u8]) -> Result<Scene, serde_json::Error> {
        let scene: Scene = serde_json::from_slice(bytes)?;
        for node in &scene.nodes {
            let empty = node.points.is_empty()
                || node.width.is_empty()
                || node.visibility.as_ref().is_some_and(Vec::is_empty);
            if empty {
                return Err(serde::de::Error::custom(format!(
                    "{} has an empty track",
                    node.name
                )));
            }
        }
        Ok(scene)
    }
}

/// Where scenes come from: a set's `scenes/` folder, or scenes the builder holds in memory.
pub trait SceneSource: Sync {
    /// Loads the scene for one action (`<name>.scene.json`).
    fn load(&self, name: &str) -> Result<Scene, ContractError>;
}

/// Scenes read from `<dir>/<name>.scene.json`.
#[derive(Debug, Clone)]
pub struct ScenesDir(pub PathBuf);

impl SceneSource for ScenesDir {
    fn load(&self, name: &str) -> Result<Scene, ContractError> {
        let path = self.0.join(format!("{name}.scene.json"));
        let bytes = std::fs::read(&path).map_err(|source| ContractError::Io {
            path: path.clone(),
            source,
        })?;
        Scene::from_slice(&bytes).map_err(|source| ContractError::Json { path, source })
    }
}

/// The `set.json` fields clearance reads.
#[derive(Debug, Clone, Deserialize)]
pub struct SetManifest {
    /// Set id, e.g. `39-food-drink`.
    pub id: String,
    /// Icons in file order.
    pub icons: Vec<SetIcon>,
}

/// One icon record of `set.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct SetIcon {
    /// Base icon name; the contract key.
    pub name: String,
    /// Public icon id (equal to `name` in the catalog; set 45 reads it).
    #[serde(default)]
    pub icon: Option<String>,
    /// Every action in build order: Light, Light reverse, Strong, Strong reverse.
    #[serde(default)]
    pub animations: Vec<SetAction>,
}

/// One action (animation) record of `set.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct SetAction {
    /// Action name, the scene file stem.
    pub name: String,
    /// `"forward"` or `"reverse"`.
    #[serde(default)]
    pub direction: Option<String>,
    /// `"light"` or `"strong"`.
    #[serde(rename = "motionId", default)]
    pub motion_id: Option<String>,
}

impl SetManifest {
    /// Reads and parses a `set.json`.
    pub fn read(path: &Path) -> Result<SetManifest, ContractError> {
        let bytes = std::fs::read(path).map_err(|source| ContractError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| ContractError::Json {
            path: path.to_path_buf(),
            source,
        })
    }
}
