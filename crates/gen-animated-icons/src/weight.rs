//! Bakes an in-between outline weight into a `.lottie` file.
//!
//! Ports `rewriteStrokeWidth` and its minimal ZIP reader and writer from
//! `packages/runtime/src/weight.mjs`: native players ignore the `style.outline.width`
//! token, so the width is written into every token-bound stroke. Rewritten JSON is
//! compact `JSON.stringify` output and the archive is rewritten with stored entries.

use crate::error::{CliError, ErrorCode, Result};
use crate::js::truthy;
use serde_json::{Number, Value, json};

/// Slot and theme-rule id of the continuous outline weight.
pub const STROKE_WIDTH_TOKEN: &str = "style.outline.width";

/// Rewrites every token-bound stroke width, the slot default and theme rules to `weight`.
pub fn rewrite_stroke_width(bytes: &[u8], weight: f64) -> Result<Vec<u8>> {
    let width = Number::from_f64(weight)
        .map(Value::Number)
        .ok_or_else(|| CliError::usage(format!("Invalid stroke weight {weight}")))?;
    let mut entries = read_zip(bytes)?;
    for entry in &mut entries {
        if !entry.name.ends_with(".json") {
            continue;
        }
        let is_animation = entry.name.starts_with("a/");
        if !is_animation && !entry.name.starts_with("t/") {
            continue;
        }
        let text = String::from_utf8_lossy(&entry.data);
        let mut json: Value = serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(&text))
            .map_err(|error| corrupt(format!("Cannot parse {}: {error}", entry.name)))?;
        if is_animation {
            if let Some(Value::Array(layers)) = json.get_mut("layers") {
                for layer in layers {
                    rewrite_shapes(layer.get_mut("shapes"), &width);
                }
            }
            if let Some(slot) = json.get_mut("slots").and_then(|slots| slots.get_mut(STROKE_WIDTH_TOKEN))
                && truthy(slot)
            {
                *slot = json!({ "p": { "a": 0, "k": width } });
            }
        } else if let Some(Value::Array(rules)) = json.get_mut("rules") {
            for rule in
                rules.iter_mut().filter(|rule| rule.get("id").and_then(Value::as_str) == Some(STROKE_WIDTH_TOKEN))
            {
                if let Value::Object(rule) = rule {
                    rule.insert("value".into(), width.clone());
                }
            }
        }
        entry.data = gai_core::to_js_json(&json).into_bytes();
    }
    Ok(write_zip(&entries))
}

fn rewrite_shapes(shapes: Option<&mut Value>, width: &Value) {
    let Some(Value::Array(shapes)) = shapes else { return };
    for shape in shapes {
        match shape.get("ty").and_then(Value::as_str) {
            Some("gr") => rewrite_shapes(shape.get_mut("it"), width),
            // Constant-width family strokes only; an animated width keeps its keyframes.
            Some("st") => {
                if let Some(Value::Object(w)) = shape.get_mut("w")
                    && w.get("sid").and_then(Value::as_str) == Some(STROKE_WIDTH_TOKEN)
                    && !w.get("a").is_some_and(truthy)
                {
                    w.insert("k".into(), width.clone());
                }
            }
            _ => {}
        }
    }
}

fn corrupt(message: impl Into<String>) -> CliError {
    CliError::new(ErrorCode::CorruptArchive, message)
}

/// One archive member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Member name, e.g. `a/heart.json`.
    pub name: String,
    /// Uncompressed bytes.
    pub data: Vec<u8>,
}

fn u16_at(bytes: &[u8], at: usize) -> Result<u16> {
    bytes.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or_else(|| corrupt("Corrupt .lottie archive"))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| corrupt("Corrupt .lottie archive"))
}

/// Most members an archive may have (dotlottie-rs's limit).
const MAX_ENTRIES: usize = 4096;
/// Largest member, uncompressed (dotlottie-rs's limit).
const MAX_ENTRY_BYTES: usize = 64 << 20;
/// Largest archive, uncompressed.
const MAX_TOTAL_BYTES: usize = 256 << 20;

