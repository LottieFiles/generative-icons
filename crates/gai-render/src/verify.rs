//! The per-action visual checks of `prototype/icons/35/verify.html` (the page
//! sets 35 onwards and every new set use), run natively.
//!
//! [`verify_action`] reproduces the page's result record key for key and value
//! for value (except the wall-clock `playback.observedFrameEvents`), its
//! failure messages including the page's stale `context`, and its PNG sheets.

use crate::catalog::{ActionMeta, Family, Motion, SetManifest};
use crate::player::{Paint, PaintedPlayer, Style, Theme};
use crate::raster::{bounds, counters, equivalent, fnv1a, raster_difference};
use crate::rules::SetRules;
use crate::sheets::{self, STYLE_LABELS, SheetImage};
use crate::RenderError;
use dotlottie_rs::PlayerEvent;
use gai_core::{js_number_to_string, sha256_hex, to_js_json};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Raster sizes every action is checked at, in order.
pub const SIZES: [u32; 4] = [24, 32, 48, 192];

/// The browser's frame clock: one tick per 60 Hz display frame, in ms.
const TICK_MS: f32 = 1000.0 / 60.0;

/// A JSON number as `JSON.stringify` would emit it and Python would re-read
/// it: integral values become integers (`2`, never `2.0`; `-0` → `0`),
/// non-finite values become `null`.
pub fn js_number(x: f64) -> Value {
    if !x.is_finite() {
        Value::Null
    } else if x.fract() == 0.0 && x.abs() < 1e21 {
        if x.abs() <= 9_007_199_254_740_992.0 {
            json!(x as i64)
        } else {
            // Exact integers above 2^53 keep their digits through the i128 cast.
            serde_json::from_str(&(x as i128).to_string()).unwrap_or(Value::Null)
        }
    } else {
        json!(x)
    }
}

/// Whether and how an action's visual checks ended.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Every check passed: the evidence record (`report.assets[]` entry).
    Passed(Map<String, Value>),
    /// The first failed check, as the page records it (`"Error: … — …"`).
    Failed(String),
}

/// The result of checking one action, with the page state the set report
/// carries forward.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionRun {
    /// Action name.
    pub name: String,
    /// Result record or failure message.
    pub outcome: Outcome,
    /// The page's `context` when the action ended (used by the final
    /// "Renderer errors" message).
    pub context: String,
    /// `report.checking` when the action ended.
    pub checking: String,
    /// Sheets produced before the action ended; kept even when it failed.
    pub images: Vec<SheetImage>,
}

impl ActionRun {
    /// The run recorded for an action whose renderer process died (crash
    /// isolation, see the crate docs). `detail` is e.g. `"signal 11"`.
    pub fn crashed(name: &str, detail: &str) -> ActionRun {
        ActionRun {
            name: name.to_string(),
            outcome: Outcome::Failed(format!("Error: {name} \u{2014} Renderer crashed ({detail})")),
            context: name.to_string(),
            checking: name.to_string(),
            images: Vec::new(),
        }
    }

    /// Writes the run to `dir` as `run.json` plus its PNGs, for a parent process
    /// to read back with [`ActionRun::read_from`].
    pub fn write_to(&self, dir: &Path) -> Result<(), RenderError> {
        let io = |source| RenderError::Io { path: dir.to_path_buf(), source };
        fs::create_dir_all(dir).map_err(io)?;
        let outcome = match &self.outcome {
            Outcome::Passed(record) => json!({"result": Value::Object(record.clone())}),
            Outcome::Failed(message) => json!({"message": message}),
        };
        let images: Vec<&str> = self.images.iter().map(|i| i.name.as_str()).collect();
        let doc = json!({"name": self.name, "outcome": outcome, "context": self.context,
                         "checking": self.checking, "images": images});
        for image in &self.images {
            if !is_png_name(&image.name) {
                return Err(RenderError::Evidence(format!("bad image name {:?}", image.name)));
            }
            let path = dir.join(&image.name);
            fs::write(&path, &image.png).map_err(|source| RenderError::Io { path, source })?;
        }
        let path = dir.join("run.json");
        fs::write(&path, to_js_json(&doc)).map_err(|source| RenderError::Io { path, source })
    }

