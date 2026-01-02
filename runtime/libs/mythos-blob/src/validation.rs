use crate::types::*;
use mythos_can::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    #[error("Version must be 1, got {0}")]
    InvalidVersion(u64),

    #[error("Kind must be 3 (ChunkLeaf), got {0}")]
    InvalidKind(u64),

    #[error("Hash must be 32 bytes, got {0}")]
    InvalidHashLength(usize),
}

type Result<T> = std::result::Result<T, Error>;

pub fn parse_chunked_blob_node(decoded: &Value) -> Result<ChunkedBlobNode> {
    let fields = match decoded {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("Node must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    let version = match get_field(1) {
        Some(Value::UVarint(v)) => *v,
        _ => return Err(Error::InvalidStructure("Missing version".into())),
    };

    if version != VERSION {
        return Err(Error::InvalidVersion(version));
    }

    let kind = match get_field(2) {
        Some(Value::UVarint(k)) => *k,
        _ => return Err(Error::InvalidStructure("Missing kind".into())),
    };

    let payload = match get_field(3) {
        Some(Value::Bytes(p)) => p.clone(),
        _ => return Err(Error::InvalidStructure("Missing payload".into())),
    };

    Ok(ChunkedBlobNode {
        version,
        kind,
        payload,
    })
}

pub fn validate_chunk_leaf(payload: &[u8]) -> Result<ChunkLeaf> {
    let decoded = mythos_can::decode_value_exact(payload)
        .map_err(|e| Error::InvalidStructure(format!("Payload decode: {}", e)))?;

    let fields = match decoded {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("ChunkLeaf must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // Field 1: chunk_size
    let chunk_size = match get_field(1) {
        Some(Value::UVarint(s)) => *s,
        _ => return Err(Error::InvalidStructure("Missing chunk_size".into())),
    };

    // Field 2: chunks (list of ChunkDesc)
    let chunks_list = match get_field(2) {
        Some(Value::List(items)) => items,
        _ => return Err(Error::InvalidStructure("Missing chunks list".into())),
    };

    let mut chunks = Vec::new();
    for (i, item) in chunks_list.iter().enumerate() {
        let chunk_desc = parse_chunk_desc(item)
            .map_err(|e| Error::InvalidStructure(format!("chunks[{}]: {}", i, e)))?;
        chunks.push(chunk_desc);
    }

    // Field 3: total_size
    let total_size = match get_field(3) {
        Some(Value::UVarint(s)) => *s,
        _ => return Err(Error::InvalidStructure("Missing total_size".into())),
    };

    Ok(ChunkLeaf {
        chunk_size,
        chunks,
        total_size,
    })
}

fn parse_chunk_desc(val: &Value) -> Result<ChunkDesc> {
    let fields = match val {
        Value::Map(pairs) => pairs,
        _ => return Err(Error::InvalidStructure("ChunkDesc must be MAP".into())),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // Field 1: hash (Hash struct with alg=1, bytes=32)
    let hash_map = match get_field(1) {
        Some(Value::Map(hm)) => hm,
        _ => return Err(Error::InvalidStructure("Missing hash".into())),
    };

    let hash_bytes = parse_hash_bytes(hash_map)?;

    // Field 2: len
    let len = match get_field(2) {
        Some(Value::UVarint(l)) => *l,
        _ => return Err(Error::InvalidStructure("Missing len".into())),
    };

    Ok(ChunkDesc {
        hash: hash_bytes,
        len,
    })
}

fn parse_hash_bytes(hash_map: &[(Value, Value)]) -> Result<Vec<u8>> {
    let get_field = |n: u64| {
        hash_map
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    // alg must be 1
    match get_field(1) {
        Some(Value::UVarint(1)) => {}
        Some(Value::UVarint(a)) => {
            return Err(Error::InvalidStructure(format!(
                "Hash alg must be 1, got {}",
                a
            )))
        }
        _ => return Err(Error::InvalidStructure("Hash missing alg".into())),
    }

    // bytes must be 32
    let bytes = match get_field(2) {
        Some(Value::Bytes(b)) => b.clone(),
        _ => return Err(Error::InvalidStructure("Hash missing bytes".into())),
    };

    if bytes.len() != 32 {
        return Err(Error::InvalidHashLength(bytes.len()));
    }

    Ok(bytes)
}

/// Compute chunk hashes by splitting payload
pub fn compute_chunk_hashes(payload: &[u8], chunk_size: usize) -> Vec<[u8; 32]> {
    let mut hashes = Vec::new();
    let mut pos = 0;

    while pos < payload.len() {
        let end = (pos + chunk_size).min(payload.len());
        let chunk = &payload[pos..end];

        let mut hasher = Sha256::new();
        hasher.update(chunk);
        hashes.push(hasher.finalize().into());

        pos = end;
    }

    hashes
}
