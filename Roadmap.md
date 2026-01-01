# MYTHOS v0.2

## Roadmap Paper

**Purpose:** This roadmap turns the MYTHOS RFCs into a sequence of deliverable milestones, with clear acceptance criteria, dependencies, and staffing guidance. The objective is a working, production-hardened MYTHOS substrate that supports agent execution, verifiable effects, blob handling, deterministic datasets, and gated continuous improvement.

---

## 1. Roadmap Strategy

Ship in thin vertical slices.

1. Interop foundation.
2. Executor and effects.
3. Blobs and ledger correctness.
4. Deterministic dataset builder.
5. Learning loop with ring0 canary.
6. Scale, security hardening, and operational maturity.

Keep human-friendly syntax optional until late.

---

## 2. Milestones and Phases

### Phase 0. Spec lock and conformance scaffold

**Goal:** Freeze v0.2 spec, create conformance harness.

**Deliverables**

* RFC 0001 to 0005 tagged and versioned.
* Baseline codebook object published.
* Field numbering tables verified.
* Conformance test harness repo.

**Acceptance criteria**

* Test harness can run golden vectors locally and in CI.
* Spec changes require a new version tag.

**Dependencies**

* None.

---

### Phase 1. Interop Core Libraries

**Goal:** Make it possible to parse, hash, sign, and exchange packets.

**Deliverables**

* mythos-can encoder and decoder.
* mythos-hash utilities.
* mythos-x framing and signature verification.
* Baseline CodebookID support.

**Acceptance criteria**

* Packet decode plus signature verify passes golden vectors.
* Canonicalization is stable under field reorder.

**Dependencies**

* Phase 0.

---

### Phase 2. MYTHOS-IR Validator and Minimal Executor

**Goal:** Execute deterministic IR and enforce budgets, without real tools.

**Deliverables**

* IR bundle parser and validator.
* Minimum opcode executor.
* Budget enforcement, cpu and io_count.
* ASSERT and basic contract checks.

**Acceptance criteria**

* Deterministic tasks replay identically.
* Budget exhaustion fails closed.
* Two executors produce identical outputs for the same IR bundle.

**Dependencies**

* Phase 1.

---

### Phase 3. Effect System, Tool Gateway, and Receipts

**Goal:** Real effects with receipts, but without ledger semantics yet.

**Deliverables**

* Tool Gateway service.
* EFFECT opcode integration.
* Receipt creation and signing.
* Receipt verification library.

**Acceptance criteria**

* Every effect yields a valid receipt.
* Request and response hashes match the bytes on the wire.

**Dependencies**

* Phase 2.

---

### Phase 4. Receipt Ledger and Idempotency

**Goal:** Deterministic deduplication and safe retries.

**Deliverables**

* Receipt ledger service.
* Register, commit, divergence state machine.
* Lease takeover behavior.

**Acceptance criteria**

* Repeated submissions with the same idempotency_key do not re-invoke tools.
* Conflicting request_hash triggers divergence and fails closed.
* Multiple commit conflict triggers divergence.

**Dependencies**

* Phase 3.

---

### Phase 5. Blob CAS and Merkle Proofs

**Goal:** Large data is real, verifiable, and streamable.

**Deliverables**

* Blob CAS service.
* MerkleList and ChunkedBlob DAG encoding per RFC 0004.
* Proof generation and streaming verification.
* BlobRef caps enforced.

**Acceptance criteria**

* Root CID matches golden vectors.
* Chunk proof verification passes.
* Range reads are correctly verified.

**Dependencies**

* Phase 1.

---

### Phase 6. Traces, Episodes, and Signals

**Goal:** Observations become learning objects.

**Deliverables**

* Trace emission in executor.
* EpisodeRef construction.
* Signal ingestion API.
* Minimal privacy controls, redaction transform hooks.

**Acceptance criteria**

* Traces and signals are content addressed.
* Signals are signature verified.
* Redacted traces preserve determinism for dataset building.

