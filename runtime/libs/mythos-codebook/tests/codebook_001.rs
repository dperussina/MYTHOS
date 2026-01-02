use mythos_codebook::codebook_id_from_bytes;
use std::fs;

#[test]
fn test_codebook_001_id_matches() {
    let entries =
        fs::read("../../../mythos-v0.2-conformance/vectors/codebook/codebook_baseline_entries.bin")
            .unwrap();
    let expected = fs::read_to_string(
        "../../../mythos-v0.2-conformance/vectors/codebook/codebook_baseline_id.hex",
    )
    .unwrap()
    .trim()
    .to_string();

    let computed = codebook_id_from_bytes(&entries);
    assert_eq!(hex::encode(computed), expected, "Codebook ID mismatch");
}
