//! Commands. Each returns a [`Report`](crate::output::Report) or a
//! [`CliError`](crate::error::CliError).

pub mod add;
pub mod build;
pub mod check;
pub mod context;
pub mod create;
pub mod diff;
pub mod doctor;
pub mod info;
pub mod init;
pub mod library;
pub mod login;
pub mod preview;
pub mod remove;
pub mod search;
pub mod submit;
pub mod verify_action;

use crate::Context;
use crate::config::Layout;
use crate::error::Result;
use crate::managed::ProcessLock;
use std::fmt::Write;

/// Unique values in first-seen order.
fn unique(values: &[String]) -> Vec<&str> {
    let mut seen = indexmap::IndexSet::new();
    for value in values {
        seen.insert(value.as_str());
    }
    seen.into_iter().collect()
}

/// Takes the project lock for a command that writes managed files. Legacy
/// projects have no `.gen-animated-icons/` folder, so they go unlocked.
fn lock_project(ctx: &Context, layout: Layout) -> Result<Option<ProcessLock>> {
    match layout {
        Layout::Current => {
            ProcessLock::acquire(&ctx.cwd, || ctx.out.log("waiting for another gen-animated-icons run in this project"))
                .map(Some)
        }
        Layout::Legacy => Ok(None),
    }
}

/// `Title:` followed by indented lines, or `Title: none`.
fn text_list(out: &mut String, title: &str, items: &[String]) {
    if items.is_empty() {
        let _ = writeln!(out, "{title}: none");
        return;
    }
    let _ = writeln!(out, "{title}:");
    for item in items {
        let _ = writeln!(out, "  {item}");
    }
}
