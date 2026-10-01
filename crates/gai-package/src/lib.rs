//! Packaging: dotLottie v2 archives, set zips, `icon-library.zip`, the asset audit and the
//! npm package contents. Ports `motion/icon-sets/library/package-set.py`, the per-set
//! `package.py` wrappers, `library/package.py`, `scripts/audit-assets.py`,
//! `scripts/build-packages.py` and `scripts/pack-release.mjs`.
//!
//! Outputs are byte-identical to the Python pipeline: archives use a hand-written ZIP writer
//! over classic C zlib ([`zip`]) with fixed dates, and JSON is written in Python
//! `json.dumps` format where the originals used it.
//!
//! Not ported: `library/package-ai.py` (`ai-icons.zip`) and
//! `library/director-revision/merge-render-evidence.py`.

#![warn(missing_docs)]

mod bundle;
mod config;
mod dotlottie;
mod error;
mod io;
mod library;
mod npm;
mod paths;
mod profile;
mod record;
mod release;
mod set_package;
pub mod zip;

pub use config::{SetConfig, SetPaths};
pub use dotlottie::{Archive, THEMES, build_icon_archives, final_manifest};
pub use error::{PackageError, Result};
pub use io::{py_compact, py_pretty};
pub use library::{AssetAudit, Catalog, Category, DeliveryProof, audit_assets, package_library};
pub use npm::{
    BuildPackagesOptions, DescriptorSource, PACKAGES, PackagesReport, build_packages,
    icon_descriptor, validate_descriptor,
};
pub use record::{ActionList, build_set_archives, encode_set_json, record_actions};
pub use release::{
    GitState, NpmPack, PackReleaseOptions, PackReleaseReport, PackedPackage, npm_pack, pack_release,
};
pub use set_package::{ArchiveSummary, PackageReport, package_set};