/// Reads members in central-directory order; stored and deflated entries only. A member
/// larger than its declared size or 64 MiB, more than 4096 members, or more than 256 MiB
/// in total is refused, so a crafted archive cannot exhaust memory.
pub fn read_zip(bytes: &[u8]) -> Result<Vec<Entry>> {
    const EOCD: u32 = 0x0605_4b50;
    let last = bytes.len().checked_sub(22).ok_or_else(|| corrupt("Not a .lottie archive"))?;
    let eocd = (0..=last)
        .rev()
        .find(|&at| bytes[at..at + 4] == EOCD.to_le_bytes())
        .ok_or_else(|| corrupt("Not a .lottie archive"))?;
    let count = u16_at(bytes, eocd + 10)?;
    if usize::from(count) > MAX_ENTRIES {
        return Err(corrupt(format!("The .lottie archive has more than {MAX_ENTRIES} entries")));
    }
    let mut offset = u32_at(bytes, eocd + 16)? as usize;
    let mut entries = Vec::with_capacity(count.into());
    let mut total = 0usize;
    let too_large = || corrupt("The .lottie archive expands beyond its declared or allowed size");
    for _ in 0..count {
        if u32_at(bytes, offset)? != 0x0201_4b50 {
            return Err(corrupt("Corrupt .lottie archive"));
        }
        let method = u16_at(bytes, offset + 10)?;
        let compressed = u32_at(bytes, offset + 20)? as usize;
        let declared = u32_at(bytes, offset + 24)? as usize;
        if declared > MAX_ENTRY_BYTES || total + declared > MAX_TOTAL_BYTES {
            return Err(too_large());
        }
        let name_len = usize::from(u16_at(bytes, offset + 28)?);
        let extra_len = usize::from(u16_at(bytes, offset + 30)?);
        let comment_len = usize::from(u16_at(bytes, offset + 32)?);
        let local = u32_at(bytes, offset + 42)? as usize;
        let name = bytes.get(offset + 46..offset + 46 + name_len).ok_or_else(|| corrupt("Corrupt .lottie archive"))?;
        let start = local + 30 + usize::from(u16_at(bytes, local + 26)?) + usize::from(u16_at(bytes, local + 28)?);
        let raw = bytes.get(start..start + compressed).ok_or_else(|| corrupt("Corrupt .lottie archive"))?;
        let data = match method {
            0 if raw.len() == declared => raw.to_vec(),
            0 => return Err(corrupt("Corrupt .lottie archive")),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, declared).map_err(|error| {
                match error.status {
                    miniz_oxide::inflate::TINFLStatus::HasMoreOutput => too_large(),
                    _ => corrupt("Corrupt .lottie archive"),
                }
            })?,
            other => return Err(corrupt(format!("Unsupported .lottie compression method {other}"))),
        };
        total += data.len();
        entries.push(Entry { name: String::from_utf8_lossy(name).into_owned(), data });
        offset += 46 + name_len + extra_len + comment_len;
    }
    Ok(entries)
}

