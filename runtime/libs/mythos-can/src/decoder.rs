/// MYTHOS-CAN Decoder
///
/// Implements decoding for all MYTHOS value types.
/// The decoder must validate:
/// - UTF-8 validity for TEXT
/// - Proper varint encoding
/// - Unknown tags are rejected
use crate::{tags, varint, Error, Result, Value};
use std::io::Read;

/// Decode a Value from bytes (lenient - allows trailing bytes)
pub fn decode_value(bytes: &[u8]) -> Result<Value> {
    decode_value_from(&mut &bytes[..])
}

/// Decode a Value from bytes, rejecting trailing bytes
///
/// This is the strict decoding mode for test vectors and validation.
/// Use this when decoding a complete buffer that should contain exactly one value.
pub fn decode_value_exact(bytes: &[u8]) -> Result<Value> {
    let mut reader = &bytes[..];
    let value = decode_value_from(&mut reader)?;

    // Check for trailing bytes
    if !reader.is_empty() {
        return Err(Error::TrailingBytes(reader.len()));
    }

    Ok(value)
}

/// Decode a Value from a reader
pub fn decode_value_from(reader: &mut impl Read) -> Result<Value> {
    // Read type tag
    let mut tag = [0u8; 1];
    reader
        .read_exact(&mut tag)
        .map_err(|_| Error::UnexpectedEof)?;

    match tag[0] {
        tags::NULL => Ok(Value::Null),

        tags::BOOL_FALSE => Ok(Value::Bool(false)),

        tags::BOOL_TRUE => Ok(Value::Bool(true)),

        tags::UVARINT => {
            let n = varint::decode_uvarint(reader)?;
            Ok(Value::UVarint(n))
        }

        tags::IVARINT => {
            let n = varint::decode_ivarint(reader)?;
            Ok(Value::IVarint(n))
        }

        tags::BYTES => {
            let len = varint::decode_uvarint(reader)? as usize;
            let mut bytes = vec![0u8; len];
            reader
                .read_exact(&mut bytes)
                .map_err(|_| Error::UnexpectedEof)?;
            Ok(Value::Bytes(bytes))
        }

        tags::TEXT => {
            let len = varint::decode_uvarint(reader)? as usize;
            let mut bytes = vec![0u8; len];
            reader
                .read_exact(&mut bytes)
                .map_err(|_| Error::UnexpectedEof)?;

            // MUST validate UTF-8
            let text = String::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?;

            Ok(Value::Text(text))
        }

        tags::LIST => {
            let count = varint::decode_uvarint(reader)? as usize;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(decode_value_from(reader)?);
            }
            Ok(Value::List(items))
        }

        tags::MAP => {
            let count = varint::decode_uvarint(reader)? as usize;
            let mut pairs = Vec::with_capacity(count);
            let mut last_key_bytes: Option<Vec<u8>> = None;

            for _ in 0..count {
                // Decode key and capture its encoded bytes
                let key = decode_value_from(reader)?;
                let current_key_bytes = crate::encoder::encode_value(&key)?;

                // Enforce canonical order: keys must be strictly ascending
                if let Some(ref last) = last_key_bytes {
                    match current_key_bytes.cmp(last) {
                        std::cmp::Ordering::Equal => {
                            // Duplicate key detected
                            return Err(Error::DuplicateMapKey);
                        }
                        std::cmp::Ordering::Less => {
                            // Keys not in canonical order
                            return Err(Error::NonCanonicalMapOrder);
                        }
                        std::cmp::Ordering::Greater => {
                            // OK - ascending order
                        }
                    }
                }

                let value = decode_value_from(reader)?;
                pairs.push((key, value));
                last_key_bytes = Some(current_key_bytes);
            }

            Ok(Value::Map(pairs))
        }

        unknown => Err(Error::UnknownTag(unknown)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_null() {
        let bytes = vec![0x00];
        let value = decode_value(&bytes).unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_decode_bool() {
        assert_eq!(decode_value(&[0x01]).unwrap(), Value::Bool(false));
        assert_eq!(decode_value(&[0x02]).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_decode_uvarint() {
        let bytes = vec![0x03, 0xAC, 0x02]; // tag + 300
        let value = decode_value(&bytes).unwrap();
        assert_eq!(value, Value::UVarint(300));
    }

    #[test]
    fn test_decode_text() {
        let bytes = vec![0x06, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let value = decode_value(&bytes).unwrap();
        assert_eq!(value, Value::Text("hello".to_string()));
    }

    #[test]
    fn test_decode_invalid_utf8() {
        // Invalid UTF-8 sequence
        let bytes = vec![0x06, 0x02, 0xFF, 0xFE];
        let result = decode_value(&bytes);
        assert!(matches!(result, Err(Error::InvalidUtf8)));
    }

    #[test]
    fn test_roundtrip() {
        use crate::encoder::encode_value;

        let values = vec![
            Value::Null,
            Value::Bool(false),
            Value::Bool(true),
            Value::UVarint(0),
            Value::UVarint(300),
            Value::IVarint(-1),
            Value::Bytes(vec![1, 2, 3]),
            Value::Text("test".to_string()),
            Value::List(vec![Value::UVarint(1), Value::UVarint(2)]),
            Value::Map(vec![(Value::UVarint(1), Value::Text("a".to_string()))]),
        ];

        for value in values {
            let encoded = encode_value(&value).unwrap();
            let decoded = decode_value(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }
}
