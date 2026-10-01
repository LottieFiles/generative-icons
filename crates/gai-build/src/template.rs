//! The `.engine.js` the old builders wrote next to every scene of sets 03–63.
//!
//! It is `motion/src/engine-scene.js` after the builders' five replacements
//! (`library/build-set.mjs:62-84`), a constant: every `scenes/*.engine.js` of those sets
//! and every `native/*.provenance.json` `scriptSha256` matches it. The hosted engine runs a
//! different, group-based script, so this file is a parity artifact only. Sets 01 and 02
//! are never rebuilt, so their two-replacement variant is not needed.

/// The script bytes (3,059 bytes).
pub const ENGINE_SCRIPT: &[u8] = include_bytes!("../assets/scene.engine.js");

/// sha256 of [`ENGINE_SCRIPT`], as recorded in provenance files.
pub const ENGINE_SCRIPT_SHA256: &str = "b082af2a96891ba2f0ec05d1e2dbb4ae0bceb6714b9c7b5dfde7bff059877d21";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_matches_its_recorded_hash() {
        assert_eq!(ENGINE_SCRIPT.len(), 3059);
        assert_eq!(gai_core::sha256_hex(ENGINE_SCRIPT), ENGINE_SCRIPT_SHA256);
    }
}
