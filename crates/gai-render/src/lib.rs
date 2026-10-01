//! Renders .lottie files headlessly with dotlottie-rs (the dotLottie players' own
//! ThorVG core), runs the visual checks and writes review sheets.
//!
//! The renderer is dotlottie-rs at `e112d3f`, the commit dotlottie-web 0.80.0's
//! WASM player was built from. It reproduces the browser evidence of
//! `prototype/icons/NN/verify.html` bit for bit.
//!
//! # Visual verification
//!
//! [`verify_action`] runs every check of the canonical page (sets 35 onwards
//! and every new set) on one action; [`verify_set`] drives a whole set and
//! [`EvidenceWriter`] writes the evidence files `serve-verification.py` wrote.
//! Legacy sets add their page's rules through [`SetRules::for_set`];
//! [`legacy::validate_evidence`] checks their old evidence against a canonical
//! re-run on the keys both share.
//!
//! # Previews
//!
//! [`render_frame`] paints a frame the way the runtime's `paintIcon` does, and
//! [`contact_sheet`] lays out frames × looks in one PNG for review.
//!
//! # Build requirement
//!
//! ThorVG must be compiled without floating-point contraction:
//! `CXXFLAGS=-ffp-contract=off`, for example through the workspace's
//! `.cargo/config.toml` (`[env] CXXFLAGS = "-ffp-contract=off"`). The browser's
//! WASM player cannot fuse multiply-adds; an arm64 build that does moves edge
//! pixels on 3.4 % of the catalog's actions. [`probe::renderer_matches_browser`]
//! detects such a build, and [`verify_set`] and [`legacy::validate_evidence`]
//! refuse to run on it.
//!
//! # Crash isolation
//!
//! A native renderer fault (for example the second-slot-flush SIGSEGV that
//! [`PaintedPlayer`] exists to avoid) kills the process; `catch_unwind` cannot
//! catch it. Everything here runs in process. The CLI isolates crashes by
//! running each action in a child process of itself, through a hidden
//! subcommand:
//!
//! 1. The parent checks [`probe::require_browser_parity`] once, then drives a
//!    [`SetRun`]: `start_action(name)`, then spawns
//!    `gen-animated-icons __verify-action --assets DIR --set NN --action NAME --out TMP`.
//! 2. The child calls [`verify_action`] with [`SetRules::for_set`] and writes
//!    the result with [`ActionRun::write_to`]`(TMP)`, exiting 0.
//! 3. On exit 0 the parent reads [`ActionRun::read_from`]`(TMP)`; if the child
//!    died (signal or other status) it records [`ActionRun::crashed`]`(name,
//!    "signal 11")`, an ordinary action failure. Either way it calls
//!    `SetRun::record` and saves with [`EvidenceWriter::save`].
//!
//! Child processes are also the way to verify actions in parallel: players may
//! live on several threads, but their calls into ThorVG are serialized
//! process-wide (ThorVG is built without its thread feature, like the WASM
//! player).

pub mod catalog;
pub mod evidence;
pub mod legacy;
pub mod player;
pub mod preview;
pub mod probe;
pub mod raster;
pub mod rules;
pub mod sheets;
pub mod verify;

pub use catalog::{ActionMeta, Family, Motion, SetManifest};
pub use evidence::{EvidenceWriter, RENDERER, RUNTIME, Report, SetOptions, SetRun, Verdict, iso_date, verify_set};
pub use player::{Look, Paint, PaintedPlayer, Style, Theme};
pub use preview::{ContactSheet, Frame, STROKE_WIDTH_SLOT_SUPPORTED, WeightMethod, contact_sheet, render_frame};
pub use rules::SetRules;
pub use sheets::SheetImage;
pub use verify::{ActionOptions, ActionRun, Outcome, verify_action};

use std::path::PathBuf;

/// Errors from rendering, set manifests and evidence files.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The raster size is zero or above 4096 pixels.
    #[error("unsupported raster size {0}")]
    Size(u32),
    /// The software target could not be bound.
    #[error("could not bind the pixel buffer")]
    Target,
    /// The `.lottie` (or an animation inside it) failed to load.
    #[error("could not load the .lottie")]
    Load,
    /// The requested animation is not in the `.lottie`.
    #[error("no animation {0:?} in the .lottie")]
    Animation(String),
    /// The requested theme is not in the `.lottie`.
    #[error("no theme {0:?} in the .lottie")]
    Theme(String),
    /// A name contains a NUL byte.
    #[error("invalid name {0:?}")]
    Name(String),
    /// A preview weight outside 1–3.
    #[error("weight {0} is outside 1–3")]
    Weight(f64),
    /// A preview frame outside the animation.
    #[error("frame {frame} is outside 0–{last}")]
    Frame {
        /// Requested frame.
        frame: f64,
        /// Last frame of the animation.
        last: f64,
    },
    /// The player rejected a paint.
    #[error("paint rejected: {0}")]
    Paint(&'static str),
    /// An invalid contact sheet request.
    #[error("{0}")]
    Sheet(String),
    /// set.json is missing a field the checks need.
    #[error("set.json: {0}")]
    Manifest(String),
    /// An evidence or run file is malformed.
    #[error("invalid evidence: {0}")]
    Evidence(String),
    /// A file could not be read or written.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error,
    },
    /// PNG encoding failed.
    #[error("PNG encoding failed: {0}")]
    Png(String),
    /// This ThorVG build fuses multiply-adds, so its pixels differ from the
    /// browser's on some icons; see [`probe`].
    #[error("this build's renderer does not match the browser's (ThorVG was compiled with FMA contraction); rebuild with CXXFLAGS=-ffp-contract=off")]
    RendererBuild,
}
