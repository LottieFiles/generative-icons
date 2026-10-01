//! Set profiles: the builder knobs that differ between sets (builder spec §8, K1–K18).
//!
//! Sets 03–06 have no `set-config.json`; their builders hard-coded everything, so they get
//! built-in profiles. Sets 07 onward read `set-config.json`, whose optional `"builder"` object
//! can change the few knobs that ever varied (K4–K6, K12, K13). The historical values of
//! 07–15 live in a built-in table under the same shape. Everything else defaults to the
//! canonical builder of sets 16–63.
//!
//! Knobs that are constant everywhere (K7, fps, size, colours, weights, compaction, markers,
//! slots, manifests and themes) are not part of a profile.

use gai_js::ModernProfile;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// How a record's `continuous` field is written (K12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Continuous {
    /// No `continuous` field (03–11, 13–15).
    Omit,
    /// `Boolean(icon.continuous)` (16–63 and new sets).
    Icon,
    /// Always `true` (12).
    Always,
}

/// The record key of the builder hash and the file it hashes (K13, K18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuilderHash {
    /// `setBuilderSha256` of the set's `build.mjs` (16–63 and new sets).
    Set,
    /// `continuousBuilderSha256` of the set's `build.mjs` (12).
    Continuous,
    /// `sharedBuilderSha256` of `library/build-set.mjs` (07–11, 13–15).
    Shared,
}

impl BuilderHash {
    /// The record key.
    pub fn key(self) -> &'static str {
        match self {
            BuilderHash::Set => "setBuilderSha256",
            BuilderHash::Continuous => "continuousBuilderSha256",
            BuilderHash::Shared => "sharedBuilderSha256",
        }
    }

    /// The hashed file, given the set folder and the kit folder.
    pub fn file(self, root: &Path, library: &Path) -> PathBuf {
        match self {
            BuilderHash::Set | BuilderHash::Continuous => root.join(SOURCE_FILE),
            BuilderHash::Shared => library.join("build-set.mjs"),
        }
    }
}

/// The file behind `sourceSha256` (K17), relative to the set folder.
pub const SOURCE_FILE: &str = "build.mjs";

/// The optional `"builder"` object of `set-config.json`. Every field defaults to the
/// canonical builder's value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuilderConfig {
    /// Top-level `set.json` `category` (K4). Default: the set name.
    pub category: Option<String>,
    /// `set.json` `version` (K5). Default `"1.0"`.
    pub version: Option<String>,
    /// `set.json` `defaultMotion` (K6). Default `"strong"`.
    pub default_motion: Option<String>,
    /// The record `continuous` field (K12). Default [`Continuous::Icon`].
    pub continuous: Option<Continuous>,
    /// The builder hash (K13). Default [`BuilderHash::Set`].
    pub builder_hash: Option<BuilderHash>,
}

impl BuilderConfig {
    /// `self`, with unset fields taken from `base`.
    fn or(self, base: BuilderConfig) -> BuilderConfig {
        BuilderConfig {
            category: self.category.or(base.category),
            version: self.version.or(base.version),
            default_motion: self.default_motion.or(base.default_motion),
            continuous: self.continuous.or(base.continuous),
            builder_hash: self.builder_hash.or(base.builder_hash),
        }
    }
}

/// The fields of `set-config.json` the builder reads. Other fields are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetConfig {
    /// The set id (`NN-slug`): `set.json` `id`, `meta.set` and the asset folder `NN`.
    pub id: String,
    /// The set name: `set.json` `name`, composition names and the record `category`.
    pub name: String,
    /// Kit files hashed into `directionSources` (K16). `null` or absent means the default
    /// `["drawing.mjs", "direction.mjs"]`; an empty list hashes nothing, as in JS.
    #[serde(default)]
    pub shared_sources: Option<Vec<String>>,
    #[serde(default)]
    pub builder: BuilderConfig,
}

impl SetConfig {
    /// The file name, inside the set folder.
    pub const FILE: &'static str = "set-config.json";

    /// Reads `<root>/set-config.json`.
    pub fn load(root: &Path) -> Result<SetConfig, ProfileError> {
        let path = root.join(Self::FILE);
        let bytes = std::fs::read(&path).map_err(|source| ProfileError::Read { path: path.clone(), source })?;
        serde_json::from_slice(&bytes).map_err(|source| ProfileError::Config { path, source })
    }
}

/// Why a set's profile could not be determined.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// Sets 01 and 02 keep their authoring inside `build.mjs` and are never rebuilt.
    #[error("{} has no definitions.mjs; sets 01 and 02 are frozen and never rebuilt", .0.display())]
    NoDefinitions(PathBuf),
    #[error("cannot read {}: {source}", path.display())]
    Read { path: PathBuf, source: std::io::Error },
    #[error("{}: {source}", path.display())]
    Config { path: PathBuf, source: serde_json::Error },
}

