# CTVP-RUNNER v0.2

Deterministic Conformance Runner for MYTHOS

## Purpose

ctvp-runner is the canonical verification tool for the MYTHOS Conformance Test Vector Pack (CTVP). It is the project's "byte court".

A passing implementation must:
- Reproduce exact *.bin bytes where applicable
- Recompute and match all expected SHA-256 hashes and expected hex IDs (CIDs, receipt_id, idem_id, etc.)
- Validate signatures where the pack includes them
- Enforce canonical decoding rules (reject non-canonical encodings, duplicates, bad ordering, trailing bytes) where the spec requires, or where the pack implies determinism

## Why a Dedicated Runner (Not Just Unit Tests)

- Unit tests are easy to accidentally narrow (only a few files, only "happy path")
- The runner provides:
  - Vector discovery via manifest.json
  - Structured reporting (per-vector PASS/FAIL + mismatch offsets)
  - A single command that CI and humans trust

## Non-Negotiable Behavior

### 1) Manifest-driven execution

Use `mythos-v0.2-conformance/manifest.json` as the authoritative list of vectors and associated files.

Optional: allow `--scan` as a debug mode, but normal operation is manifest-driven.

### 2) Byte-for-byte comparisons

If the runner (or any lib) produces bytes, comparisons are:
- Exact length
- Exact byte equality
- On mismatch, report:
  - First mismatching offset
  - Expected byte, actual byte
  - Hexdump window around the mismatch

### 3) SHA-256 verification

Verify using both:
- Sibling `*.sha256` file if present
- `expected.sha256_of_bin` from the manifest if present

### 4) Minimal "Phase 1" scope

MVP is CAN only:
- Load CAN_* vectors
- Verify .bin and .sha256
- Decode .bin strictly (decode_value_exact)
- Re-encode and compare bytes

JSON shape validation is optional for CAN MVP, but the runner must be architected so adapters can be added cleanly.

## CLI Contract

### Command surface

```bash
ctvp-runner verify --pack <path> [--suite can|all] [--vector CAN_001] [--fail-fast] [--json]
ctvp-runner list   --pack <path> [--suite can|all]
ctvp-runner info   --pack <path> --vector <id>
```

### Output contract

- One line per vector:
  - `PASS <ID> <description>`
  - `FAIL <ID> <reason>`
- Summary:
  - Total tested, total passed, total failed
- Exit code 0 if all pass, 1 otherwise

## Directory & Crate Layout

```
runtime/tools/ctvp-runner/
├── Cargo.toml
├── IMPLEMENTATION.md (this file)
└── src/
    ├── main.rs          - Entry point, CLI parsing
    ├── cli.rs           - Command definitions
    ├── manifest.rs      - Manifest loading and parsing
    ├── verify/
    │   ├── mod.rs       - Verification orchestration
    │   ├── can.rs       - CAN suite verification
    │   └── utils.rs     - Byte comparison, SHA256, diff reporting
    └── report.rs        - Output formatting
```

## Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
hex = "0.4"
anyhow = "1.0"
walkdir = "2.5"  # For optional scan mode

# Workspace dependencies
mythos-can = { path = "../../libs/mythos-can" }
# Future: mythos-hash, mythos-receipts, etc.
```

## Data Model

### Manifest schema

Load from `<pack>/manifest.json`.

Minimal representation:
```rust
struct PackManifest {
    pack: String,
    version: String,
    vectors: Vec<VectorEntry>,
}

struct VectorEntry {
    id: String,
    description: String,
    rfc_reference: String,
    files: HashMap<String, String>,
    expected: HashMap<String, serde_json::Value>,
}
```

Do not overfit. Expect additional keys over time.

## Verification Pipeline (MVP = CAN)

### Algorithm (CAN suite)

For each VectorEntry with id prefix `CAN_`:

1. **Resolve required paths**
   - bin path from `entry.files["bin"]`
   - json optional from `entry.files.get("json")`

2. **Load bytes**
   - read `<pack>/<bin>`

3. **Verify SHA-256**
   - if sibling file exists (`<bin>.sha256`): compare
   - if manifest contains `expected.sha256_of_bin`: compare
   - fail if either expected check fails

4. **Strict decode** (canonical acceptance policy)
   - `decoded = mythos_can::decode_value_exact(bin_bytes)`
   - if decode fails, FAIL the vector

5. **Re-encode**
   - `re = mythos_can::encode_value(&decoded)`
   - compare to `bin_bytes` byte-for-byte
   - if mismatch:
     - find first mismatch offset
     - print diff window
     - FAIL

6. **Optional JSON checks** (phase 1 optional)
   - If json exists, don't block MVP on type adapters
   - Implement a pluggable adapter hook:
     - `Adapter::value_to_json(decoded) -> serde_json::Value`
     - `Adapter::json_to_value(json) -> Value`

## MVP Definition of Done

```bash
cargo run -p ctvp-runner -- verify --pack ./mythos-v0.2-conformance --suite can
```

Prints PASS for all CAN vectors and exits 0.

## Reporting & Debuggability

### Byte diff

On mismatch:
- Show first mismatch offset `i`
- Show `expected[i]`, `actual[i]`
- Print window `[max(0,i-16) .. min(len,i+16)]` in hex for both streams

### Fail-fast option

`--fail-fast` stops on first failure (useful in CI and quick local iteration).

## CI Gate (Add Immediately)

Create a GitHub Action job that runs:

```yaml
- name: CTVP Conformance Gate
  run: |
    cargo test -p mythos-can
    cargo run -p ctvp-runner -- verify --pack ./mythos-v0.2-conformance --suite can --fail-fast
```

This locks in "canonical bytes" as an always-on invariant.

## Canonical Hardening Notes

### Non-minimal varints

If `decode_uvarint` accepts overlong encodings, your decoder can accept multiple byte representations for the same value.

**Required behavior in strict contexts (runner decode)**:
- Reject non-minimal varints
- Most robust implementation: capture consumed bytes, re-encode decoded value, compare
- If different, return `NonCanonicalVarint`

### MAP key order validation

Your decoder checks canonical order by re-encoding each decoded key. That's OK for v0.2 vectors, but as scale grows, you eventually want `decode_value_from` to return `(Value, raw_bytes_consumed)` so MAP validation can compare actual encoded key bytes without re-encoding.

Not required for milestone 1, but keep in mind for "infinite scale".

## Agent Consult Triggers

Use domain experts deliberately:

- **CTVP & Conformance Expert**
  - Manifest rules
  - How to treat .sha256, .hex, and "expected" keys
  - What "passing" means for each suite

- **Canonical Encoding Expert**
  - Non-minimal varints policy
  - Duplicate key policy
  - Decoder strictness rules (trailing bytes, canonical ordering)

- **Wire Protocol Expert** (later)
  - WIRE_001 "vector-only signing rule" to avoid circular signing

- **Receipt & Ledger Expert** (milestone 2)
  - receipt_id/idem_id computation windows (field exclusions)
  - commit/register invariants

## Immediate Next Sprint After Runner (Milestone 2)

Once `ctvp-runner verify --suite can` is green in CI:

1. mythos-hash
2. Implement receipt_id and idem_id rules
3. Extend runner with `--suite receipts` and `--suite ledger`
4. Pass:
   - vectors/receipts/*
   - vectors/ledger/* (start with ID + hash checks, then state machine invariants)
