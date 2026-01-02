//! MYTHOS Dataset (RFC-0003)
//!
//! DatasetDef ID computation with field exclusion

use mythos_can::Value;
use sha2::{Digest, Sha256};

/// Compute CID from canonical bytes
pub fn cid_from_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

/// Compute DatasetDef ID from canonical bytes excluding field 1
///
/// Similar to receipt_id: dataset_def_id = SHA-256(canonical_bytes(def_without_field_1))
pub fn compute_dataset_def_id(def_map: &Value) -> Result<[u8; 32], String> {
    let fields = match def_map {
        Value::Map(pairs) => pairs,
        _ => return Err("DatasetDef must be MAP".into()),
    };

    // Verify field 1 exists (drift detection)
    let has_id_field = fields
        .iter()
        .any(|(k, _)| matches!(k, Value::UVarint(1) | Value::IVarint(1)));

    if !has_id_field {
        return Err("DatasetDef missing field 1 (dataset_def_id)".into());
    }

    // Exclude field 1 (dataset_def_id itself)
    // Accept both UVarint and IVarint keys (defensive)
    let fields_without_id: Vec<_> = fields
        .iter()
        .filter(|(k, _)| !matches!(k, Value::UVarint(1) | Value::IVarint(1)))
        .cloned()
        .collect();

    let def_without_field_1 = Value::Map(fields_without_id);

    let canonical_bytes = mythos_can::encode_value(&def_without_field_1)
        .map_err(|e| format!("Encoding failed: {}", e))?;

    Ok(cid_from_bytes(&canonical_bytes))
}
