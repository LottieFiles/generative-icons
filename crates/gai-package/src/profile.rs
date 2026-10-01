//! What each set's own `package.py` did around the shared packager, as data: `set.json`
//! patches applied before packaging, and extra files appended to the set zip afterwards.
//! Historical sets use the built-in table below; new sets use `set-config.json`
//! (`categoryOverride`, `defaultMotionOverride`, `packageExtras`).
//!
//! Deviations from the originals, all without effect on today's tree: recursive listings
//! are sorted (the originals used filesystem order), and the `invalid-` and
//! `before-commerce` path filters look only at components below the reviewed folder (the
//! originals also matched ancestors of the checkout).

use crate::config::{SetConfig, SetPaths};
use crate::error::Result;
use crate::io::read_json;
use crate::paths::{files_below, glob, py_suffix};
use crate::zip::ZipDate;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// A file appended to the set zip as `source/<repo-relative path>`.
#[derive(Debug, Clone)]
pub(crate) struct Extra {
    pub path: PathBuf,
    /// The original read it without an existence check (sets 54 and 60), so a missing
    /// file is an error instead of being skipped.
    pub required: bool,
}

/// Extras appended after the shared set zip.
#[derive(Debug, Clone)]
pub(crate) struct Extras {
    pub date: ZipDate,
    pub files: Vec<Extra>,
    /// Skip names already in the zip. Set 45 appended without this check.
    pub skip_present: bool,
}

/// A set's packaging customizations.
#[derive(Debug, Clone, Default)]
pub(crate) struct Profile {
    /// Top-level `set.json` keys replaced before packaging.
    pub patches: Vec<(&'static str, String)>,
    pub extras: Vec<Extras>,
}

/// The profile of a set with a `set-config.json`: the built-in entry for historical sets,
/// followed by the config's own overrides and extras.
pub(crate) fn profile(paths: &SetPaths, config: &SetConfig) -> Result<Profile> {
    let mut profile = historical(paths)?;
    if let Some(category) = &config.category_override {
        profile.patches.push(("category", category.clone()));
    }
    if let Some(motion) = &config.default_motion_override {
        profile.patches.push(("defaultMotion", motion.clone()));
    }
    if !config.package_extras.is_empty() {
        let mut files = Vec::new();
        for pattern in &config.package_extras {
            let found = if pattern.contains(['*', '?']) {
                glob(&paths.repo, pattern)?
            } else {
                vec![paths.repo.join(pattern)]
            };
            files.extend(found.into_iter().map(optional));
        }
        profile.extras.push(Extras {
            date: ZipDate::SETS,
            files,
            skip_present: true,
        });
    }
    Ok(profile)
}

fn optional(path: PathBuf) -> Extra {
    Extra {
        path,
        required: false,
    }
}

fn required(path: PathBuf) -> Extra {
    Extra {
        path,
        required: true,
    }
}

/// `set.json` patches of the wrappers for 11 and 13–15: category, and Strong as the default motion.
const CATEGORIES: [(&str, &str, bool); 4] = [
    ("11-feedback-status", "Feedback & status", false),
    ("13-fintech-finance", "Fintech & finance", true),
    ("14-travel", "Travel", true),
    ("15-commerce-delivery", "Commerce & delivery", true),
];

/// Library files the review-packet wrappers (16–34) bundle.
const HARNESS: [&str; 5] = [
    "family-contract.json",
    "generation-context.mjs",
    "preflight.mjs",
    "build-checked.mjs",
    "director-revision/serve-verification.py",
];
const ELLIPSIS_HARNESS: [&str; 6] = [
    "verify-filled-ellipsis.mjs",
    "family-contract.json",
    "generation-context.mjs",
    "preflight.mjs",
    "build-checked.mjs",
    "director-revision/serve-verification.py",
];
const SUBPATH_HARNESS: [&str; 8] = [
    "geometry-subpath.mjs",
    "geometry-subpath.test.mjs",
    "verify-filled-ellipsis.mjs",
    "family-contract.json",
    "generation-context.mjs",
    "preflight.mjs",
    "build-checked.mjs",
    "director-revision/serve-verification.py",
];
const MEASURED: [&str; 2] = ["measure-motion.mjs", "evidence/motion-measurements.json"];
const CONTRACTED: [&str; 3] = [
    "geometry-contract.json",
    "measure-motion.mjs",
    "evidence/motion-measurements.json",
];

/// Review-packet wrappers (16–34): set-local files, library files, and whether the
/// `finish-pending-27` captures are added. The review folder is the id without `NN-`.
type Packet = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    bool,
);
const PACKETS: [Packet; 19] = [
    ("16-voice", &[], &HARNESS, false),
    ("17-media-playback", &[], &HARNESS, false),
    ("18-motion-design", &MEASURED, &HARNESS, false),
    ("19-agent-operations", &MEASURED, &HARNESS, false),
    ("20-arrows", &MEASURED, &HARNESS, false),
    ("21-annotation-arrows", &MEASURED, &HARNESS, false),
    ("22-teamwork", &CONTRACTED, &HARNESS, false),
    ("23-get-help", &CONTRACTED, &HARNESS, false),
    ("24-file-transfer", &CONTRACTED, &HARNESS, false),
    (
        "25-everyday-controls",
        &CONTRACTED,
        &ELLIPSIS_HARNESS,
        false,
    ),
    ("26-account-privacy", &CONTRACTED, &ELLIPSIS_HARNESS, false),
    ("27-forms-input", &CONTRACTED, &ELLIPSIS_HARNESS, false),
    ("28-tables-analytics", &CONTRACTED, &ELLIPSIS_HARNESS, false),
    (
        "29-text-document-editing",
        &CONTRACTED,
        &ELLIPSIS_HARNESS,
        false,
    ),
    (
        "30-accessibility-reading",
        &CONTRACTED,
        &ELLIPSIS_HARNESS,
        false,
    ),
    (
        "31-developer-version-control",
        &CONTRACTED,
        &ELLIPSIS_HARNESS,
        false,
    ),
    (
        "32-infrastructure-devices",
        &CONTRACTED,
        &SUBPATH_HARNESS,
        true,
    ),
    (
        "33-communication-social",
        &CONTRACTED,
        &SUBPATH_HARNESS,
        true,
    ),
    ("34-scheduling-time", &CONTRACTED, &SUBPATH_HARNESS, true),
];

