//! `search`: ranked (the default) or all-words search over the library and this
//! project's own installed icons.
//!
//! Ranked search expands common synonyms ([`crate::search::SYNONYMS`]) and drops weak
//! matches (see [`relevant`]); `context`'s candidates keep the pinned ranking.

use crate::Context;
use crate::cli::{Match, SearchArgs};
use crate::config::Config;
use crate::descriptor::read_installed;
use crate::error::{CliError, Result};
use crate::lock::Lock;
use crate::output::Report;
use crate::search::{Ranked, Searchable, all_words, ranked_with_synonyms, synonyms, tokens};
use serde_json::{Map, Value, json};
use std::fmt::Write;

/// Results shown by ranked search without `--limit`.
const DEFAULT_LIMIT: usize = 20;

/// A result: the entry's index, with its score and matched terms when ranked.
type Hit = (usize, Option<(u32, Vec<String>)>);

/// One searchable icon and where it comes from.
struct Entry {
    id: String,
    label: String,
    category: Option<String>,
    intent: Option<String>,
    brief: Option<String>,
    aliases: Vec<String>,
    stateful: bool,
    origin: &'static str,
    /// The library set it was built in (`source.set`), which decides whether `create
    /// --from` can start from it.
    set: Option<String>,
}

impl Entry {
    fn searchable(&self) -> Searchable<'_> {
        Searchable {
            id: &self.id,
            label: &self.label,
            category: self.category.as_deref(),
            intent: self.intent.as_deref(),
            brief: self.brief.as_deref(),
            aliases: &self.aliases,
        }
    }

    /// How directly the query names this icon, best first: 0 when it is the id (as words
    /// or kebab-case) or the label; 1 when the id's or label's words appear together in it
    /// and every other word is a synonym of theirs ("notification bell" names `bell`); else 2.
    fn naming(&self, query: &str) -> u8 {
        let lower = query.trim().to_lowercase();
        let words = ascii_words(&lower);
        if words.join("-") == self.id || lower == self.label.to_lowercase() {
            return 0;
        }
        let label = self.label.to_lowercase();
        let named = [self.id.split('-').collect::<Vec<_>>(), ascii_words(&label)].into_iter().any(|name| {
            let inside = words.windows(name.len().max(1)).any(|run| run == name.as_slice());
            inside && tokens(query).iter().all(|token| name.iter().any(|word| word == token || synonyms(token, word)))
        });
        if named { 1 } else { 2 }
    }

    fn to_json(&self, score: Option<&(u32, Vec<String>)>) -> Value {
        let mut item = Map::new();
        item.insert("id".into(), json!(self.id));
        item.insert("label".into(), json!(self.label));
        if let Some(category) = &self.category {
            item.insert("category".into(), json!(category));
        }
        item.insert("stateful".into(), json!(self.stateful));
        item.insert("origin".into(), json!(self.origin));
        if let Some(set) = &self.set {
            item.insert("set".into(), json!(set));
        }
        if let Some((score, terms)) = score {
            item.insert("score".into(), json!(score));
            item.insert("matchedTerms".into(), json!(terms));
        }
        Value::Object(item)
    }
}

