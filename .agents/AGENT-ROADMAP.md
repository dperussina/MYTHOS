# MYTHOS Subject Matter Expert Agents

This document tracks the domain expert agents created and planned for MYTHOS development.

## Status: Core Foundation Agents (COMPLETE)

These 7 agents are CREATED and ready for consultation. They cover the immediate priority: core libraries + CTVP runner.

### 1. ✅ CTVP & Conformance Expert
- **Domain**: Byte-for-byte conformance, test vector validation, interoperability
- **Covers**: mythos-v0.2-conformance/SPEC.md, all vectors/, manifest.json
- **Critical For**: Validating every implementation component
- **When to Consult**: "Does my implementation pass CTVP?", "What test vectors should I add?"

### 2. ✅ Canonical Encoding Expert
- **Domain**: MYTHOS-CAN encoding, varint, zigzag, map sorting, type tags
- **Covers**: RFC-0001 §7, SPEC.md §1, vectors/can/
- **Critical For**: mythos-can library implementation
- **When to Consult**: "How do I encode [type]?", "Why is my map ordering wrong?"

### 3. ✅ Content Addressing & Hashing Expert
- **Domain**: SHA-256, Hash struct, TypeID/ToolID/CID computation
- **Covers**: RFC-0001 §6, all ID computation rules
- **Critical For**: mythos-hash library implementation
- **When to Consult**: "How do I compute [ID type]?", "Why doesn't my hash match?"

### 4. ✅ Merkle Structures Expert
- **Domain**: RFC-0004 MerkleList, ChunkedBlob DAG, fanout=1024, proofs
- **Covers**: RFC-0004 entire, vectors/merkle/, vectors/blob/
- **Critical For**: mythos-merkle library implementation
- **When to Consult**: "How do I build a MerkleList?", "What's the root CID?"

### 5. ✅ Receipt & Ledger Expert
- **Domain**: RFC-0005 idempotency semantics, register/commit protocol, divergence
- **Covers**: RFC-0005 entire, vectors/receipts/, vectors/ledger/
- **Critical For**: mythos-receipts and mythos-ledger library implementation
- **When to Consult**: "How does register-commit work?", "When is an effect divergent?"

### 6. ✅ Wire Protocol Expert
- **Domain**: MYTHOS-X packet framing, signature verification, codebooks
- **Covers**: RFC-0001 §8-9, Appendix B, vectors/wire/, vectors/codebook/
- **Critical For**: mythos-x library implementation
- **When to Consult**: "How do I parse MYTHOS-X packets?", "How do codebooks work?"

### 7. ✅ Capabilities & Security Expert
- **Domain**: Capability structure, scope descriptors, enforcement, attenuation
- **Covers**: RFC-0001 §12, §19, Appendix A.11-A.12, ADR-001 threat model
- **Critical For**: mythos-caps library, security boundaries
- **When to Consult**: "How do I validate a capability?", "What's the correct scope?"

---

## Status: Phase 2 Agents (PLANNED)

These 5 agents will be created when we move beyond core libs into execution, learning, and packaging.

### 8. 🔄 Blob Subsystem Expert (NEXT PRIORITY)
- **Domain**: Blob operations beyond Merkle (chunking strategy, BlobRef usage, encryption, provenance)
- **Covers**: RFC-0001 §13, blob operations, streaming, BlobRef lifecycle
- **Critical For**: Blob CAS implementation, blob API design
- **When to Create**: When implementing blob store and blob operations (after merkle lib works)
- **When to Consult**:
  - "How do I implement blob streaming?"
  - "How does encryption work with content addressing?"
  - "What's the blob capability flow?"
  - "How do I track blob provenance?"

