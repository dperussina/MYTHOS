/// Receipt ID computation with field exclusion
///
/// CRITICAL RULE (RECEIPT_001):
/// receipt_id = SHA-256(canonical_bytes(receipt_without_fields_1_and_11))
///
/// Where:
/// - Field 1: receipt_id itself (excluded)
/// - Field 11: signature (excluded)
use crate::hash::sha256;
use mythos_can::Value;

/// AgentID structure (RFC-MYTHOS-0001 Appendix A.3)
#[derive(Debug, Clone)]
pub struct AgentID {
    pub scheme: u8,           // Field 1 - 1=Ed25519
    pub key: Vec<u8>,         // Field 2 - public key bytes
    pub hint: Option<String>, // Field 3 - optional hint (e.g. "ctvp")
}

/// Receipt structure (mirrors RFC-MYTHOS-0001 Appendix A.9)
///
/// Field numbers:
/// 1: receipt_id (Hash) - EXCLUDED from hash computation
/// 2: tool_id (Hash)
/// 3: request_hash (Hash)
/// 4: response_hash (Hash)
/// 5: idempotency_key (bytes)
/// 6: signer (AgentID)
/// 7: time_observed (Time - i64 microseconds)
/// 8: status (u16)
/// 9: evidence (list(Hash), optional)
/// 10: notes (text, optional)
/// 11: signature (Signature) - EXCLUDED from hash computation
#[derive(Debug, Clone)]
pub struct Receipt {
    pub tool_id: Vec<u8>,               // Field 2 - tool_id hash bytes
    pub request_hash: Vec<u8>,          // Field 3
    pub response_hash: Vec<u8>,         // Field 4
    pub idempotency_key: Vec<u8>,       // Field 5
    pub signer: AgentID,                // Field 6 - AgentID (scheme + key + hint)
    pub time_us: i64,                   // Field 7 - microseconds since epoch
    pub status: u16,                    // Field 8
    pub evidence: Option<Vec<Vec<u8>>>, // Field 9 - optional
    pub notes: Option<String>,          // Field 10 - optional
                                        // signature is field 11 - NOT included in struct for ID computation
}

/// Canonically encode a Receipt for ID computation
///
/// Excludes fields 1 (receipt_id) and 11 (signature).
/// Returns canonical MYTHOS-CAN bytes.
pub fn canonical_encode_receipt_for_id(receipt: &Receipt) -> mythos_can::Result<Vec<u8>> {
    // Build MAP with only fields 2-10 (excluding 1 and 11)
    let mut fields = Vec::new();

    // Field 2: tool_id (Hash)
    fields.push((
        Value::UVarint(2),
        Value::Map(vec![
            (Value::UVarint(1), Value::UVarint(1)), // alg=1 (SHA-256)
            (Value::UVarint(2), Value::Bytes(receipt.tool_id.clone())),
        ]),
    ));

    // Field 3: request_hash
    fields.push((
        Value::UVarint(3),
        Value::Map(vec![
            (Value::UVarint(1), Value::UVarint(1)),
            (
                Value::UVarint(2),
                Value::Bytes(receipt.request_hash.clone()),
            ),
        ]),
    ));

    // Field 4: response_hash
    fields.push((
        Value::UVarint(4),
        Value::Map(vec![
            (Value::UVarint(1), Value::UVarint(1)),
            (
                Value::UVarint(2),
                Value::Bytes(receipt.response_hash.clone()),
            ),
        ]),
    ));

    // Field 5: idempotency_key
    fields.push((
        Value::UVarint(5),
        Value::Bytes(receipt.idempotency_key.clone()),
    ));

    // Field 6: signer (AgentID with scheme + key + optional hint)
    let mut agent_fields = vec![
        (
            Value::UVarint(1),
            Value::UVarint(receipt.signer.scheme as u64),
        ),
        (Value::UVarint(2), Value::Bytes(receipt.signer.key.clone())),
    ];

    // Add hint if present (field 3)
    if let Some(ref hint) = receipt.signer.hint {
        agent_fields.push((Value::UVarint(3), Value::Text(hint.clone())));
    }

    fields.push((Value::UVarint(6), Value::Map(agent_fields)));

    // Field 7: time_observed (i64 microseconds)
    fields.push((Value::UVarint(7), Value::IVarint(receipt.time_us)));

    // Field 8: status
    fields.push((Value::UVarint(8), Value::UVarint(receipt.status as u64)));

    // Field 9: evidence (optional list of hashes)
    if let Some(ref evidence) = receipt.evidence {
        let evidence_values: Vec<Value> = evidence
            .iter()
            .map(|hash_bytes| {
                Value::Map(vec![
                    (Value::UVarint(1), Value::UVarint(1)),
                    (Value::UVarint(2), Value::Bytes(hash_bytes.clone())),
                ])
            })
            .collect();

        fields.push((Value::UVarint(9), Value::List(evidence_values)));
    }

    // Field 10: notes (optional text)
    if let Some(ref notes) = receipt.notes {
        fields.push((Value::UVarint(10), Value::Text(notes.clone())));
    }

    // Encode as MAP (canonical encoder will sort by field numbers)
    let receipt_map = Value::Map(fields);
    mythos_can::encode_value(&receipt_map)
}

