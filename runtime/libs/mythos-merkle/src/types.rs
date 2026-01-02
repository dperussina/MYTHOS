/// Merkle structure types

/// MerkleNode outer structure (MAP with 3 fields)
#[derive(Debug, Clone)]
pub struct MerkleNodeHeader {
    pub version: u64,     // Field 1: must be 1 for v0.2
    pub kind: u64,        // Field 2: 1=MerkleListLeaf, 2=MerkleListInternal
    pub payload: Vec<u8>, // Field 3: nested structure bytes
}

/// MerkleListLeaf (nested payload structure)
#[derive(Debug, Clone)]
pub struct MerkleListLeaf {
    pub values: Vec<HashValue>, // Ordered list of Hash structs
}

/// Hash value (32-byte digest with algorithm ID)
#[derive(Debug, Clone, PartialEq)]
pub struct HashValue {
    pub alg: u64,       // 1 = SHA-256
    pub bytes: Vec<u8>, // 32 bytes for SHA-256
}

// Constants from RFC-0004
pub const VERSION: u64 = 1;
#[allow(dead_code)]
pub const KIND_MERKLE_LIST_LEAF: u64 = 1;
#[allow(dead_code)]
pub const KIND_MERKLE_LIST_INTERNAL: u64 = 2;
pub const FANOUT: usize = 1024;
pub const SHA256_ALG: u64 = 1;
