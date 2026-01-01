# MYTHOS v0.2

## An AI‑First Executable Language, Protocol, and Learning Substrate for Infinite-Scale Multi‑Agent Systems

**Status:** Draft Whitepaper (v0.2)
**Audience:** Systems engineers, AI platform builders, security and governance teams
**Scope:** Execution semantics, wire protocol, blob handling, learning substrate, safety and promotion gates

---

## Abstract

MYTHOS is an AI‑first language that unifies executable code, agent-to-agent communication, and verifiable learning into a single, content-addressed artifact format. MYTHOS is designed for infinite horizontal scaling by making determinism, explicit side effects, and reproducible training first-class. Programs and messages compile to a compact intermediate representation (MYTHOS‑IR) and are exchanged as dense binary packets (MYTHOS‑X) optimized for machine transfer rather than human readability. Human tooling exists as an optional layer, similar to how assemblers and debuggers exist for CPU binaries.

MYTHOS provides a bulletproof foundation for self-improving systems by turning execution traces, feedback signals, datasets, training jobs, evaluations, and promotions into typed, signed, hash-addressed objects. It enables immediate adaptation through safe, reversible learning layers, while enforcing strict promotion contracts for heavier updates.

---

## 1. Problem Statement

Multi-agent AI systems struggle to scale because they mix:

* Informal language and ambiguous messages
* Untracked tool calls and side effects
* Ad hoc data pipelines for training
* Weak provenance and limited auditability
* Unbounded recursion and unsafe self-modification

This leads to brittle systems that cannot reliably:

* Reproduce results
* Prove correctness or policy compliance
* Learn continuously without drift and regressions
* Scale across nodes without central coordination

MYTHOS addresses these failures by specifying a machine-native substrate where code, communication, data, and learning are all deterministic, typed, capability-gated, and verifiable.

---

## 2. Design Goals

### 2.1 Machine-first, not human-first

* Agents exchange dense IDs, opcodes, and hashes.
* Names and prose are metadata, not semantics.
* Humans can decode with tooling, but the system does not depend on readability.

### 2.2 Determinism and replay

* Pure computation is replayable from hashes.
* Side effects are explicit and produce signed receipts.

### 2.3 Infinite scale by construction

* Content addressing enables deduplication, caching, and distribution.
* State sharding via cells removes global locks.
* Budgets and capability boundaries keep systems stable under load.

### 2.4 Continuous learning without runaway

* Learning is native, auditable, and reproducible.
* Immediate adaptation is safe and reversible.
* Promotion gates prevent silent regressions.

---

## 3. Core Thesis

A scalable AI system needs the equivalent of:

* A CPU instruction set (execution)
* An OS process model (budgets and isolation)
* A network protocol (communication)
* A filesystem (content-addressed blobs)
* A CI pipeline and registry (evaluation and promotion)

MYTHOS provides all of these through one canonical artifact model.

---

## 4. The MYTHOS Stack

MYTHOS uses three representations, like source code, assembly, and machine code.

### 4.1 MYTHOS‑S (optional)

Human-friendly authoring format. Useful for governance, debugging, and hand-written modules.

### 4.2 MYTHOS‑IR (canonical meaning)

A typed DAG plus a compact instruction stream. All references are IDs, not names.

### 4.3 MYTHOS‑X (wire exchange)

A binary encoding of IR with signatures, receipts, and optional encryption. Optimized for agent-to-agent transfer.

**Key rule:** Only IR and X are normative for execution and interoperability.

---

## 5. Foundational Invariants

These invariants are mandatory across all runtimes and implementations.

1. **Deterministic core:** Pure ops are replayable.
2. **Explicit effects:** All I/O uses `EFFECT` and emits receipts.
3. **Capability-only authority:** No ambient file, network, money, identity, or training rights.
4. **Immutable logs:** Append-only event and episode logs.
5. **Content addressing:** Code, data, models, policies, and messages are hash-addressed.
6. **Promotion gates:** Model or policy changes require eval attestations.
7. **Budgeted recursion:** Every task has enforceable compute, I/O, and learning budgets.

---

## 6. Artifact Model

A MYTHOS bundle contains typed sections, each content-addressed.

* `meta` identity, version, signatures
* `types` canonical schema definitions
* `facts` immutable typed statements
* `contracts` executable requirements and guarantees
* `modules` functions and IR bodies
* `messages` typed agent communications
* `tasks` runnable instances with budgets

All sections have canonical ordering and serialization to support stable hashing.

---

## 7. Execution Semantics

### 7.1 Pure vs effectful

* **Pure ops:** cannot access time, random, network, or external state.
* **Effect ops:** must declare capability requirements and produce receipts.

### 7.2 Budgets

Every task includes budgets, enforced by the runtime scheduler.

* `cpu` maximum compute time
* `tokens` maximum reasoning budget, if applicable
* `io` maximum tool calls or external actions
* `state` maximum persisted state writes
* `deadline` wall-clock timeouts for effects

### 7.3 Receipts

Receipts are typed facts that prove an effect occurred.
They include:

* op type and tool ID
* request hash and response hash
* signer identity
* idempotency key
* timestamps as effect facts, not for determinism

Receipts enable replay, audit, caching, and training provenance.

---

## 8. MYTHOS‑IR Overview

MYTHOS‑IR is a minimal, machine-first execution format.

### 8.1 Structure

* Constant pool
* Node pool (typed DAG nodes)
* Op stream (bytecode-like instructions)
* Effect declarations (tool IDs, cap IDs)
* Contract references

### 8.2 Identifiers

* TypeID = hash(canonical type definition)
* SymbolID = hash(canonical module + symbol signature)
* ContractID = hash(canonical contract)
* ToolID = hash(tool interface definition)
* CapID = hash(capability descriptor)

Names are optional metadata and never required for execution.

### 8.3 Minimal instruction set

A v0.2 reference runtime can implement a small set of opcodes, grouped into:

* control flow
* data construction and pattern matching
* capability enforcement
* effect invocation
* assertions and contract checks

This enables deterministic replay and portable execution.

---

## 9. MYTHOS‑X Wire Protocol

MYTHOS‑X is a binary packet format for agent exchange.

### 9.1 Packet framing

* magic, version
* codec ID
* codebook ID
* payload length
* payload bytes
* signature block

### 9.2 Codebooks

Codebooks map compact codewords to IR atoms and op fragments.
They provide:

* dense transfer over the wire
* stable decoding across runtimes
* optional tokenizer-aware packing for LLM channels

A standard baseline codebook is required for interoperability.
Model-tuned codebooks are optional, negotiated, and always include a fallback decode path.

### 9.3 Encryption

Payloads can be encrypted. Semantics remain verifiable through:

* signed headers
* receipt hashes
* optional plaintext identity hashes

---

## 10. Blobs and Large Data

Blobs are never semantics. They are content-addressed objects referenced by `BlobRef`.

### 10.1 BlobRef

A BlobRef contains:

* `cid` content hash or merkle root
* `size` bytes
* `media` MIME type
* `codec` compression or container
* `chunks` optional chunk count
* `encryption` optional encryption descriptor
* `provenance` optional lineage descriptor

### 10.2 Content addressing and Merkle chunking

Large blobs use fixed-size chunks and a merkle DAG.
This supports streaming, partial fetch, resumable transfer, and integrity verification.

### 10.3 Blob capabilities

Blob read and write are capability-gated.

* Read can be limited by CID and byte range.
* Write can be limited by namespace, max size, and MIME allowlist.
* Pinning and deletion are separate privileges.

### 10.4 Provenance

Derived artifacts must include lineage:

* parent CIDs
* transform name and parameters
* code hash of transform implementation

This makes datasets and training reproducible.

---

## 11. Learning Substrate

Learning is a native pipeline in MYTHOS, not an external process.

### 11.1 First-class learning objects

* **TraceRef:** a blob containing deterministic execution trace and receipts
* **EpisodeRef:** trace + context + outcome
* **Signal:** reward, label, constraint feedback for an episode
* **DatasetRef:** a deterministic set of episodes, defined by a query over logs
* **RecipeRef:** training configuration and code hashes
* **ModelRef:** content-addressed model artifact
* **EvalSuiteRef:** executable tests and metrics
* **PromotionRecord:** signed decision for deployment

### 11.2 Three-layer adaptation model

MYTHOS supports continuous improvement without unsafe drift.

1. **Instant adaptation** (always-on, reversible)

   * caching and retrieval index updates
   * memory writes to bounded stores
   * codebook tuning and routing heuristics

2. **Fast adaptation** (near-real-time, gated)

   * small adapters and routing heads
   * trained from recent episodes
   * requires eval suite pass for ring0 deployment

3. **Consolidation** (batch, strongest gates)

   * distillation or larger fine-tunes
   * full regression suites
   * gradual promotion across rings

### 11.3 Reproducible datasets

Datasets are defined by deterministic queries over immutable logs.
A dataset definition includes:

* selection predicate
* sampling strategy
* stratification rules
* time window as explicit ranges
* all input hashes

Dataset materialization yields a DatasetRef that can be rebuilt exactly.

---

## 12. Training, Evaluation, and Promotion as Effects

Training is an effect with receipts and capabilities.

### 12.1 Training effect

* `TRAIN(recipeRef, datasetRef) -> modelCandidateRef, trainReceipt`

Requires:

* `cap Train.Run(recipeId, budget)`

### 12.2 Evaluation effect

* `EVAL(modelRef, suiteRef) -> metricsRef, evalReceipt`

Requires:

* `cap Eval.Run(suiteId)`

### 12.3 Promotion effect

* `PROMOTE(modelRef, ringId) -> promotionReceipt`

Requires:

* `cap Deploy.Promote(ringId)`
* quorum attestations from verifiers

### 12.4 Promotion contracts

A candidate can be promoted only if:

* critical metrics do not regress
* at least one target metric improves beyond threshold
* dataset and recipe are reproducible by hash
* rollback model is pinned and available

Promotion records are signed and stored as immutable facts.

