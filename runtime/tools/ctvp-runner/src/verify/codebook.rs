/// Codebook Suite Verification
use crate::manifest::VectorEntry;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn verify_codebook_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    let expected_id = entry
        .expected
        .get("codebook_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing expected codebook_id"))?;

    let entries_file = entry
        .files
        .get("entries_bin")
        .ok_or_else(|| anyhow::anyhow!("Missing entries_bin file"))?;

    let entries_path = pack_dir.join(entries_file);
    let entries_bytes = fs::read(&entries_path)?;

    let computed_id = mythos_codebook::codebook_id_from_bytes(&entries_bytes);

    if hex::encode(computed_id) != expected_id {
        bail!(
            "Codebook ID mismatch:\n  Expected: {}\n  Computed: {}",
            expected_id,
            hex::encode(computed_id)
        );
    }

    Ok(())
}
