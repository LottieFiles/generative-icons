//! Diagnostics: every problem names a JSON Pointer into the spec, a stable code and, where
//! one exists, the fix, so an agent can patch the spec without reading the compiler.

use serde_json::{Value, json};
use std::fmt;

/// How bad a finding is. Errors stop `draft` and `build` before any engine call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Info,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Info => "info",
        }
    }
}

/// One finding.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub level: Level,
    /// A stable code, e.g. `V2_SCHEMA`, `ONE_SUBPATH`, `REST_VALUE`.
    pub code: &'static str,
    /// JSON Pointer into the spec (`/motion/door.swing/strong/2`), or `""` for the whole spec.
    pub path: String,
    pub message: String,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic { level: Level::Error, code, path: path.into(), message: message.into(), hint: None }
    }

    pub fn warning(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic { level: Level::Warning, code, path: path.into(), message: message.into(), hint: None }
    }

    pub fn info(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic { level: Level::Info, code, path: path.into(), message: message.into(), hint: None }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// `{severity, code, path, message, hint?}`, the shape `check` prints.
    pub fn to_json(&self) -> Value {
        let mut value = json!({
            "severity": self.level.as_str(),
            "code": self.code,
            "path": self.path,
            "message": self.message,
        });
        if let Some(hint) = &self.hint {
            value["hint"] = json!(hint);
        }
        value
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = if self.path.is_empty() { String::new() } else { format!(" at {}", self.path) };
        write!(f, "{} {}{at}: {}", self.level.as_str(), self.code, self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, " (hint: {hint})")?;
        }
        Ok(())
    }
}

/// A list of findings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    pub fn push(&mut self, d: Diagnostic) {
        self.0.push(d);
    }

    pub fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.level == Level::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter().filter(|d| d.level == Level::Error)
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.0.extend(other.0);
    }

    pub fn to_json(&self) -> Value {
        Value::Array(self.0.iter().map(Diagnostic::to_json).collect())
    }
}

/// A JSON Pointer segment, escaped (`~` → `~0`, `/` → `~1`).
pub fn seg(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// `base/key`, with `key` escaped.
pub fn at(base: &str, key: impl fmt::Display) -> String {
    format!("{base}/{}", seg(&key.to_string()))
}

/// The closest of `options` to `word` by edit distance, if it is close enough to be a typo.
pub fn did_you_mean<'a>(word: &str, options: &[&'a str]) -> Option<&'a str> {
    let limit = if word.len() <= 4 { 1 } else { 2 };
    options
        .iter()
        .map(|o| (levenshtein(&word.to_lowercase(), &o.to_lowercase()), *o))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, o)| o)
}

/// Optimal string alignment distance: a swap of two neighbours counts as one edit.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions() {
        assert_eq!(did_you_mean("cirle", &["circle", "rect", "line"]), Some("circle"));
        assert_eq!(did_you_mean("hti", &["hit", "in", "out"]), Some("hit"));
        assert_eq!(did_you_mean("banana", &["circle", "rect"]), None);
    }

    #[test]
    fn pointers_escape() {
        assert_eq!(at("/motion", "a/b~c"), "/motion/a~1b~0c");
    }
}
