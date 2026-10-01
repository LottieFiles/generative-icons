//! The hand-off between the builder and whatever runs the engine.

/// A scene exported to Lottie by the engine.
#[derive(Debug, Clone)]
pub struct NativeExport {
    /// The normalized native Lottie text written to `native/<name>.json`: one layer per
    /// scene node with its shapes flat (`[sh, fl|st]`), keys sorted recursively, compact,
    /// numbers in the engine's (serde) notation. `nativeSha256` hashes exactly these bytes.
    pub text: String,
    /// Provenance written to `native/<name>.json.provenance.json` (pretty, 2-space).
    pub provenance: serde_json::Value,
}

/// Turns one scene (the `scene.json` text the builder wrote) into native Lottie.
pub trait Exporter: Send + Sync {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport>;
}
