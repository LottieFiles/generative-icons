//! Where released icons come from.
//!
//! Resolution order: `--library <dir|url>` (or `GEN_ANIMATED_ICONS_LIBRARY`), then the
//! nearest folder above the project that is a library checkout or holds an installed
//! `@lottiefiles/generative-icons` package (a checkout wins at the same level, since
//! it links its own packages), then the published library over HTTP.

mod catalog;
mod http;
mod local;
pub mod search_index;

pub use catalog::{Catalog, LibraryIcon, is_icon_id};
pub use http::{DEFAULT_LIBRARY_URL, Http};
pub use local::{LocalDir, is_checkout};
pub use search_index::SearchIndex;

use crate::error::{CliError, ErrorCode, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A source of released icons.
pub trait LibrarySource {
    /// Folder or URL, for reports.
    fn location(&self) -> String;

    /// The descriptor catalog.
    fn catalog(&self) -> Result<Catalog>;

    /// Search text the descriptors lack (briefs); empty when unavailable.
    fn search_index(&self, catalog: &Catalog) -> Result<SearchIndex>;

    /// The icon's `.lottie` bytes, verified against the catalog sha256.
    fn asset(&self, icon: &LibraryIcon) -> Result<Vec<u8>>;

    /// The library checkout behind this source, whose set sources can be copied.
    fn checkout(&self) -> Option<&Path> {
        None
    }

    /// Icons reanimated as specs: a package's `specs/<id>.icon.json` (hashed in its
    /// `release.json`), a checkout's `motion/icon-sets/<set>/specs/`.
    fn spec_ids(&self) -> Result<BTreeSet<String>> {
        Ok(BTreeSet::new())
    }

    /// An icon's spec, verified against `release.json`; `None` when it has none.
    fn spec(&self, _id: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

/// The package file of an icon's spec.
pub fn spec_path(id: &str) -> String {
    format!("specs/{id}{}", gai_spec::SUFFIX)
}

/// Spec ids and hashes from a package's `release.json` `files`.
fn release_specs(release: &[u8], location: &str) -> Result<BTreeMap<String, String>> {
    let value: serde_json::Value = serde_json::from_slice(release)
        .map_err(|error| CliError::new(ErrorCode::InvalidCatalog, format!("Invalid release.json at {location}: {error}")))?;
    Ok(value
        .get("files")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(path, sha)| {
            let id = path.strip_prefix("specs/")?.strip_suffix(gai_spec::SUFFIX)?;
            Some((id.to_string(), sha.as_str()?.to_string()))
        })
        .collect())
}

/// `bytes` when their sha256 is `sha256`, else `HASH_MISMATCH`.
fn verify_spec(id: &str, sha256: &str, bytes: Vec<u8>) -> Result<Vec<u8>> {
    if gai_core::sha256_hex(&bytes) == sha256 {
        Ok(bytes)
    } else {
        Err(CliError::new(ErrorCode::HashMismatch, format!("Package spec hash mismatch: {id}")).with_id(id).with_path(spec_path(id)))
    }
}

/// Specs of a library checkout: id → (set folder, spec file).
pub fn checkout_specs(root: &Path) -> BTreeMap<String, (String, PathBuf)> {
    let mut out = BTreeMap::new();
    let Ok(sets) = std::fs::read_dir(root.join("motion/icon-sets")) else { return out };
    for set in sets.flatten() {
        let Ok(specs) = std::fs::read_dir(set.path().join(gai_check::verify::adopted::SPECS_DIR)) else { continue };
        for spec in specs.flatten() {
            let name = spec.file_name().to_string_lossy().into_owned();
            if let Some(id) = name.strip_suffix(gai_spec::SUFFIX) {
                out.insert(id.to_string(), (set.file_name().to_string_lossy().into_owned(), spec.path()));
            }
        }
    }
    out
}

/// Returns `bytes` when their sha256 matches the catalog, else `HASH_MISMATCH`.
pub fn verify_asset(icon: &LibraryIcon, bytes: Vec<u8>) -> Result<Vec<u8>> {
    if gai_core::sha256_hex(&bytes) == icon.asset_sha256 {
        Ok(bytes)
    } else {
        Err(CliError::new(ErrorCode::HashMismatch, format!("Package asset hash mismatch: {}", icon.id))
            .with_id(&icon.id)
            .with_path(&icon.asset_path))
    }
}

/// Download cache: `GEN_ANIMATED_ICONS_CACHE_DIR`, else the user cache folder.
pub fn cache_dir() -> Option<PathBuf> {
    match std::env::var_os("GEN_ANIMATED_ICONS_CACHE_DIR") {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => dirs::cache_dir().map(|dir| dir.join("generative-icons")),
    }
}

/// Fails unless `url` is https, or http to a loopback host (`localhost`, `127.0.0.0/8`,
/// `[::1]`). `what` names the endpoint in the error. The URL is parsed as the HTTP clients
/// parse it (WHATWG), so the decision is about the host they connect to: `\\` is `/`
/// there, and userinfo is not a host.
pub fn require_secure_url(url: &str, what: &str) -> Result<()> {
    let secure = url::Url::parse(url).is_ok_and(|parsed| match (parsed.scheme(), parsed.host()) {
        ("https", Some(_)) => true,
        ("http", Some(url::Host::Domain(host))) => host.eq_ignore_ascii_case("localhost"),
        ("http", Some(url::Host::Ipv4(ip))) => ip.is_loopback(),
        ("http", Some(url::Host::Ipv6(ip))) => ip.is_loopback(),
        _ => false,
    });
    if secure {
        return Ok(());
    }
    Err(CliError::usage(format!("The {what} URL must use https: {url}"))
        .with_hint("Use an https:// address (plain http is allowed only for localhost)"))
}

/// Folder of the npm package's `dist` below a `node_modules`.
const PACKAGE_DIST: &str = "node_modules/@lottiefiles/generative-icons/dist";

/// Picks the library for a command run in `cwd`. A URL must be https, or http on a
/// loopback host: the catalog that pins every asset's sha256 comes over the same channel.
pub fn resolve(explicit: Option<&str>, cwd: &Path, offline: bool) -> Result<Box<dyn LibrarySource>> {
    if let Some(location) = explicit.filter(|location| !location.is_empty()) {
        if location.starts_with("http://") || location.starts_with("https://") {
            require_secure_url(location, "library")?;
            return Ok(Box::new(Http::new(location, cache_dir(), offline)));
        }
        return Ok(Box::new(LocalDir::open(&cwd.join(location))?));
    }
    for dir in cwd.ancestors() {
        if is_checkout(dir) {
            return Ok(Box::new(LocalDir::open(dir)?));
        }
        let dist = dir.join(PACKAGE_DIST);
        if dist.join("catalog.json").is_file() {
            return Ok(Box::new(LocalDir::open(&dist)?));
        }
    }
    Ok(Box::new(Http::new(DEFAULT_LIBRARY_URL, cache_dir(), offline)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_urls_must_be_https_unless_local() {
        for good in ["https://example.com/lib/", "http://127.0.0.1:8080/x", "http://localhost/", "http://[::1]:9/"] {
            require_secure_url(good, "library").unwrap();
        }
        for bad in [
            "http://example.com/",
            "http://127.0.0.1.evil.test/",
            "http://user@example.com/",
            "ftp://x",
            // The clients read `\\` as `/`, so the host is the address before it.
            "http://192.168.45.216:18902\\@localhost/lib",
            "http://evil\\@127.0.0.1/",
            "HTTP://evil.test\\@[::1]/",
            "http://localhost@evil.test/",
            "http://localhost.evil.test/",
            "http://0x7f000001.evil/",
            "not a url",
        ] {
            assert_eq!(require_secure_url(bad, "library").unwrap_err().code, ErrorCode::Usage, "{bad}");
        }
        // Percent-encoded backslashes stay in the userinfo; the host is still loopback.
        for good in ["http://evil%5C@localhost/", "HTTP://LOCALHOST/", "http://127.1.2.3/", "http://0x7f000001/"] {
            require_secure_url(good, "library").unwrap();
        }
        let error = resolve(Some("http://example.com/library/"), Path::new("."), false).err().unwrap();
        assert_eq!(error.message, "The library URL must use https: http://example.com/library/");
    }
}
