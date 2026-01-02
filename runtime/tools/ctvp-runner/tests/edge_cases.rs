use ctvp_runner::report::{TestResult, TestStatus};
/// Edge case tests to prevent silent-green regressions
use ctvp_runner::suite::{infer_suite_from_id, is_implemented};

#[test]
fn test_unknown_prefix_detection() {
    // Unknown vector prefix should be caught
    let unknown_suite = infer_suite_from_id("UNKNOWN_FOO_001");
    assert_eq!(unknown_suite, "unknown");
    assert!(!is_implemented(unknown_suite));
}

#[test]
fn test_known_unimplemented_vs_unknown() {
    // Known-but-unimplemented suites
    assert_eq!(infer_suite_from_id("MERKLE_001"), "merkle");
    assert!(!is_implemented("merkle"));

    // Unknown prefix
    assert_eq!(infer_suite_from_id("BOGUS_001"), "unknown");
    assert!(!is_implemented("unknown"));
}

#[test]
fn test_exit_code_with_failures() {
    use ctvp_runner::report::exit_code;

    // Passes and skips → exit 0
    let results = vec![
        TestResult::pass("T1".to_string(), "Test".to_string()),
        TestResult::skip("T2".to_string(), "Test".to_string(), "Not impl".to_string()),
    ];
    assert_eq!(exit_code(&results), 0);

    // Any failure → exit 1
    let results = vec![
        TestResult::pass("T1".to_string(), "Test".to_string()),
        TestResult::fail("T2".to_string(), "Test".to_string(), "Mismatch".to_string()),
    ];
    assert_eq!(exit_code(&results), 1);

    // Unknown prefix should eventually FAIL, not SKIP
    let results = vec![TestResult::fail(
        "UNKNOWN_001".to_string(),
        "Test".to_string(),
        "Unknown prefix".to_string(),
    )];
    assert_eq!(exit_code(&results), 1);
}
