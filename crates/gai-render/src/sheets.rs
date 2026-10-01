//! PNG review sheets: `rest.png`, `styles.png` and `poses.png`, plus the
//! drawing primitives the contact sheets share.
//!
//! The browser drew these with canvas 2D; the port uses its own compositor,
//! resampler and font. Above the label band every pixel is within ±1 of the
//! browser's and alpha is exact. The labels are not reproduced: the browser
//! drew its system `monospace` font with the OS's glyph smoothing, this module
//! draws embedded DejaVu Sans Mono, so label pixels differ by up to about 155
//! levels. The PNG bytes differ too (one IDAT chunk, another deflate). Sheets
//! are review aids; no check depends on their bytes.

use crate::RenderError;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use std::sync::OnceLock;

/// DejaVu Sans Mono 2.37 (Bitstream Vera licence plus public-domain DejaVu
/// changes; see `fonts/LICENSE-DejaVu.txt`), subset to printable ASCII with
/// `pyftsubset DejaVuSansMono.ttf --unicodes=U+0020-007E --layout-features=''
/// --no-hinting --name-IDs='*' --notdef-outline`.
const FONT: &[u8] = include_bytes!("../fonts/DejaVuSansMono-ASCII.ttf");

/// Sheet background `#10171d`.
pub const BACKGROUND: [u8; 3] = [16, 23, 29];
/// Label color `#a5b1bc`.
pub const LABEL: [u8; 3] = [165, 177, 188];

/// A named PNG produced by the checks (`{action}-rest.png`, …).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetImage {
    /// File name, `[a-z0-9-]+\.png`.
    pub name: String,
    /// Encoded PNG bytes.
    pub png: Vec<u8>,
}

/// An RGBA8 image with straight alpha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA bytes.
    pub rgba: Vec<u8>,
}

fn font() -> &'static FontRef<'static> {
    static FONT_REF: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT_REF.get_or_init(|| FontRef::try_from_slice(FONT).expect("embedded font parses"))
}

/// `(src * a + dst * (255 - a)) / 255`, rounded half up.
fn over(src: u8, dst: u8, alpha: u8) -> u8 {
    let a = u32::from(alpha);
    ((u32::from(src) * a + u32::from(dst) * (255 - a) + 127) / 255) as u8
}

impl Image {
    /// A `width`×`height` image filled with an opaque color.
    pub fn filled(width: u32, height: u32, color: [u8; 3]) -> Image {
        let mut image = Image { width, height, rgba: vec![0; (width * height * 4) as usize] };
        image.fill_rect(0, 0, width, height, color);
        image
    }

