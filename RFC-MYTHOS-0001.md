# RFC-MYTHOS-0001

## MYTHOS Core Protocol, Runtime Semantics, and Learning Substrate (v0.2)

**Category:** Standards Track (Draft)
**Intended Status:** Proposed Standard
**Version:** 0.2

---

## 1. Status of This Memo

This document specifies the core interoperability surface for MYTHOS runtimes. Distribution of this memo is unlimited.

This is a draft. Implementations MAY ship v0.2 with feature flags. Any deviation from the normative requirements in this document MUST be documented.

---

## 2. Abstract

MYTHOS defines an AI-first executable language and agent-to-agent protocol that unifies code execution, effect auditing, blob transport, and continuous learning into a single, content-addressed substrate.

Interoperability is achieved via:

* A canonical intermediate representation, MYTHOS-IR
* A compact wire encoding, MYTHOS-X
* A capability-gated effect model with signed receipts
* A blob subsystem using content addressing and Merkle chunking
* A learning substrate where traces, feedback signals, datasets, training, evaluation, and promotion are first-class typed objects

MYTHOS is machine-first. Human readability is optional. Semantics depend on canonical encodings, stable hashes, and explicit budgets.

---

## 3. Conventions and Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described in RFC 2119.

**Agent:** A principal that can sign messages and execute tasks.

**Artifact:** A content-addressed bundle of MYTHOS objects.

**Capability (Cap):** A least-privilege authorization token required for effects.

**Cell:** A shardable state machine with an append-only event log.

**Effect:** A non-deterministic operation (tool call, I/O, training) requiring caps.

**Receipt:** A signed, typed proof of an effect request and response.

**Attestation:** A signed claim that a contract or evaluation passed.

**CID:** Content identifier for blobs. In MYTHOS v0.2, CID refers to a Merkle root hash of canonical blob bytes or canonical chunk DAG.

**Ring:** A deployment stage. ring0 is canary, ringN is progressively broader.

---

## 4. Goals and Non-Goals

### 4.1 Goals

1. Deterministic execution of pure computation.
2. Explicit effect boundaries with receipts suitable for audit and caching.
3. Wire-level interoperability for agent-to-agent exchange.
4. Infinite horizontal scaling through content addressing, sharding, and budgets.
5. Continuous learning with reproducible data provenance and gated promotion.

### 4.2 Non-Goals

1. A human-friendly surface syntax is not required for interoperability.
2. This RFC does not mandate a specific model architecture.
3. This RFC does not define UI tooling.

---

## 5. System Model Overview

A compliant MYTHOS deployment contains:

* One or more MYTHOS runtimes (executors and schedulers)
* A blob store (CAS) with Merkle chunking
* A receipt ledger (append-only)
* A cell store for state (append-only event logs plus snapshots)
* Optional training and evaluation workers
* Verifier agents for attestations and quorum policies

Execution is described by MYTHOS-IR and exchanged via MYTHOS-X. Effects are allowed only with caps and emit receipts.

---

## 6. Canonical Types and Hashing

### 6.1 Hash Type

All hashes MUST be self-describing.

**Hash** is a struct:

* `alg` (u8) hash algorithm id
* `bytes` (opaque byte string)

For v0.2, implementations MUST support:

* `alg=1` SHA-256 with 32-byte output

Implementations MAY support additional algorithms. Interop MUST use alg=1 unless both sides explicitly negotiate.

### 6.2 Canonicalization Requirement

Any object that is hashed MUST be canonicalized prior to hashing.

Canonicalization MUST include:

* Stable field ordering
* Stable map ordering (sorted by canonical key bytes)
* Stable integer encodings (no leading zeros)
* Stable floating behavior (floats SHOULD NOT be used in hashed objects, if used MUST follow IEEE 754 binary64 and canonical NaN)
* Explicit typing for all nodes

### 6.3 Content Addressing

The following MUST be content-addressed:

* Types
* Contracts
* Tool interfaces
* Modules and symbols
* MYTHOS-IR bundles
* Blobs and blob chunk DAG nodes
* Training recipes and evaluation suites
* Model artifacts and promotion records

IDs are computed as `Hash( canonical_bytes(object) )`.

### 6.4 Core IDs

* TypeID = Hash(canonical TypeDef)
* ContractID = Hash(canonical ContractDef)
* ToolID = Hash(canonical ToolDef)
* SymbolID = Hash(canonical SymbolSig)
* RecipeID = Hash(canonical RecipeDef)
* EvalSuiteID = Hash(canonical EvalSuiteDef)

Names MAY be stored as metadata but MUST NOT affect IDs.

---

## 7. Canonical Encoding (MYTHOS-CAN)

MYTHOS objects MUST be encoded using a canonical binary format.

