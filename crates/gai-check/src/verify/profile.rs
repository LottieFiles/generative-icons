//! What each set runs: the frozen variant table for the catalog (verify.md §8A), and the
//! template plus `SET/verify.rules.json` for every other set.

use super::generic::Generic;
use super::oneoff::OneOff;
use super::rules::RuleDoc;
use super::{VerifyError, read};
use std::path::Path;

/// A set's own rules file, which takes precedence over the built-in rules.
pub const RULES_FILE: &str = "verify.rules.json";

/// Where a stage runs relative to the `directed-source-verification.json` write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Before it: a failure leaves the directed report unwritten (45, 49, 50, 53, 54,
    /// 60–63). A stage here may embed its results in the report.
    BeforeDirected,
    /// After it: the directed report is already written (42–44, 58, 12–39 clearance).
    AfterDirected,
}

/// Everything one set runs, in order.
#[derive(Debug, Clone)]
pub struct Profile {
    /// The generic-check variant.
    pub generic: Generic,
    /// Set asserts and drawing rules.
    pub rules: Option<RuleDoc>,
    /// Where the clearance stage runs, when the set has one.
    pub clearance: Stage,
    /// Fixed one-off checks after everything else.
    pub one_offs: Vec<OneOff>,
}

/// Built-in rules of the catalog sets whose `verify.mjs` carried custom checks.
const BUILTIN_RULES: &[(&str, &str)] = &[
    (
        "42-maps-navigation",
        include_str!("../../rules/verify/42-maps-navigation.rules.json"),
    ),
    (
        "43-photos-camera",
        include_str!("../../rules/verify/43-photos-camera.rules.json"),
    ),
    (
        "44-music-audio",
        include_str!("../../rules/verify/44-music-audio.rules.json"),
    ),
    (
        "45-chat-messaging",
        include_str!("../../rules/verify/45-chat-messaging.rules.json"),
    ),
    (
        "49-onboarding-tips",
        include_str!("../../rules/verify/49-onboarding-tips.rules.json"),
    ),
    (
        "50-preferences",
        include_str!("../../rules/verify/50-preferences.rules.json"),
    ),
    (
        "53-video-editing",
        include_str!("../../rules/verify/53-video-editing.rules.json"),
    ),
    (
        "54-gamification-rewards",
        include_str!("../../rules/verify/54-gamification-rewards.rules.json"),
    ),
    (
        "58-automotive",
        include_str!("../../rules/verify/58-automotive.rules.json"),
    ),
    (
        "60-models-providers",
        include_str!("../../rules/verify/60-models-providers.rules.json"),
    ),
    (
        "61-model-training",
        include_str!("../../rules/verify/61-model-training.rules.json"),
    ),
    (
        "62-vector-editing",
        include_str!("../../rules/verify/62-vector-editing.rules.json"),
    ),
    (
        "63-voice-agents",
        include_str!("../../rules/verify/63-voice-agents.rules.json"),
    ),
];

/// The built-in rules document of a catalog set, if it has one.
pub fn builtin_rules(set_id: &str) -> Option<&'static str> {
    BUILTIN_RULES
        .iter()
        .find(|(id, _)| *id == set_id)
        .map(|(_, doc)| *doc)
}

