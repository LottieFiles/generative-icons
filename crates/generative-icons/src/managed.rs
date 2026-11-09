//! Managed files: path validation, all-or-nothing batches that never overwrite user
//! edits, and the per-project process lock.
//!
//! Every change is validated before anything is written, identical content is skipped, new
//! files are created exclusively, and a failure rolls back in reverse order.

use crate::error::{CliError, ErrorCode, Result};
use gai_core::sha256_hex;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

/// Resolves a project-relative managed path, refusing traversal, absolute paths,
/// backslashes, drive or stream colons (`C:x`, which `PathBuf::push` lets replace the
/// root on Windows), Windows device names and symlinks at any existing component below
/// `root`.
pub fn inside(root: &Path, relative: &str) -> Result<PathBuf> {
    if malformed(relative, cfg!(windows)) {
        return Err(CliError::new(ErrorCode::InvalidPath, format!("Expected a project-relative path: {relative}"))
            .with_path(relative));
    }
    let mut part = root.to_path_buf();
    for name in relative.split('/') {
        part.push(name);
        // Any lstat error means "absent", as in the JS original.
        if fs::symlink_metadata(&part).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(CliError::new(
                ErrorCode::SymlinkPath,
                format!("Symlinks are not allowed in managed paths: {relative}"),
            )
            .with_path(relative));
        }
    }
    Ok(part)
}

/// Whether [`inside`] refuses `relative` as a path; `windows` also refuses Windows device
/// names.
fn malformed(relative: &str, windows: bool) -> bool {
    relative.is_empty()
        || relative.contains('\\')
        || relative.contains(':')
        || relative.starts_with('/')
        || Path::new(relative).is_absolute()
        || relative.split('/').any(|part| !is_plain_component(part) || (windows && is_windows_reserved(part)))
}

/// Whether `name` is a path [`inside`] accepts on every platform, with no control
/// characters: a name an archive can hold and any system can extract as written.
pub fn is_portable_path(name: &str) -> bool {
    !malformed(name, true) && !name.chars().any(char::is_control)
}

/// Exactly one normal path component: not empty, `.`, `..`, a root or a prefix.
fn is_plain_component(part: &str) -> bool {
    let mut components = Path::new(part).components();
    matches!((components.next(), components.next()), (Some(Component::Normal(name)), None) if name == part)
}

/// A name Windows does not store as given: a device (`CON`, `NUL`, `COM1`, … with any
/// extension) or one ending in a dot or space.
pub fn is_windows_reserved(part: &str) -> bool {
    let stem = part.split('.').next().unwrap_or(part).trim_end_matches(' ').to_ascii_uppercase();
    let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    device || part.ends_with('.') || part.ends_with(' ')
}

/// Creates the project folder `relative` (see [`inside`]) and returns it. The path is
/// checked again once the folders exist, so a symlink planted below `root` is refused.
pub fn create_dir_inside(root: &Path, relative: &str) -> Result<PathBuf> {
    let dir = inside(root, relative)?;
    fs::create_dir_all(&dir).map_err(|error| CliError::io("create", &dir, &error))?;
    let dir = inside(root, relative)?;
    if !fs::symlink_metadata(&dir).is_ok_and(|meta| meta.is_dir()) {
        return Err(CliError::new(ErrorCode::InvalidPath, format!("Expected a folder: {relative}")).with_path(relative));
    }
    Ok(dir)
}

/// Writes `bytes` to `relative` below `dir`, creating the folders between: the folders
/// are checked like [`create_dir_inside`], and the file is written by [`write_replacing`].
/// `dir` itself is the caller's to verify.
pub fn write_inside(dir: &Path, relative: &str, bytes: &[u8]) -> Result<PathBuf> {
    let (folder, name) = relative.rsplit_once('/').map_or((None, relative), |(folder, name)| (Some(folder), name));
    let plain =
        !name.contains(['\\', ':']) && is_plain_component(name) && !(cfg!(windows) && is_windows_reserved(name));
    if !plain {
        return Err(CliError::new(ErrorCode::InvalidPath, format!("Expected a project-relative path: {relative}"))
            .with_path(relative));
    }
    let folder = match folder {
        Some(folder) => create_dir_inside(dir, folder)?,
        None => dir.to_path_buf(),
    };
    let file = folder.join(name);
    write_replacing(&file, bytes)?;
    Ok(file)
}

