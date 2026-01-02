/// Negative tests for canonical encoding violations
///
/// These tests ensure the decoder REJECTS non-canonical encodings
use mythos_can::{decode_value, decode_value_exact, Error, Value};

// Test 1: Duplicate MAP keys
#[test]
fn test_reject_duplicate_map_keys() {
    // MAP with duplicate keys (both TEXT("a"))
    // Should be rejected by decoder
    let bytes = vec![
        0x08, 0x02, // MAP, count=2
        0x06, 0x01, 0x61, // TEXT("a")
        0x03, 0x01, // UVARINT(1)
        0x06, 0x01, 0x61, // TEXT("a") again - DUPLICATE!
        0x03, 0x02, // UVARINT(2)
    ];

    let result = decode_value(&bytes);
    assert!(
        matches!(result, Err(Error::DuplicateMapKey)),
        "Should reject duplicate keys, got: {:?}",
        result
    );
}

// Test 2: Unsorted MAP keys
#[test]
fn test_reject_unsorted_map() {
    // MAP with keys in wrong order: "b" before "a"
    // TEXT("a") = [0x06, 0x01, 0x61]
    // TEXT("b") = [0x06, 0x01, 0x62]
    // Canonical order is "a" then "b", but this has "b" then "a"
    let bytes = vec![
        0x08, 0x02, // MAP, count=2
        0x06, 0x01, 0x62, // TEXT("b") - WRONG ORDER
        0x03, 0x02, // UVARINT(2)
        0x06, 0x01, 0x61, // TEXT("a") - should be first
        0x03, 0x01, // UVARINT(1)
    ];

    let result = decode_value(&bytes);
    assert!(
        matches!(result, Err(Error::NonCanonicalMapOrder)),
        "Should reject misordered keys, got: {:?}",
        result
    );
}

// Test 3: Trailing bytes
#[test]
fn test_reject_trailing_bytes() {
    // Valid UVARINT(42) followed by garbage
    let bytes = vec![0x03, 0x2A, 0xFF, 0xFF]; // Extra 0xFF 0xFF

    // decode_value (lenient) should succeed
    let result = decode_value(&bytes);
    assert!(
        result.is_ok(),
        "Lenient decode should accept trailing bytes"
    );

    // decode_value_exact (strict) should fail
    let result = decode_value_exact(&bytes);
    assert!(
        matches!(result, Err(Error::TrailingBytes(2))),
        "Strict decode should reject trailing bytes, got: {:?}",
        result
    );
}

// Test 4: INT64_MIN zigzag edge case
#[test]
fn test_int64_min_zigzag() {
    use mythos_can::{zigzag_decode, zigzag_encode};

    // INT64_MIN should zigzag to u64::MAX
    let encoded = zigzag_encode(i64::MIN);
    assert_eq!(encoded, u64::MAX, "INT64_MIN should zigzag to u64::MAX");

    // Should roundtrip correctly
    let decoded = zigzag_decode(encoded);
    assert_eq!(decoded, i64::MIN, "Should roundtrip INT64_MIN correctly");

    // Test full encode/decode through IVARINT
    let value = Value::IVarint(i64::MIN);
    let bytes = mythos_can::encode_value(&value).unwrap();
    let decoded_value = decode_value_exact(&bytes).unwrap();
    assert_eq!(value, decoded_value);
}

// Test 5: INT64_MAX zigzag edge case
#[test]
fn test_int64_max_zigzag() {
    use mythos_can::{zigzag_decode, zigzag_encode};

    let encoded = zigzag_encode(i64::MAX);
    let decoded = zigzag_decode(encoded);
    assert_eq!(decoded, i64::MAX);
}

// Test 6: Zigzag mapping correctness
#[test]
fn test_zigzag_mapping() {
    use mythos_can::{zigzag_decode, zigzag_encode};

    let test_cases = [
        (0i64, 0u64),
        (-1i64, 1u64),
        (1i64, 2u64),
        (-2i64, 3u64),
        (2i64, 4u64),
        (-64i64, 127u64),
        (64i64, 128u64),
    ];

    for (signed, unsigned) in &test_cases {
        assert_eq!(
            zigzag_encode(*signed),
            *unsigned,
            "zigzag_encode({}) should be {}",
            signed,
            unsigned
        );
        assert_eq!(
            zigzag_decode(*unsigned),
            *signed,
            "zigzag_decode({}) should be {}",
            unsigned,
            signed
        );
    }
}

// Additional test: Empty MAP should work
#[test]
fn test_empty_map_canonical() {
    let value = Value::Map(vec![]);
    let bytes = mythos_can::encode_value(&value).unwrap();
    assert_eq!(bytes, vec![0x08, 0x00]); // MAP tag, count 0

    let decoded = decode_value_exact(&bytes).unwrap();
    assert_eq!(decoded, value);
}

// Additional test: Single entry MAP
#[test]
fn test_single_entry_map() {
    let value = Value::Map(vec![(Value::UVarint(1), Value::Text("test".to_string()))]);

    let bytes = mythos_can::encode_value(&value).unwrap();
    let decoded = decode_value_exact(&bytes).unwrap();
    assert_eq!(decoded, value);
}
