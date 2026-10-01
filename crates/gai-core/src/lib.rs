//! Foundations shared by every gen-animated-icons crate.
//!
//! The authoring pipeline this workspace replaces was written in JavaScript, and its
//! committed outputs were produced by `JSON.stringify` and `Math.round`. These helpers
//! reproduce those semantics exactly so Rust output stays byte-identical.

pub mod hash;
pub mod js;
pub mod json;
pub mod jsmath;
pub mod parallel;
pub mod pyjson;

pub use hash::sha256_hex;
pub use json::{to_js_json, to_js_json_pretty};
pub use jsmath::{js_number_to_string, js_round, round5};
pub use pyjson::{py_float_repr, to_py_json};
pub mod export;
