/// Blob Suite Verification
use crate::manifest::VectorEntry;
use crate::verify::utils::compare_bytes;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn verify_blob_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // 1. Get expected root_cid
    let expected_cid = entry
        .expected
        .get("root_cid")
        .and_then(|v| v.as_str())
        .context("Missing expected root_cid")?;

    // 2. Load rootnode binary
    let rootnode_file = entry
        .files
        .get("rootnode_bin")
        .context("Missing rootnode_bin file")?;

    let rootnode_path = pack_dir.join(rootnode_file);
    let rootnode_bytes = fs::read(&rootnode_path)
        .with_context(|| format!("Failed to read {}", rootnode_path.display()))?;

    // 3. Compute CID = SHA-256(canonical bytes)
    let computed_cid = mythos_blob::cid_from_bytes(&rootnode_bytes);

    if hex::encode(computed_cid) != expected_cid {
        bail!(
            "Root CID mismatch:\n  Expected: {}\n  Computed: {}",
            expected_cid,
            hex::encode(computed_cid)
        );
    }

    // 4. Validate structure
    let decoded =
        mythos_can::decode_value_exact(&rootnode_bytes).context("Failed to decode rootnode")?;

    let node = mythos_blob::parse_chunked_blob_node(&decoded)
        .context("Failed to parse ChunkedBlobNode")?;

    let leaf =
        mythos_blob::validate_chunk_leaf(&node.payload).context("Failed to validate ChunkLeaf")?;

    // 5. Validate chunk count matches expected
    if let Some(expected_count) = entry.expected.get("chunk_count").and_then(|v| v.as_u64()) {
        if leaf.chunks.len() as u64 != expected_count {
            bail!(
                "Chunk count mismatch: expected {}, got {}",
                expected_count,
                leaf.chunks.len()
            );
        }
    }

    // 6. Verify canonical encoding roundtrip
    let re_encoded = mythos_can::encode_value(&decoded).context("Failed to re-encode")?;

    compare_bytes(&rootnode_bytes, &re_encoded).context("Re-encoded bytes don't match original")?;

    // 7. Optional: Verify chunk hashes if payload.bin available
    if let Some(payload_file) = entry.files.get("payload_bin") {
        let payload_path = pack_dir.join(payload_file);
        if payload_path.exists() {
            let payload_data = fs::read(&payload_path)?;
            let computed_hashes =
                mythos_blob::compute_chunk_hashes(&payload_data, leaf.chunk_size as usize);

            if computed_hashes.len() != leaf.chunks.len() {
                bail!(
                    "Computed {} chunks, metadata has {}",
                    computed_hashes.len(),
                    leaf.chunks.len()
                );
            }

            for (i, (computed, chunk_desc)) in
                computed_hashes.iter().zip(leaf.chunks.iter()).enumerate()
            {
                if computed != chunk_desc.hash.as_slice() {
                    bail!(
                        "Chunk {} hash mismatch:\n  Expected: {}\n  Computed: {}",
                        i,
                        hex::encode(&chunk_desc.hash),
                        hex::encode(computed)
                    );
                }
            }
        }
    }

    Ok(())
}
