//! `export`: writes an icon as an animated SVG that browsers play with SMIL, no player or
//! script needed ([`gai_svg`]). The source is the same as `preview`'s: the project's build
//! of a local icon, else the installed asset, else the library's `.lottie`.

use crate::Context;
use crate::cli::{ExportArgs, ExportMotion, SvgTrigger};
use crate::commands::preview::{relative, source};
use crate::error::{CliError, ErrorCode, Result};
use crate::managed::write_replacing;
use crate::output::Report;
use crate::weight::read_zip;
use gai_core::sha256_hex;
use gai_render::Style;
use gai_svg::{SvgError, SvgOptions, Trigger};
use serde_json::{Value, json};

/// Runs `export`.
pub fn run(ctx: &Context, args: &ExportArgs) -> Result<Report> {
    let (source, lottie, icon) = source(ctx, &args.id)?;
    let entries = read_zip(&lottie)?;
    let animation = |name: &str| -> Option<Value> {
        let member = format!("a/{name}.json");
        entries.iter().find(|entry| entry.name == member).and_then(|entry| serde_json::from_slice(&entry.data).ok())
    };
    let motion = match args.motion {
        ExportMotion::Light => "light",
        ExportMotion::Strong => "strong",
    };
    let trigger = match args.trigger {
        SvgTrigger::Auto => gai_svg::default_trigger(&icon, motion),
        SvgTrigger::Hover => Trigger::Hover,
        SvgTrigger::Click => Trigger::Click,
        SvgTrigger::Toggle => Trigger::Toggle,
        SvgTrigger::Loop => Trigger::Loop,
        SvgTrigger::Load => Trigger::Load,
        SvgTrigger::Js => Trigger::Js,
    };
    let style = match args.style {
        Style::Outline => gai_svg::Style::Outline,
        Style::Solid => gai_svg::Style::Solid,
        Style::Duotone => gai_svg::Style::Duotone,
    };
    let options = SvgOptions {
        motion: motion.into(),
        style,
        weight: args.weight,
        color: args.color.clone(),
        trigger,
        id: args.id_prefix.clone(),
        size: args.size,
        state: args.state.clone(),
    };
    let svg = gai_svg::icon_to_svg(&icon, &animation, &options).map_err(|error| {
        let code = match error {
            SvgError::Options(_) => ErrorCode::Usage,
            SvgError::Unsupported(_) => ErrorCode::CheckFailed,
            SvgError::Invalid(_) => ErrorCode::CorruptArchive,
        };
        CliError::new(code, error.to_string()).with_id(&args.id)
    })?;

    let mut value = json!({
        "id": args.id,
        "source": source,
        "motion": motion,
        "style": style.name(),
        "weight": args.weight,
        "color": args.color,
        "trigger": trigger.name(),
        "bytes": svg.len(),
        "sha256": sha256_hex(svg.as_bytes()),
    });
    let note = match trigger {
        Trigger::Hover | Trigger::Click | Trigger::Toggle | Trigger::Js => "inline it as <svg>; <img> shows it at rest",
        _ => "inline it, or use it as an <img> or CSS background",
    };
    value["use"] = Value::String(note.into());
    let Some(out) = &args.out else {
        value["svg"] = Value::String(svg.clone());
        return Ok(Report::new(value, svg));
    };
    let path = ctx.cwd.join(out);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| CliError::io("create", parent, &error))?;
    }
    // A symlink planted at the target is replaced, never written through.
    write_replacing(&path, svg.as_bytes())?;
    let shown = relative(&ctx.cwd, &path);
    value["path"] = Value::String(shown.clone());
    let text = format!("Wrote {shown} ({} bytes, {} trigger: {note})", svg.len(), trigger.name());
    Ok(Report::new(value, text))
}
