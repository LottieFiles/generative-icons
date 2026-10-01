//! `preview`: renders an icon's motions to contact sheets with the renderer the players
//! use (dotlottie-rs), so an AI or a person can look at real frames. Rows are looks
//! (style, weight, theme), columns are the same elapsed times for every motion, so
//! Light and Strong compare at equal time.
//!
//! The source is the project's build of a local icon, else the installed asset, else
//! the library's `.lottie`.

use crate::Context;
use crate::cli::{MAX_COLUMNS, MotionChoice, PreviewArgs};
use crate::config::Config;
use crate::descriptor::read_installed;
use crate::error::{CliError, ErrorCode, Result};
use crate::library::is_icon_id;
use crate::local_set::{Built, LocalSet, PREVIEW_DIR, is_icon_name};
use crate::lock::Lock;
use crate::managed::{create_dir_inside, inside, read_optional, write_replacing};
use crate::output::Report;
use gai_core::sha256_hex;
use gai_render::{ContactSheet, Look, RENDERER, Style, Theme, contact_sheet};
use serde_json::{Value, json};
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Columns of a default sheet: six equal steps over the longest active time, then rest.
const STEPS: usize = 6;
/// Cell size, in pixels, of the review sheets `build` writes and `submit` bundles.
pub const REVIEW_CELL: u32 = 96;

/// One motion of an icon, as its descriptor or set.json records it.
#[derive(Debug, Clone, PartialEq)]
pub struct Motion {
    /// `light` or `strong`.
    pub id: String,
    /// The animation inside the `.lottie`.
    pub name: String,
    /// Active time in seconds.
    pub active: f64,
    /// Rest frame.
    pub rest: f64,
    /// Frames per second.
    pub fps: f64,
    /// Frames in the timeline.
    pub frames: f64,
}

impl Motion {
    /// The top-level motions of a descriptor or set.json icon.
    pub fn of(icon: &Value) -> Vec<Motion> {
        motions(icon).filter_map(|m| Motion::read(m, "")).collect()
    }

    /// The reverse action of each motion of a stateful icon; `id` is `<motionId>-reverse`.
    pub fn reverses(icon: &Value) -> Vec<Motion> {
        motions(icon).filter_map(|m| Motion::read(m.get("reverse")?, "-reverse")).collect()
    }

    fn read(m: &Value, suffix: &str) -> Option<Motion> {
        Some(Motion {
            id: format!("{}{suffix}", m.get("motionId")?.as_str()?),
            name: m.get("name")?.as_str()?.to_string(),
            active: m.get("active").and_then(Value::as_f64).unwrap_or(0.0),
            rest: m.get("rest").and_then(Value::as_f64).unwrap_or(0.0),
            fps: m.get("fps").and_then(Value::as_f64).unwrap_or(gai_js::FPS),
            frames: m.get("frames").and_then(Value::as_f64).unwrap_or(gai_js::DURATION * gai_js::FPS),
        })
    }
}

fn motions(icon: &Value) -> impl Iterator<Item = &Value> {
    icon.get("motions").and_then(Value::as_array).into_iter().flatten()
}

/// A written sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// Where it was written.
    pub path: PathBuf,
    /// The PNG.
    pub png: Vec<u8>,
    /// The motion shown.
    pub motion: Motion,
    /// Columns, in seconds.
    pub times: Vec<f64>,
    /// Row labels.
    pub looks: Vec<String>,
}

impl Sheet {
    /// `{path, sha256, width, height, sheet, motion, animation, times, looks}`; `path`
    /// relative to `root` when inside it.
    pub fn to_json(&self, root: &Path) -> Value {
        let (width, height) = png_size(&self.png);
        json!({
            "path": relative(root, &self.path),
            "sha256": sha256_hex(&self.png),
            "width": width,
            "height": height,
            "sheet": "contact",
            "motion": self.motion.id,
            "animation": self.motion.name,
            "times": self.times,
            "looks": self.looks,
        })
    }
}

