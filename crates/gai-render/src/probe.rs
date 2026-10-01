//! A self-check that this native ThorVG build rasterizes like the browser's
//! WASM player.
//!
//! Clang fuses `a * b + c` into fused multiply-adds by default on arm64 (and
//! wherever FMA is in the baseline); the WASM build cannot, and the evidence
//! was produced there. An FMA build moves anti-aliased edge pixels on 48 of
//! the 1,428 actions of sets 35–63 (3.4 %). Build ThorVG with
//! `CXXFLAGS=-ffp-contract=off` (see the crate docs) to match.

use crate::player::{Canvas, engine};
use crate::raster::fnv1a;
use crate::RenderError;
use std::ffi::CString;
use std::sync::OnceLock;

/// One Light-weight arm of set 35's `snowflake-strong` (Lottie JSON with its
/// two slots), whose rest pose lands on sub-pixel edges that FMA changes.
const PROBE: &str = include_str!("renderer-probe.json");

/// The probe's frame 40 as dotlottie-web 0.80.0's WASM player renders it:
/// `(size, FNV-1a of the RGBA buffer)`.
pub const BROWSER_HASHES: [(u32, u32); 3] = [(24, 2_582_267_197), (48, 2_358_436_859), (192, 4_112_197_911)];

/// This build's hashes of the probe, in [`BROWSER_HASHES`] order.
pub fn probe_hashes() -> Result<[(u32, u32); 3], RenderError> {
    let data = CString::new(PROBE).map_err(|_| RenderError::Load)?;
    let mut out = BROWSER_HASHES;
    for (size, hash) in &mut out {
        let _engine = engine();
        let mut canvas = Canvas::new(*size)?;
        canvas.player.load_animation_data(&data).map_err(|_| RenderError::Load)?;
        canvas.set_frame(40.0);
        *hash = fnv1a(canvas.rgba());
    }
    Ok(out)
}

/// Whether this build renders the probe exactly like the browser's player;
/// computed once per process. Evidence written by a build that fails this
/// does not match the browser's.
pub fn renderer_matches_browser() -> bool {
    static MATCHES: OnceLock<bool> = OnceLock::new();
    *MATCHES.get_or_init(|| probe_hashes().is_ok_and(|hashes| hashes == BROWSER_HASHES))
}

/// [`renderer_matches_browser`] as an error for the evidence entry points.
pub fn require_browser_parity() -> Result<(), RenderError> {
    if renderer_matches_browser() { Ok(()) } else { Err(RenderError::RendererBuild) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_matches_the_browser_player() {
        let hashes = probe_hashes().unwrap();
        assert_eq!(hashes, BROWSER_HASHES, "ThorVG was compiled with FMA contraction; build with CXXFLAGS=-ffp-contract=off");
        assert!(renderer_matches_browser());
    }
}
