//! The downloadable set zip: every action's `.lottie` and `.json`, `set.json`, the download
//! README, verified evidence images and a `source/` bundle of the authoring files
//! (`package-set.py:71-146`), plus the six legacy layouts of sets 01–06 and the extras of
//! each set's own `package.py`.
//!
//! Listings are sorted where the originals used filesystem order (01–06 dependencies), so
//! those zips can differ from a Python run in the order of such entries only.

use crate::config::{SetConfig, SetPaths};
use crate::error::{PackageError, Result, invalid};
use crate::io::{py_truthy, read, read_json};
use crate::paths::{dedupe_paths, glob, relative, rglob, with_suffix};
use crate::profile::Extras;
use crate::record::ActionList;
use crate::zip::{ZipDate, ZipEntry, write_zip};
use gai_core::parallel::parallel_map;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A legacy set (01–06) whose own packager used a different zip layout.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Legacy {
    pub zip_name: &'static str,
    pub date: ZipDate,
    pub actions: ActionList,
    layout: Layout,
}

#[derive(Debug, Clone, Copy)]
enum Layout {
    /// 01, 02: no notices, no source list and no REBUILD.txt.
    Prototype,
    /// 03: a fixed source list read without existence checks.
    Essentials,
    /// 04, 05: existing sources, without deduplication.
    Catalog {
        evidence: &'static [&'static str],
        rebuild: &'static str,
    },
    /// 06: as 04 plus the keyframes review and shared tools.
    Tools,
}

/// The legacy layout of `set_id`, if it is one of the sets 01–06.
pub(crate) fn legacy(set_id: &str) -> Option<Legacy> {
    const ACTIONS_EVIDENCE: [&str; 4] = [
        "geometry.json",
        "browser-verification.json",
        "gallery-responsive.json",
        "gallery-controls.json",
    ];
    const INTERFACE_EVIDENCE: [&str; 8] = [
        "geometry.json",
        "browser-verification.json",
        "gallery-responsive.json",
        "gallery-controls.json",
        "native-rotation-pixel-comparison.json",
        "gallery-dark.jpg",
        "gallery-small-light.jpg",
        "family-poses.jpg",
    ];
    let (zip_name, layout) = match set_id {
        "01-heart-bell" => ("heart-and-bell.zip", Layout::Prototype),
        "02-search" => ("search.zip", Layout::Prototype),
        "03-essentials" => ("five-icons.zip", Layout::Essentials),
        "04-actions" => (
            "ten-icons.zip",
            Layout::Catalog {
                evidence: &ACTIONS_EVIDENCE,
                rebuild: "actions",
            },
        ),
        "05-interface" => (
            "twenty-icons.zip",
            Layout::Catalog {
                evidence: &INTERFACE_EVIDENCE,
                rebuild: "interface",
            },
        ),
        "06-tools" => ("twenty-icons.zip", Layout::Tools),
        _ => return None,
    };
    let actions = if matches!(layout, Layout::Prototype) {
        ActionList::Motions
    } else {
        ActionList::Animations
    };
    let date = if set_id == "06-tools" {
        ZipDate::SETS
    } else {
        ZipDate::LEGACY_SETS
    };
    Some(Legacy {
        zip_name,
        date,
        actions,
        layout,
    })
}

/// What the set zip is assembled from.
pub(crate) struct SetZip<'a> {
    pub paths: &'a SetPaths,
    /// `None` for the legacy sets, which have no `set-config.json`.
    pub config: Option<&'a SetConfig>,
    pub legacy: Option<Legacy>,
    /// Every action of the set, in record order.
    pub actions: &'a [String],
    /// The newest archive bytes per action name.
    pub lotties: &'a HashMap<String, Vec<u8>>,
    /// The final `set.json` text.
    pub set_json: &'a str,
    /// Verified evidence image names, copied from `evidence/`.
    pub images: &'a [String],
    pub extras: &'a [Extras],
}

