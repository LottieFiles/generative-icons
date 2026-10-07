//! The library catalog: `catalog.json` from `@lottiefiles/generative-icons` (schema 1),
//! whose `icons` are descriptor v1 objects.

use crate::error::{CliError, ErrorCode, Result};
use serde_json::Value;

/// `^[a-z][a-z0-9-]*$`: the only ids the catalog, the lock and file names accept.
pub fn is_icon_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// One released icon.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryIcon {
    /// Stable id.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Category label.
    pub category: Option<String>,
    /// What the motion communicates.
    pub intent: Option<String>,
    /// Two-state icon with a reverse action.
    pub stateful: bool,
    /// Extra search words.
    pub aliases: Vec<String>,
    /// Catalog-relative `.lottie` path, e.g. `assets/heart.lottie`.
    pub asset_path: String,
    /// sha256 of the `.lottie` bytes.
    pub asset_sha256: String,
    /// The full descriptor, in catalog key order.
    pub descriptor: Value,
}

/// A parsed catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    /// Library version, e.g. `0.1.0`.
    pub version: String,
    /// Icons in catalog order.
    pub icons: Vec<LibraryIcon>,
    /// sha256 of the catalog bytes.
    pub sha256: String,
}

impl Catalog {
    /// Parses and validates `catalog.json` bytes read from `location`.
    pub fn parse(bytes: &[u8], location: &str) -> Result<Catalog> {
        let invalid = |detail: String| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Invalid library catalog at {location}: {detail}"))
                .with_path(location)
        };
        let value: Value = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
        if value.get("schemaVersion").and_then(Value::as_f64) != Some(1.0) {
            return Err(invalid("unsupported schemaVersion".into()));
        }
        let version = value.get("version").and_then(Value::as_str).ok_or_else(|| invalid("missing version".into()))?;
        let Some(Value::Array(entries)) = value.get("icons") else { return Err(invalid("missing icons".into())) };
        let mut icons: Vec<LibraryIcon> = Vec::with_capacity(entries.len());
        let mut seen = std::collections::HashSet::new();
        for descriptor in entries {
            let icon = LibraryIcon::parse(descriptor).map_err(invalid)?;
            if !seen.insert(icon.id.clone()) {
                return Err(invalid(format!("duplicate id {}", icon.id)));
            }
            icons.push(icon);
        }
        Ok(Catalog { version: version.to_string(), icons, sha256: gai_core::sha256_hex(bytes) })
    }

    /// The icon with this id.
    pub fn get(&self, id: &str) -> Option<&LibraryIcon> {
        self.icons.iter().find(|icon| icon.id == id)
    }
}

impl LibraryIcon {
    fn parse(descriptor: &Value) -> Result<LibraryIcon, String> {
        let text = |key: &str| descriptor.get(key).and_then(Value::as_str).map(str::to_string);
        let id = text("id").filter(|id| is_icon_id(id)).ok_or("an icon has an invalid id")?;
        let label = text("label").ok_or_else(|| format!("{id} has no label"))?;
        let stateful =
            descriptor.get("stateful").and_then(Value::as_bool).ok_or_else(|| format!("{id} has no stateful flag"))?;
        let asset = descriptor.get("asset");
        let asset_path = asset
            .and_then(|a| a.get("path"))
            .and_then(Value::as_str)
            .filter(|path| safe_relative(path))
            .ok_or_else(|| format!("{id} has an invalid asset.path"))?;
        let asset_sha256 = asset
            .and_then(|a| a.get("sha256"))
            .and_then(Value::as_str)
            .filter(|sha| sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
            .ok_or_else(|| format!("{id} has an invalid asset.sha256"))?;
        // Motion names become file names (preview sheets), so they must be plain ids.
        let motions = descriptor.get("motions").and_then(Value::as_array).into_iter().flatten();
        for motion in motions {
            for name in [motion.get("name"), motion.pointer("/reverse/name")].into_iter().flatten() {
                if !name.as_str().is_some_and(is_icon_id) {
                    return Err(format!("{id} has an invalid motion name"));
                }
            }
        }
        let aliases = match descriptor.get("aliases") {
            Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).map(str::to_string).collect(),
            _ => Vec::new(),
        };
        Ok(LibraryIcon {
            label,
            category: text("category"),
            intent: text("intent"),
            stateful,
            aliases,
            asset_path: asset_path.to_string(),
            asset_sha256: asset_sha256.to_string(),
            descriptor: descriptor.clone(),
            id,
        })
    }
}

/// A plain relative path: no traversal, empty or `.` parts, backslashes or leading `/`.
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_follow_the_catalog_alphabet() {
        for good in ["a", "heart", "a-0", "x9"] {
            assert!(is_icon_id(good), "{good}");
        }
        for bad in ["", "9a", "-a", "A", "a_b", "a.b", "a/b"] {
            assert!(!is_icon_id(bad), "{bad}");
        }
    }

    #[test]
    fn parse_validates_entries() {
        let sha = "a".repeat(64);
        let icon = format!(
            r#"{{"id":"heart","label":"Heart","stateful":false,"asset":{{"path":"assets/heart.lottie","sha256":"{sha}"}}}}"#
        );
        let catalog = |icons: &str| format!(r#"{{"schemaVersion":1,"version":"1.2.3","icons":[{icons}]}}"#);
        let parsed = Catalog::parse(catalog(&icon).as_bytes(), "x").unwrap();
        assert_eq!(parsed.get("heart").unwrap().asset_path, "assets/heart.lottie");
        let traversal = catalog(&icon.replace("assets/heart.lottie", "../heart.lottie"));
        assert_eq!(Catalog::parse(traversal.as_bytes(), "x").unwrap_err().code, ErrorCode::InvalidCatalog);
        let duplicate = Catalog::parse(catalog(&format!("{icon},{icon}")).as_bytes(), "x").unwrap_err();
        assert_eq!(duplicate.message, "Invalid library catalog at x: duplicate id heart");
        for name in ["/tmp/pwned", "../up", "a/b", "C:x"] {
            let motion = icon.replace("\"asset\"", &format!("\"motions\":[{{\"name\":{name:?}}}],\"asset\""));
            let error = Catalog::parse(catalog(&motion).as_bytes(), "x").unwrap_err();
            assert_eq!(error.message, "Invalid library catalog at x: heart has an invalid motion name", "{name}");
        }
        let reverse =
            icon.replace("\"asset\"", "\"motions\":[{\"name\":\"heart\",\"reverse\":{\"name\":\"../x\"}}],\"asset\"");
        assert!(Catalog::parse(catalog(&reverse).as_bytes(), "x").is_err());
    }
}
