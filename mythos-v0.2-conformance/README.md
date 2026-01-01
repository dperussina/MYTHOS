# MYTHOS v0.2 Conformance Test Vector Pack

This archive contains byte-for-byte vectors for RFC-MYTHOS-0001..0005.

## What “passing” means
A passing implementation:
- encodes each vector object into **exactly the same bytes**
- decodes each `*.bin` back into the expected `*.json` shape (where present)
- recomputes all expected SHA-256 hashes and CIDs
- verifies all Ed25519 signatures with the included public key

## Validation order (recommended)
1) **Encoding:** `vectors/can/*`
2) **Merkle:** `vectors/merkle/*` and `vectors/blob/*`
3) **Receipts:** `vectors/receipts/*`
4) **Ledger:** `vectors/ledger/*`
5) **Dataset:** `vectors/dataset/*`
6) **Codebook:** `vectors/codebook/*`
7) **Wire:** `vectors/wire/*`

## Keys
- `keys/ed25519_test_seed.hex` is included for reproducibility.
- Do not use these keys in production.

## The one “vector-only” deviation
WIRE_001 uses a **vector-only signing rule** described in SPEC.md §8 to avoid circularity in signing the signature block.

## Manifest
`manifest.json` enumerates every vector, including:
- id
- description
- RFC reference
- file list
- expected values (CIDs, hashes, etc.)
