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

## 23. Appendix A, Required Struct Field Numbers (Normative)

This appendix defines the canonical field numbers for core structs in MYTHOS v0.2. All implementations MUST use these field numbers when encoding structs in MYTHOS-CAN.

### A.1 Hash

Struct `Hash` fields:

1. `alg` (u8)
2. `bytes` (bytes)

Hash algorithm ids:

* 1 SHA-256

### A.2 Time

Time is encoded as a signed integer `i64` representing **microseconds since Unix epoch (UTC)**.

### A.3 AgentID

Struct `AgentID` fields:

1. `scheme` (u8)
2. `key` (bytes)
3. `hint` (text, optional)

AgentID scheme ids:

* 1 Ed25519 public key

### A.4 Signature

Struct `Signature` fields:

1. `alg` (u8)
2. `key_id` (Hash, optional)
3. `sig_bytes` (bytes)

Signature algorithm ids:

* 1 Ed25519

### A.5 Meta

Struct `Meta` fields:

1. `spec` (text)
2. `version` (u16)
3. `created_at` (Time, optional)
4. `author` (AgentID)
5. `artifact_id` (Hash)
6. `signatures` (list(Signature))
7. `notes` (text, optional)
8. `extensions` (map(text, bytes), optional)

### A.6 EncryptRef

Struct `EncryptRef` fields:

1. `scheme` (u8)
2. `key_handle` (text)
3. `nonce` (bytes)
4. `aad` (bytes, optional)
5. `plain_hash` (Hash, optional)

Encryption scheme ids:

* 1 age
* 2 aes-gcm
* 3 kms-envelope

### A.7 ProvenanceRef

Struct `ProvenanceRef` fields:

1. `parents` (list(Hash))
2. `transform` (text)
3. `params` (map(text, text))
4. `code_hash` (Hash)
5. `time_observed` (Time, optional)

### A.8 BlobRef

Struct `BlobRef` fields:

1. `cid` (Hash)
2. `size` (u64)
3. `media` (text)
4. `codec` (u8)
5. `chunks` (u32)
6. `encryption` (EncryptRef, optional)
7. `provenance` (ProvenanceRef, optional)

Codec ids:

* 0 raw
* 1 zstd
* 2 gzip
* 3 jpeg
* 4 png
* 5 jsoncbor
* 6 parquet

### A.9 Receipt

Struct `Receipt` fields:

1. `receipt_id` (Hash)
2. `tool_id` (Hash)
3. `request_hash` (Hash)
4. `response_hash` (Hash)
5. `idempotency_key` (bytes)
6. `signer` (AgentID)
7. `time_observed` (Time)
8. `status` (u16)
9. `evidence` (list(Hash), optional)
10. `notes` (text, optional)
11. `signature` (Signature)

Receipt id computation (normative):

* `receipt_id = SHA-256( canonical_bytes(Receipt without fields 1 and 11) )`
* The signature signs `receipt_id.bytes`.

### A.10 Attestation

Struct `Attestation` fields:

1. `attestation_id` (Hash)
2. `subject_hash` (Hash)
3. `claim_type` (u8)
4. `result` (u16)
5. `evidence` (list(Hash), optional)
6. `signer` (AgentID)
7. `time_observed` (Time)
8. `signature` (Signature)
9. `notes` (text, optional)

Claim types:

* 1 ContractPass
* 2 EvalPass
* 3 EvalMetric
* 4 QuorumSignoff

Attestation id computation:

* `attestation_id = SHA-256( canonical_bytes(Attestation without fields 1 and 8) )`
* The signature signs `attestation_id.bytes`.

### A.11 Capability

Struct `Capability` fields:

1. `cap_id` (Hash)
2. `issuer` (AgentID)
3. `subject` (AgentID)
4. `scope` (CapScope)
5. `not_before` (Time)
6. `expires_at` (Time)
7. `delegable` (bool)
8. `attenuation_parent` (Hash, optional)
9. `revocation_required` (bool, optional, default false)
10. `signature` (Signature)

Capability id computation:

