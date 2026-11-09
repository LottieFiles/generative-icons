//! The npm package contents (`scripts/build-packages.py`): per-icon descriptors and assets,
//! the package `catalog.json`, docs, schemas, the CLI skill and `release.json`, written to
//! `packages/<name>/dist` for `runtime`, `react`, `icons` and `cli`. Nothing is published.

use crate::error::{PackageError, Result, invalid};
use crate::io::{
    array_field, field, py_compact, py_pretty, py_truthy, read, read_json, str_field, write,
};
use crate::library::{Catalog, CatalogSet};
use crate::paths::{glob, py_suffix, sort_paths};
use crate::release::GitState;
use crate::zip::{find_file, read_zip};
use gai_core::js::truthy;
use gai_core::sha256_hex;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// The packages in build order.
pub const PACKAGES: [&str; 4] = ["runtime", "react", "icons", "cli"];

/// Public docs copied into the icons package.
const DOCS: [&str; 9] = [
    "using-packages.md",
    "using-icons.md",
    "package-contract.md",
    "creating-icons.md",
    "development.md",
    "verification.md",
    "limitations.md",
    "design-principles.md",
    "motion-principles.md",
];
const SCHEMAS: [&str; 3] = [
    "icon.schema.json",
    "project.schema.json",
    "release.schema.json",
];
/// The dotLottie player version the packages target.
const PLAYER_VERSION: &str = "0.80.0";

/// Where `build_packages` writes and which source state it records.
#[derive(Debug, Clone, Default)]
pub struct BuildPackagesOptions {
    /// Folder holding `<package>/dist` and `<package>/LICENSE`; defaults to `<repo>/packages`.
    pub output: Option<PathBuf>,
    /// The commit and dirty state recorded in `release.json`; read from git when `None`.
    pub git: Option<GitState>,
}

/// What `build_packages` built.
#[derive(Debug, Clone)]
pub struct PackagesReport {
    /// Icon descriptors (and assets) in the icons package.
    pub icons: usize,
    /// The package version, from `packages/icons/package.json`.
    pub version: String,
    /// Whether the sources had uncommitted changes (`status` `prerelease-dev`).
    pub dirty: bool,
}

impl PackagesReport {
    /// The line `build-packages.py` printed.
    pub fn summary(&self) -> String {
        let state = if self.dirty {
            "uncommitted sources, status prerelease-dev"
        } else {
            "clean source tree, status release"
        };
        format!(
            "Built {} icon descriptors/assets and four packages ({}; {state}). Nothing was published.",
            self.icons, self.version
        )
    }
}

/// Files staged per package, by path relative to its `dist`.
type Stage = BTreeMap<&'static str, BTreeMap<String, Vec<u8>>>;