/// Entries collected in write order, compressed together at the end.
struct Plan<'a> {
    repo: &'a Path,
    entries: Vec<(String, Vec<u8>, ZipDate)>,
    names: HashSet<String>,
}

impl Plan<'_> {
    fn push(&mut self, name: String, data: Vec<u8>, date: ZipDate) {
        self.names.insert(name.clone());
        self.entries.push((name, data, date));
    }

    fn source_name(&self, path: &Path) -> Result<String> {
        Ok(format!("source/{}", relative(path, self.repo)?))
    }

    /// Adds `path` as `source/<repo-relative path>`.
    fn push_source(&mut self, path: &Path, date: ZipDate) -> Result<()> {
        let name = self.source_name(path)?;
        self.push(name, read(path)?, date);
        Ok(())
    }

    /// Adds each existing path.
    fn push_existing(&mut self, paths: &[PathBuf], date: ZipDate) -> Result<()> {
        for path in paths.iter().filter(|p| p.exists()) {
            self.push_source(path, date)?;
        }
        Ok(())
    }

    /// Adds each existing path whose name is not in the zip yet.
    fn push_new(&mut self, paths: &[PathBuf], date: ZipDate) -> Result<()> {
        for path in paths.iter().filter(|p| p.exists()) {
            let name = self.source_name(path)?;
            if !self.names.contains(&name) {
                self.push(name, read(path)?, date);
            }
        }
        Ok(())
    }

    fn finish(self) -> std::result::Result<Vec<u8>, crate::zip::ZipError> {
        let entries = parallel_map(self.entries, |(name, data, date)| {
            ZipEntry::deflate(name, &data, date)
        });
        write_zip(&entries)
    }
}

/// The zip's file name in the asset folder (`<set-id>.zip`, or a legacy name).
pub(crate) fn zip_name(zip: &SetZip) -> String {
    zip.legacy.map_or_else(
        || format!("{}.zip", zip.paths.id()),
        |l| l.zip_name.to_string(),
    )
}

/// Assembles the complete set zip.
pub(crate) fn build_set_zip(zip: &SetZip) -> Result<Vec<u8>> {
    let paths = zip.paths;
    let date = zip.legacy.map_or(ZipDate::SETS, |l| l.date);
    let mut plan = Plan {
        repo: &paths.repo,
        entries: Vec::new(),
        names: HashSet::new(),
    };
    for name in zip.actions {
        let lottie = zip
            .lotties
            .get(name)
            .ok_or_else(|| invalid(format!("no archive for {name}")))?;
        plan.push(format!("{name}.lottie"), lottie.clone(), date);
        plan.push(
            format!("{name}.json"),
            read(&paths.assets.join(format!("{name}.json")))?,
            date,
        );
    }
    plan.push("set.json".into(), zip.set_json.as_bytes().to_vec(), date);
    plan.push(
        "README.txt".into(),
        read(&paths.root.join("DOWNLOAD_README.txt"))?,
        date,
    );
    if zip.legacy.is_none() {
        for notice in ["LICENSE", "THIRD_PARTY_NOTICES.md"] {
            plan.push(notice.into(), read(&paths.repo.join(notice))?, date);
        }
    }
    for image in zip.images {
        plan.push(
            image.clone(),
            read(&paths.root.join("evidence").join(image))?,
            date,
        );
    }

    match zip.legacy.map(|l| l.layout) {
        None => {
            let config = zip
                .config
                .ok_or_else(|| invalid(format!("{}: set-config.json is required", paths.id())))?;
            plan.push_existing(&modern_sources(paths, config)?, date)?;
            let id = paths.id();
            let rebuild = format!(
                "Install source/motion dependencies with npm install. The Engine exporter is required; set CREATOR_MCP_BIN to its absolute executable path. From source/, run node motion/icon-sets/{id}/build.mjs and node motion/icon-sets/{id}/verify.mjs. Native scenes and paths remain editable. The complete gallery also requires the other catalog asset folders. The Engine exporter is an external prerequisite.\n"
            );
            plan.push("source/REBUILD.txt".into(), rebuild.into_bytes(), date);
            let passed = public_polish(&mut plan, paths, date)?;
            if passed {
                review_sources(&mut plan, paths, date)?;
            }
        }
        Some(Layout::Prototype) => legacy_tail(&mut plan, paths, date)?,
        Some(Layout::Essentials) => {
            for path in essentials_sources(paths)? {
                plan.push_source(&path, date)?;
            }
            plan.push(
                "source/REBUILD.txt".into(),
                rebuild_text("essentials").into_bytes(),
                date,
            );
            legacy_tail(&mut plan, paths, date)?;
        }
        Some(Layout::Catalog { evidence, rebuild }) => {
            let evidence = if zip.images.is_empty() {
                &[][..]
            } else {
                evidence
            };
            plan.push_existing(&catalog_sources(paths, evidence)?, date)?;
            plan.push(
                "source/REBUILD.txt".into(),
                rebuild_text(rebuild).into_bytes(),
                date,
            );
            legacy_tail(&mut plan, paths, date)?;
        }
        Some(Layout::Tools) => {
            plan.push_existing(&tools_sources(paths, !zip.images.is_empty())?, date)?;
            plan.push(
                "source/REBUILD.txt".into(),
                rebuild_text("tools").into_bytes(),
                date,
            );
            legacy_tail(&mut plan, paths, date)?;
        }
    }

    for extras in zip.extras {
        for extra in &extras.files {
            let name = plan.source_name(&extra.path)?;
            if extras.skip_present && plan.names.contains(&name) {
                continue;
            }
            if !extra.required && !extra.path.exists() {
                continue;
            }
            plan.push(name, read(&extra.path)?, extras.date);
        }
    }
    let target = paths.assets.join(zip_name(zip));
    plan.finish().map_err(|e| PackageError::zip(&target, e))
}

