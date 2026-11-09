//! Source-tree state for `release.json`, and `pack_release` (`scripts/pack-release.mjs`),
//! which turns the built `packages/*/dist` into npm tarballs plus a release record.
//! Tarballs come from `npm pack`, the one npm dependency left, behind [`npm_pack`].

use crate::error::{PackageError, Result, invalid};
use crate::io::{field, read, read_json, write};
use gai_core::js::truthy;
use gai_core::{sha256_hex, to_js_json_pretty};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The packages in pack order.
const PACK_ORDER: [&str; 3] = ["icons", "runtime", "react"];

/// The commit and cleanliness of the source tree, as `release.json` records them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitState {
    /// `git rev-parse HEAD`.
    pub commit: String,
    /// `git status --porcelain` is non-empty (untracked files count).
    pub dirty: bool,
}

impl GitState {
    /// Reads the state of the repository at `repo`.
    pub fn read(repo: &Path) -> Result<Self> {
        let commit = git(repo, &["rev-parse", "HEAD"])?.trim().to_string();
        Ok(GitState {
            commit,
            dirty: is_dirty(repo)?,
        })
    }
}

fn is_dirty(repo: &Path) -> Result<bool> {
    Ok(!git(repo, &["status", "--porcelain"])?.trim().is_empty())
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let command = format!("git {}", args.join(" "));
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| PackageError::Command {
            command: command.clone(),
            detail: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(PackageError::Command {
            command,
            detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Options of `pack_release`.
#[derive(Debug, Clone, Default)]
pub struct PackReleaseOptions {
    /// Folder to create for the tarballs and `release.json`; defaults to `<repo>/dist/packages`.
    /// It must not exist yet.
    pub output: Option<PathBuf>,
    /// Pack a build from, or a tree with, uncommitted changes (status `prerelease-dev`).
    pub allow_dirty: bool,
}

/// One packed tarball.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedPackage {
    /// npm package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Tarball file name.
    pub file: String,
    /// Tarball sha256 (lowercase hex).
    pub sha256: String,
    /// Tarball size.
    pub bytes: u64,
}

/// What `pack_release` produced.
#[derive(Debug, Clone)]
pub struct PackReleaseReport {
    /// The created output folder.
    pub output: PathBuf,
    /// `release` or `prerelease-dev`.
    pub status: String,
    /// Tarballs in pack order.
    pub packages: Vec<PackedPackage>,
}

impl PackReleaseReport {
    /// The line `pack-release.mjs` printed.
    pub fn summary(&self) -> String {
        format!(
            "Packed {} packages into {} (status {}). Nothing was published.",
            self.packages.len(),
            self.output.display(),
            self.status
        )
    }
}

/// `npm pack` metadata for one package.
#[derive(Debug, Clone)]
pub struct NpmPack {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Tarball file name inside the destination.
    pub filename: String,
    /// Paths inside the tarball.
    pub files: Vec<String>,
}

/// Runs `npm pack --ignore-scripts --json --pack-destination <destination>` in `package_dir`.
pub fn npm_pack(package_dir: &Path, destination: &Path) -> Result<NpmPack> {
    let command = format!("npm pack ({})", package_dir.display());
    let output = Command::new("npm")
        .args(["pack", "--ignore-scripts", "--json", "--pack-destination"])
        .arg(destination)
        .current_dir(package_dir)
        .output()
        .map_err(|e| PackageError::Command {
            command: command.clone(),
            detail: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(PackageError::Command {
            command,
            detail: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    let parsed: Value =
        serde_json::from_slice(&output.stdout).map_err(|e| PackageError::Command {
            command: command.clone(),
            detail: e.to_string(),
        })?;
    let meta = parsed.get(0).ok_or_else(|| PackageError::Command {
        command: command.clone(),
        detail: "empty result".into(),
    })?;
    let text = |key: &str| {
        field(meta, key, "npm pack").map(|v| v.as_str().unwrap_or_default().to_string())
    };
    let files = meta
        .get("files")
        .and_then(Value::as_array)
        .map(|files| {
            files
                .iter()
                .filter_map(|f| f.get("path").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(NpmPack {
        name: text("name")?,
        version: text("version")?,
        filename: text("filename")?,
        files,
    })
}

/// Packs `icons`, `runtime` and `react` from their built `dist` folders into a new
/// output folder with a `release.json` (JS `JSON.stringify` format): the build's
/// `packages/icons/dist/release.json` with `status` and `sourceDirty` updated in place and
/// `packages` appended. Refuses dirty builds and trees unless `allow_dirty`. Never publishes.
pub fn pack_release(repo: &Path, options: &PackReleaseOptions) -> Result<PackReleaseReport> {
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| repo.join("dist/packages"));
    let output = std::path::absolute(&output).map_err(|e| PackageError::io(&output, e))?;
    let mut release = read_json(&repo.join("packages/icons/dist/release.json"))?;
    let dirty_now = is_dirty(repo)?;
    let source_dirty = release.get("sourceDirty").cloned().unwrap_or(Value::Null);
    let built_release = release.get("status").and_then(Value::as_str) == Some("release");
    if (truthy(Some(&source_dirty)) || dirty_now || !built_release) && !options.allow_dirty {
        return Err(PackageError::DirtyRelease);
    }
    if output.exists() {
        return Err(invalid(
            "Release output exists. Use a fresh directory to preserve the previous pack.",
        ));
    }
    let parent = output.parent().unwrap_or(Path::new("/"));
    std::fs::create_dir_all(parent).map_err(|e| PackageError::io(parent, e))?;
    // Removed on every exit; after the rename there is nothing left to remove.
    let stage = tempfile::Builder::new()
        .prefix(".pack-")
        .tempdir_in(parent)
        .map_err(|e| PackageError::io(parent, e))?;

    let mut packages = Vec::new();
    for name in PACK_ORDER {
        let meta = npm_pack(&repo.join("packages").join(name), stage.path())?;
        if let Some(private) = meta.files.iter().find(|path| is_private(path)) {
            return Err(invalid(format!(
                "Private file entered a package: {private}"
            )));
        }
        let bytes = read(&stage.path().join(&meta.filename))?;
        packages.push(PackedPackage {
            name: meta.name,
            version: meta.version,
            file: meta.filename,
            sha256: sha256_hex(&bytes),
            bytes: bytes.len() as u64,
        });
    }
    let status = if built_release && !dirty_now {
        "release"
    } else {
        "prerelease-dev"
    };
    let record = release
        .as_object_mut()
        .ok_or_else(|| invalid("release.json is not an object"))?;
    record.insert("status".into(), json!(status));
    record.insert(
        "sourceDirty".into(),
        if truthy(Some(&source_dirty)) {
            source_dirty
        } else {
            json!(dirty_now)
        },
    );
    let listed: Vec<Value> = packages
        .iter()
        .map(|p| json!({ "name": p.name, "version": p.version, "file": p.file, "sha256": p.sha256, "bytes": p.bytes }))
        .collect();
    record.insert("packages".into(), Value::Array(listed));
    write(
        &stage.path().join("release.json"),
        to_js_json_pretty(&release, 2) + "\n",
    )?;
    std::fs::rename(stage.path(), &output).map_err(|e| PackageError::io(&output, e))?;
    Ok(PackReleaseReport {
        output,
        status: status.to_string(),
        packages,
    })
}

/// `/(^|\/)(\.local|wip|unreleased|node_modules)(\/|$)/i`: a private folder anywhere in the path.
fn is_private(path: &str) -> bool {
    path.split('/').any(|part| {
        [".local", "wip", "unreleased", "node_modules"]
            .iter()
            .any(|p| part.eq_ignore_ascii_case(p))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_paths() {
        assert!(is_private("dist/WIP/x.mjs"));
        assert!(is_private(".local/notes.md"));
        assert!(is_private("a/node_modules"));
        assert!(!is_private("dist/wipe/x.mjs"));
        assert!(!is_private("dist/unreleased.md"));
    }

    fn git_repo(release: &Value) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write(
            &dir.path().join("packages/icons/dist/release.json"),
            release.to_string(),
        )
        .unwrap();
        std::fs::write(dir.path().join(".gitignore"), "dist/\n").unwrap();
        for args in [
            &["init", "-q"][..],
            &["add", "-A"],
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@example.com",
                "commit",
                "-qm",
                "x",
            ],
        ] {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(dir.path())
                    .status()
                    .unwrap()
                    .success()
            );
        }
        dir
    }

    #[test]
    fn refuses_dirty_builds() {
        let repo = git_repo(&json!({"status": "prerelease-dev", "sourceDirty": true}));
        let output = repo.path().join("out");
        let error = pack_release(
            repo.path(),
            &PackReleaseOptions {
                output: Some(output.clone()),
                allow_dirty: false,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("uncommitted"));
        assert!(!output.exists());
    }

    #[test]
    fn reads_git_state() {
        let repo = git_repo(&json!({}));
        let state = GitState::read(repo.path()).unwrap();
        assert_eq!(state.commit.len(), 40);
        assert!(!state.dirty);
        std::fs::write(repo.path().join("new.txt"), "").unwrap();
        assert!(
            GitState::read(repo.path()).unwrap().dirty,
            "untracked files make the tree dirty"
        );
    }
}
