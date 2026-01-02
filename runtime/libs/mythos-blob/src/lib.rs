//! MYTHOS Blob Structures (RFC-0004 ChunkedBlob)
//!
//! Minimal implementation for BLOB_001 conformance.

mod types;
mod validation;

pub use types::{ChunkDesc, ChunkLeaf, ChunkedBlobNode};
pub use validation::{compute_chunk_hashes, parse_chunked_blob_node, validate_chunk_leaf};

use sha2::{Digest, Sha256};

/// Compute CID from canonical bytes
pub fn cid_from_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}