fn joined(base: &Path, names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|n| base.join(n)).collect()
}

/// `package-set.py:84-108`: the authoring sources of a current set, first occurrence kept.
fn modern_sources(paths: &SetPaths, config: &SetConfig) -> Result<Vec<PathBuf>> {
    let (repo, root, library) = (&paths.repo, &paths.root, paths.library());
    let icons = repo.join("prototype/icons");
    let skill = repo.join(".agents/skills/animated-icon-workflow");
    let mut sources = joined(
        root,
        &[
            "definitions.mjs",
            "build.mjs",
            "verify.mjs",
            "package.py",
            "set-config.json",
            "README.md",
            "DOWNLOAD_README.txt",
        ],
    );
    sources.extend(glob(&root.join("batches"), "*.mjs")?);
    sources.push(root.join("batches/.gitkeep"));
    sources.extend(glob(root, "verify-*.mjs")?);
    sources.extend(glob(&root.join("scenes"), "*")?);
    sources.extend(glob(&root.join("native"), "*.json")?);
    sources.extend(joined(
        &library,
        &[
            "drawing.mjs",
            "direction.mjs",
            "build-set.mjs",
            "package-set.py",
            "verify-directed.mjs",
            "verify-paint.mjs",
            "verify-filled-ellipsis.mjs",
        ],
    ));
    sources.extend(config.shared_sources.iter().map(|n| library.join(n)));
    sources.extend(joined(
        &repo.join("motion/src"),
        &["export.mjs", "exporter-config.mjs", "engine-scene.js"],
    ));
    sources.extend(joined(
        &repo.join("motion"),
        &["package.json", "package-lock.json"],
    ));
    sources.extend(joined(repo, &["LICENSE", "THIRD_PARTY_NOTICES.md"]));
    sources.push(icons.join("catalog.json"));
    sources.extend(joined(
        &icons.join("01"),
        &["app.js", "index.html", "styles.css", "catalog.mjs"],
    ));
    sources.push(icons.join("03/controller.mjs"));
    sources.push(icons.join(paths.number()).join("verify.html"));
    sources.extend(rglob(&skill, "*.md")?);
    sources.extend(glob(&skill.join("scripts"), "*.py")?);
    let default_review = format!("library/design-reviews/ai-category/{}", config.id);
    let review = repo
        .join("motion/icon-sets")
        .join(config.design_review.as_deref().unwrap_or(&default_review));
    sources.extend(rglob(&review, "*.md")?);
    sources.extend(rglob(&review, "invocation.json")?);
    sources.extend(joined(
        repo,
        &[
            "docs/design-principles.md",
            "docs/creating-icons.md",
            "docs/motion-principles.md",
        ],
    ));
    sources.extend(joined(
        &root.join("evidence"),
        &[
            "directed-source-verification.json",
            "browser-verification.json",
            "clearance-verification.json",
            "visual-review.md",
        ],
    ));
    Ok(dedupe_paths(sources))
}

