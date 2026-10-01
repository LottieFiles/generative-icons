//! The packaging fields of a set's `set-config.json`, and the paths of one set.

use crate::error::{PackageError, Result, invalid};
use crate::io::read;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Locations of one icon set inside the library repository.
#[derive(Debug, Clone)]
pub struct SetPaths {
    /// Repository root: the folder holding `motion/icon-sets` and `prototype/icons`.
    pub repo: PathBuf,
    /// `motion/icon-sets/<set-id>`: definitions, evidence and `set-config.json`.
    pub root: PathBuf,
    /// `prototype/icons/<NN>/assets`: the published assets and `set.json`.
    pub assets: PathBuf,
    id: String,
}

impl SetPaths {
    /// The paths of set `set_id`, the folder name under `motion/icon-sets` (`13-fintech-finance`).
    pub fn new(repo: impl Into<PathBuf>, set_id: &str) -> Result<Self> {
        let bytes = set_id.as_bytes();
        let valid = bytes.len() > 3
            && bytes[..2].iter().all(u8::is_ascii_digit)
            && bytes[2] == b'-'
            && !set_id.contains(['/', '\\']);
        if !valid {
            return Err(invalid(format!(
                "Invalid set id {set_id}: expected NN-slug"
            )));
        }
        let repo = repo.into();
        Ok(SetPaths {
            root: repo.join("motion/icon-sets").join(set_id),
            assets: repo
                .join("prototype/icons")
                .join(&set_id[..2])
                .join("assets"),
            repo,
            id: set_id.to_string(),
        })
    }

    /// The set id (`13-fintech-finance`).
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The two-digit set number (`13`).
    pub fn number(&self) -> &str {
        &self.id[..2]
    }

    /// `motion/icon-sets/library`.
    pub(crate) fn library(&self) -> PathBuf {
        self.repo.join("motion/icon-sets/library")
    }
}

/// The fields of `set-config.json` that packaging reads. Unknown fields are ignored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetConfig {
    /// The set id; must equal the set's folder name.
    pub id: String,
    /// Extra `motion/icon-sets/library/<name>` files for the source bundle.
    #[serde(default)]
    pub shared_sources: Vec<String>,
    /// Review folder relative to `motion/icon-sets`, bundled as its `*.md` and
    /// `invocation.json` files. Defaults to `library/design-reviews/ai-category/<id>`.
    #[serde(default)]
    pub design_review: Option<String>,
    /// Replaces `set.json`'s `category` before packaging.
    #[serde(default)]
    pub category_override: Option<String>,
    /// Replaces `set.json`'s `defaultMotion` before packaging.
    #[serde(default)]
    pub default_motion_override: Option<String>,
    /// Repository-relative files or patterns appended to the set zip as `source/<path>`.
    /// `*` and `?` match within a path component and `**` matches any number of folders;
    /// matches are sorted, missing files and names already in the zip are skipped.
    #[serde(default)]
    pub package_extras: Vec<String>,
}

impl SetConfig {
    /// Reads `<root>/set-config.json`.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("set-config.json");
        serde_json::from_slice(&read(&path)?).map_err(|source| PackageError::Json { path, source })
    }
}
