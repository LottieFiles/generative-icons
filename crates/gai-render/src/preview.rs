//! Previews for the CLI and an AI review loop: single frames and contact sheets.

use crate::player::{Canvas, Look, Paint, PaintedPlayer, Style, Theme, engine, is_authored_weight};
use crate::sheets::{BACKGROUND, Image, LABEL, text_width, to_fixed};
use crate::RenderError;
use dotlottie_rs::{DotLottieManager, SlotType};
use serde_json::{Value, json};
use std::ffi::CString;

/// Whether this renderer applies the `style.outline.width` stroke-width slot.
/// ThorVG gained stroke-width slots in 1.1.2; the pinned build has 1.1.1.
pub const STROKE_WIDTH_SLOT_SUPPORTED: bool = false;

/// How a preview shows its weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightMethod {
    /// An authored layer (1.5, 2 or 2.5).
    Authored,
    /// Regular geometry with every `style.outline.width` stroke rewritten to
    /// the weight, like the runtime's `rewriteStrokeWidth` bridge.
    Rewrite,
}

impl WeightMethod {
    /// `"authored"` or `"rewrite"`.
    pub fn as_str(self) -> &'static str {
        match self {
            WeightMethod::Authored => "authored",
            WeightMethod::Rewrite => "rewrite",
        }
    }
}

/// The method a weight is previewed with; weights must lie in 1–3.
pub fn weight_method(weight: f64) -> Result<WeightMethod, RenderError> {
    if !(1.0..=3.0).contains(&weight) {
        return Err(RenderError::Weight(weight));
    }
    Ok(if is_authored_weight(weight) { WeightMethod::Authored } else { WeightMethod::Rewrite })
}

/// A rendered frame: straight RGBA, `size * size * 4` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Width and height in pixels.
    pub size: u32,
    /// Row-major RGBA bytes with straight alpha.
    pub rgba: Vec<u8>,
    /// How the look's weight was applied.
    pub weight_method: WeightMethod,
}

impl Frame {
    /// Encodes the frame as a PNG.
    pub fn to_png(&self) -> Result<Vec<u8>, RenderError> {
        crate::sheets::encode_png(&self.rgba, self.size, self.size)
    }
}

