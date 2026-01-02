/// CTVP Test Vectors for MYTHOS-CAN
///
/// These tests validate mythos-can against the conformance test vectors
/// in mythos-v0.2-conformance/vectors/can/
use mythos_can::{decode_value, decode_value_exact, encode_value, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const VECTORS_PATH: &str = "../../../mythos-v0.2-conformance/vectors/can";

fn verify_sha256(data: &[u8], expected_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash) == expected_hex.trim()
}

#[test]
#[ignore] // Enable when ready to test against vectors
fn test_hash_001_encode() {
    let json_path = Path::new(VECTORS_PATH).join("hash_001.json");
    let bin_path = Path::new(VECTORS_PATH).join("hash_001.bin");
    let sha_path = Path::new(VECTORS_PATH).join("hash_001.bin.sha256");

    // Skip if vectors not available
    if !json_path.exists() {
        eprintln!("Test vectors not found, skipping");
        return;
    }

    // Load expected binary
    let expected_bin = fs::read(&bin_path).expect("Failed to read .bin file");
    let expected_sha = fs::read_to_string(&sha_path).expect("Failed to read .sha256 file");

    // Parse JSON and create Value
    // For now, manually construct the expected value
    let value = Value::Map(vec![
        (Value::UVarint(1), Value::UVarint(1)),
        (Value::UVarint(2), Value::Bytes(vec![0; 32])),
    ]);

    // Encode
    let encoded = encode_value(&value).expect("Failed to encode");

    // Verify against expected binary
    assert_eq!(
        encoded, expected_bin,
        "Encoded bytes don't match expected .bin file"
    );

    // Verify SHA256
    assert!(
        verify_sha256(&encoded, &expected_sha),
        "SHA256 hash doesn't match"
    );
}

#[test]
#[ignore] // Enable when ready to test against vectors
fn test_map_order_001_encode() {
    let bin_path = Path::new(VECTORS_PATH).join("map_order_001.bin");
    let sha_path = Path::new(VECTORS_PATH).join("map_order_001.bin.sha256");

    if !bin_path.exists() {
        eprintln!("Test vectors not found, skipping");
        return;
    }

    let expected_bin = fs::read(&bin_path).expect("Failed to read .bin file");
    let expected_sha = fs::read_to_string(&sha_path).expect("Failed to read .sha256 file");

    // Map with keys that must be sorted canonically
    let value = Value::Map(vec![
        (Value::Text("a".to_string()), Value::UVarint(1)),
        (Value::Text("b".to_string()), Value::UVarint(2)),
    ]);

    let encoded = encode_value(&value).expect("Failed to encode");

    assert_eq!(
        encoded, expected_bin,
        "Encoded bytes don't match expected .bin file"
    );

    assert!(
        verify_sha256(&encoded, &expected_sha),
        "SHA256 hash doesn't match"
    );
}

#[test]
#[ignore] // Enable when ready to test against vectors
fn test_agentid_001_encode() {
    let bin_path = Path::new(VECTORS_PATH).join("agentid_001.bin");
    let sha_path = Path::new(VECTORS_PATH).join("agentid_001.bin.sha256");

    if !bin_path.exists() {
        eprintln!("Test vectors not found, skipping");
        return;
    }

    let expected_bin = fs::read(&bin_path).expect("Failed to read .bin file");
    let expected_sha = fs::read_to_string(&sha_path).expect("Failed to read .sha256 file");

    // AgentID structure: scheme=1, key=32 bytes, hint="ctvp"
    let key_bytes = vec![
        0x8a, 0x88, 0xe3, 0xdd, 0x74, 0x09, 0xf1, 0x95, 0xfd, 0x52, 0xdb, 0x2d, 0x3c, 0xba, 0x5d,
        0x72, 0xca, 0x67, 0x09, 0xbf, 0x1d, 0x94, 0x12, 0x1b, 0xf3, 0x74, 0x88, 0x01, 0xb4, 0x0f,
        0x6f, 0x5c,
    ];

    let value = Value::Map(vec![
        (Value::UVarint(1), Value::UVarint(1)),       // scheme=1
        (Value::UVarint(2), Value::Bytes(key_bytes)), // key
        (Value::UVarint(3), Value::Text("ctvp".to_string())), // hint
    ]);

    let encoded = encode_value(&value).expect("Failed to encode");

    assert_eq!(
        encoded, expected_bin,
        "Encoded bytes don't match expected .bin file"
    );

    assert!(
        verify_sha256(&encoded, &expected_sha),
        "SHA256 hash doesn't match"
    );
}