/// The frozen generic variant of a catalog set; `None` means the template.
fn frozen_generic(set_id: &str) -> Option<Generic> {
    let template = |ellipsis, builder_message| {
        Some(Generic::Template {
            ellipsis,
            builder_message,
        })
    };
    match set_id {
        "01-heart-bell" => Some(Generic::HeartBell),
        "02-search" => Some(Generic::Search),
        "03-essentials"
        | "04-actions"
        | "05-interface"
        | "06-tools"
        | "07-ai-conversation"
        | "08-ai-knowledge"
        | "09-ai-creation"
        | "10-ai-agents"
        | "11-feedback-status"
        | "13-fintech-finance"
        | "14-travel"
        | "15-commerce-delivery" => Some(Generic::Directed),
        "12-loading-indicators" => Some(Generic::Continuous),
        "16-voice" => template(false, "Stale voice builder"),
        "17-media-playback" => template(false, "Stale media builder"),
        "18-motion-design" => template(false, "Stale motion editor builder"),
        "19-agent-operations" => template(false, "Stale agent operations builder"),
        "20-arrows" | "21-annotation-arrows" | "22-teamwork" => {
            template(false, "Stale arrows builder")
        }
        "23-get-help" => template(false, "Stale get-help builder"),
        "24-file-transfer" => template(false, "Stale file-transfer builder"),
        "25-everyday-controls" => template(true, "Stale everyday-controls builder"),
        "26-account-privacy" => template(true, "Stale account-privacy builder"),
        "27-forms-input" => template(true, "Stale forms-input builder"),
        "28-tables-analytics" => template(true, "Stale tables-analytics builder"),
        "29-text-document-editing" => template(true, "Stale text-document-editing builder"),
        "30-accessibility-reading" => template(true, "Stale accessibility-reading builder"),
        "31-developer-version-control" => template(true, "Stale developer-version-control builder"),
        "32-infrastructure-devices" => template(true, "Stale infrastructure-devices builder"),
        "33-communication-social" => template(true, "Stale communication-social builder"),
        "34-scheduling-time" => template(true, "Stale scheduling-time builder"),
        _ => None,
    }
}

impl Profile {
    /// The profile of a set: the frozen table for catalog sets, the template otherwise.
    /// Rules come from `SET/verify.rules.json` when present, else from the built-in table.
    pub fn for_set(set_id: &str, set_dir: &Path) -> Result<Profile, VerifyError> {
        let own = set_dir.join(RULES_FILE);
        let rules = if own.is_file() {
            Some(RuleDoc::from_slice(&read(&own)?)?)
        } else {
            builtin_rules(set_id)
                .map(|doc| RuleDoc::from_slice(doc.as_bytes()))
                .transpose()?
        };
        if let Some(named) = rules
            .as_ref()
            .and_then(|r| r.set.as_deref())
            .filter(|s| *s != set_id)
        {
            return Err(VerifyError::Rules(format!(
                "rules are for {named}, not {set_id}"
            )));
        }
        let one_offs = match set_id {
            "10-ai-agents" => vec![OneOff::ApprovalClearance],
            "11-feedback-status" => vec![OneOff::Attachments],
            _ => Vec::new(),
        };
        Ok(Profile {
            generic: frozen_generic(set_id).unwrap_or(Generic::Template {
                ellipsis: true,
                builder_message: "Stale set builder",
            }),
            rules,
            clearance: if set_id == "45-chat-messaging" {
                Stage::BeforeDirected
            } else {
                Stage::AfterDirected
            },
            one_offs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_parse_and_name_their_set() {
        for (id, doc) in BUILTIN_RULES {
            let rules = RuleDoc::from_slice(doc.as_bytes()).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(rules.set.as_deref(), Some(*id));
        }
    }

    #[test]
    fn variants_follow_the_frozen_table() {
        let nowhere = Path::new("/nonexistent");
        let generic = |id: &str| Profile::for_set(id, nowhere).unwrap().generic;
        assert_eq!(generic("05-interface"), Generic::Directed);
        assert_eq!(generic("12-loading-indicators"), Generic::Continuous);
        assert_eq!(
            generic("20-arrows"),
            Generic::Template {
                ellipsis: false,
                builder_message: "Stale arrows builder"
            }
        );
        assert_eq!(
            generic("64-new"),
            Generic::Template {
                ellipsis: true,
                builder_message: "Stale set builder"
            }
        );
        assert_eq!(
            Profile::for_set("45-chat-messaging", nowhere)
                .unwrap()
                .clearance,
            Stage::BeforeDirected
        );
        assert!(
            Profile::for_set("40-notifications", nowhere)
                .unwrap()
                .rules
                .is_none()
        );
        assert_eq!(
            Profile::for_set("11-feedback-status", nowhere)
                .unwrap()
                .one_offs,
            [OneOff::Attachments]
        );
    }
}
