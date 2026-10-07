//! `library packages` builds the npm packages' `dist` folders (`build-packages.py`) and
//! the search sidecar; `library release` does the same, then packs the tarballs
//! (`pack-release.mjs`). Nothing is ever published.
//!
//! The sidecar (`icons/dist/search-index.json`) is not in the icons package's npm
//! `files` list, so `release.json` and the tarballs are unchanged; the hosted library
//! serves it beside `catalog.json`, and local dists give it to `search` and `context`.

use crate::cli::{PackagesArgs, ReleaseArgs};
use crate::error::{CliError, ErrorCode, Result};
use crate::library::search_index;
use crate::output::Report;
use gai_package::{BuildPackagesOptions, PackReleaseOptions, PackageError, build_packages, pack_release};
use serde_json::json;
use std::path::{Path, PathBuf};

/// Where the sidecar goes below the packages folder.
const SIDECAR: &str = "icons/dist/search-index.json";

fn package_error(error: PackageError) -> CliError {
    match error {
        error @ PackageError::DirtyRelease => CliError::new(ErrorCode::CheckFailed, error.to_string())
            .with_hint("Commit the sources, or pass --allow-dirty for a prerelease"),
        other => CliError::new(ErrorCode::Io, other.to_string()),
    }
}

/// Builds every package `dist` and the sidecar; returns `(summary, packages folder)`.
fn build(repo: &Path, output: Option<&Path>) -> Result<(gai_package::PackagesReport, PathBuf)> {
    let output = output.map(Path::to_path_buf);
    let options = BuildPackagesOptions { output: output.clone(), git: None };
    let report = build_packages(repo, &options).map_err(package_error)?;
    let folder = output.unwrap_or_else(|| repo.join("packages"));
    let sidecar = folder.join(SIDECAR);
    let text = gai_core::to_js_json(&search_index::build(repo)?) + "\n";
    crate::managed::write_replacing(&sidecar, text.as_bytes())?;
    Ok((report, folder))
}

/// Runs `library packages`.
pub fn packages(repo: &Path, args: &PackagesArgs) -> Result<Report> {
    let (report, folder) = build(repo, args.output.as_deref())?;
    let value = json!({
        "icons": report.icons,
        "version": report.version,
        "dirty": report.dirty,
        "output": folder.display().to_string(),
        "searchIndex": folder.join(SIDECAR).display().to_string(),
    });
    Ok(Report::new(value, format!("{}\n", report.summary())))
}

/// Runs `library release`.
pub fn release(repo: &Path, args: &ReleaseArgs) -> Result<Report> {
    let (built, _) = build(repo, None)?;
    let options = PackReleaseOptions { output: args.output.clone(), allow_dirty: args.allow_dirty };
    let packed = pack_release(repo, &options).map_err(package_error)?;
    let packages: Vec<_> = packed
        .packages
        .iter()
        .map(|p| json!({ "name": p.name, "version": p.version, "file": p.file, "sha256": p.sha256, "bytes": p.bytes }))
        .collect();
    let value = json!({
        "icons": built.icons,
        "version": built.version,
        "status": packed.status,
        "output": packed.output.display().to_string(),
        "packages": packages,
    });
    Ok(Report::new(value, format!("{}\n{}\n", built.summary(), packed.summary())))
}
