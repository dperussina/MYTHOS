use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Varint overflow")]
    VarintOverflow,

    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Unknown type tag: {0:#x}")]
    UnknownTag(u8),

    #[error("Invalid encoding")]
    InvalidEncoding,

    #[error("Duplicate key in MAP")]
    DuplicateMapKey,

    #[error("MAP keys not in canonical order")]
    NonCanonicalMapOrder,

    #[error("Non-canonical varint encoding")]
    NonCanonicalVarint,

    #[error("Trailing bytes after value: {0} bytes remaining")]
    TrailingBytes(usize),
}

pub type Result<T> = std::result::Result<T, Error>;
