# ADR-001: Implementation Direction and Priorities

**Status**: Accepted
**Date**: 2026-01-01
**Architect**: Project Architect
**Context**: Establishing clear direction for MYTHOS v0.2 implementation

---

## Decision

This ADR establishes the implementation strategy, language choices, priorities, and non-negotiable constraints for MYTHOS v0.2 development.

---

## 1. Current Status and Immediate Priorities

**Status**: This repository is deliberately spec-first. The goal is to lock the substrate (canonical bytes, IDs, receipts, blobs, ledger semantics) before implementing runtime code.

**Next Deliverables** (in order):

1. **Core Libraries** (highest priority)
   - mythos-can (canonical encoder/decoder)
   - mythos-hash (SHA-256 and ID constructors)
   - mythos-merkle (MerkleList + ChunkedBlob per RFC-0004)
   - mythos-receipts (receipt construction and verification)
   - mythos-ledger (RFC-0005 idempotency semantics)
   - mythos-x (wire framing and signature verification)
   - **CTVP Conformance Runner** (validates implementation against test vectors)

2. **RFC-0006: Package Format + Module Execution Contract**
   - Write in parallel with core libs, but slightly behind
   - Learn what needs specification by building loader/dispatcher
   - Don't wait for perfect RFC to start core libs

3. **Minimal Executor**
   - Loads packages, verifies signatures, runs demo
   - Built on top of working core libs

**Rationale**: Core libs are fully testable against CTVP today. Packaging must be built on a working encoder/hash/merkle/receipt stack. Implementation experience informs RFC-0006.

---

## 2. Runtime Language Choice

**Primary Runtime Language**: Rust

**Rationale**:
- Deterministic byte format handling
- Safe parsing of untrusted input
- Fast hashing and Merkle operations
- Easy static builds for distribution
- Clear path to WASM-friendly tooling
- Not religious, pragmatic

**Flexibility for Non-Core Components**:
- TypeScript: pack tools, manifest authoring UX, visualization
- Python: dataset transforms, ML experiments
- Other languages acceptable for tooling layer

**Non-Negotiable Constraint**: Any component implementing MYTHOS-CAN, hashing, IDs, merkle structures, receipts, or wire framing MUST produce byte-identical output across implementations. This is not "nice to have" - it is the entire substrate.

---

## 3. Canonical Bytes as Law

**Strictness Level**: Absolute

If two language implementations of the encoder exist, they MUST produce byte-identical output and hashes for the same object graph, or you don't have canonical meaning.

**Acceptable "Wiggle Room"**: None at the byte layer. Only at human tooling layer (pretty printers, AST builders, UX).

**Consequence**: This enables:
- Deterministic verification
- Global deduplication
- Content-addressed caching
- Cryptographic provenance
- Infinite replication without trust

---

## 4. Forcing Function: Why MYTHOS Exists

**Immediate Driver**: Create a substrate where agents can operate in the real world with:
- Replayability
- Idempotent effects
- Verifiable evidence
- Bounded authority

Right now, "agents" are mostly chatty. MYTHOS makes them operational.

**Strategic Driver**: Foundation for self-distributing, self-improving systems where:
- "What ran" is provable
- "What changed" is auditable
- Unsafe changes can't propagate

**Use Case**: Real-world execution under control. Not exploratory research - operational necessity.

---

## 5. Learning Loop Design

**Paradigm**: Deliberately agnostic

MYTHOS does NOT bet on:
- RLHF vs fine-tuning
- Retrieval vs distillation
- Specific training algorithms

**What MYTHOS Defines**:
- What counts as an episode
- How traces/signals become datasets deterministically
- How candidate models/packages are evaluated
- How promotions are gated and audited

**Practical First Loop** (lowest risk):
1. Retrieval improvements
2. Policy + rule improvements
3. Tool behavior improvements
4. Evaluator suite growth

Then move into training where deltas can be proven with eval attestations.

---

## 6. Human Readability Boundary

**Position**: Humans should NOT hand-author canonical objects as primary workflow.

