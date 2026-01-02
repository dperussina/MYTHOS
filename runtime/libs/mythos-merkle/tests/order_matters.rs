use mythos_can::Value;
/// Test that hash order affects CID (prevents accidental sorting)
use mythos_merkle::{cid_from_bytes, HashValue};

#[test]
fn test_hash_order_affects_encoding() {
    let h1 = HashValue {
        alg: 1,
        bytes: vec![0x11; 32],
    };

    let h2 = HashValue {
        alg: 1,
        bytes: vec![0x22; 32],
    };

    // Encode as Hash structs
    let hash1_value = Value::Map(vec![
        (Value::UVarint(1), Value::UVarint(1)),
        (Value::UVarint(2), Value::Bytes(h1.bytes.clone())),
    ]);

    let hash2_value = Value::Map(vec![
        (Value::UVarint(1), Value::UVarint(1)),
        (Value::UVarint(2), Value::Bytes(h2.bytes.clone())),
    ]);

    // Create two lists with different order
    let list_12 = Value::List(vec![hash1_value.clone(), hash2_value.clone()]);
    let list_21 = Value::List(vec![hash2_value, hash1_value]);

    // Encode both
    let bytes_12 = mythos_can::encode_value(&list_12).unwrap();
    let bytes_21 = mythos_can::encode_value(&list_21).unwrap();

    // Order MUST affect bytes
    assert_ne!(bytes_12, bytes_21, "Hash order must affect encoded bytes");

    // Therefore CID must differ
    let cid_12 = cid_from_bytes(&bytes_12);
    let cid_21 = cid_from_bytes(&bytes_21);

    assert_ne!(cid_12, cid_21, "Hash order must affect CID");
}
