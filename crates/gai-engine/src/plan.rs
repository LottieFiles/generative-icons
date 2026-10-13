//! Spec v2 on the hosted engine: one `creator_exec` per icon runs a plan script (the script and
//! its params come from the caller, `gai-spec`), on a document pool of its own.
//!
//! - [`PlanRunner::draft`] returns the images the script attached (`renderFilmstrip`); nothing
//!   is stored.
//! - [`PlanRunner::export_bundle`] has the script export every motion as precomps of one
//!   bundle comp (ONE stored file), downloads it through its signed URL (falling back to
//!   `resources/read`), deletes it, splits it into one Lottie per motion and normalizes each
//!   with [`normalize_native`], unchanged from v1.
//!
//! v2 documents are separate from v1's: the v2 script leaves precomps behind, and v1's export
//! script clears in an order that fails on them.

use std::thread;
use std::time::{Duration, Instant, SystemTime};

use gai_core::sha256_hex;
use reqwest::blocking::Client;
use serde_json::{Map, Value, json};

use crate::config::{EngineConfig, MAX_DOCUMENTS, RetryPolicy};
use crate::error::{Error, Result};
use crate::export::DocumentPool;
use crate::http;
use crate::mcp::{McpClient, ToolImage};
use crate::normalize::{NativeLottie, normalize_native};

/// The engine's maximum script run time.
const SCRIPT_TIMEOUT_MS: u64 = 30_000;
/// Default prefix of the v2 document ids: `<prefix>-1`, `<prefix>-2`, … (was
/// `gen-animated-icons-v2` until engine 0.4.1 could no longer save those; see UNSAVABLE).
pub const DOCUMENT_PREFIX: &str = "generative-icons-v2";
/// Polls of `creator_download_file` while a file is still encoding (Lottie is ready at once).
const MAX_ENCODING_POLLS: u32 = 10;
/// The engine's error for a document it can no longer save: under engine 0.4.1, removing a
/// composition 0.4.0 wrote fails it, so every run that clears such a document does, while the
/// same script runs on a fresh document. The engine resets a document only with an interactive
/// confirmation, so a run that meets it moves to a fresh document.
const UNSAVABLE: &str = "snapshot serialization failed";

/// What one script run returned.
#[derive(Debug, Clone)]
pub struct Run {
    /// The script's return value (`result`).
    pub result: Value,
    /// Attached images (PNG sheets).
    pub images: Vec<ToolImage>,
    pub document: String,
    pub revision: Value,
    /// Wall time of the `creator_exec` round trip.
    pub round_trip: Duration,
    /// When the request was sent, by this machine's clock: with the script's own
    /// `startedAt` it splits the round trip into before, in and after the script.
    pub sent: SystemTime,
}

/// One motion of an exported bundle.
#[derive(Debug, Clone)]
pub struct BundleAnimation {
    pub name: String,
    pub native: NativeLottie,
    pub provenance: Value,
}

/// An exported bundle, split.
#[derive(Debug, Clone)]
pub struct Bundle {
    pub animations: Vec<BundleAnimation>,
    pub run: Run,
    pub hosted_bytes: usize,
    /// Round trips of the download and the delete.
    pub download: Duration,
}

/// Runs plan scripts on a pool of v2 documents.
pub struct PlanRunner {
    client: McpClient,
    download: Client,
    engine: String,
    retry: RetryPolicy,
    pool: DocumentPool,
}

impl PlanRunner {
    /// A runner for `config` on documents `<prefix>-1..n` (`config.documents` of them).
    pub fn new(config: &EngineConfig, prefix: &str) -> Result<Self> {
        let ids = (1..=config.documents.clamp(1, MAX_DOCUMENTS)).map(|i| format!("{prefix}-{i}")).collect();
        Ok(PlanRunner { client: McpClient::new(config)?, download: http::client()?, engine: config.url.to_string(), retry: config.retry, pool: DocumentPool::new(ids) })
    }

    /// The engine's identity (`initialize`).
    pub fn server(&self) -> Result<crate::mcp::ServerInfo> {
        self.client.server_info()
    }

    /// Runs `code` with `params` once, retrying failures the engine marks retryable.
    pub fn run(&self, code: &str, params: Value) -> Result<Run> {
        let document = self.pool.acquire();
        self.run_on(document.id(), code, params)
    }

    fn run_on(&self, document: &str, code: &str, params: Value) -> Result<Run> {
        let mut document = document.to_owned();
        let mut arguments = json!({"code": code, "documentId": document, "params": params, "timeoutMs": SCRIPT_TIMEOUT_MS});
        let mut attempt = 1;
        let mut moved = false;
        loop {
            let sent = SystemTime::now();
            let started = Instant::now();
            match self.client.call_tool_full("creator_exec", arguments.clone()) {
                Err(Error::Tool { retryable: true, .. }) if attempt < self.retry.attempts => {
                    thread::sleep(http::backoff(&self.retry, attempt, None));
                    attempt += 1;
                }
                // A document the engine can no longer save: run once more on a fresh one.
                Err(Error::Tool { message, .. }) if !moved && message.contains(UNSAVABLE) => {
                    let nonce = getrandom::u32().map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
                    document = format!("{document}-{nonce:08x}");
                    arguments["documentId"] = json!(document);
                    moved = true;
                }
                Err(error) => return Err(error),
                Ok(output) => {
                    let round_trip = started.elapsed();
                    let payload = output.payload;
                    return Ok(Run {
                        result: payload.get("result").cloned().unwrap_or(Value::Null),
                        revision: payload.get("revision").cloned().unwrap_or(Value::Null),
                        images: output.images,
                        document,
                        round_trip,
                        sent,
                    });
                }
            }
        }
    }