/// Builds all four packages' `dist` folders from the catalog and the package sources.
pub fn build_packages(repo: &Path, options: &BuildPackagesOptions) -> Result<PackagesReport> {
    let package_json = read_json(&repo.join("packages/icons/package.json"))?;
    let version = field(&package_json, "version", "packages/icons/package.json")?.clone();
    let catalog = Catalog::load(repo)?;
    let mut stage: Stage = PACKAGES.iter().map(|p| (*p, BTreeMap::new())).collect();
    let staged =
        |stage: &mut Stage, package: &'static str, name: String, source: &Path| -> Result<()> {
            stage
                .get_mut(package)
                .expect("known package")
                .insert(name, regular_file(source)?);
            Ok(())
        };

    for package in PACKAGES {
        for file in glob(&repo.join("packages").join(package).join("src"), "*")? {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if file.is_file() && (py_suffix(&name) == ".mjs" || name.ends_with(".d.ts")) {
                staged(&mut stage, package, name, &file)?;
            }
        }
        if package == "runtime" {
            // The tested gallery controller stays canonical during migration.
            staged(
                &mut stage,
                package,
                "controller.mjs".into(),
                &repo.join("prototype/icons/03/controller.mjs"),
            )?;
        }
    }

    let (mut descriptors, mut ids, mut inputs) = (Vec::new(), HashSet::new(), Map::new());
    for relative in &catalog.sets {
        if !is_set_manifest_path(relative) {
            return Err(invalid(format!("Invalid catalog path {relative}")));
        }
        let set = CatalogSet::load(repo, relative)?;
        let source_sha = sha256_hex(&set.bytes);
        inputs.insert(
            format!("prototype/icons/{relative}"),
            Value::String(source_sha.clone()),
        );
        let category = catalog
            .category_of(relative)
            .ok_or_else(|| invalid(format!("No catalog category lists {relative}")))?;
        let set_id = field(&set.record, "id", relative)?;
        let terms = search_terms(repo, set_id.as_str().unwrap_or_default(), &mut inputs)?;
        let mut set_ids = HashSet::new();
        for icon in set.icons()? {
            let name = field(icon, "name", relative)?;
            let identity = icon
                .get("icon")
                .unwrap_or(name)
                .as_str()
                .unwrap_or_default()
                .to_string();
            if !is_icon_id(&identity) || !ids.insert(identity.clone()) {
                return Err(invalid(format!("Invalid or duplicate ID {identity}")));
            }
            set_ids.insert(identity.clone());
            let context = format!("icon {identity}");
            let motions = array_field(icon, "motions", &context)?;
            let strong = motions
                .iter()
                .find(|m| m.get("motionId").and_then(Value::as_str) == Some("strong"))
                .ok_or_else(|| invalid(format!("Missing Strong motion: {identity}")))?;
            let strong_name = str_field(strong, "name", &context)?;
            let asset = set.folder().join(format!("{strong_name}.lottie"));
            if strong_name.contains(['/', '\\']) || is_symlink(&asset) {
                return Err(invalid("Invalid asset path"));
            }
            let data = read(&asset)?;
            let asset_sha = sha256_hex(&data);
            if strong.get("sha256").and_then(Value::as_str) != Some(asset_sha.as_str()) {
                return Err(invalid(format!("Stale asset: {identity}")));
            }
            let source = DescriptorSource {
                category_id: &category.id,
                category_label: &category.label,
                set_id,
                manifest: relative,
                manifest_sha256: &source_sha,
            };
            let mut descriptor = icon_descriptor(&identity, icon, &data, &asset, &source)?;
            if let Some(words) = terms.get(&identity) {
                descriptor["aliases"] = words.clone();
            }
            let text = py_compact(&descriptor);
            let icons = stage.get_mut("icons").expect("known package");
            icons.insert(format!("assets/{identity}.lottie"), data);
            icons.insert(
                format!("icons/{identity}.mjs"),
                format!("export default {text};\n").into_bytes(),
            );
            icons.insert(
                format!("icons/{identity}.d.ts"),
                format!("declare const icon: {text};\nexport default icon;\n").into_bytes(),
            );
            descriptors.push(descriptor);
            // An icon reanimated as a spec ships it, for `create --from` without a checkout.
            let spec = repo
                .join("motion/icon-sets")
                .join(set_id.as_str().unwrap_or_default())
                .join("specs")
                .join(format!("{identity}.icon.json"));
            if is_symlink(&spec) {
                return Err(invalid(format!("Invalid spec path: {identity}")));
            }
            if spec.is_file() {
                let bytes = read(&spec)?;
                stage
                    .get_mut("icons")
                    .expect("known package")
                    .insert(format!("specs/{identity}.icon.json"), bytes);
            }
        }
        if let Some(stray) = terms.keys().find(|id| !set_ids.contains(*id)) {
            return Err(invalid(format!("{}: {stray} is not an icon of this set", terms_path(set_id.as_str().unwrap_or_default()))));
        }
    }

    let categories: Vec<Value> = catalog
        .categories
        .iter()
        .map(|c| json!({ "id": c.id, "label": c.label }))
        .collect();
    let icon_count = descriptors.len();
    let package_catalog = json!({ "schemaVersion": 1, "version": version, "categories": categories, "icons": descriptors });
    stage.get_mut("icons").expect("known package").insert(
        "catalog.json".into(),
        (py_pretty(&package_catalog) + "\n").into_bytes(),
    );
    for name in DOCS {
        staged(
            &mut stage,
            "icons",
            format!("docs/{name}"),
            &repo.join("docs").join(name),
        )?;
    }
    staged(
        &mut stage,
        "icons",
        "docs/ai-operating-guide.md".into(),
        &repo.join("prototype/icons/guide/ai-operating-guide.md"),
    )?;
    for name in SCHEMAS {
        staged(
            &mut stage,
            "icons",
            format!("schemas/{name}"),
            &repo.join("schemas").join(name),
        )?;
    }
    for name in ["runtime.md", "authoring.md"] {
        staged(
            &mut stage,
            "cli",
            format!("skill/references/{name}"),
            &repo.join("packages/cli/skill/references").join(name),
        )?;
    }
    staged(
        &mut stage,
        "cli",
        "skill/SKILL.md".into(),
        &repo.join("packages/cli/skill/SKILL.md"),
    )?;

    let git = match &options.git {
        Some(git) => git.clone(),
        None => GitState::read(repo)?,
    };
    let icons = &stage["icons"];
    let mut listed: Vec<PathBuf> = icons.keys().map(PathBuf::from).collect();
    sort_paths(&mut listed);
    let files: Map<String, Value> = listed
        .iter()
        .map(|p| {
            let key = p.to_string_lossy().into_owned();
            let digest = sha256_hex(&icons[&key]);
            (key, Value::String(digest))
        })
        .collect();
    // A build from uncommitted sources is never a release; pack_release refuses it by default.
    let release = json!({
        "schemaVersion": 1,
        "version": version,
        "status": if git.dirty { "prerelease-dev" } else { "release" },
        "sourceCommit": git.commit,
        "sourceDirty": git.dirty,
        "runtimeVersion": version,
        "cliVersion": version,
        "playerVersion": PLAYER_VERSION,
        "exporter": { "requiredFor": "authoring-only", "distribution": "external-not-bundled" },
        "inputs": inputs,
        "files": files,
    });
    stage.get_mut("icons").expect("known package").insert(
        "release.json".into(),
        (py_pretty(&release) + "\n").into_bytes(),
    );

    let license = regular_file(&repo.join("LICENSE"))?;
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| repo.join("packages"));
    for (package, files) in &stage {
        let target = output.join(package);
        let dist = target.join("dist");
        if dist.exists() {
            std::fs::remove_dir_all(&dist).map_err(|e| PackageError::io(&dist, e))?;
        }
        std::fs::create_dir_all(&dist).map_err(|e| PackageError::io(&dist, e))?;
        for (name, bytes) in files {
            write(&dist.join(name), bytes)?;
        }
        write(&target.join("LICENSE"), &license)?;
    }
    let version = version
        .as_str()
        .map_or_else(|| version.to_string(), str::to_string);
    Ok(PackagesReport {
        icons: icon_count,
        version,
        dirty: git.dirty,
    })
}

