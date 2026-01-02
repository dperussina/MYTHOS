/// Wire Suite Verification
use crate::manifest::VectorEntry;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn verify_wire_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    let expected_sha = entry
        .expected
        .get("packet_sha256")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing expected packet_sha256"))?;

    let packet_file = entry
        .files
        .get("packet_bin")
        .ok_or_else(|| anyhow::anyhow!("Missing packet_bin file"))?;

    let packet_path = pack_dir.join(packet_file);
    let packet_bytes = fs::read(&packet_path)?;

    let computed = mythos_wire::packet_sha256(&packet_bytes);

    if hex::encode(computed) != expected_sha {
        bail!(
            "Packet SHA-256 mismatch:\n  Expected: {}\n  Computed: {}",
            expected_sha,
            hex::encode(computed)
        );
    }

    Ok(())
}