/// Writes stored members with zeroed timestamps: local headers, central directory, end record.
pub fn write_zip(entries: &[Entry]) -> Vec<u8> {
    let mut locals = Vec::new();
    let mut centrals = Vec::new();
    for entry in entries {
        let name = entry.name.as_bytes();
        let crc = crc32fast::hash(&entry.data);
        let size = entry.data.len() as u32;
        let offset = locals.len() as u32;
        // Local header: signature, version 20, flags/method/time/date 0, crc, sizes, name length.
        locals.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        locals.extend_from_slice(&20u16.to_le_bytes());
        locals.extend_from_slice(&[0; 8]);
        locals.extend_from_slice(&crc.to_le_bytes());
        locals.extend_from_slice(&size.to_le_bytes());
        locals.extend_from_slice(&size.to_le_bytes());
        locals.extend_from_slice(&(name.len() as u16).to_le_bytes());
        locals.extend_from_slice(&[0; 2]);
        locals.extend_from_slice(name);
        locals.extend_from_slice(&entry.data);
        // Central entry: made-by and needed 20, zeroed flags/method/time/date/extras, local offset.
        centrals.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        centrals.extend_from_slice(&20u16.to_le_bytes());
        centrals.extend_from_slice(&20u16.to_le_bytes());
        centrals.extend_from_slice(&[0; 8]);
        centrals.extend_from_slice(&crc.to_le_bytes());
        centrals.extend_from_slice(&size.to_le_bytes());
        centrals.extend_from_slice(&size.to_le_bytes());
        centrals.extend_from_slice(&(name.len() as u16).to_le_bytes());
        centrals.extend_from_slice(&[0; 12]);
        centrals.extend_from_slice(&offset.to_le_bytes());
        centrals.extend_from_slice(name);
    }
    let count = (entries.len() as u16).to_le_bytes();
    let mut out = locals;
    let central_offset = out.len() as u32;
    let central_size = centrals.len() as u32;
    out.extend_from_slice(&centrals);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&count);
    out.extend_from_slice(&count);
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&[0; 2]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, data: &str) -> Entry {
        Entry { name: name.into(), data: data.as_bytes().to_vec() }
    }

    #[test]
    fn stored_archives_round_trip() {
        let entries = vec![entry("manifest.json", "{}"), entry("a/x.json", "{\"v\":1}")];
        assert_eq!(read_zip(&write_zip(&entries)).unwrap(), entries);
    }

    #[test]
    fn deflated_members_are_inflated() {
        let data = "{\"layers\":[]}".repeat(20);
        let mut zip = write_zip(&[entry("a/x.json", &data)]);
        // Swap the stored body for a raw deflate stream and patch method and sizes.
        let packed = miniz_oxide::deflate::compress_to_vec(data.as_bytes(), 9);
        let name_end = 30 + "a/x.json".len();
        let central = zip[name_end + data.len()..].to_vec();
        zip.truncate(name_end);
        zip[8..10].copy_from_slice(&8u16.to_le_bytes());
        zip[18..22].copy_from_slice(&(packed.len() as u32).to_le_bytes());
        zip.extend_from_slice(&packed);
        let central_at = zip.len();
        zip.extend_from_slice(&central);
        zip[central_at + 10..central_at + 12].copy_from_slice(&8u16.to_le_bytes());
        zip[central_at + 20..central_at + 24].copy_from_slice(&(packed.len() as u32).to_le_bytes());
        let eocd = zip.len() - 22;
        zip[eocd + 16..eocd + 20].copy_from_slice(&(central_at as u32).to_le_bytes());
        assert_eq!(read_zip(&zip).unwrap()[0].data, data.as_bytes());

        // A member that inflates past its declared size is refused, not expanded.
        let mut bomb = zip.clone();
        bomb[central_at + 24..central_at + 28].copy_from_slice(&10u32.to_le_bytes());
        assert_eq!(
            read_zip(&bomb).unwrap_err().message,
            "The .lottie archive expands beyond its declared or allowed size"
        );
        // So is one declaring more than the per-member limit.
        let mut huge = zip;
        huge[central_at + 24..central_at + 28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(read_zip(&huge).unwrap_err().code, ErrorCode::CorruptArchive);
    }

    #[test]
    fn archives_with_too_many_entries_are_refused() {
        let entries: Vec<Entry> = (0..=MAX_ENTRIES).map(|i| entry(&format!("a/{i}.json"), "{}")).collect();
        assert_eq!(
            read_zip(&write_zip(&entries)).unwrap_err().message,
            format!("The .lottie archive has more than {MAX_ENTRIES} entries")
        );
    }

    #[test]
    fn garbage_is_not_an_archive() {
        assert_eq!(read_zip(b"not a zip").unwrap_err().message, "Not a .lottie archive");
        assert_eq!(read_zip(b"").unwrap_err().message, "Not a .lottie archive");
    }

    #[test]
    fn rewrites_token_strokes_slots_and_rules() {
        let animation = r#"{"layers":[{"shapes":[{"ty":"gr","it":[{"ty":"st","w":{"a":0,"k":2,"sid":"style.outline.width"}},{"ty":"st","w":{"a":1,"k":[],"sid":"style.outline.width"}},{"ty":"st","w":{"k":2}}]}]}],"slots":{"style.outline.width":{"p":{"a":0,"k":2}},"x":{"p":1.0}}}"#;
        let theme = r#"{"rules":[{"id":"style.outline.width","type":"Scalar","value":2},{"id":"other","value":2}]}"#;
        let archive = write_zip(&[
            entry("manifest.json", "{\"v\": 1.0}"),
            entry("a/i.json", animation),
            entry("t/dark.json", theme),
        ]);
        let out = read_zip(&rewrite_stroke_width(&archive, 1.75).unwrap()).unwrap();
        assert_eq!(out[0].data, b"{\"v\": 1.0}");
        assert_eq!(
            String::from_utf8(out[1].data.clone()).unwrap(),
            r#"{"layers":[{"shapes":[{"ty":"gr","it":[{"ty":"st","w":{"a":0,"k":1.75,"sid":"style.outline.width"}},{"ty":"st","w":{"a":1,"k":[],"sid":"style.outline.width"}},{"ty":"st","w":{"k":2}}]}]}],"slots":{"style.outline.width":{"p":{"a":0,"k":1.75}},"x":{"p":1}}}"#
        );
        assert_eq!(
            String::from_utf8(out[2].data.clone()).unwrap(),
            r#"{"rules":[{"id":"style.outline.width","type":"Scalar","value":1.75},{"id":"other","value":2}]}"#
        );
    }
}