v0.2 defines **MYTHOS-CAN**, a canonical subset of CBOR-like encoding.

### 7.1 Primitive Encodings

* Unsigned integer: varint
* Signed integer: zigzag varint
* Byte string: length varint + bytes
* Text string: UTF-8 byte string
* List: length varint + items
* Map: length varint + key-value pairs, keys sorted by canonical key bytes

### 7.2 Field Numbering

All structs MUST be encoded as maps with integer field numbers. Field numbers are part of the spec for each struct.

Unknown fields:

* Receivers MUST ignore unknown fields.
* Senders MUST NOT rely on unknown field behavior.

---

## 8. MYTHOS-X Wire Protocol

MYTHOS-X is the normative wire format for agent exchange.

### 8.1 Packet Framing

A MYTHOS-X packet is:

1. Magic: 4 bytes `0x4D 0x59 0x54 0x48` ("MYTH")
2. Version: u16 (network order). v0.2 is `0x0002`
3. Flags: u16 bitfield
4. CodecID: u8
5. CodebookID: Hash
6. PayloadLen: u32 (network order)
7. Payload: bytes (length PayloadLen)
8. SigBlockLen: u32
9. SigBlock: bytes

### 8.2 Codecs

CodecID specifies how payload bytes are encoded.

v0.2 REQUIRED:

* `CodecID=1` MYTHOS-CAN encoding of a MYTHOS-IR bundle

OPTIONAL:

* `CodecID=2` zstd(MYTHOS-CAN)

Receivers MUST support CodecID=1. Senders SHOULD use CodecID=2 only if receiver advertises support.

### 8.3 Signature Block

SigBlock contains one or more signatures over the header and payload.

Signed bytes MUST be:

* Magic through end of Payload (inclusive)

SigBlock MUST be MYTHOS-CAN encoded list of:

* `Signer` (AgentID)
* `SigAlg` (u8)
* `SigBytes` (bytes)

v0.2 REQUIRED:

* SigAlg=1 Ed25519

If multiple signatures exist, receivers SHOULD verify all, and MUST verify at least one trusted signer.

### 8.4 Encryption

Encryption MAY be applied to payload bytes.
If encryption is used:

* Flags MUST indicate encryption
* The packet MUST include an EncryptionDescriptor in payload header fields
* Integrity MUST still be verifiable via signatures on ciphertext or via AEAD. Both are allowed.

---

## 9. Codebooks

Codebooks map compact codewords to IR atoms, op fragments, and common constants.

### 9.1 Baseline Codebook

A baseline codebook MUST be defined for v0.2 interoperability. Its CodebookID is fixed by this RFC release.

### 9.2 Negotiated Codebooks

Peers MAY negotiate alternate codebooks for higher density.
Negotiation MUST include:

* Proposed CodebookID
* Fallback support to baseline
* Explicit acceptance message signed by both peers

Receivers MUST reject packets with unknown CodebookID unless negotiation has occurred.

---

## 10. MYTHOS-IR Bundle

MYTHOS-IR is the canonical semantic form.

### 10.1 Bundle Structure

A MYTHOS-IR bundle is a struct with fields:

1. `meta` (Meta)
2. `const_pool` (ConstPool)
3. `node_pool` (NodePool)
4. `types` (TypeTable)
5. `symbols` (SymbolTable)
6. `contracts` (ContractTable)
7. `tools` (ToolTable)
8. `ops` (OpStream)
9. `tasks` (TaskTable)

All fields MUST be present. Empty sections MUST be encoded as empty lists.

### 10.2 Constant Pool

Constants are immutable and include:

* ints, bytes, text
* small inline blobs (see Section 13)

### 10.3 Node Pool

Nodes are typed DAG records and lists.
Each node MUST include:

* `type_id` (TypeID)
* `payload` (fields or list items)

Nodes MUST be immutable. Updates create new nodes.

### 10.4 Symbols

Symbol signatures MUST include:

* module_id
* name bytes (optional)
* arg type ids
* return type id

SymbolID is computed from the signature. Names do not affect SymbolID.

---

## 11. Opcode Set (Minimum)

v0.2 defines a minimum opcode set for interoperability. Opcodes are u8.

### 11.1 Control

* 0x01 NOP
* 0x02 HALT
* 0x10 JMP label
* 0x11 JZ reg,label
* 0x12 CALL symbol_id,argc
* 0x13 RET reg

### 11.2 Data

* 0x20 CONST reg,const_id
* 0x21 MAKE reg,type_id,field_count,(field_regs...)
* 0x22 GET reg,src_reg,field_id
* 0x23 SET reg,src_reg,field_id,val_reg
* 0x24 LIST reg,count,(regs...)
* 0x25 MATCH reg,pattern_id,label

