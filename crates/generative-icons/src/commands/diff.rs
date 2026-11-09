//! `diff`: every managed file in the lock and whether it is unchanged, modified or
//! missing. Exits 0 either way.

use crate::Context;
use crate::error::Result;
use crate::lock::Lock;
use crate::output::Report;
use serde_json::Value;
use std::fmt::Write;

/// Runs `diff`.
pub fn run(ctx: &Context) -> Result<Report> {
    let status = Lock::load(&ctx.cwd)?.status(&ctx.cwd)?;
    let value = Value::Array(status.iter().map(|s| s.to_json()).collect());
    let mut text = String::new();
    let unchanged = status.iter().filter(|s| s.status == "unchanged").count();
    for entry in status.iter().filter(|s| s.status != "unchanged") {
        let _ = writeln!(text, "{:<9} {}", entry.status, entry.file);
    }
    let _ = match (status.len(), unchanged) {
        (0, _) => writeln!(text, "No managed files."),
        (total, same) if total == same => writeln!(text, "All {total} managed files are unchanged."),
        (_, same) => writeln!(text, "{same} unchanged."),
    };
    Ok(Report::new(value, text))
}