    /// A draft: the script renders its sheet and returns the attached images.
    pub fn draft(&self, code: &str, params: Value) -> Result<Run> {
        let run = self.run(code, params)?;
        if run.images.is_empty() {
            return Err(Error::InvalidExport(format!("the draft attached no images: {}", run.result)));
        }
        Ok(run)
    }

    /// Exports the bundle `params.file` is set to and splits it into `motions` (by name).
    /// `plan_sha256` and `script_sha256` go into each motion's provenance.
    pub fn export_bundle(&self, code: &str, mut params: Value, name: &str, motions: &[String], plan_sha256: &str) -> Result<Bundle> {
        let nonce = getrandom::u32().map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
        let file = format!("{name}-{}-{nonce:08x}.json", std::process::id());
        params["file"] = json!(file);
        let document = self.pool.acquire();
        let run = self.run_on(document.id(), code, params)?;
        let stored = run.result.pointer("/stored/0").cloned().unwrap_or(Value::Null);
        let ready = stored.get("status").and_then(Value::as_str) == Some("ready") && stored.get("file").and_then(Value::as_str) == Some(file.as_str());
        if !ready {
            return Err(Error::InvalidExport(format!("the bundle export returned {stored}")));
        }
        let started = Instant::now();
        // The script's document: the pool's, or the fresh one a run moved to.
        let fetched = self.fetch(&run.document, &file);
        // The engine expires files by itself; a failed delete must not fail the export.
        let _ = self.client.call_tool("creator_delete_file", json!({"file": file, "documentId": run.document}));
        let download = started.elapsed();
        let hosted = fetched?;
        if let Some(reported) = stored.get("bytes").and_then(Value::as_u64).filter(|&n| n != hosted.len() as u64) {
            return Err(Error::InvalidExport(format!("downloaded {} bytes of {file}, the engine reported {reported}", hosted.len())));
        }
        let hosted = String::from_utf8(hosted).map_err(|e| Error::InvalidExport(format!("{file}: {e}")))?;
        let server = self.client.server_info()?;
        let origin = json!({
            "engine": self.engine,
            "server": {"name": server.name, "version": server.version},
            "protocolVersion": server.protocol_version,
            "documentId": run.document,
            "documentRevision": run.revision,
        });
        let animations = animations_from_bundle(&hosted, motions, code, plan_sha256, origin)?;
        Ok(Bundle { animations, hosted_bytes: hosted.len(), run, download })
    }

    /// Downloads `file` through its signed URL, or `resources/read` when there is none or the
    /// URL fails.
    fn fetch(&self, document: &str, file: &str) -> Result<Vec<u8>> {
        let arguments = json!({"file": file, "documentId": document});
        let mut polls = 0;
        let located = loop {
            let located = self.client.call_tool("creator_download_file", arguments.clone())?;
            if located.get("status").and_then(Value::as_str) != Some("encoding") || polls == MAX_ENCODING_POLLS {
                break located;
            }
            let wait = located.get("retryAfterS").and_then(Value::as_f64).unwrap_or(1.0);
            thread::sleep(Duration::from_secs_f64(wait.clamp(0.1, 10.0)));
            polls += 1;
        };
        let uri = located.get("uri").and_then(Value::as_str);
        if let Some(url) = located.get("downloadUrl").and_then(Value::as_str) {
            match self.get(url) {
                Ok(bytes) => return Ok(bytes),
                Err(e) if uri.is_none() => return Err(e),
                Err(_) => {}
            }
        }
        match uri {
            Some(uri) => self.client.read_resource(uri),
            None => Err(Error::Protocol(format!("creator_download_file returned no link for {file}: {located}"))),
        }
    }

    fn get(&self, url: &str) -> Result<Vec<u8>> {
        let mut attempt = 1;
        loop {
            match self.download.get(url).send() {
                Ok(response) if response.status().is_success() => return Ok(response.bytes()?.to_vec()),
                Ok(response) if http::is_transient(response.status().as_u16()) && attempt < self.retry.attempts => {
                    thread::sleep(http::backoff(&self.retry, attempt, Some(response.headers())));
                }
                Ok(response) => return Err(http::status_error(response)),
                Err(e) if http::is_transient_error(&e) && attempt < self.retry.attempts => {
                    thread::sleep(http::backoff(&self.retry, attempt, None));
                }
                Err(e) => return Err(e.into()),
            }
            attempt += 1;
        }
    }
}