    /// Reads a run written by [`ActionRun::write_to`].
    pub fn read_from(dir: &Path) -> Result<ActionRun, RenderError> {
        let path = dir.join("run.json");
        let bytes = fs::read(&path).map_err(|source| RenderError::Io { path: path.clone(), source })?;
        let doc: Value = serde_json::from_slice(&bytes).map_err(|e| RenderError::Evidence(e.to_string()))?;
        let text = |key: &str| {
            doc.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| RenderError::Evidence(format!("run.json has no `{key}`")))
        };
        let outcome = match (doc.pointer("/outcome/result"), doc.pointer("/outcome/message")) {
            (Some(Value::Object(record)), _) => Outcome::Passed(record.clone()),
            (_, Some(Value::String(message))) => Outcome::Failed(message.clone()),
            _ => return Err(RenderError::Evidence("run.json has no outcome".into())),
        };
        let mut images = Vec::new();
        for name in doc.get("images").and_then(Value::as_array).into_iter().flatten() {
            let name = name.as_str().filter(|n| is_png_name(n)).ok_or_else(|| RenderError::Evidence(format!("bad image {name}")))?;
            let path = dir.join(name);
            let png = fs::read(&path).map_err(|source| RenderError::Io { path, source })?;
            images.push(SheetImage { name: name.to_string(), png });
        }
        Ok(ActionRun { name: text("name")?, outcome, context: text("context")?, checking: text("checking")?, images })
    }
}

/// `[a-z0-9-]+\.png`, the names the evidence sink accepts.
pub(crate) fn is_png_name(name: &str) -> bool {
    name.strip_suffix(".png")
        .is_some_and(|stem| !stem.is_empty() && stem.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'))
}

/// What to produce besides the checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionOptions {
    /// Render and encode the rest, styles and poses PNGs.
    pub sheets: bool,
}

impl Default for ActionOptions {
    fn default() -> ActionOptions {
        ActionOptions { sheets: true }
    }
}

/// Runs every visual check on one action's packaged `.lottie`, in process.
///
/// A native renderer crash would take the whole process down; run this in a
/// child process when isolation matters (see the crate docs). Callers writing
/// evidence check [`crate::probe::require_browser_parity`] first.
pub fn verify_action(set: &SetManifest, meta: &ActionMeta, lottie: &[u8], rules: &SetRules, options: ActionOptions) -> ActionRun {
    let mut check = ActionCheck {
        set,
        meta,
        rules,
        lottie,
        options,
        context: meta.name.clone(),
        checking: meta.name.clone(),
        result: Map::new(),
        images: Vec::new(),
    };
    let outcome = match check.run() {
        Ok(()) => Outcome::Passed(std::mem::take(&mut check.result)),
        Err(message) => Outcome::Failed(message),
    };
    ActionRun { name: meta.name.clone(), outcome, context: check.context, checking: check.checking, images: check.images }
}

/// A failure message the page would record (`String(error)`).
type Failure = String;

fn plain(message: &str) -> Failure {
    format!("Error: {message}")
}

fn distinct(hashes: &[u32]) -> usize {
    hashes.iter().collect::<HashSet<_>>().len()
}

fn width_label(width: f64) -> String {
    js_number_to_string(width)
}

/// Main-matrix hash sequences, keyed by (style, width, theme).
type Sequences = Vec<((Style, u64, Theme), Vec<u32>)>;

struct ActionCheck<'a> {
    set: &'a SetManifest,
    meta: &'a ActionMeta,
    rules: &'a SetRules,
    lottie: &'a [u8],
    options: ActionOptions,
    context: String,
    checking: String,
    result: Map<String, Value>,
    images: Vec<SheetImage>,
}

impl<'a> ActionCheck<'a> {
    /// The page's `check(test, message)`: `Error(context + " — " + message)`.
    fn check(&self, ok: bool, message: impl FnOnce() -> String) -> Result<(), Failure> {
        if ok { Ok(()) } else { Err(self.fail(&message())) }
    }

    fn fail(&self, message: &str) -> Failure {
        format!("Error: {} \u{2014} {}", self.context, message)
    }

    fn paint(&self, player: &mut PaintedPlayer, paint: &Paint, theme_rejected: Option<&'static str>) -> Result<(), Failure> {
        player.paint(paint, theme_rejected).map_err(|message| self.fail(message))
    }

