//! `library register <set>`: adds a built, verified set to `prototype/icons/catalog.json`
//! (`icon-set.py register`). The preflight and set verification gates run in process;
//! the catalog is written in Python `json.dumps` format, only when it changes, and keeps
//! its file mode.

use super::sets_dir;
use crate::cli::RegisterArgs;
use crate::error::{CliError, ErrorCode, Result};
use crate::js::truthy;
use crate::local_set::is_icon_name;
use crate::output::Report;
use gai_check::preflight::{Preflight, Status};
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;

/// Runs `library register`.
pub fn run(repo: &Path, args: &RegisterArgs) -> Result<Report> {
    let set = args.set.as_str();
    let bytes = set.as_bytes();
    let valid =
        bytes.len() > 3 && bytes[..2].iter().all(u8::is_ascii_digit) && bytes[2] == b'-' && is_icon_name(&set[3..]);
    if !valid {
        return Err(CliError::usage("Use a source set ID, such as 35-landscaping."));
    }
    let source = sets_dir(repo).join(set);
    let config = read_json(&source.join("set-config.json"))?;
    let config_id = config.get("id").ok_or_else(|| CliError::new(ErrorCode::InvalidConfig, "'id'"))?;
    if config_id.as_str() != Some(set) {
        return Err(CliError::new(ErrorCode::InvalidConfig, "The source directory and configured set ID disagree."));
    }
    let relative = format!("{}/assets/set.json", &set[..2]);
    let manifest = repo.join("prototype/icons").join(&relative);
    if !manifest.is_file() {
        return Err(CliError::new(
            ErrorCode::ProjectNotFound,
            "Build the complete set before registration; its asset manifest is missing.",
        ));
    }
    let data = read_json(&manifest)?;
    if data.get("id").and_then(Value::as_str) != Some(set) || !data.get("icons").is_some_and(truthy) {
        return Err(CliError::new(
            ErrorCode::InvalidConfig,
            "The built manifest must match this set and contain icons.",
        ));
    }
    let catalog_file = repo.join("prototype/icons/catalog.json");
    let before = std::fs::read(&catalog_file).map_err(|error| CliError::io("read", &catalog_file, &error))?;
    let mut catalog: Value = serde_json::from_slice(&before).map_err(|error| {
        CliError::new(ErrorCode::InvalidCatalog, format!("Cannot parse {}: {error}", catalog_file.display()))
    })?;

    gate("preflight.mjs", || {
        let preflight = Preflight::open(repo).map_err(|error| error.to_string())?;
        let report = preflight.check_set(&source).map_err(|error| error.to_string())?;
        match report.report.status {
            Status::Pass => Ok(()),
            Status::Fail => Err(gai_core::to_js_json_pretty(&report.to_json(), 2)),
        }
    })?;
    gate("verify.mjs", || {
        let options = gai_check::verify::VerifyOptions {
            builder_identity_sha256: Some(gai_build::records::builder_identity_sha256()),
        };
        let outcome = gai_check::verify::verify_set(&gai_check::verify::SetPaths::catalog(&source), &options);
        crate::commands::check::write_evidence(&outcome, &source).map_err(|error| error.message)?;
        outcome.error.map_or(Ok(()), |error| Err(error.to_string()))
    })?;

    let category = &set[3..];
    let label = config.get("name").cloned().unwrap_or(Value::Null);
    let invalid = || CliError::new(ErrorCode::InvalidCatalog, "The catalog needs sets and categories lists");
    let categories = catalog.get("categories").and_then(Value::as_array).ok_or_else(invalid)?;
    let current = categories.iter().position(|c| c.get("id").and_then(Value::as_str) == Some(category));
    if let Some(index) = current {
        let listed = categories[index]
            .get("sets")
            .and_then(Value::as_array)
            .is_some_and(|sets| sets.iter().any(|s| s.as_str() == Some(&relative)));
        if !listed {
            return Err(CliError::new(
                ErrorCode::Exists,
                format!("Category {category} already belongs to another set; edit that category deliberately."),
            ));
        }
    }
    let sets = catalog.get_mut("sets").and_then(Value::as_array_mut).ok_or_else(invalid)?;
    if !sets.iter().any(|s| s.as_str() == Some(&relative)) {
        sets.push(json!(relative));
    }
    if current.is_none() {
        let categories = catalog.get_mut("categories").and_then(Value::as_array_mut).ok_or_else(invalid)?;
        categories.push(json!({ "id": category, "label": label, "sets": [relative] }));
    }
    let updated = gai_core::to_py_json(&catalog, Some(2), true) + "\n";
    let changed = updated.as_bytes() != before.as_slice();
    if changed {
        replace(&catalog_file, updated.as_bytes(), &before)?;
    }
    let value = json!({ "set": set, "manifest": relative, "category": { "id": category, "label": label }, "catalogChanged": changed });
    let text = "Registered the verified build. Inspect the gallery. Registration is not visual approval. Repackage after final review to include the updated catalog and evidence.\n";
    Ok(Report::new(value, text.to_string()))
}

/// Runs one registration gate; a failure stops registration with its report.
fn gate(script: &str, check: impl FnOnce() -> std::result::Result<(), String>) -> Result<()> {
    check().map_err(|detail| {
        CliError::new(ErrorCode::CheckFailed, format!("Registration stopped: {script} failed.\n{detail}"))
    })
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path).map_err(|error| CliError::io("read", path, &error))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| CliError::new(ErrorCode::InvalidConfig, format!("Cannot parse {}: {error}", path.display())))
}

/// Replaces the catalog through a temporary sibling with the old file's mode, unless it
/// changed while the gates ran.
fn replace(file: &Path, bytes: &[u8], before: &[u8]) -> Result<()> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::Builder::new()
        .prefix(".catalog-")
        .tempfile_in(dir)
        .map_err(|error| CliError::io("create", dir, &error))?;
    temp.write_all(bytes).map_err(|error| CliError::io("write", temp.path(), &error))?;
    let permissions = std::fs::metadata(file).map_err(|error| CliError::io("read", file, &error))?.permissions();
    std::fs::set_permissions(temp.path(), permissions).map_err(|error| CliError::io("write", temp.path(), &error))?;
    if std::fs::read(file).map_err(|error| CliError::io("read", file, &error))? != before {
        return Err(CliError::new(
            ErrorCode::FileChanged,
            "The catalog changed during verification. Retry registration.",
        ));
    }
    temp.persist(file).map_err(|error| CliError::io("write", file, &error.error))?;
    Ok(())
}
