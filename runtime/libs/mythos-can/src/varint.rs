/// Varint encoding/decoding (LEB128 for unsigned, zigzag for signed)
///
/// This module implements the core varint operations required by MYTHOS-CAN.
/// All varints are little-endian with continuation bits.
use crate::error::{Error, Result};
use std::io::{Read, Write};

/// Encode an unsigned 64-bit integer as LEB128 varint
///
/// # Format
/// - Each byte carries 7 bits of payload
/// - MSB (bit 7) = 1 means "more bytes follow"
/// - MSB (bit 7) = 0 means "final byte"
/// - Little-endian bit order
///
/// # Example
/// ```
/// use mythos_can::varint::encode_uvarint;
/// let mut buf = Vec::new();
/// encode_uvarint(&mut buf, 300).unwrap();
/// assert_eq!(buf, vec![0xAC, 0x02]);  // 300 = [0b1_0101100, 0b0_0000010]
/// ```
pub fn encode_uvarint(writer: &mut impl Write, mut value: u64) -> Result<()> {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80; // Set continuation bit
        }

        writer.write_all(&[byte])?;

        if value == 0 {
            break;
        }
    }
    Ok(())
}

/// Decode an unsigned 64-bit integer from LEB128 varint
///
/// # Errors
/// Returns `Error::VarintOverflow` if more than 10 bytes are read (> 64 bits)
pub fn decode_uvarint(reader: &mut impl Read) -> Result<u64> {
    let mut result: u64 = 0;
    let mut shift = 0;

    loop {
        let mut byte = [0u8; 1];
        reader
            .read_exact(&mut byte)
            .map_err(|_| Error::UnexpectedEof)?;
        let b = byte[0];

        // Check for overflow before shifting
        if shift >= 64 {
            return Err(Error::VarintOverflow);
        }

        result |= ((b & 0x7F) as u64) << shift;
        shift += 7;

        // Final byte (MSB = 0)
        if (b & 0x80) == 0 {
            break;
        }
    }

    Ok(result)
}

/// Zigzag encode a signed 64-bit integer to unsigned
///
/// # Formula
/// `zigzag(x) = (x << 1) ^ (x >> 63)`
///
/// This maps:
/// - 0 → 0
/// - -1 → 1
/// - 1 → 2
/// - -2 → 3
/// - 2 → 4
/// ...
///
/// # Example
/// ```
/// use mythos_can::varint::zigzag_encode;
/// assert_eq!(zigzag_encode(0), 0);
/// assert_eq!(zigzag_encode(-1), 1);
/// assert_eq!(zigzag_encode(1), 2);
/// assert_eq!(zigzag_encode(-2), 3);
/// ```
pub fn zigzag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Zigzag decode an unsigned integer back to signed
///
/// # Formula
/// `x = (zz >> 1) ^ (-(zz & 1))`
///
/// # Example
/// ```
/// use mythos_can::varint::{zigzag_encode, zigzag_decode};
/// assert_eq!(zigzag_decode(zigzag_encode(-42)), -42);
/// ```
pub fn zigzag_decode(zz: u64) -> i64 {
    ((zz >> 1) as i64) ^ (-((zz & 1) as i64))
}

/// Encode a signed 64-bit integer as zigzag + LEB128 varint
pub fn encode_ivarint(writer: &mut impl Write, value: i64) -> Result<()> {
    let zz = zigzag_encode(value);
    encode_uvarint(writer, zz)
}

/// Decode a signed 64-bit integer from zigzag + LEB128 varint
pub fn decode_ivarint(reader: &mut impl Read) -> Result<i64> {
    let zz = decode_uvarint(reader)?;
    Ok(zigzag_decode(zz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uvarint_zero() {
        let mut buf = Vec::new();
        encode_uvarint(&mut buf, 0).unwrap();
        assert_eq!(buf, vec![0x00]);

        let decoded = decode_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(decoded, 0);
    }

    #[test]
    fn test_uvarint_300() {
        let mut buf = Vec::new();
        encode_uvarint(&mut buf, 300).unwrap();
        assert_eq!(buf, vec![0xAC, 0x02]);

        let decoded = decode_uvarint(&mut &buf[..]).unwrap();
        assert_eq!(decoded, 300);
    }

    #[test]
    fn test_zigzag_roundtrip() {
        let values = [0i64, -1, 1, -2, 2, -64, 64, i64::MIN, i64::MAX];
        for &v in &values {
            let zz = zigzag_encode(v);
            let decoded = zigzag_decode(zz);
            assert_eq!(v, decoded, "zigzag roundtrip failed for {}", v);
        }
    }

    #[test]
    fn test_ivarint_negative() {
        let mut buf = Vec::new();
        encode_ivarint(&mut buf, -1).unwrap();

        let decoded = decode_ivarint(&mut &buf[..]).unwrap();
        assert_eq!(decoded, -1);
    }
}
