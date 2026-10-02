//! Scene export on the hosted engine: [`HostedExporter`] implements
//! [`gai_core::export::Exporter`].

use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use gai_core::export::{Exporter, NativeExport};
use gai_core::sha256_hex;
use reqwest::blocking::Client;
use serde_json::{Map, Value, json};

use crate::config::{EngineConfig, RetryPolicy};
use crate::error::{Error, Result};
use crate::http;
use crate::mcp::McpClient;
use crate::normalize::normalize_native;

/// The script `creator_exec` runs, with params `{file, scene}`: one composition-level group per
/// scene node (rigid transform and visibility), holding a path (geometry and paint).
pub const EXPORT_SCRIPT: &str = include_str!("export-scene.js");
/// The engine's maximum script run time.
const SCRIPT_TIMEOUT_MS: u64 = 30_000;
/// Polls of `creator_download_file` while a file is still encoding (Lottie is ready at once).
const MAX_ENCODING_POLLS: u32 = 10;

/// Exports scenes through `creator_exec` on a small pool of engine documents.
///
/// Each export takes a document from the pool (so at most `documents` run at once and the
/// per-user document limit holds), runs [`EXPORT_SCRIPT`], downloads the file through its signed
/// URL (falling back to `resources/read`), deletes it on the engine, and normalizes it.
pub struct HostedExporter {
    client: McpClient,
    download: Client,
    engine: String,
    retry: RetryPolicy,
    pool: DocumentPool,
}

impl HostedExporter {
    /// An exporter for `config`; nothing is sent before the first export.
    pub fn new(config: &EngineConfig) -> Result<Self> {
        Ok(HostedExporter {
            client: McpClient::new(config)?,
            download: http::client()?,
            engine: config.url.to_string(),
            retry: config.retry,
            pool: DocumentPool::new(config.document_ids()),
        })
    }

    /// Exports one scene; `name` starts the engine file's name (see [`export_file_name`]).
    pub fn export_scene(&self, name: &str, scene_json: &str) -> Result<NativeExport> {
        let scene: Value = serde_json::from_str(scene_json).map_err(Error::InvalidScene)?;
        let file = export_file_name(name)?;
        let document = self.pool.acquire();
        let exec = self.run_script(document.id(), &file, &scene)?;
        let revision = exec.get("revision").cloned().unwrap_or(Value::Null);
        let reported = exec.pointer("/result/bytes").and_then(Value::as_u64);
        let fetched = self.fetch(document.id(), &file);
        // The engine expires files by itself; a failed delete must not fail the export.
        let _ = self.client.call_tool(
            "creator_delete_file",
            json!({"file": file, "documentId": document.id()}),
        );
        let hosted = fetched?;
        if let Some(reported) = reported.filter(|&n| n != hosted.len() as u64) {
            return Err(Error::InvalidExport(format!(
                "downloaded {} bytes of {file}, the engine reported {reported}",
                hosted.len()
            )));
        }
        let hosted =
            String::from_utf8(hosted).map_err(|e| Error::InvalidExport(format!("{file}: {e}")))?;
        let native = normalize_native(&hosted)?;
        let server = self.client.server_info()?;

        let mut provenance = Map::new();
        provenance.insert("engine".into(), json!(self.engine));
        provenance.insert(
            "server".into(),
            json!({"name": server.name, "version": server.version}),
        );
        provenance.insert("protocolVersion".into(), json!(server.protocol_version));
        provenance.insert(
            "exporter".into(),
            json!(concat!("gai-engine ", env!("CARGO_PKG_VERSION"))),
        );
        provenance.insert("documentId".into(), json!(document.id()));
        provenance.insert("documentRevision".into(), revision);
        provenance.insert(
            "scriptSha256".into(),
            json!(sha256_hex(EXPORT_SCRIPT.as_bytes())),
        );
        provenance.insert(
            "sceneSha256".into(),
            json!(sha256_hex(scene_json.as_bytes())),
        );
        provenance.insert("hostedSha256".into(), json!(sha256_hex(hosted.as_bytes())));
        provenance.insert("hostedBytes".into(), json!(hosted.len()));
        provenance.insert("sha256".into(), json!(sha256_hex(native.text.as_bytes())));
        provenance.insert("bytes".into(), json!(native.text.len()));
        provenance.insert("width".into(), native.width.clone());
        provenance.insert("height".into(), native.height.clone());
        provenance.insert("fps".into(), json!(native.frame_rate));
        provenance.insert("duration".into(), json!(native.duration));
        provenance.insert("layers".into(), json!(native.layers));
        provenance.insert(
            "completedAt".into(),
            json!(humantime::format_rfc3339_millis(SystemTime::now()).to_string()),
        );
        Ok(NativeExport {
            text: native.text,
            provenance: Value::Object(provenance),
        })
    }