* `cap_id = SHA-256( canonical_bytes(Capability without fields 1 and 10) )`
* The signature signs `cap_id.bytes`.

### A.12 CapScope

Struct `CapScope` fields:

1. `kind` (u8)
2. `payload` (bytes, MYTHOS-CAN encoded nested scope struct)

CapScope kinds:

* 1 BlobRead
* 2 BlobWrite
* 3 BlobPin
* 4 BlobDelete
* 5 ToolCall
* 6 TrainRun
* 7 EvalRun
* 8 DeployPromote
* 9 ComputeSpawn
* 10 CellRead
* 11 CellWrite

Nested scope structs (encoded into `payload`):

**BlobReadScope** fields:

1. `cid` (Hash)
2. `range_start` (u64, optional)
3. `range_len` (u64, optional)

**BlobWriteScope** fields:

1. `namespace` (text)
2. `max_size` (u64)
3. `media_allow` (list(text))

**BlobPinScope** fields:

1. `cid` (Hash)
2. `ttl_us` (u64)

**BlobDeleteScope** fields:

1. `cid` (Hash)

**ToolCallScope** fields:

1. `tool_id` (Hash)
2. `max_calls` (u32)
3. `constraints` (map(text, text), optional)

**TrainRunScope** fields:

1. `recipe_id` (Hash)
2. `max_jobs` (u32)
3. `max_tokens` (u64)
4. `max_gpu_ms` (u64, optional)

**EvalRunScope** fields:

1. `suite_id` (Hash)
2. `max_runs` (u32)

**DeployPromoteScope** fields:

1. `ring_id` (u8)
2. `policy_id` (Hash)

**ComputeSpawnScope** fields:

1. `pool_id` (Hash)
2. `max_workers` (u32)
3. `max_cpu_us` (u64)
4. `max_io` (u32)

**CellReadScope** fields:

1. `cell_id` (Hash)
2. `key_prefix` (bytes, optional)

**CellWriteScope** fields:

1. `cell_id` (Hash)
2. `max_events` (u32)
3. `event_type_allow` (list(u32), optional)

### A.13 MYTHOS-X SigBlockEntry

Struct `SigBlockEntry` fields:

1. `signer` (AgentID)
2. `sig_alg` (u8)
3. `sig_bytes` (bytes)

### A.14 MYTHOS-IR Core Structs

Struct `IRBundle` fields:

1. `meta` (Meta)
2. `const_pool` (ConstPool)
3. `node_pool` (NodePool)
4. `types` (TypeTable)
5. `symbols` (SymbolTable)
6. `contracts` (ContractTable)
7. `tools` (ToolTable)
8. `ops` (OpStream)
9. `tasks` (TaskTable)

Struct `ConstPool` fields:

1. `consts` (list(Const))

Struct `Const` fields:

1. `kind` (u8)
2. `i64` (i64, optional)
3. `u64` (u64, optional)
4. `bytes` (bytes, optional)
5. `text` (text, optional)
6. `hash` (Hash, optional)

Const kinds:

* 1 I64
* 2 U64
* 3 Bytes
* 4 Text
* 5 Hash

Struct `NodePool` fields:

1. `nodes` (list(Node))

Struct `Node` fields:

1. `type_id` (Hash)
2. `kind` (u8)
3. `fields` (map(u32, Operand), optional)
4. `items` (list(Operand), optional)

Node kinds:

* 1 Record
* 2 List

Struct `TypeTable` fields:

1. `types` (list(TypeDef))

Struct `TypeDef` fields:

1. `type_id` (Hash)
2. `kind` (u8)
3. `name` (text, optional)
4. `fields` (list(FieldDef), optional)
5. `enum_tags` (list(text), optional)

Type kinds:

* 1 Record
* 2 Enum

Struct `FieldDef` fields:

1. `field_id` (u32)
2. `name` (text, optional)
3. `field_type_id` (Hash)
4. `required` (bool)

Struct `SymbolTable` fields:

1. `symbols` (list(SymbolSig))

Struct `SymbolSig` fields:

1. `module_id` (Hash)
2. `name` (text, optional)
3. `arg_type_ids` (list(Hash))
4. `return_type_id` (Hash)

Struct `ContractTable` fields:

1. `contracts` (list(ContractDef))

Struct `ContractDef` fields:

1. `contract_id` (Hash)
2. `name` (text, optional)
3. `requires_expr` (Expr)
4. `ensures_expr` (Expr)
5. `invariant_expr` (Expr, optional)

Struct `ToolTable` fields:

1. `tools` (list(ToolDef))

Struct `ToolDef` fields:

1. `tool_id` (Hash)
2. `name` (text, optional)
3. `request_type_id` (Hash)
4. `response_type_id` (Hash)

Struct `OpStream` fields:

1. `ops` (list(Op))

Struct `Op` fields:

1. `opcode` (u8)
2. `operands` (list(Operand))

Struct `Operand` fields:

1. `kind` (u8)
2. `u64` (u64, optional)
3. `i64` (i64, optional)
4. `hash` (Hash, optional)
5. `node_ref` (u32, optional)
6. `const_ref` (u32, optional)

Operand kinds:

* 1 Reg (u64 holds register index)
* 2 ConstRef (const_ref)
* 3 NodeRef (node_ref)
* 4 U64 (u64)
* 5 I64 (i64)
* 6 Hash (hash)

Struct `TaskTable` fields:

1. `tasks` (list(TaskDef))

Struct `TaskDef` fields:

1. `task_id` (Hash)
2. `entry_symbol` (Hash)
3. `args` (list(Operand))
4. `budget` (Budget)
5. `emit_trace` (bool)
6. `emit_attestation` (bool)

Struct `Budget` fields:

1. `cpu_us` (u64)
2. `tokens` (u64, optional)
3. `io_count` (u32)
4. `state_writes` (u32)
5. `deadline_us` (u64, optional)
6. `max_depth` (u16, optional)
7. `max_spawn` (u16, optional)

### A.15 Learning Structs

Struct `TraceRef` fields:

1. `trace_blob` (BlobRef)
2. `receipt_ids` (list(Hash))

Struct `EpisodeRef` fields:

1. `episode_id` (Hash)
2. `trace_ref` (TraceRef)
3. `context_hash` (Hash)
4. `outcome_hash` (Hash)
5. `time_observed` (Time)

Episode id computation:

* `episode_id = SHA-256( canonical_bytes(EpisodeRef without field 1) )`

Struct `Signal` fields:

1. `signal_id` (Hash)
2. `episode_id` (Hash)
3. `signal_type` (u8)
4. `value_kind` (u8)
5. `value_bytes` (bytes)
6. `signer` (AgentID)
7. `time_observed` (Time)
8. `signature` (Signature)

Signal types:

* 1 Reward
* 2 Label
* 3 Constraint

Struct `DatasetDef` fields:

1. `dataset_def_id` (Hash)
2. `corpus_roots` (list(Hash))
3. `query` (QueryDef)
4. `sampling` (SamplingDef)
5. `stratify` (StratifyDef, optional)

Struct `DatasetRef` fields:

1. `dataset_def_id` (Hash)
2. `manifest` (BlobRef)
3. `count` (u64)
4. `build_receipt_id` (Hash)

Struct `RecipeDef` fields:

1. `recipe_id` (Hash)
2. `base_model` (Hash)
3. `train_code_hash` (Hash)
4. `hyperparams` (map(text, text))

Struct `ModelRef` fields:

1. `model_id` (Hash)
2. `weights` (BlobRef)
3. `config` (BlobRef)
4. `code_hash` (Hash)
5. `parent_model_id` (Hash, optional)

Struct `EvalSuiteDef` fields:

1. `suite_id` (Hash)
2. `suite_code_hash` (Hash)
3. `tests_manifest` (BlobRef)
4. `metrics` (list(text))

Struct `PromotionRecord` fields:

1. `promotion_id` (Hash)
2. `model_id` (Hash)
3. `ring_id` (u8)
4. `policy_id` (Hash)
5. `eval_attestations` (list(Hash))
6. `signer` (AgentID)
7. `time_observed` (Time)
8. `signature` (Signature)