/// What a descriptor records about an icon's origin besides its `set.json` entry.
#[derive(Debug, Clone, Copy)]
pub struct DescriptorSource<'a> {
    /// The catalog category id (`fintech-finance`).
    pub category_id: &'a str,
    /// The catalog category label.
    pub category_label: &'a str,
    /// The set's `set.json` `id`.
    pub set_id: &'a Value,
    /// `source.manifest`, for example `13/assets/set.json`.
    pub manifest: &'a str,
    /// sha256 of the manifest bytes.
    pub manifest_sha256: &'a str,
}

/// The package descriptor (schema 1) of one `set.json` icon (`build-packages.py:60-91`).
/// `asset` is its Strong `.lottie`, read from `asset_path`; the archive must bundle every
/// motion and reverse action, and the descriptor must pass [`validate_descriptor`].
pub fn icon_descriptor(
    identity: &str,
    icon: &Value,
    asset: &[u8],
    asset_path: &Path,
    source: &DescriptorSource<'_>,
) -> Result<Value> {
    let context = format!("icon {identity}");
    let actions: Vec<Value> = array_field(icon, "motions", &context)?
        .iter()
        .map(|m| action(m, &context))
        .collect::<Result<_>>()?;
    require_bundled_actions(asset, asset_path, &actions, identity)?;
    let descriptor = json!({
        "schemaVersion": 1,
        "id": identity,
        "name": identity,
        "label": field(icon, "label", &context)?,
        "intent": icon.get("intent").cloned().unwrap_or(json!("")),
        "category": source.category_label,
        "categoryId": source.category_id,
        "family": "generative-icons",
        "stateful": icon.get("stateful").is_some_and(py_truthy),
        "previewReset": icon.get("previewReset").is_some_and(py_truthy),
        "startState": icon.get("startState").cloned().unwrap_or(json!("rest")),
        "endState": icon.get("endState").cloned().unwrap_or(json!("rest")),
        "width": field(icon, "width", &context)?,
        "height": field(icon, "height", &context)?,
        "styles": field(icon, "styles", &context)?,
        "weights": field(field(icon, "strokeWidth", &context)?, "presets", &context)?,
        "motions": actions,
        "license": "MIT",
        "aliases": icon.get("keywords").cloned().unwrap_or(json!([])),
        "source": { "set": source.set_id, "manifest": source.manifest, "sha256": source.manifest_sha256 },
        "asset": { "path": format!("assets/{identity}.lottie"), "sha256": sha256_hex(asset), "bytes": asset.len() },
    });
    validate_descriptor(&descriptor)?;
    Ok(descriptor)
}