/// `path` relative to `root` when inside it, `/`-separated.
pub fn relative(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(inner) => inner.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"),
        Err(_) => path.display().to_string(),
    }
}

/// Width and height from a PNG's IHDR chunk.
fn png_size(png: &[u8]) -> (u32, u32) {
    let read = |at: usize| png.get(at..at + 4).map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
    (read(16), read(20))
}

/// Default columns: six equal steps from 0 to the longest active time, then the latest
/// rest time.
pub fn default_times(motions: &[Motion]) -> Vec<f64> {
    let active = motions.iter().map(|m| m.active).fold(0.0, f64::max);
    let rest = motions.iter().map(|m| m.rest / m.fps).fold(0.0, f64::max);
    let mut times: Vec<f64> =
        (0..STEPS).map(|step| gai_core::round5(active * step as f64 / (STEPS - 1) as f64)).collect();
    if rest > active {
        times.push(gai_core::round5(rest));
    }
    times
}

/// `count` evenly spaced columns from 0 to the latest end of `motions`' active time or
/// rest frame, so every column shows the action rather than the held pose after it.
pub fn spread(motions: &[Motion], count: usize) -> Vec<f64> {
    let end = motions.iter().map(|m| m.active.max(m.rest / m.fps)).fold(0.0, f64::max);
    let steps = count.max(2) - 1;
    (0..count).map(|i| gai_core::round5(end * i as f64 / steps as f64)).collect()
}

/// Renders one sheet per motion into `out`, as `<animation>.png`. Animation names come
/// from descriptors and builds, so only plain ids become file names.
pub fn render(
    lottie: &[u8],
    motions: &[Motion],
    times: &[f64],
    looks: &[Look],
    cell: u32,
    out: &Path,
) -> Result<Vec<Sheet>> {
    let rendered = render_pngs(lottie, motions, times, looks, cell)?;
    std::fs::create_dir_all(out).map_err(|error| CliError::io("create", out, &error))?;
    let labels: Vec<String> = looks.iter().map(look_label).collect();
    let mut sheets = Vec::new();
    for (motion, png) in motions.iter().zip(rendered) {
        // A symlink planted at the sheet's path is replaced, never written through.
        let path = out.join(sheet_name(motion));
        write_replacing(&path, &png)?;
        sheets.push(Sheet { path, png, motion: motion.clone(), times: times.to_vec(), looks: labels.clone() });
    }
    Ok(sheets)
}

/// The file name of a motion's sheet: `<animation>.png`.
pub fn sheet_name(motion: &Motion) -> String {
    format!("{}.png", motion.name)
}

/// One PNG per motion, rendered in memory; motions whose names are not plain ids are
/// refused before anything renders.
fn render_pngs(lottie: &[u8], motions: &[Motion], times: &[f64], looks: &[Look], cell: u32) -> Result<Vec<Vec<u8>>> {
    if let Some(motion) = motions.iter().find(|motion| !is_icon_id(&motion.name)) {
        return Err(CliError::new(
            ErrorCode::InvalidCatalog,
            format!("Refusing to render motion {:?}: its name is not a plain icon id", motion.name),
        ));
    }
    let mut pngs = Vec::new();
    for motion in motions {
        let request = ContactSheet {
            animation: Some(motion.name.clone()),
            title: Some(format!("{} ({})", motion.name, motion.id)),
            frames: times.iter().map(|t| t * motion.fps).collect(),
            looks: looks.to_vec(),
            cell,
        };
        let png = contact_sheet(lottie, &request).map_err(|error| {
            CliError::new(ErrorCode::CorruptArchive, format!("Cannot render {}: {error}", motion.name))
        })?;
        pngs.push(png);
    }
    Ok(pngs)
}

/// The review sheets of a build, as `build` writes them and `submit` bundles them: one per
/// motion of `icon` (Light, Strong; reverse actions aside) at the default columns, in the
/// review looks, with [`REVIEW_CELL`] cells. The same build always gives the same bytes.
pub struct Review {
    /// The icon's motions, reverse actions aside.
    pub motions: Vec<Motion>,
    /// Columns, in seconds: [`default_times`].
    pub times: Vec<f64>,
    /// Rows: [`ContactSheet::review_looks`].
    pub looks: Vec<Look>,
}