    fn family(&self) -> Result<&'a Family, Failure> {
        self.set
            .family(&self.meta.icon)
            .ok_or_else(|| "TypeError: Cannot read properties of undefined (reading 'motions')".to_string())
    }

    fn run(&mut self) -> Result<(), Failure> {
        let meta = self.meta;
        let actual = sha256_hex(self.lottie);
        self.check(actual == meta.sha256, || format!("Package sha256 {actual} differs from set.json {}", meta.sha256))?;
        self.result.insert("name".into(), json!(meta.name));
        self.result.insert("sha256".into(), json!(meta.sha256));
        self.result.insert("sizes".into(), json!([]));
        self.result.insert("colorOverrides".into(), json!([]));
        self.result.insert("playback".into(), Value::Null);
        self.result.insert("motionId".into(), json!(meta.motion_id));
        for size in SIZES {
            let mut player = PaintedPlayer::open(self.lottie, size).map_err(|_| plain("Load error"))?;
            self.check(f64::from(player.total_frames()) == meta.frames, || "Frame count".into())?;
            self.check(player.animation() == meta.name, || "Download starts with the selected motion".into())?;
            if size == 192 {
                self.motion_switches(&mut player)?;
            }
            let (sample, sequences) = self.paint_matrix(&mut player, size)?;
            self.duotone_policy(&sequences)?;
            self.result.insert("duotoneWholeActionChecked".into(), json!(true));
            if let Some(Value::Array(sizes)) = self.result.get_mut("sizes") {
                sizes.push(sample);
            }
            if size == 192 {
                self.color_overrides(&mut player)?;
                self.style_views(&mut player)?;
                self.rest_and_poses(&mut player)?;
                self.playback(&mut player)?;
            }
        }
        Ok(())
    }

    /// Size-192 prelude: Light vs Strong share a rest pose but not their
    /// action, and stateful forward/reverse actions meet.
    fn motion_switches(&mut self, player: &mut PaintedPlayer) -> Result<(), Failure> {
        let meta = self.meta;
        self.result.insert("motionSwitches".into(), json!([]));
        let family = self.family()?;
        if family.motions.len() < 2 {
            return Err(self.fail(&format!("Family {} needs Light and Strong motions", family.name)));
        }
        let max_active = family.motions.iter().map(|m| m.active).fold(f64::NEG_INFINITY, f64::max);
        let switch_end = (max_active * meta.fps).ceil();
        let mut switches = Vec::new();
        for theme in Theme::ALL {
            for &style in &meta.styles {
                let mut rests = Vec::new();
                let mut actions = Vec::new();
                for variant in &family.motions {
                    player.select(&variant.name);
                    self.check(player.animation() == variant.name, || "Motion switch rejected".into())?;
                    self.paint(player, &Paint::bind_style(theme, style, 2.0), Some("Variant theme rejected"))?;
                    player.set_segment(0.0, variant.frames - 1.0);
                    player.set_frame(variant.rest);
                    rests.push(player.rgba().to_vec());
                    let mut samples = Vec::new();
                    let mut f = 0.0;
                    while f <= switch_end {
                        player.set_frame(f);
                        samples.push(fnv1a(player.rgba()));
                        f += 3.0;
                    }
                    actions.push(samples);
                }
                if !meta.continuous {
                    self.check(equivalent(&rests[0], &rests[1], meta.is_cyclic()), || {
                        "Motion variants change the rest drawing".into()
                    })?;
                } else {
                    let readable = rests.iter().all(|r| bounds(r, 192).n > 0);
                    self.check(readable, || self.rules.loader.empty_rest.into())?;
                }
                self.check(actions[0] != actions[1], || "Motion variants have identical action pixels".into())?;
                switches.push(json!({"theme": theme.id(), "style": style.name(), "sharedRest": !meta.continuous,
                                     "readableRest": true, "restRasterDifference": js_number(raster_difference(&rests[0], &rests[1])),
                                     "distinctAction": true}));
            }
        }
        self.result.insert("motionSwitches".into(), Value::Array(switches));
        if family.stateful {
            self.result.insert("stateContinuity".into(), json!([]));
            let mut continuity = Vec::new();
            for variant in &family.motions {
                for &style in &meta.styles {
                    let reverse = variant
                        .reverse
                        .as_deref()
                        .ok_or_else(|| "TypeError: Cannot read properties of undefined (reading 'name')".to_string())?;
                    let mut ends = Vec::new();
                    for action in [variant, reverse] {
                        player.select(&action.name);
                        self.paint(player, &Paint::bind_style(Theme::Dark, style, 2.0), None)?;
                        player.set_segment(0.0, action.frames - 1.0);
                        player.set_frame(0.0);
                        let first = fnv1a(player.rgba());
                        player.set_frame(action.rest);
                        ends.push((first, fnv1a(player.rgba())));
                    }
                    self.check(ends[0].0 == ends[1].1 && ends[0].1 == ends[1].0, || {
                        "Forward and reverse states do not meet".into()
                    })?;
                    continuity.push(json!({"motion": variant.motion_id, "style": style.name(), "matched": true}));
                }
            }
            self.result.insert("stateContinuity".into(), Value::Array(continuity));
        }
        player.select(&meta.name);
        player.set_segment(0.0, meta.frames - 1.0);
        Ok(())
    }

    /// `result[key] = Math.min(result[key] ?? Infinity, value)`.
    fn keep_min(&mut self, key: &str, value: u32) {
        let current = self.result.get(key).and_then(Value::as_u64).unwrap_or(u64::MAX);
        self.result.insert(key.into(), json!(current.min(u64::from(value))));
    }

    /// `result[key] = Math.max(result[key] ?? 0, value)`.
    fn keep_max(&mut self, key: &str, value: u32) {
        let current = self.result.get(key).and_then(Value::as_u64).unwrap_or(0);
        self.result.insert(key.into(), json!(current.max(u64::from(value))));
    }

    /// Per-set raster rules, run on every main-matrix frame after the buffer
    /// size check.
    fn frame_rules(&mut self, rgba: &[u8], size: u32, style: Style, frame: u32) -> Result<(), Failure> {
        let meta = self.meta;
        if let Some(rule) = self.rules.native_counters
            && let Some(minimum) = rule.minimum(&meta.icon, style, size)
        {
            let count = counters(rgba, size, 1);
            self.check(count >= minimum, || format!("{} loses a native opening at frame {frame} / {size}px", meta.icon))?;
            self.keep_min("minimumNativeCounters", count);
        }
        if self.rules.keyframe_openings && meta.icon == "keyframes" && style == Style::Outline {
            let forward = meta.direction == "forward";
            let strong = meta.motion_id == "strong";
            let mut centers = vec![10.5];
            if forward && frame >= if strong { 30 } else { 14 } {
                centers.push(21.5);
            }
            for x in centers {
                let row = (20.0 * f64::from(size) / 32.0).floor() as usize;
                let col = (x * f64::from(size) / 32.0).floor() as usize;
                let alpha = rgba[(row * size as usize + col) * 4 + 3];
                self.check(alpha < 96, || "Rail or stroke closes a keyframe counter".into())?;
                self.keep_max("maximumCounterCenterAlpha", u32::from(alpha));
            }
            if size >= 32 && forward {
                let settled = frame >= if strong { 38 } else { 21 };
                let openings = counters(rgba, size, if size == 32 { 1 } else { 4 });
                self.check(openings >= if settled { 2 } else { 1 }, || format!("Missing enclosed keyframe opening at frame {frame}"))?;
            }
        }
        if self.rules.solid_counters && size == 192 && style == Style::Solid && (meta.icon == "credit-card" || meta.icon == "wallet") {
            let count = counters(rgba, size, 4);
            self.check(count >= 2, || format!("An enclosed Solid detail is painted over: {}", meta.name))?;
            self.keep_min("minimumSolidCounters", count);
        }
        Ok(())
    }

    /// The 14-paint × every-frame matrix at one size. Returns the size sample
    /// and the hash sequences for the duotone policy.
    fn paint_matrix(&mut self, player: &mut PaintedPlayer, size: u32) -> Result<(Value, Sequences), Failure> {
        let meta = self.meta;
        let mut themes = Vec::new();
        let mut sequences = Sequences::new();
        for &style in &meta.styles {
            let widths: &[f64] = if style == Style::Solid { &[2.0] } else { &[1.5, 2.0, 2.5] };
            for &width in widths {
                for theme in Theme::ALL {
                    let w = width_label(width);
                    self.checking = format!("Checking {} / {} / {w} / {size} px / {}", meta.name, style.name(), theme.id());
                    self.paint(player, &Paint::bind_style(theme, style, width), Some("Theme rejected"))?;
                    self.context = format!("{} / {} / {w} / {size} / {}", meta.name, style.name(), theme.id());
                    let (sample, hashes) = self.frames(player, size, style, theme, width)?;
                    themes.push(sample);
                    sequences.push(((style, width.to_bits(), theme), hashes));
                }
            }
        }
        Ok((json!({"size": size, "themes": themes}), sequences))
    }

    /// Every frame of one paint plus the rest frame, and the sample record.
    fn frames(&mut self, player: &mut PaintedPlayer, size: u32, style: Style, theme: Theme, width: f64) -> Result<(Value, Vec<u32>), Failure> {
        let meta = self.meta;
        let expected_len = (size * size * 4) as usize;
        let mut hashes = Vec::new();
        let mut first: Option<Vec<u8>> = None;
        let mut previous: Option<Vec<u8>> = None;
        let mut previous_hash = None;
        let (mut max_adjacent, mut still, mut max_still) = (0.0f64, 0u32, 0u32);
        let mut clearance = i64::MAX;
        let mut frame = 0u32;
        while f64::from(frame) < meta.frames {
            player.set_frame(f64::from(frame));
            let rgba = player.rgba();
            let hash = fnv1a(rgba);
            still = if previous_hash == Some(hash) { still + 1 } else { 0 };
            max_still = max_still.max(still);
            previous_hash = Some(hash);
            match &mut previous {
                Some(prev) => {
                    max_adjacent = max_adjacent.max(raster_difference(prev, rgba));
                    prev.copy_from_slice(rgba);
                }
                None => previous = Some(rgba.to_vec()),
            }
            if frame == 0 {
                first = Some(rgba.to_vec());
            }
            self.check(rgba.len() == expected_len, || "Buffer size".into())?;
            self.frame_rules(rgba, size, style, frame)?;
            let b = bounds(rgba, size);
            self.check(b.n > 0 || meta.is_reset_reverse(), || "Invisible main action".into())?;
            let inside = b.left > 0 && b.top > 0 && b.right < i64::from(size) - 1 && b.bottom < i64::from(size) - 1;
            self.check(b.n == 0 || inside, || "Clipped icon".into())?;
            clearance = clearance.min(b.edge_clearance(size));
            hashes.push(hash);
            frame += 1;
        }
        let (Some(first), Some(last)) = (first, previous) else {
            return Err(self.fail("Frame count"));
        };
        player.set_frame(meta.rest);
        let rest = player.rgba().to_vec();
        let rest_hash = fnv1a(&rest);
        let count = distinct(&hashes);
        let mut wrap = 0.0;
        if meta.continuous {
            let loader = self.rules.loader;
            wrap = raster_difference(&last, &first);
            self.check(wrap <= max_adjacent * 1.5 + 0.0005, || "Discontinuous loop boundary".into())?;
            self.check(max_still < 5, || loader.pauses.into())?;
            self.check(count >= loader.distinct_minimum(&meta.icon), || {
                if loader.count_in_message {
                    format!("Insufficient continuous motion: {count}")
                } else {
                    "Insufficient continuous motion".into()
                }
            })?;
        } else {
            let boundary_ok = hashes.last() == Some(&rest_hash)
                && if meta.stateful { hashes[0] != rest_hash } else { equivalent(&first, &rest, meta.is_cyclic()) };
            self.check(boundary_ok, || {
                format!(
                    "Loop or state boundary is incorrect (premultiplied raster difference: {})",
                    js_number_to_string(raster_difference(&first, &rest))
                )
            })?;
            let need = if meta.is_reset_reverse() {
                4.0
            } else {
                match meta.icon.as_str() {
                    "image-caption" => 12.0,
                    "code-generate" => 10.0,
                    // Math.min(16, Math.max(6, x)); NaN stays NaN and fails the check.
                    _ => (meta.active * meta.fps * 0.5).floor().clamp(6.0, 16.0),
                }
            };
            self.check(count as f64 >= need, || format!("Insufficient animation states: {count}"))?;
            let hold_from = meta.rest.ceil().max(0.0) as usize;
            self.check(hashes.iter().skip(hold_from).all(|&h| h == rest_hash), || "Hold is not still".into())?;
        }
        let sample = json!({
            "theme": theme.id(), "style": style.name(), "width": js_number(width),
            "frames": hashes.len(), "distinctFrames": count, "restHash": rest_hash,
            "boundaryRasterDifference": js_number(raster_difference(&first, &rest)),
            "restPixels": bounds(&rest, size).n, "minimumEdgeClearance": clearance,
            "continuous": meta.continuous, "wrapDifference": js_number(wrap),
            "maxAdjacentDifference": js_number(max_adjacent), "maxStillRun": max_still,
            "loopBoundaryMatches": !meta.stateful, "finalStateHeld": !meta.continuous, "holdIsStill": !meta.continuous,
        });
        Ok((sample, hashes))
    }

    /// Outline and Duotone differ somewhere in the action iff the icon declares
    /// a duotone fill.
    fn duotone_policy(&self, sequences: &Sequences) -> Result<(), Failure> {
        let find = |style: Style, width: f64, theme: Theme| {
            sequences.iter().find(|(key, _)| *key == (style, width.to_bits(), theme)).map(|(_, h)| h)
        };
        for width in [1.5, 2.0, 2.5] {
            for theme in Theme::ALL {
                let (Some(outline), Some(duotone)) = (find(Style::Outline, width, theme), find(Style::Duotone, width, theme)) else {
                    return Err(self.fail("Duotone policy needs the outline and duotone paints"));
                };
                let changed = outline.iter().enumerate().any(|(i, h)| duotone.get(i) != Some(h));
                self.check(changed == self.meta.expects_duotone_fill(), || "Duotone policy disagrees with the full action".into())?;
            }
        }
        Ok(())
    }

    /// A primary-color override changes the rest frame in every style and theme.
    fn color_overrides(&mut self, player: &mut PaintedPlayer) -> Result<(), Failure> {
        let meta = self.meta;
        let mut overrides = Vec::new();
        for &style in &meta.styles {
            for theme in Theme::ALL {
                let base = Paint::bind_style(theme, style, 2.0);
                self.paint(player, &base, None)?;
                player.set_frame(meta.rest);
                let before = fnv1a(player.rgba());
                let rgba = match theme {
                    Theme::Dark => [0.35, 0.89, 0.91, 1.0],
                    Theme::Light => [0.03, 0.49, 0.53, 1.0],
                };
                self.paint(player, &base.with_color(rgba), None)?;
                player.set_frame(meta.rest);
                self.check(before != fnv1a(player.rgba()), || "Color override has no visible effect".into())?;
                overrides.push(json!({"theme": theme.id(), "style": style.name(), "changedPixels": true}));
            }
        }
        self.result.insert("colorOverrides".into(), Value::Array(overrides));
        Ok(())
    }

    /// Light, Regular, Bold and Solid differ at the probe frame; `styles.png`.
    fn style_views(&mut self, player: &mut PaintedPlayer) -> Result<(), Failure> {
        let meta = self.meta;
        self.paint(player, &Paint::bind_style(Theme::Dark, Style::Outline, 2.0), None)?;
        player.set_frame(meta.rest);
        let views = [(Style::Outline, 1.5), (Style::Outline, 2.0), (Style::Outline, 2.5), (Style::Solid, 2.0), (Style::Duotone, 2.0)];
        let probe = if meta.is_reset_reverse() { 0.0 } else { meta.rest };
        let mut hashes = Vec::new();
        let mut tiles = Vec::new();
        for (style, width) in views {
            self.paint(player, &Paint::bind_style(Theme::Dark, style, width), None)?;
            player.set_frame(probe);
            hashes.push(fnv1a(player.rgba()));
            tiles.push(player.rgba().to_vec());
        }
        self.check(distinct(&hashes[..4]) == 4, || {
            let listed: Vec<Value> = views
                .iter()
                .zip(STYLE_LABELS)
                .zip(&hashes)
                .map(|(((style, width), label), hash)| json!({"view": [style.name(), js_number(*width), label], "hash": hash}))
                .collect();
            format!("Style or weight has no visible effect: {}", to_js_json(&Value::Array(listed)))
        })?;
        self.check(meta.expects_duotone_fill() || hashes[4] == hashes[1], || "Open icon has a final-frame tint".into())?;
        self.result.insert("duotoneFill".into(), json!(meta.expects_duotone_fill()));
        self.result.insert("styleAndWeightDistinct".into(), json!(true));
        self.result.insert("styleProbeFrame".into(), js_number(probe));
        if self.options.sheets {
            let png = sheets::styles_png(&tiles, &STYLE_LABELS).map_err(|e| plain(&e.to_string()))?;
            self.images.push(SheetImage { name: format!("{}-styles.png", meta.name), png });
        }
        Ok(())
    }

    /// `rest.png` (dark Regular outline at the rest frame) and `poses.png`.
    fn rest_and_poses(&mut self, player: &mut PaintedPlayer) -> Result<(), Failure> {
        let meta = self.meta;
        self.paint(player, &Paint::bind_style(Theme::Dark, Style::Outline, 2.0), None)?;
        player.set_frame(meta.rest);
        let rest = player.rgba().to_vec();
        let mut poses = Vec::new();
        for &time in &meta.key_poses {
            player.set_frame(time * 60.0);
            poses.push((time, player.rgba().to_vec()));
        }
        if self.options.sheets {
            let rest_png = sheets::rest_png(&rest, 192).map_err(|e| plain(&e.to_string()))?;
            self.images.push(SheetImage { name: format!("{}-rest.png", meta.name), png: rest_png });
            let poses_png = sheets::poses_png(&poses).map_err(|e| plain(&e.to_string()))?;
            self.images.push(SheetImage { name: format!("{}-poses.png", meta.name), png: poses_png });
        }
        Ok(())
    }

    /// Plays the action on a deterministic 60 Hz clock: one-shot actions must
    /// complete, loaders must wrap twice and park at their rest frame.
    fn playback(&mut self, player: &mut PaintedPlayer) -> Result<(), Failure> {
        let meta = self.meta;
        player.drain_events();
        let mut seen = Vec::new();
        let mut wraps = 0u32;
        let speed = if meta.continuous {
            self.controller_playback(player, &mut seen, &mut wraps)?;
            self.rules.loader.speed
        } else {
            player.set_segment(0.0, (meta.active * 60.0).ceil());
            player.set_frame(0.0);
            collect_frames(player, &mut seen);
            player.play();
            let budget = (5000.0 / TICK_MS).ceil() as u32;
            let mut completed = false;
            for _ in 0..budget {
                player.tick(TICK_MS);
                while let Some(event) = player.poll_event() {
                    match event {
                        PlayerEvent::Frame { frame_no } => seen.push(frame_no),
                        PlayerEvent::Complete => completed = true,
                        _ => {}
                    }
                }
                if completed {
                    break;
                }
            }
            if !completed {
                return Err(plain("Playback completion timeout"));
            }
            1.0
        };
        player.pause();
        self.check(seen.len() > 5, || "No real playback progression".into())?;
        let mut playback = json!({"completed": true, "speed": js_number(f64::from(speed)), "nativeLoopEvents": wraps,
                                  "observedFrameEvents": seen.len()});
        if let (Some(last), Value::Object(map)) = (seen.last(), &mut playback) {
            map.insert("lastFrame".into(), js_number(f64::from(*last)));
        }
        self.result.insert("playback".into(), playback);
        Ok(())
    }

    /// The runtime `IconController` driving a loader: construct (load + rest),
    /// set the speed, `play('forward', {repeat: () => wraps < 2})`.
    fn controller_playback(&mut self, player: &mut PaintedPlayer, seen: &mut Vec<f32>, wraps: &mut u32) -> Result<(), Failure> {
        let meta = self.meta;
        let family = self.family()?;
        let motion = family
            .motions
            .iter()
            .find(|m| m.motion_id == meta.motion_id)
            .or(family.motions.first())
            .ok_or_else(|| self.fail("Family has no motions"))?;
        let loader = self.rules.loader;
        self.controller_load(player, motion, 1.0)?;
        controller_rest(player, motion);
        player.set_speed(loader.speed);
        // play(): cancel, load, then start the segment from frame 0.
        player.pause();
        self.controller_load(player, motion, loader.speed)?;
        let end = if motion.continuous {
            motion.frames - 1.0
        } else {
            (motion.frames - 1.0).min((motion.active * motion.fps).ceil())
        };
        player.drain_events();
        player.set_segment(0.0, end);
        player.set_loop(motion.continuous);
        player.set_frame(0.0);
        player.play();
        let mut running = true;
        let budget = (loader.timeout_ms / TICK_MS).ceil() as u32;
        for _ in 0..budget {
            player.tick(TICK_MS);
            while let Some(event) = player.poll_event() {
                match event {
                    PlayerEvent::Frame { frame_no } => seen.push(frame_no),
                    PlayerEvent::Loop { .. } => {
                        *wraps += 1;
                        if running && *wraps >= 2 {
                            running = false;
                            controller_rest(player, motion);
                        }
                    }
                    PlayerEvent::Complete if running => {
                        running = false;
                        controller_rest(player, motion);
                    }
                    _ => {}
                }
            }
            if !running {
                break;
            }
        }
        collect_frames(player, seen);
        if running {
            return Err(plain("Native controller loop timeout"));
        }
        self.check(player.current_frame() == 0.0, || "Native loader fails to finish at its rest pose".into())?;
        player.pause();
        player.set_speed(1.0);
        Ok(())
    }

    /// `IconController.load(meta)`.
    fn controller_load(&self, player: &mut PaintedPlayer, motion: &Motion, speed: f32) -> Result<(), Failure> {
        player.select(&motion.name);
        if player.animation() != motion.name {
            return Err(plain(&format!("Animation selection failed: {}", motion.name)));
        }
        player.set_speed(speed);
        player.set_loop(false);
        player.set_segment(0.0, motion.frames - 1.0);
        self.paint(player, &Paint::bind_style(Theme::Dark, Style::Outline, 2.0), None)
    }
}

