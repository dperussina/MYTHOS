/// Temporary test to decode MERKLE_001 leaf and understand structure
use std::fs;

#[test]
fn decode_merkle_001_leaf() {
    let path = "/Users/djperussina/Code/MYTHOS/mythos-v0.2-conformance/vectors/merkle/merklelist_001_leaf.bin";
    let bytes = fs::read(path).expect("Failed to read leaf bin");

    println!("Total size: {} bytes", bytes.len());
    println!(
        "First 50 bytes (hex): {}",
        hex::encode(&bytes[..50.min(bytes.len())])
    );

    // Try to decode with mythos-can
    let decoded = mythos_can::decode_value_exact(&bytes).expect("Failed to decode");
    println!("Decoded value: {:#?}", decoded);
}