/// Runs `search`.
pub fn run(ctx: &Context, args: &SearchArgs) -> Result<Report> {
    let query = args.words.join(" ");
    if args.mode == Match::Ranked && query.trim().is_empty() {
        return Err(CliError::usage(
            "Describe the icon to search for, e.g. gen-animated-icons search \"notification bell\"",
        ));
    }
    let mut warnings = Vec::new();
    let mut entries = Vec::new();
    // Icons reanimated as specs say so: `create --from` starts from one.
    let mut specs = std::collections::BTreeSet::new();
    if !args.local_only {
        let library = ctx.library()?;
        specs = library.spec_ids().unwrap_or_else(|error| {
            warnings.push(format!("Cannot tell which icons have specs: {}", error.message));
            Default::default()
        });
        let catalog = library.catalog()?;
        let index = library.search_index(&catalog)?;
        entries.extend(catalog.icons.into_iter().map(|icon| Entry {
            brief: index.brief(&icon.id).map(str::to_string),
            set: icon.descriptor.pointer("/source/set").and_then(Value::as_str).map(str::to_string),
            id: icon.id,
            label: icon.label,
            category: icon.category,
            intent: icon.intent,
            aliases: icon.aliases,
            stateful: icon.stateful,
            origin: "library",
        }));
    }
    if !args.library_only {
        entries.extend(local_entries(ctx, args.local_only, &mut warnings)?);
    }

    let searchables: Vec<Searchable<'_>> = entries.iter().map(Entry::searchable).collect();
    let hits: Vec<Hit> = match args.mode {
        Match::Ranked => ranked_hits(&entries, &query, args.limit.map_or(DEFAULT_LIMIT, |n| n as usize)),
        Match::AllWords => {
            let limit = args.limit.map_or(usize::MAX, |n| n as usize);
            all_words(&searchables, &query).into_iter().take(limit).map(|index| (index, None)).collect()
        }
    };

    let value = Value::Array(
        hits.iter()
            .map(|(index, score)| {
                let entry = &entries[*index];
                let mut item = entry.to_json(score.as_ref());
                if entry.origin == "library" && specs.contains(&entry.id) {
                    item["spec"] = json!(true);
                }
                item
            })
            .collect(),
    );
    let mut text = String::new();
    if hits.is_empty() {
        let _ = writeln!(text, "No icons match \"{}\". Try a synonym or the underlying object.", query.trim());
    }
    for (index, score) in &hits {
        let entry = &entries[*index];
        let score = score.as_ref().map(|(score, _)| *score);
        let line = result_line(&entry.id, &entry.label, entry.category.as_deref(), entry.origin == "local", score);
        let _ = writeln!(text, "{line}");
    }
    if args.mode == Match::Ranked && tokens(&query).is_empty() {
        warnings.push("The query has only common words; describe the meaning, e.g. \"bell\" or \"payment\"".into());
    }
    Ok(Report { warnings, ..Report::new(value, text) })
}

/// The ranked results for `query`, at most `limit`: the ranking with synonyms, without
/// weak matches ([`relevant`]), icons the query names first (their scores raised to match).
fn ranked_hits(entries: &[Entry], query: &str, limit: usize) -> Vec<Hit> {
    let searchables: Vec<Searchable<'_>> = entries.iter().map(Entry::searchable).collect();
    let mut hits = relevant(entries, ranked_with_synonyms(&searchables, query, usize::MAX), query);
    // Icons the query names come first, whatever their score (a stable sort, so scores
    // still order each group); their scores are raised to match.
    hits.sort_by_key(|hit| entries[hit.index].naming(query));
    hits.truncate(limit);
    let groups: Vec<u8> = hits.iter().map(|hit| entries[hit.index].naming(query)).collect();
    let ids: Vec<&str> = hits.iter().map(|hit| entries[hit.index].id.as_str()).collect();
    let mut scores: Vec<u32> = hits.iter().map(|hit| hit.score).collect();
    keep_scores_in_order(&mut scores, &groups, &ids);
    hits.into_iter().zip(scores).map(|(hit, score)| (hit.index, Some((score, hit.matched_terms)))).collect()
}

/// The hits worth showing. An icon the query names, or one whose id, label, category or
/// keywords hold a query word or a synonym of one, always is. One that matches only in its
/// description (intent or brief) must match every query word there, directly or by a
/// synonym, and score at least a quarter of the best score (and 2, unless nothing scores
/// more): a word or synonym in a description beside icons named for the query is padding.
fn relevant(entries: &[Entry], hits: Vec<Ranked>, query: &str) -> Vec<Ranked> {
    let top = hits.iter().map(|hit| hit.score).max().unwrap_or(0);
    let floor = top.div_ceil(4).max(top.min(2));
    let typed = tokens(query);
    hits.into_iter()
        .filter(|hit| {
            let entry = &entries[hit.index];
            let category = entry.category.as_deref().unwrap_or_default();
            let shown = tokens(&format!("{} {} {category} {}", entry.id, entry.label, entry.aliases.join(" ")));
            if entry.naming(query) < 2 || hit.matched_terms.iter().any(|term| shown.contains(term)) {
                return true;
            }
            let covered = |word: &String| hit.matched_terms.iter().any(|term| term == word || synonyms(word, term));
            typed.iter().all(covered) && hit.score >= floor
        })
        .collect()
}

