//! Node's lexical `path.resolve` and `path.join` (symlinks are never resolved, so `/tmp`
//! stays `/tmp` on macOS), and JavaScript's default sort order.

use std::{
    cmp::Ordering,
    path::{Component, Path, PathBuf},
};

/// `path.resolve(arg)` against `cwd`: absolute and folded, without a trailing slash.
pub fn resolve(cwd: &Path, arg: &Path) -> PathBuf {
    normalize(&cwd.join(arg))
}

/// `path.join(base, relative)` for a relative `relative`.
pub fn join(base: &Path, relative: impl AsRef<Path>) -> PathBuf {
    normalize(&base.join(relative))
}

/// Folds `.` and `..` and collapses separators, as `path.normalize` does.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    let mut depth = 0usize;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => out.push(component),
            Component::CurDir => {}
            Component::ParentDir if depth > 0 => {
                out.pop();
                depth -= 1;
            }
            // `/..` is `/`; a relative path keeps its leading `..`.
            Component::ParentDir => {
                if !out.has_root() {
                    out.push("..");
                }
            }
            Component::Normal(name) => {
                out.push(name);
                depth += 1;
            }
        }
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

/// `path.basename(dir)` of a folded path.
pub fn basename(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

/// `Array.prototype.sort`'s default order: UTF-16 code units, not code points.
pub fn js_cmp(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_fold_like_node() {
        let cwd = Path::new("/work/motion");
        assert_eq!(resolve(cwd, Path::new("icon-sets/16-voice/")), Path::new("/work/motion/icon-sets/16-voice"));
        assert_eq!(resolve(cwd, Path::new("../x/./y//z")), Path::new("/work/x/y/z"));
        assert_eq!(resolve(cwd, Path::new("/a/../../b")), Path::new("/b"));
        assert_eq!(
            join(Path::new("/r/"), "prototype/icons/01/assets/set.json"),
            Path::new("/r/prototype/icons/01/assets/set.json")
        );
        assert_eq!(join(Path::new("/r/set"), "../library"), Path::new("/r/library"));
        assert_eq!(normalize(Path::new("../a/..")), Path::new(".."));
        assert_eq!(basename(Path::new("/r/icon-harness-pass-x")), "icon-harness-pass-x");
    }

    #[test]
    fn sort_order_is_utf16() {
        // U+FF5E is one code unit; U+1F600 is a surrogate pair starting 0xD83D.
        assert_eq!(js_cmp("\u{1F600}", "\u{FF5E}"), Ordering::Less);
        assert_eq!("\u{1F600}".cmp("\u{FF5E}"), Ordering::Greater);
        assert_eq!(js_cmp("/a/NN-x", "/a/library"), Ordering::Less);
    }
}
