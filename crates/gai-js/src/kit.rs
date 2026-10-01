//! The authoring kit: the files a user project keeps in `gen-animated-icons/library/`, so its
//! set imports them (`../library/ai-authoring.mjs`) as library sets import
//! `motion/icon-sets/library/`. The closure is `ai-authoring.mjs → {drawing.mjs,
//! direction.mjs}`, `direction.mjs → drawing.mjs`, `drawing.mjs → svgpath` (embedded), plus
//! the numeric contract. A build evaluates the on-disk copies, so they must hash to these.

use std::sync::LazyLock;

/// One kit file.
#[derive(Debug)]
pub struct KitFile {
    /// Path relative to the kit folder.
    pub name: &'static str,
    /// The exact contents, embedded from `motion/icon-sets/library/`.
    pub text: &'static str,
    /// Lowercase hex sha256 of `text`.
    pub sha256: String,
}

macro_rules! library {
    ($($name:literal),+) => {
        [$(($name, include_str!(concat!("../../../motion/icon-sets/library/", $name)))),+]
    };
}

const SOURCES: [(&str, &str); 4] = library!("ai-authoring.mjs", "direction.mjs", "drawing.mjs", "family-contract.json");

static FILES: LazyLock<Vec<KitFile>> = LazyLock::new(|| {
    SOURCES.iter().map(|&(name, text)| KitFile { name, text, sha256: gai_core::sha256_hex(text) }).collect()
});

/// The kit files in bytewise name order.
pub fn files() -> &'static [KitFile] {
    &FILES
}