/// Gallery files every legacy source list shares.
fn gallery_sources(repo: &Path) -> Vec<PathBuf> {
    let icons = repo.join("prototype/icons");
    let mut sources = vec![icons.join("catalog.json")];
    sources.extend(joined(
        &icons.join("01"),
        &["app.js", "index.html", "styles.css", "catalog.mjs"],
    ));
    sources.push(repo.join("motion/icon-sets/library/controller.test.mjs"));
    sources.extend(joined(
        &icons.join("03"),
        &["controller.mjs", "app.js", "index.html", "styles.css"],
    ));
    sources
}

fn legacy_head(paths: &SetPaths, last: &str, natives: &str) -> Result<Vec<PathBuf>> {
    let root = &paths.root;
    let mut sources = joined(
        root,
        &[
            "definitions.mjs",
            "build.mjs",
            "verify.mjs",
            "package.py",
            "README.md",
            "DOWNLOAD_README.txt",
            last,
        ],
    );
    sources.extend(glob(&root.join("scenes"), "*")?);
    sources.extend(glob(&root.join("native"), natives)?);
    sources.extend(joined(
        &paths.repo.join("motion/src"),
        &["export.mjs", "engine-scene.js"],
    ));
    sources.push(paths.repo.join("motion/package.json"));
    Ok(sources)
}

/// Set 03 (`03/package.py:81-90`).
fn essentials_sources(paths: &SetPaths) -> Result<Vec<PathBuf>> {
    let mut sources = legacy_head(paths, "controller.test.mjs", "*.provenance.json")?;
    sources.extend(gallery_sources(&paths.repo));
    Ok(sources)
}

/// Sets 04 and 05 (`04/package.py:81-93`); the docs repeat as in the original.
fn catalog_sources(paths: &SetPaths, evidence: &[&str]) -> Result<Vec<PathBuf>> {
    let mut sources = legacy_head(paths, "BRIEFS.md", "*.json")?;
    sources.extend(gallery_sources(&paths.repo));
    sources.push(
        paths
            .repo
            .join("prototype/icons")
            .join(paths.number())
            .join("verify.html"),
    );
    sources.extend(joined(&paths.root.join("evidence"), evidence));
    let docs = [
        "docs/design-principles.md",
        "docs/creating-icons.md",
        "docs/motion-principles.md",
        "docs/motion-principles.md",
        "docs/design-principles.md",
    ];
    sources.extend(joined(&paths.repo, &docs));
    Ok(sources)
}

