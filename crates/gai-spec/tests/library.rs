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
