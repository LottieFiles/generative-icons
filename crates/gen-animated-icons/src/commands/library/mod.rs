//! `library …`: maintainer commands for the library checkout. They replace the per-set
//! `build.mjs`/`verify.mjs`/`package.py`/`measure-motion.mjs`, `preflight.mjs`,
//! `build-checked.mjs`, `icon-set.py`, `verify.html`, `build-packages.py` and
//! `pack-release.mjs`.
//!
//! The checkout is `--library-root` (or `GAI_LIBRARY_ROOT`), else the nearest folder
//! above the working one that holds `motion/icon-sets`. Sets are named by folder
//! (`63-voice-agents`), by number (`63`) or by path.

pub mod measure;
pub mod new_set;
pub mod register;
pub mod release;
pub mod sets;
pub mod visual;

use crate::Context;
use crate::cli::LibraryCommand;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use std::path::{Path, PathBuf};

/// Runs one `library` subcommand.
pub fn run(ctx: &Context, command: &LibraryCommand) -> Result<Report> {
    let repo = ctx.library_repo()?;
    match command {
        LibraryCommand::NewSet(args) => new_set::run(&repo, args),
        LibraryCommand::Preflight(args) => sets::preflight(&repo, args),
        LibraryCommand::Build(args) => sets::build(ctx, &repo, args),
        LibraryCommand::Verify(args) => sets::verify(&repo, args),
        LibraryCommand::VerifyVisual(args) => visual::run(ctx, &repo, args),
        LibraryCommand::Measure(args) => measure::run(&repo, args),
        LibraryCommand::Package(args) => sets::package(&repo, args),
        LibraryCommand::Packages(args) => release::packages(&repo, args),
        LibraryCommand::Register(args) => register::run(&repo, args),
        LibraryCommand::Release(args) => release::release(&repo, args),
    }
}

/// `motion/icon-sets` of the checkout.
pub fn sets_dir(repo: &Path) -> PathBuf {
    repo.join("motion/icon-sets")
}

/// The set folder name for a folder name, a two-digit number or a path to the folder.
pub fn resolve_set(repo: &Path, name: &str) -> Result<String> {
    let sets = sets_dir(repo);
    let folder = Path::new(name).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if is_set_folder(&folder) && sets.join(&folder).is_dir() {
        return Ok(folder);
    }
    if folder.len() == 2 && folder.bytes().all(|b| b.is_ascii_digit()) {
        let mut matches: Vec<String> =
            set_folders(repo)?.into_iter().filter(|set| set.starts_with(&format!("{folder}-"))).collect();
        if matches.len() == 1 {
            return Ok(matches.remove(0));
        }
    }
    Err(CliError::new(ErrorCode::UnknownIcon, format!("No set {name} in {}", sets.display()))
        .with_hint("Name a folder of motion/icon-sets, such as 63-voice-agents"))
}

/// `NN-…`.
fn is_set_folder(name: &str) -> bool {
    matches!(name.as_bytes(), [a, b, b'-', _, ..] if a.is_ascii_digit() && b.is_ascii_digit())
}

/// Every `NN-*` folder of `motion/icon-sets`, sorted.
pub fn set_folders(repo: &Path) -> Result<Vec<String>> {
    let sets = sets_dir(repo);
    let entries = std::fs::read_dir(&sets).map_err(|error| CliError::io("read", &sets, &error))?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_set_folder(name))
        .collect();
    names.sort();
    Ok(names)
}

/// The sets named on the command line, or with `all` every set `keep` accepts.
pub fn select(repo: &Path, names: &[String], all: bool, keep: impl Fn(&Path) -> bool) -> Result<Vec<String>> {
    match (all, names.is_empty()) {
        (true, true) => Ok(set_folders(repo)?.into_iter().filter(|set| keep(&sets_dir(repo).join(set))).collect()),
        (true, false) => Err(CliError::usage("Name sets or pass --all, not both")),
        (false, true) => Err(CliError::usage("Name at least one set, or pass --all")),
        (false, false) => names.iter().map(|name| resolve_set(repo, name)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_resolve_by_folder_number_or_path() {
        let dir = tempfile::tempdir().unwrap();
        for set in ["35-weather", "63-voice-agents", "library"] {
            std::fs::create_dir_all(dir.path().join("motion/icon-sets").join(set)).unwrap();
        }
        assert_eq!(resolve_set(dir.path(), "63").unwrap(), "63-voice-agents");
        assert_eq!(resolve_set(dir.path(), "motion/icon-sets/35-weather/").unwrap(), "35-weather");
        assert_eq!(resolve_set(dir.path(), "35-weather").unwrap(), "35-weather");
        assert_eq!(resolve_set(dir.path(), "library").unwrap_err().code, ErrorCode::UnknownIcon);
        assert_eq!(set_folders(dir.path()).unwrap(), ["35-weather", "63-voice-agents"]);
        let all = select(dir.path(), &[], true, |dir| dir.ends_with("63-voice-agents")).unwrap();
        assert_eq!(all, ["63-voice-agents"]);
    }
}
