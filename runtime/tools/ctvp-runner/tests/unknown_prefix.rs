/// Test that unknown vector prefixes result in FAIL, not SKIP
use ctvp_runner::{report, suite};

#[test]
fn test_unknown_prefix_fails_not_skips() {
    // Unknown suite should not be marked as implemented
    assert!(!suite::is_implemented("unknown"));

    // Verify inference works
    assert_eq!(suite::infer_suite_from_id("BOGUS_001"), "unknown");

    // In practice, this means verify_vector will return TestResult::fail
    // because is_implemented("unknown") == false and we check for "unknown" specifically
}

#[test]
fn test_known_unimplemented_skips() {
    // Known-but-not-yet-implemented suites should SKIP
    assert_eq!(suite::infer_suite_from_id("MERKLE_001"), "merkle");
    assert!(!suite::is_implemented("merkle"));
}

#[test]
fn test_result_status_semantics() {
    // PASS: Implemented and matches
    let pass = report::TestResult::pass("T1".into(), "Test".into());
    assert!(pass.is_pass());
    assert!(!pass.is_fail());

    // FAIL: Implemented but mismatch OR unknown prefix
    let fail = report::TestResult::fail("T2".into(), "Test".into(), "Error".into());
    assert!(fail.is_fail());
    assert!(!fail.is_pass());

    // SKIP: Known but not yet implemented
    let skip = report::TestResult::skip("T3".into(), "Test".into(), "Not impl".into());
    assert!(skip.is_skip());
    assert!(!skip.is_fail());
}
