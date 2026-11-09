//! Icon search: the ranked lexical search of `generation-context.mjs`, the same ranking
//! with a wider vocabulary for the `search` command, and the all-words substring filter
//! (`search --match all-words`).
//!
//! [`ranked`] is pinned to `generation-context.mjs` (its [`ALIASES`] included), and so is
//! `context`, which ranks with it. [`ranked_with_synonyms`] also expands [`SYNONYMS`], for
//! `search` only.

use indexmap::IndexSet;

/// Words that never count as search terms.
const STOP_WORDS: [&str; 31] = [
    "a", "an", "and", "are", "as", "at", "be", "by", "can", "create", "draw", "for", "from", "icon", "icons", "in",
    "is", "it", "like", "more", "new", "of", "on", "or", "our", "please", "the", "this", "to", "use", "with",
];

/// Vocabulary expansions: a query term from a group adds the whole group. Pinned for
/// parity with `generation-context.mjs`.
const ALIASES: [&[&str]; 7] = [
    &["notification", "notifications", "alert", "bell"],
    &["search", "find", "magnifier", "lookup"],
    &["favorite", "favourite", "like", "heart", "star"],
    &["payment", "pay", "bank", "finance", "fintech", "money"],
    &["loading", "loader", "spinner", "progress", "waiting"],
    &["settings", "preferences", "sliders", "adjust", "filter"],
    &["delivery", "shipping", "parcel", "package", "truck"],
];

/// Common UI verbs and nouns `search` also expands, as [`ALIASES`] do. No word is in two
/// groups, or in a group of [`ALIASES`]; a group names one meaning, not related ones.
pub const SYNONYMS: [&[&str]; 41] = [
    &["delete", "remove", "trash", "bin", "erase", "discard"],
    &["close", "dismiss", "cancel", "exit"],
    &["add", "plus", "insert"],
    &["edit", "pencil", "modify"],
    &["copy", "duplicate", "clone"],
    &["refresh", "reload", "sync", "retry"],
    &["send", "submit"],
    &["link", "chain", "url", "hyperlink"],
    &["attach", "attachment", "paperclip"],
    &["expand", "fullscreen", "maximize", "enlarge"],
    &["collapse", "minimize", "shrink"],
    &["sort", "arrange"],
    &["play", "resume"],
    &["power", "shutdown"],
    &["check", "done", "tick", "success", "complete", "approve"],
    &["warning", "caution", "danger"],
    &["error", "failure", "failed"],
    &["info", "information", "about"],
    &["help", "question", "support"],
    &["lock", "padlock", "locked", "secure"],
    &["eye", "visible", "visibility", "reveal"],
    &["user", "person", "profile", "avatar", "contact"],
    &["mail", "email", "envelope", "inbox"],
    &["chat", "message", "messages", "comment", "conversation"],
    &["home", "house"],
    &["image", "photo", "picture", "gallery"],
    &["document", "file", "page", "doc", "paper"],
    &["folder", "directory"],
    &["calendar", "date", "schedule"],
    &["clock", "time"],
    &["location", "map", "place", "marker", "gps"],
    &["phone", "call", "telephone", "dial"],
    &["cart", "basket", "checkout", "shopping"],
    &["menu", "hamburger"],
    &["options", "ellipsis", "overflow", "kebab"],
    &["music", "song", "melody"],
    &["volume", "speaker", "sound", "audio"],
    &["video", "movie", "film"],
    &["microphone", "mic"],
    &["trophy", "award", "prize", "achievement", "medal"],
    &["plant", "sprout", "seedling", "sapling"],
];

/// The searchable text of one icon.
#[derive(Debug, Clone, Copy)]
pub struct Searchable<'a> {
    /// Icon id.
    pub id: &'a str,
    /// Display label.
    pub label: &'a str,
    /// Category label.
    pub category: Option<&'a str>,
    /// Motion intent.
    pub intent: Option<&'a str>,
    /// Design brief (ranked search only).
    pub brief: Option<&'a str>,
    /// Extra keywords.
    pub aliases: &'a [String],
}

