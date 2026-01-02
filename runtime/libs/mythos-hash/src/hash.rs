/// Hash types and SHA-256 computation
use sha2::{Digest, Sha256};

/// Hash algorithm IDs (v0.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HashAlg {
    Sha256 = 1,
}

/// Hash struct (self-describing hash)
///
/// MYTHOS-CAN encoding:
/// ```text
/// Hash {
///   1: alg (u8)      - algorithm ID
///   2: bytes (bytes) - hash digest
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    pub alg: HashAlg,
    pub bytes: Vec<u8>,
}

impl Hash {
    /// Create a new SHA-256 hash
    pub fn sha256(bytes: [u8; 32]) -> Self {
        Hash {
            alg: HashAlg::Sha256,
            bytes: bytes.to_vec(),
        }
    }

    /// Compute SHA-256 hash of data
    pub fn from_data(data: &[u8]) -> Self {
        Self::sha256(sha256(data))
    }

    /// Get the hash bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to MYTHOS-CAN Value for encoding
    pub fn to_can_value(&self) -> mythos_can::Value {
        use mythos_can::Value;

        Value::Map(vec![
            (Value::UVarint(1), Value::UVarint(self.alg as u64)),
            (Value::UVarint(2), Value::Bytes(self.bytes.clone())),
        ])
    }
}

/// Compute SHA-256 hash of data
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        assert_eq!(
            hex::encode(hash),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        let hash = sha256(b"hello");
        assert_eq!(
            hex::encode(hash),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_struct_encoding() {
        let hash = Hash::sha256([0; 32]);
        let value = hash.to_can_value();

        // Should encode as MAP with fields 1 and 2
        let _encoded = mythos_can::encode_value(&value).unwrap();

        // Verify structure
        use mythos_can::Value;
        match value {
            Value::Map(pairs) => {
                assert_eq!(pairs.len(), 2);
                assert_eq!(pairs[0].0, Value::UVarint(1)); // alg field
                assert_eq!(pairs[1].0, Value::UVarint(2)); // bytes field
            }
            _ => panic!("Expected Map"),
        }
    }
}