    /// Fills a rectangle (clipped to the image) with an opaque color.
    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: [u8; 3]) {
        for row in y..(y + height).min(self.height) {
            for col in x..(x + width).min(self.width) {
                let i = ((row * self.width + col) * 4) as usize;
                self.rgba[i..i + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
    }

    /// Draws a straight-alpha RGBA tile source-over at `(x, y)` onto this
    /// opaque image (canvas `drawImage` without scaling).
    pub fn draw(&mut self, tile: &[u8], tile_width: u32, x: u32, y: u32) {
        let tile_height = (tile.len() / 4) as u32 / tile_width.max(1);
        for ty in 0..tile_height {
            for tx in 0..tile_width {
                let (px, py) = (x + tx, y + ty);
                if px >= self.width || py >= self.height {
                    continue;
                }
                let s = ((ty * tile_width + tx) * 4) as usize;
                let d = ((py * self.width + px) * 4) as usize;
                let a = tile[s + 3];
                for c in 0..3 {
                    self.rgba[d + c] = over(tile[s + c], self.rgba[d + c], a);
                }
            }
        }
    }

    /// Draws premultiplied float RGBA (0–255 scale) source-over at `(x, y)`.
    fn draw_premultiplied(&mut self, tile: &[f32], tile_width: u32, x: u32, y: u32) {
        let tile_height = (tile.len() / 4) as u32 / tile_width.max(1);
        for ty in 0..tile_height {
            for tx in 0..tile_width {
                let (px, py) = (x + tx, y + ty);
                if px >= self.width || py >= self.height {
                    continue;
                }
                let s = ((ty * tile_width + tx) * 4) as usize;
                let d = ((py * self.width + px) * 4) as usize;
                let a = tile[s + 3].clamp(0.0, 255.0) / 255.0;
                for c in 0..3 {
                    let value = tile[s + c].clamp(0.0, 255.0) + f32::from(self.rgba[d + c]) * (1.0 - a);
                    self.rgba[d + c] = (value + 0.5).floor().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Draws a straight-alpha square tile scaled to `target`×`target` at `(x, y)`
    /// with [`resample`].
    pub fn draw_scaled(&mut self, tile: &[u8], tile_size: u32, target: u32, x: u32, y: u32) {
        let scaled = resample(tile, tile_size, target);
        self.draw_premultiplied(&scaled, target, x, y);
    }

    /// Draws `text` in the embedded monospace font at `px` pixels (CSS font
    /// size), horizontally centred on `center_x` with its baseline at `baseline`
    /// (canvas `textAlign = "center"`). Characters outside ASCII are skipped.
    pub fn draw_text_centered(&mut self, text: &str, px: f32, center_x: f32, baseline: f32, color: [u8; 3]) {
        let width = text_width(text, px);
        self.draw_text(text, px, center_x - width / 2.0, baseline, color);
    }

    /// Draws `text` starting at `x` with its baseline at `baseline`.
    pub fn draw_text(&mut self, text: &str, px: f32, x: f32, baseline: f32, color: [u8; 3]) {
        let font = font();
        let scaled = font.as_scaled(px_scale(px));
        let mut pen = x;
        for ch in text.chars().filter(|c| c.is_ascii() && !c.is_ascii_control()) {
            let id = font.glyph_id(ch);
            let glyph = id.with_scale_and_position(scaled.scale(), point(pen, baseline));
            pen += scaled.h_advance(id);
            let Some(outline) = font.outline_glyph(glyph) else { continue };
            let bounds = outline.px_bounds();
            outline.draw(|gx, gy, coverage| {
                let px = bounds.min.x as i64 + i64::from(gx);
                let py = bounds.min.y as i64 + i64::from(gy);
                if px < 0 || py < 0 || px >= i64::from(self.width) || py >= i64::from(self.height) {
                    return;
                }
                let d = ((py as u32 * self.width + px as u32) * 4) as usize;
                let alpha = (coverage.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                for (c, &value) in color.iter().enumerate() {
                    self.rgba[d + c] = over(value, self.rgba[d + c], alpha);
                }
            });
        }
    }

    /// Encodes as an 8-bit RGBA PNG with no ancillary chunks.
    pub fn encode_png(&self) -> Result<Vec<u8>, RenderError> {
        encode_png(&self.rgba, self.width, self.height)
    }
}

/// The scale giving an em size of `px` pixels (ab_glyph scales by ascent − descent).
fn px_scale(px: f32) -> PxScale {
    let font = font();
    let em = font.units_per_em().unwrap_or(2048.0);
    PxScale::from(px * font.height_unscaled() / em)
}

/// Advance width of `text` at `px` pixels.
pub fn text_width(text: &str, px: f32) -> f32 {
    let font = font();
    let scaled = font.as_scaled(px_scale(px));
    text.chars()
        .filter(|c| c.is_ascii() && !c.is_ascii_control())
        .map(|c| scaled.h_advance(font.glyph_id(c)))
        .sum()
}

/// Encodes straight RGBA bytes as an 8-bit RGBA PNG (IHDR, IDAT, IEND only).
///
/// Adaptive filtering and deflate level 6: the encoder's own default (fast
/// deflate, Sub filter) made sheets up to 1.9x the browser's, and these files
/// are packed into set zips. The bytes follow flate2's backend (zlib once
/// gai-package is linked); the pixels do not.
pub fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);
    encoder.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);
    let mut writer = encoder.write_header().map_err(|e| RenderError::Png(e.to_string()))?;
    writer.write_image_data(rgba).map_err(|e| RenderError::Png(e.to_string()))?;
    writer.finish().map_err(|e| RenderError::Png(e.to_string()))?;
    Ok(out)
}

/// Normalised bilinear taps for one axis: `(first source index, weights)` per
/// output pixel. Pixel centres map as `(x + 0.5) * scale - 0.5`.
fn taps(source: u32, target: u32) -> Vec<(usize, Vec<f64>)> {
    let scale = f64::from(source) / f64::from(target);
    (0..target)
        .map(|x| {
            let center = (f64::from(x) + 0.5) * scale;
            let first = (center - 1.0).floor().max(0.0) as usize;
            let last = ((center + 1.0).ceil() as usize).min(source as usize);
            let mut weights: Vec<f64> = (first..last).map(|i| (1.0 - (i as f64 + 0.5 - center).abs()).max(0.0)).collect();
            let total: f64 = weights.iter().sum();
            if total != 0.0 {
                weights.iter_mut().for_each(|w| *w /= total);
            }
            (first, weights)
        })
        .collect()
}

/// Resamples a straight-alpha square raster to `target`×`target` and returns
/// premultiplied RGBA floats on a 0–255 scale.
///
/// The filter is fixed: premultiply, then separable bilinear interpolation
/// (two taps per axis, no widening when downscaling), horizontal pass first.
/// That is Chrome's default canvas `drawImage` smoothing; on the catalog's
/// `poses.png` tiles it lands within ±1 of the browser's.
pub fn resample(rgba: &[u8], size: u32, target: u32) -> Vec<f32> {
    let n = size as usize;
    let premultiplied: Vec<f64> = rgba
        .chunks_exact(4)
        .flat_map(|p| {
            let a = f64::from(p[3]) / 255.0;
            [f64::from(p[0]) * a, f64::from(p[1]) * a, f64::from(p[2]) * a, f64::from(p[3])]
        })
        .collect();
    let taps = taps(size, target);
    let t = target as usize;
    let mut horizontal = vec![0.0f64; t * n * 4];
    for y in 0..n {
        for (x, (first, weights)) in taps.iter().enumerate() {
            for (k, w) in weights.iter().enumerate() {
                let s = (y * n + first + k) * 4;
                let d = (y * t + x) * 4;
                for c in 0..4 {
                    horizontal[d + c] += premultiplied[s + c] * w;
                }
            }
        }
    }
    let mut out = vec![0.0f32; t * t * 4];
    for (y, (first, weights)) in taps.iter().enumerate() {
        for x in 0..t {
            let d = (y * t + x) * 4;
            let mut acc = [0.0f64; 4];
            for (k, w) in weights.iter().enumerate() {
                let s = ((first + k) * t + x) * 4;
                for c in 0..4 {
                    acc[c] += horizontal[s + c] * w;
                }
            }
            // Keep colour within alpha so the premultiplied composite stays valid.
            let alpha = acc[3].clamp(0.0, 255.0);
            out[d + 3] = alpha as f32;
            for c in 0..3 {
                out[d + c] = acc[c].clamp(0.0, alpha) as f32;
            }
        }
    }
    out
}

/// ECMAScript `Number.prototype.toFixed(digits)`: exact decimal value, ties
/// away from zero (Rust's `{:.3}` rounds exact ties to even).
pub fn to_fixed(x: f64, digits: usize) -> String {
    if !x.is_finite() || x.abs() >= 1e21 {
        return gai_core::js_number_to_string(x);
    }
    // Rust prints the exact binary value's decimal expansion at any precision;
    // 1100 fraction digits cover every finite f64.
    let exact = format!("{:.1100}", x.abs());
    let (int, frac) = exact.split_once('.').expect("fixed notation");
    let mut kept: Vec<u8> = int.bytes().chain(frac.bytes().take(digits)).collect();
    let rest = &frac.as_bytes()[digits..];
    if rest.first().is_some_and(|&d| d >= b'5') {
        let mut i = kept.len();
        loop {
            if i == 0 {
                kept.insert(0, b'1');
                break;
            }
            i -= 1;
            if kept[i] == b'9' {
                kept[i] = b'0';
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    let split = kept.len() - digits;
    let (int, frac) = kept.split_at(split);
    let body = if digits == 0 {
        String::from_utf8_lossy(int).into_owned()
    } else {
        format!("{}.{}", String::from_utf8_lossy(int), String::from_utf8_lossy(frac))
    };
    if x < 0.0 { format!("-{body}") } else { body }
}

/// `{name}-rest.png`: the 192-px rest frame as is (transparent background).
pub fn rest_png(rgba: &[u8], size: u32) -> Result<Vec<u8>, RenderError> {
    encode_png(rgba, size, size)
}

/// The labels of the five `styles.png` tiles.
pub const STYLE_LABELS: [&str; 5] = ["Light", "Regular", "Bold", "Solid", "Duotone"];

/// `{name}-styles.png`: five 192-px tiles (Light, Regular, Bold, Solid,
/// Duotone) over the background, labels in 12 px centred at baseline 209.
pub fn styles_png(tiles: &[Vec<u8>], labels: &[&str]) -> Result<Vec<u8>, RenderError> {
    let mut image = Image::filled(192 * tiles.len() as u32, 224, BACKGROUND);
    for (i, tile) in tiles.iter().enumerate() {
        image.draw(tile, 192, i as u32 * 192, 0);
    }
    image.fill_rect(0, 192, image.width, 32, BACKGROUND);
    for (i, label) in labels.iter().enumerate() {
        image.draw_text_centered(label, 12.0, i as f32 * 192.0 + 96.0, 209.0, LABEL);
    }
    image.encode_png()
}

/// `{name}-poses.png`: each key pose's 192-px frame scaled to 128 px, labelled
/// `time.toFixed(3) + " s"` in 11 px centred at baseline 146.
pub fn poses_png(poses: &[(f64, Vec<u8>)]) -> Result<Vec<u8>, RenderError> {
    let mut image = Image::filled(128 * poses.len() as u32, 158, BACKGROUND);
    for (i, (_, tile)) in poses.iter().enumerate() {
        image.draw_scaled(tile, 192, 128, i as u32 * 128, 0);
    }
    image.fill_rect(0, 128, image.width, 30, BACKGROUND);
    for (i, (time, _)) in poses.iter().enumerate() {
        image.draw_text_centered(&format!("{} s", to_fixed(*time, 3)), 11.0, i as f32 * 128.0 + 64.0, 146.0, LABEL);
    }
    image.encode_png()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_fixed_rounds_exact_ties_up() {
        assert_eq!(to_fixed(0.0625, 3), "0.063");
        assert_eq!(to_fixed(1.0625, 3), "1.063");
        assert_eq!(to_fixed(0.09200000000000001, 3), "0.092");
        assert_eq!(to_fixed(0.36800000000000005, 3), "0.368");
        assert_eq!(to_fixed(0.0, 3), "0.000");
        assert_eq!(to_fixed(2.3, 3), "2.300");
        assert_eq!(to_fixed(0.9995, 3), "1.000"); // 0.9995 is above the tie in binary
        assert_eq!(to_fixed(9.9999, 3), "10.000");
        assert_eq!(to_fixed(-0.0625, 3), "-0.063");
        assert_eq!(to_fixed(-0.0001, 3), "-0.000");
        assert_eq!(to_fixed(-0.0, 3), "0.000");
        assert_eq!(to_fixed(2.5, 0), "3");
    }

    #[test]
    fn compositing_rounds_half_up() {
        let mut image = Image::filled(1, 1, BACKGROUND);
        image.draw(&[255, 255, 255, 128], 1, 0, 0);
        // (255*128 + 16*127) / 255 = 136.03…
        assert_eq!(&image.rgba[..4], &[136, 139, 142, 255]);
        let mut clear = Image::filled(1, 1, BACKGROUND);
        clear.draw(&[0, 0, 0, 0], 1, 0, 0);
        assert_eq!(&clear.rgba[..3], &BACKGROUND);
    }

    #[test]
    fn resample_preserves_flat_regions() {
        let flat: Vec<u8> = [200u8, 100, 50, 255].repeat(192 * 192);
        let out = resample(&flat, 192, 128);
        assert_eq!(out.len(), 128 * 128 * 4);
        for px in out.chunks_exact(4) {
            assert!((px[0] - 200.0).abs() < 1e-3 && (px[3] - 255.0).abs() < 1e-3);
        }
        let clear = resample(&vec![0u8; 192 * 192 * 4], 192, 128);
        assert!(clear.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn sheets_have_the_browser_dimensions() {
        let tile = vec![0u8; 192 * 192 * 4];
        let styles = styles_png(&vec![tile.clone(); 5], &STYLE_LABELS).unwrap();
        let poses = poses_png(&[(0.0, tile.clone()), (0.46, tile)]).unwrap();
        for (png, w, h) in [(styles, 960, 224), (poses, 256, 158)] {
            let decoder = png::Decoder::new(png.as_slice());
            let reader = decoder.read_info().unwrap();
            let info = reader.info();
            assert_eq!((info.width, info.height, info.color_type), (w, h, png::ColorType::Rgba));
        }
    }

    /// An antialiased ring on transparency, like a rendered icon frame.
    fn ring_tile() -> Vec<u8> {
        (0..192 * 192)
            .flat_map(|i| {
                let (x, y) = (f64::from(i % 192) - 95.5, f64::from(i / 192) - 95.5);
                let coverage = (1.0 - ((x.hypot(y) - 60.0).abs() - 6.0)).clamp(0.0, 1.0);
                [0, 221, 179, (coverage * 255.0).round() as u8]
            })
            .collect()
    }

    #[test]
    fn sheets_are_deflated_with_adaptive_filters() {
        let tile = ring_tile();
        let poses: Vec<(f64, Vec<u8>)> = (0..6).map(|i| (f64::from(i) * 0.4, tile.clone())).collect();
        for (png, rgba, width, height) in [
            (rest_png(&tile, 192).unwrap(), tile.clone(), 192, 192),
            (poses_png(&poses).unwrap(), decode(&poses_png(&poses).unwrap()), 768, 158),
        ] {
            assert_eq!(decode(&png), rgba);
            // The png crate's own default: fast deflate and the Sub filter.
            let mut fast = Vec::new();
            let mut encoder = png::Encoder::new(&mut fast, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().unwrap().write_image_data(&rgba).unwrap();
            // At least 10% smaller with either flate2 backend (miniz_oxide alone, zlib when
            // gai-package is in the build).
            assert!(png.len() * 10 < fast.len() * 9, "{} bytes, fast deflate {}", png.len(), fast.len());
        }
    }

    fn decode(png: &[u8]) -> Vec<u8> {
        let mut reader = png::Decoder::new(png).read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        buf.truncate(info.buffer_size());
        buf
    }

    #[test]
    fn text_is_drawn_centred() {
        let mut image = Image::filled(64, 20, BACKGROUND);
        image.draw_text_centered("0.46 s", 11.0, 32.0, 14.0, LABEL);
        let lit: Vec<u32> = (0..64u32).filter(|&x| (0..20u32).any(|y| image.rgba[((y * 64 + x) * 4) as usize] > 60)).collect();
        let (first, last) = (lit[0], *lit.last().unwrap());
        assert!(first > 5 && last < 58 && (i64::from(first + last) - 64).abs() <= 4, "{first}..{last}");
        assert!((text_width("abc", 12.0) - 3.0 * text_width("a", 12.0)).abs() < 1e-3);
    }
}
