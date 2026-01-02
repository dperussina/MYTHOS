/// Validation and parsing for Merkle structures
use crate::types::*;
use mythos_can::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    #[error("Version must be 1, got {0}")]
    InvalidVersion(u64),

    #[error("Kind must be 1 (MerkleListLeaf) for MERKLE_001, got {0}")]
    InvalidKind(u64),

    #[error("Hash algorithm must be 1 (SHA-256), got {0}")]
    InvalidHashAlg(u64),

    #[error("Hash bytes must be 32, got {0}")]
    InvalidHashLength(usize),

    #[error("List must contain 1 to {FANOUT} items, got {0}")]
    InvalidListLength(usize),
}

type Result<T> = std::result::Result<T, Error>;

/// Parse MerkleNode from decoded Value
pub fn parse_merkle_node(decoded: &Value) -> Result<MerkleNodeHeader> {
    let fields = match decoded {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("MerkleNode must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // Field 1: version
    let version = match get_field(1) {
        Some(Value::UVarint(v)) => *v,
        _ => return Err(Error::InvalidStructure("Missing version field".into())),
    };

    if version != VERSION {
        return Err(Error::InvalidVersion(version));
    }

    // Field 2: kind
    let kind = match get_field(2) {
        Some(Value::UVarint(k)) => *k,
        _ => return Err(Error::InvalidStructure("Missing kind field".into())),
    };

    // Field 3: payload
    let payload = match get_field(3) {
        Some(Value::Bytes(p)) => p.clone(),
        _ => return Err(Error::InvalidStructure("Missing payload field".into())),
    };

    Ok(MerkleNodeHeader {
        version,
        kind,
        payload,
    })
}

/// Validate and parse MerkleListLeaf from payload bytes
pub fn validate_merkle_list_leaf(payload: &[u8]) -> Result<MerkleListLeaf> {
    // Decode payload
    let decoded = mythos_can::decode_value_exact(payload)
        .map_err(|e| Error::InvalidStructure(format!("Payload decode failed: {}", e)))?;

    // Payload should be MAP with field 1 = values
    let fields = match decoded {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("MerkleListLeaf must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // Field 1: values (list of Hash structs)
    let values_list = match get_field(1) {
        Some(Value::List(items)) => items,
        _ => return Err(Error::InvalidStructure("Missing values field".into())),
    };

    // Validate list length (1 to FANOUT)
    if values_list.is_empty() || values_list.len() > FANOUT {
        return Err(Error::InvalidListLength(values_list.len()));
    }

    // Parse each Hash struct
    let mut values = Vec::with_capacity(values_list.len());
    for (i, item) in values_list.iter().enumerate() {
        let hash = parse_hash_value(item)
            .map_err(|e| Error::InvalidStructure(format!("values[{}]: {}", i, e)))?;
        values.push(hash);
    }

    Ok(MerkleListLeaf { values })
}

fn parse_hash_value(val: &Value) -> Result<HashValue> {
    let fields = match val {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("Hash must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // Field 1: alg
    let alg = match get_field(1) {
        Some(Value::UVarint(a)) => *a,
        _ => return Err(Error::InvalidStructure("Hash missing alg".into())),
    };

    if alg != SHA256_ALG {
        return Err(Error::InvalidHashAlg(alg));
    }

    // Field 2: bytes
    let bytes = match get_field(2) {
        Some(Value::Bytes(b)) => b.clone(),
        _ => return Err(Error::InvalidStructure("Hash missing bytes".into())),
    };

    if bytes.len() != 32 {
        return Err(Error::InvalidHashLength(bytes.len()));
    }

    Ok(HashValue { alg, bytes })
}