### 11.3 Capabilities and Effects

* 0x30 REQUIRE_CAP cap_ref
* 0x31 EFFECT tool_id,arg_reg -> out_reg,receipt_reg
* 0x32 ASSERT contract_id,args_reg,receipt_reg_opt

### 11.4 Blobs

* 0x40 BLOB_PUT bytes_or_stream_ref,meta_node -> blobref_reg,receipt_reg
* 0x41 BLOB_GET blobref_reg,range_node_opt -> bytes_or_stream_ref,receipt_reg
* 0x42 BLOB_CHUNK_GET blobref_reg,chunk_index_reg -> bytes_ref,receipt_reg

### 11.5 Determinism Rule

Pure opcodes MUST NOT depend on wall-clock time, randomness, network, or external state.
Effects MUST be explicit and recorded.

---

## 12. Capabilities

### 12.1 Capability Object

A capability is a signed struct with fields:

* `cap_id` (Hash) computed from descriptor
* `issuer` (AgentID)
* `subject` (AgentID)
* `scope` (structured descriptor)
* `not_before` (Time)
* `expires_at` (Time)
* `delegable` (Bool)
* `attenuation_parent` (Hash?)
* `signature` (Sig)

### 12.2 Scope Descriptor

Scope MUST be structured, not free text. Examples:

* Blob.Read(CID, range)
* Blob.Write(namespace, max_size, media_allow)
* Tool.Call(tool_id, constraints)
* Train.Run(recipe_id, budget)
* Deploy.Promote(ring_id)

### 12.3 Enforcement

Runtimes MUST enforce:

* subject matches executing agent
* time bounds
* scope constraints
* attenuation chains

### 12.4 Revocation

Revocation MAY be supported via a revocation list reference. If supported, runtimes MUST check it for caps with `revocation_required=true`.

---

## 13. Blob Subsystem

### 13.1 BlobRef

BlobRef is a struct:

1. `cid` (Hash)
2. `size` (u64)
3. `media` (text)
4. `codec` (u8)
5. `chunks` (u32, 0 if unchunked)
6. `encryption` (EncryptRef, optional)
7. `provenance` (ProvenanceRef, optional)

### 13.2 Content Addressing

Blobs MUST be content addressed.

If unchunked:

* `cid = Hash(bytes)` after applying codec and encryption steps as defined by policy

If chunked:

* Blob is split into fixed-size chunks
* Each chunk hash is computed over canonical chunk bytes
* A Merkle DAG node format MUST be canonical and hashed
* `cid` is the Merkle root

### 13.3 Chunking

Default chunk size SHOULD be 4 MiB. Implementations MUST support chunk sizes from 256 KiB to 16 MiB.

### 13.4 Blob Capabilities

Blob operations MUST require caps:

* Blob.Read(cid, range)
* Blob.Write(namespace, max_size, media_allow)
* Blob.Pin(cid, ttl)
* Blob.Delete(cid)

### 13.5 Inline Blobs

Inline blobs MUST be limited to 8 KiB. Larger payloads MUST use BlobRef.

### 13.6 Encryption

Encryption MAY be applied. If used, EncryptRef MUST include:

* scheme (u8)
* key_handle (text)
* nonce (bytes)
* aad (bytes, optional)

The system MUST define whether `cid` hashes ciphertext or plaintext. v0.2 RECOMMENDS hashing ciphertext and optionally including `plain_hash` in provenance.

### 13.7 Provenance

ProvenanceRef MUST include:

* parent CIDs
* transform name
* transform parameters
* code hash of transform implementation

---

## 14. Receipts and Attestations

### 14.1 Receipt Object

Receipt is a typed struct:

* `receipt_id` (Hash)
* `tool_id` (ToolID)
* `request_hash` (Hash)
* `response_hash` (Hash)
* `idempotency_key` (bytes)
* `signer` (AgentID)
* `time_observed` (Time)
* `status` (u16)
* `evidence` (Hash list, optional)

receipt_id MUST be computed from canonical receipt bytes without the signature.

### 14.2 Attestation

An attestation is a signed claim:

* subject object hash
* claim type
* result status
* evidence hashes

Attestations MUST be verifiable and MUST reference immutable hashes.

---

## 15. State Sharding with Cells

### 15.1 Cell Model

A cell is an isolated state machine.

* Writes are events appended to the cell log.
* Reads use snapshots plus event replay.

### 15.2 Cell Addressing

Cell IDs MUST be stable and content independent. A typical choice is:

* CellID = Hash("cell" + namespace + key)

### 15.3 Consistency

Within a cell, writes MUST be serialized. Across cells, consistency is implementation-defined.