---

## 24. Appendix B, Baseline Codebook (Normative)

This appendix defines the baseline codebook used by MYTHOS-X when codebook compression is enabled. For CodecID=1 (raw MYTHOS-CAN IRBundle), the baseline codebook MUST still be declared in the packet header unless a negotiated codebook is in effect.

### B.1 CodebookEntry

Struct `CodebookEntry` fields:

1. `codeword` (u16)
2. `kind` (u8)
3. `value_u64` (u64, optional)
4. `value_bytes` (bytes, optional)
5. `value_hash` (Hash, optional)
6. `value_text` (text, optional)

Codebook entry kinds:

* 1 Opcode
* 2 SmallInt
* 3 FieldNum
* 4 EnumTag

### B.2 Codeword Ranges

Baseline ranges are reserved as:

* 0x0100 to 0x01FF, opcodes, codeword = 0x0100 + opcode
* 0x0200 to 0x02FF, small integers 0 to 255, codeword = 0x0200 + n
* 0x0300 to 0x03FF, common field numbers 0 to 255, codeword = 0x0300 + field
* 0x0400 to 0x04FF, enum tags for CapScope kinds, codeword = 0x0400 + kind

### B.3 Opcode Entries

For each opcode `op` in Section 11, the baseline codebook contains:

* codeword = 0x0100 + op
* kind = Opcode
* value_u64 = op

### B.4 SmallInt Entries

For each integer n in 0..255:

* codeword = 0x0200 + n
* kind = SmallInt
* value_u64 = n

### B.5 FieldNum Entries

For each field number f in 0..255:

* codeword = 0x0300 + f
* kind = FieldNum
* value_u64 = f

### B.6 CapScope Kind Entries

For each CapScope kind k in Section 12.2:

* codeword = 0x0400 + k
* kind = EnumTag
* value_u64 = k

### B.7 Baseline CodebookID

Baseline CodebookID is computed as:

* `CodebookID = SHA-256( canonical_bytes( list(CodebookEntry) ) )`

The entry list MUST be ordered by ascending `codeword`.

---

# RFC-MYTHOS-0003

## Deterministic Dataset Query and Sampling (v0.2)

**Category:** Standards Track (Draft)
**Intended Status:** Proposed Standard
**Version:** 0.2

---

## 1. Abstract

This RFC specifies a deterministic dataset query language and sampling algorithm for MYTHOS learning objects.

Given:

* A fixed corpus of immutable episode logs (corpus roots)
* A DatasetDef that includes a QueryDef and SamplingDef

All compliant implementations MUST produce an identical DatasetRef manifest, byte-for-byte, including the same ordered list of EpisodeIDs.

---

## 2. Corpus Model

### 2.1 Corpus Roots

A dataset is built from a set of corpus roots:

* `corpus_roots` is a list of Hash values.

Each corpus root identifies a canonical episode list, encoded as a Merkle list of EpisodeIDs.

### 2.2 EpisodeID

EpisodeID is `episode_id` from `EpisodeRef` (Appendix A.15).

### 2.3 Canonical Episode Order

Within a corpus root, EpisodeIDs MUST be ordered ascending by canonical EpisodeID bytes.

The dataset build algorithm MUST preserve a deterministic order:

1. Corpus roots are ordered ascending by canonical hash bytes.
2. EpisodeIDs are streamed in that corpus order.
3. After filtering and sampling, the final manifest is ordered ascending by EpisodeID bytes.

---

## 3. DatasetDef and Manifest

### 3.1 DatasetDef Canonicalization

`DatasetDef.dataset_def_id` MUST be computed as SHA-256 of canonical DatasetDef bytes excluding field 1.

### 3.2 Manifest

A dataset manifest is a BlobRef whose blob bytes are a canonical Merkle list of EpisodeIDs, in ascending order.

The manifest BlobRef MUST have:

* media = `application/mythos.episode.manifest`
* codec = raw or zstd

---

## 4. QueryDef

### 4.1 QueryDef Fields

