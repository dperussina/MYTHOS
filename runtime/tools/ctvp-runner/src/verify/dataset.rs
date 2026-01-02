/// Dataset Suite Verification
use crate::manifest::VectorEntry;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn verify_dataset_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // Verify dataset_def_id (with field exclusion like receipt_id)
    if let Some(expected_defid) = entry
        .expected
        .get("dataset_def_id")
        .and_then(|v| v.as_str())
    {
        if let Some(def_bin_file) = entry.files.get("dataset_def_bin") {
            let def_bin_path = pack_dir.join(def_bin_file);
            let def_bytes = fs::read(&def_bin_path)?;

            // Decode and compute ID with field exclusion
            let decoded = mythos_can::decode_value_exact(&def_bytes)?;
            let computed = mythos_dataset::compute_dataset_def_id(&decoded)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            if hex::encode(computed) != expected_defid {
                bail!(
                    "DatasetDef ID mismatch:\n  Expected: {}\n  Computed: {}",
                    expected_defid,
                    hex::encode(computed)
                );
            }
        }
    }

    // Verify corpus root CID
    if let Some(expected_cid) = entry
        .expected
        .get("corpus_root_cid")
        .and_then(|v| v.as_str())
    {
        if let Some(corpus_node_file) = entry.files.get("corpus_rootnode_bin") {
            let corpus_path = pack_dir.join(corpus_node_file);
            let corpus_bytes = fs::read(&corpus_path)?;
            let computed = mythos_dataset::cid_from_bytes(&corpus_bytes);

            if hex::encode(computed) != expected_cid {
                bail!("Corpus root CID mismatch");
            }
        }
    }

    // Verify manifest root CID
    if let Some(expected_cid) = entry
        .expected
        .get("manifest_root_cid")
        .and_then(|v| v.as_str())
    {
        if let Some(manifest_node_file) = entry.files.get("manifest_rootnode_bin") {
            let manifest_path = pack_dir.join(manifest_node_file);
            let manifest_bytes = fs::read(&manifest_path)?;
            let computed = mythos_dataset::cid_from_bytes(&manifest_bytes);

            if hex::encode(computed) != expected_cid {
                bail!("Manifest root CID mismatch");
            }
        }
    }

    Ok(())
}