fn historical(paths: &SetPaths) -> Result<Profile> {
    let id = paths.id();
    let slug = &id[3..];
    let mut profile = Profile::default();
    if let Some((_, category, strong)) = CATEGORIES.iter().find(|(set, ..)| *set == id) {
        profile.patches.push(("category", category.to_string()));
        if *strong {
            profile
                .patches
                .push(("defaultMotion", "strong".to_string()));
        }
    }
    profile.extras = match id {
        "13-fintech-finance" | "14-travel" | "15-commerce-delivery" => family_review(paths, slug)?,
        "45-chat-messaging" => chat_captures(paths)?,
        "54-gamification-rewards" => gamification(paths),
        "60-models-providers" => models_providers(paths),
        _ => match PACKETS.iter().find(|packet| packet.0 == id) {
            Some((_, local, library, captures)) => {
                let date = if id == "16-voice" {
                    ZipDate::VOICE_EXTRAS
                } else {
                    ZipDate::REVIEW_EXTRAS
                };
                let files = review_packet(paths, local, library, slug, *captures)?;
                vec![Extras {
                    date,
                    files,
                    skip_present: true,
                }]
            }
            None => Vec::new(),
        },
    };
    Ok(profile)
}

/// Sets 13–15: the family review folder, the review page and the evidence sheets.
fn family_review(paths: &SetPaths, slug: &str) -> Result<Vec<Extras>> {
    let (repo, library, evidence) = (&paths.repo, paths.library(), paths.root.join("evidence"));
    let reviews = library.join("design-reviews");
    let review = reviews.join(slug);
    let commerce = paths.number() == "15";
    let mut files = glob(&review, "*.py")?;
    if paths.number() != "13" {
        files.extend(glob(&review, "visual-*")?);
        files.extend(glob(&review, "motion-before-*.png")?);
    }
    let named = [
        "asset-baseline.json",
        "scope.json",
        "live-interactions.json",
        "delivery-verification.json",
        "family-before-supervisor-refinement.png",
    ];
    files.extend(named.iter().map(|n| review.join(n)));
    files.push(library.join("director-revision/serve-verification.py"));
    files.push(
        repo.join("prototype/icons")
            .join(paths.number())
            .join("family-review.html"),
    );
    if commerce {
        files.extend(glob(&evidence, "browser-verification-*.json")?);
    }
    files.extend(glob(&evidence, "family-*.png")?);
    files.extend(glob(&evidence, "motion-review-*.png")?);
    if commerce {
        // The Return package correction and the Strong motion audit.
        let returns = reviews.join("return-package-v2");
        files.extend(
            files_below(&returns)?
                .into_iter()
                .filter(|p| file_name(p) != "archive-verification.json"),
        );
        let strength = reviews.join("strong-motion-audit");
        let audited = files_below(&strength)?.into_iter().filter(|p| {
            let name = file_name(p);
            !below(p, &strength)
                .iter()
                .any(|part| part == "before-commerce")
                && [".md", ".json", ".py", ".png"].contains(&py_suffix(&name))
                && name != "archive-verification.json"
        });
        files.extend(audited);
        files.push(repo.join("prototype/icons/15/strength-review.html"));
        let changed = strength.join("changed-icons.json");
        if changed.exists() {
            let names = read_json(&changed)?;
            for name in names
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|n| *n != "delivery-truck")
            {
                files.push(
                    strength
                        .join("before-commerce/assets")
                        .join(format!("{name}-strong-poses.png")),
                );
            }
        }
    }
    Ok(vec![Extras {
        date: ZipDate::SETS,
        files: files.into_iter().map(optional).collect(),
        skip_present: true,
    }])
}

