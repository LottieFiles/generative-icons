//! Runs icon authoring code (a set's `definitions.mjs`, its batches, the kit and svgpath)
//! in an embedded QuickJS-NG whose `Math` returns what Node's V8 returns, and hands the
//! results to Rust without letting QuickJS format any number.
//!
//! ```no_run
//! # fn main() -> gai_js::Result<()> {
//! use gai_js::{Limits, SetRuntime, SetSource};
//! let limits = Limits::default();
//! let source = SetSource::library_set("motion/icon-sets/34-scheduling-time")?;
//! gai_js::run_on_worker(&limits, || {
//!     let mut runtime = SetRuntime::load(&source, &limits)?;
//!     let description = runtime.install()?;
//!     for (i, icon) in description.icons.iter().enumerate() {
//!         for j in 0..icon.variants.len() {
//!             let samples = runtime.sample_variant(i, j)?;
//!             assert_eq!(samples.parts.len(), icon.parts.len());
//!         }
//!     }
//!     Ok(())
//! })
//! # }
//! ```
//!
//! The engine is QuickJS-NG 0.8.0 through `rquickjs =0.9.0`, with `rquickjs-sys` vendored
//! and patched (`vendor/rquickjs-sys/PATCHED.md`): `Math.*` and `**` use V8's fdlibm built
//! with FMA contraction, `Math.hypot` uses V8's algorithm, and `pow` is correctly rounded.
//!
//! # Time limits
//!
//! [`Limits::call_timeout`] bounds each call into a set, and [`Limits::budget`] all its
//! calls together. QuickJS enforces both by polling an interrupt handler: between
//! bytecodes and, in the patched engine (`vendor/rquickjs-sys/PATCHED.md`), in regexp
//! execution and compilation, source parsing and name resolution, builtins that loop over
//! an array-like, string search and sorting. The patch also makes Unicode normalization
//! and forward named back references linear, where they were quadratic. A poll calls the
//! handler only every 10,000 steps, so the [`watchdog`]'s timer thread also asks the
//! runtime to call it as soon as the deadline passes: a call stops within one builtin
//! call of its deadline.
//!
//! Some native code runs to completion, past the deadline: libbf's BigInt arithmetic,
//! for one, takes minutes on a 10^9-bit number. The [`watchdog`] is the hard stop: a
//! one-shot process [`install`](watchdog::install)s it, and it ends the process with the
//! call's timeout error once a call runs [`Limits::grace`] past its deadline. A process
//! that must outlive a stuck call runs its JS work in a child process that installs it.

mod describe;
mod error;
pub mod kit;
mod loader;
mod probe;
mod project;
mod runtime;
mod samples;
mod source;
pub mod watchdog;
mod worker;

pub use describe::{
    Callbacks, DURATION, Description, FPS, FRAMES, Icon, JsType, Kind, Node, Part, SourcePart, Variant, frame_time,
};
pub use error::{Error, Location, ModuleError, Result};
pub use probe::{
    PosedPoint, ProbeGeometry, ProbeOutcome, ProbeRequest, ProbeResult, ProbeSample, ProbeStage, ProbeTransform,
    ProbeVisibility, ProbeWidth,
};
pub use runtime::{JsScope, Limits, SetRuntime, check_cpu};
pub use samples::{PartSamples, Rigid, VariantSamples};
pub use source::{LegacySet, ModernProfile, ReverseName, SetSource};
pub use worker::{UNSIZED_THREAD_STACK, map_parallel, run_on_worker, worker_stack_size};

/// The rquickjs version gai-js is built on, for code that inspects values in
/// [`SetRuntime::with_icons`].
pub use rquickjs;

/// The JavaScript engine, for `--version` output.
pub const ENGINE: &str = "QuickJS-NG 0.8.0 (rquickjs 0.9.0)";

/// How `Math` is computed, as the linked engine reports it, for `--version` output.
///
/// The string is defined by the patched `rquickjs-sys` (`vendor/rquickjs-sys`), so gai-js
/// fails to link against a stock QuickJS.
pub fn math() -> &'static str {
    unsafe extern "C" {
        safe fn gai_math_description() -> *const std::ffi::c_char;
    }
    // SAFETY: the function returns a NUL-terminated string literal (static storage).
    let description = unsafe { std::ffi::CStr::from_ptr(gai_math_description()) };
    description.to_str().unwrap_or("V8-exact Math")
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_patched_engine_is_linked() {
        assert_eq!(super::math(), "V8 14.6.202.34 fdlibm with FMA contraction, V8 Math.hypot, CORE-MATH pow 708e86ef");
    }
}
