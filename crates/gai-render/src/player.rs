//! A dotlottie-rs player that applies paints the way dotlottie-web does,
//! without the native crash.
//!
//! dotlottie-rs at this pin segfaults on the second slot flush after a render
//! (for example `set_theme → render → set_slot_str → render`); the WASM build
//! does not. [`PaintedPlayer`] therefore flushes slots at most once per load
//! and reloads the animation for every further paint change. A frame's pixels
//! depend only on (animation, effective slot values, frame), so the output is
//! identical to the browser's.

use crate::RenderError;
use dotlottie_rs::{ColorSpace, Player, PlayerEvent, Segment};
use gai_core::js_number_to_string;
use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, Once, PoisonError};

/// A theme bundled in every icon `.lottie` (`t/dark.json`, `t/light.json`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Theme {
    /// `dark`: the primary color for dark backgrounds.
    Dark,
    /// `light`: the primary color for light backgrounds.
    Light,
}

impl Theme {
    /// Both themes in the order the checks visit them.
    pub const ALL: [Theme; 2] = [Theme::Dark, Theme::Light];

    /// The theme id in the `.lottie` manifest.
    pub fn id(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    /// Parses a theme id.
    pub fn parse(id: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.id() == id)
    }
}

/// An icon style, selected through opacity slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    /// Stroked outline at one of the authored weights.
    Outline,
    /// Filled drawing; ignores the weight.
    Solid,
    /// Outline plus an 18 % interior tint.
    Duotone,
}

impl Style {
    /// Every style, in the catalog's order.
    pub const ALL: [Style; 3] = [Style::Outline, Style::Solid, Style::Duotone];

    /// The style name used in set.json and evidence.
    pub fn name(self) -> &'static str {
        match self {
            Style::Outline => "outline",
            Style::Solid => "solid",
            Style::Duotone => "duotone",
        }
    }

    /// Parses a style name.
    pub fn parse(name: &str) -> Option<Style> {
        Style::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// The authored outline weights and their opacity slots.
pub const AUTHORED_WEIGHTS: [(f64, &str); 3] =
    [(1.5, "style.outline.light"), (2.0, "style.outline.regular"), (2.5, "style.outline.bold")];

/// One pending `set_slot_str` call and the message the page reports when the
/// player rejects it.
#[derive(Clone, Debug, PartialEq)]
struct SlotWrite {
    id: &'static str,
    json: String,
    rejected: &'static str,
}

/// A theme plus slot overrides: everything a rendered pixel depends on besides
/// the animation and the frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    theme: Theme,
    writes: Vec<SlotWrite>,
}

fn scalar(id: &'static str, value: f64, rejected: &'static str) -> SlotWrite {
    // JSON.stringify({a: 0, k: value})
    SlotWrite { id, json: format!("{{\"a\":0,\"k\":{}}}", js_number_to_string(value)), rejected }
}

fn color(rgba: [f64; 4], rejected: &'static str) -> SlotWrite {
    let k: Vec<String> = rgba.iter().map(|&c| js_number_to_string(c)).collect();
    SlotWrite { id: "color.primary", json: format!("{{\"a\":0,\"k\":[{}]}}", k.join(",")), rejected }
}

impl Paint {
    /// verify.html's `setTheme(theme)` followed by `bindStyle(style, width)`.
    /// Solid ignores `width`; `style.outline.width` keeps the theme's value.
    pub fn bind_style(theme: Theme, style: Style, width: f64) -> Paint {
        const REJECTED: [&str; 3] =
            ["Weight paint rejected: light", "Weight paint rejected: regular", "Weight paint rejected: bold"];
        let mut writes: Vec<SlotWrite> = AUTHORED_WEIGHTS
            .iter()
            .zip(REJECTED)
            .map(|(&(w, id), rejected)| scalar(id, if style != Style::Solid && width == w { 100.0 } else { 0.0 }, rejected))
            .collect();
        writes.push(scalar("style.solid", if style == Style::Solid { 100.0 } else { 0.0 }, "Solid slot rejected"));
        writes.push(scalar("style.tint", if style == Style::Duotone { 18.0 } else { 0.0 }, "Tint slot rejected"));
        Paint { theme, writes }
    }

