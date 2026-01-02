/// ChunkedBlob types

#[derive(Debug, Clone)]
pub struct ChunkedBlobNode {
    pub version: u64,
    pub kind: u64,        // 3 = ChunkLeaf
    pub payload: Vec<u8>, // Nested ChunkLeaf bytes
}

#[derive(Debug, Clone)]
pub struct ChunkLeaf {
    pub chunk_size: u64,
    pub chunks: Vec<ChunkDesc>,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct ChunkDesc {
    pub hash: Vec<u8>, // 32 bytes
    pub len: u64,
}

pub const VERSION: u64 = 1;
#[allow(dead_code)]
pub const KIND_CHUNK_LEAF: u64 = 3;
