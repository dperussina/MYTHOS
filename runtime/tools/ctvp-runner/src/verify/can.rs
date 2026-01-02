/// CAN Suite Verification
///
/// Verifies MYTHOS-CAN encoding/decoding conformance
use crate::manifest::VectorEntry;
use crate::verify::utils::{compare_bytes, verify_sha256};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn verify_can_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // 1. Resolve bin path
    let bin_path = entry
        .bin_path(pack_dir)
        .ok_or_else(|| anyhow::anyhow!("No bin file specified for {}", entry.id))?;

    if !bin_path.exists() {
        bail!("Bin file not found: {:?}", bin_path);
    }

    // 2. Load bytes
    let bin_bytes =
        fs::read(&bin_path).with_context(|| format!("Failed to read bin file: {:?}", bin_path))?;

    // 3. Verify SHA256 (sibling file)
    let sha_path = bin_path.with_extension("bin.sha256");
    if sha_path.exists() {
        let expected_sha = fs::read_to_string(&sha_path)?;
        if !verify_sha256(&bin_bytes, &expected_sha) {
            bail!("SHA256 mismatch (sibling file)");
        }
    }

    // 4. Verify SHA256 (manifest expected)
    if let Some(expected_sha) = entry.expected_sha256() {
        if !verify_sha256(&bin_bytes, &expected_sha) {
            bail!("SHA256 mismatch (manifest expected)");
        }
    }

    // 5. Strict decode (enforce canonical rules)
    let decoded = mythos_can::decode_value_exact(&bin_bytes)
        .with_context(|| "Failed to decode bin file (strict mode)")?;

    // 6. Re-encode and verify byte-identical
    let re_encoded =
        mythos_can::encode_value(&decoded).with_context(|| "Failed to re-encode value")?;

    compare_bytes(&bin_bytes, &re_encoded)
        .with_context(|| "Re-encoded bytes don't match original")?;

    // 7. Optional: JSON validation (future work)
    // For MVP, we trust decode-encode roundtrip

    Ok(())
}