/// Writes `bytes` to `file` through a new temporary sibling renamed over it: whatever is
/// at `file`, a symlink included, is replaced rather than written through, and readers
/// never see a partial file. The folder must exist. A new file gets 0666 minus the umask.
pub fn write_replacing(file: &Path, bytes: &[u8]) -> Result<()> {
    let written = temp_beside(file, None).and_then(|mut temp| {
        temp.write_all(bytes)?;
        temp.persist(file).map(drop).map_err(|error| error.error)
    });
    written.map_err(|error| CliError::io("write", file, &error))
}

/// Reads the regular file `relative` below `dir` (a path [`inside`] accepts, so no symlink
/// on the way), or `None` when it does not exist. A folder, FIFO, socket or device there is
/// refused with `INVALID_PATH` before it is opened, so a planted path can neither leave
/// `dir` nor block the read. `dir` itself is the caller's to verify.
pub fn read_inside(dir: &Path, relative: &str) -> Result<Option<Vec<u8>>> {
    let path = inside(dir, relative)?;
    let not_regular =
        || CliError::new(ErrorCode::InvalidPath, format!("{relative} is not a regular file")).with_path(relative);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Err(not_regular()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CliError::io("read", &path, &error)),
    }
    let mut file = File::open(&path).map_err(|error| CliError::io("read", &path, &error))?;
    // What was opened is checked too, as the file may have been replaced since.
    if !file.metadata().is_ok_and(|meta| meta.is_file()) {
        return Err(not_regular());
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| CliError::io("read", &path, &error))?;
    Ok(Some(bytes))
}

/// Reads a file, returning `None` when it does not exist.
pub fn read_optional(file: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(file) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CliError::io("read", file, &error)),
    }
}

/// One planned change to a managed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Create or replace the file with these bytes.
    Write(Vec<u8>),
    /// Delete the file if it exists.
    Delete,
}

/// An ordered set of changes keyed by project-relative path. Setting a path again
/// replaces its change and keeps its position, like assigning a JS object key.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    changes: IndexMap<String, Change>,
}

impl Batch {
    /// An empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Plans writing `bytes` to `relative`.
    pub fn write(&mut self, relative: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.changes.insert(relative.into(), Change::Write(bytes.into()));
    }

    /// Plans deleting `relative`.
    pub fn delete(&mut self, relative: impl Into<String>) {
        self.changes.insert(relative.into(), Change::Delete);
    }

    /// The planned writes in order, for recording their digests in the lock.
    pub fn writes(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.changes.iter().filter_map(|(path, change)| match change {
            Change::Write(bytes) => Some((path.as_str(), bytes.as_slice())),
            Change::Delete => None,
        })
    }

    /// Validates every change against the disk before anything is written.
    ///
    /// An existing file may be replaced or deleted only when its sha256 equals
    /// `expected[path]` (the digest this tool last wrote) or it already holds the new
    /// bytes. Anything else is `MODIFIED_FILE`, and nothing is written.
    pub fn plan(self, root: &Path, expected: &HashMap<String, String>) -> Result<Plan> {
        let mut items = Vec::with_capacity(self.changes.len());
        for (relative, change) in self.changes {
            let file = inside(root, &relative)?;
            let previous = read_optional(&file)?;
            if let Some(previous) = &previous {
                let identical = matches!(&change, Change::Write(bytes) if bytes == previous);
                if !identical && expected.get(&relative) != Some(&sha256_hex(previous)) {
                    return Err(CliError::new(
                        ErrorCode::ModifiedFile,
                        format!("Preserving modified file: {relative}"),
                    )
                    .with_path(relative)
                    .with_hint(
                        "Run generative-icons diff; keep your edit elsewhere or restore the file, then retry",
                    ));
                }
            }
            items.push(Item { relative, file, change, previous });
        }
        Ok(Plan { items })
    }
}

#[derive(Debug)]
struct Item {
    relative: String,
    file: PathBuf,
    change: Change,
    previous: Option<Vec<u8>>,
}

impl Item {
    fn is_noop(&self) -> bool {
        match &self.change {
            Change::Write(bytes) => self.previous.as_ref() == Some(bytes),
            Change::Delete => self.previous.is_none(),
        }
    }
}