/// One result as text: `<id> <label> · <category> (local)  [<score>]`, the id padded to
/// 28 columns.
pub(crate) fn result_line(id: &str, label: &str, category: Option<&str>, local: bool, score: Option<u32>) -> String {
    let category = category.map(|c| format!(" · {c}")).unwrap_or_default();
    let local = if local { " (local)" } else { "" };
    let score = score.map(|score| format!("  [{score}]")).unwrap_or_default();
    format!("{id:<28} {label}{category}{local}{score}")
}

/// Raises the `scores` of each run of equal `groups` (best first, each run's scores
/// descending) by one amount per run where needed, so that ordering by score, then by id
/// (the ranking's tie-break), gives the ranked order: a run's last icon must score above
/// the next icon, or the same with an earlier id.
fn keep_scores_in_order(scores: &mut [u32], groups: &[u8], ids: &[&str]) {
    let mut end = scores.len();
    while end > 0 {
        let start = (0..end).rev().take_while(|&i| groups[i] == groups[end - 1]).last().unwrap_or(end - 1);
        // The runs after this one are in order already, so the next icon scores highest.
        if let Some(&next) = scores.get(end) {
            let last = end - 1;
            if scores[last] < next || (scores[last] == next && ids[last] > ids[end]) {
                let raise = next + 1 - scores[last];
                scores[start..end].iter_mut().for_each(|score| *score += raise);
            }
        }
        end = start;
    }
}

/// The ASCII `[a-z0-9]+` runs of lowercased text.
fn ascii_words(lower: &str) -> Vec<&str> {
    lower.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit())).filter(|w| !w.is_empty()).collect()
}