**Dependencies**

* Phases 3, 5.

---

### Phase 7. Deterministic Dataset Builder

**Goal:** Build datasets that rebuild identically.

**Deliverables**

* Dataset builder service implementing RFC 0003.
* Query AST evaluator.
* Sampling and stratification.
* Manifest writer as MerkleList blob.

**Acceptance criteria**

* DatasetRef manifests match expected CIDs for test corpora.
* Rebuilds produce byte-identical manifests.

**Dependencies**

* Phase 6.

---

### Phase 8. Training, Evaluation, and ring0 Promotion

**Goal:** Continuous improvement with bulletproof gates.

**Deliverables**

* Train worker and Eval worker as tools.
* Policy registry storing eval suites and thresholds.
* PromotionRecord creation.
* ring0 deployment mechanism.

**Acceptance criteria**

* Train produces ModelRef candidate with receipts.
* Eval produces attestations.
* Promotion requires quorum attestations and passes suite.
* Rollback is pinned.

**Dependencies**

* Phase 7.

---

### Phase 9. Scheduler Scale and Multi-Cell State

**Goal:** Infinite scale in the systems sense.

**Deliverables**

* Work queue and work stealing across executors.
* Pure result memoization by hash.
* Optional cell store for stateful workflows.
* Quota enforcement for spawn and IO.

**Acceptance criteria**

* Throughput increases linearly with added executors on pure-heavy loads.
* Effect load is bounded by caps and quotas.
* Cell writes serialize correctly.

**Dependencies**

* Phase 4.

---

### Phase 10. Production hardening

**Goal:** Make it operational.

**Deliverables**

* Observability, metrics, tracing, structured logs.
* Key rotation playbooks.
* Disaster recovery for blob store and ledger.
* Security review of caps and policy.

**Acceptance criteria**

* SLOs defined and measured.
* Kill switch for promotions and tool calls.
* Incident playbooks.

**Dependencies**

* All prior phases.

---

## 3. Staffing Plan

Minimal team for early phases:

* Protocol engineer, canonical encoding and hashing.
* Runtime engineer, IR executor and budgets.
* Infra engineer, blob store and ledger.
* Security engineer, caps and signatures.

Add for learning phases:

* ML engineer, training and eval tools.
* Data engineer, dataset builder and manifests.

Add for hardening:

* SRE, observability and reliability.

---

## 4. Deliverable Checklist

This list helps you track "is it real".

* [ ] v0.2 test vectors for encoding, hashing, merkle, receipts.
* [ ] Executor runs IR deterministically.
* [ ] Tool Gateway produces request and response hashes.
* [ ] Receipt ledger enforces idempotency and divergence fail closed.
* [ ] Blob CAS stores chunked DAGs and generates proofs.
* [ ] Traces and signals become episodes.
* [ ] Dataset builder yields identical manifests on rebuild.
* [ ] Training and eval produce receipts and attestations.
* [ ] ring0 promotion gates are enforced by policy.
* [ ] Quotas and budgets prevent runaway recursion and spawning.

---

## 5. Risk Register

### Drift and regression

Mitigation: ringed deployment, mandatory eval suites, rollback pinned.

### Ledger divergence

Mitigation: fail closed, quorum required for resolution.

### Privacy leakage

Mitigation: redaction transforms, encrypted blobs, cap gating.

### Tool nondeterminism

Mitigation: receipts, redundancy checks, conservative caching.

### Spec churn

Mitigation: tagged releases, conformance harness, no breaking changes without version bump.

---

## 6. First Build Recommendation

Start with a single end-to-end demo:

1. Send MYTHOS-X packet with one task.
2. Executor runs pure ops.
3. Executor performs one idempotent tool call.
4. Receipt ledger dedupes a retry.
5. Store a blob, verify a chunk proof.
6. Emit trace and signal.
7. Build deterministic dataset manifest.

Once that demo is stable, bolt on training and promotion.
