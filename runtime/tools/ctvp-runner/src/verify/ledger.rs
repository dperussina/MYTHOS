/// Ledger Suite Verification
///
/// Verifies IdempotencyID computation against LEDGER_* test vectors
use crate::manifest::VectorEntry;
use crate::verify::utils::compare_bytes;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn verify_ledger_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // 1. Get expected IdempotencyID from manifest or from idemid.hex file
    let expected_idem_id = if let Some(id) = entry
        .expected
        .get("idempotency_id")
        .and_then(|v| v.as_str())
    {
        id.to_string()
    } else if let Some(idemid_file) = entry.files.get("idemid_hex") {
        let path = pack_dir.join(idemid_file);
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to read idemid hex file: {:?}", path))?
            .trim()
            .to_string()
    } else {
        bail!("No expected idempotency_id found for {}", entry.id);
    };

    // 2. For LEDGER_001, we have the inputs from the JSON
    // tool_id and idempotency_key are defined in the test vector
    // We need to compute IdempotencyID and verify it matches

    // Load the JSON to get tool_id and idempotency_key
    if let Some(json_file) = entry.files.get("json") {
        let json_path = pack_dir.join(json_file);
        if json_path.exists() {
            let json_str = fs::read_to_string(&json_path)?;
            let json: serde_json::Value = serde_json::from_str(&json_str)?;

            let tool_id_hex = json
                .get("tool_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing tool_id in JSON"))?;

            let idem_key_hex = json
                .get("idempotency_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing idempotency_key in JSON"))?;

            // Decode from hex
            let tool_id_bytes = hex::decode(tool_id_hex)?;
            let idem_key_bytes = hex::decode(idem_key_hex)?;

            if tool_id_bytes.len() != 32 {
                bail!("tool_id must be 32 bytes, got {}", tool_id_bytes.len());
            }

            let tool_id: [u8; 32] = tool_id_bytes.try_into().unwrap();

            // Compute IdempotencyID
            let computed_idem_id = mythos_hash::compute_idempotency_id(&tool_id, &idem_key_bytes);
            let computed_hex = hex::encode(computed_idem_id);

            // Compare with expected
            if computed_hex != expected_idem_id {
                bail!(
                    "IdempotencyID mismatch:\n  Expected: {}\n  Computed: {}",
                    expected_idem_id,
                    computed_hex
                );
            }
        }
    }

    // 3. Also verify register/commit bins if present (canonical encoding roundtrip)
    if let Some(register_file) = entry.files.get("register_bin") {
        let path = pack_dir.join(register_file);
        if path.exists() {
            let bin = fs::read(&path)?;
            let decoded = mythos_can::decode_value_exact(&bin)
                .with_context(|| "Failed to decode register bin")?;
            let re_encoded = mythos_can::encode_value(&decoded)?;
            compare_bytes(&bin, &re_encoded)?;
        }
    }

    if let Some(commit_file) = entry.files.get("commit_bin") {
        let path = pack_dir.join(commit_file);
        if path.exists() {
            let bin = fs::read(&path)?;
            let decoded = mythos_can::decode_value_exact(&bin)
                .with_context(|| "Failed to decode commit bin")?;
            let re_encoded = mythos_can::encode_value(&decoded)?;
            compare_bytes(&bin, &re_encoded)?;
        }
    }

    Ok(())
}
