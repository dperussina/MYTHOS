# Next Session: MERKLE_001 → 100% Coverage

## Current State

**Completed:**
- ✅ mythos-can (canonical encoding)
- ✅ mythos-hash (content addressing, receipt_id, idempotency_id)
- ✅ ctvp-runner (3 suites: can, receipts, ledger)
- ✅ 5/10 vectors passing (50%)

**Remaining:** 5 vectors (50% → 100%)

---

## Next Vector: MERKLE_001

**Why First:** Foundation for all tree structures (BLOB, DATASET depend on it)

**Deliverables:**
1. mythos-merkle library
2. MerkleList implementation (RFC-0004)
3. CID computation for Merkle roots
4. Extend ctvp-runner with --suite merkle

**Minimal API:**
```rust
// Core types
pub enum Node {
    Leaf { values: Vec<Hash> },
    Internal { children: Vec<CID>, count: u64 },
}

// Core functions
pub fn compute_root(node: &Node) -> CID;
pub fn verify_proof(root: CID, leaf_hash: Hash, proof: &[ProofStep]) -> Result<()>;
```

**Critical Constraints:**
- Canonical node encoding (spec-driven per RFC-0004)
- Fanout = 1024 (REQUIRED for v0.2)
- Proof order matters (no sorting)
- Single leaf can be root

**Agent to Consult:** Merkle Structures Expert

**Test Vector:** MERKLE_001
- Input: 10 EpisodeIDs in single leaf
- Expected root CID: `af3a30050b542dcba9903f244280ea6c0f3797eb56869f278b2a7df90cf237d4`

---

## Implementation Pattern (Reuse)

**For each new suite, follow this pattern:**

### 1. Canonical Bytes from Binary Only
*(Previously: Binary is Source of Truth. Same rule, more explicit wording.)*

```rust
// CRITICAL: Canonical bytes are produced from decoded binary only
// JSON is presence-checked, but NOT used for hashing

// Decode binary
let decoded = mythos_can::decode_value_exact(&bin_bytes)?;

// Parse from decoded structure (not JSON)
let structure = parse_from_decoded(&decoded)?;

// Compute ID/hash from binary-derived structure
let computed_id = compute_id(&structure);
```

### 2. JSON Guards Against Rot
```rust
// Even though we use binary, JSON must exist and parse
if expected_value_present {
    let json_file = entry.files.get("json").context("...")?;
    let json_path = pack_dir.join(json_file);
    if !json_path.exists() { bail!("..."); }

    let json_str = fs::read_to_string(&json_path)?;
    let _json: serde_json::Value = serde_json::from_str(&json_str)?;
}
```

### 3. Validate Everything
```rust
// Lengths
if hash_bytes.len() != 32 { bail!("..."); }

// Bounds
if value > MAX { bail!("..."); }

// Types
match value {
    Value::Map(_) => { /* OK */ }
    _ => bail!("Must be MAP"),
}
```

### 4. Optional Field Semantics
```rust
// Distinguish absent vs present-but-empty
match get_field(9) {
    Some(Value::List(items)) => Some(parse_items(items)?),  // Even if empty!
    None => None,  // Truly absent
    _ => bail!("..."),
}
```

### 5. Add Upfront Tests
```rust
#[test]
fn test_order_matters() {
    // List order must affect hash
    assert_ne!(hash([a, b]), hash([b, a]));
}

#[test]
fn test_optional_semantics() {
    // None vs Some([]) must affect hash
    assert_ne!(hash_with_none(), hash_with_empty());
}
```

---

## "Don't Get Cute" Rules

1. **Order matters** until spec says otherwise → Add test upfront
2. **absent ≠ empty** until spec says otherwise → Add test upfront
3. **Binary is source** for hashing → JSON validates it parses
4. **Lazy contexts** for errors → `F: Fn() -> String` generic pattern
5. **Reuse helpers** → Factor generic parsers (like parse_hash_value)

---

## Success Criteria for Next Session

**MERKLE_001 passing:**
```
✅ PASS MERKLE_001 - MerkleList with 10 EpisodeIDs
📊 Summary: 6 passed / 4 skipped / 0 failed / 10 total
```

**Then burn through:**
- BLOB_001 (uses merkle)
- DATASET_001 (uses blob/merkle)
- CODEBOOK_001 (ordering/canonicalization)
- WIRE_001 (framing)

**Target:** 10/10 vectors (100%) in 1-2 sessions

---

## Agents to Consult

- **Merkle Structures Expert** - For MERKLE_001 and BLOB_001
- **Dataset & Query Expert** - For DATASET_001
- **Wire Protocol Expert** - For WIRE_001
- **CTVP & Conformance Expert** - For any vector interpretation questions

---

**Foundation is bulletproof. Next 50% will be equally boring. 🚀**
