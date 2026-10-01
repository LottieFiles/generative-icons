//! A library served over HTTP with the package `dist` layout: `catalog.json`,
//! optional `search-index.json`, and `assets/<id>.lottie`.
//!
//! Assets are verified against the catalog before use and cached by sha256. Each
//! fetched catalog is cached per base URL and version so `--offline` can reuse the
//! newest one. Downloads use the engine client's HTTP stack and the OS trust store.

use super::{Catalog, LibraryIcon, LibrarySource, SearchIndex, verify_asset};
use crate::error::{CliError, ErrorCode, Result};
use gai_engine::Downloader;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the released library is published.
pub const DEFAULT_LIBRARY_URL: &str = "https://gen-animated-icons.lottiefiles.com/library/v1/";

/// Largest response accepted (the 0.1.0 catalog is 2.1 MB).
const MAX_BODY: u64 = 64 << 20;

/// A library at a base URL.
#[derive(Debug)]
pub struct Http {
    base: String,
    /// The client, or why it could not be set up (reported on first use).
    downloader: std::result::Result<Downloader, String>,
    cache: Option<PathBuf>,
    offline: bool,
}

impl Http {
    /// A library at `base` (a trailing `/` is added). `cache` holds downloads;
    /// `offline` serves only from it.
    pub fn new(base: &str, cache: Option<PathBuf>, offline: bool) -> Http {
        let base = if base.ends_with('/') { base.to_string() } else { format!("{base}/") };
        let downloader = Downloader::new(Duration::from_secs(60)).map_err(|error| error.to_string());
        Http { base, downloader, cache, offline }
    }

    /// Fetches `base + relative`; `None` on 404.
    fn get(&self, relative: &str) -> Result<Option<Vec<u8>>> {
        let url = format!("{}{relative}", self.base);
        let network = |detail: String| {
            CliError::new(ErrorCode::Network, format!("Cannot fetch {url}: {detail}")).with_hint(
                "Check the connection, pass --library <dir> for a local copy, or use --offline with cached data",
            )
        };
        let downloader = self.downloader.as_ref().map_err(|detail| network(detail.clone()))?;
        downloader.get(&url, MAX_BODY).map_err(|error| network(with_causes(&error)))
    }

    fn offline_error(&self, what: &str) -> CliError {
        CliError::new(ErrorCode::Network, format!("Offline, and {what} is not cached"))
            .with_hint("Run once without --offline, or pass --library <dir>")
    }

    fn cached(&self, relative: &str) -> Option<PathBuf> {
        self.cache.as_ref().map(|cache| cache.join(relative))
    }

    /// This library's catalog cache folder; catalogs of different URLs never mix.
    fn catalog_cache(&self) -> Option<PathBuf> {
        self.cached(&format!("catalog/{}", &gai_core::sha256_hex(&self.base)[..16]))
    }
}

/// `error` and each cause it wraps, `: `-joined, skipping a cause whose text is already
/// shown: the client's own message never says why a request failed (DNS, refused, TLS).
fn with_causes(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(error) = cause {
        let message = error.to_string();
        if !text.contains(&message) {
            text.push_str(": ");
            text.push_str(&message);
        }
        cause = error.source();
    }
    text
}

/// Versions become file names, so only plain semver characters are cached.
fn cacheable_version(version: &str) -> bool {
    !version.is_empty() && version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
}

/// Writes a cache file atomically. Caching is best-effort: failures are ignored.
fn store(file: &Path, bytes: &[u8]) {
    let Some(dir) = file.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    if let Ok(mut temp) = tempfile::NamedTempFile::new_in(dir) {
        use std::io::Write;
        if temp.write_all(bytes).is_ok() {
            let _ = temp.persist(file);
        }
    }
}

/// The most recently written `<version>.json` in a catalog cache folder.
fn newest_cached_catalog(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".json") && !name.ends_with(".search-index.json")
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .max()
        .map(|(_, path)| path)
}

impl LibrarySource for Http {
    fn location(&self) -> String {
        self.base.clone()
    }

    fn catalog(&self) -> Result<Catalog> {
        if self.offline {
            let file = self
                .catalog_cache()
                .and_then(|dir| newest_cached_catalog(&dir))
                .ok_or_else(|| self.offline_error("the library catalog"))?;
            let bytes = std::fs::read(&file).map_err(|error| CliError::io("read", &file, &error))?;
            return Catalog::parse(&bytes, &file.display().to_string());
        }
        let bytes = self.get("catalog.json")?.ok_or_else(|| {
            CliError::new(ErrorCode::Network, format!("No library catalog at {}catalog.json", self.base))
                .with_hint("Pass --library <dir|url> to choose another library")
        })?;
        let catalog = Catalog::parse(&bytes, &format!("{}catalog.json", self.base))?;
        if cacheable_version(&catalog.version)
            && let Some(dir) = self.catalog_cache()
        {
            let file = dir.join(format!("{}.json", catalog.version));
            store(&file, &bytes);
        }
        Ok(catalog)
    }

    fn search_index(&self, catalog: &Catalog) -> Result<SearchIndex> {
        let cached = cacheable_version(&catalog.version)
            .then(|| self.catalog_cache())
            .flatten()
            .map(|dir| dir.join(format!("{}.search-index.json", catalog.version)));
        let bytes = if self.offline {
            cached.and_then(|file| std::fs::read(file).ok())
        } else {
            let bytes = self.get("search-index.json")?;
            if let (Some(file), Some(bytes)) = (&cached, &bytes) {
                store(file, bytes);
            }
            bytes
        };
        match bytes {
            Some(bytes) => SearchIndex::parse(&bytes, &format!("{}search-index.json", self.base)),
            None => Ok(SearchIndex::default()),
        }
    }

    fn asset(&self, icon: &LibraryIcon) -> Result<Vec<u8>> {
        let cached = self.cached(&format!("assets/{}.lottie", icon.asset_sha256));
        if let Some(bytes) = cached.as_ref().and_then(|file| std::fs::read(file).ok())
            && gai_core::sha256_hex(&bytes) == icon.asset_sha256
        {
            return Ok(bytes);
        }
        if self.offline {
            return Err(self.offline_error(&format!("icon {}", icon.id)).with_id(&icon.id));
        }
        let bytes = self.get(&icon.asset_path)?.ok_or_else(|| {
            CliError::new(ErrorCode::Network, format!("No asset at {}{}", self.base, icon.asset_path)).with_id(&icon.id)
        })?;
        let bytes = verify_asset(icon, bytes)?;
        if let Some(file) = cached {
            store(&file, &bytes);
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_errors_say_why_the_request_failed() {
        // Nothing listens on port 1, so the connection is refused.
        let library = Http::new("http://127.0.0.1:1/library/", None, false);
        let error = library.catalog().unwrap_err();
        assert_eq!(error.code, ErrorCode::Network);
        let message = error.message.to_lowercase();
        assert!(
            message.starts_with("cannot fetch http://127.0.0.1:1/library/catalog.json: network error: "),
            "{message}"
        );
        assert!(message.contains("refused"), "{message}");
    }

    #[test]
    fn repeated_causes_are_shown_once() {
        #[derive(Debug)]
        struct Wrap(&'static str, Option<Box<Wrap>>);
        impl std::fmt::Display for Wrap {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.0)
            }
        }
        impl std::error::Error for Wrap {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.1.as_deref().map(|e| e as _)
            }
        }
        let error = Wrap("send failed", Some(Box::new(Wrap("send failed", Some(Box::new(Wrap("dns error", None)))))));
        assert_eq!(with_causes(&error), "send failed: dns error");
    }
}