/// Set 06 (`06/package.py:81-102`).
fn tools_sources(paths: &SetPaths, verified: bool) -> Result<Vec<PathBuf>> {
    let (repo, library) = (&paths.repo, paths.library());
    let keyframes = library.join("design-reviews/keyframes");
    let skill = repo.join(".agents/skills/animated-icon-workflow");
    let mut sources = legacy_head(paths, "BRIEFS.md", "*.json")?;
    sources.extend(joined(
        &library,
        &[
            "drawing.mjs",
            "direction.mjs",
            "directed-tools.mjs",
            "keyframes.mjs",
            "verify-directed.mjs",
        ],
    ));
    sources.extend(rglob(&keyframes, "*.md")?);
    sources.extend(glob(&keyframes, "proposal-*/invocation.json")?);
    sources.extend(joined(
        &keyframes,
        &[
            "delivery-verification.json",
            "solid-preserved-paints.json",
            "gallery-controls.json",
            "validation.json",
        ],
    ));
    sources.extend(rglob(&skill, "*.md")?);
    sources.extend(glob(&skill.join("scripts"), "*.py")?);
    sources.extend(gallery_sources(repo));
    sources.push(repo.join("prototype/icons/06/verify.html"));
    if verified {
        let evidence = [
            "geometry.json",
            "browser-verification.json",
            "browser-verification-gift-connected.json",
            "gift-unchanged-assets.json",
            "gift-gallery.jpg",
            "gallery-responsive.json",
            "gallery-controls.json",
            "original-native-hashes.json",
            "gallery-dark.jpg",
            "gallery-small-light.jpg",
            "family-poses.jpg",
        ];
        sources.extend(joined(&paths.root.join("evidence"), &evidence));
    }
    let docs = [
        "docs/design-principles.md",
        "docs/creating-icons.md",
        "docs/motion-principles.md",
        "docs/motion-principles.md",
        "docs/design-principles.md",
        "docs/motion-principles.md",
    ];
    sources.extend(joined(repo, &docs));
    sources.push(library.join("motion-directions.json"));
    Ok(sources)
}

fn rebuild_text(task: &str) -> String {
    let preview = if task == "essentials" {
        "the Heart, Bell, and Search asset folders"
    } else {
        "the existing icon asset folders"
    };
    format!(
        "Install dependencies in source/motion with npm install. Set CREATOR_MCP_BIN to the Engine exporter executable. Run npm run build:{task} and npm run verify:{task}. The Engine exporter is an external prerequisite, not included in this archive. The cumulative preview also needs {preview} from the project. Generated scene files are independently editable Engine authoring data.\n"
    )
}

/// The part every legacy packager ends with: public polish, the shared authoring
/// dependencies (names not yet present), then review sources.
fn legacy_tail(plan: &mut Plan, paths: &SetPaths, date: ZipDate) -> Result<()> {
    let passed = public_polish(plan, paths, date)?;
    let (repo, root, library) = (&paths.repo, &paths.root, paths.library());
    let skill = repo.join(".agents/skills/animated-icon-workflow");
    let mut sources = glob(&library, "*.mjs")?;
    sources.push(library.join("motion-directions.json"));
    sources.extend(joined(
        root,
        &[
            "build.mjs",
            "definitions.mjs",
            "verify.mjs",
            "package.py",
            "README.md",
        ],
    ));
    sources.extend(glob(&root.join("scenes"), "*.json")?);
    sources.extend(glob(&root.join("native"), "*.json")?);
    sources.extend(joined(
        &repo.join("motion/src"),
        &["export.mjs", "engine-scene.js"],
    ));
    sources.push(repo.join("motion/package.json"));
    sources.extend(rglob(&skill, "*.md")?);
    sources.extend(glob(&skill.join("scripts"), "*.py")?);
    sources.push(repo.join("docs/design-principles.md"));
    sources.push(root.join("evidence/browser-verification.json"));
    plan.push_new(&sources, date)?;
    if passed {
        review_sources(plan, paths, date)?;
    }
    Ok(())
}

fn public_polish_dir(paths: &SetPaths) -> PathBuf {
    paths.library().join("design-reviews/public-motion-polish")
}