impl Review {
    /// The review sheets of the icon record `icon`.
    pub fn of(icon: &Value) -> Review {
        let motions = Motion::of(icon);
        let times = default_times(&motions);
        Review { motions, times, looks: ContactSheet::review_looks() }
    }

    /// Renders them into `out` (see [`render`]).
    pub fn render(&self, lottie: &[u8], out: &Path) -> Result<Vec<Sheet>> {
        render(lottie, &self.motions, &self.times, &self.looks, REVIEW_CELL, out)
    }

    /// Renders them in memory: `(file name, PNG)` in motion order.
    pub fn pngs(&self, lottie: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
        let pngs = render_pngs(lottie, &self.motions, &self.times, &self.looks, REVIEW_CELL)?;
        Ok(self.motions.iter().map(sheet_name).zip(pngs).collect())
    }
}

fn look_label(look: &Look) -> String {
    match look.style {
        Style::Solid => format!("solid {}", look.theme.id()),
        style => format!("{} {} {}", style.name(), gai_core::js_number_to_string(look.weight), look.theme.id()),
    }
}

/// Runs `preview`.
pub fn run(ctx: &Context, args: &PreviewArgs) -> Result<Report> {
    if let Some(times) = &args.times {
        if times.len() > MAX_COLUMNS {
            return Err(CliError::usage(format!("--times takes at most {MAX_COLUMNS} columns")));
        }
        if let Some(time) = times.iter().find(|time| !(time.is_finite() && **time >= 0.0)) {
            let time = gai_core::js_number_to_string(*time);
            return Err(CliError::usage(format!("--times takes seconds from 0, not {time}")));
        }
    }
    let root = ctx.cwd.as_path();
    let (source, lottie, icon) = source(ctx, &args.id)?;
    let (forward, reverse) = (Motion::of(&icon), Motion::reverses(&icon));
    let motions = select(&args.id, args, &forward, &reverse)?;
    // Light and Strong share columns, and so do their reverse actions, which get their own.
    let columns = |motion: &Motion| -> Vec<f64> {
        let group = if reverse.contains(motion) { &reverse } else { &forward };
        match (&args.times, args.frames) {
            (Some(times), _) => times.clone(),
            (None, Some(count)) => spread(group, count),
            (None, None) => default_times(group),
        }
    };
    let looks = looks(args)?;
    let out = match &args.out {
        Some(out) => ctx.cwd.join(out),
        None => create_dir_inside(root, &format!("{PREVIEW_DIR}/{}", args.id))?,
    };
    let mut sheets = Vec::new();
    for motion in &motions {
        sheets.extend(render(&lottie, std::slice::from_ref(motion), &columns(motion), &looks, args.cell, &out)?);
    }
    let mut weight_method = "authored";
    for look in &looks {
        let method =
            gai_render::preview::weight_method(look.weight).map_err(|error| CliError::usage(error.to_string()))?;
        if method.as_str() == "rewrite" {
            weight_method = "rewrite";
        }
    }
    let value = json!({
        "id": args.id,
        "source": source,
        "renderer": RENDERER,
        "weightMethod": weight_method,
        "files": sheets.iter().map(|sheet| sheet.to_json(root)).collect::<Vec<_>>(),
    });
    let mut text = String::new();
    let _ = writeln!(text, "Rendered {} from its {source} with {RENDERER}:", args.id);
    for sheet in &sheets {
        let _ = writeln!(text, "  {}", relative(root, &sheet.path));
    }
    Ok(Report::new(value, text))
}