/// `motion/icon-sets/<set>/search-terms.json`, relative to the repository.
fn terms_path(set_id: &str) -> String {
    format!("motion/icon-sets/{set_id}/search-terms.json")
}

/// A set's search terms (`search-terms.json`): icon id to the words and phrases that should
/// find it, shipped as each descriptor's `aliases`. The file is a build input; a set
/// without one keeps its `keywords`.
fn search_terms(repo: &Path, set_id: &str, inputs: &mut Map<String, Value>) -> Result<Map<String, Value>> {
    let relative = terms_path(set_id);
    let path = repo.join(&relative);
    if is_symlink(&path) {
        return Err(invalid(format!("Invalid search terms path: {relative}")));
    }
    if !path.is_file() {
        return Ok(Map::new());
    }
    let bytes = read(&path)?;
    inputs.insert(relative.clone(), Value::String(sha256_hex(&bytes)));
    let Ok(Value::Object(terms)) = serde_json::from_slice::<Value>(&bytes) else {
        return Err(invalid(format!("{relative}: expected an object of icon id to terms")));
    };
    for (id, words) in &terms {
        let valid = words.as_array().is_some_and(|words| {
            (1..=MAX_TERMS).contains(&words.len()) && words.iter().all(|w| w.as_str().is_some_and(is_search_term))
        });
        if !valid {
            return Err(invalid(format!(
                "{relative}: {id} needs 1 to {MAX_TERMS} terms of lowercase letters, digits, single spaces and hyphens"
            )));
        }
    }
    Ok(terms)
}

/// The most search terms one icon carries.
const MAX_TERMS: usize = 24;

/// A search term: 1–40 characters of `[a-z0-9 -]`, starting and ending with a letter or digit,
/// with no doubled space.
fn is_search_term(term: &str) -> bool {
    let edge = |c: Option<char>| c.is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    (1..=40).contains(&term.len())
        && term.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' ' || c == '-')
        && edge(term.chars().next())
        && edge(term.chars().last())
        && !term.contains("  ")
}

/// A build input's bytes; symlinks and non-files are refused.
fn regular_file(path: &Path) -> Result<Vec<u8>> {
    if is_symlink(path) || !path.is_file() {
        return Err(invalid(format!(
            "Expected a regular build input: {}",
            path.display()
        )));
    }
    read(path)
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
}

/// `\d{2}/assets/set\.json`.
fn is_set_manifest_path(relative: &str) -> bool {
    let bytes = relative.as_bytes();
    bytes.len() > 2
        && bytes[..2].iter().all(u8::is_ascii_digit)
        && &relative[2..] == "/assets/set.json"
}

/// `[a-z][a-z0-9-]*`.
fn is_icon_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A descriptor motion (`build-packages.py:30-37`): `direction`, `startState` and `endState`
/// default first, then the timing keys that exist, then a truthy `reverse`.
fn action(value: &Value, context: &str) -> Result<Value> {
    let source = value
        .as_object()
        .ok_or_else(|| invalid(format!("{context}: motion is not an object")))?;
    let mut result = Map::new();
    for (key, default) in [
        ("direction", "forward"),
        ("startState", "rest"),
        ("endState", "rest"),
    ] {
        result.insert(key.into(), json!(default));
    }
    for key in [
        "name",
        "motionId",
        "fps",
        "frames",
        "active",
        "rest",
        "startState",
        "endState",
        "continuous",
        "direction",
    ] {
        if let Some(v) = source.get(key) {
            result.insert(key.into(), v.clone());
        }
    }
    if let Some(reverse) = source.get("reverse").filter(|r| py_truthy(r)) {
        result.insert("reverse".into(), action(reverse, context)?);
    }
    Ok(Value::Object(result))
}

