use anyhow::{bail, Result};
use sha2::{Digest, Sha256};

/// Verify SHA256 of data against expected hex string
pub fn verify_sha256(data: &[u8], expected_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash) == expected_hex.trim()
}

// NOTE: verify_sha256_file removed - not used in manifest-driven flow
// Manifest uses expected.sha256_of_bin, not sibling files

/// Compare two byte slices and report first mismatch
pub fn compare_bytes(expected: &[u8], actual: &[u8]) -> Result<()> {
    if expected.len() != actual.len() {
        bail!(
            "Length mismatch: expected {} bytes, got {} bytes",
            expected.len(),
            actual.len()
        );
    }

    for (i, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        if e != a {
            let context_start = i.saturating_sub(16);
            let context_end = (i + 16).min(expected.len());

            eprintln!("\n❌ Byte mismatch at offset {}:", i);
            eprintln!("   Expected: 0x{:02x}", e);
            eprintln!("   Actual:   0x{:02x}", a);
            eprintln!("\nContext (offset {}..{}):", context_start, context_end);
            eprintln!(
                "Expected: {}",
                hex_window(expected, context_start, context_end)
            );
            eprintln!(
                "Actual:   {}",
                hex_window(actual, context_start, context_end)
            );

            bail!("Byte mismatch at offset {}", i);
        }
    }

    Ok(())
}

fn hex_window(data: &[u8], start: usize, end: usize) -> String {
    data[start..end]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
