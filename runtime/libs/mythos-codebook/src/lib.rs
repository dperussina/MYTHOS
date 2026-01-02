//! MYTHOS Codebook

use sha2::{Digest, Sha256};

pub fn codebook_id_from_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}
