# CTVP v0.2: 100% conformance (10/10 vectors) + CI gate

## Summary

Implements complete MYTHOS v0.2 conformance testing infrastructure with 100% test vector coverage.

## Achievement

- ✅ **10/10 test vectors passing** (CAN, MERKLE, BLOB, RECEIPT, LEDGER, DATASET, CODEBOOK, WIRE)
- ✅ **7 core libraries** shipped (can, hash, merkle, blob, dataset, codebook, wire)
- ✅ **ctvp-runner CLI** - Manifest-driven conformance verification tool
- ✅ **CI gate** enforcing 100% conformance on every commit
- ✅ **60+ automated tests** - Unit tests + conformance tests
- ✅ **Zero technical debt**

## Conformance Verification

```bash
$ cargo run -p ctvp-runner -- verify --pack ../mythos-v0.2-conformance --suite all --fail-fast

✅ PASS CAN_001 - Canonical encoding of Hash{alg=1, bytes=32 zero bytes}.
✅ PASS CAN_002 - Canonical encoding of AgentID{scheme=1, key=Ed25519 public key, hint='ctvp'}.
✅ PASS CAN_003 - Canonical ordering of MAP keys by encoded key bytes ("a" before "b").
✅ PASS MERKLE_001 - MerkleList with 10 EpisodeIDs in a single leaf node; root CID equals leaf CID.
✅ PASS BLOB_001 - ChunkedBlob root node is a single ChunkLeaf; includes per-chunk SHA-256 hashes and root CID.
✅ PASS RECEIPT_001 - Receipt: receipt_id = SHA-256(canonical bytes excluding fields 1 and 11). Signature is Ed25519 over receipt_id bytes.
✅ PASS LEDGER_001 - EffectRegister + EffectCommit for a single idempotency key, including IdempotencyID and recommended register/commit id signatures.
✅ PASS DATASET_001 - DatasetDef with predicate TRUE and HASH_N sampling (N=5). Includes expected selected EpisodeIDs and manifest root CID.
✅ PASS CODEBOOK_001 - Baseline codebook entry list (encoded) and CodebookID = SHA-256(canonical_bytes(list(entries))).
✅ PASS WIRE_001 - MYTHOS-X packet framing with one SigBlockEntry and a minimal IRBundle payload.

📊 Summary: 10 passed / 0 skipped / 0 failed / 10 total
Exit code: 0
```

## Test Coverage

```bash
$ cargo test --workspace

60+ automated tests passing across:
- mythos-can: 29 tests (canonical encoding + negative tests)
- mythos-hash: 9 tests (SHA-256, receipt_id, idempotency_id, evidence semantics)
- mythos-merkle: 4 tests (MerkleList validation, order preservation)
- mythos-blob: 3 tests (ChunkedBlob, chunk hash validation)
- mythos-dataset: 3 tests (DatasetDef ID with field exclusion)
- mythos-codebook: 1 test (Codebook ID)
- mythos-wire: SHA-256 validation
- ctvp-runner: 12 tests (suite routing, status semantics, edge cases)
```

## Key Implementation Details

### Field-Exclusion Rules
- **receipt_id:** Excludes fields 1 (receipt_id) and 11 (signature) before hashing
- **dataset_def_id:** Excludes field 1 (dataset_def_id) before hashing
- Defensive: Accepts both UVarint and IVarint field keys
- Drift detection: Requires excluded fields to exist

### Canonical Encoding
- **Binary is source of truth** for all ID/hash computation
- **JSON validates it parses** (prevents pack rot)
- **Evidence semantics:** None ≠ Some([]) (tests verify distinction)
- **List order preserved:** All lists maintain order (tests verify)

### Architecture
- Manifest-driven (won't rot as vectors grow)
- Suite routing centralized (single source of truth)
- Unknown prefix → FAIL (drift detection)
- Lazy error contexts (zero allocation on success)

## CI Changes

Updated `.github/workflows/conformance.yml`:
- **Before:** Separate suite jobs
- **After:** Single gate: `--suite all --fail-fast` (100% coverage enforced)

## Files Changed

**New Libraries:**
- `runtime/libs/mythos-can/` - MYTHOS-CAN canonical encoding
- `runtime/libs/mythos-hash/` - SHA-256, receipt_id, idempotency_id
- `runtime/libs/mythos-merkle/` - MerkleList validation
- `runtime/libs/mythos-blob/` - ChunkedBlob validation
- `runtime/libs/mythos-dataset/` - DatasetDef ID
- `runtime/libs/mythos-codebook/` - Codebook ID
- `runtime/libs/mythos-wire/` - Wire packet SHA-256

**New Tools:**
- `runtime/tools/ctvp-runner/` - Conformance verification CLI

**Infrastructure:**
- `.github/workflows/conformance.yml` - CI gate (updated for 100%)
- `runtime/Cargo.toml` - Workspace configuration

## Pre-Merge Checklist

- [x] `cargo fmt` applied
- [x] `cargo test --workspace` passing (60+ tests)
- [x] `cargo run -p ctvp-runner -- verify --suite all --fail-fast` passing (10/10)
- [x] CI workflow updated
- [x] Zero warnings (except allowed dead_code)
- [x] No technical debt introduced

## Post-Merge Work (Separate Ticket)

- Add `make conformance` helper
- Add release checklist documentation
- Optional: Use constants in match (remove #[allow(dead_code)])
- Optional: Improve test metrics reporting

---

**Ready for review and merge.**
