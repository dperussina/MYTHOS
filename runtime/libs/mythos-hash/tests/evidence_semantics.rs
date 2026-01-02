/// Test that evidence: None vs Some([]) produces different receipt_ids
use mythos_hash::{compute_receipt_id, AgentID, Receipt};

#[test]
fn test_evidence_none_vs_empty_affects_hash() {
    let base_receipt = || Receipt {
        tool_id: vec![0xAA; 32],
        request_hash: vec![0xBB; 32],
        response_hash: vec![0xCC; 32],
        idempotency_key: b"test".to_vec(),
        signer: AgentID {
            scheme: 1,
            key: vec![0xDD; 32],
            hint: None,
        },
        time_us: 1700000000000000,
        status: 200,
        evidence: None,
        notes: None,
    };

    // Receipt with evidence = None (field 9 absent)
    let receipt_none = base_receipt();
    let id_none = compute_receipt_id(&receipt_none);

    // Receipt with evidence = Some([]) (field 9 present but empty)
    let mut receipt_empty = base_receipt();
    receipt_empty.evidence = Some(vec![]);
    let id_empty = compute_receipt_id(&receipt_empty);

    // These MUST produce different receipt_ids
    // because canonical encoding distinguishes "field absent" vs "field present but empty"
    assert_ne!(
        id_none, id_empty,
        "evidence: None vs Some([]) must produce different receipt_ids"
    );
}

#[test]
fn test_evidence_order_matters() {
    // Evidence is a list - order must be preserved in canonical encoding
    let base_receipt = || Receipt {
        tool_id: vec![0xAA; 32],
        request_hash: vec![0xBB; 32],
        response_hash: vec![0xCC; 32],
        idempotency_key: b"test".to_vec(),
        signer: AgentID {
            scheme: 1,
            key: vec![0xDD; 32],
            hint: None,
        },
        time_us: 1700000000000000,
        status: 200,
        evidence: None,
        notes: None,
    };

    let h1 = vec![0x11; 32];
    let h2 = vec![0x22; 32];

    // Receipt with evidence = [h1, h2]
    let mut receipt_12 = base_receipt();
    receipt_12.evidence = Some(vec![h1.clone(), h2.clone()]);
    let id_12 = compute_receipt_id(&receipt_12);

    // Receipt with evidence = [h2, h1] (different order)
    let mut receipt_21 = base_receipt();
    receipt_21.evidence = Some(vec![h2, h1]);
    let id_21 = compute_receipt_id(&receipt_21);

    // Order MUST matter (evidence is an ordered list)
    assert_ne!(id_12, id_21, "Evidence list order must affect receipt_id");
}

#[test]
fn test_notes_none_vs_empty_affects_hash() {
    let base_receipt = || Receipt {
        tool_id: vec![0xAA; 32],
        request_hash: vec![0xBB; 32],
        response_hash: vec![0xCC; 32],
        idempotency_key: b"test".to_vec(),
        signer: AgentID {
            scheme: 1,
            key: vec![0xDD; 32],
            hint: None,
        },
        time_us: 1700000000000000,
        status: 200,
        evidence: None,
        notes: None,
    };

    // Receipt with notes = None (field 10 absent)
    let receipt_none = base_receipt();
    let id_none = compute_receipt_id(&receipt_none);

    // Receipt with notes = Some("") (field 10 present but empty)
    let mut receipt_empty = base_receipt();
    receipt_empty.notes = Some("".to_string());
    let id_empty = compute_receipt_id(&receipt_empty);

    // These MUST produce different receipt_ids
    assert_ne!(
        id_none, id_empty,
        "notes: None vs Some(\"\") must produce different receipt_ids"
    );
}
