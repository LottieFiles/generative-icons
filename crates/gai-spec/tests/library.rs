//! Every library icon reanimated as a spec (`motion/icon-sets/*/specs/*.icon.json`) still
//! compiles: they are the worked examples `create --spec --from` starts new icons from.

use std::path::Path;

#[test]
fn every_library_spec_compiles() {
    let sets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets");
    let mut count = 0;
    let mut failed = Vec::new();
    for set in std::fs::read_dir(&sets).unwrap().flatten() {
        let Ok(specs) = std::fs::read_dir(set.path().join("specs")) else { continue };
        for spec in specs.flatten() {
            let text = std::fs::read_to_string(spec.path()).unwrap();
            count += 1;
            if let Err(d) = gai_spec::load(&text) {
                let codes: Vec<&str> = d.errors().map(|e| e.code).collect();
                failed.push(format!("{}: {codes:?}", spec.file_name().to_string_lossy()));
            }
        }
    }
    assert!(count >= 400, "{count} library specs");
    assert!(failed.is_empty(), "{failed:#?}");
}

/// None of the library's specs, every one reviewed as an event, is a nudge: `NUDGE` was
/// measured on them, and a new library icon that warns needs a real event first.
#[test]
fn no_library_spec_is_a_nudge() {
    let sets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets");
    let mut nudges = Vec::new();
    for set in std::fs::read_dir(&sets).unwrap().flatten() {
        let Ok(specs) = std::fs::read_dir(set.path().join("specs")) else { continue };
        for spec in specs.flatten() {
            let text = std::fs::read_to_string(spec.path()).unwrap();
            if let Ok(c) = gai_spec::load(&text)
                && c.diagnostics.0.iter().any(|d| d.code == "NUDGE")
            {
                nudges.push(spec.file_name().to_string_lossy().into_owned());
            }
        }
    }
    assert!(nudges.is_empty(), "{nudges:#?}");
}

/// No library spec swaps its subject for a copy of itself (`DUPLICATE_DRAWING`): the family
/// draws the subject once and transforms it, so a library icon that warns needs its motion
/// redone on the drawing it has.
#[test]
fn no_library_spec_swaps_in_a_copy() {
    let sets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets");
    let mut swaps = Vec::new();
    for set in std::fs::read_dir(&sets).unwrap().flatten() {
        let Ok(specs) = std::fs::read_dir(set.path().join("specs")) else { continue };
        for spec in specs.flatten() {
            let text = std::fs::read_to_string(spec.path()).unwrap();
            if let Ok(c) = gai_spec::load(&text) {
                for d in c.diagnostics.0.iter().filter(|d| d.code == "DUPLICATE_DRAWING") {
                    swaps.push(format!("{}: {}", spec.file_name().to_string_lossy(), d.message));
                }
            }
        }
    }
    swaps.sort();
    assert!(swaps.is_empty(), "{swaps:#?}");
}
