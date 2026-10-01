//! Geometry contracts and clearance: painted margins, part gaps, attachments and joints
//! checked on every frame of every action's world geometry (`scenes/*.scene.json`).
//!
//! A set's clearance program is one of:
//! - its `geometry-contract.json` (v1, sets 22–39 and CLI-authored sets), read natively;
//! - built-in rules converted from the hand-coded 16–20 scripts (`rules/*.rules.json`);
//! - a frozen port of the hand-coded 12–15, 21 or 45 script ([`legacy`]).
//!
//! Every path reproduces the committed `evidence/clearance-verification.json` byte for byte.
//!
//! ```no_run
//! # use gai_check::contracts::{Clearance, ScenesDir, SetManifest, EVIDENCE_FILE};
//! # use std::path::Path;
//! let set_dir = Path::new("motion/icon-sets/39-food-drink");
//! let manifest = SetManifest::read(Path::new("prototype/icons/39/assets/set.json"))?;
//! if let Some(clearance) = Clearance::for_set(set_dir, &manifest.id)? {
//!     let report = clearance.run(&manifest, &ScenesDir(set_dir.join("scenes")))?;
//!     std::fs::write(set_dir.join(EVIDENCE_FILE), report.to_json())?;
//!     assert!(report.passed());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod engine;
pub mod evidence;
pub mod expr;
pub mod geom;
pub mod jsnum;
pub mod legacy;
pub mod rules;
pub mod scene;

pub use evidence::{ClearanceReport, EVIDENCE_FILE};
pub use legacy::LegacySet;
pub use rules::RuleSet;
pub use scene::{Scene, SceneNode, SceneSource, ScenesDir, SetAction, SetIcon, SetManifest};

use std::path::{Path, PathBuf};

/// Clearance failures that abort before evidence is written. Checks that merely fail in
/// collect mode are reported in [`ClearanceReport::errors`] instead.
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    /// A file could not be read.
    #[error("cannot read {}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error,
    },
    /// A file is not valid JSON of the expected shape.
    #[error("cannot parse {}: {source}", path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: serde_json::Error,
    },
    /// A rules document or contract is malformed.
    #[error("invalid rules: {0}")]
    Rules(String),
    /// A rule names a part the scene does not have.
    #[error("missing scene part {0}")]
    MissingPart(String),
    /// A `set.json` icon has no entry in a contract that requires one.
    #[error("{0} missing supervisor geometry contract")]
    MissingContract(String),
    /// A live rule has an unknown kind.
    #[error("Unknown geometry rule {0}")]
    UnknownRule(String),
    /// `aSegments`/`bSegments` is not a valid `[start, end)` command range.
    #[error("Invalid subpath range")]
    InvalidSubpath,
    /// A subpath range starts on a command that is not a point.
    #[error("Subpath must start with a point")]
    SubpathStart,
    /// A path uses a command code other than M, L, C or Z.
    #[error("Unknown path command")]
    UnknownPathCommand,
    /// Scene geometry the checks cannot read (too few coordinates, drawing before a move).
    #[error("{0}")]
    Geometry(String),
    /// An expression evaluated to the wrong type.
    #[error("expression error: {0}")]
    Expr(String),
    /// An assert-mode check failed; no evidence is written.
    #[error("{0}")]
    Assertion(String),
}

/// Built-in rules converted from the hand-coded 16–20 scripts. They reproduce the
/// committed PASS evidence byte for byte; on failure, messages and counts follow the
/// legacy scripts except that a missing second part is reported only when the first
/// part is visible.
const BUILTIN_RULES: &[(&str, &str)] = &[
    ("16-voice", include_str!("../../rules/16-voice.rules.json")),
    (
        "17-media-playback",
        include_str!("../../rules/17-media-playback.rules.json"),
    ),
    (
        "18-motion-design",
        include_str!("../../rules/18-motion-design.rules.json"),
    ),
    (
        "19-agent-operations",
        include_str!("../../rules/19-agent-operations.rules.json"),
    ),
    (
        "20-arrows",
        include_str!("../../rules/20-arrows.rules.json"),
    ),
];

/// The contract file of a data-driven set, relative to its folder.
pub const CONTRACT_FILE: &str = "geometry-contract.json";

/// The built-in rules for a set id, if it has any.
pub fn builtin_rules(set_id: &str) -> Option<Result<RuleSet, ContractError>> {
    BUILTIN_RULES
        .iter()
        .find(|(id, _)| *id == set_id)
        .map(|(_, doc)| RuleSet::from_slice(doc.as_bytes()))
}

/// A set's clearance program.
#[derive(Debug, Clone)]
pub enum Clearance {
    /// Data rules: a geometry contract or built-in converted rules.
    Rules(Box<RuleSet>),
    /// A frozen hand-coded check.
    Legacy(LegacySet),
}

impl Clearance {
    /// Finds a set's program: `SET/geometry-contract.json`, then the built-in rules, then a
    /// frozen legacy check. `None` means the set has no clearance stage.
    pub fn for_set(set_dir: &Path, set_id: &str) -> Result<Option<Clearance>, ContractError> {
        let contract = set_dir.join(CONTRACT_FILE);
        if contract.is_file() {
            let bytes = std::fs::read(&contract).map_err(|source| ContractError::Io {
                path: contract.clone(),
                source,
            })?;
            return RuleSet::from_slice(&bytes).map(|r| Some(Clearance::Rules(Box::new(r))));
        }
        if let Some(rules) = builtin_rules(set_id) {
            return rules.map(|r| Some(Clearance::Rules(Box::new(r))));
        }
        Ok(LegacySet::for_set(set_id).map(Clearance::Legacy))
    }

    /// Runs the program over the set's actions. `Err` means the run aborted (a thrown
    /// error or an assert-mode failure) and no evidence should be written.
    pub fn run(
        &self,
        manifest: &SetManifest,
        scenes: &dyn SceneSource,
    ) -> Result<ClearanceReport, ContractError> {
        match self {
            Clearance::Rules(rules) => engine::run(rules, manifest, scenes),
            Clearance::Legacy(set) => set.run(manifest, scenes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_parse_and_dispatch() {
        for (id, _) in BUILTIN_RULES {
            let rules = builtin_rules(id).unwrap().unwrap();
            assert_eq!(rules.set.as_deref(), Some(*id));
            assert!(!rules.icons.is_empty());
        }
        let nowhere = Path::new("/nonexistent");
        assert!(matches!(
            Clearance::for_set(nowhere, "18-motion-design"),
            Ok(Some(Clearance::Rules(_)))
        ));
        assert!(matches!(
            Clearance::for_set(nowhere, "21-annotation-arrows"),
            Ok(Some(Clearance::Legacy(LegacySet::AnnotationArrows)))
        ));
        assert!(matches!(
            Clearance::for_set(nowhere, "40-notifications"),
            Ok(None)
        ));
    }
}