/// This project's own icons (`origin: "local"` in the lock), read from their descriptors.
fn local_entries(ctx: &Context, required: bool, warnings: &mut Vec<String>) -> Result<Vec<Entry>> {
    let config = match required {
        true => Some(Config::require(&ctx.cwd, ctx.platform)?),
        false => Config::load(&ctx.cwd, ctx.platform)?,
    };
    let Some(config) = config else { return Ok(Vec::new()) };
    let lock = Lock::load(&ctx.cwd, config.layout)?;
    let mut entries = Vec::new();
    for (id, record) in lock.icons() {
        if record.get("origin").and_then(Value::as_str) != Some("local") {
            continue;
        }
        let Some(source) = record.get("source").and_then(Value::as_str) else { continue };
        let descriptor = match read_installed(&ctx.cwd, source) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                warnings.push(format!("Skipped local icon {id}: {}", error.message));
                continue;
            }
        };
        let text = |key: &str| descriptor.get(key).and_then(Value::as_str).map(str::to_string);
        entries.push(Entry {
            id: id.to_string(),
            label: text("label").unwrap_or_else(|| id.to_string()),
            category: text("category"),
            intent: text("intent"),
            brief: None,
            aliases: descriptor
                .get("aliases")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default(),
            stateful: descriptor.get("stateful").and_then(Value::as_bool).unwrap_or(false),
            origin: "local",
            set: None,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, label: &str) -> Entry {
        Entry {
            id: id.into(),
            label: label.into(),
            category: None,
            intent: None,
            brief: None,
            aliases: Vec::new(),
            stateful: false,
            origin: "library",
            set: None,
        }
    }

    fn described(id: &str, label: &str, category: &str, intent: &str) -> Entry {
        Entry { category: Some(category.into()), intent: Some(intent.into()), ..entry(id, label) }
    }

    /// `(id, score)` of each result `run` shows for `query` among `entries`.
    fn shown(entries: &[Entry], query: &str) -> Vec<(String, u32)> {
        let hits = ranked_hits(entries, query, DEFAULT_LIMIT);
        hits.into_iter().map(|(index, score)| (entries[index].id.clone(), score.unwrap().0)).collect()
    }

    #[test]
    fn synonyms_find_the_icon_and_weak_matches_are_dropped() {
        let entries = [
            described("delete-message", "Delete message", "Chat", "The message folds away."),
            described("trash", "Trash", "Actions", "The lid snaps open."),
            described("checkpoint", "Checkpoint", "Agents", "A flag plants. Delete lifts the flag."),
            described("archive", "Archive", "Files", "The box closes; remove it later."),
            described("chat-bubble", "Bubble", "Chat", "A bubble pops."),
        ];
        // A synonym in the name finds trash; a typed word only in another icon's
        // description, and a synonym only in a description, are padding beside it.
        assert_eq!(shown(&entries, "delete"), [("delete-message".to_string(), 15), ("trash".into(), 5)]);
        // A category the query names is not padding (here with "message", a synonym of chat).
        assert_eq!(shown(&entries, "chat"), [("chat-bubble".to_string(), 15), ("delete-message".into(), 8)]);
        // Nor is a curated keyword.
        let keyword = Entry { aliases: vec!["bin".into()], ..described("bucket", "Bucket", "Tools", "It tips over.") };
        let entries = [described("trash", "Trash", "Actions", "The lid opens."), keyword];
        assert_eq!(shown(&entries, "trash"), [("trash".to_string(), 15), ("bucket".into(), 1)]);
        // Descriptions alone must match every word the query has, directly or by a synonym.
        let garden = [
            described("dark-mode", "Dark mode", "Preferences", "The rays grow out around the moon."),
            described("sunrise", "Sunrise", "Weather", "The rays grow in."),
            described("seed-packet", "Packet", "Garden", "A sprout pushes up and the leaves grow."),
        ];
        assert_eq!(shown(&garden, "plant grow"), [("seed-packet".to_string(), 4)]);
        // When descriptions are all there is, a typed word in one still counts.
        assert_eq!(shown(&garden, "moon"), [("dark-mode".to_string(), 3)]);
    }

    #[test]
    fn raised_scores_keep_the_ranked_order() {
        // Each group (how directly the query names the icon) keeps its own spacing.
        let ids = ["a", "b", "c", "d", "e", "f", "g"];
        let mut scores = vec![15, 12, 12, 19, 16, 16, 3];
        keep_scores_in_order(&mut scores, &[0, 0, 0, 2, 2, 2, 2], &ids);
        assert_eq!(scores, [23, 20, 20, 19, 16, 16, 3]);
        // A tie stays when the ids already order it, as the ranking breaks ties.
        let mut tied = vec![5, 5, 5];
        keep_scores_in_order(&mut tied, &[0, 1, 2], &["c", "b", "a"]);
        assert_eq!(tied, [7, 6, 5]);
        let mut ordered = vec![5, 5, 5];
        keep_scores_in_order(&mut ordered, &[0, 1, 2], &["a", "b", "c"]);
        assert_eq!(ordered, [5, 5, 5]);
        let mut sorted = vec![9, 9, 4, 1];
        keep_scores_in_order(&mut sorted, &[1, 1, 2, 2], &ids[..4]);
        assert_eq!(sorted, [9, 9, 4, 1]);
    }

    #[test]
    fn icons_the_query_names_rank_first() {
        let (bell, ring) = (entry("bell", "Bell"), entry("bell-ring", "Bell ring"));
        assert_eq!((bell.naming("Bell"), ring.naming("bell ring"), ring.naming("bell-ring")), (0, 0, 0));
        assert_eq!((bell.naming("notification bell"), bell.naming("an alert bell")), (1, 1));
        assert_eq!(ring.naming("notification bell"), 2);
        // Other words that are not synonyms leave the score in charge.
        let user = entry("user", "User");
        assert_eq!((user.naming("add user to team"), bell.naming("bell tower")), (2, 2));
        assert_eq!(entry("key", "Key").naming("Café İstanbul key"), 2);
    }
}
