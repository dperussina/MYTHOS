/// Integration tests for ctvp-runner
use ctvp_runner::report::{TestResult, TestStatus};

#[test]
fn test_exit_code_logic() {
    use ctvp_runner::report::exit_code;

    // All passing → exit 0
    let results = vec![
        TestResult::pass("T1".to_string(), "Test 1".to_string()),
        TestResult::pass("T2".to_string(), "Test 2".to_string()),
    ];
    assert_eq!(exit_code(&results), 0);

    // Passing + skipped → exit 0
    let results = vec![
        TestResult::pass("T1".to_string(), "Test 1".to_string()),
        TestResult::skip(
            "T2".to_string(),
            "Test 2".to_string(),
            "Not implemented".to_string(),
        ),
    ];
    assert_eq!(exit_code(&results), 0);

    // Any failure → exit 1
    let results = vec![
        TestResult::pass("T1".to_string(), "Test 1".to_string()),
        TestResult::fail(
            "T2".to_string(),
            "Test 2".to_string(),
            "Mismatch".to_string(),
        ),
    ];
    assert_eq!(exit_code(&results), 1);

    // All skipped → exit 0
    let results = vec![TestResult::skip(
        "T1".to_string(),
        "Test 1".to_string(),
        "Not impl".to_string(),
    )];
    assert_eq!(exit_code(&results), 0);
}

#[test]
fn test_result_status_checks() {
    let pass = TestResult::pass("T1".to_string(), "Test".to_string());
    assert!(pass.is_pass());
    assert!(!pass.is_fail());
    assert!(!pass.is_skip());

    let fail = TestResult::fail("T2".to_string(), "Test".to_string(), "Error".to_string());
    assert!(!fail.is_pass());
    assert!(fail.is_fail());
    assert!(!fail.is_skip());

    let skip = TestResult::skip("T3".to_string(), "Test".to_string(), "Not impl".to_string());
    assert!(!skip.is_pass());
    assert!(!skip.is_fail());
    assert!(skip.is_skip());
}