/// The runtime's `rewriteStrokeWidth` for one animation: every top-level
/// layer's stroke bound to `style.outline.width` (and not animated) gets
/// `w.k = weight`, and the default slot becomes the weight.
pub fn rewrite_stroke_width(animation: &mut Value, weight: f64) {
    fn shapes(list: Option<&mut Value>, weight: f64) {
        let Some(Value::Array(items)) = list else { return };
        for shape in items {
            match shape.get("ty").and_then(Value::as_str) {
                Some("gr") => shapes(shape.get_mut("it"), weight),
                Some("st") => {
                    let Some(w) = shape.get_mut("w").and_then(Value::as_object_mut) else { continue };
                    let bound = w.get("sid").and_then(Value::as_str) == Some("style.outline.width");
                    let animated = w.get("a").is_some_and(|a| a.as_f64().is_some_and(|x| x != 0.0) || a.as_bool() == Some(true));
                    if bound && !animated {
                        w.insert("k".into(), json!(weight));
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(Value::Array(layers)) = animation.get_mut("layers") {
        for layer in layers {
            shapes(layer.get_mut("shapes"), weight);
        }
    }
    if let Some(slot) = animation.get_mut("slots").and_then(|s| s.get_mut("style.outline.width"))
        && !slot.is_null()
    {
        *slot = json!({"p": {"a": 0, "k": weight}});
    }
}

/// One look of one animation, ready to render frames.
enum LookPlayer {
    Painted(PaintedPlayer),
    Rewritten(Canvas),
}

impl LookPlayer {
    fn open(lottie: &[u8], animation: Option<&str>, look: &Look, size: u32) -> Result<(LookPlayer, WeightMethod), RenderError> {
        let method = weight_method(look.weight)?;
        let player = match method {
            WeightMethod::Authored => {
                let mut player = PaintedPlayer::open(lottie, size)?;
                if let Some(id) = animation {
                    player.select(id);
                    if player.animation() != id {
                        return Err(RenderError::Animation(id.to_string()));
                    }
                }
                player.paint(&Paint::runtime(look), Some("Theme rejected")).map_err(RenderError::Paint)?;
                LookPlayer::Painted(player)
            }
            WeightMethod::Rewrite => LookPlayer::Rewritten(rewritten(lottie, animation, look, size)?),
        };
        Ok((player, method))
    }

    fn total_frames(&self) -> f32 {
        match self {
            LookPlayer::Painted(p) => p.total_frames(),
            LookPlayer::Rewritten(c) => {
                let _engine = engine();
                c.player.total_frames()
            }
        }
    }

    fn fps(&self) -> f64 {
        match self {
            LookPlayer::Painted(p) => p.fps(),
            LookPlayer::Rewritten(c) => {
                let _engine = engine();
                c.fps()
            }
        }
    }

    fn set_frame(&mut self, frame: f64) {
        match self {
            LookPlayer::Painted(p) => {
                p.set_frame(frame);
            }
            LookPlayer::Rewritten(c) => {
                let _engine = engine();
                c.set_frame(frame);
            }
        }
    }

    fn rgba(&mut self) -> &[u8] {
        match self {
            LookPlayer::Painted(p) => p.rgba(),
            LookPlayer::Rewritten(c) => c.rgba(),
        }
    }
}

/// Loads the animation JSON with rewritten stroke widths and applies the theme
/// and runtime slots in one flush.
fn rewritten(lottie: &[u8], animation: Option<&str>, look: &Look, size: u32) -> Result<Canvas, RenderError> {
    let manager = DotLottieManager::new(lottie).map_err(|_| RenderError::Load)?;
    let id = animation.map_or_else(|| manager.active_animation_id(), str::to_string);
    let text = manager.get_animation(&id).map_err(|_| RenderError::Animation(id.clone()))?;
    let mut json: Value = serde_json::from_str(&text).map_err(|_| RenderError::Load)?;
    rewrite_stroke_width(&mut json, look.weight);
    let data = CString::new(serde_json::to_string(&json).map_err(|_| RenderError::Load)?).map_err(|_| RenderError::Load)?;
    let theme = manager.get_theme(look.theme.id()).map_err(|_| RenderError::Theme(look.theme.id().into()))?;
    let _engine = engine();
    let mut canvas = Canvas::new(size)?;
    let player = &mut canvas.player;
    player.load_animation_data(&data).map_err(|_| RenderError::Load)?;
    for (slot_id, slot) in theme.to_slot_types(&id) {
        let applied = match slot {
            SlotType::Color(s) => player.set_color_slot(&slot_id, s),
            SlotType::Gradient(s) => player.set_gradient_slot(&slot_id, s),
            SlotType::Image(s) => player.set_image_slot(&slot_id, s),
            SlotType::Text(s) => player.set_text_slot(&slot_id, s),
            SlotType::Scalar(s) => player.set_scalar_slot(&slot_id, s),
            SlotType::Vector(s) => player.set_vector_slot(&slot_id, s),
            SlotType::Position(s) => player.set_position_slot(&slot_id, s),
        };
        applied.map_err(|_| RenderError::Theme(look.theme.id().into()))?;
    }
    for (id, json) in Paint::runtime(look).slot_json() {
        player.set_slot_str(id, json).map_err(|_| RenderError::Paint("Slot rejected"))?;
    }
    // The single slot flush for this load; Err only means "nothing changed".
    let _ = player.render();
    Ok(canvas)
}

fn check_frame(frame: f64, total: f32) -> Result<(), RenderError> {
    let last = f64::from(total) - 1.0;
    if frame.is_finite() && (0.0..=last).contains(&frame) { Ok(()) } else { Err(RenderError::Frame { frame, last }) }
}

/// Renders one frame of a `.lottie` painted like the runtime's `paintIcon`.
///
/// `animation` selects an action inside the archive (default: the manifest's
/// initial animation). `frame` may be fractional and must lie in
/// `0..=total_frames - 1`. Weights between the authored layers are applied
/// with [`WeightMethod::Rewrite`], since this renderer ignores the
/// stroke-width slot ([`STROKE_WIDTH_SLOT_SUPPORTED`]).
pub fn render_frame(lottie: &[u8], animation: Option<&str>, frame: f64, size: u32, look: &Look) -> Result<Frame, RenderError> {
    let (mut player, weight_method) = LookPlayer::open(lottie, animation, look, size)?;
    check_frame(frame, player.total_frames())?;
    player.set_frame(frame);
    Ok(Frame { size, rgba: player.rgba().to_vec(), weight_method })
}

/// Contact sheet page colour, darker than the dark cells so they stay visible.
const PAGE: [u8; 3] = [11, 15, 20];

/// Cell background for a theme: the review sheets' dark ground or a light page.
fn cell_background(theme: Theme) -> [u8; 3] {
    match theme {
        Theme::Dark => BACKGROUND,
        Theme::Light => [246, 248, 251],
    }
}

/// A compact grid of frames × looks for reviewing an animation at a glance.
///
/// Layout (all sizes in pixels, `P = 8`, `G = 4`, labels in 11 px monospace,
/// page `#0b0f14`, text `#a5b1bc`):
///
/// ```text
/// P ┌ title (optional, 16 px row) ───────────────────────────────┐
///   │ row labels │ column label 0 │ column label 1 │ …  (16 px row) │
///   │ "outline   │ ┌──────────┐ G │ ┌──────────┐   │                │
///   │  2 dark"   │ │ cell px  │   │ │          │   │                │
///   │            │ └──────────┘   │ └──────────┘   │                │
///   │    G between rows …                                           │
/// P └───────────────────────────────────────────────────────────────┘
/// ```
///
/// Each row is one [`Look`] (label `"{style} {weight} {theme}"`, Solid without
/// weight), each column one frame (label `"{seconds:.3} s"` at the animation's
/// frame rate, JS `toFixed`). Cells are rendered natively at `cell` pixels on
/// the look's theme ground: `#10171d` for dark, `#f6f8fb` for light.
#[derive(Clone, Debug, PartialEq)]
pub struct ContactSheet {
    /// Action inside the `.lottie`; `None` for the initial animation.
    pub animation: Option<String>,
    /// Optional heading, e.g. the action name.
    pub title: Option<String>,
    /// Frames to show, one column each; clamped into the animation.
    pub frames: Vec<f64>,
    /// Looks to show, one row each.
    pub looks: Vec<Look>,
    /// Cell size in pixels.
    pub cell: u32,
}

impl ContactSheet {
    /// Every style and weight on dark, plus Regular on light: the rows an AI
    /// reviewer needs to judge an icon's paints.
    pub fn review_looks() -> Vec<Look> {
        let dark = |style, weight| Look { theme: Theme::Dark, style, weight, color: None };
        vec![
            dark(Style::Outline, 1.5),
            dark(Style::Outline, 2.0),
            dark(Style::Outline, 2.5),
            dark(Style::Solid, 2.0),
            dark(Style::Duotone, 2.0),
            Look::regular(Theme::Light),
        ]
    }

    /// A sheet of `frames` with [`ContactSheet::review_looks`] at 96 px.
    pub fn new(frames: Vec<f64>) -> ContactSheet {
        ContactSheet { animation: None, title: None, frames, looks: ContactSheet::review_looks(), cell: 96 }
    }

    /// Frames at `seconds` (key poses) for a 60 fps animation.
    pub fn at_seconds(seconds: &[f64]) -> ContactSheet {
        ContactSheet::new(seconds.iter().map(|t| t * 60.0).collect())
    }
}

fn look_label(look: &Look) -> String {
    let weight = gai_core::js_number_to_string(look.weight);
    match look.style {
        Style::Solid => format!("solid {}", look.theme.id()),
        style => format!("{} {weight} {}", style.name(), look.theme.id()),
    }
}

/// Renders a [`ContactSheet`] to PNG bytes.
pub fn contact_sheet(lottie: &[u8], sheet: &ContactSheet) -> Result<Vec<u8>, RenderError> {
    const PAD: u32 = 8;
    const GAP: u32 = 4;
    const ROW: u32 = 16;
    const FONT_PX: f32 = 11.0;
    if sheet.frames.is_empty() || sheet.looks.is_empty() {
        return Err(RenderError::Sheet("a contact sheet needs at least one frame and one look".into()));
    }
    let labels: Vec<String> = sheet.looks.iter().map(look_label).collect();
    let label_width = labels.iter().map(|l| text_width(l, FONT_PX)).fold(0.0f32, f32::max).ceil() as u32 + GAP * 2;
    let cell = sheet.cell;
    let columns = sheet.frames.len() as u32;
    let rows = sheet.looks.len() as u32;
    let title_height = if sheet.title.is_some() { ROW } else { 0 };
    let grid_top = PAD + title_height + ROW;
    let width = PAD * 2 + label_width + columns * cell + (columns - 1) * GAP;
    let height = grid_top + rows * cell + (rows - 1) * GAP + PAD;
    let mut image = Image::filled(width, height, PAGE);
    if let Some(title) = &sheet.title {
        image.draw_text(title, FONT_PX, PAD as f32, (PAD + 12) as f32, LABEL);
    }
    for (row, look) in sheet.looks.iter().enumerate() {
        let (mut player, _) = LookPlayer::open(lottie, sheet.animation.as_deref(), look, cell)?;
        let last = f64::from(player.total_frames()) - 1.0;
        let fps = player.fps();
        let y = grid_top + row as u32 * (cell + GAP);
        image.draw_text(&labels[row], FONT_PX, PAD as f32, (y + cell / 2 + 4) as f32, LABEL);
        for (column, &frame) in sheet.frames.iter().enumerate() {
            let x = PAD + label_width + column as u32 * (cell + GAP);
            if row == 0 {
                let label = format!("{} s", to_fixed(frame / fps, 3));
                image.draw_text_centered(&label, FONT_PX, (x + cell / 2) as f32, (grid_top - 4) as f32, LABEL);
            }
            player.set_frame(frame.clamp(0.0, last.max(0.0)));
            image.fill_rect(x, y, cell, cell, cell_background(look.theme));
            image.draw(player.rgba(), cell, x, y);
        }
    }
    image.encode_png()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_touches_only_bound_static_strokes() {
        let mut anim = json!({
            "layers": [{"shapes": [
                {"ty": "gr", "it": [{"ty": "st", "w": {"a": 0, "k": 2, "sid": "style.outline.width"}}]},
                {"ty": "st", "w": {"a": 1, "k": [], "sid": "style.outline.width"}},
                {"ty": "st", "w": {"a": 0, "k": 1.5}},
            ]}],
            "slots": {"style.outline.width": {"p": {"a": 0, "k": 2}}, "style.solid": {"p": {"a": 0, "k": 0}}},
        });
        rewrite_stroke_width(&mut anim, 1.75);
        assert_eq!(anim["layers"][0]["shapes"][0]["it"][0]["w"], json!({"a": 0, "k": 1.75, "sid": "style.outline.width"}));
        assert_eq!(anim["layers"][0]["shapes"][1]["w"]["k"], json!([]));
        assert_eq!(anim["layers"][0]["shapes"][2]["w"]["k"], json!(1.5));
        assert_eq!(anim["slots"]["style.outline.width"], json!({"p": {"a": 0, "k": 1.75}}));
    }

    #[test]
    fn weights_outside_the_range_are_rejected() {
        assert_eq!(weight_method(2.5).unwrap(), WeightMethod::Authored);
        assert_eq!(weight_method(1.0).unwrap(), WeightMethod::Rewrite);
        assert!(weight_method(3.5).is_err() && weight_method(f64::NAN).is_err());
    }

    #[test]
    fn look_labels() {
        assert_eq!(look_label(&Look::regular(Theme::Light)), "outline 2 light");
        assert_eq!(look_label(&Look { theme: Theme::Dark, style: Style::Solid, weight: 2.0, color: None }), "solid dark");
    }
}