    /// Adds verify.html's `setColorSlot("color.primary", rgba)`.
    pub fn with_color(mut self, rgba: [f64; 4]) -> Paint {
        self.writes.push(color(rgba, "Color override rejected"));
        self
    }

    /// The runtime's `paintIcon` (packages/runtime): authored weights select
    /// their layers; any other weight shows Regular and sets `style.outline.width`.
    /// `color` overrides the theme's primary color when given.
    pub fn runtime(look: &Look) -> Paint {
        let authored = is_authored_weight(look.weight);
        let mut writes = Vec::new();
        if let Some([r, g, b]) = look.color {
            writes.push(color([f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0, 1.0], "Color slot rejected"));
        }
        for (w, id) in AUTHORED_WEIGHTS {
            let on = look.style != Style::Solid && if authored { look.weight == w } else { w == 2.0 };
            writes.push(scalar(id, if on { 100.0 } else { 0.0 }, "Weight slot rejected"));
        }
        writes.push(scalar("style.outline.width", if authored { 2.0 } else { look.weight }, "Stroke width slot rejected"));
        writes.push(scalar("style.solid", if look.style == Style::Solid { 100.0 } else { 0.0 }, "Solid slot rejected"));
        writes.push(scalar("style.tint", if look.style == Style::Duotone { 18.0 } else { 0.0 }, "Tint slot rejected"));
        Paint { theme: look.theme, writes }
    }

    /// The theme this paint starts from.
    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// The slot writes in call order: `(slot id, JSON value)`.
    pub fn slot_json(&self) -> impl Iterator<Item = (&'static str, &str)> {
        self.writes.iter().map(|w| (w.id, w.json.as_str()))
    }
}

/// How a preview is painted: the runtime's options (`paintIcon`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Theme id to apply.
    pub theme: Theme,
    /// Icon style.
    pub style: Style,
    /// Outline weight from 1 to 3; 1.5, 2 and 2.5 are the authored layers.
    pub weight: f64,
    /// Primary color override (sRGB bytes); `None` keeps the theme's color.
    pub color: Option<[u8; 3]>,
}

impl Look {
    /// Outline at the Regular weight in the given theme.
    pub fn regular(theme: Theme) -> Look {
        Look { theme, style: Style::Outline, weight: 2.0, color: None }
    }
}

/// Whether `weight` selects an authored outline layer (1.5, 2 or 2.5).
pub fn is_authored_weight(weight: f64) -> bool {
    AUTHORED_WEIGHTS.iter().any(|&(w, _)| w == weight)
}

/// ThorVG keeps process-wide state that is not thread-safe without its thread
/// feature (the Lottie expression engine, the renderer registry), so every
/// call into it is serialized across threads.
static ENGINE: Mutex<()> = Mutex::new(());

thread_local! {
    static ENGINE_HELD: Cell<bool> = const { Cell::new(false) };
}

