//! The packaged `set.json` as verification reads it: a parsed JSON tree with JS property
//! semantics, since the checks echo records verbatim (`designConstraints`), compare them
//! with `deepStrictEqual`, and test flags by truthiness.

use super::VerifyError;
use super::js;
use serde_json::Value;
use std::path::Path;

/// A parsed `set.json`.
#[derive(Debug, Clone)]
pub struct SetManifest {
    /// The whole document.
    pub doc: Value,
}

impl SetManifest {
    /// Reads and parses a `set.json`.
    pub fn read(path: &Path) -> Result<SetManifest, VerifyError> {
        Ok(SetManifest {
            doc: super::read_json(path)?,
        })
    }

    /// `set.id` (empty when absent).
    pub fn id(&self) -> &str {
        self.doc
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    /// `set.icons`; the verifiers crash on a manifest without them.
    pub fn icons(&self) -> Result<Vec<Record<'_>>, VerifyError> {
        array(self.doc.get("icons"), "set.icons").map(|a| a.iter().map(Record).collect())
    }

    /// `set.icons.flatMap(icon => icon.animations)`: every action, reverses included.
    pub fn animations(&self) -> Result<Vec<Record<'_>>, VerifyError> {
        let mut out = Vec::new();
        for icon in self.icons()? {
            out.extend(icon.list("animations")?);
        }
        Ok(out)
    }

    /// `set.icons.flatMap(icon => icon.motions)`: the forward Light and Strong actions.
    pub fn motions(&self) -> Result<Vec<Record<'_>>, VerifyError> {
        let mut out = Vec::new();
        for icon in self.icons()? {
            out.extend(icon.list("motions")?);
        }
        Ok(out)
    }
}

/// A JSON array, or the TypeError the JS would throw on anything else.
pub fn array<'a>(v: Option<&'a Value>, what: &str) -> Result<&'a Vec<Value>, VerifyError> {
    v.and_then(Value::as_array)
        .ok_or_else(|| VerifyError::Assertion(format!("{what} is not an array")))
}

/// One icon or action record.
#[derive(Debug, Clone, Copy)]
pub struct Record<'a>(pub &'a Value);

impl<'a> Record<'a> {
    /// A property; `None` is `undefined`.
    pub fn get(&self, key: &str) -> Option<&'a Value> {
        self.0.get(key)
    }

    /// A string property.
    pub fn str(&self, key: &str) -> Option<&'a str> {
        self.get(key).and_then(Value::as_str)
    }

    /// `record.name`, which names every file of the action.
    pub fn name(&self) -> Result<&'a str, VerifyError> {
        self.str("name")
            .ok_or_else(|| VerifyError::Assertion("a set.json record has no name".into()))
    }

    /// The truthiness of a property.
    pub fn flag(&self, key: &str) -> bool {
        js::truthy(self.get(key))
    }

    /// A property coerced to a number, as JS arithmetic would.
    pub fn number(&self, key: &str) -> f64 {
        js::to_number(self.get(key))
    }

    /// An array property of records (`icon.animations`, `icon.motions`).
    pub fn list(&self, key: &str) -> Result<Vec<Record<'a>>, VerifyError> {
        array(self.get(key), key).map(|a| a.iter().map(Record).collect())
    }
}
