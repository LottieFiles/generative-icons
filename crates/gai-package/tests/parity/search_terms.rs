//! Every library icon has search terms (`motion/icon-sets/<set>/search-terms.json`), which
//! the packages ship as its descriptor's `aliases`. A new icon needs them too.

use crate::common::repo;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;

#[test]
fn every_library_icon_has_search_terms() {
    let Some(repo) = repo() else { return };
    let catalog = gai_package::Catalog::load(&repo).unwrap();
    let (mut missing, mut problems) = (Vec::new(), Vec::new());
    for relative in &catalog.sets {
        let record: Value = serde_json::from_slice(&fs::read(repo.join("prototype/icons").join(relative)).unwrap()).unwrap();
        let set = record["id"].as_str().unwrap();
        let path = repo.join("motion/icon-sets").join(set).join("search-terms.json");
        let terms: Map<String, Value> =
            if path.is_file() { serde_json::from_slice(&fs::read(&path).unwrap()).unwrap() } else { Map::new() };
        for icon in record["icons"].as_array().unwrap() {
            let id = icon.get("icon").unwrap_or(&icon["name"]).as_str().unwrap();
            let Some(words) = terms.get(id).and_then(Value::as_array) else {
                missing.push(format!("{set}/{id}"));
                continue;
            };
            let words: Vec<&str> = words.iter().filter_map(Value::as_str).collect();
            let own = [icon["label"].as_str().unwrap().to_lowercase(), id.replace('-', " ")];
            if words.len() < 3 {
                problems.push(format!("{id}: {} terms; give it at least 3", words.len()));
            }
            if words.iter().collect::<HashSet<_>>().len() != words.len() {
                problems.push(format!("{id}: a term repeats"));
            }
            if let Some(word) = words.iter().find(|w| own.iter().any(|o| o == *w)) {
                problems.push(format!("{id}: \"{word}\" is its own label or id, which search already matches"));
            }
        }
    }
    assert!(missing.is_empty(), "icons without search terms: {missing:?}");
    assert!(problems.is_empty(), "{problems:#?}");
}