/// Exclusive use of the engine; re-entrant on the holding thread.
pub(crate) struct EngineGuard(Option<MutexGuard<'static, ()>>);

impl Drop for EngineGuard {
    fn drop(&mut self) {
        if self.0.is_some() {
            ENGINE_HELD.set(false);
        }
    }
}

pub(crate) fn engine() -> EngineGuard {
    if ENGINE_HELD.get() {
        return EngineGuard(None);
    }
    let guard = ENGINE.lock().unwrap_or_else(PoisonError::into_inner);
    ENGINE_HELD.set(true);
    EngineGuard(Some(guard))
}

/// A software render target: a player bound to a fixed pixel buffer.
///
/// Callers hold [`engine`] around every use of `player`.
pub(crate) struct Canvas {
    // Dropped explicitly (under the engine lock) before `target`, which the
    // player's canvas points into.
    pub(crate) player: ManuallyDrop<Player>,
    target: Box<[u32]>,
    bytes: Vec<u8>,
    size: u32,
}

impl Canvas {
    /// A player configured like verify.html's: DPR 1, no autoplay or loop, no
    /// frame interpolation, default `contain` layout, transparent background.
    /// The caller holds [`engine`].
    pub(crate) fn new(size: u32) -> Result<Canvas, RenderError> {
        if size == 0 || size > 4096 {
            return Err(RenderError::Size(size));
        }
        // ThorVG terminates its engine when the last renderer drops, freeing
        // every thread's memory pool but clearing only the calling thread's
        // pointer to it; another thread would then render into freed memory.
        // One player that is never dropped keeps the engine up for good.
        static KEEP_ENGINE: Once = Once::new();
        KEEP_ENGINE.call_once(|| std::mem::forget(Player::new()));
        let pixels = (size * size) as usize;
        let mut canvas = Canvas {
            player: ManuallyDrop::new(Player::new()),
            target: vec![0u32; pixels].into_boxed_slice(),
            bytes: vec![0u8; pixels * 4],
            size,
        };
        canvas.player.set_use_frame_interpolation(false);
        canvas
            .player
            .set_sw_target(&mut canvas.target, size, size, ColorSpace::ABGR8888S)
            .map_err(|_| RenderError::Target)?;
        Ok(canvas)
    }

    /// The target as RGBA bytes with straight alpha (ABGR8888S, little-endian words).
    pub(crate) fn rgba(&mut self) -> &[u8] {
        for (out, word) in self.bytes.chunks_exact_mut(4).zip(self.target.iter()) {
            out.copy_from_slice(&word.to_le_bytes());
        }
        &self.bytes
    }

    /// dotlottie-web's `setFrame`: re-renders only when the frame was accepted
    /// (inside the segment and different from the current frame). The caller
    /// holds [`engine`].
    pub(crate) fn set_frame(&mut self, frame: f64) -> bool {
        let accepted = self.player.set_frame(frame as f32).is_ok();
        if accepted {
            // Err only means "nothing changed".
            let _ = self.player.render();
        }
        accepted
    }

    /// Frames per second of the loaded animation (60 when unknown). The caller
    /// holds [`engine`].
    pub(crate) fn fps(&self) -> f64 {
        // dotlottie-rs reports the duration in milliseconds.
        let duration_ms = f64::from(self.player.duration());
        if duration_ms > 0.0 { f64::from(self.player.total_frames()) * 1000.0 / duration_ms } else { 60.0 }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        let _engine = engine();
        // SAFETY: dropped exactly once, here; the field is never used afterwards.
        unsafe { ManuallyDrop::drop(&mut self.player) };
    }
}

fn c_string(value: &str) -> Result<CString, RenderError> {
    CString::new(value).map_err(|_| RenderError::Name(value.to_string()))
}

/// A player for one `.lottie` at one size that mirrors dotlottie-web's call
/// semantics (verify.html's `playerFor`) and never flushes slots twice per load.
///
/// Players may live on any number of threads; their engine calls are
/// serialized process-wide.
pub struct PaintedPlayer {
    canvas: Canvas,
    animation: String,
    applied: Option<Paint>,
    flushed_since_load: bool,
    segment: Option<Segment>,
}

impl PaintedPlayer {
    /// Loads a `.lottie` archive at `size`×`size` pixels. The manifest's initial
    /// animation is selected; no theme is applied yet.
    pub fn open(lottie: &[u8], size: u32) -> Result<PaintedPlayer, RenderError> {
        let _engine = engine();
        let mut canvas = Canvas::new(size)?;
        canvas.player.load_dotlottie_data(lottie).map_err(|_| RenderError::Load)?;
        let animation = canvas
            .player
            .animation_id()
            .and_then(|id| id.to_str().ok())
            .map(str::to_string)
            .ok_or(RenderError::Load)?;
        Ok(PaintedPlayer { canvas, animation, applied: None, flushed_since_load: false, segment: None })
    }

    /// The active animation id (`activeAnimationId`).
    pub fn animation(&self) -> &str {
        &self.animation
    }

    /// Frames in the active animation (`totalFrames`).
    pub fn total_frames(&self) -> f32 {
        let _engine = engine();
        self.canvas.player.total_frames()
    }

    /// Frames per second of the active animation.
    pub fn fps(&self) -> f64 {
        let _engine = engine();
        self.canvas.fps()
    }

    /// The current frame as dotlottie-web reports it: `Math.round(frame * 100) / 100`.
    pub fn current_frame(&self) -> f64 {
        let _engine = engine();
        gai_core::js_round(f64::from(self.canvas.player.current_frame()) * 100.0) / 100.0
    }

    /// The raster size in pixels.
    pub fn size(&self) -> u32 {
        self.canvas.size
    }

    /// dotlottie-web's `loadAnimation(id)`: a no-op for the active id. A failed
    /// load leaves the previous animation active, so callers check [`Self::animation`].
    pub fn select(&mut self, id: &str) {
        if self.animation == id {
            return;
        }
        let _engine = engine();
        let loaded = c_string(id).is_ok_and(|cid| self.canvas.player.load_animation(&cid).is_ok());
        if loaded {
            self.animation = id.to_string();
            self.flushed_since_load = false;
            self.segment = None;
        }
        self.applied = None;
    }

    /// Applies a paint at the current frame, like the page's `setTheme` and slot
    /// calls. `theme_rejected` is the page's message when it checks `setTheme`
    /// (`None` where the page ignores the result). Returns the page's message
    /// for the first rejected call.
    pub fn paint(&mut self, paint: &Paint, theme_rejected: Option<&'static str>) -> Result<(), &'static str> {
        if self.applied.as_ref() == Some(paint) {
            return Ok(());
        }
        let _engine = engine();
        let player = &mut self.canvas.player;
        if self.flushed_since_load {
            // A second flush after a render crashes natively: reload instead,
            // then restore the segment and frame the page would still have.
            let frame = player.current_frame();
            let id = c_string(&self.animation).map_err(|_| "Load error")?;
            player.load_animation(&id).map_err(|_| "Load error")?;
            if self.segment.is_some() {
                let _ = player.set_segment(self.segment);
            }
            let _ = player.set_frame(frame);
            self.flushed_since_load = false;
        }
        self.applied = None;
        let theme = c_string(paint.theme.id()).map_err(|_| "Theme rejected")?;
        if player.set_theme(&theme).is_err()
            && let Some(message) = theme_rejected
        {
            return Err(message);
        }
        for write in &paint.writes {
            player.set_slot_str(write.id, &write.json).map_err(|_| write.rejected)?;
        }
        // The single flush for this load; Err only means "nothing changed".
        let _ = player.render();
        self.flushed_since_load = true;
        self.applied = Some(paint.clone());
        Ok(())
    }

    /// dotlottie-web's `setSegment(start, end)`; ignored unless `start < end`.
    pub fn set_segment(&mut self, start: f64, end: f64) {
        let _engine = engine();
        let segment = Some(Segment { start: start as f32, end: end as f32 });
        if self.canvas.player.set_segment(segment).is_ok() {
            self.segment = segment;
        }
    }

    /// dotlottie-web's `setFrame(frame)`. Out-of-segment or unchanged frames are
    /// rejected and leave the last rendered pixels in place.
    pub fn set_frame(&mut self, frame: f64) -> bool {
        let _engine = engine();
        self.canvas.set_frame(frame)
    }

    /// The rendered pixels: straight RGBA, `size * size * 4` bytes.
    pub fn rgba(&mut self) -> &[u8] {
        self.canvas.rgba()
    }

    /// Starts playback (`play()`); a no-op when already playing.
    pub fn play(&mut self) {
        let _engine = engine();
        let _ = self.canvas.player.play();
    }

    /// Pauses playback (`pause()`); a no-op when not playing.
    pub fn pause(&mut self) {
        let _engine = engine();
        let _ = self.canvas.player.pause();
    }

    /// `setLoop(value)`.
    pub fn set_loop(&mut self, value: bool) {
        self.canvas.player.set_loop(value);
    }

    /// `setSpeed(value)`.
    pub fn set_speed(&mut self, value: f32) {
        self.canvas.player.set_speed(value);
    }

    /// Advances the playback clock by `ms` milliseconds and renders when the
    /// frame changed (one animation-frame callback).
    pub fn tick(&mut self, ms: f32) {
        let _engine = engine();
        let _ = self.canvas.player.tick(ms);
    }

    /// Discards queued player events.
    pub fn drain_events(&mut self) {
        while self.canvas.player.poll_event().is_some() {}
    }

    /// The next queued player event (`Frame`, `Loop`, `Complete`, …).
    pub fn poll_event(&mut self) -> Option<PlayerEvent> {
        self.canvas.player.poll_event()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_style_matches_the_page_slot_json() {
        let paint = Paint::bind_style(Theme::Light, Style::Duotone, 2.5);
        let calls: Vec<(&str, &str)> = paint.writes.iter().map(|w| (w.id, w.json.as_str())).collect();
        assert_eq!(
            calls,
            [
                ("style.outline.light", "{\"a\":0,\"k\":0}"),
                ("style.outline.regular", "{\"a\":0,\"k\":0}"),
                ("style.outline.bold", "{\"a\":0,\"k\":100}"),
                ("style.solid", "{\"a\":0,\"k\":0}"),
                ("style.tint", "{\"a\":0,\"k\":18}"),
            ]
        );
        let solid = Paint::bind_style(Theme::Dark, Style::Solid, 1.5);
        assert_eq!(solid.writes[0].json, "{\"a\":0,\"k\":0}");
        assert_eq!(solid.writes[3].json, "{\"a\":0,\"k\":100}");
        let colored = Paint::bind_style(Theme::Dark, Style::Outline, 2.0).with_color([0.35, 0.89, 0.91, 1.0]);
        assert_eq!(colored.writes[5].json, "{\"a\":0,\"k\":[0.35,0.89,0.91,1]}");
    }

    #[test]
    fn runtime_paint_follows_paint_icon() {
        let between = Paint::runtime(&Look { theme: Theme::Light, style: Style::Duotone, weight: 1.75, color: Some([0, 140, 118]) });
        let calls: Vec<(&str, &str)> = between.writes.iter().map(|w| (w.id, w.json.as_str())).collect();
        assert_eq!(
            calls,
            [
                ("color.primary", "{\"a\":0,\"k\":[0,0.5490196078431373,0.4627450980392157,1]}"),
                ("style.outline.light", "{\"a\":0,\"k\":0}"),
                ("style.outline.regular", "{\"a\":0,\"k\":100}"),
                ("style.outline.bold", "{\"a\":0,\"k\":0}"),
                ("style.outline.width", "{\"a\":0,\"k\":1.75}"),
                ("style.solid", "{\"a\":0,\"k\":0}"),
                ("style.tint", "{\"a\":0,\"k\":18}"),
            ]
        );
        let bold = Paint::runtime(&Look { theme: Theme::Dark, style: Style::Outline, weight: 2.5, color: None });
        assert_eq!(bold.writes.len(), 6);
        assert_eq!(bold.writes[2].json, "{\"a\":0,\"k\":100}");
        assert_eq!(bold.writes[3].json, "{\"a\":0,\"k\":2}");
    }

    #[test]
    fn names_round_trip() {
        for theme in Theme::ALL {
            assert_eq!(Theme::parse(theme.id()), Some(theme));
        }
        for style in Style::ALL {
            assert_eq!(Style::parse(style.name()), Some(style));
        }
        assert!(is_authored_weight(2.0) && !is_authored_weight(1.75));
    }
}
