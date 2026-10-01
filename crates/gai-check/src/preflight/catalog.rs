//! The published catalog (`loadCatalog`, `generation-context.mjs:24-45`): every icon of every
//! set listed in `prototype/icons/catalog.json`, and the hashes of the files read.

use super::{
    error::{Error, Result},
    paths,
};
use gai_core::js::{to_js_string, truthy};
use gai_core::{js_number_to_string, sha256_hex};
use serde_json::{Map, Value as Json};
use std::{collections::HashSet, fs, path::Path};

/// A file a report depends on, with the sha256 of its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The path as it was built (lexically joined, never canonicalized).
    pub path: String,
    /// Lowercase hex.
    pub sha256: String,
}

impl Input {
    /// Reads and hashes `path`.
    pub fn read(path: &Path) -> Result<(Self, Vec<u8>)> {
        let bytes = fs::read(path).map_err(Error::io(path))?;
        let input = Input { path: path.to_string_lossy().into_owned(), sha256: sha256_hex(&bytes) };
        Ok((input, bytes))
    }

    /// `{"path", "sha256"}`.
    pub fn to_json(&self) -> Json {
        let mut map = Map::new();
        map.insert("path".into(), self.path.clone().into());
        map.insert("sha256".into(), self.sha256.clone().into());
        Json::Object(map)
    }
}

/// One published icon.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogIcon {
    /// `icon.icon || icon.name`; `None` when that is `undefined`.
    pub id: Option<Json>,
    /// The set's `id` (for example `01-heart-bell`); `None` when absent.
    pub set: Option<Json>,
    /// `prototype/icons/<entry of catalog.sets>`, for example `prototype/icons/01/assets/set.json`.
    pub manifest: String,
    /// `{...icon, id, category, set, manifest, assetDirectory}` as `loadCatalog` builds it,
    /// without the keys whose value is `undefined`.
    pub record: Map<String, Json>,
}

/// Every icon in `catalog.sets` order (not sorted), and the files read: `catalog.json`
/// first, then each `set.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    /// Every icon, set by set.
    pub icons: Vec<CatalogIcon>,
    /// `catalog.json`, then each `set.json`, as read.
    pub inputs: Vec<Input>,
}

impl Catalog {
    /// The catalog file below the repo root.
    pub const FILE: &'static str = "prototype/icons/catalog.json";

    /// Loads the catalog of the repo at `root`.
    pub fn load(root: &Path) -> Result<Self> {
        let mut inputs = Vec::new();
        let mut read = |relative: &str| -> Result<(Json, std::path::PathBuf)> {
            let path = paths::join(root, relative);
            let (input, bytes) = Input::read(&path)?;
            inputs.push(input);
            let json = serde_json::from_slice(&bytes).map_err(|source| Error::Json { path: path.clone(), source })?;
            Ok((json, path))
        };
        let (catalog, catalog_path) = read(Self::FILE)?;
        let invalid = |path: &Path, message: &str| Error::Invalid { path: path.to_owned(), message: message.into() };
        let sets = catalog
            .get("sets")
            .and_then(Json::as_array)
            .ok_or_else(|| invalid(&catalog_path, "`sets` must be an array"))?;
        let categories = catalog
            .get("categories")
            .and_then(Json::as_array)
            .ok_or_else(|| invalid(&catalog_path, "`categories` must be an array"))?;
        let mut icons = Vec::new();
        let mut seen = HashSet::new();
        for relative in sets {
            let relative = relative.as_str().ok_or_else(|| invalid(&catalog_path, "`sets` must list paths"))?;
            let manifest = format!("prototype/icons/{relative}");
            let (set, set_path) = read(&manifest)?;
            let category = find_category(categories, relative).map_err(|message| invalid(&catalog_path, message))?;
            // `category?.label || set.category || set.name`
            let category = or(or(category.and_then(|c| c.get("label")), set.get("category")), set.get("name"));
            let set_id = set.get("id").cloned();
            let members = set
                .get("icons")
                .and_then(Json::as_array)
                .ok_or_else(|| invalid(&set_path, "`icons` must be an array"))?;
            for icon in members {
                let icon = icon.as_object().ok_or_else(|| invalid(&set_path, "every icon must be an object"))?;
                let id = or(icon.get("icon"), icon.get("name")).cloned();
                if let Some(key) = same_value_key(id.as_ref())
                    && !seen.insert(key)
                {
                    return Err(Error::DuplicateCatalogId(to_js_string(id.as_ref())));
                }
                let mut record = icon.clone();
                let asset_directory = manifest.rsplit_once('/').map_or(".", |(dir, _)| dir).to_owned();
                let fields = [
                    ("id", id.clone()),
                    ("category", category.cloned()),
                    ("set", set_id.clone()),
                    ("manifest", Some(manifest.clone().into())),
                    ("assetDirectory", Some(asset_directory.into())),
                ];
                for (key, value) in fields {
                    match value {
                        Some(value) => record.insert(key.into(), value),
                        None => record.shift_remove(key),
                    };
                }
                icons.push(CatalogIcon { id, set: set_id.clone(), manifest: manifest.clone(), record });
            }
        }
        Ok(Catalog { icons, inputs })
    }

    /// The first icon whose ID is the string `id` (`catalogIcons.find(i => i.id === name)`).
    pub fn find(&self, id: &str) -> Option<&CatalogIcon> {
        self.icons.iter().find(|icon| icon.id.as_ref().and_then(Json::as_str) == Some(id))
    }
}

/// `catalog.categories.find(c => c.sets.includes(relative))`.
fn find_category<'a>(categories: &'a [Json], relative: &str) -> Result<Option<&'a Map<String, Json>>, &'static str> {
    for category in categories {
        let includes = match category.get("sets") {
            Some(Json::Array(sets)) => sets.iter().any(|set| set.as_str() == Some(relative)),
            Some(Json::String(sets)) => sets.contains(relative),
            _ => return Err("every category needs a `sets` list"),
        };
        if includes {
            return Ok(category.as_object());
        }
    }
    Ok(None)
}

/// JS `a || b`.
fn or<'a>(a: Option<&'a Json>, b: Option<&'a Json>) -> Option<&'a Json> {
    if truthy(a) { a } else { b }
}

/// A key equal for exactly the values a JS `Set` treats as the same. Objects and arrays
/// are distinct from everything, so they get none.
fn same_value_key(value: Option<&Json>) -> Option<String> {
    match value {
        None => Some("undefined".into()),
        Some(Json::Array(_) | Json::Object(_)) => None,
        Some(Json::Number(n)) => n.as_f64().map(|n| format!("n{}", js_number_to_string(n))),
        Some(other) => Some(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_keys() {
        assert_eq!(same_value_key(Some(&json!("1"))), Some("\"1\"".into()));
        assert_ne!(same_value_key(Some(&json!("1"))), same_value_key(Some(&json!(1))));
        assert_eq!(same_value_key(Some(&json!(1))), same_value_key(Some(&json!(1.0))));
        assert_eq!(same_value_key(Some(&json!({}))), None);
    }
}
