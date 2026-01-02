/// Receipts Suite Verification
///
/// Verifies Receipt ID computation against RECEIPT_* test vectors
use crate::manifest::VectorEntry;
use crate::verify::utils::compare_bytes;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn verify_receipt_vector(entry: &VectorEntry, pack_dir: &Path) -> Result<()> {
    // 1. Load bin file
    let bin_path = entry
        .bin_path(pack_dir)
        .ok_or_else(|| anyhow::anyhow!("No bin file for {}", entry.id))?;

    if !bin_path.exists() {
        bail!("Bin file not found: {:?}", bin_path);
    }

    let bin_bytes =
        fs::read(&bin_path).with_context(|| format!("Failed to read bin file: {:?}", bin_path))?;

    // 2. Strict decode using mythos-can
    let decoded = mythos_can::decode_value_exact(&bin_bytes)
        .with_context(|| "Failed to decode receipt bin (strict mode)")?;

    // 3. Re-encode and verify byte-identical
    let re_encoded =
        mythos_can::encode_value(&decoded).with_context(|| "Failed to re-encode receipt")?;

    compare_bytes(&bin_bytes, &re_encoded)
        .with_context(|| "Re-encoded receipt doesn't match original")?;

    // 4. Verify expected receipt_id from manifest (STRICT - no silent pass)
    if let Some(expected_id) = entry.expected.get("receipt_id").and_then(|v| v.as_str()) {
        // Strict: JSON must exist and parse (prevents pack rot)
        let json_file = entry
            .files
            .get("json")
            .context("receipt_id expected but no json file in manifest")?;

        let json_path = pack_dir.join(json_file);
        if !json_path.exists() {
            bail!(
                "receipt_id expected but JSON missing: {}",
                json_path.display()
            );
        }

        let json_str = fs::read_to_string(&json_path)?;
        let _json: serde_json::Value =
            serde_json::from_str(&json_str).context("JSON parse failed")?;

        // Build Receipt from decoded binary (source of truth)
        let receipt = parse_receipt_from_decoded(&decoded)?;
        let computed = mythos_hash::compute_receipt_id(&receipt);

        if hex::encode(computed) != expected_id {
            bail!(
                "Receipt ID mismatch:\n  Expected: {}\n  Computed: {}",
                expected_id,
                hex::encode(computed)
            );
        }
    }

    Ok(())
}

/// Parse a Hash value (MAP with alg=1, bytes field)
/// Context is lazy-evaluated via closure (zero allocation on success path)
fn parse_hash_value<F>(hash_value: &mythos_can::Value, ctx: F) -> Result<Vec<u8>>
where
    F: Fn() -> String,
{
    use mythos_can::Value;

    if let Value::Map(hash_fields) = hash_value {
        let mut alg = None;
        let mut bytes = None;

        for (k, v) in hash_fields {
            match k {
                Value::UVarint(1) => {
                    if let Value::UVarint(a) = v {
                        alg = Some(*a);
                    }
                }
                Value::UVarint(2) => {
                    if let Value::Bytes(b) = v {
                        bytes = Some(b.clone());
                    }
                }
                _ => {}
            }
        }

        if alg != Some(1) {
            bail!("{} Hash must have alg=1 (SHA-256)", ctx());
        }

        let hash_bytes = bytes.with_context(|| format!("{} Hash missing bytes", ctx()))?;

        if hash_bytes.len() != 32 {
            bail!("{} Hash bytes must be 32, got {}", ctx(), hash_bytes.len());
        }

        return Ok(hash_bytes);
    }

    bail!("{} must be Hash struct (MAP)", ctx())
}

fn parse_receipt_from_decoded(decoded: &mythos_can::Value) -> Result<mythos_hash::Receipt> {
    use mythos_can::Value;

    let fields = match decoded {
        Value::Map(pairs) => pairs,
        _ => bail!("Receipt must be MAP"),
    };

    let get_field = |n: u64| {
        fields
            .iter()
            .find(|(k, _)| matches!(k, Value::UVarint(x) if *x == n))
            .map(|(_, v)| v)
    };

    let extract_hash = |field_num: u64| -> Result<Vec<u8>> {
        let hash_value =
            get_field(field_num).with_context(|| format!("Missing field {}", field_num))?;
        parse_hash_value(hash_value, || format!("Field {}", field_num))
    };

    let tool_id = extract_hash(2)?;
    let request_hash = extract_hash(3)?;
    let response_hash = extract_hash(4)?;

    let idempotency_key = match get_field(5) {
        Some(Value::Bytes(b)) => b.clone(),
        _ => bail!("idempotency_key must be BYTES"),
    };

    // Parse AgentID (scheme, key, optional hint)
    let signer = match get_field(6) {
        Some(Value::Map(af)) => {
            let mut scheme = None;
            let mut key = None;
            let mut hint = None;
            for (k, v) in af {
                match k {
                    Value::UVarint(1) => {
                        if let Value::UVarint(s) = v {
                            scheme = Some(*s as u8);
                        }
                    }
                    Value::UVarint(2) => {
                        if let Value::Bytes(k) = v {
                            key = Some(k.clone());
                        }
                    }
                    Value::UVarint(3) => {
                        if let Value::Text(h) = v {
                            hint = Some(h.clone());
                        }
                    }
                    _ => {}
                }
            }
            let k = key.context("AgentID missing key")?;
            if k.len() != 32 {
                bail!("AgentID key must be 32 bytes");
            }
            mythos_hash::AgentID {
                scheme: scheme.context("AgentID missing scheme")?,
                key: k,
                hint,
            }
        }
        _ => bail!("signer must be AgentID MAP"),
    };

    let time_us = match get_field(7) {
        Some(Value::IVarint(t)) if *t >= 0 => *t,
        Some(Value::IVarint(t)) => bail!("time_us must be non-negative, got {}", t),
        _ => bail!("time_us must be IVARINT"),
    };

    let status = match get_field(8) {
        Some(Value::UVarint(s)) if *s <= u16::MAX as u64 => *s as u16,
        Some(Value::UVarint(s)) => bail!("status too large: {}", s),
        _ => bail!("status must be UVARINT"),
    };

    // Evidence: None (absent) vs Some([]) (present but empty) vs Some([h1, h2, ...])
    let evidence = match get_field(9) {
        Some(Value::List(items)) => {
            let mut hashes = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                hashes.push(parse_hash_value(item, || format!("Field 9[{}]", i))?);
            }
            Some(hashes) // present, even if empty
        }
        None => None, // absent
        _ => bail!("Field 9 (evidence) must be LIST if present"),
    };

    let notes = match get_field(10) {
        Some(Value::Text(t)) => Some(t.clone()),
        None => None,
        _ => bail!("notes must be TEXT"),
    };

    Ok(mythos_hash::Receipt {
        tool_id,
        request_hash,
        response_hash,
        idempotency_key,
        signer,
        time_us,
        status,
        evidence,
        notes,
    })
}