/// A validated batch, ready to apply.
#[derive(Debug)]
pub struct Plan {
    items: Vec<Item>,
}

impl Plan {
    /// Paths that applying would change, in order.
    pub fn pending(&self) -> Vec<String> {
        self.items.iter().filter(|item| !item.is_noop()).map(|item| item.relative.clone()).collect()
    }

    /// Applies every change, rolling back on failure. Returns the changed paths.
    /// Directories created along the way are left in place.
    pub fn apply(self) -> Result<Vec<String>> {
        let mut done: Vec<&Item> = Vec::new();
        for item in self.items.iter().filter(|item| !item.is_noop()) {
            if let Err(error) = apply_item(item) {
                return Err(rollback(&done, error));
            }
            done.push(item);
        }
        Ok(done.iter().map(|item| item.relative.clone()).collect())
    }
}

fn apply_item(item: &Item) -> Result<()> {
    let parent = item.file.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| CliError::io("create", parent, &e))?;
    // Recheck in case another process changed the file since validation.
    if read_optional(&item.file)? != item.previous {
        return Err(changed(&item.relative));
    }
    let result = match (&item.change, &item.previous) {
        (Change::Write(bytes), None) => create_new(&item.file, bytes),
        (Change::Write(bytes), Some(_)) => replace(&item.file, bytes),
        (Change::Delete, _) => fs::remove_file(&item.file),
    };
    result.map_err(|error| match error.kind() {
        io::ErrorKind::AlreadyExists => changed(&item.relative),
        _ => CliError::io("write", &item.file, &error),
    })
}

fn changed(relative: &str) -> CliError {
    CliError::new(ErrorCode::FileChanged, format!("File changed during operation: {relative}"))
        .with_path(relative)
        .with_hint("Another process is editing this project; retry when it finishes")
}

fn rollback(done: &[&Item], error: CliError) -> CliError {
    let failures: Vec<String> = done
        .iter()
        .rev()
        .filter_map(|item| {
            let restored = match &item.previous {
                Some(previous) => replace(&item.file, previous),
                None => fs::remove_file(&item.file),
            };
            restored.err().map(|e| format!("{}: {e}", item.relative))
        })
        .collect();
    if failures.is_empty() {
        return error;
    }
    let message = format!("{}; rollback also failed for {}", error.message, failures.join(", "));
    CliError { message, ..error }
}

/// Creates a file that must not exist yet: written to a temporary sibling, then linked
/// into place without replacing anything.
fn create_new(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut temp = temp_beside(file, None)?;
    temp.write_all(bytes)?;
    temp.persist_noclobber(file).map(drop).map_err(|e| e.error)
}

/// Replaces an existing file atomically, keeping its permission bits.
fn replace(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let permissions = fs::metadata(file)?.permissions();
    let mut temp = temp_beside(file, Some(permissions))?;
    temp.write_all(bytes)?;
    temp.persist(file).map(drop).map_err(|e| e.error)
}

fn temp_beside(file: &Path, permissions: Option<fs::Permissions>) -> io::Result<tempfile::NamedTempFile> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let mut builder = tempfile::Builder::new();
    builder.prefix(".generative-icons-").suffix(".tmp");
    match permissions {
        Some(permissions) => {
            let temp = builder.tempfile_in(dir)?;
            fs::set_permissions(temp.path(), permissions)?;
            Ok(temp)
        }
        None => {
            // New files get the usual 0666 minus umask, as `writeFileSync` did.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(fs::Permissions::from_mode(0o666));
            }
            builder.tempfile_in(dir)
        }
    }
}

/// Removes now-empty directories from `start` up to (not including) `root`.
pub fn prune_empty_dirs(root: &Path, start: &Path) {
    let mut dir = start.to_path_buf();
    while dir.starts_with(root) && dir != root {
        // `remove_dir` refuses non-empty directories, which ends the walk.
        if fs::remove_dir(&dir).is_err() {
            break;
        }
        if !dir.pop() {
            break;
        }
    }
}

/// Exclusive advisory lock on `.generative-icons/.lock`, held for a whole command so
/// parallel agent runs in one project serialize. Released on drop.
#[derive(Debug)]
pub struct ProcessLock {
    _file: File,
}

impl ProcessLock {
    /// The lock file, relative to the project root.
    pub const PATH: &'static str = ".generative-icons/.lock";

