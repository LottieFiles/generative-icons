//! The authoring kit a project keeps in `gen-animated-icons/library/`: the files
//! [`gai_js::kit`] embeds, one hash naming them all, and the check that the copy on
//! disk is exactly what the build will evaluate.

use crate::error::{CliError, ErrorCode, Result};
use crate::managed::{inside, read_optional};
use gai_core::sha256_hex;
use gai_js::kit::KitFile;
use std::path::Path;

/// The kit files, in bytewise name order.
pub fn files() -> &'static [KitFile] {
    gai_js::kit::files()
}

/// The embedded family contract (`family-contract.json`).
pub fn family_contract() -> &'static str {
    files()
        .iter()
        .find(|file| file.name == "family-contract.json")
        .map(|file| file.text)
        .expect("gai-js embeds the contract")
}

/// `kitSha256`, the kit version a lock record or submission names: sha256 over one
/// `"<name>\0<sha256>\n"` line per kit file (lowercase hex digest of its bytes), in
/// [`files`] order.
pub fn sha256() -> String {
    let listing: String = files().iter().map(|file| format!("{}\0{}\n", file.name, file.sha256)).collect();
    sha256_hex(listing)
}

/// Fails unless every kit file under `kit_dir` (project-relative) equals the embedded
/// bytes: an edited copy is `MODIFIED_FILE`, a missing one `PROJECT_NOT_FOUND`.
pub fn verify(root: &Path, kit_dir: &str) -> Result<()> {
    for file in files() {
        let path = format!("{kit_dir}/{}", file.name);
        match read_optional(&inside(root, &path)?)? {
            Some(bytes) if sha256_hex(&bytes) == file.sha256 => {}
            Some(_) => {
                return Err(CliError::new(
                    ErrorCode::ModifiedFile,
                    format!("The authoring kit copy {path} was edited; builds evaluate the kit exactly as shipped"),
                )
                .with_path(path)
                .with_hint(
                    "Move your changes into your own batch files, then run gen-animated-icons init --update-kit",
                ));
            }
            None => {
                return Err(CliError::new(ErrorCode::ProjectNotFound, format!("The authoring kit is missing: {path}"))
                    .with_path(path)
                    .with_hint("Run gen-animated-icons init"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_names_the_edited_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = verify(dir.path(), "kit").unwrap_err();
        assert_eq!(missing.code, ErrorCode::ProjectNotFound);
        std::fs::create_dir(dir.path().join("kit")).unwrap();
        for file in files() {
            std::fs::write(dir.path().join("kit").join(file.name), file.text).unwrap();
        }
        verify(dir.path(), "kit").unwrap();
        std::fs::write(dir.path().join("kit/drawing.mjs"), "export {};\n").unwrap();
        let edited = verify(dir.path(), "kit").unwrap_err();
        assert_eq!((edited.code, edited.path.as_deref()), (ErrorCode::ModifiedFile, Some("kit/drawing.mjs")));
    }

    #[test]
    fn kit_hash_lists_every_file() {
        let listing: Vec<&str> = files().iter().map(|f| f.name).collect();
        assert_eq!(listing, ["ai-authoring.mjs", "direction.mjs", "drawing.mjs", "family-contract.json"]);
        let mut lines = Vec::new();
        for file in files() {
            lines.extend_from_slice(file.name.as_bytes());
            lines.push(0);
            lines.extend_from_slice(sha256_hex(file.text).as_bytes());
            lines.push(b'\n');
        }
        assert_eq!(sha256(), sha256_hex(&lines));
    }
}
