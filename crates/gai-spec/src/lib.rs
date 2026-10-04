//! Icon spec v2 (`gai-icon/2`): an animated icon as JSON data (shapes, roles, rigs and
//! eased keys, no functions of time), validated offline and compiled into ONE engine call
//! that builds every motion with native eased keys, rigs, masks, mattes and markers, then
//! exports them as one bundle or renders a draft sheet.
//!
//! ```text
//! <id>.icon.json ─ parse (structure) ─ compile (names, rigs, motions, layers) ─ lint
//!                 └──────────────────────────────► Plan (gai-plan/1) + SCRIPT ─► creator_exec
//! ```

pub mod compile;
pub mod diag;
pub mod ease;
pub mod geom;
pub mod lint;
pub mod motion;
pub mod plan;
pub mod port;
pub mod spec;

pub use compile::{Compiled, compile};
pub use diag::{Diagnostic, Diagnostics, Level};
pub use spec::{FORMAT, Spec, parse};

use serde_json::{Value, json};

/// The JSON Schema of a spec (editors, MCP tool inputs). The CLI validates by hand with the
/// same rules and better messages.
pub const SCHEMA: &str = include_str!("../assets/icon-v2.schema.json");
/// The engine script: one `creator_exec` per icon, params `{plan, file?, sheet?}`.
pub const SCRIPT: &str = include_str!("../assets/build-v2.js");
/// The file name suffix of a v2 spec.
pub const SUFFIX: &str = ".icon.json";

/// Parses, compiles and lints a spec.
pub fn load(text: &str) -> Result<Compiled, Diagnostics> {
    let spec = parse(text)?;
    compile(&spec)
}

/// `params` of a build: every motion exported as one bundle file.
pub fn build_params(c: &Compiled, file: &str) -> Value {
    json!({"plan": c.plan.to_json(), "file": file})
}

/// `params` of a draft: the review sheet only, nothing stored.
pub fn draft_params(c: &Compiled) -> Value {
    json!({"plan": c.plan.to_json(), "sheet": c.sheet()})
}

/// The complete engine call of a build, minus the document id: the golden form of a compile.
pub fn engine_call(c: &Compiled, file: &str) -> Value {
    json!({"code_sha256": gai_core::sha256_hex(SCRIPT.as_bytes()), "params": build_params(c, file)})
}
