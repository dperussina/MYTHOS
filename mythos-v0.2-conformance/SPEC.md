# MYTHOS Conformance Test Vector Pack (CTVP) v0.2

This document is **normative for the test vectors in this pack**. It fully defines the canonical encoding and the “vector-only” signing rule used by the MYTHOS-X wire vector.

If your implementation matches these rules, it will match every `*.bin` byte sequence and every expected SHA-256 in this pack.

---

## 1. MYTHOS-CAN v0.2 (Canonical Encoding)

MYTHOS-CAN is a compact, deterministic, self-describing encoding.

### 1.1 Type Tags
Each value begins with a one-byte tag:

- `0x00` NULL
- `0x01` BOOL(false)
- `0x02` BOOL(true)
- `0x03` UVARINT (unsigned integer)
- `0x04` IVARINT (signed integer, zigzag)
- `0x05` BYTES
- `0x06` TEXT (UTF-8)
- `0x07` LIST
- `0x08` MAP

### 1.2 Varints (LEB128)
UVARINT values are encoded as unsigned LEB128:

- Each byte carries 7 bits of payload.
- MSB `1` means “more bytes follow”, MSB `0` means “final byte”.

### 1.3 Zigzag for IVARINT
IVARINT uses zigzag transform on signed i64:

`zz = (x << 1) ^ (x >> 63)` (arithmetic right shift)

Then `zz` is encoded as UVARINT.

### 1.4 BYTES
`0x05` + `len` (UVARINT) + raw bytes

### 1.5 TEXT
`0x06` + `len` (UVARINT) + UTF-8 bytes  
Implementations MUST reject invalid UTF-8 when decoding TEXT.

### 1.6 LIST
`0x07` + `count` (UVARINT) + concatenation of `count` encoded values.

### 1.7 MAP
`0x08` + `count` (UVARINT) + concatenation of `count` pairs: `key` then `value`, where each is an encoded value.

**Canonical ordering (required):** map pairs MUST be sorted by the **encoded key bytes** (lexicographic byte compare), ascending.

- Keys MUST be encoded to bytes first.
- Sort by those bytes.
- Then serialize pairs in that order.

### 1.8 Struct Convention
All “structs” in v0.2 vectors are represented as MAP where keys are integer field numbers.

Field keys are encoded as UVARINT values (tag `0x03`) whose numeric value is the field number.

Example: `{1: U(1), 2: <bytes>}` encodes as a MAP with two pairs, key `1` then `2`.

---

## 2. Hashing
Unless otherwise specified, all digests are SHA-256.

- `SHA-256(x)` yields 32 bytes.
- Hex files (`*.hex`) are lowercase hex without prefix.

---

## 3. Content IDs in this pack
A CID is represented as a SHA-256 Hash struct:

`Hash { 1: alg=1, 2: bytes=32 }`

---

## 4. Receipt ID rule used by RECEIPT_001
In these vectors:

`receipt_id = SHA-256( canonical_bytes(receipt_without_fields_1_and_11) )`

Where:
- field `1` is the receipt_id itself
- field `11` is the Signature struct

The receipt signature is Ed25519 over the **raw receipt_id bytes** (32 bytes).

---

## 5. Ledger ID rule used by LEDGER_001
The pack includes “recommended” IDs for ledger entries (not required by the RFC, but used for vector signatures):

- `register_id = SHA-256( canonical_bytes(register_without_signature_field) )`
- `commit_id   = SHA-256( canonical_bytes(commit_without_signature_field) )`

Signatures are Ed25519 over those raw 32-byte ids.

**IdempotencyID** (required by RFC 0005):
`idem_id = SHA-256( tool_id_bytes || idempotency_key_bytes )`

Where `tool_id_bytes` is the 32-byte SHA-256 digest inside the ToolID Hash struct.

---

## 6. Merkle CIDs (RFC 0004)
Merkle node bytes are the canonical bytes of the MerkleNode struct:

`MerkleNode = { 1: version, 2: kind, 3: payload_bytes }`

CID is:
`cid = SHA-256( merkle_node_bytes )`

Payload bytes are the canonical bytes of the nested payload struct.

---

## 7. CodebookID (baseline)
The baseline codebook list is encoded as a LIST of CodebookEntry structs.

`codebook_id = SHA-256( canonical_bytes(list(entries)) )`

---

## 8. MYTHOS-X Wire Vector Signing Rule (vector-only)
The WIRE_001 packet uses this vector-only signing rule:

- Compute `digest = SHA-256( header_bytes || payload_bytes )`
- Signature is Ed25519 over `digest`

The packet contains:
- header
- sigblock length + sigblock (which includes signature)
- payload length + payload

Production systems MAY choose a different signing rule, but to match WIRE_001 you MUST implement the rule above in your test harness.

---

## 9. Files and Hashes
Every `*.bin` file has a sibling `*.sha256` containing the SHA-256 of the binary file bytes. Implementations SHOULD verify these first to ensure the archive is intact.