---

## 16. Learning Substrate

Learning is a first-class part of interoperability.

### 16.1 Learning Objects

The following objects MUST be supported:

* TraceRef: BlobRef to trace bytes plus receipt references
* EpisodeRef: TraceRef plus context hash plus outcome hash
* Signal: feedback attached to EpisodeRef
* DatasetRef: deterministic dataset definition plus a manifest CID
* RecipeRef: training configuration plus code hashes
* ModelRef: model artifact pointers
* EvalSuiteRef: tests and metrics definitions
* PromotionRecord: signed decision with evidence

### 16.2 Three-Layer Adaptation

Implementations SHOULD support layered learning:

1. Instant adaptation, reversible, low risk
2. Fast adaptation, small adapters, gated
3. Consolidation, batch updates, strict gates

This RFC mandates the gate model in Section 17 for any deployment promotion.

### 16.3 Dataset Determinism

DatasetRef MUST include:

* selection predicate expressed as a deterministic query over immutable logs
* sampling strategy and seed
* stratification rules
* time windows as explicit ranges
* input root hashes

Rebuilding a dataset MUST yield identical episode sets.

---

## 17. Training, Evaluation, Promotion

### 17.1 Training as Effect

Training MUST be an effect:

* TRAIN(recipe_ref, dataset_ref, budget) -> model_candidate_ref, train_receipt

Requires cap:

* Train.Run(recipe_id, budget)

### 17.2 Evaluation as Effect

Evaluation MUST be an effect:

* EVAL(model_ref, suite_ref) -> metrics_ref, eval_receipt

Requires cap:

* Eval.Run(suite_id)

### 17.3 Promotion as Effect

Promotion MUST be an effect:

* PROMOTE(model_ref, ring_id) -> promotion_receipt

Requires cap:

* Deploy.Promote(ring_id)

### 17.4 Promotion Contract

A runtime MUST NOT promote a model unless:

1. A required eval suite has passed for that ring
2. Critical metrics have not regressed beyond thresholds
3. At least one target metric improved or a policy objective was met
4. DatasetRef and RecipeRef are reproducible by hash
5. Rollback model is pinned and available
6. A quorum of verifier attestations is present

Thresholds and suites are policy objects, content-addressed and signed.

### 17.5 Rings

Rings MUST be monotonic.

* ring0 is canary
* higher rings require equal or stricter suites

---

## 18. Scheduler and Budgets

### 18.1 Task Budget Object

Tasks MUST declare budgets:

* cpu
* tokens (optional)
* io_count
* state_writes
* deadline

### 18.2 Enforcement

Runtimes MUST enforce budgets. Budget exhaustion MUST halt the task or fail closed.

### 18.3 Spawn Control

Any spawning of workers MUST be an effect requiring a Compute.Spawn cap with quotas.

---

## 19. Security Considerations

* Capabilities are the primary containment boundary. Implementations MUST deny effects without caps.
* Receipts and promotion records MUST be signed.
* Logs and datasets SHOULD support encryption at rest.
* Traces SHOULD support redaction transforms with provenance.
* Negotiated codebooks and encryption MUST be explicitly accepted and signed.

---

## 20. Privacy Considerations

* Episode logs can contain sensitive content. Implementations SHOULD minimize captured payloads.
* Datasets SHOULD be built from redacted episodes when feasible.
* Access to learning objects MUST be capability-gated.

---

## 21. Interoperability and Conformance

A v0.2 compliant runtime MUST support:

1. MYTHOS-X packet parsing and signature verification
2. CodecID=1 MYTHOS-CAN payload decoding
3. Baseline CodebookID
4. Canonical hashing with SHA-256
5. Minimum opcode set in Section 11
6. Capability enforcement in Section 12
7. Receipt creation in Section 14
8. BlobRef operations and Merkle chunking in Section 13
9. Learning objects in Section 16
10. Promotion contract enforcement in Section 17

Implementations SHOULD provide:

* a decompiler for MYTHOS-IR
* test vectors for hashing and encoding
* conformance tests for receipts and promotion gating

---

## 22. Implementation Notes (Non-Normative)

A practical v0.2 build strategy is IR-first:

1. Implement canonical encoding and hashing
2. Implement IR executor and effect receipts
3. Implement blob CAS with chunking
4. Implement logs, traces, and signals
5. Implement dataset builder, training and eval effects
6. Implement promotion records and ring rollout

---

## 23. Appendix A, Required Struct Field Numbers

This appendix is reserved for the definitive field numbering tables for each struct in v0.2. Implementations MUST use the published field numbers for wire compatibility.

---

## 24. Appendix B, Baseline Codebook

This appendix is reserved for the baseline codebook entries for v0.2.
