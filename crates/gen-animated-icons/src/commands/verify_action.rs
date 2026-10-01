//! `__verify-action` (hidden): runs the visual checks of one action in this process and
//! writes the outcome for the parent. `library verify-visual` spawns one per action, so a
//! renderer fault kills only the child, which the parent records as a crash.

use crate::cli::VerifyActionArgs;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use gai_render::{ActionOptions, SetManifest, SetRules, verify_action};
use serde_json::json;

/// Runs `__verify-action`.
pub fn run(args: &VerifyActionArgs) -> Result<Report> {
    let render = |error: gai_render::RenderError| CliError::new(ErrorCode::Internal, error.to_string());
    let set_file = args.assets.join("set.json");
    let bytes = std::fs::read(&set_file).map_err(|error| CliError::io("read", &set_file, &error))?;
    let set = SetManifest::from_slice(&bytes).map_err(render)?;
    let meta = set.all_actions().into_iter().find(|meta| meta.name == args.action).ok_or_else(|| {
        CliError::new(ErrorCode::UnknownIcon, format!("No action {} in {}", args.action, set_file.display()))
    })?;
    let file = args.assets.join(format!("{}.lottie", args.action));
    let lottie = std::fs::read(&file).map_err(|error| CliError::io("read", &file, &error))?;
    let run =
        verify_action(&set, meta, &lottie, &SetRules::for_set(&args.set), ActionOptions { sheets: !args.no_sheets });
    run.write_to(&args.out).map_err(render)?;
    Ok(Report::new(json!({ "action": args.action, "out": args.out.display().to_string() }), String::new()))
}