/// The motions `--motion` or `--animation` selects from the icon's forward and reverse
/// actions.
fn select(id: &str, args: &PreviewArgs, forward: &[Motion], reverse: &[Motion]) -> Result<Vec<Motion>> {
    let every: Vec<&Motion> = forward.iter().chain(reverse).collect();
    if let Some(names) = &args.animation {
        return names
            .iter()
            .map(|name| {
                every.iter().find(|m| &m.name == name).map(|m| (*m).clone()).ok_or_else(|| {
                    let known: Vec<&str> = every.iter().map(|m| m.name.as_str()).collect();
                    CliError::usage(format!("{id} has no animation {name}; it has {}", known.join(", "))).with_id(id)
                })
            })
            .collect();
    }
    let chosen: Vec<Motion> = match args.motion {
        MotionChoice::Both => forward.to_vec(),
        MotionChoice::Light | MotionChoice::Strong => {
            forward.iter().filter(|m| m.id == args.motion.as_str()).cloned().collect()
        }
        MotionChoice::Reverse => reverse.to_vec(),
        MotionChoice::All => every.into_iter().cloned().collect(),
    };
    if chosen.is_empty() {
        let what = match args.motion {
            MotionChoice::Reverse => "reverse action (only stateful icons have one)".to_string(),
            other => format!("{} motion", other.as_str()),
        };
        return Err(CliError::new(ErrorCode::InvalidCatalog, format!("{id} has no {what}")).with_id(id));
    }
    Ok(chosen)
}

/// Rows: the review looks, or the product of `--styles`, `--weights` and `--themes`.
fn looks(args: &PreviewArgs) -> Result<Vec<Look>> {
    if args.styles.is_none() && args.weights.is_none() && args.themes.is_none() {
        return Ok(ContactSheet::review_looks());
    }
    let styles = args.styles.clone().unwrap_or_else(|| vec![Style::Outline]);
    let weights = args.weights.clone().unwrap_or_else(|| vec![2.0]);
    let themes = args.themes.clone().unwrap_or_else(|| vec![Theme::Dark]);
    let mut looks = Vec::new();
    for &theme in &themes {
        for &style in &styles {
            // Solid ignores the weight, so it gets one row per theme.
            let row_weights: &[f64] = if style == Style::Solid { &weights[..1] } else { &weights };
            for &weight in row_weights {
                if !(1.0..=3.0).contains(&weight) {
                    return Err(CliError::usage(format!("Weight {weight} is outside 1-3")));
                }
                looks.push(Look { theme, style, weight, color: None });
            }
        }
    }
    Ok(looks)
}