/// Everything the builder does differently for one set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// `set.json` `id` and `meta.set` (K3).
    pub id: String,
    /// `set.json` `name` (K2); also the record `category` when [`Profile::authoring`] is set.
    pub name: String,
    /// Top-level `set.json` `category` (K4); `None` omits it (03–06).
    pub category: Option<String>,
    /// `set.json` `version` (K5).
    pub version: String,
    /// `set.json` `defaultMotion` (K6), with the 11 and 13–15 packaging patches folded in.
    pub default_motion: String,
    /// The composition name up to the icon label (K8): `Motion / <name> / ` or, for 03–06,
    /// `Motion / Set NN / `.
    pub composition: String,
    /// Sampling differences (K9 guards, K10 reverse names, K11 width argument).
    pub sampling: ModernProfile,
    /// The record `continuous` field (K12).
    pub continuous: Continuous,
    /// The record builder hash (K13, K18); `None` for 03–06.
    pub builder_hash: Option<BuilderHash>,
    /// Records carry `authoringSources`, `brief`, `constraints` and `category` (K14).
    pub authoring: bool,
    /// Records carry `duotoneFill` (K15); 03 does not.
    pub duotone_fill: bool,
    /// Kit files hashed into `directionSources`, in record key order (K16).
    pub direction_sources: Vec<String>,
}

/// The kit files every configured set hashes unless `sharedSources` says otherwise.
const DEFAULT_SHARED_SOURCES: [&str; 2] = ["drawing.mjs", "direction.mjs"];

impl Profile {
    /// The profile of a library set folder (`motion/icon-sets/NN-slug`): built in for 03–06,
    /// otherwise `set-config.json` over the built-in values of 07–15. Folders without
    /// `definitions.mjs` (01 and 02) are refused.
    pub fn library_set(set_dir: &Path) -> Result<Profile, ProfileError> {
        if !set_dir.join("definitions.mjs").is_file() {
            return Err(ProfileError::NoDefinitions(set_dir.to_path_buf()));
        }
        let folder = set_dir.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if let Some(profile) = directed(folder) {
            return Ok(profile);
        }
        let config = SetConfig::load(set_dir)?;
        let builder = config.builder.clone().or(historical(&config.id));
        Ok(Profile::configured(&config, builder))
    }

    /// The profile of a set described by `config` outside the library, such as a user
    /// project: the canonical builder with `config.builder` applied, and no built-in table.
    pub fn project(config: &SetConfig) -> Profile {
        Profile::configured(config, config.builder.clone())
    }

    fn configured(config: &SetConfig, builder: BuilderConfig) -> Profile {
        let sources = config
            .shared_sources
            .clone()
            .unwrap_or_else(|| DEFAULT_SHARED_SOURCES.iter().map(|s| s.to_string()).collect());
        Profile {
            id: config.id.clone(),
            name: config.name.clone(),
            category: Some(builder.category.unwrap_or_else(|| config.name.clone())),
            version: builder.version.unwrap_or_else(|| "1.0".into()),
            default_motion: builder.default_motion.unwrap_or_else(|| "strong".into()),
            composition: format!("Motion / {} / ", config.name),
            sampling: ModernProfile::SHARED,
            continuous: builder.continuous.unwrap_or(Continuous::Icon),
            builder_hash: Some(builder.builder_hash.unwrap_or(BuilderHash::Set)),
            authoring: true,
            duotone_fill: true,
            direction_sources: sources,
        }
    }

    /// The asset folder number: the first two characters of the id (`config.id.slice(0,2)`).
    pub fn number(&self) -> String {
        self.id.chars().take(2).collect()
    }
}

/// The historical builder values of 07–15, which differ from the canonical builder. The
/// 11 and 13–15 values include their `package.py` patches (category = the set name, and
/// Strong for 13–15), which the canonical defaults already give.
fn historical(id: &str) -> BuilderConfig {
    let shared = BuilderConfig {
        continuous: Some(Continuous::Omit),
        builder_hash: Some(BuilderHash::Shared),
        ..BuilderConfig::default()
    };
    match id {
        "07-ai-conversation" | "08-ai-knowledge" | "09-ai-creation" | "10-ai-agents" => {
            BuilderConfig { category: Some("AI".into()), default_motion: Some("light".into()), ..shared }
        }
        "11-feedback-status" => BuilderConfig { default_motion: Some("light".into()), ..shared },
        "13-fintech-finance" | "14-travel" | "15-commerce-delivery" => shared,
        "12-loading-indicators" => BuilderConfig {
            continuous: Some(Continuous::Always),
            builder_hash: Some(BuilderHash::Continuous),
            ..BuilderConfig::default()
        },
        _ => BuilderConfig::default(),
    }
}