    /// Waits for and takes the project lock. `waiting` runs once if another process holds it.
    pub fn acquire(root: &Path, waiting: impl FnOnce()) -> Result<Self> {
        let path = inside(root, Self::PATH)?;
        let parent = path.parent().unwrap_or(root);
        fs::create_dir_all(parent).map_err(|e| CliError::io("create", parent, &e))?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| CliError::io("open", &path, &e))?;
        match file.try_lock() {
            Ok(()) => {}
            Err(fs::TryLockError::WouldBlock) => {
                waiting();
                file.lock().map_err(|e| CliError::io("lock", &path, &e))?;
            }
            Err(fs::TryLockError::Error(error)) => return Err(CliError::io("lock", &path, &error)),
        }
        Ok(Self { _file: file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected(pairs: &[(&str, &[u8])]) -> HashMap<String, String> {
        pairs.iter().map(|(path, bytes)| (path.to_string(), sha256_hex(bytes))).collect()
    }

    #[test]
    fn inside_rejects_traversal_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        for bad in ["", "/abs", "a//b", "./a", "a/../b", "a\\b", "a/", "C:x", "a/C:..", "x:stream", "a/b:c"] {
            assert_eq!(inside(dir.path(), bad).unwrap_err().code, ErrorCode::InvalidPath, "{bad}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path(), dir.path().join("link")).unwrap();
            let error = inside(dir.path(), "link/file").unwrap_err();
            assert_eq!(error.code, ErrorCode::SymlinkPath);
            assert_eq!(error.message, "Symlinks are not allowed in managed paths: link/file");
        }
        assert_eq!(inside(dir.path(), "a/b.txt").unwrap(), dir.path().join("a/b.txt"));
    }

    #[test]
    fn portable_paths_hold_no_separator_or_control_character_of_any_system() {
        for good in ["a", "evidence/notes.md", "built/a/a-strong.lottie", "\u{e9}t\u{e9}.md"] {
            assert!(is_portable_path(good), "{good}");
        }
        for bad in ["", "/a", "a/../b", "a\\b", "c:x", "a/NUL.md", "name.", "a\nb", "a\tb", "a\u{7f}", "a\u{85}b"] {
            assert!(!is_portable_path(bad), "{bad:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn reads_inside_take_regular_files_only() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        assert_eq!(read_inside(dir.path(), "a.txt").unwrap(), Some(b"a".to_vec()));
        assert_eq!(read_inside(dir.path(), "gone.txt").unwrap(), None);
        fs::create_dir(dir.path().join("folder")).unwrap();
        assert_eq!(read_inside(dir.path(), "folder").unwrap_err().code, ErrorCode::InvalidPath);
        std::os::unix::fs::symlink("a.txt", dir.path().join("link")).unwrap();
        assert_eq!(read_inside(dir.path(), "link").unwrap_err().code, ErrorCode::SymlinkPath);
        assert_eq!(read_inside(dir.path(), "../a.txt").unwrap_err().code, ErrorCode::InvalidPath);
        let fifo = dir.path().join("fifo");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        assert_eq!(read_inside(dir.path(), "fifo").unwrap_err().code, ErrorCode::InvalidPath);
    }

    #[test]
    fn windows_device_names_are_recognized() {
        for reserved in ["CON", "nul", "Aux.txt", "com1", "LPT9.lottie", "conout$", "name.", "name "] {
            assert!(is_windows_reserved(reserved), "{reserved}");
        }
        for plain in ["console", "com", "com0", "lpt10", "nullable.mjs", "heart.lottie"] {
            assert!(!is_windows_reserved(plain), "{plain}");
        }
        assert_eq!(inside(Path::new("/p"), "a/NUL").is_err(), cfg!(windows));
    }

    #[test]
    fn batch_writes_skips_identical_and_reports_changes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("same.txt"), "x").unwrap();
        let mut batch = Batch::new();
        batch.write("new/a.txt", "a");
        batch.write("same.txt", "x");
        batch.write("new/a.txt", "b");
        let changed = batch.plan(dir.path(), &HashMap::new()).unwrap().apply().unwrap();
        assert_eq!(changed, ["new/a.txt"]);
        assert_eq!(fs::read_to_string(dir.path().join("new/a.txt")).unwrap(), "b");
    }

    #[test]
    fn modified_files_block_the_whole_batch() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("edited.txt"), "user edit").unwrap();
        let mut batch = Batch::new();
        batch.write("first.txt", "1");
        batch.write("edited.txt", "tool");
        let error = batch.plan(dir.path(), &expected(&[("edited.txt", b"tool v1")])).unwrap_err();
        assert_eq!(error.code, ErrorCode::ModifiedFile);
        assert_eq!(error.message, "Preserving modified file: edited.txt");
        assert!(!dir.path().join("first.txt").exists());
    }