/// The `.lottie` to render and the icon record describing its motions:
/// `(source name, bytes, icon)`.
fn source(ctx: &Context, id: &str) -> Result<(&'static str, Vec<u8>, Value)> {
    let root = ctx.cwd.as_path();
    if let Some(config) = Config::load(root, ctx.platform)? {
        if let Some(built) = Built::read(root, id)? {
            return Ok(("build", built.lottie, built.icon));
        }
        let lock = Lock::load(root, config.layout)?;
        if let Some(record) = lock.icon(id) {
            let asset = record.get("asset").and_then(Value::as_str);
            let source = record.get("source").and_then(Value::as_str);
            if let (Some(asset), Some(source)) = (asset, source)
                && let Some(bytes) = read_optional(&inside(root, asset)?)?
            {
                return Ok(("installed asset", bytes, read_installed(root, source)?));
            }
        }
        // A local icon `create` made has nothing to render until it is built.
        let batch = LocalSet::of(&config).batch(id);
        if is_icon_name(id) && inside(root, &batch).is_ok_and(|path| path.is_file()) {
            return Err(CliError::new(
                ErrorCode::UnknownIcon,
                format!("{id} is a local icon that is not built yet, so there is nothing to render"),
            )
            .with_id(id)
            .with_path(batch)
            .with_hint(format!(
                "Run gen-animated-icons check {id}, then gen-animated-icons build {id}, then preview it"
            )));
        }
    }
    let library = ctx.library()?;
    let catalog = library.catalog()?;
    let icon = catalog.get(id).ok_or_else(|| {
        CliError::new(ErrorCode::UnknownIcon, format!("Unknown icon: {id}. Build it, or search the library."))
            .with_id(id)
            .with_hint(format!(
                "Find an icon with gen-animated-icons search \"<meaning>\", or make one with gen-animated-icons create {id}"
            ))
    })?;
    Ok(("library asset", library.asset(icon)?, icon.descriptor.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn motion(id: &str, active: f64, rest: f64) -> Motion {
        Motion { id: id.into(), name: id.into(), active, rest, fps: 60.0, frames: 144.0 }
    }

    #[test]
    fn default_columns_share_times_across_motions() {
        let times = default_times(&[motion("light", 0.5, 33.0), motion("strong", 0.4, 27.0)]);
        assert_eq!(times, [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.55]);
        assert_eq!(default_times(&[motion("light", 1.0, 30.0)]), [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
    }

    #[test]
    fn motion_names_that_are_not_ids_never_become_paths() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("sheets");
        let victim = dir.path().join("victim");
        for name in [victim.join("pwned").display().to_string(), "../pwned".into(), "a/b".into(), "C:x".into()] {
            let motions = [Motion { name: name.clone(), ..motion("strong", 0.4, 27.0) }];
            let error = render(b"not read", &motions, &[0.0], &ContactSheet::review_looks(), 32, &out).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidCatalog, "{name}");
        }
        assert!(!out.exists() && !victim.exists() && !dir.path().join("pwned.png").exists());
    }

    #[test]
    fn reverse_actions_can_be_previewed() {
        let action = |name: &str, motion: &str| json!({"name": name, "motionId": motion, "active": 0.3, "rest": 21});
        let mut light = action("bell", "light");
        light["reverse"] = action("bell-reset", "light");
        let mut strong = action("bell-strong", "strong");
        strong["reverse"] = action("bell-reset-strong", "strong");
        let icon = json!({"motions": [light, strong]});
        let (forward, reverse) = (Motion::of(&icon), Motion::reverses(&icon));
        let names = |motions: &[Motion]| motions.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        let args = |motion: &str, animation: Option<&str>| {
            let mut argv = vec!["gen-animated-icons", "preview", "bell", "--motion", motion];
            if let Some(animation) = animation {
                argv = vec!["gen-animated-icons", "preview", "bell", "--animation", animation];
            }
            let cli = <crate::cli::Cli as clap::Parser>::try_parse_from(argv).unwrap();
            let crate::cli::Command::Preview(args) = cli.command else { panic!("preview") };
            args
        };
        let chosen = select("bell", &args("reverse", None), &forward, &reverse).unwrap();
        assert_eq!(names(&chosen), ["bell-reset", "bell-reset-strong"]);
        assert_eq!(chosen[0].id, "light-reverse");
        assert_eq!(select("bell", &args("all", None), &forward, &reverse).unwrap().len(), 4);
        assert_eq!(select("bell", &args("strong", None), &forward, &reverse).unwrap()[0].name, "bell-strong");
        let named = select("bell", &args("both", Some("bell-reset-strong,bell")), &forward, &reverse).unwrap();
        assert_eq!(names(&named), ["bell-reset-strong", "bell"]);
        let error = select("bell", &args("both", Some("bell-nope")), &forward, &reverse).unwrap_err();
        assert_eq!(
            (error.code, error.message.as_str()),
            (
                ErrorCode::Usage,
                "bell has no animation bell-nope; it has bell, bell-strong, bell-reset, bell-reset-strong"
            )
        );
        let error = select("bell", &args("reverse", None), &forward, &[]).unwrap_err();
        assert_eq!(error.message, "bell has no reverse action (only stateful icons have one)");
    }

    #[test]
    fn png_size_reads_the_header() {
        let mut png = vec![0u8; 24];
        png[16..20].copy_from_slice(&640u32.to_be_bytes());
        png[20..24].copy_from_slice(&480u32.to_be_bytes());
        assert_eq!(png_size(&png), (640, 480));
    }
}
