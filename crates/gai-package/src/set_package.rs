//! `package_set`: the port of `library/package-set.py` together with each set's own
//! `package.py` (legacy packagers 01–06, the 11/13–15 `set.json` patches and the extras).
//!
//! Unlike the original, every output is computed before anything is written, so a failure
//! in the zip or extras stage leaves `set.json`, `build.json` and the archives unchanged.

use crate::bundle::{SetZip, build_set_zip, legacy, zip_name};
use crate::config::{SetConfig, SetPaths};
use crate::error::{PackageError, Result, invalid};
use crate::io::{field, object, read_json, str_field, write};
use crate::profile::{Profile, profile};
use crate::record::{ActionList, build_set_archives, encode_set_json, record_actions};
use crate::zip::ZipDate;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// What `package_set` wrote.
#[derive(Debug, Clone)]
pub struct PackageReport {
    /// The set id.
    pub set_id: String,
    /// One entry per `.lottie` written, in `set.json` order.
    pub archives: Vec<ArchiveSummary>,
    /// Evidence images copied beside the assets and added to the set zip.
    pub verified_images: usize,
    /// The set zip.
    pub zip: PathBuf,
    /// Size of the set zip in bytes.
    pub zip_bytes: u64,
}

/// A written `.lottie`.
#[derive(Debug, Clone)]
pub struct ArchiveSummary {
    /// The action name.
    pub name: String,
    /// Archive size in bytes.
    pub bytes: u64,
    /// Archive sha256 (lowercase hex).
    pub sha256: String,
    /// Whether the set lists its actions under `motions` (the frozen sets 01 and 02).
    legacy_motions: bool,
}

impl fmt::Display for ArchiveSummary {
    /// The line the Python packager printed, e.g. `bank.lottie: 12,345 bytes; all actions preserved`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let note = if self.legacy_motions {
            "both motion exports preserved"
        } else {
            "all actions preserved"
        };
        write!(
            f,
            "{}.lottie: {} bytes; {note}",
            self.name,
            thousands(self.bytes)
        )
    }
}

fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Packages one built set: rewrites every `<name>.manifest.json`, writes every `.lottie`,
/// adds `bytes`/`sha256` to `set.json` and writes it (Python `json.dumps` format) to both
/// `set.json` and `evidence/build.json`, copies verified evidence images, and writes the
/// set zip with its source bundle and extras.
///
/// Inputs are the build outputs in `paths.assets` (`<name>.json`, `<name>.manifest.json`,
/// `<name>.theme-{dark,light}.json`, `set.json`) and the set's `motion/icon-sets` folder.
pub fn package_set(paths: &SetPaths) -> Result<PackageReport> {
    let legacy = legacy(paths.id());
    let config = match legacy {
        Some(_) => None,
        None => {
            let config = SetConfig::load(&paths.root)?;
            if config.id != paths.id() {
                return Err(invalid(format!(
                    "set-config.json id {} does not match the folder {}",
                    config.id,
                    paths.id()
                )));
            }
            Some(config)
        }
    };
    let profile = match &config {
        Some(config) => profile(paths, config)?,
        None => Profile::default(),
    };
    let (list, date) = legacy.map_or((ActionList::Animations, ZipDate::SETS), |l| {
        (l.actions, l.date)
    });

    let mut record = read_json(&paths.assets.join("set.json"))?;
    let top = object(&mut record, "set.json")?;
    for (key, value) in &profile.patches {
        top.insert((*key).to_string(), Value::String(value.clone()));
    }
    let archives = build_set_archives(&mut record, &paths.assets, list, date)?;
    let set_json = encode_set_json(&record);

    let mut actions = Vec::new();
    let mut hashes = Vec::new();
    for item in record_actions(&record, list)? {
        actions.push(str_field(item, "name", "action")?.to_string());
        hashes.push(str_field(item, "sha256", "action")?.to_string());
    }
    let images = verified_images(paths, &actions, &hashes)?;
    let lotties: HashMap<String, Vec<u8>> = archives
        .iter()
        .map(|a| (a.name.clone(), a.lottie.clone()))
        .collect();
    let zip = SetZip {
        paths,
        config: config.as_ref(),
        legacy,
        actions: &actions,
        lotties: &lotties,
        set_json: &set_json,
        images: &images,
        extras: &profile.extras,
    };
    let zip_bytes = build_set_zip(&zip)?;
    let zip_path = paths.assets.join(zip_name(&zip));

    for archive in &archives {
        write(
            &paths.assets.join(format!("{}.manifest.json", archive.name)),
            &archive.manifest,
        )?;
        write(
            &paths.assets.join(format!("{}.lottie", archive.name)),
            &archive.lottie,
        )?;
    }
    write(&paths.assets.join("set.json"), &set_json)?;
    write(&paths.root.join("evidence/build.json"), &set_json)?;
    let evidence = paths.root.join("evidence");
    for image in &images {
        let (from, to) = (evidence.join(image), paths.assets.join(image));
        std::fs::copy(&from, &to).map_err(|e| PackageError::io(&from, e))?;
    }
    write(&zip_path, &zip_bytes)?;

    let legacy_motions = list == ActionList::Motions;
    Ok(PackageReport {
        set_id: paths.id().to_string(),
        archives: archives
            .iter()
            .map(|a| ArchiveSummary {
                name: a.name.clone(),
                bytes: a.lottie.len() as u64,
                sha256: a.sha256.clone(),
                legacy_motions,
            })
            .collect(),
        verified_images: images.len(),
        zip: zip_path,
        zip_bytes: zip_bytes.len() as u64,
    })
}

