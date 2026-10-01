//! Shared helpers for the parity tests: repository discovery, oracle files and
//! PNG comparison.

use gai_render::Report;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The repository root (two levels above this crate).
pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A set's packaged assets and committed evidence.
pub struct SetDirs {
    pub assets: PathBuf,
    pub evidence: PathBuf,
}

impl SetDirs {
    /// The set's dirs when set.json and `browser-verification.json` exist.
    pub fn find(id: &str) -> Option<SetDirs> {
        let root = repo();
        let prefix = format!("{id}-");
        let evidence = fs::read_dir(root.join("motion/icon-sets"))
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&prefix)))?
            .join("evidence");
        let assets = root.join("prototype/icons").join(id).join("assets");
        (assets.join("set.json").is_file() && evidence.join("browser-verification.json").is_file())
            .then_some(SetDirs { assets, evidence })
    }

    /// Every set id with an evidence report, sorted.
    pub fn all_ids() -> Vec<String> {
        let mut ids: Vec<String> = fs::read_dir(repo().join("motion/icon-sets"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_str()?.to_string();
                let id = name.get(..2)?.to_string();
                (name.as_bytes().get(2) == Some(&b'-') && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
            })
            .filter(|id| SetDirs::find(id).is_some())
            .collect();
        ids.sort();
        ids
    }

    /// The committed `browser-verification.json` bytes and parsed value.
    pub fn oracle(&self) -> (Vec<u8>, Value) {
        let bytes = fs::read(self.evidence.join("browser-verification.json")).expect("oracle");
        let value = serde_json::from_slice(&bytes).expect("oracle parses");
        (bytes, value)
    }

    pub fn oracle_report(&self) -> Report {
        Report::from_value(&self.oracle().1).expect("oracle report")
    }

    pub fn lottie(&self, action: &str) -> Vec<u8> {
        fs::read(self.assets.join(format!("{action}.lottie"))).expect("lottie")
    }
}

/// The oracle's asset record for an action.
pub fn oracle_record<'a>(oracle: &'a Value, name: &str) -> &'a Value {
    oracle["assets"]
        .as_array()
        .and_then(|a| a.iter().find(|r| r["name"] == name))
        .unwrap_or_else(|| panic!("no oracle record for {name}"))
}

/// Copies the browser's wall-clock values into a native record: the playback
/// frame-event count (rAF timing plus stale events) is not reproducible.
pub fn adopt_clock_record(native: &mut Value, oracle: &Value) {
    if let Some(events) = oracle.pointer("/playback/observedFrameEvents")
        && let Some(slot) = native.pointer_mut("/playback/observedFrameEvents")
    {
        *slot = events.clone();
    }
}

/// Copies the run date and every action's frame-event count from the oracle report.
pub fn adopt_clock(native: &mut Value, oracle: &Value) {
    native["date"] = oracle["date"].clone();
    let Some(assets) = native["assets"].as_array_mut() else { return };
    for record in assets {
        let name = record["name"].as_str().unwrap_or_default().to_string();
        if let Some(old) = oracle["assets"].as_array().and_then(|a| a.iter().find(|r| r["name"] == name.as_str())) {
            adopt_clock_record(record, old);
        }
    }
}

/// Decodes a PNG to RGBA8.
pub fn decode_png(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut decoder = png::Decoder::new(bytes);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().expect("png header");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png data");
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        other => panic!("unexpected PNG color type {other:?}"),
    };
    (info.width, info.height, rgba)
}

/// How a sheet differs from the browser's.
pub struct SheetDifference {
    /// Largest channel difference over the drawing (rows `0..rows`).
    pub drawing: u8,
    /// Largest channel difference over the label band (rows `rows..`), drawn in another font.
    pub labels: u8,
    /// Whether any alpha differs, anywhere.
    pub alpha: bool,
}

/// Compares two sheets of the `expected` size whose drawing fills rows `0..rows`.
pub fn sheet_difference(ours: &[u8], theirs: &[u8], expected: (u32, u32), rows: u32) -> Result<SheetDifference, String> {
    let (w, h, a) = decode_png(ours);
    let (w2, h2, b) = decode_png(theirs);
    if (w, h) != expected || (w2, h2) != expected {
        return Err(format!("size {w}x{h} vs browser {w2}x{h2}, expected {expected:?}"));
    }
    let end = (rows * w * 4) as usize;
    let mut difference = SheetDifference { drawing: 0, labels: 0, alpha: false };
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        let max = if i < end { &mut difference.drawing } else { &mut difference.labels };
        *max = (*max).max(x.abs_diff(*y));
        difference.alpha |= i % 4 == 3 && x != y;
    }
    Ok(difference)
}
