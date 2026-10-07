use crate::{Error, Result};
use std::path::{Path, PathBuf};

/// The sampling differences between the builders of sets 03 onward (js-runtime §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModernProfile {
    /// `d` receives the part's width as a 4th argument (06 onward; 03–05 pass 3 arguments).
    pub pass_width: bool,
    /// Icons whose declared `duotoneFill` disagrees with their tint parts are rejected (07 onward).
    pub check_duotone: bool,
    /// How reverse animations are named.
    pub reverse_name: ReverseName,
}

impl ModernProfile {
    /// 03-essentials.
    pub const ESSENTIALS: Self =
        Self { pass_width: false, check_duotone: false, reverse_name: ReverseName::Essentials };
    /// 04-actions and 05-interface.
    pub const DIRECTED: Self = Self { pass_width: false, check_duotone: false, reverse_name: ReverseName::Suffix };
    /// 06-tools.
    pub const TOOLS: Self = Self { pass_width: true, check_duotone: false, reverse_name: ReverseName::Suffix };
    /// Sets 07–63 (`library/build-set.mjs` and its set-owned copies) and user projects.
    pub const SHARED: Self = Self { pass_width: true, check_duotone: true, reverse_name: ReverseName::Suffix };
}

/// How a stateful icon's reverse animation is named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseName {
    /// `icon.reverseSuffix`.
    Suffix,
    /// 03-essentials: `-reset` for `check`, `-unlock` otherwise (its icons have no suffix).
    Essentials,
}

/// Sets 01 and 02, whose authoring lives inline in their `build.mjs`. gai-js ships frozen
/// copies of it as the builtin modules `gai:legacy/01` and `gai:legacy/02`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacySet {
    HeartBell,
    Search,
}

impl LegacySet {
    /// The builtin module holding the set's definitions.
    pub fn module(self) -> &'static str {
        match self {
            LegacySet::HeartBell => "gai:legacy/01",
            LegacySet::Search => "gai:legacy/02",
        }
    }

    /// The set folder name.
    pub fn set_id(self) -> &'static str {
        match self {
            LegacySet::HeartBell => "01-heart-bell",
            LegacySet::Search => "02-search",
        }
    }
}

/// Where a set's definitions come from, and how they are sampled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetSource {
    Legacy(LegacySet),
    /// `set_dir/definitions.mjs`. Relative imports may reach `set_dir` and `library_dir` only.
    Definitions {
        set_dir: PathBuf,
        library_dir: PathBuf,
        profile: ModernProfile,
    },
}

impl SetSource {
    /// A library set folder (`motion/icon-sets/NN-slug`), with the profile of the builder
    /// that set used: 01 and 02 get their shims, 03–06 their own profiles, 07 onward
    /// [`ModernProfile::SHARED`]. The kit is `set_dir/../library`.
    pub fn library_set(set_dir: impl Into<PathBuf>) -> Result<Self> {
        let set_dir = set_dir.into();
        let number = set_number(&set_dir).ok_or_else(|| Error::NotASet(set_dir.clone()))?;
        let profile = match number {
            1 => return Ok(SetSource::Legacy(LegacySet::HeartBell)),
            2 => return Ok(SetSource::Legacy(LegacySet::Search)),
            3 => ModernProfile::ESSENTIALS,
            4 | 5 => ModernProfile::DIRECTED,
            6 => ModernProfile::TOOLS,
            _ => ModernProfile::SHARED,
        };
        let library_dir = set_dir.join("..").join("library");
        Ok(SetSource::Definitions { set_dir, library_dir, profile })
    }

    /// A user project's icon source (`generative-icons/icons`), a `build-set.mjs`-style set
    /// whose managed kit copy is `generative-icons/library`.
    pub fn user_project(icons_dir: impl Into<PathBuf>) -> Self {
        let set_dir = icons_dir.into();
        let library_dir = set_dir.join("..").join("library");
        SetSource::Definitions { set_dir, library_dir, profile: ModernProfile::SHARED }
    }

    /// A short name for messages: the set folder name.
    pub fn label(&self) -> String {
        match self {
            SetSource::Legacy(set) => set.set_id().to_owned(),
            SetSource::Definitions { set_dir, .. } => set_dir
                .file_name()
                .map_or_else(|| set_dir.display().to_string(), |name| name.to_string_lossy().into_owned()),
        }
    }
}

/// The leading number of a `NN-slug` folder name.
fn set_number(set_dir: &Path) -> Option<u32> {
    let name = set_dir.file_name()?.to_str()?;
    let digits = name.split('-').next()?;
    if digits.len() != 2 || !name[2..].starts_with('-') {
        return None;
    }
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_sets_use_their_builders_profile() {
        let source = |name: &str| SetSource::library_set(Path::new("/r/motion/icon-sets").join(name)).unwrap();
        assert_eq!(source("01-heart-bell"), SetSource::Legacy(LegacySet::HeartBell));
        assert_eq!(source("02-search"), SetSource::Legacy(LegacySet::Search));
        let profile = |name: &str| match source(name) {
            SetSource::Definitions { profile, .. } => profile,
            other => panic!("{other:?}"),
        };
        assert_eq!(profile("03-essentials"), ModernProfile::ESSENTIALS);
        assert_eq!(profile("04-actions"), ModernProfile::DIRECTED);
        assert_eq!(profile("05-interface"), ModernProfile::DIRECTED);
        assert_eq!(profile("06-tools"), ModernProfile::TOOLS);
        assert_eq!(profile("07-ai-conversation"), ModernProfile::SHARED);
        assert_eq!(profile("63-voice-agents"), ModernProfile::SHARED);
        assert!(matches!(SetSource::library_set("/r/motion/icon-sets/library"), Err(Error::NotASet(_))));
    }

    #[test]
    fn user_projects_resolve_the_kit_next_to_the_icons() {
        let source = SetSource::user_project("/p/generative-icons/icons");
        assert_eq!(
            source,
            SetSource::Definitions {
                set_dir: "/p/generative-icons/icons".into(),
                library_dir: "/p/generative-icons/icons/../library".into(),
                profile: ModernProfile::SHARED,
            }
        );
        assert_eq!(source.label(), "icons");
    }
}
