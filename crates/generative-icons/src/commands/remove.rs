//! `remove`: deletes an icon's managed files, only when none was edited, and drops it
//! from the lock.

use super::{lock_project, text_list, unique};
use crate::Context;
use crate::cli::RemoveArgs;
use crate::config::Config;
use crate::error::{CliError, ErrorCode, Result};
use crate::lock::Lock;
use crate::managed::Batch;
use crate::output::Report;
use serde_json::{Value, json};
use std::fmt::Write;

/// Runs `remove`.
pub fn run(ctx: &Context, args: &RemoveArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let config = Config::require(root, ctx.platform)?;
    let _guard = lock_project(ctx, config.layout)?;
    let lock = Lock::load(root, config.layout)?;
    let ids = unique(&args.ids);
    let mut next = lock.clone();
    let mut batch = Batch::new();
    for &id in &ids {
        let record = lock.icon(id).ok_or_else(|| {
            CliError::new(ErrorCode::UnknownIcon, format!("Not installed: {id}"))
                .with_id(id)
                .with_hint("generative-icons info lists installed icons")
        })?;
        for path in managed_files(record) {
            // Only files this tool recorded are deleted; `plan` refuses edited ones.
            if next.object_mut("files").shift_remove(&path).is_some() {
                batch.delete(path);
            }
        }
        next.object_mut("icons").shift_remove(id);
    }
    batch.write(next.path, next.to_bytes());
    let changed = batch.plan(root, &lock.expected())?.apply()?;

    let value = json!({ "removed": ids, "changed": changed });
    let mut text = String::new();
    let _ = writeln!(text, "Removed {}.", ids.join(", "));
    text_list(&mut text, "Changed", &changed);
    Ok(Report::new(value, text))
}

/// The asset, the descriptor and, for web modules, its `.d.mts` types.
fn managed_files(record: &Value) -> Vec<String> {
    let mut files = Vec::new();
    for key in ["asset", "source"] {
        if let Some(path) = record.get(key).and_then(Value::as_str) {
            files.push(path.to_string());
            if let Some(stem) = path.strip_suffix(".mjs").filter(|_| key == "source") {
                files.push(format!("{stem}.d.mts"));
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    #[test]
    fn web_descriptors_bring_their_types() {
        let record = serde_json::json!({ "source": "src/i/heart.mjs", "asset": "public/i/heart.lottie" });
        assert_eq!(super::managed_files(&record), ["public/i/heart.lottie", "src/i/heart.mjs", "src/i/heart.d.mts"]);
    }
}