---

## 13. Scalability Architecture

### 13.1 Cells for state sharding

State is stored in independent cells.

* Each cell is an isolated state machine with an append-only log.
* Writes serialize per cell, avoiding global locks.
* Reads use immutable facts and snapshots.

### 13.2 Work distribution

Schedulers operate like an OS:

* tasks declare budgets and required capabilities
* work stealing balances load
* pure computations memoize by hash
* effect receipts provide idempotency

### 13.3 Infinite horizontal scaling

With more nodes, the system scales by:

* distributing tasks and effect calls
* caching pure results
* streaming blobs by chunk
* building datasets from shared immutable logs

---

## 14. Self-improvement Without Chaos

MYTHOS separates improvement into two safe lanes.

### 14.1 Model improvement lane

* traces and signals create datasets
* training produces candidates
* evaluation produces attestations
* promotion gates allow deployment in rings

### 14.2 Code and policy improvement lane

Changes to modules or policies happen via patch proposals.
A patch is a content-addressed diff with:

* tests as executable tasks
* contract preservation or strengthening
* performance claims and budgets
* verifier quorum signoff

No runtime accepts unproven self-modifications.

---

## 15. Security and Governance

### 15.1 Capabilities

Capabilities are least-privilege tokens.
They are:

* time-limited
* scope-limited
* delegable with attenuation
* revocable

### 15.2 Identity and signatures

Agents sign:

* MYTHOS bundles
* receipts
* attestations
* promotion records

### 15.3 Audit and explainability

Humans can decode MYTHOS‑X with tooling.
Auditability is preserved through:

* immutable logs
* signed receipts
* reproducible datasets
* contract checks and attestations

---

## 16. Interoperability

A compliant MYTHOS ecosystem includes:

* a reference canonicalizer and hasher
* baseline codebook and decoder
* minimum opcode set for IR
* standard receipt format
* standard learning objects and promotion contracts

Multiple runtimes can interoperate by exchanging MYTHOS‑X packets and verifying signatures and hashes.

---

## 17. Reference Implementation Plan

### Phase 1, Two to four weeks

* implement canonical encoding and hashing
* implement IR executor with pure ops and effect receipts
* implement MYTHOS‑X framing and signatures
* implement blob store with CID and merkle chunking

### Phase 2, Four to eight weeks

* implement cells and append-only logs
* implement trace and episode emission
* implement signal ingestion
* implement dataset builder from deterministic queries

### Phase 3, Eight to twelve weeks

* implement training and evaluation effects
* implement promotion records and ring deployment
* implement verifier agents and quorum policy

### Phase 4, Ongoing

* add optional human-friendly source and decompiler
* add model-tuned codebooks
* expand opcode set and standard libraries

---

## 18. Risks and Mitigations

### Risk: model drift from constant updates

Mitigation: three-layer adaptation model, mandatory eval and rings.

### Risk: runaway task spawning

Mitigation: capability-gated spawn with quotas and budgets.

### Risk: unverifiable external tool outputs

Mitigation: receipts with request and response hashes, attestation policies, redundancy checks.

### Risk: privacy leakage in traces and datasets

Mitigation: encrypted blobs, capability limits, data minimization policies, dataset redaction transforms with provenance.

---

## 19. Conclusion

MYTHOS makes AI systems scalable by treating execution, communication, and learning as a single verifiable substrate. It removes ambiguity through typed IR and content addressing, prevents runaway autonomy through capability gating and budgets, and enables continuous improvement through native learning objects and promotion contracts. The result is an AI-first protocol that can scale horizontally without sacrificing reproducibility, auditability, or operational safety.

---

## Appendix A, Canonical Object Types (v0.2)

### A.1 BlobRef

* cid: Hash
* size: Int
* media: Text
* codec: Enum
* chunks: Int?
* encryption: EncryptRef?
* provenance: ProvenanceRef?

### A.2 TraceRef and EpisodeRef

* TraceRef: BlobRef of trace bytes + receipt list
* EpisodeRef: TraceRef + context hash + outcome hash

### A.3 Signal

* episodeCid: Hash
* signalType: Enum("reward","label","constraint")
* value: Number|Text|Struct
* signer: Agent
* signed: Signature

### A.4 DatasetRef

* definitionHash: Hash
* manifestCid: Hash
* episodeCids: merkle list CID

### A.5 ModelRef

* modelCid: Hash
* configCid: Hash
* codeHash: Hash
* parentModelCid: Hash?

---

## Appendix B, Minimum Op Families

* Control flow ops
* Data construction ops
* Capability ops
* Effect ops
* Blob ops
* Assertion ops

---

## Appendix C, Glossary

* **Artifact:** a content-addressed MYTHOS bundle.
* **Capability:** a least-privilege authorization token.
* **Receipt:** signed proof of an effect.
* **Attestation:** signed claim that a contract or evaluation passed.
* **Cell:** shardable state machine with an append-only log.
* **Ring deployment:** staged rollout levels from canary to full.
* **Codebook:** mapping from compact codewords to IR atoms.