/// A ranked match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranked {
    /// Index into the searched slice.
    pub index: usize,
    /// Sum over matched terms of (5 in id/label, else 1) × (3 if typed, else 1).
    pub score: u32,
    /// Matched terms in expansion order.
    pub matched_terms: Vec<String>,
}

/// Unique ASCII `[a-z0-9]+` runs of the lowercased text, minus stop words.
pub fn tokens(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut seen = IndexSet::new();
    for run in lower.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit())) {
        if !run.is_empty() {
            seen.insert(run);
        }
    }
    seen.into_iter().filter(|t| !STOP_WORDS.contains(t)).map(str::to_string).collect()
}

/// Whether `a` and `b` share a group of [`ALIASES`] or [`SYNONYMS`], as `notification` and
/// `bell` do, or `delete` and `trash`.
pub fn synonyms(a: &str, b: &str) -> bool {
    ALIASES.iter().chain(&SYNONYMS).any(|group| group.contains(&a) && group.contains(&b))
}

/// Query terms plus the `groups` they touch, in one pass that sees earlier additions.
fn expand<'a>(direct: &[String], groups: impl IntoIterator<Item = &'a &'a [&'a str]>) -> IndexSet<String> {
    let mut expanded: IndexSet<String> = direct.iter().cloned().collect();
    for group in groups {
        if group.iter().any(|term| expanded.contains(*term)) {
            expanded.extend(group.iter().map(|term| term.to_string()));
        }
    }
    expanded
}

