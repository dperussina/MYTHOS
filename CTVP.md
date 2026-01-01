# MYTHOS v0.2

## Conformance Test Vector Pack (CTVP)

**Purpose:** This pack provides byte-for-byte test vectors that prove independent implementations interoperate and implement the normative rules in RFC-MYTHOS-0001 through RFC-MYTHOS-0005.

The pack is designed so your team can:

* implement encoder/decoder and hashing
* validate MerkleList and ChunkedBlob DAGs
* validate receipts and signatures
* validate the idempotency ledger state machine
* validate deterministic dataset building
* validate MYTHOS-X packet parsing and signature verification

---

## 1. Pack Contents

The CTVP is delivered as a zip archive with this structure:

```
mythos-v0.2-conformance/
  README.md
  SPEC.md
  manifest.json
  keys/
    ed25519_test_seed.hex
    ed25519_test_public.hex
  vectors/
    can/
    merkle/
    blob/
    receipts/
    ledger/
    dataset/
    codebook/
    wire/
```

---

## 2. Encoding Reference (MYTHOS-CAN v0.2)

This pack includes `SPEC.md`, which defines the exact canonical encoding used in the vectors.

High level:

* Every value is encoded as a one-byte **type tag** followed by payload bytes.
* Maps are encoded with deterministic key ordering: keys sorted by their **encoded key bytes** (lexicographic byte compare).
* Integers use LEB128 varints. Signed integers use zigzag.

**This encoding is used for the test vectors** so canonical bytes and SHA-256 digests match exactly across implementations.

---

## 3. Crypto Reference

The pack includes a fixed Ed25519 test seed and derived public key.

* Use these ONLY for conformance testing.
* Never use them in production.

Signatures in the pack verify:

* Receipt signatures
* Ledger entry signatures
* MYTHOS-X packet signature block

---

## 4. Vector Categories

### 4.1 Canonical Encoding and Hashing

Vectors prove:

* canonical encoding is deterministic
* maps are sorted properly
* hashes match exactly

### 4.2 MerkleList

Vectors prove:

* MerkleList leaf node encoding
* root CID computation

### 4.3 ChunkedBlob DAG

Vectors prove:

* chunk hashing
* ChunkLeaf encoding
* root CID computation
* proof verification inputs

### 4.4 Receipts

Vectors prove:

* receipt_id computation
* signature over receipt_id

### 4.5 Idempotency Ledger

Vectors prove:

* IdempotencyID computation
* register-before-invoke semantics
* commit semantics
* divergence triggers

### 4.6 Deterministic Dataset Builder

Vectors prove:

* corpus root ordering
* HASH_N sampling scoring
* final manifest ordering
* manifest root CID

### 4.7 Baseline Codebook

Vectors prove:

* baseline CodebookID computation

### 4.8 MYTHOS-X Wire

Vectors prove:

* packet framing
* payload decoding
* signature verification

---

## 5. How to Use

1. Implement MYTHOS-CAN encoder/decoder.
2. Run `vectors/can/*` to validate canonical bytes and SHA-256.
3. Implement Merkle node encoding per RFC 0004 and validate `vectors/merkle/*` and `vectors/blob/*`.
4. Implement receipts per RFC 0001 and validate `vectors/receipts/*`.
5. Implement ledger semantics per RFC 0005 and validate `vectors/ledger/*`.
6. Implement dataset builder per RFC 0003 and validate `vectors/dataset/*`.
7. Implement MYTHOS-X framing and signature verification and validate `vectors/wire/*`.

---

## 6. Pack Manifest

`manifest.json` lists every vector with:

* id
* description
* normative reference (RFC section)
* input files
* expected hashes / IDs
* expected decoded JSON object

---

## 7. Acceptance Criteria

A runtime is v0.2 conformant when:

* every vector validates without tolerances
* all byte outputs match expected bytes exactly
* all hashes match expected hashes exactly
* all signatures verify with the included test public key
