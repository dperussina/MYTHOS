//! MYTHOS-CAN v0.2 Canonical Encoding
//!
//! This crate implements the MYTHOS-CAN canonical encoding specification.
//! MYTHOS-CAN is a deterministic, self-describing binary encoding format.
//!
//! # Key Properties
//! - **Deterministic**: Same value always encodes to identical bytes
//! - **Canonical**: Only one valid encoding per value
//! - **Self-describing**: Every value starts with a type tag
//! - **Content-addressable**: Enables cryptographic hashing
//!
//! # Type Tags
//! - `0x00`: NULL
//! - `0x01`: BOOL(false)
//! - `0x02`: BOOL(true)
//! - `0x03`: UVARINT (unsigned integer, LEB128)
//! - `0x04`: IVARINT (signed integer, zigzag + LEB128)
//! - `0x05`: BYTES (raw byte string)
//! - `0x06`: TEXT (UTF-8 string)
//! - `0x07`: LIST (ordered list of values)
//! - `0x08`: MAP (key-value pairs, sorted by encoded key bytes)

mod decoder;
mod encoder;
mod error;
mod value;
pub mod varint;

pub use decoder::{decode_value, decode_value_exact, decode_value_from};
pub use encoder::{encode_value, encode_value_to};
pub use error::{Error, Result};
pub use value::{tags, Value};

// Re-export varint functions for advanced use
pub use varint::{
    decode_ivarint, decode_uvarint, encode_ivarint, encode_uvarint, zigzag_decode, zigzag_encode,
};