/// The hard-coded builders of 03–06 (`03-essentials/build.mjs` and siblings).
fn directed(folder: &str) -> Option<Profile> {
    let (name, version, sampling, duotone_fill, sources): (&str, &str, ModernProfile, bool, &[&str]) = match folder {
        "03-essentials" => (
            "Five useful actions",
            "1.1",
            ModernProfile::ESSENTIALS,
            false,
            &["drawing.mjs", "direction.mjs", "directed-essentials.mjs"],
        ),
        "04-actions" => (
            "Ten everyday actions",
            "1.0",
            ModernProfile::DIRECTED,
            true,
            &["drawing.mjs", "direction.mjs", "directed-actions.mjs"],
        ),
        "05-interface" => (
            "Twenty interface actions",
            "1.0",
            ModernProfile::DIRECTED,
            true,
            &["drawing.mjs", "direction.mjs", "directed-interface.mjs"],
        ),
        "06-tools" => (
            "Tools and controls",
            "1.0",
            ModernProfile::TOOLS,
            true,
            &["drawing.mjs", "direction.mjs", "directed-tools.mjs", "keyframes.mjs"],
        ),
        _ => return None,
    };
    Some(Profile {
        id: folder.to_owned(),
        name: name.to_owned(),
        category: None,
        version: version.to_owned(),
        default_motion: "light".to_owned(),
        composition: format!("Motion / Set {} / ", &folder[..2]),
        sampling,
        continuous: Continuous::Omit,
        builder_hash: None,
        authoring: false,
        duotone_fill,
        direction_sources: sources.iter().map(|s| s.to_string()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(id: &str, name: &str) -> SetConfig {
        SetConfig { id: id.into(), name: name.into(), shared_sources: None, builder: BuilderConfig::default() }
    }

    fn library(id: &str, name: &str) -> Profile {
        let config = config(id, name);
        let builder = config.builder.clone().or(historical(id));
        Profile::configured(&config, builder)
    }

    #[test]
    fn canonical_defaults() {
        let profile = library("34-scheduling-time", "Scheduling & time tools");
        assert_eq!(profile.category.as_deref(), Some("Scheduling & time tools"));
        assert_eq!((profile.version.as_str(), profile.default_motion.as_str()), ("1.0", "strong"));
        assert_eq!(profile.composition, "Motion / Scheduling & time tools / ");
        assert_eq!(profile.continuous, Continuous::Icon);
        assert_eq!(profile.builder_hash, Some(BuilderHash::Set));
        assert_eq!(profile.sampling, ModernProfile::SHARED);
        assert_eq!(profile.direction_sources, ["drawing.mjs", "direction.mjs"]);
        assert_eq!(profile.number(), "34");
    }

    #[test]
    fn historical_sets() {
        let ai = library("08-ai-knowledge", "AI knowledge");
        assert_eq!((ai.category.as_deref(), ai.default_motion.as_str()), (Some("AI"), "light"));
        assert_eq!((ai.continuous, ai.builder_hash), (Continuous::Omit, Some(BuilderHash::Shared)));
        let feedback = library("11-feedback-status", "Feedback & status");
        assert_eq!(
            (feedback.category.as_deref(), feedback.default_motion.as_str()),
            (Some("Feedback & status"), "light")
        );
        let travel = library("14-travel", "Travel");
        assert_eq!((travel.category.as_deref(), travel.default_motion.as_str()), (Some("Travel"), "strong"));
        assert_eq!(travel.builder_hash, Some(BuilderHash::Shared));
        let loading = library("12-loading-indicators", "Loading indicators");
        assert_eq!((loading.continuous, loading.builder_hash), (Continuous::Always, Some(BuilderHash::Continuous)));
    }

    #[test]
    fn directed_sets() {
        let essentials = directed("03-essentials").unwrap();
        assert_eq!(essentials.composition, "Motion / Set 03 / ");
        assert_eq!((essentials.version.as_str(), essentials.duotone_fill), ("1.1", false));
        assert_eq!(essentials.sampling, ModernProfile::ESSENTIALS);
        let tools = directed("06-tools").unwrap();
        assert_eq!((tools.category, tools.builder_hash, tools.authoring), (None, None, false));
        assert_eq!(tools.direction_sources.last().map(String::as_str), Some("keyframes.mjs"));
        assert!(directed("07-ai-conversation").is_none());
    }

    #[test]
    fn builder_object_overrides_the_table() {
        let config: SetConfig = serde_json::from_str(
            r#"{"id":"12-loading-indicators","name":"Loading","sharedSources":["drawing.mjs"],"designReview":"x",
                "builder":{"version":"2.0","continuous":"icon"}}"#,
        )
        .unwrap();
        let profile = Profile::configured(&config, config.builder.clone().or(historical(&config.id)));
        assert_eq!(profile.version, "2.0");
        assert_eq!((profile.continuous, profile.builder_hash), (Continuous::Icon, Some(BuilderHash::Continuous)));
        assert_eq!(profile.direction_sources, ["drawing.mjs"]);
        // A user project ignores the table.
        assert_eq!(Profile::project(&config).builder_hash, Some(BuilderHash::Set));
        let typo = serde_json::from_str::<SetConfig>(r#"{"id":"70-x","name":"X","builder":{"categroy":"Y"}}"#);
        assert!(typo.is_err());
    }
}