/// `package-set.py:58-69`: when `evidence/browser-verification.json` passed for exactly the
/// new archive hashes, the `<action>-{rest,poses,styles}.png` sheets that exist.
fn verified_images(paths: &SetPaths, actions: &[String], hashes: &[String]) -> Result<Vec<String>> {
    let evidence = paths.root.join("evidence");
    let proof_path = evidence.join("browser-verification.json");
    if !proof_path.exists() {
        return Ok(Vec::new());
    }
    let proof = read_json(&proof_path)?;
    let mut proven = HashMap::new();
    if let Some(assets) = proof.get("assets") {
        for asset in assets
            .as_array()
            .ok_or_else(|| invalid("browser-verification.json: assets is not an array"))?
        {
            let context = "browser-verification.json asset";
            proven.insert(
                str_field(asset, "name", context)?.to_string(),
                field(asset, "sha256", context)?.clone(),
            );
        }
    }
    let passed = proof.get("result").and_then(Value::as_str) == Some("PASS")
        && actions
            .iter()
            .zip(hashes)
            .all(|(name, sha)| proven.get(name).and_then(Value::as_str) == Some(sha.as_str()));
    if !passed {
        return Ok(Vec::new());
    }
    let mut images = Vec::new();
    for name in actions {
        for suffix in ["rest", "poses", "styles"] {
            let image = format!("{name}-{suffix}.png");
            if evidence.join(&image).exists() {
                images.push(image);
            }
        }
    }
    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separator() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(51244197), "51,244,197");
    }

    #[test]
    fn summary_line() {
        let summary = ArchiveSummary {
            name: "bank".into(),
            bytes: 12345,
            sha256: String::new(),
            legacy_motions: false,
        };
        assert_eq!(
            summary.to_string(),
            "bank.lottie: 12,345 bytes; all actions preserved"
        );
    }

    /// A one-icon library with a passing public-polish review and a config extra.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let files = [
            ("LICENSE", "MIT"),
            ("THIRD_PARTY_NOTICES.md", "none"),
            ("docs/extra.md", "extra"),
            (
                "motion/icon-sets/70-test/set-config.json",
                r#"{"id":"70-test","name":"Test","designReview":"70-test/evidence","packageExtras":["docs/extra.md"]}"#,
            ),
            ("motion/icon-sets/70-test/DOWNLOAD_README.txt", "readme\n"),
            ("motion/icon-sets/70-test/evidence/visual-review.md", "ok"),
            (
                "motion/icon-sets/library/design-reviews/public-motion-polish/final-review.json",
                r#"{"result":"PASS","icons":[{"set":"70-test","claudeFirstDesign":{"file":"reviews/dot/proposal.md"}}]}"#,
            ),
            (
                "motion/icon-sets/library/design-reviews/public-motion-polish/README.md",
                "polish",
            ),
            ("reviews/dot/proposal.md", "proposal"),
            ("reviews/dot/supervisor-review.md", "review"),
            ("reviews/dot/claude-invocation.json", "{}"),
            (
                "prototype/icons/70/assets/set.json",
                r#"{"name":"Test","id":"70-test","icons":[{"name":"dot","motions":[{"name":"dot"},{"name":"dot-strong"}],"animations":[{"name":"dot"},{"name":"dot-strong"}]}]}"#,
            ),
        ];
        for (path, text) in files {
            write(&dir.path().join(path), text).unwrap();
        }
        let manifest = r#"{"version":"2","generator":"g","initial":{"animation":"x"},"animations":[],"themes":[{"id":"dark"},{"id":"light"}]}"#;
        for name in ["dot", "dot-strong"] {
            let assets = dir.path().join("prototype/icons/70/assets");
            write(
                &assets.join(format!("{name}.json")),
                format!("{{\"nm\":\"{name}\"}}"),
            )
            .unwrap();
            write(&assets.join(format!("{name}.manifest.json")), manifest).unwrap();
            for theme in ["dark", "light"] {
                write(&assets.join(format!("{name}.theme-{theme}.json")), "{}").unwrap();
            }
        }
        dir
    }

    #[test]
    fn packages_a_set() {
        let dir = fixture();
        let paths = SetPaths::new(dir.path(), "70-test").unwrap();
        let report = package_set(&paths).unwrap();
        assert_eq!(report.archives.len(), 2);

        let set_json = std::fs::read_to_string(paths.assets.join("set.json")).unwrap();
        assert_eq!(
            set_json,
            std::fs::read_to_string(paths.root.join("evidence/build.json")).unwrap()
        );
        let record: Value = serde_json::from_str(&set_json).unwrap();
        let icon = &record["icons"][0];
        let keys: Vec<&String> = icon.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["name", "motions", "animations", "bytes", "sha256"]);
        assert_eq!(icon["sha256"], icon["motions"][0]["sha256"]);
        assert_eq!(
            icon["motions"][1]["sha256"],
            report.archives[1].sha256.as_str()
        );

        let files = crate::zip::read_zip(&std::fs::read(&report.zip).unwrap()).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        let polish = "source/motion/icon-sets/library/design-reviews/public-motion-polish";
        assert_eq!(
            names,
            [
                "dot.lottie",
                "dot.json",
                "dot-strong.lottie",
                "dot-strong.json",
                "set.json",
                "README.txt",
                "LICENSE",
                "THIRD_PARTY_NOTICES.md",
                "source/motion/icon-sets/70-test/set-config.json",
                "source/motion/icon-sets/70-test/DOWNLOAD_README.txt",
                "source/LICENSE",
                "source/THIRD_PARTY_NOTICES.md",
                "source/motion/icon-sets/70-test/evidence/visual-review.md",
                "source/REBUILD.txt",
                &format!("{polish}/final-review.json"),
                &format!("{polish}/README.md"),
                "source/reviews/dot/claude-invocation.json",
                "source/reviews/dot/proposal.md",
                "source/reviews/dot/supervisor-review.md",
                "source/docs/extra.md",
            ]
        );
    }
}