### 9. 🔄 MYTHOS-IR Expert (HIGH PRIORITY)
- **Domain**: IR bundle structure, opcode semantics, execution model, determinism
- **Covers**: RFC-0001 §10-11 (IR Bundle, Opcode Set), Appendix A.14
- **Critical For**: mythos-ir library, executor core implementation
- **When to Create**: When starting minimal executor implementation (after core libs stable)
- **When to Consult**:
  - "How do I execute [opcode]?"
  - "What's the IR bundle validation sequence?"
  - "How does the const pool / node pool work?"
  - "What's deterministic vs non-deterministic in IR?"
  - "How do EFFECT/REQUIRE_CAP opcodes work?"

### 10. 🔄 Learning Substrate Expert (MEDIUM PRIORITY)
- **Domain**: Episodes, signals, traces, recipes, eval suites, promotion records
- **Covers**: RFC-0001 §16-17, Appendix A.15 (Learning Structs)
- **Critical For**: Learning loop implementation, promotion gates
- **When to Create**: When implementing episode capture and promotion pipeline (post-MVP)
- **When to Consult**:
  - "How do I emit traces and episodes?"
  - "What's the structure of a Signal?"
  - "How do promotion records work?"
  - "What's required for ring promotion?"
  - "How are eval attestations used?"

### 11. 🔄 Dataset & Query Expert (MEDIUM PRIORITY)
- **Domain**: RFC-0003 deterministic queries, sampling (HASH_N, FIRST_N), stratification
- **Covers**: RFC-0003 entire, vectors/dataset/, deterministic dataset building
- **Critical For**: Dataset builder service, query execution
- **When to Create**: When implementing learning substrate and dataset pipeline
- **When to Consult**:
  - "How does HASH_N sampling work?"
  - "How do I build a deterministic dataset?"
  - "What's the query expression model?"
  - "How does stratification work?"
  - "Why doesn't my manifest match expected CID?"

### 12. 🔄 Package & Module Expert (HIGH PRIORITY)
- **Domain**: RFC-0006 (when written), PackageManifest, module ABI, WASM/process runners
- **Covers**: DEVELOPER_PLAYBOOK.md packaging strategy, RFC-0006, module contracts
- **Critical For**: Package format, module runner implementation, mythos pack CLI
- **When to Create**: After RFC-0006 is written, before implementing package loading
- **When to Consult**:
  - "What's the PackageManifest structure?"
  - "How does the WASM module ABI work?"
  - "How do I implement a process runner?"
  - "What's the ToolRequest/ToolResponse protocol?"
  - "How do modules communicate with executor?"

---

## Creation Priority Sequence

Based on ADR-001 implementation roadmap:

**Phase 1: Core Libs (DONE)**
- Agents 1-7 created ✅

**Phase 2: Executor Core**
- Create Agent 9 (MYTHOS-IR Expert) - for IR execution
- Create Agent 8 (Blob Subsystem Expert) - for blob operations
- Create Agent 12 (Package & Module Expert) - for package loading

**Phase 3: Learning Loop**
- Create Agent 10 (Learning Substrate Expert) - for traces/episodes
- Create Agent 11 (Dataset & Query Expert) - for dataset building

---

## How to Consult an Agent

When you need expert guidance:

1. **Identify the domain** - Which agent covers your question?
2. **Ask specific questions** - Reference RFCs, test vectors, or implementation challenges
3. **Request validation** - "Does this design match the spec?"
4. **Clarify ambiguities** - "What does the RFC mean by [term]?"
5. **Debug failures** - "Why doesn't my [component] pass CTVP?"

Agents have deep knowledge of their RFCs, test vectors, and common gotchas. Use them liberally.

---

## Future Specialized Agents (Optional)

If needed, we may add:
- **Policy & Promotion Expert** - Ring policies, promotion contracts, eval suite design
- **Cell State Expert** - Cell store, event logs, snapshots, consistency
- **Tool Gateway Expert** - Tool dispatch, request/response hashing, tool policies
- **Distributed Systems Expert** - Sharding, replication, CAS distribution, multi-region

These would be created on-demand based on implementation complexity.
