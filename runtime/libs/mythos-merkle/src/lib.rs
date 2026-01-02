//! MYTHOS Merkle Structures (RFC-0004)
//!
//! Minimal implementation for MERKLE_001 conformance.
//! Validates MerkleListLeaf structure and computes CIDs.

mod types;
mod validation;

pub use types::{HashValue, MerkleListLeaf, MerkleNodeHeader};
pub use validation::{parse_merkle_node, validate_merkle_list_leaf};

use sha2::{Digest, Sha256};

/// Compute CID from canonical bytes
///
/// For MERKLE_001: CID = SHA-256(canonical node bytes)
pub fn cid_from_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}