/// Ranks icons against `query`: score descending, then id bytewise; at most `limit`.
pub fn ranked(icons: &[Searchable<'_>], query: &str, limit: usize) -> Vec<Ranked> {
    ranked_over(icons, query, limit, expand(&tokens(query), &ALIASES))
}

/// [`ranked`], with the query also expanded by [`SYNONYMS`] (after [`ALIASES`]): a synonym
/// counts as an expansion, never as a typed term.
pub fn ranked_with_synonyms(icons: &[Searchable<'_>], query: &str, limit: usize) -> Vec<Ranked> {
    ranked_over(icons, query, limit, expand(&tokens(query), ALIASES.iter().chain(&SYNONYMS)))
}

/// Ranks icons against the `expanded` terms of `query`.
fn ranked_over(icons: &[Searchable<'_>], query: &str, limit: usize, expanded: IndexSet<String>) -> Vec<Ranked> {
    let direct = tokens(query);
    let mut hits: Vec<Ranked> = icons
        .iter()
        .enumerate()
        .filter_map(|(index, icon)| {
            let names = tokens(&format!("{} {}", icon.id, icon.label));
            let mut other_text = vec![icon.category.unwrap_or(""), icon.intent.unwrap_or(""), icon.brief.unwrap_or("")];
            other_text.extend(icon.aliases.iter().map(String::as_str));
            let other = tokens(&other_text.join(" "));
            let matched_terms: Vec<String> =
                expanded.iter().filter(|t| names.contains(t) || other.contains(t)).cloned().collect();
            let score = matched_terms
                .iter()
                .map(|t| if names.contains(t) { 5 } else { 1 } * if direct.contains(t) { 3 } else { 1 })
                .sum();
            (score > 0).then_some(Ranked { index, score, matched_terms })
        })
        .collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| icons[a.index].id.cmp(icons[b.index].id)));
    hits.truncate(limit);
    hits
}

/// JavaScript `\s`: Unicode White_Space without U+0085, plus U+FEFF.
fn is_js_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// Indices of icons whose `[id, label, intent, category, ...aliases]` text contains
/// every query word, in catalog order.
pub fn all_words(icons: &[Searchable<'_>], query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    let words: Vec<&str> = query.split(is_js_space).filter(|w| !w.is_empty()).collect();
    icons
        .iter()
        .enumerate()
        .filter(|(_, icon)| {
            let mut parts = vec![icon.id, icon.label, icon.intent.unwrap_or(""), icon.category.unwrap_or("")];
            parts.extend(icon.aliases.iter().map(String::as_str));
            let haystack = parts.join(" ").to_lowercase();
            words.iter().all(|word| haystack.contains(word))
        })
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon<'a>(id: &'a str, label: &'a str, intent: &'a str) -> Searchable<'a> {
        Searchable { id, label, category: Some("Essentials"), intent: Some(intent), brief: None, aliases: &[] }
    }

    #[test]
    fn tokens_match_the_js_tokenizer() {
        assert_eq!(tokens("Create a NEW icon for the Payment-Page 2"), ["payment", "page", "2"]);
        // İ lowercases to i + U+0307 in both JS and Rust.
        assert_eq!(tokens("Café İstanbul key"), ["caf", "i", "stanbul", "key"]);
    }

    #[test]
    fn aliases_expand_in_one_ordered_pass() {
        let expanded: Vec<String> = expand(&tokens("heart"), &ALIASES).into_iter().collect();
        assert_eq!(expanded, ["heart", "favorite", "favourite", "like", "star"]);
        // "like" is a stop word, so it never expands on its own.
        assert!(expand(&tokens("like"), &ALIASES).is_empty());
    }

    #[test]
    fn ranking_weights_names_and_typed_terms() {
        let icons = [
            icon("bell-ring", "Bell ring", "Rings."),
            icon("heart", "Heart", "A notification beat."),
            icon("alarm", "Alarm", ""),
        ];
        let hits = ranked(&icons, "notification", 10);
        let summary: Vec<(&str, u32, Vec<String>)> =
            hits.iter().map(|h| (icons[h.index].id, h.score, h.matched_terms.clone())).collect();
        assert_eq!(
            summary,
            [("bell-ring", 5, vec!["bell".to_string()]), ("heart", 3, vec!["notification".to_string()])]
        );
    }

    #[test]
    fn synonyms_share_an_alias_group() {
        assert!(synonyms("notification", "bell") && synonyms("bell", "alert"));
        assert!(synonyms("delete", "trash") && synonyms("envelope", "mail"));
        assert!(!synonyms("bell", "heart") && !synonyms("rocket", "rocket"));
    }

    #[test]
    fn synonym_groups_are_disjoint_and_search_only() {
        let mut seen = std::collections::HashSet::new();
        for word in ALIASES.iter().chain(&SYNONYMS).flat_map(|group| group.iter()) {
            assert!(seen.insert(*word), "{word} is in two groups");
        }
        for word in SYNONYMS.iter().flat_map(|group| group.iter()) {
            assert_eq!(tokens(word), [word.to_string()], "{word} is not one search term");
        }
        let icons = [icon("trash", "Trash", "The lid opens."), icon("delete-message", "Delete message", "")];
        let ids = |hits: Vec<Ranked>| hits.iter().map(|hit| icons[hit.index].id).collect::<Vec<_>>();
        // The pinned ranking never sees them; search does, as expansions worth 1 or 5.
        assert_eq!(ids(ranked(&icons, "delete", 10)), ["delete-message"]);
        let hits = ranked_with_synonyms(&icons, "delete", 10);
        assert_eq!((hits[1].score, hits[1].matched_terms.as_slice()), (5, ["trash".to_string()].as_slice()));
        assert_eq!(ids(hits), ["delete-message", "trash"]);
    }

    #[test]
    fn all_words_is_a_substring_filter() {
        let icons = [icon("heart", "Heart", "beat"), icon("heart-rate", "Heart rate", "pulse")];
        // U+FEFF separates words in JS; U+0085 does not.
        assert_eq!(all_words(&icons, "  HEA\u{feff}rate "), [1]);
        assert!(all_words(&icons, "hea\u{85}rt").is_empty());
        assert_eq!(all_words(&icons, ""), [0, 1]);
    }
}
