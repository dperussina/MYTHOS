use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug)]
pub struct TestResult {
    pub id: String,
    pub description: String,
    pub status: TestStatus,
    pub error: Option<String>,
}

impl TestResult {
    pub fn pass(id: String, description: String) -> Self {
        TestResult {
            id,
            description,
            status: TestStatus::Pass,
            error: None,
        }
    }

    pub fn fail(id: String, description: String, error: String) -> Self {
        TestResult {
            id,
            description,
            status: TestStatus::Fail,
            error: Some(error),
        }
    }

    pub fn skip(id: String, description: String, reason: String) -> Self {
        TestResult {
            id,
            description,
            status: TestStatus::Skip,
            error: Some(reason),
        }
    }

    pub fn from_result(id: String, description: String, result: anyhow::Result<()>) -> Self {
        match result {
            Ok(()) => Self::pass(id, description),
            Err(e) => Self::fail(id, description, format!("{:#}", e)),
        }
    }

    pub fn is_pass(&self) -> bool {
        self.status == TestStatus::Pass
    }

    pub fn is_fail(&self) -> bool {
        self.status == TestStatus::Fail
    }

    #[allow(dead_code)]
    pub fn is_skip(&self) -> bool {
        self.status == TestStatus::Skip
    }
}

pub fn print_results(results: &[TestResult], json_output: bool) {
    if json_output {
        print_json_results(results);
    } else {
        print_text_results(results);
    }
}

fn print_text_results(results: &[TestResult]) {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for result in results {
        match result.status {
            TestStatus::Pass => {
                println!("✅ PASS {} - {}", result.id, result.description);
                passed += 1;
            }
            TestStatus::Fail => {
                println!("❌ FAIL {} - {}", result.id, result.description);
                if let Some(ref err) = result.error {
                    eprintln!("   Error: {}", err);
                }
                failed += 1;
            }
            TestStatus::Skip => {
                println!("⏭️  SKIP {} - {}", result.id, result.description);
                if let Some(ref reason) = result.error {
                    println!("   Reason: {}", reason);
                }
                skipped += 1;
            }
        }
    }

    println!(
        "\n📊 Summary: {} passed / {} skipped / {} failed / {} total",
        passed,
        skipped,
        failed,
        results.len()
    );

    if failed > 0 {
        eprintln!("\n❌ {} test(s) failed", failed);
    }
}

fn print_json_results(results: &[TestResult]) {
    #[derive(Serialize)]
    struct JsonOutput {
        total: usize,
        passed: usize,
        failed: usize,
        results: Vec<JsonResult>,
    }

    #[derive(Serialize)]
    struct JsonResult {
        id: String,
        description: String,
        status: String,
        error: Option<String>,
    }

    let json_results: Vec<JsonResult> = results
        .iter()
        .map(|r| JsonResult {
            id: r.id.clone(),
            description: r.description.clone(),
            status: match r.status {
                TestStatus::Pass => "PASS".to_string(),
                TestStatus::Fail => "FAIL".to_string(),
                TestStatus::Skip => "SKIP".to_string(),
            },
            error: r.error.clone(),
        })
        .collect();

    let passed = json_results.iter().filter(|r| r.status == "PASS").count();
    let failed = json_results.len() - passed;

    let output = JsonOutput {
        total: results.len(),
        passed,
        failed,
        results: json_results,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

pub fn exit_code(results: &[TestResult]) -> i32 {
    // Exit 0 if no failures (passes and skips are OK)
    // Exit 1 if any failures
    if results.iter().any(|r| r.is_fail()) {
        1
    } else {
        0
    }
}