/// The supervised motion review, added only while its `final-review.json` says PASS.
/// Returns whether it passed. (The folder is absent from the public repository.)
fn public_polish(plan: &mut Plan, paths: &SetPaths, date: ZipDate) -> Result<bool> {
    let polish = public_polish_dir(paths);
    let review = polish.join("final-review.json");
    if !review.exists() || read_json(&review)?.get("result").and_then(Value::as_str) != Some("PASS")
    {
        return Ok(false);
    }
    let names = [
        "final-review.json",
        "final-review.md",
        "visual-observations.json",
        "light-integrity-screening.json",
        "current-motion-audit.json",
        "traversal-screening.json",
        "traversal-screen-review.json",
        "light-timing-integrity.json",
        "clock-sheet-coverage.json",
        "clock-capture-method.json",
        "pause-screen-review.json",
        "README.md",
    ];
    plan.push_existing(&joined(&polish, &names), date)?;
    Ok(true)
}

/// Design, visual, timing and secondary review files that `final-review.json` lists for this set.
fn review_sources(plan: &mut Plan, paths: &SetPaths, date: ZipDate) -> Result<()> {
    let review = read_json(&public_polish_dir(paths).join("final-review.json"))?;
    let repo = &paths.repo;
    let mut sources = BTreeSet::new();
    let text = |value: &Value, key: &str| -> Result<String> {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| invalid(format!("final-review.json: missing \"{key}\"")))
    };
    let parent = |path: &Path| path.parent().map(Path::to_path_buf).unwrap_or_default();
    for item in review
        .get("icons")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("final-review.json: missing \"icons\""))?
    {
        if text(item, "set")? != paths.id() {
            continue;
        }
        let proposal = repo.join(text(
            item.get("claudeFirstDesign").unwrap_or(&Value::Null),
            "file",
        )?);
        sources.insert(parent(&proposal).join("supervisor-review.md"));
        sources.extend(glob(&parent(&proposal), "*invocation*.json")?);
        sources.insert(proposal);
        if item.get("finalVisualReview").is_some_and(py_truthy) {
            let visual = repo.join(text(item, "finalVisualReview")?);
            let folder = parent(&visual);
            sources.extend(joined(
                &folder,
                &[
                    "proposal.json",
                    "accepted-replacements.json",
                    "visual-revision.md",
                ],
            ));
            sources.extend(glob(&folder, "*invocation*.json")?);
            sources.insert(visual);
        }
        if item.get("timingReview").is_some_and(py_truthy) {
            let timing = repo.join(text(&item["timingReview"], "file")?);
            sources.insert(with_suffix(&timing, ".md"));
            sources.insert(parent(&parent(&timing)).join("supervisor-review.md"));
            sources.extend(glob(&parent(&timing), "*invocation*.json")?);
            sources.insert(timing);
        }
        if item.get("secondaryReview").is_some_and(py_truthy) {
            let secondary = repo.join(text(item, "secondaryReview")?);
            sources.insert(parent(&secondary).join("proposal.json"));
            sources.extend(glob(&parent(&secondary), "*invocation*.json")?);
            sources.insert(secondary);
        }
    }
    // A BTreeSet of paths iterates in pathlib's sorted() order.
    plan.push_new(&sources.into_iter().collect::<Vec<_>>(), date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_table() {
        let tools = legacy("06-tools").unwrap();
        assert_eq!(
            (tools.zip_name, tools.date, tools.actions),
            ("twenty-icons.zip", ZipDate::SETS, ActionList::Animations)
        );
        let bell = legacy("01-heart-bell").unwrap();
        assert_eq!(
            (bell.zip_name, bell.date, bell.actions),
            (
                "heart-and-bell.zip",
                ZipDate::LEGACY_SETS,
                ActionList::Motions
            )
        );
        assert!(legacy("07-ai-conversation").is_none());
    }

    #[test]
    fn rebuild_texts() {
        assert!(rebuild_text("essentials").contains("build:essentials and npm run verify:essentials. The Engine exporter is an external prerequisite, not included in this archive. The cumulative preview also needs the Heart, Bell, and Search asset folders from the project."));
        assert!(
            rebuild_text("tools")
                .contains("also needs the existing icon asset folders from the project. Generated")
        );
    }
}