/// Sets 16–34: set-local measurement files, the generation harness and the review packet.
fn review_packet(
    paths: &SetPaths,
    local: &[&str],
    library_files: &[&str],
    review: &str,
    captures: bool,
) -> Result<Vec<Extra>> {
    let library = paths.library();
    let reviews = library.join("design-reviews");
    let mut files: Vec<PathBuf> = local.iter().map(|n| paths.root.join(n)).collect();
    files.extend(library_files.iter().map(|n| library.join(n)));
    let packet = reviews.join(review);
    let packet_files = files_below(&packet)?.into_iter().filter(|p| {
        let name = file_name(p);
        [".json", ".md", ".mjs", ".py", ".html", ".png"].contains(&py_suffix(&name))
            && ![
                "assets-before.json",
                "delivery.json",
                "archive-verification.json",
            ]
            .contains(&name.as_str())
            && !below(p, &packet)
                .iter()
                .any(|part| part.starts_with("invalid-"))
    });
    files.extend(packet_files);
    if captures {
        // Acceptance evidence referenced by the per-icon ledger.
        let finish = reviews.join("finish-pending-27");
        let named = [
            "scope.md",
            "icon-review-ledger.json",
            "render-summary.json",
            "native-change-scope.json",
            "alarm-light-preservation.json",
            "gallery-verification.json",
        ];
        files.extend(named.iter().map(|n| finish.join(n)));
        let captured = files_below(&finish.join("captures").join(review))?;
        files.extend(
            captured
                .into_iter()
                .filter(|p| [".json", ".png"].contains(&py_suffix(&file_name(p)))),
        );
    }
    Ok(files.into_iter().map(optional).collect())
}

