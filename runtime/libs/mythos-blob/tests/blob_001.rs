/// BLOB_001 conformance test
use mythos_blob::{
    cid_from_bytes, compute_chunk_hashes, parse_chunked_blob_node, validate_chunk_leaf,
};
use std::fs;

const VECTORS_PATH: &str = "../../../mythos-v0.2-conformance/vectors/blob";

#[test]
fn test_blob_001_cid_matches() {
    let rootnode_path = format!("{}/chunkedblob_001_rootnode.bin", VECTORS_PATH);
    let rootcid_path = format!("{}/chunkedblob_001_rootcid.hex", VECTORS_PATH);

    let rootnode_bytes = fs::read(&rootnode_path).expect("Failed to read rootnode");
    let computed_cid = cid_from_bytes(&rootnode_bytes);

    let expected_cid = fs::read_to_string(&rootcid_path)
        .expect("Failed to read expected CID")
        .trim()
        .to_string();

    assert_eq!(hex::encode(computed_cid), expected_cid, "Root CID mismatch");
}

#[test]
fn test_blob_001_structure_valid() {
    let rootnode_path = format!("{}/chunkedblob_001_rootnode.bin", VECTORS_PATH);
    let rootnode_bytes = fs::read(&rootnode_path).expect("Failed to read");

    let decoded = mythos_can::decode_value_exact(&rootnode_bytes).expect("Decode failed");
    let node = parse_chunked_blob_node(&decoded).expect("Parse failed");

    assert_eq!(node.version, 1, "Version must be 1");
    assert_eq!(node.kind, 3, "Kind must be 3 (ChunkLeaf)");

    let leaf = validate_chunk_leaf(&node.payload).expect("Validate failed");

    assert_eq!(leaf.chunk_size, 4096, "Chunk size must be 4096");
    assert_eq!(leaf.chunks.len(), 3, "Must have 3 chunks");
    assert_eq!(leaf.total_size, 10025, "Total size must be 10025");

    // Validate chunk lengths
    assert_eq!(leaf.chunks[0].len, 4096);
    assert_eq!(leaf.chunks[1].len, 4096);
    assert_eq!(leaf.chunks[2].len, 1833);
}

#[test]
fn test_blob_001_chunk_hashes_match() {
    let payload_path = format!("{}/chunkedblob_001_payload.bin", VECTORS_PATH);
    let chunks_json_path = format!("{}/chunkedblob_001_chunks.json", VECTORS_PATH);

    let payload = fs::read(&payload_path).expect("Failed to read payload");
    let chunks_json = fs::read_to_string(&chunks_json_path).expect("Failed to read chunks JSON");
    let chunks_meta: serde_json::Value =
        serde_json::from_str(&chunks_json).expect("Parse JSON failed");

    let computed_hashes = compute_chunk_hashes(&payload, 4096);

    let expected_chunks = chunks_meta["chunks"]
        .as_array()
        .expect("chunks must be array");
    assert_eq!(
        computed_hashes.len(),
        expected_chunks.len(),
        "Chunk count mismatch"
    );

    for (i, expected) in expected_chunks.iter().enumerate() {
        let expected_hash = expected["chunk_hash"].as_str().expect("Missing chunk_hash");
        assert_eq!(
            hex::encode(computed_hashes[i]),
            expected_hash,
            "Chunk {} hash mismatch",
            i
        );
    }
}