    /// Runs the export script, retrying failures the engine marks retryable.
    fn run_script(&self, document: &str, file: &str, scene: &Value) -> Result<Value> {
        let arguments = json!({
            "code": EXPORT_SCRIPT,
            "documentId": document,
            "params": {"file": file, "scene": scene},
            "timeoutMs": SCRIPT_TIMEOUT_MS,
        });
        let mut attempt = 1;
        let exec = loop {
            match self.client.call_tool("creator_exec", arguments.clone()) {
                Err(Error::Tool {
                    retryable: true, ..
                }) if attempt < self.retry.attempts => {
                    thread::sleep(http::backoff(&self.retry, attempt, None));
                    attempt += 1;
                }
                other => break other?,
            }
        };
        let result = exec.get("result").unwrap_or(&Value::Null);
        let ready = result.get("status").and_then(Value::as_str) == Some("ready")
            && result.get("file").and_then(Value::as_str) == Some(file)
            && result.get("format").and_then(Value::as_str) == Some("lottie");
        if !ready {
            return Err(Error::InvalidExport(format!(
                "creator.export returned {result}"
            )));
        }
        Ok(exec)
    }

    /// Downloads `file` through its signed URL, or `resources/read` when there is none or the
    /// URL fails.
    fn fetch(&self, document: &str, file: &str) -> Result<Vec<u8>> {
        let arguments = json!({"file": file, "documentId": document});
        let mut polls = 0;
        let located = loop {
            let located = self
                .client
                .call_tool("creator_download_file", arguments.clone())?;
            if located.get("status").and_then(Value::as_str) != Some("encoding")
                || polls == MAX_ENCODING_POLLS
            {
                break located;
            }
            let wait = located
                .get("retryAfterS")
                .and_then(Value::as_f64)
                .unwrap_or(1.0);
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
            None => Err(Error::Protocol(format!(
                "creator_download_file returned no link for {file}: {located}"
            ))),
        }
    }

    /// GETs a signed URL (no credentials), retrying 429, 5xx and network failures.
    fn get(&self, url: &str) -> Result<Vec<u8>> {
        let mut attempt = 1;
        loop {
            match self.download.get(url).send() {
                Ok(response) if response.status().is_success() => {
                    return Ok(response.bytes()?.to_vec());
                }
                Ok(response)
                    if http::is_transient(response.status().as_u16())
                        && attempt < self.retry.attempts =>
                {
                    thread::sleep(http::backoff(
                        &self.retry,
                        attempt,
                        Some(response.headers()),
                    ));
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

/// `<name>-<pid>-<random>.json`: every process shares the fixed engine documents, so the name
/// must not collide with another process exporting the same animation. The engine names the
/// Lottie after the composition, not the file.
fn export_file_name(name: &str) -> Result<String> {
    let nonce = getrandom::u32().map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
    Ok(format!("{name}-{}-{nonce:08x}.json", std::process::id()))
}

impl Exporter for HostedExporter {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
        Ok(self.export_scene(name, scene_json)?)
    }
}

/// Fixed engine document ids handed out one export at a time; doubles as the concurrency limit.
pub(crate) struct DocumentPool {
    free: Mutex<Vec<String>>,
    returned: Condvar,
}

pub(crate) struct Lease<'a> {
    pool: &'a DocumentPool,
    id: String,
}

impl DocumentPool {
    pub(crate) fn new(mut ids: Vec<String>) -> Self {
        // Hand out the first id first.
        ids.reverse();
        DocumentPool {
            free: Mutex::new(ids),
            returned: Condvar::new(),
        }
    }

    /// Blocks until a document is free.
    pub(crate) fn acquire(&self) -> Lease<'_> {
        let mut free = self.free.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if let Some(id) = free.pop() {
                return Lease { pool: self, id };
            }
            free = self.returned.wait(free).unwrap_or_else(|e| e.into_inner());
        }
    }
}

impl Lease<'_> {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        let mut free = self.pool.free.lock().unwrap_or_else(|e| e.into_inner());
        free.push(std::mem::take(&mut self.id));
        self.pool.returned.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn pool_limits_concurrency_and_rotates_ids() {
        let pool = Arc::new(DocumentPool::new(vec!["d1".into(), "d2".into()]));
        assert_eq!(pool.acquire().id(), "d1");
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let workers: Vec<_> = (0..6)
            .map(|_| {
                let (pool, active, peak) =
                    (Arc::clone(&pool), Arc::clone(&active), Arc::clone(&peak));
                thread::spawn(move || {
                    let lease = pool.acquire();
                    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(20));
                    active.fetch_sub(1, Ordering::SeqCst);
                    lease.id().to_owned()
                })
            })
            .collect();
        for worker in workers {
            assert!(["d1", "d2"].contains(&worker.join().unwrap().as_str()));
        }
        assert_eq!(peak.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn export_file_names_are_unique_per_call_and_process() {
        let first = export_file_name("filter-strong").unwrap();
        let second = export_file_name("filter-strong").unwrap();
        assert_ne!(first, second);
        let prefix = format!("filter-strong-{}-", std::process::id());
        assert!(
            first.starts_with(&prefix) && first.ends_with(".json"),
            "{first}"
        );
        assert_eq!(first.len(), prefix.len() + 8 + ".json".len());
    }

    #[test]
    fn script_clears_with_remove_comp_and_exports_lottie() {
        assert!(EXPORT_SCRIPT.contains("creator.removeComp(c.id)"));
        assert!(!EXPORT_SCRIPT.contains("creator.deleteNodes"));
        assert!(
            EXPORT_SCRIPT
                .trim_end()
                .ends_with(r#"return creator.export(params.file, "lottie");"#)
        );
    }
}
