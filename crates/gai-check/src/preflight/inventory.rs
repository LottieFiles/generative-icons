//! The source inventory of `checkSet` (`preflight.mjs:146-151`): the files a set's report
//! depends on, hashed but never parsed.

use super::{
    catalog::Input,
    error::{Error, Result},
    paths,
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

/// `definitions.mjs`, `set-config.json`, every `*.mjs`/`*.json` directly in `library_dir`
/// and in `set_dir/batches`, those that exist, in JS sort order over the whole paths. So a
/// set outside the repo may list the library files before its own.
pub(super) fn sources(set_dir: &Path, library_dir: &Path) -> Result<Vec<Input>> {
    let mut candidates = vec![paths::join(set_dir, "definitions.mjs"), paths::join(set_dir, "set-config.json")];
    candidates.extend(sources_in(library_dir)?);
    let batches = paths::join(set_dir, "batches");
    if batches.exists() {
        candidates.extend(sources_in(&batches)?);
    }
    let mut unique = HashSet::new();
    let mut files: Vec<String> = candidates
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| unique.insert(path.clone()) && Path::new(path).exists())
        .collect();
    files.sort_by(|a, b| paths::js_cmp(a, b));
    files.iter().map(|file| Input::read(Path::new(file)).map(|(input, _)| input)).collect()
}

/// Entries of `dir` whose name matches `/\.(mjs|json)$/`, dotfiles included.
fn sources_in(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).map_err(Error::io(dir))? {
        let name = entry.map_err(Error::io(dir))?.file_name().to_string_lossy().into_owned();
        if name.ends_with(".mjs") || name.ends_with(".json") {
            found.push(paths::join(dir, name));
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_hashes_existing_sources_in_js_order() {
        let temp = tempfile::tempdir().unwrap();
        let (set, library) = (temp.path().join("NN-set"), temp.path().join("library"));
        fs::create_dir_all(set.join("batches")).unwrap();
        fs::create_dir_all(&library).unwrap();
        for (path, text) in [
            (set.join("definitions.mjs"), ""),
            (set.join("batches/b.mjs"), "b"),
            (set.join("batches/.gitkeep"), ""),
            (set.join("batches/.x.json"), "x"),
            (library.join("kit.mjs"), "k"),
            (library.join("README.md"), ""),
        ] {
            fs::write(path, text).unwrap();
        }
        let inputs = sources(&set, &library).unwrap();
        let root = temp.path().to_string_lossy().into_owned();
        let listed: Vec<_> = inputs.iter().map(|input| input.path.strip_prefix(&root).unwrap()).collect();
        // No set-config.json; dotfiles match; "NN-set" sorts before "library".
        assert_eq!(
            listed,
            ["/NN-set/batches/.x.json", "/NN-set/batches/b.mjs", "/NN-set/definitions.mjs", "/library/kit.mjs"]
        );
        assert_eq!(inputs[0].sha256, gai_core::sha256_hex("x"));
    }
}