/// DATA-DRIVEN TEST: Iterate all vectors/can/*.bin files
#[test]
#[ignore]
fn test_all_can_vectors_data_driven() {
    let vectors_dir = Path::new(VECTORS_PATH);

    if !vectors_dir.exists() {
        eprintln!("CTVP vectors not found at {:?}, skipping", vectors_dir);
        return;
    }

    let mut tested = 0;
    let mut passed = 0;
    let mut failed = Vec::new();

    // Discover all .bin files
    for entry in fs::read_dir(vectors_dir).expect("Failed to read vectors dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension() == Some(OsStr::new("bin")) {
            let stem = path.file_stem().unwrap().to_str().unwrap();

            // Skip .sha256 files
            if stem.ends_with(".bin") {
                continue;
            }

            tested += 1;

            match test_single_vector(&stem) {
                Ok(()) => {
                    println!("✅ PASS: {}", stem);
                    passed += 1;
                }
                Err(e) => {
                    eprintln!("❌ FAIL: {} - {}", stem, e);
                    failed.push((stem.to_string(), e));
                }
            }
        }
    }

    println!("\n📊 Results: {} passed / {} tested", passed, tested);

    if !failed.is_empty() {
        panic!("❌ {} vector test(s) failed:\n{:#?}", failed.len(), failed);
    }

    assert!(tested > 0, "No test vectors found!");
}

fn test_single_vector(name: &str) -> Result<(), String> {
    let bin_path = Path::new(VECTORS_PATH).join(format!("{}.bin", name));
    let sha_path = Path::new(VECTORS_PATH).join(format!("{}.bin.sha256", name));
    let json_path = Path::new(VECTORS_PATH).join(format!("{}.json", name));

    // 1. Verify .bin file SHA256
    let bin_data = fs::read(&bin_path).map_err(|e| format!("Failed to read .bin: {}", e))?;

    if sha_path.exists() {
        let expected_sha =
            fs::read_to_string(&sha_path).map_err(|e| format!("Failed to read .sha256: {}", e))?;

        if !verify_sha256(&bin_data, &expected_sha) {
            return Err(format!("SHA256 mismatch"));
        }
    }

    // 2. Decode .bin file
    let decoded =
        decode_value_exact(&bin_data).map_err(|e| format!("Failed to decode .bin: {}", e))?;

    // 3. Re-encode and verify byte-identical
    let re_encoded = encode_value(&decoded).map_err(|e| format!("Failed to re-encode: {}", e))?;

    if re_encoded != bin_data {
        return Err(format!(
            "Re-encode mismatch: expected {} bytes, got {} bytes",
            bin_data.len(),
            re_encoded.len()
        ));
    }

    // 4. If .json exists, verify structure (optional validation)
    if json_path.exists() {
        // JSON validation would go here
        // For now, we trust decode-encode roundtrip
    }

    Ok(())
}

#[test]
fn test_basic_roundtrip() {
    // Test basic encode/decode roundtrip
    let values = vec![
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::UVarint(0),
        Value::UVarint(300),
        Value::IVarint(-1),
        Value::IVarint(42),
        Value::Bytes(vec![1, 2, 3]),
        Value::Text("hello".to_string()),
        Value::List(vec![Value::UVarint(1), Value::UVarint(2)]),
        Value::Map(vec![(Value::UVarint(1), Value::Text("test".to_string()))]),
    ];

    for value in values {
        let encoded = encode_value(&value).expect("Encoding failed");
        let decoded = decode_value(&encoded).expect("Decoding failed");
        assert_eq!(value, decoded, "Roundtrip failed for {:?}", value);
    }
}
