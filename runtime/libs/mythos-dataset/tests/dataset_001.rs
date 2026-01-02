/// DATASET_001 conformance
use mythos_dataset::{cid_from_bytes, compute_dataset_def_id};
use std::fs;

const VECTORS_PATH: &str = "../../../mythos-v0.2-conformance/vectors/dataset";

#[test]
fn test_dataset_001_def_id_matches() {
    let def_bin = fs::read(format!("{}/dataset_001_def.bin", VECTORS_PATH)).unwrap();
    let expected = fs::read_to_string(format!("{}/dataset_001_defid.hex", VECTORS_PATH))
        .unwrap()
        .trim()
        .to_string();

    // Decode and compute ID with field exclusion
    let decoded = mythos_can::decode_value_exact(&def_bin).unwrap();
    let computed = compute_dataset_def_id(&decoded).unwrap();

    assert_eq!(hex::encode(computed), expected, "DatasetDef ID mismatch");
}

#[test]
fn test_dataset_001_corpus_cid_matches() {
    let corpus_node =
        fs::read(format!("{}/dataset_001_corpus_rootnode.bin", VECTORS_PATH)).unwrap();
    let expected = fs::read_to_string(format!("{}/dataset_001_corpus_rootcid.hex", VECTORS_PATH))
        .unwrap()
        .trim()
        .to_string();

    let computed = cid_from_bytes(&corpus_node);
    assert_eq!(hex::encode(computed), expected, "Corpus root CID mismatch");
}

#[test]
fn test_dataset_001_manifest_cid_matches() {
    let manifest_node = fs::read(format!(
        "{}/dataset_001_manifest_rootnode.bin",
        VECTORS_PATH
    ))
    .unwrap();
    let expected = fs::read_to_string(format!("{}/dataset_001_manifest_rootcid.hex", VECTORS_PATH))
        .unwrap()
        .trim()
        .to_string();

    let computed = cid_from_bytes(&manifest_node);
    assert_eq!(
        hex::encode(computed),
        expected,
        "Manifest root CID mismatch"
    );
}
