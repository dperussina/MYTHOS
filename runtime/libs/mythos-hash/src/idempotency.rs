/// IdempotencyID Computation (RFC-MYTHOS-0005)
///
/// IdempotencyID = SHA-256(tool_id_bytes || idempotency_key)
///
/// Where:
/// - tool_id_bytes: 32-byte SHA-256 digest from ToolID Hash struct
/// - idempotency_key: Raw bytes provided by caller
/// - || denotes simple byte concatenation (no additional encoding)
use crate::hash::sha256;

/// Compute IdempotencyID
///
/// # Arguments
/// - `tool_id_bytes`: 32-byte SHA-256 digest from ToolID (NOT the full Hash struct, just the digest)
/// - `idempotency_key`: Raw idempotency key bytes
///
/// # Returns
/// 32-byte SHA-256 hash
///
/// # Example
/// ```
/// use mythos_hash::compute_idempotency_id;
///
/// let tool_id = [0xAA; 32];
/// let idem_key = b"idem:001";
/// let idem_id = compute_idempotency_id(&tool_id, idem_key);
/// ```
pub fn compute_idempotency_id(tool_id_bytes: &[u8; 32], idempotency_key: &[u8]) -> [u8; 32] {
    // Simple concatenation: tool_id || idempotency_key
    let mut data = Vec::with_capacity(32 + idempotency_key.len());
    data.extend_from_slice(tool_id_bytes);
    data.extend_from_slice(idempotency_key);

    sha256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_idempotency_id() {
        // Test from LEDGER_001
        let tool_id =
            hex::decode("d5762d1026d1cfab5015b4821a0aa1f8d3ae1dea85084b3122ea63a7a4244458")
                .unwrap();
        let tool_id_bytes: [u8; 32] = tool_id.try_into().unwrap();

        let idempotency_key = hex::decode("6964656d3a303031").unwrap(); // "idem:001" ASCII

        let idem_id = compute_idempotency_id(&tool_id_bytes, &idempotency_key);

        // Expected from LEDGER_001
        let expected = "7f24e6dcd855c1cec0f714e71e9721ecb75055274361f658b3813eceff0ae6d3";
        assert_eq!(
            hex::encode(idem_id),
            expected,
            "IdempotencyID computation mismatch"
        );
    }

    #[test]
    fn test_idempotency_id_simple() {
        // Simple test with known values
        let tool_id = [0x00; 32];
        let idem_key = b"test";

        let idem_id = compute_idempotency_id(&tool_id, idem_key);

        // Verify it's deterministic
        let idem_id2 = compute_idempotency_id(&tool_id, idem_key);
        assert_eq!(idem_id, idem_id2);

        // Verify it's different with different key
        let idem_id3 = compute_idempotency_id(&tool_id, b"other");
        assert_ne!(idem_id, idem_id3);
    }
}