Struct `QueryDef` fields:

1. `query_id` (Hash)
2. `predicate` (Expr)

Query id computation:

* `query_id = SHA-256( canonical_bytes(QueryDef without field 1) )`

### 4.2 Expr

Expr is a deterministic AST.

Struct `Expr` fields:

1. `op` (u8)
2. `args` (list(Expr), optional)
3. `path` (FieldPath, optional)
4. `lit` (Operand, optional)
5. `set` (list(Operand), optional)

Expr ops:

* 1 TRUE
* 2 FALSE
* 10 EQ
* 11 NE
* 12 LT
* 13 LE
* 14 GT
* 15 GE
* 20 AND
* 21 OR
* 22 NOT
* 30 IN_SET
* 40 HAS_TOOL
* 41 HAS_STATUS

### 4.3 FieldPath

Struct `FieldPath` fields:

1. `root` (u8)
2. `segments` (list(u32))

Roots:

* 1 Episode
* 2 Signal

Segments are field numbers within the corresponding struct schema.

### 4.4 Evaluation Semantics

An expression evaluates over an EpisodeRef and its attached Signals.

* Field reads MUST return a canonical "missing" if any segment is absent.
* Comparisons against missing MUST evaluate as FALSE, except NE which is TRUE if the literal is present.
* `HAS_TOOL(tool_id)` is TRUE if any Receipt in the Episode trace references that tool_id.
* `HAS_STATUS(status)` is TRUE if any Receipt status equals the provided u16.

Regex and locale-dependent string operations are NOT allowed in v0.2.

---

## 5. SamplingDef

### 5.1 SamplingDef Fields

Struct `SamplingDef` fields:

1. `mode` (u8)
2. `n` (u64, optional)
3. `seed` (Hash)

Sampling modes:

* 1 ALL
* 2 FIRST_N
* 3 HASH_N

### 5.2 FIRST_N

FIRST_N selects the first N EpisodeIDs after filtering, in streaming corpus order.

### 5.3 HASH_N

HASH_N selects N EpisodeIDs with the lowest deterministic scores.

Score definition:

* `score(id) = U64( first 8 bytes of SHA-256( seed.bytes || id.bytes ) )`

Selection:

* Compute score for every candidate.
* Select the N smallest scores.
* Ties are broken by EpisodeID bytes ascending.

This algorithm is deterministic and order independent.

---

## 6. StratifyDef

### 6.1 StratifyDef Fields

Struct `StratifyDef` fields:

1. `key_path` (FieldPath)
2. `per_bucket_n` (u64)
3. `seed` (Hash)

### 6.2 Semantics

Stratification partitions candidates by key value read from `key_path`.

For each bucket:

* Define bucket seed as `SHA-256( seed.bytes || canonical_bytes(bucket_key) )`
* Apply HASH_N with N = per_bucket_n within that bucket

The final manifest is the union of bucket selections, ordered by EpisodeID ascending.

Buckets with missing keys MUST be treated as a single bucket with key = empty bytes.

---

## 7. Dataset Build Algorithm (Normative)

Given DatasetDef:

1. Load and stream EpisodeIDs from each corpus root in canonical corpus order.
2. For each EpisodeID, load EpisodeRef and attached Signals as needed to evaluate the predicate.
3. Filter by predicate.
4. If StratifyDef is present, partition candidates by key and sample per bucket.
5. Else, sample using SamplingDef.
6. Sort selected EpisodeIDs ascending.
7. Write manifest as a Merkle list blob.
8. Emit a build receipt and return DatasetRef.

Implementations MAY cache EpisodeRef and Signal retrieval. Caching MUST NOT affect results.

---

## 8. Conformance Tests (Recommended)

Implementations SHOULD publish:

* test corpora roots (episode lists)
* dataset definitions
* expected manifest hashes

---

## 9. Security Considerations

Dataset queries can leak sensitive attributes through selection. Access to corpus roots and dataset construction MUST be capability-gated.

---

## 10. Privacy Considerations

Datasets SHOULD prefer redacted episodes and minimized traces when feasible.
