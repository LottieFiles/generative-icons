//! dotLottie v2 archives (`package-set.py:21-45`). Each action of an icon gets its own
//! `.lottie` holding every action of that icon; only `manifest.json`'s `initial` differs.
//!
//! Entry order: `manifest.json`, `a/<action>.json` for each action in the icon's order, then
//! `t/dark.json` and `t/light.json` copied from the initial action's theme files.

use crate::error::{Result, invalid};
use crate::io::{object, py_compact, read, read_json};
use crate::zip::{ZipDate, ZipEntry, find_file, read_zip, write_zip};
use gai_core::sha256_hex;
use serde_json::{Value, json};
use std::path::Path;

/// The theme ids every archive carries, in entry order.
pub const THEMES: [&str; 2] = ["dark", "light"];

/// One action's `.lottie` and the manifest written beside it as `<name>.manifest.json`.
#[derive(Debug, Clone)]
pub struct Archive {
    /// The action name, which is also the file stem.
    pub name: String,
    /// `manifest.json`, Python `json.dumps` format without a trailing newline.
    pub manifest: String,
    /// The archive bytes.
    pub lottie: Vec<u8>,
    /// Lowercase hex sha256 of `lottie`.
    pub sha256: String,
}

/// The final manifest of `initial`'s archive. `template` is the manifest the build wrote;
/// `initial` and `animations` are reassigned in place, so its key order is kept.
pub fn final_manifest(mut template: Value, initial: &str, actions: &[String]) -> Result<Value> {
    let map = object(&mut template, "manifest")?;
    map.insert("initial".into(), json!({ "animation": initial }));
    let animations = actions
        .iter()
        .map(|id| json!({ "id": id, "initialTheme": "dark", "themes": THEMES }))
        .collect();
    map.insert("animations".into(), Value::Array(animations));
    Ok(template)
}

/// Builds the `.lottie` of every action of one icon from the asset folder, reading
/// `<a>.json` for each action and `<name>.manifest.json` plus `<name>.theme-*.json` for each
/// archive. Every archive is re-read and checked like `testzip()`.
pub fn build_icon_archives(
    assets: &Path,
    actions: &[String],
    date: ZipDate,
) -> Result<Vec<Archive>> {
    let sources: Vec<Vec<u8>> = actions
        .iter()
        .map(|a| read(&assets.join(format!("{a}.json"))))
        .collect::<Result<_>>()?;
    // Deflate is deterministic, so each action is compressed once for all archives.
    let shared: Vec<ZipEntry> = actions
        .iter()
        .zip(&sources)
        .map(|(a, data)| ZipEntry::deflate(format!("a/{a}.json"), data, date))
        .collect();
    let mut archives = Vec::with_capacity(actions.len());
    for name in actions {
        let template = read_json(&assets.join(format!("{name}.manifest.json")))?;
        let manifest = py_compact(&final_manifest(template, name, actions)?);
        let mut entries = Vec::with_capacity(actions.len() + 3);
        entries.push(ZipEntry::deflate(
            "manifest.json",
            manifest.as_bytes(),
            date,
        ));
        entries.extend(shared.iter().cloned());
        for theme in THEMES {
            let data = read(&assets.join(format!("{name}.theme-{theme}.json")))?;
            entries.push(ZipEntry::deflate(format!("t/{theme}.json"), &data, date));
        }
        let path = assets.join(format!("{name}.lottie"));
        let lottie = write_zip(&entries).map_err(|e| crate::PackageError::zip(&path, e))?;
        let files = read_zip(&lottie).map_err(|e| crate::PackageError::zip(&path, e))?;
        for (action, data) in actions.iter().zip(&sources) {
            if find_file(&files, &format!("a/{action}.json")) != Some(data.as_slice()) {
                return Err(invalid(format!(
                    "{name}.lottie does not preserve a/{action}.json"
                )));
            }
        }
        let sha256 = sha256_hex(&lottie);
        archives.push(Archive {
            name: name.clone(),
            manifest,
            lottie,
            sha256,
        });
    }
    Ok(archives)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_keeps_template_order() {
        let template: Value = serde_json::from_str(
            r#"{"version":"2","generator":"LottieFiles Motion / Engine MCP","initial":{"animation":"x"},"animations":[{"id":"x","initialTheme":"dark","themes":["dark","light"]}],"themes":[{"id":"dark"},{"id":"light"}]}"#,
        )
        .unwrap();
        let manifest = final_manifest(
            template,
            "bank-strong",
            &["bank".into(), "bank-strong".into()],
        )
        .unwrap();
        assert_eq!(
            py_compact(&manifest),
            r#"{"version": "2", "generator": "LottieFiles Motion / Engine MCP", "initial": {"animation": "bank-strong"}, "animations": [{"id": "bank", "initialTheme": "dark", "themes": ["dark", "light"]}, {"id": "bank-strong", "initialTheme": "dark", "themes": ["dark", "light"]}], "themes": [{"id": "dark"}, {"id": "light"}]}"#
        );
    }

    #[test]
    fn archive_layout() {
        let dir = tempfile::tempdir().unwrap();
        let template = r#"{"version":"2","generator":"g","initial":{"animation":"a"},"animations":[],"themes":[{"id":"dark"},{"id":"light"}]}"#;
        for name in ["a", "b"] {
            std::fs::write(
                dir.path().join(format!("{name}.json")),
                format!("{{\"nm\":\"{name}\"}}"),
            )
            .unwrap();
            std::fs::write(dir.path().join(format!("{name}.manifest.json")), template).unwrap();
            for theme in THEMES {
                std::fs::write(
                    dir.path().join(format!("{name}.theme-{theme}.json")),
                    format!("{{\"{theme}\":\"{name}\"}}"),
                )
                .unwrap();
            }
        }
        let archives =
            build_icon_archives(dir.path(), &["a".into(), "b".into()], ZipDate::SETS).unwrap();
        let files = read_zip(&archives[1].lottie).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "manifest.json",
                "a/a.json",
                "a/b.json",
                "t/dark.json",
                "t/light.json"
            ]
        );
        assert_eq!(
            find_file(&files, "t/light.json"),
            Some(br#"{"light":"b"}"#.as_slice())
        );
        assert!(
            archives[1]
                .manifest
                .contains(r#""initial": {"animation": "b"}"#)
        );
    }
}
