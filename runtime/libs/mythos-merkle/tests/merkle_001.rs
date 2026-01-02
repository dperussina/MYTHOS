/// MERKLE_001 conformance test
use mythos_merkle::{cid_from_bytes, parse_merkle_node, validate_merkle_list_leaf};
use std::fs;

const VECTORS_PATH: &str = "../../../mythos-v0.2-conformance/vectors/merkle";

#[test]
fn test_merkle_001_cid_matches() {
    let leaf_bin_path = format!("{}/merklelist_001_leaf.bin", VECTORS_PATH);
    let rootcid_path = format!("{}/merklelist_001_rootcid.hex", VECTORS_PATH);

    // 1. Read binary
    let leaf_bytes = fs::read(&leaf_bin_path).expect("Failed to read leaf.bin");

    // 2. Compute CID = SHA-256(canonical bytes)
    let computed_cid = cid_from_bytes(&leaf_bytes);

    // 3. Load expected CID
    let expected_cid = fs::read_to_string(&rootcid_path)
        .expect("Failed to read expected CID")
        .trim()
        .to_string();

    // 4. Compare
    assert_eq!(hex::encode(computed_cid), expected_cid, "Root CID mismatch");
}

#[test]
fn test_merkle_001_structure_valid() {
    let leaf_bin_path = format!("{}/merklelist_001_leaf.bin", VECTORS_PATH);

    // 1. Read and decode
    let leaf_bytes = fs::read(&leaf_bin_path).expect("Failed to read");
    let decoded = mythos_can::decode_value_exact(&leaf_bytes).expect("Failed to decode");

    // 2. Parse node header
    let node = parse_merkle_node(&decoded).expect("Failed to parse node");

    // Version must be 1
    assert_eq!(node.version, 1, "Version must be 1");

    // Kind must be 1 (MerkleListLeaf)
    assert_eq!(node.kind, 1, "Kind must be 1 for MerkleListLeaf");

    // 3. Validate payload
    let leaf = validate_merkle_list_leaf(&node.payload).expect("Failed to validate leaf");

    // Should have 10 episode IDs
    assert_eq!(leaf.values.len(), 10, "MERKLE_001 has 10 episode IDs");

    // Each should be alg=1, 32 bytes
    for (i, hash) in leaf.values.iter().enumerate() {
        assert_eq!(hash.alg, 1, "Hash[{}] alg must be 1", i);
        assert_eq!(hash.bytes.len(), 32, "Hash[{}] must be 32 bytes", i);
    }
}

#[test]
fn test_merkle_001_roundtrip() {
    let leaf_bin_path = format!("{}/merklelist_001_leaf.bin", VECTORS_PATH);

    let original = fs::read(&leaf_bin_path).expect("Failed to read");

    // Decode
    let decoded = mythos_can::decode_value_exact(&original).expect("Decode failed");

    // Re-encode
    let re_encoded = mythos_can::encode_value(&decoded).expect("Re-encode failed");

    // Must be byte-identical
    assert_eq!(
        original, re_encoded,
        "Roundtrip must produce identical bytes"
    );
}
