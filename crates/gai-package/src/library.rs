//! Catalog-wide outputs: `icon-library.zip` (`motion/icon-sets/library/package.py`) and the
//! asset audit (`scripts/audit-assets.py`).

use crate::error::{PackageError, Result, invalid};
use crate::io::{array_field, field, py_pretty, read, read_json, str_field, write};
use crate::zip::{ZipDate, deflate_all, read_zip, write_zip};
use gai_core::parallel::parallel_map;
use gai_core::sha256_hex;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// `prototype/icons/catalog.json`: the published sets in gallery order and their categories.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// Set manifests relative to `prototype/icons` (`13/assets/set.json`), in gallery order.
    pub sets: Vec<String>,
    /// Categories in catalog order.
    pub categories: Vec<Category>,
}

/// A catalog category.
#[derive(Debug, Clone)]
pub struct Category {
    /// The category id (`fintech-finance`).
    pub id: String,
    /// The display label.
    pub label: String,
    /// Set manifests in this category.
    pub sets: Vec<String>,
}

impl Catalog {
    /// Reads `<repo>/prototype/icons/catalog.json`.
    pub fn load(repo: &Path) -> Result<Self> {
        let value = read_json(&repo.join("prototype/icons/catalog.json"))?;
        let strings = |items: &Vec<Value>, context: &str| -> Result<Vec<String>> {
            items
                .iter()
                .map(|s| {
                    s.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| invalid(format!("{context}: expected strings")))
                })
                .collect()
        };
        let sets = strings(
            array_field(&value, "sets", "catalog.json")?,
            "catalog.json sets",
        )?;
        let categories = array_field(&value, "categories", "catalog.json")?
            .iter()
            .map(|c| {
                Ok(Category {
                    id: str_field(c, "id", "catalog category")?.to_string(),
                    label: str_field(c, "label", "catalog category")?.to_string(),
                    sets: strings(
                        array_field(c, "sets", "catalog category")?,
                        "catalog category sets",
                    )?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Catalog { sets, categories })
    }

    /// The first category listing `set` (the npm package uses this, not `set.json.category`).
    pub fn category_of(&self, set: &str) -> Option<&Category> {
        self.categories
            .iter()
            .find(|c| c.sets.iter().any(|s| s == set))
    }
}

/// A set manifest from the catalog.
pub(crate) struct CatalogSet {
    /// `<repo>/prototype/icons/<relative>`.
    pub path: PathBuf,
    pub relative: String,
    pub bytes: Vec<u8>,
    pub record: Value,
}

impl CatalogSet {
    pub(crate) fn load(repo: &Path, relative: &str) -> Result<Self> {
        let path = repo.join("prototype/icons").join(relative);
        let bytes = read(&path)?;
        let record = serde_json::from_slice(&bytes).map_err(|source| PackageError::Json {
            path: path.clone(),
            source,
        })?;
        Ok(CatalogSet {
            path,
            relative: relative.to_string(),
            bytes,
            record,
        })
    }

    pub(crate) fn folder(&self) -> &Path {
        self.path.parent().unwrap_or(Path::new("."))
    }

    pub(crate) fn icons(&self) -> Result<&Vec<Value>> {
        array_field(&self.record, "icons", &self.relative)
    }
}

/// `icon.get("animations", icon["motions"])`: `motions` must exist even when `animations`
/// does, and an existing `animations` list is used even when empty.
pub(crate) fn icon_actions(icon: &Value) -> Result<&Vec<Value>> {
    let name = icon.get("name").and_then(Value::as_str).unwrap_or("?");
    let motions = array_field(icon, "motions", &format!("icon {name}"))?;
    match icon.get("animations") {
        Some(animations) => animations
            .as_array()
            .ok_or_else(|| invalid(format!("icon {name}: animations is not an array"))),
        None => Ok(motions),
    }
}

/// The `delivery.json` record of an `icon-library.zip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryProof {
    /// Icons in the archive.
    pub icons: usize,
    /// `.lottie` entries in the archive.
    pub actions: usize,
    /// Archive size.
    pub bytes: u64,
    /// Archive sha256 (lowercase hex).
    pub sha256: String,
    /// Repository-relative POSIX path, or the file name when written outside the repository.
    pub path: String,
}

impl DeliveryProof {
    /// `{icons, actions, bytes, sha256, path}` in that key order.
    pub fn to_json(&self) -> Value {
        json!({ "icons": self.icons, "actions": self.actions, "bytes": self.bytes, "sha256": self.sha256, "path": self.path })
    }
}

const LIBRARY_README: &str = "Generative Icons by LottieFiles\n{N} original icons, Light and Strong motion. Stateful icons include explicit reverse or preview-reset actions.\nNative vector Lottie: 32 x 32, 60 fps. Each .lottie contains all actions for its icon; the filename selects its initial action. Each .json contains one action. Named color and style slots remain editable. Metadata records native provenance, timings, and hashes.\nGallery default: Strong motion, Outline, Regular weight, dark theme. Loading indicators marked continuous:true use native player looping; use frame0 for reduced motion. Themes: dark/light. Style slots: style.outline.light, style.outline.regular, style.outline.bold, style.solid, style.tint (Duotone: 18). Stroke-width token: style.outline.width (the Regular outline at any weight, 1 to 3). Primary color slot: color.primary.\nSource and documentation: https://github.com/LottieFiles/generative-icons\n";

/// Writes `icon-library.zip`: the notices, then per catalog set every action's `.lottie` and
/// `.json` (flat) and `metadata/<set-id>.json`, then a README. Every `.lottie` must match
/// both `set.json` and the set's passing `browser-verification.json`.
///
/// With `output = None` the archive goes to `prototype/icons/01/assets/icon-library.zip`
/// and the proof is also written to `motion/icon-sets/library/evidence/delivery.json`.
pub fn package_library(repo: &Path, output: Option<&Path>) -> Result<DeliveryProof> {
    let catalog = Catalog::load(repo)?;
    let date = ZipDate::ICON_LIBRARY;
    let mut items: Vec<(String, Vec<u8>)> = ["LICENSE", "THIRD_PARTY_NOTICES.md"]
        .iter()
        .map(|n| Ok((n.to_string(), read(&repo.join(n))?)))
        .collect::<Result<_>>()?;
    let mut icons = 0;
    for relative in &catalog.sets {
        let set = CatalogSet::load(repo, relative)?;
        let id = str_field(&set.record, "id", relative)?;
        let proof = read_json(
            &repo
                .join("motion/icon-sets")
                .join(id)
                .join("evidence/browser-verification.json"),
        )?;
        if proof.get("result").and_then(Value::as_str) != Some("PASS") {
            return Err(invalid(format!("Native proof incomplete: {id}")));
        }
        let mut verified = HashMap::new();
        for asset in array_field(&proof, "assets", "browser-verification.json")? {
            verified.insert(
                str_field(asset, "name", "browser-verification.json")?.to_string(),
                field(asset, "sha256", "browser-verification.json")?.clone(),
            );
        }
        for icon in set.icons()? {
            icons += 1;
            for action in icon_actions(icon)? {
                let name = str_field(action, "name", "action")?;
                let lottie = read(&set.folder().join(format!("{name}.lottie")))?;
                let digest = Value::String(sha256_hex(&lottie));
                if action.get("sha256") != Some(&digest) || verified.get(name) != Some(&digest) {
                    return Err(invalid(format!("Native proof stale: {name}")));
                }
                items.push((format!("{name}.lottie"), lottie));
                items.push((
                    format!("{name}.json"),
                    read(&set.folder().join(format!("{name}.json")))?,
                ));
            }
        }
        items.push((
            format!("metadata/{id}.json"),
            py_pretty(&set.record).into_bytes(),
        ));
    }
    items.push((
        "README.txt".into(),
        LIBRARY_README
            .replace("{N}", &icons.to_string())
            .into_bytes(),
    ));

    let target = output.map_or_else(
        || repo.join("prototype/icons/01/assets/icon-library.zip"),
        Path::to_path_buf,
    );
    let bytes = write_zip(&deflate_all(items, date)).map_err(|e| PackageError::zip(&target, e))?;
    let files = read_zip(&bytes).map_err(|e| PackageError::zip(&target, e))?;
    write(&target, &bytes)?;

    let resolved = target
        .canonicalize()
        .map_err(|e| PackageError::io(&target, e))?;
    let root = repo.canonicalize().map_err(|e| PackageError::io(repo, e))?;
    let path = match resolved.strip_prefix(&root) {
        Ok(rest) => rest
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
        Err(_) => target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
    };
    let proof = DeliveryProof {
        icons,
        actions: files.iter().filter(|f| f.name.ends_with(".lottie")).count(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
        path,
    };
    if output.is_none() {
        write(
            &repo.join("motion/icon-sets/library/evidence/delivery.json"),
            py_pretty(&proof.to_json()) + "\n",
        )?;
    }
    Ok(proof)
}

/// The counts `audit_assets` checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetAudit {
    /// Catalog sets.
    pub sets: usize,
    /// Distinct icons.
    pub icons: usize,
    /// Actions (`.lottie` + `.json` pairs).
    pub actions: usize,
}

impl AssetAudit {
    /// The report `audit-assets.py` printed: `{result, sets, icons, actions, scope}`.
    pub fn to_json(&self) -> Value {
        json!({
            "result": "PASS",
            "sets": self.sets,
            "icons": self.icons,
            "actions": self.actions,
            "scope": "Catalog IDs, native package hashes, ZIP integrity and loose JSON presence. Not a visual or motion-quality review.",
        })
    }
}

/// `scripts/audit-assets.py`: icon names are unique across the catalog, every action's
/// `.lottie` matches its `set.json` sha256 and passes an archive integrity check, and every
/// action's loose `.json` exists.
pub fn audit_assets(repo: &Path) -> Result<AssetAudit> {
    enum Check {
        Duplicate(String),
        Action {
            folder: PathBuf,
            name: String,
            sha256: Option<String>,
        },
    }
    let catalog = Catalog::load(repo)?;
    let mut ids = HashSet::new();
    let mut checks = Vec::new();
    for relative in &catalog.sets {
        let set = CatalogSet::load(repo, relative)?;
        for icon in set.icons()? {
            let name = str_field(icon, "name", relative)?;
            if !ids.insert(name.to_string()) {
                checks.push(Check::Duplicate(name.to_string()));
            }
            for action in icon_actions(icon)? {
                let action_name = str_field(action, "name", name)?.to_string();
                let sha256 = action
                    .get("sha256")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                checks.push(Check::Action {
                    folder: set.folder().to_path_buf(),
                    name: action_name,
                    sha256,
                });
            }
        }
    }
    let actions = checks
        .iter()
        .filter(|c| matches!(c, Check::Action { .. }))
        .count();
    // Checked in parallel; the first failure in catalog order is reported.
    let results = parallel_map(checks, |check| match check {
        Check::Duplicate(name) => Err(invalid(format!("Duplicate icon: {name}"))),
        Check::Action {
            folder,
            name,
            sha256,
        } => check_action(&folder, &name, sha256.as_deref()),
    });
    results.into_iter().collect::<Result<()>>()?;
    Ok(AssetAudit {
        sets: catalog.sets.len(),
        icons: ids.len(),
        actions,
    })
}

fn check_action(folder: &Path, name: &str, sha256: Option<&str>) -> Result<()> {
    let path = folder.join(format!("{name}.lottie"));
    let bytes = read(&path)?;
    if Some(sha256_hex(&bytes).as_str()) != sha256 {
        return Err(invalid(format!(
            "Changed native package: {}",
            path.display()
        )));
    }
    read_zip(&bytes).map_err(|_| invalid(format!("Invalid archive: {}", path.display())))?;
    let loose = folder.join(format!("{name}.json"));
    if !loose.exists() {
        return Err(invalid(format!("Missing loose JSON: {}", loose.display())));
    }
    Ok(())
}