/// The strong archive's manifest must list every motion and reverse action.
fn require_bundled_actions(
    archive: &[u8],
    path: &Path,
    actions: &[Value],
    identity: &str,
) -> Result<()> {
    let files = read_zip(archive).map_err(|e| PackageError::zip(path, e))?;
    let manifest_bytes = find_file(&files, "manifest.json")
        .ok_or_else(|| invalid(format!("{}: no manifest.json", path.display())))?;
    let manifest: Value =
        serde_json::from_slice(manifest_bytes).map_err(|source| PackageError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    let bundled: HashSet<&str> = array_field(&manifest, "animations", "manifest.json")?
        .iter()
        .filter_map(|a| a.get("id").and_then(Value::as_str))
        .collect();
    for motion in actions {
        let reverse = motion.get("reverse").into_iter();
        for action in std::iter::once(motion).chain(reverse) {
            let name = str_field(action, "name", identity)?;
            if !bundled.contains(name) {
                return Err(invalid(format!(
                    "Bundle lacks required actions: {identity}"
                )));
            }
        }
    }
    Ok(())
}

/// The runtime's `validateIcon` (`packages/runtime/src/index.mjs:12-22`), with JS truthiness.
pub fn validate_descriptor(icon: &Value) -> Result<()> {
    let id = icon.get("id").and_then(Value::as_str).unwrap_or_default();
    let fail = |message: &str| Err(invalid(format!("{message}: {id}")));
    if icon.get("schemaVersion").and_then(Value::as_f64) != Some(1.0) || !is_icon_id(id) {
        return fail("Unsupported icon descriptor");
    }
    let motions = icon
        .get("motions")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let has = |motion_id: &str| {
        motions
            .iter()
            .any(|m| m.get("motionId").and_then(Value::as_str) == Some(motion_id))
    };
    if motions.len() != 2 || !has("light") || !has("strong") {
        return fail("Light and Strong motions are required");
    }
    let number = |action: &Value, key: &str| {
        action
            .get(key)
            .and_then(Value::as_f64)
            .filter(|x| x.is_finite())
    };
    let stateful = truthy(icon.get("stateful"));
    for motion in motions {
        let reverse = motion.get("reverse").filter(|r| truthy(Some(r)));
        for action in std::iter::once(motion).chain(reverse) {
            let timing = (
                number(action, "fps"),
                number(action, "frames"),
                number(action, "rest"),
                number(action, "active"),
            );
            let valid = truthy(action.get("name"))
                && matches!(timing, (Some(fps), Some(frames), Some(rest), Some(active))
                    if fps > 0.0 && frames > 0.0 && rest >= 0.0 && rest < frames && active >= 0.0 && active * fps <= frames);
            if !valid {
                return fail("Invalid animation timing");
            }
        }
        if stateful {
            let same = |a: Option<&Value>, b: Option<&Value>| a == b;
            let ok = reverse.is_some()
                && same(motion.get("startState"), icon.get("startState"))
                && same(motion.get("endState"), icon.get("endState"))
                && same(
                    reverse.and_then(|r| r.get("endState")),
                    icon.get("startState"),
                );
            if !ok {
                return fail("Invalid state transition");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_key_order() {
        let motion = json!({"name": "a", "icon": "x", "motionId": "light", "direction": "forward", "fps": 60, "frames": 144, "rest": 60, "active": 1.2,
            "reverse": {"name": "a-reverse", "direction": "reverse", "fps": 60}});
        let result = action(&motion, "test").unwrap();
        assert_eq!(
            py_compact(&result),
            r#"{"direction": "forward", "startState": "rest", "endState": "rest", "name": "a", "motionId": "light", "fps": 60, "frames": 144, "active": 1.2, "rest": 60, "reverse": {"direction": "reverse", "startState": "rest", "endState": "rest", "name": "a-reverse", "fps": 60}}"#
        );
    }

    #[test]
    fn manifest_paths_and_ids() {
        assert!(is_set_manifest_path("13/assets/set.json"));
        assert!(!is_set_manifest_path("1/assets/set.json"));
        assert!(!is_set_manifest_path("13/assets/set.jsonx"));
        assert!(is_icon_id("arrow-up-2"));
        assert!(!is_icon_id("2up") && !is_icon_id("Up") && !is_icon_id(""));
    }

    #[test]
    fn descriptor_validation() {
        let timing = json!({"fps": 60, "frames": 144, "rest": 60, "active": 1.2});
        let motion = |id: &str| {
            let mut m = timing.clone();
            m["name"] = json!(id);
            m["motionId"] = json!(id);
            m
        };
        let mut icon =
            json!({"schemaVersion": 1, "id": "ok", "motions": [motion("light"), motion("strong")]});
        assert!(validate_descriptor(&icon).is_ok());
        icon["motions"][0]["rest"] = json!(144);
        assert!(validate_descriptor(&icon).is_err());
        icon["motions"][0]["rest"] = json!(0);
        icon["stateful"] = json!(true);
        assert!(
            validate_descriptor(&icon).is_err(),
            "stateful icons need reverse actions"
        );
    }
}
