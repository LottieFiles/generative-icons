//! A library on disk: a built package `dist` or a library checkout.

use super::{Catalog, LibraryIcon, LibrarySource, SearchIndex, verify_asset};
use crate::error::{CliError, ErrorCode, Result};
use crate::managed::{inside, read_optional};
use std::path::{Path, PathBuf};

/// Checkout-relative folder of the built `@lottiefiles/generative-icons` package.
const CHECKOUT_DIST: &str = "packages/icons/dist";

/// A library read from a folder.
#[derive(Debug, Clone)]
pub struct LocalDir {
    dist: PathBuf,
    checkout: Option<PathBuf>,
}

impl LocalDir {
    /// Opens `path`: a `dist` folder holding `catalog.json`, that `catalog.json`
    /// itself, or a library checkout with built packages.
    pub fn open(path: &Path) -> Result<LocalDir> {
        let path = path.canonicalize().map_err(|error| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Cannot open library {}: {error}", path.display()))
        })?;
        let path = match path.parent() {
            Some(parent) if path.is_file() => parent.to_path_buf(),
            _ => path,
        };
        if path.join("catalog.json").is_file() {
            // A checkout's own built dist still reads briefs from the checkout.
            let checkout = path
                .ancestors()
                .nth(3)
                .filter(|repo| is_checkout(repo) && repo.join(CHECKOUT_DIST) == path)
                .map(Path::to_path_buf);
            return Ok(LocalDir { dist: path, checkout });
        }
        if is_checkout(&path) {
            let dist = path.join(CHECKOUT_DIST);
            if !dist.join("catalog.json").is_file() {
                return Err(CliError::new(
                    ErrorCode::InvalidCatalog,
                    format!("The library checkout {} has no built packages", path.display()),
                )
                .with_hint("Build the packages in the checkout first: gen-animated-icons library packages"));
            }
            return Ok(LocalDir { dist, checkout: Some(path) });
        }
        Err(CliError::new(ErrorCode::InvalidCatalog, format!("No library catalog in {}", path.display()))
            .with_hint("Pass the package's dist folder or a library checkout"))
    }
}

/// A library checkout holds the catalog and the family contract.
pub fn is_checkout(dir: &Path) -> bool {
    dir.join("prototype/icons/catalog.json").is_file()
        && dir.join("motion/icon-sets/library/family-contract.json").is_file()
}

impl LibrarySource for LocalDir {
    fn location(&self) -> String {
        self.dist.display().to_string()
    }

    fn catalog(&self) -> Result<Catalog> {
        let file = self.dist.join("catalog.json");
        let bytes = std::fs::read(&file).map_err(|error| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Cannot read {}: {error}", file.display()))
        })?;
        Catalog::parse(&bytes, &file.display().to_string())
    }

    fn search_index(&self, _catalog: &Catalog) -> Result<SearchIndex> {
        if let Some(checkout) = &self.checkout {
            return SearchIndex::from_checkout(checkout);
        }
        let file = self.dist.join("search-index.json");
        match read_optional(&file)? {
            Some(bytes) => SearchIndex::parse(&bytes, &file.display().to_string()),
            None => Ok(SearchIndex::default()),
        }
    }

    fn asset(&self, icon: &LibraryIcon) -> Result<Vec<u8>> {
        let file = inside(&self.dist, &icon.asset_path)
            .map_err(|error| CliError::new(ErrorCode::InvalidCatalog, error.message).with_id(&icon.id))?;
        let bytes = std::fs::read(&file).map_err(|error| {
            CliError::new(ErrorCode::InvalidCatalog, format!("Cannot read {}: {error}", file.display()))
                .with_id(&icon.id)
        })?;
        verify_asset(icon, bytes)
    }

    fn checkout(&self) -> Option<&Path> {
        self.checkout.as_deref()
    }
}
