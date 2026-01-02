mod blob;
mod can;
mod codebook;
mod dataset;
mod ledger;
mod merkle;
mod receipts;
pub mod utils;
mod wire;

pub use blob::verify_blob_vector;
pub use can::verify_can_vector;
pub use codebook::verify_codebook_vector;
pub use dataset::verify_dataset_vector;
pub use ledger::verify_ledger_vector;
pub use merkle::verify_merkle_vector;
pub use receipts::verify_receipt_vector;
pub use wire::verify_wire_vector;

use crate::manifest::{PackManifest, VectorEntry};
use ctvp_runner::report::TestResult;
use std::path::Path;

pub fn verify_suite(
    manifest: &PackManifest,
    pack_dir: &Path,
    suite: &str,
    specific_vector: Option<&str>,
    fail_fast: bool,
) -> Vec<TestResult> {
    let vectors = if let Some(vec_id) = specific_vector {
        manifest
            .get_vector(vec_id)
            .map(|v| vec![v])
            .unwrap_or_default()
    } else {
        manifest.vectors_for_suite(suite)
    };

    let mut results = Vec::new();

    for entry in vectors {
        let result = verify_vector(entry, pack_dir, suite);
        let is_fail = !result.is_pass();
        results.push(result);

        if is_fail && fail_fast {
            break;
        }
    }

    results
}

fn verify_vector(entry: &VectorEntry, pack_dir: &Path, suite: &str) -> TestResult {
    use ctvp_runner::suite::{infer_suite_from_id, is_implemented};

    // Infer suite from vector ID when running --suite all
    let suite_for_entry = if suite == "all" {
        infer_suite_from_id(&entry.id)
    } else {
        suite
    };

    // Handle unknown suite prefix (drift detection)
    if suite_for_entry == "unknown" {
        return TestResult::fail(
            entry.id.clone(),
            entry.description.clone(),
            format!(
                "Unknown vector prefix '{}' - update suite routing in suite.rs",
                entry.id
            ),
        );
    }

    // Check if suite is implemented
    if !is_implemented(suite_for_entry) {
        return TestResult::skip(
            entry.id.clone(),
            entry.description.clone(),
            format!("Suite '{}' not yet implemented", suite_for_entry),
        );
    }

    let result = match suite_for_entry {
        "can" => verify_can_vector(entry, pack_dir),
        "receipts" => verify_receipt_vector(entry, pack_dir),
        "ledger" => verify_ledger_vector(entry, pack_dir),
        "merkle" => verify_merkle_vector(entry, pack_dir),
        "blob" => verify_blob_vector(entry, pack_dir),
        "dataset" => verify_dataset_vector(entry, pack_dir),
        "codebook" => verify_codebook_vector(entry, pack_dir),
        "wire" => verify_wire_vector(entry, pack_dir),
        _ => Err(anyhow::anyhow!(
            "Suite dispatcher bug: {} is marked implemented but has no verifier",
            suite_for_entry
        )),
    };

    TestResult::from_result(entry.id.clone(), entry.description.clone(), result)
}
