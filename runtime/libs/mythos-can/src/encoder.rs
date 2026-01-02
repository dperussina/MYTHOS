/// MYTHOS-CAN Encoder
///
/// Implements canonical encoding for all MYTHOS value types.
/// The encoder ensures deterministic output - same input always produces
/// identical bytes.
use crate::{tags, varint, Error, Result, Value};
use std::io::Write;

/// Encode a Value to bytes
pub fn encode_value(value: &Value) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    encode_value_to(&mut buf, value)?;
    Ok(buf)
}

/// Encode a Value to a writer
pub fn encode_value_to(writer: &mut impl Write, value: &Value) -> Result<()> {
    match value {
        Value::Null => {
            writer.write_all(&[tags::NULL])?;
        }

        Value::Bool(false) => {
            writer.write_all(&[tags::BOOL_FALSE])?;
        }

        Value::Bool(true) => {
            writer.write_all(&[tags::BOOL_TRUE])?;
        }

        Value::UVarint(n) => {
            writer.write_all(&[tags::UVARINT])?;
            varint::encode_uvarint(writer, *n)?;
        }

        Value::IVarint(n) => {
            writer.write_all(&[tags::IVARINT])?;
            varint::encode_ivarint(writer, *n)?;
        }

        Value::Bytes(bytes) => {
            writer.write_all(&[tags::BYTES])?;
            varint::encode_uvarint(writer, bytes.len() as u64)?;
            writer.write_all(bytes)?;
        }

        Value::Text(text) => {
            writer.write_all(&[tags::TEXT])?;
            let bytes = text.as_bytes();
            varint::encode_uvarint(writer, bytes.len() as u64)?;
            writer.write_all(bytes)?;
        }

        Value::List(items) => {
            writer.write_all(&[tags::LIST])?;
            varint::encode_uvarint(writer, items.len() as u64)?;
            for item in items {
                encode_value_to(writer, item)?;
            }
        }

        Value::Map(pairs) => {
            encode_map(writer, pairs)?;
        }
    }

    Ok(())
}

/// Encode a MAP with canonical key ordering
///
/// CRITICAL: Keys MUST be sorted by their encoded bytes (lexicographic).
/// This is the most error-prone part of MYTHOS-CAN encoding.
///
/// # Algorithm
/// 1. Encode each key to bytes
/// 2. Sort pairs by encoded key bytes (lexicographic comparison)
/// 3. Emit tag + count + sorted pairs
fn encode_map(writer: &mut impl Write, pairs: &[(Value, Value)]) -> Result<()> {
    writer.write_all(&[tags::MAP])?;
    varint::encode_uvarint(writer, pairs.len() as u64)?;

    // Encode all keys and collect (key_bytes, key, value) tuples
    let mut encoded_pairs: Vec<(Vec<u8>, &Value, &Value)> = Vec::new();
    for (key, value) in pairs {
        let key_bytes = encode_value(key)?;
        encoded_pairs.push((key_bytes, key, value));
    }

    // Sort by encoded key bytes (lexicographic)
    // This is CANONICAL ordering requirement
    encoded_pairs.sort_by(|a, b| a.0.cmp(&b.0));

    // Check for duplicate keys (adjacent entries after sorting)
    for i in 1..encoded_pairs.len() {
        if encoded_pairs[i].0 == encoded_pairs[i - 1].0 {
            return Err(Error::DuplicateMapKey);
        }
    }

    // Write sorted pairs
    for (key_bytes, _, value) in encoded_pairs {
        // Write pre-encoded key bytes
        writer.write_all(&key_bytes)?;
        // Encode value
        encode_value_to(writer, value)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_null() {
        let value = Value::Null;
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x00]);
    }

    #[test]
    fn test_encode_bool_false() {
        let value = Value::Bool(false);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x01]);
    }

    #[test]
    fn test_encode_bool_true() {
        let value = Value::Bool(true);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x02]);
    }

    #[test]
    fn test_encode_uvarint_zero() {
        let value = Value::UVarint(0);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x03, 0x00]);
    }

    #[test]
    fn test_encode_uvarint_300() {
        let value = Value::UVarint(300);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x03, 0xAC, 0x02]);
    }

    #[test]
    fn test_encode_bytes_empty() {
        let value = Value::Bytes(vec![]);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x05, 0x00]);
    }

    #[test]
    fn test_encode_text() {
        let value = Value::Text("hello".to_string());
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x06, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_encode_list_empty() {
        let value = Value::List(vec![]);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x07, 0x00]);
    }

    #[test]
    fn test_encode_map_empty() {
        let value = Value::Map(vec![]);
        let encoded = encode_value(&value).unwrap();
        assert_eq!(encoded, vec![0x08, 0x00]);
    }

    #[test]
    fn test_map_canonical_ordering() {
        // Test that map key ordering is canonical regardless of input order
        let map1 = Value::Map(vec![
            (Value::Text("b".to_string()), Value::UVarint(2)),
            (Value::Text("a".to_string()), Value::UVarint(1)),
        ]);

        let map2 = Value::Map(vec![
            (Value::Text("a".to_string()), Value::UVarint(1)),
            (Value::Text("b".to_string()), Value::UVarint(2)),
        ]);

        let enc1 = encode_value(&map1).unwrap();
        let enc2 = encode_value(&map2).unwrap();

        // Both should encode identically (keys sorted by encoded bytes)
        assert_eq!(enc1, enc2);

        // Expected: MAP tag, count 2, then "a":1, then "b":2
        // TEXT("a") = [0x06, 0x01, 0x61]
        // TEXT("b") = [0x06, 0x01, 0x62]
        // "a" < "b" lexicographically
        assert_eq!(
            enc1,
            vec![
                0x08, 0x02, // MAP, count=2
                0x06, 0x01, 0x61, // TEXT("a")
                0x03, 0x01, // UVARINT(1)
                0x06, 0x01, 0x62, // TEXT("b")
                0x03, 0x02, // UVARINT(2)
            ]
        );
    }
}
