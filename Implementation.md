# MYTHOS v0.2

## Technical Implementation Paper

**Purpose:** This paper translates the MYTHOS RFC set (0001 through 0005) into an implementable system blueprint. It describes the minimum components, interfaces, data stores, and test strategy required to ship a compliant runtime and a working learning loop.

---

## 1. Implementation Principles

1. **IR-first delivery.** Implement MYTHOS-CAN, hashing, MYTHOS-IR, MYTHOS-X, receipts, capabilities, blobs, and ledger semantics before any human-friendly surface syntax.
2. **Determinism by default.** Any non-deterministic action is an EFFECT with a receipt.
3. **Content addressing everywhere.** Every meaningful object is hash-addressed and reproducible.
4. **Fail closed.** Divergent ledgers, unverifiable receipts, unknown codebooks, or cap violations halt execution.
5. **Interop test vectors are the contract.** If two implementations match the vectors, they interoperate.

---

## 2. Minimum Viable System, End-to-End

A minimal MYTHOS deployment that is "real" includes:

### 2.1 Libraries

* **mythos-can:** canonical encoder/decoder (MYTHOS-CAN), varints, map sorting.
* **mythos-hash:** SHA-256 hashing and ID constructors.
* **mythos-x:** wire framing, signature verification, codec handling.
* **mythos-ir:** IR bundle parser, validator, executor (minimum opcodes).
* **mythos-caps:** cap parsing, signature verification, scope checks, attenuation chain validation.
* **mythos-receipts:** receipt construction, signing, verification.
* **mythos-merkle:** MerkleList and ChunkedBlob DAG encoding per RFC 0004.

### 2.2 Services

* **Executor:** runs tasks, enforces budgets, enforces caps, emits receipts, optionally emits traces.
* **Tool Gateway:** implements EFFECT execution for tools. Produces tool request/response hashes.
* **Blob Store (CAS):** stores chunks and Merkle nodes, returns BlobRef.
* **Receipt Ledger:** append-only register/commit/divergence records per RFC 0005.

### 2.3 Optional but recommended early

* **Policy Registry:** signed policy objects (eval suites required per ring, thresholds, retry policy).
* **Verifier Service:** validates receipts, checks contracts, issues attestations.

---

## 3. Reference Architecture

### 3.1 Control plane vs data plane

**Control plane**

* Policy Registry (signed, content-addressed)
* Codebook Registry (baseline, negotiated)
* Capability Authority (cap issuance and revocation policy)
* Model Registry (ModelRef, PromotionRecord)

**Data plane**

* Executors (scale out)
* Tool Gateways (scale out)
* Blob CAS nodes (scale out)
* Receipt ledger shards (scale out)
* Cell store shards (optional at first, required for stateful workflows)

### 3.2 Deployment topology

* Start with a single region cluster.
* Run N Executors behind a queue.
* Tool Gateway is stateless. Scale horizontally.
* Blob CAS uses an object store for chunks plus a small KV store for Merkle nodes.
* Receipt ledger shards by IdempotencyID.

---

## 4. Data Stores

### 4.1 Blob CAS

Store types:

* **Chunks:** raw bytes keyed by chunk_hash.
* **Merkle nodes:** canonical MYTHOS-CAN bytes keyed by node CID.

Recommended backing stores:

* Chunks in S3, Azure Blob, or GCS. Key is hex(sha256).
* Merkle nodes in RocksDB, DynamoDB, Cosmos DB, or Postgres KV table.

Required operations:

* PutChunk(hash, bytes)
* GetChunk(hash)
* PutNode(cid, bytes)
* GetNode(cid)
* PutBlob(manifest or unchunked bytes) returns BlobRef

### 4.2 Receipt ledger

Ledger is append-only.

Recommended implementation:

* Cell per IdempotencyID (logical). Backed by a KV store with append semantics.
* Alternative, a log store like Kafka plus a compaction view. Only if you can guarantee deterministic reads by key.

Ledger operations:

* Register(IdempotencyID, request_hash, owner, lease)
* Commit(IdempotencyID, receipt_id, request_hash, response_hash)
* MarkDivergent(IdempotencyID, reason)
* Read(IdempotencyID) returns latest state plus history

### 4.3 Cells (state, if needed)

If you implement stateful processes, add a cell store.

Minimal cell features:

* Append event
* Read snapshot
* Replay events from offset
* Snapshotting policy

---

## 5. Protocol and Interface Surface

### 5.1 Agent to agent

Agents exchange MYTHOS-X packets.

Required steps on receive:

1. parse framing
2. verify CodebookID support
3. decode payload (CodecID)
4. verify signatures
5. validate IR bundle (canonical references, opcode validity)
6. enqueue tasks

### 5.2 Executor API (internal)

Executor needs at least these calls. Use HTTP, gRPC, or in-process. The RFCs do not constrain transport.

