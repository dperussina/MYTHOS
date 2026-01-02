/// Suite inference and routing
///
/// Centralizes all suite-related logic to prevent drift between
/// manifest filtering and verification dispatching.

/// Infer suite name from vector ID
pub fn infer_suite_from_id(id: &str) -> &'static str {
    if id.starts_with("CAN_") {
        "can"
    } else if id.starts_with("RECEIPT_") {
        "receipts"
    } else if id.starts_with("LEDGER_") {
        "ledger"
    } else if id.starts_with("MERKLE_") {
        "merkle"
    } else if id.starts_with("BLOB_") {
        "blob"
    } else if id.starts_with("DATASET_") {
        "dataset"
    } else if id.starts_with("CODEBOOK_") {
        "codebook"
    } else if id.starts_with("WIRE_") {
        "wire"
    } else {
        "unknown"
    }
}

/// Get ID prefix for a suite name
pub fn prefix_for_suite(suite: &str) -> &'static str {
    match suite {
        "can" => "CAN_",
        "receipts" => "RECEIPT_",
        "ledger" => "LEDGER_",
        "merkle" => "MERKLE_",
        "blob" => "BLOB_",
        "dataset" => "DATASET_",
        "codebook" => "CODEBOOK_",
        "wire" => "WIRE_",
        _ => "",
    }
}

/// Check if a suite is currently implemented
pub fn is_implemented(suite: &str) -> bool {
    matches!(
        suite,
        "can" | "receipts" | "ledger" | "merkle" | "blob" | "dataset" | "codebook" | "wire"
    )
}

// NOTE: expand_suite_alias removed until needed
// If we add --suite implemented, implement it properly in CLI first

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suite_inference() {
        assert_eq!(infer_suite_from_id("CAN_001"), "can");
        assert_eq!(infer_suite_from_id("CAN_002"), "can");
        assert_eq!(infer_suite_from_id("RECEIPT_001"), "receipts");
        assert_eq!(infer_suite_from_id("LEDGER_001"), "ledger");
        assert_eq!(infer_suite_from_id("MERKLE_001"), "merkle");
        assert_eq!(infer_suite_from_id("BLOB_001"), "blob");
        assert_eq!(infer_suite_from_id("DATASET_001"), "dataset");
        assert_eq!(infer_suite_from_id("CODEBOOK_001"), "codebook");
        assert_eq!(infer_suite_from_id("WIRE_001"), "wire");
        assert_eq!(infer_suite_from_id("UNKNOWN_FOO"), "unknown");
    }

    #[test]
    fn test_prefix_for_suite() {
        assert_eq!(prefix_for_suite("can"), "CAN_");
        assert_eq!(prefix_for_suite("receipts"), "RECEIPT_");
        assert_eq!(prefix_for_suite("ledger"), "LEDGER_");
        assert_eq!(prefix_for_suite("merkle"), "MERKLE_");
        assert_eq!(prefix_for_suite("unknown"), "");
    }

    #[test]
    fn test_is_implemented() {
        // Implemented suites
        assert!(is_implemented("can"));
        assert!(is_implemented("receipts"));
        assert!(is_implemented("ledger"));

        // Not yet implemented
        assert!(!is_implemented("merkle"));
        assert!(!is_implemented("blob"));
        assert!(!is_implemented("dataset"));
        assert!(!is_implemented("codebook"));
        assert!(!is_implemented("wire"));
        assert!(!is_implemented("unknown"));
    }
}
