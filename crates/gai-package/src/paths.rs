//! The pathlib listing semantics the Python packagers relied on: `glob` and `rglob` include
//! dotfiles and directories, `sorted()` compares paths component by component, and
//! `dict.fromkeys` keeps the first of each duplicate.
//!
//! Every listing here is sorted. Several originals used unsorted `list(glob(...))`, whose
//! order is the filesystem's; sorting makes those archives deterministic.

use crate::error::{PackageError, Result, invalid};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// `sorted(paths)` for pathlib paths: component-wise, byte order within a component
/// (`a/b` before `a-c`). Rust's `Path` ordering is exactly that.
pub(crate) fn sort_paths(paths: &mut [PathBuf]) {
    paths.sort();
}

/// `list(dict.fromkeys(paths))`.
pub(crate) fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

/// `sorted(dir.glob(pattern))`. Components may use `*` and `?`; a `**` component matches
/// the directory itself and every subdirectory (symlinked directories are not entered).
/// A missing `dir` yields nothing.
pub(crate) fn glob(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let parts: Vec<&str> = pattern.split('/').collect();
    let mut found = Vec::new();
    select(dir, &parts, &mut found)?;
    let mut found = dedupe_paths(found);
    sort_paths(&mut found);
    Ok(found)
}

/// `sorted(dir.rglob(pattern))`.
pub(crate) fn rglob(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    glob(dir, &format!("**/{pattern}"))
}

/// Every regular file below `dir` (`[p for p in dir.rglob('*') if p.is_file()]`), sorted.
pub(crate) fn files_below(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(rglob(dir, "*")?
        .into_iter()
        .filter(|p| p.is_file())
        .collect())
}

fn select(dir: &Path, parts: &[&str], found: &mut Vec<PathBuf>) -> Result<()> {
    let Some((first, rest)) = parts.split_first() else {
        return Ok(());
    };
    if !dir.is_dir() {
        return Ok(());
    }
    if *first == "**" {
        for directory in directories(dir)? {
            select(&directory, rest, found)?;
        }
    } else if first.contains(['*', '?']) {
        for path in children(dir)? {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if wildcard_match(first, &name) {
                if rest.is_empty() {
                    found.push(path);
                } else {
                    select(&path, rest, found)?;
                }
            }
        }
    } else {
        let path = dir.join(first);
        if rest.is_empty() {
            if path.exists() {
                found.push(path);
            }
        } else {
            select(&path, rest, found)?;
        }
    }
    Ok(())
}

fn children(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = std::fs::read_dir(dir).map_err(|e| PackageError::io(dir, e))?;
    entries
        .map(|entry| {
            entry
                .map(|e| e.path())
                .map_err(|e| PackageError::io(dir, e))
        })
        .collect()
}

/// `dir` and every directory below it, not entering symlinked directories.
fn directories(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut all = vec![dir.to_path_buf()];
    let mut index = 0;
    while index < all.len() {
        for child in children(&all[index].clone())? {
            let is_link = child
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false);
            if child.is_dir() && !is_link {
                all.push(child);
            }
        }
        index += 1;
    }
    Ok(all)
}

/// `fnmatch` with `*` and `?`, case-sensitive (pathlib on POSIX).
fn wildcard_match(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    let (mut pi, mut ni) = (0, 0);
    let mut backtrack: Option<(usize, usize)> = None;
    while ni < n.len() {
        match p.get(pi) {
            Some('*') => {
                backtrack = Some((pi, ni));
                pi += 1;
            }
            Some(c) if *c == '?' || *c == n[ni] => {
                pi += 1;
                ni += 1;
            }
            _ => match backtrack {
                Some((star, matched)) => {
                    pi = star + 1;
                    ni = matched + 1;
                    backtrack = Some((star, matched + 1));
                }
                None => return false,
            },
        }
    }
    p[pi..].iter().all(|c| *c == '*')
}

/// `PurePath.suffix` of a file name: from the last dot, unless the dot leads or ends the name.
pub(crate) fn py_suffix(name: &str) -> &str {
    match name.rfind('.') {
        Some(i) if i > 0 && i < name.len() - 1 => &name[i..],
        _ => "",
    }
}

/// `PurePath.with_suffix(suffix)`.
pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = &name[..name.len() - py_suffix(&name).len()];
    path.with_file_name(format!("{stem}{suffix}"))
}

/// `path.relative_to(base)` as a `/`-separated string.
pub(crate) fn relative(path: &Path, base: &Path) -> Result<String> {
    let rest = path.strip_prefix(base).map_err(|_| {
        invalid(format!(
            "{} is not inside {}",
            path.display(),
            base.display()
        ))
    })?;
    let parts: Vec<String> = rest
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_by_component() {
        let mut paths = vec![
            PathBuf::from("a-c"),
            PathBuf::from("a/b"),
            PathBuf::from("a"),
        ];
        sort_paths(&mut paths);
        assert_eq!(
            paths,
            [
                PathBuf::from("a"),
                PathBuf::from("a/b"),
                PathBuf::from("a-c")
            ]
        );
    }

    #[test]
    fn wildcards() {
        assert!(wildcard_match("*.mjs", "build.mjs"));
        assert!(wildcard_match("*", ".DS_Store"));
        assert!(wildcard_match(
            "*invocation*.json",
            "claude-invocation-2.json"
        ));
        assert!(wildcard_match("verify-*.mjs", "verify-.mjs"));
        assert!(!wildcard_match("*.mjs", "build.mjs.bak"));
        assert!(!wildcard_match("family-*.png", "family.png"));
        assert!(wildcard_match("a?c", "abc"));
    }

    #[test]
    fn suffixes_follow_pathlib() {
        assert_eq!(py_suffix("a.tar.gz"), ".gz");
        assert_eq!(py_suffix(".gitkeep"), "");
        assert_eq!(py_suffix("name."), "");
        assert_eq!(
            with_suffix(Path::new("x/review.json"), ".md"),
            PathBuf::from("x/review.md")
        );
        assert_eq!(
            with_suffix(Path::new("x/README"), ".md"),
            PathBuf::from("x/README.md")
        );
    }

    #[test]
    fn globs_include_dotfiles_and_recurse() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "b.md",
            ".hidden.md",
            "sub/c.md",
            "sub/deeper/a.md",
            "sub/x.txt",
        ] {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"").unwrap();
        }
        let names = |paths: Vec<PathBuf>| {
            paths
                .iter()
                .map(|p| relative(p, dir.path()).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(glob(dir.path(), "*.md").unwrap()),
            [".hidden.md", "b.md"]
        );
        assert_eq!(
            names(rglob(dir.path(), "*.md").unwrap()),
            [".hidden.md", "b.md", "sub/c.md", "sub/deeper/a.md"]
        );
        assert_eq!(names(glob(dir.path(), "s*/*.txt").unwrap()), ["sub/x.txt"]);
        assert!(glob(&dir.path().join("missing"), "*").unwrap().is_empty());
    }
}