    #[test]
    fn tracked_files_are_replaced_and_deleted() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("tracked.txt"), "v1").unwrap();
        fs::write(dir.path().join("gone.txt"), "old").unwrap();
        let mut batch = Batch::new();
        batch.write("tracked.txt", "v2");
        batch.delete("gone.txt");
        batch.delete("never-existed.txt");
        let plan = batch.plan(dir.path(), &expected(&[("tracked.txt", b"v1"), ("gone.txt", b"old")])).unwrap();
        assert_eq!(plan.pending(), ["tracked.txt", "gone.txt"]);
        assert_eq!(plan.apply().unwrap(), ["tracked.txt", "gone.txt"]);
        assert_eq!(fs::read_to_string(dir.path().join("tracked.txt")).unwrap(), "v2");
        assert!(!dir.path().join("gone.txt").exists());
    }

    #[test]
    fn a_race_rolls_back_earlier_writes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("keep.txt"), "v1").unwrap();
        let mut batch = Batch::new();
        batch.write("keep.txt", "v2");
        batch.write("created.txt", "new");
        batch.write("raced.txt", "tool");
        let plan = batch.plan(dir.path(), &expected(&[("keep.txt", b"v1")])).unwrap();
        fs::write(dir.path().join("raced.txt"), "another process").unwrap();
        let error = plan.apply().unwrap_err();
        assert_eq!(error.code, ErrorCode::FileChanged);
        assert_eq!(error.message, "File changed during operation: raced.txt");
        assert_eq!(fs::read_to_string(dir.path().join("keep.txt")).unwrap(), "v1");
        assert!(!dir.path().join("created.txt").exists());
        assert_eq!(fs::read_to_string(dir.path().join("raced.txt")).unwrap(), "another process");
    }

    #[cfg(unix)]
    #[test]
    fn replacing_keeps_permission_bits() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("mode.txt");
        fs::write(&file, "v1").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
        let mut batch = Batch::new();
        batch.write("mode.txt", "v2");
        batch.plan(dir.path(), &expected(&[("mode.txt", b"v1")])).unwrap().apply().unwrap();
        assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o640);
    }

    #[cfg(unix)]
    #[test]
    fn outputs_replace_a_planted_symlink_and_refuse_symlinked_folders() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::write(&victim, "keep").unwrap();
        let out = dir.path().join("out");
        fs::create_dir_all(out.join("sheets")).unwrap();
        std::os::unix::fs::symlink(&victim, out.join("sheets/a.png")).unwrap();
        let file = write_inside(&out, "sheets/a.png", b"png").unwrap();
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep");
        assert!(!fs::symlink_metadata(&file).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&file).unwrap(), b"png");

        std::os::unix::fs::symlink(dir.path(), out.join("linked")).unwrap();
        assert_eq!(write_inside(&out, "linked/victim", b"x").unwrap_err().code, ErrorCode::SymlinkPath);
        assert_eq!(create_dir_inside(&out, "linked/new").unwrap_err().code, ErrorCode::SymlinkPath);
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep");
        for bad in ["", "a/", "a/..", "x:y", "a\\b"] {
            assert_eq!(write_inside(&out, bad, b"x").unwrap_err().code, ErrorCode::InvalidPath, "{bad}");
        }
        assert_eq!(create_dir_inside(&out, "made/here").unwrap(), out.join("made/here"));
    }

    #[test]
    fn prune_stops_at_non_empty_directories() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        fs::write(dir.path().join("a/keep.txt"), "").unwrap();
        prune_empty_dirs(dir.path(), &dir.path().join("a/b/c"));
        assert!(!dir.path().join("a/b").exists());
        assert!(dir.path().join("a/keep.txt").exists());
    }
}