**Intended Workflow**:
1. Humans author in human-friendly DSL / config / UI
2. Tooling compiles to MYTHOS-IR + canonical objects
3. Humans debug via "disassembly" views, receipts, traces

**Near-Term Strategy**: Build tools that generate canonical objects and let humans inspect them. Don't build fancy DSL yet unless it unblocks adoption.

**Not Required for v0.2**: Human-friendly DSL that compiles to IR is future work.

---

## 7. Scale Targets and Threat Model

**"Infinite Scale" Means**: No centralized bottleneck, replication by hash, horizontal scaling is boring.

**Real Near-Term Envelope**:
- **Phase 1**: 1–10 executors, local CAS
- **Phase 2**: 50–500 executors, shared blob store + ledger
- **Phase 3**: Multi-cell, multi-region

**Threat Model**: Start with "distributed systems are hard + accidental faults", architect for partial adversarial behavior.

**Concrete Assumptions**:
- Buggy/misconfigured nodes
- Nondeterministic tools
- Partial compromise is possible
- Full Byzantine consensus NOT solved in v0.2

**Safety Posture Relies On**:
- Receipts (signed proof of effects)
- Divergence detection (conflicting hashes fail closed)
- Fail-closed ledger semantics (RFC-0005)
- Explicit capabilities (least privilege)
- Ringed promotion gates (progressive rollout)

**Result**: 80% of safety value without pretending we solved global adversarial coordination.

---

## 8. RFC-0006 Priority and Sequencing

**Is RFC-0006 Critical?**: Yes - biggest missing piece for "single distributable package, multi-language."

**Should It Block Core Libs?**: No

**Recommended Order**:
1. Core libs + CTVP runner (prove bytes/IDs work)
2. RFC-0006 (manifest + module ABI + runners)
3. Minimal executor (loads packages, runs demo)
4. CAS + distribution by CID

**Rationale**: RFC-0006 written before implementation will be guessy. RFC-0006 written after byte layer exists will be accurate.

---

## 9. CTVP Completeness and Interop Guarantees

**Current Status**: Strong start, not complete

**What CTVP Covers Today**:
- Canonical encoding/hashing
- Merkle list and chunked blob root
- Receipt ID and signature
- Ledger register/commit objects and IDs
- Dataset HASH_N selection + manifest root
- Baseline codebook and ID
- One wire packet

**Known Gaps to Add**:
- More edge-case encoding vectors (negative ints, large varints, UTF-8 edge cases, map ordering collisions)
- Multi-level Merkle trees (internal nodes, fanout boundaries)
- Blob range proofs (not just root computation)
- Ledger divergence vectors (conflicting commits, lease expiry takeover)
- More wire vectors (multiple signatures, codec variants, corrupted frames)
- Capability attenuation + validation vectors (if caps are normative v0.2)

**Interop Guarantee Target**: Two implementations passing CTVP should interoperate for the subset CTVP covers.

**Unspecified Behavior**: If behavior is unspecified, it's a spec bug. Either:
- Add it to RFCs, or
- Add it to CTVP vectors

**Policy**: No "undefined behavior" in core substrate is acceptable long-term.

---

## 10. Bottom Line for Development Team

1. **Build core libs + conformance runner first** - This is the foundation
2. **Draft RFC-0006 next** - Lock packaging + module execution contract
3. **Build minimal executor** - Runs a demo package end-to-end
4. **Treat canonical bytes as law** - If it's not byte-identical, it's not MYTHOS

---

## Consequences

**Positive**:
- Clear implementation roadmap
- Testable foundation (CTVP runner validates everything)
- Language flexibility where it matters
- No premature optimization on human DSL
- Pragmatic threat model

**Negative**:
- Rust may limit contributor pool initially
- Byte-identical constraint is unforgiving
- CTVP gaps must be filled progressively
- Human ergonomics deferred to post-v0.2

**Neutral**:
- Learning paradigm agnosticism requires careful abstraction
- Multi-language ecosystem requires vigilance on canonical encoding

---

## Notes

This ADR represents architect direction as of 2026-01-01. Future ADRs may refine specific aspects (e.g., exact module ABI, dataset transform pipeline, promotion policy structure) as implementation progresses.