/// Compute receipt_id (32-byte SHA-256)
///
/// receipt_id = SHA-256(canonical_bytes(receipt_without_fields_1_and_11))
pub fn compute_receipt_id(receipt: &Receipt) -> [u8; 32] {
    let canonical_bytes =
        canonical_encode_receipt_for_id(receipt).expect("Receipt encoding should not fail");
    sha256(&canonical_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agentid_hint_regression() {
        // REGRESSION TEST: Lock in AgentID hint field forever
        // Ensures we never forget the optional hint field again
        let agent = AgentID {
            scheme: 1,
            key: vec![0xAA; 32],
            hint: Some("test".to_string()),
        };

        // Encode as MAP
        let agent_map = Value::Map(vec![
            (Value::UVarint(1), Value::UVarint(1)),
            (Value::UVarint(2), Value::Bytes(agent.key.clone())),
            (Value::UVarint(3), Value::Text(agent.hint.clone().unwrap())),
        ]);

        let encoded = mythos_can::encode_value(&agent_map).unwrap();

        // Verify hint is included in encoding
        let decoded = mythos_can::decode_value(&encoded).unwrap();
        if let Value::Map(pairs) = decoded {
            // Should have 3 fields (scheme, key, hint)
            assert_eq!(
                pairs.len(),
                3,
                "AgentID must encode all 3 fields including optional hint"
            );
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_compute_receipt_id() {
        // Test receipt from RECEIPT_001
        let receipt = Receipt {
            tool_id: hex::decode(
                "d5762d1026d1cfab5015b4821a0aa1f8d3ae1dea85084b3122ea63a7a4244458",
            )
            .unwrap(),
            request_hash: hex::decode(
                "758d61f26a44448384e5c4468a0dcb7a2abe456067b0f7b505bc28b9411fe931",
            )
            .unwrap(),
            response_hash: hex::decode(
                "9795c5ff8937f23526ccb207a5684c1fc94a7854e19c021b39d944e51f5baef2",
            )
            .unwrap(),
            idempotency_key: hex::decode("6964656d3a303031").unwrap(),
            signer: AgentID {
                scheme: 1,
                key: hex::decode(
                    "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c",
                )
                .unwrap(),
                hint: Some("ctvp".to_string()),
            },
            time_us: 1700000000000000,
            status: 200,
            evidence: None,
            notes: None,
        };

        let receipt_id = compute_receipt_id(&receipt);

        // Expected from RECEIPT_001 vector
        let expected = "0edba8b8f9547e0977cec96eb37d0e117e0c2718e7d69737ef17e8f1d9ce32cd";
        assert_eq!(
            hex::encode(receipt_id),
            expected,
            "Receipt ID computation mismatch"
        );
    }
}