* SubmitIRBundle(bundle_bytes) -> ack
* ExecuteTask(task_id) -> result nodes, receipts, trace ref optional

### 5.3 Tool Gateway API

* InvokeTool(tool_id, request_bytes, idempotency_key) -> response_bytes, status

Tool Gateway MUST:

* compute request_hash and response_hash
* return enough material for receipt creation

### 5.4 Blob API

* PutBlob(stream, media, codec, encryption?) -> BlobRef
* GetBlob(BlobRef, range?) -> stream
* GetProof(BlobRef, chunk_index) -> proof nodes for verification

---

## 6. Execution Pipeline

### 6.1 Task lifecycle

1. Validate task budget.
2. Load caps for task context.
3. Execute MYTHOS-IR op stream.
4. On REQUIRE_CAP, validate cap scope.
5. On EFFECT:

   * if idempotency_key present, apply RFC 0005 ledger flow
   * call Tool Gateway
   * build receipt
   * commit to ledger
6. On BLOB_PUT or BLOB_GET:

   * use Blob caps
   * enforce chunk verification
7. Optionally emit trace and episode objects.

### 6.2 Determinism boundaries

* Pure ops are replayable.
* Receipts are facts.
* Training and evaluation are effects.

---

## 7. Learning Loop Implementation

### 7.1 What is "immediate learning" in practice

Implement in layers.

Layer 1, Instant adaptation, ship first

* Retrieval index updates. Embedding and vector store updates are an effect with receipts.
* Caching. Memoize pure results by hash.
* Codebook tuning. Optional. Store negotiated codebooks as signed objects.

Layer 2, Fast adaptation

* Adapter training job produces a ModelRef candidate.
* Eval suite runs, produces metric attestations.
* ring0 promotion only.

Layer 3, Consolidation

* Batch distillation or larger fine tune.
* Stronger eval suites and progressive ring promotion.

### 7.2 Episode pipeline

* Executor emits TraceRef for selected tasks.
* A scorer service emits Signal objects.
* Dataset builder uses RFC 0003 to produce DatasetRef.

### 7.3 Training and eval integration

Training is a tool.

* Train tool takes RecipeDef, DatasetRef, budget.
* Eval tool takes ModelRef and EvalSuiteDef.

Both tools MUST produce receipts.

---

## 8. Capability Authority

### 8.1 Issuance

Start with a single Capability Authority service that:

* signs caps
* enforces issuance policy
* supports attenuation

### 8.2 Revocation

v0.2 allows optional revocation.
If implemented:

* publish revocation roots as signed objects
* executor checks revocation_required caps

### 8.3 Operational guardrails

* Separate cap issuers by domain, blob, deploy, train.
* Rotate keys.
* Log every issuance as an effect.

---

## 9. Conformance Test Strategy

Interop requires golden vectors.

### 9.1 Golden vectors you MUST publish internally

* Canonical encoding vectors for each core struct.
* Hash vectors, TypeID, ContractID, ToolID.
* MerkleList root CID vectors.
* ChunkedBlob root CID vectors.
* Receipt id vectors.
* Ledger state machine vectors for retry, takeover, divergence.
* Dataset build vectors for RFC 0003.

### 9.2 Property tests

* Encode then decode equality.
* Hash stability under field order permutations.
* Merkle proof verification.
* Ledger idempotency invariants.

### 9.3 Fuzz tests

* MYTHOS-X framing and signature parsing.
* MYTHOS-CAN decoding.
* IR validator.

---

## 10. Security Hardening Checklist

* Reject unknown CodebookID unless negotiated.
* Enforce budgets strictly. No soft limits.
* Deny all effects without caps.
* Sign every receipt and attestation.
* Fail closed on divergent ledger records.
* Encrypt blobs and traces at rest when they contain sensitive data.
* Add redaction transforms for traces before dataset building.

---

## 11. Reference Repo Layout

Suggested monorepo layout:

* /spec, RFC text and test vectors
* /libs/mythos-can
* /libs/mythos-hash
* /libs/mythos-x
* /libs/mythos-ir
* /libs/mythos-merkle
* /libs/mythos-caps
* /libs/mythos-receipts
* /services/executor
* /services/tool-gateway
* /services/blob-cas
* /services/receipt-ledger
* /services/policy-registry
* /services/cap-authority
* /services/verifier
* /services/dataset-builder
* /services/train-worker
* /services/eval-worker
* /tests/conformance

---

## 12. MVP Definition

A system is MYTHOS v0.2 MVP when:

1. Two independent executors can exchange MYTHOS-X packets and run the same IR bundle.
2. All effects are cap gated and emit verifiable receipts.
3. Blobs are stored as chunked DAGs with proofs and verification.
4. Idempotent effects follow RFC 0005 ledger semantics.
5. Dataset builder produces deterministic manifests per RFC 0003.
6. ring0 promotion requires eval attestations.