/// `IconController.rest()`.
fn controller_rest(player: &mut PaintedPlayer, motion: &Motion) {
    player.pause();
    player.set_loop(false);
    player.set_segment(0.0, motion.frames - 1.0);
    player.set_frame(motion.rest);
}

fn collect_frames(player: &mut PaintedPlayer, seen: &mut Vec<f32>) {
    while let Some(event) = player.poll_event() {
        if let PlayerEvent::Frame { frame_no } = event {
            seen.push(frame_no);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_numbers_reduce_like_json_stringify() {
        assert_eq!(js_number(2.0), json!(2));
        assert_eq!(js_number(-0.0), json!(0));
        assert_eq!(js_number(1.5), json!(1.5));
        assert_eq!(js_number(f64::NAN), Value::Null);
        assert_eq!(gai_core::to_py_json(&js_number(9.679718454307096e-05), None, true), "9.679718454307096e-05");
        assert_eq!(gai_core::to_py_json(&js_number(31.200000000000003), None, true), "31.200000000000003");
    }

    #[test]
    fn png_names_follow_the_sink() {
        assert!(is_png_name("partly-cloudy-rest.png"));
        assert!(!is_png_name("Partly.png") && !is_png_name(".png") && !is_png_name("a/b.png") && !is_png_name("a.jpg"));
    }

    #[test]
    fn action_runs_round_trip_through_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let mut record = Map::new();
        record.insert("name".into(), json!("x"));
        record.insert("maxAdjacentDifference".into(), json!(0.00190113845102311));
        let run = ActionRun {
            name: "x".into(),
            outcome: Outcome::Passed(record),
            context: "x / duotone / 2.5 / 192 / light".into(),
            checking: "Checking x / duotone / 2.5 / 192 px / light".into(),
            images: vec![SheetImage { name: "x-rest.png".into(), png: vec![1, 2, 3] }],
        };
        run.write_to(dir.path()).unwrap();
        assert_eq!(ActionRun::read_from(dir.path()).unwrap(), run);
        let crashed = ActionRun::crashed("x", "signal 11");
        assert_eq!(crashed.outcome, Outcome::Failed("Error: x \u{2014} Renderer crashed (signal 11)".into()));
    }
}