/// The motions of a bundle export, split and normalized, each with its provenance: `origin`
/// (who exported it) plus the script, plan, bundle and native hashes.
pub fn animations_from_bundle(hosted: &str, motions: &[String], code: &str, plan_sha256: &str, origin: Value) -> Result<Vec<BundleAnimation>> {
    let split = split_bundle(hosted)?;
    let mut animations = Vec::new();
    for motion in motions {
        let text = split.iter().find(|(n, _)| n == motion).map(|(_, t)| t).ok_or_else(|| Error::InvalidExport(format!("the bundle has no motion {motion}")))?;
        let native = normalize_native(text)?;
        let mut p = match origin.clone() {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        p.insert("exporter".into(), json!(concat!("gai-engine ", env!("CARGO_PKG_VERSION"))));
        p.insert("source".into(), json!("gai-icon/2"));
        p.insert("scriptSha256".into(), json!(sha256_hex(code.as_bytes())));
        p.insert("planSha256".into(), json!(plan_sha256));
        p.insert("bundleSha256".into(), json!(sha256_hex(hosted.as_bytes())));
        p.insert("bundleBytes".into(), json!(hosted.len()));
        p.insert("sha256".into(), json!(sha256_hex(native.text.as_bytes())));
        p.insert("bytes".into(), json!(native.text.len()));
        p.insert("fps".into(), json!(native.frame_rate));
        p.insert("duration".into(), json!(native.duration));
        p.insert("layers".into(), json!(native.layers));
        p.insert("completedAt".into(), json!(humantime::format_rfc3339_millis(SystemTime::now()).to_string()));
        animations.push(BundleAnimation { name: motion.clone(), native, provenance: Value::Object(p) });
    }
    Ok(animations)
}

/// Splits a bundle export (one precomp layer per motion, named after it) into one Lottie per
/// motion: the bundle's header (`v`, `fr`, `ip`, `op`, `w`, `h`), the motion's name, its
/// asset's layers and no assets. Returned in bundle layer order.
pub fn split_bundle(hosted: &str) -> Result<Vec<(String, String)>> {
    let root: Value = serde_json::from_str(hosted).map_err(|e| Error::InvalidExport(format!("bundle is not JSON: {e}")))?;
    let assets: Vec<&Value> = root.get("assets").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
    let layers = root.get("layers").and_then(Value::as_array).ok_or_else(|| Error::InvalidExport("bundle has no layers".into()))?;
    let mut out = Vec::new();
    for layer in layers {
        if layer.get("ty").and_then(Value::as_u64) != Some(0) {
            return Err(Error::InvalidExport(format!("bundle layer is not a precomp: {}", layer.get("nm").unwrap_or(&Value::Null))));
        }
        let name = layer.get("nm").and_then(Value::as_str).ok_or_else(|| Error::InvalidExport("unnamed bundle layer".into()))?;
        let reference = layer.get("refId").and_then(Value::as_str).unwrap_or_default();
        let asset = assets
            .iter()
            .find(|a| a.get("id").and_then(Value::as_str) == Some(reference))
            .ok_or_else(|| Error::InvalidExport(format!("bundle layer {name} names a missing asset {reference}")))?;
        let mut animation = Map::new();
        for key in ["v", "fr", "ip", "op", "w", "h"] {
            if let Some(v) = root.get(key) {
                animation.insert(key.into(), v.clone());
            }
        }
        animation.insert("nm".into(), json!(name));
        animation.insert("layers".into(), asset.get("layers").cloned().unwrap_or(json!([])));
        animation.insert("assets".into(), json!([]));
        out.push((name.to_owned(), serde_json::to_string(&Value::Object(animation)).map_err(|e| Error::InvalidExport(e.to_string()))?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_bundle_by_precomp() {
        let bundle = r#"{"v":"5.7.0","fr":60.0,"ip":0.0,"op":144.0,"w":32,"h":32,"nm":"x@bundle","layers":[
            {"ty":0,"nm":"x-strong","refId":"comp_0","ind":1},{"ty":0,"nm":"x","refId":"comp_1","ind":2}],
            "assets":[{"id":"comp_0","layers":[{"ty":4,"nm":"b","ind":1,"shapes":[]}]},{"id":"comp_1","layers":[{"ty":4,"nm":"a","ind":1,"shapes":[]}]}]}"#;
        let split = split_bundle(bundle).unwrap();
        assert_eq!(split.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["x-strong", "x"]);
        let x: Value = serde_json::from_str(&split[1].1).unwrap();
        assert_eq!(x["nm"], "x");
        assert_eq!(x["layers"][0]["nm"], "a");
        assert_eq!(x["assets"], json!([]));
        assert_eq!(normalize_native(&split[1].1).unwrap().layers, 1);
    }

    #[test]
    fn refuses_a_non_precomp_layer() {
        let bundle = r#"{"fr":60,"ip":0,"op":144,"layers":[{"ty":4,"nm":"x"}],"assets":[]}"#;
        assert!(split_bundle(bundle).is_err());
    }
}
