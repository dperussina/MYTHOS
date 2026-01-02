/// Merkle Suite Verification
use crate::manifest::VectorEntry;
use crate::verify::utils::compare_bytes;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn verify_merkle_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // 1. Get expected root_cid from manifest
    let expected_cid = entry
        .expected
        .get("root_cid")
        .and_then(|v| v.as_str())
        .context("Missing expected root_cid")?;

    // 2. Load leaf binary
    let leaf_bin_file = entry
        .files
        .get("leaf_bin")
        .context("Missing leaf_bin file")?;

    let leaf_bin_path = pack_dir.join(leaf_bin_file);
    let leaf_bytes = fs::read(&leaf_bin_path)
        .with_context(|| format!("Failed to read {}", leaf_bin_path.display()))?;

    // 3. Compute CID = SHA-256(canonical bytes)
    let computed_cid = mythos_merkle::cid_from_bytes(&leaf_bytes);

    if hex::encode(computed_cid) != expected_cid {
        bail!(
            "Root CID mismatch:\n  Expected: {}\n  Computed: {}",
            expected_cid,
            hex::encode(computed_cid)
        );
    }

    // 4. Validate structure (decode for validation, not for CID)
    let decoded =
        mythos_can::decode_value_exact(&leaf_bytes).context("Failed to decode leaf binary")?;

    let node = mythos_merkle::parse_merkle_node(&decoded).context("Failed to parse MerkleNode")?;

    mythos_merkle::validate_merkle_list_leaf(&node.payload)
        .context("Failed to validate MerkleListLeaf")?;

    // 5. Verify canonical encoding roundtrip
    let re_encoded = mythos_can::encode_value(&decoded).context("Failed to re-encode")?;

    compare_bytes(&leaf_bytes, &re_encoded).context("Re-encoded bytes don't match original")?;

    Ok(())
}
