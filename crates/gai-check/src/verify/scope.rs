//! The AI scope gate of `library/run-ai.mjs` (`npm run verify:ai`): before a set listed in
//! `library/design-reviews/ai-category/scope.json` is verified, its definitions' icon
//! names must equal the scope's list, in order. This executes the authoring code (QuickJS).

use super::{SetContext, VerifyError, ensure, read_json};
use gai_js::rquickjs::{Array, Object, Value as JsValue};
use gai_js::{Limits, ModernProfile, SetRuntime, SetSource, run_on_worker};
use serde_json::Value;
use std::io::ErrorKind;

/// The scope file, relative to the kit folder.
pub const SCOPE_FILE: &str = "design-reviews/ai-category/scope.json";

/// Checks the set against the AI scope, if the scope lists it.
pub fn check(set: &SetContext<'_>) -> Result<(), VerifyError> {
    let path = set.paths.library_dir.join(SCOPE_FILE);
    let scope = match read_json(&path) {
        Err(VerifyError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
            return Ok(());
        }
        other => other?,
    };
    let id = set.manifest.id();
    let Some(listed) = scope
        .get("sets")
        .and_then(Value::as_array)
        .and_then(|sets| sets.iter().find(|s| s.get("id") == Some(&Value::from(id))))
    else {
        return Ok(());
    };
    let source = SetSource::Definitions {
        set_dir: set.paths.set_dir.clone(),
        library_dir: set.paths.library_dir.clone(),
        profile: ModernProfile::SHARED,
    };
    let limits = Limits::default();
    let names = run_on_worker(&limits, || -> Result<Vec<Value>, gai_js::Error> {
        SetRuntime::load(&source, &limits)?.with_icons(|scope| {
            let icons: Array = Array::from_value(scope.icons().clone())?;
            let mut names = Vec::new();
            for icon in icons.iter::<JsValue>() {
                let name: JsValue = Object::from_value(icon?)?.get("name")?;
                // JSON.stringify writes an undefined array item as null.
                names.push(scope.project(name)?.unwrap_or(Value::Null));
            }
            Ok(names)
        })
    })
    .map_err(|e| VerifyError::Assertion(e.to_string()))?;
    let want = listed.get("icons").cloned().unwrap_or(Value::Null);
    let same = gai_core::to_js_json(&Value::Array(names)) == gai_core::to_js_json(&want);
    ensure(same, || format!("Incomplete or misordered AI scope: {id}"))
}