/// Set 45: the review tool and native review captures, appended without a name check.
fn chat_captures(paths: &SetPaths) -> Result<Vec<Extras>> {
    let evidence = paths.root.join("evidence");
    let mut files = vec![
        paths.root.join("capture-review.mjs"),
        evidence.join("native-review-capture.json"),
    ];
    files.extend(glob(&evidence, "native-small-sizes-*.png")?);
    files.extend(glob(&evidence, "native-equal-time-*.png")?);
    Ok(vec![Extras {
        date: ZipDate::SETS,
        files: files.into_iter().map(optional).collect(),
        skip_present: false,
    }])
}

/// Set 54: helper imports, the review page and docs (required), then optional captures.
fn gamification(paths: &SetPaths) -> Vec<Extras> {
    let (repo, library, evidence) = (&paths.repo, paths.library(), paths.root.join("evidence"));
    let mut files: Vec<Extra> = [
        "ai-authoring.mjs",
        "paint-tokens.mjs",
        "family-contract.json",
    ]
    .iter()
    .map(|n| required(library.join(n)))
    .collect();
    files.push(required(repo.join("prototype/icons/54/review.html")));
    files.extend(
        ["development.md", "verification.md"]
            .iter()
            .map(|n| required(repo.join("docs").join(n))),
    );
    let captures = [
        "native-small-dark.png",
        "native-small-light.png",
        "native-initial-dark.png",
        "native-equal-time.png",
    ];
    files.extend(captures.iter().map(|n| optional(evidence.join(n))));
    vec![Extras {
        date: ZipDate::SETS,
        files,
        skip_present: true,
    }]
}

/// Set 60: the paint binder and source-check entry points (required).
fn models_providers(paths: &SetPaths) -> Vec<Extras> {
    let library = [
        "paint-tokens.mjs",
        "preflight.mjs",
        "build-checked.mjs",
        "generation-context.mjs",
        "family-contract.json",
    ];
    let mut files: Vec<Extra> = library
        .iter()
        .map(|n| required(paths.library().join(n)))
        .collect();
    files.extend(
        ["development.md", "verification.md"]
            .iter()
            .map(|n| required(paths.repo.join("docs").join(n))),
    );
    vec![Extras {
        date: ZipDate::SETS,
        files,
        skip_present: true,
    }]
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The components of `path` below `base`.
fn below(path: &Path, base: &Path) -> Vec<String> {
    path.strip_prefix(base)
        .map(|rest| {
            rest.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(json: &str) -> SetConfig {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn historical_patches() {
        let paths = SetPaths::new("/repo", "14-travel").unwrap();
        let profile = profile(&paths, &config(r#"{"id":"14-travel"}"#)).unwrap();
        assert_eq!(
            profile.patches,
            [
                ("category", "Travel".to_string()),
                ("defaultMotion", "strong".to_string())
            ]
        );
        assert_eq!(profile.extras.len(), 1);
        let names: Vec<_> = profile.extras[0]
            .files
            .iter()
            .map(|e| {
                e.path
                    .strip_prefix("/repo")
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            names[5],
            "motion/icon-sets/library/director-revision/serve-verification.py"
        );
        assert_eq!(names[6], "prototype/icons/14/family-review.html");
    }

    #[test]
    fn config_overrides_and_extras() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["docs/b.md", "docs/a.md", "docs/c.txt"] {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"").unwrap();
        }
        let paths = SetPaths::new(dir.path(), "70-new-set").unwrap();
        let config = config(
            r#"{"id":"70-new-set","categoryOverride":"New","packageExtras":["docs/*.md","LICENSE"]}"#,
        );
        let profile = profile(&paths, &config).unwrap();
        assert_eq!(profile.patches, [("category", "New".to_string())]);
        let files: Vec<_> = profile.extras[0]
            .files
            .iter()
            .map(|e| e.path.strip_prefix(dir.path()).unwrap().to_path_buf())
            .collect();
        assert_eq!(
            files,
            [
                PathBuf::from("docs/a.md"),
                PathBuf::from("docs/b.md"),
                PathBuf::from("LICENSE")
            ]
        );
    }
}
