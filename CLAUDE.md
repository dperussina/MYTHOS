# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MYTHOS is an AI-first executable language and agent-to-agent protocol that unifies code execution, effect auditing, blob transport, and continuous learning into a single content-addressed substrate. This repository contains the specification (RFCs), conformance test vectors, and implementation guidance for building MYTHOS v0.2 compliant runtimes.

**Current Status**: Deliberately spec-first. The goal is to lock the substrate (canonical bytes, IDs, receipts, blobs, ledger semantics) before implementing runtime code.

**Forcing Function**: Create a substrate where agents can operate in the real world with replayability, idempotent effects, verifiable evidence, and bounded authority. Right now, "agents" are mostly chatty. MYTHOS makes them operational.

**See ADR-001-IMPLEMENTATION-DIRECTION.md for detailed architect decisions on priorities, language choice, and constraints.**

## Repository Structure

- **spec/**: Core RFC documents (RFC-MYTHOS-0001 through 0005)
  - RFC-MYTHOS-0001.md: Core protocol, runtime semantics, learning substrate
  - WHITE_PAPER.md: High-level conceptual overview
  - Philosophy.md: Design principles and architectural axioms
  - Roadmap.md: Development trajectory
  - CTVP.md: Conformance testing philosophy

- **mythos-v0.2-conformance/**: Conformance Test Vector Pack (CTVP)
  - SPEC.md: Normative encoding rules and test vector specifications
  - vectors/: Byte-for-byte test vectors organized by category
    - can/: Canonical encoding vectors
    - merkle/: Merkle list and DAG vectors
    - receipts/: Receipt construction vectors
    - ledger/: Ledger state machine vectors
    - dataset/: Dataset determinism vectors
    - codebook/: Baseline codebook vectors
    - wire/: MYTHOS-X wire protocol vectors
    - blob/: Chunked blob vectors

- **Implementation.md**: Technical implementation blueprint
- **DEVELOPER_PLAYBOOK.md**: Step-by-step guide for building a distributable MYTHOS package system

## Core Architecture Concepts

### Content Addressing and Determinism

Everything in MYTHOS is content-addressed. Identity equals hash. This enables:
- Infinite replication without trust
- Deterministic verification
- Global deduplication
- Cryptographic provenance

**Critical Rule**: Any object that is hashed MUST be canonicalized first using MYTHOS-CAN encoding rules defined in mythos-v0.2-conformance/SPEC.md.

### Canonical Encoding (MYTHOS-CAN)

MYTHOS-CAN is a deterministic binary encoding format based on tagged values with strict ordering:
- Type tags: NULL(0x00), BOOL(0x01-0x02), UVARINT(0x03), IVARINT(0x04), BYTES(0x05), TEXT(0x06), LIST(0x07), MAP(0x08)
- Structs encoded as MAPs with integer field numbers as keys
- Map keys MUST be sorted by canonical encoded bytes (lexicographic)
- Varints use unsigned LEB128 encoding
- Signed integers use zigzag encoding

### Hashing and IDs

All hashes use SHA-256 (alg=1) producing 32-byte digests:
- TypeID = Hash(canonical TypeDef)
- ContractID = Hash(canonical ContractDef)
- ToolID = Hash(canonical ToolDef)
- SymbolID = Hash(canonical SymbolSig)
- CID (Content ID) = Hash(canonical object bytes or Merkle root)

### Effect Model and Receipts

Effects are non-deterministic operations requiring capabilities:
- Every effect produces a signed Receipt
- Receipt contains: tool_id, request_hash, response_hash, idempotency_key, signer, timestamp
- Receipt ledger provides at-most-once semantics via register/commit protocol

### Capabilities

Least-privilege authorization tokens required for all effects:
- Structured scope descriptors (not free text)
- Time-bounded and revocable
- Support attenuation chains
- Examples: Blob.Read, Tool.Call, Train.Run, Deploy.Promote

### Blob Subsystem

Content-addressed blob storage with Merkle chunking:
- Blobs split into fixed chunks (default 4 MiB, range 256 KiB to 16 MiB)
- Merkle DAG with fanout=1024
- BlobRef contains: cid, size, media type, codec, chunk count
- Streaming verification via Merkle proofs

### Learning Substrate

First-class learning objects:
- TraceRef: execution traces with receipts
- EpisodeRef: trace + context + outcome
- Signal: feedback attached to episodes
- DatasetRef: deterministic dataset definition + manifest
- RecipeRef: training configuration
- ModelRef: model artifacts
- EvalSuiteRef: evaluation tests
- PromotionRecord: signed deployment decision

**Learning Paradigm**: Deliberately agnostic. MYTHOS doesn't bet on RLHF vs fine-tuning vs retrieval vs distillation. It defines what counts as an episode, how traces/signals become datasets deterministically, how candidates are evaluated, and how promotions are gated.

**Practical First Loop** (lowest risk): Start with retrieval improvements, policy/rule improvements, tool behavior improvements, and evaluator suite growth. Then move into training where deltas can be proven with eval attestations.

### Ring Deployment

Progressive deployment gates:
- ring0: canary, limited caps, high observability
- ring1: broader caps, requires eval attestations
- ringN: production, only for promoted artifacts with quorum signoff

## Implementation Strategy

**ARCHITECT DIRECTIVE**: See ADR-001 for authoritative implementation direction.

**Implementation Priority** (in order):

1. **Core Libraries + CTVP Runner** (HIGHEST PRIORITY - DO THIS FIRST)
   - These are fully testable against existing conformance vectors
   - Foundation for everything else
   - Rust is the default choice (pragmatic: determinism, safety, WASM future)

2. **RFC-0006: Package Format + Module Execution Contract**
   - Write in parallel with core libs, but slightly behind
   - Learn from implementation experience
   - Don't wait for perfect RFC to start core libs

3. **Minimal Executor**
   - Built on working core libs
   - Loads packages, verifies signatures, runs demo

4. **CAS + Distribution**
   - Content-addressed storage
   - Artifact registry

**Critical Constraint**: Canonical bytes are LAW. Any component implementing MYTHOS-CAN, hashing, IDs, merkle, receipts, or wire framing MUST produce byte-identical output across implementations. No wiggle room.

When implementing MYTHOS, follow this sequence (from DEVELOPER_PLAYBOOK.md):

1. **Package Format First** (RFC-0006, to be written)
   - Define PackageManifest structure
   - Establish module execution contracts
   - Support multiple module types: WASM (primary), process sidecars, OCI images

2. **Core Libraries** (IR-first approach)
   - mythos-can: encoder/decoder
   - mythos-hash: SHA-256 and ID constructors
   - mythos-merkle: RFC-0004 MerkleList + ChunkedBlob
   - mythos-receipts: receipt construction and verification
   - mythos-ledger: RFC-0005 idempotency semantics
   - mythos-x: wire framing and signature verification
   - mythos-package: manifest parsing and verification
   - mythos-ir: IR validator and executor

3. **Executor Core**
   - Load and verify PackageManifest
   - Execute IR deterministically
   - Dispatch effects to tool modules
   - Enforce capabilities and budgets
   - Generate receipts and ledger entries

4. **Module Runners**
   - WASM runner (primary): sandboxed, deterministic
   - Process runner (secondary): for Node/Python tools via stdin/stdout framing

5. **Content-Addressed Storage**
   - Local CAS store keyed by SHA-256
   - Remote artifact registry (HTTP API)
   - Merkle node storage

6. **Safety Gates**
   - Ring deployment enforcement
   - Capability validation
   - Ledger-based idempotency firewall

## Conformance Testing

**Critical**: Implementations MUST pass all CTVP vectors byte-for-byte. This is non-negotiable.

**CTVP Current Status**: Strong start, not complete. Known gaps include:
- More edge-case encoding vectors (negative ints, large varints, UTF-8 edge cases)
- Multi-level Merkle trees (internal nodes, fanout boundaries)
- Blob range proofs
- Ledger divergence vectors (conflicting commits, lease expiry takeover)
- More wire vectors (multiple signatures, codec variants, corrupted frames)
- Capability attenuation + validation vectors

**Interop Guarantee**: Two implementations passing CTVP should interoperate for the covered subset. Unspecified behavior is a spec bug - either add to RFCs or add to CTVP.

Conformance test execution order:
1. Encoding (vectors/can/)
2. Merkle structures (vectors/merkle/, vectors/blob/)
3. Receipts (vectors/receipts/)
4. Ledger (vectors/ledger/)
5. Datasets (vectors/dataset/)
6. Codebook (vectors/codebook/)
7. Wire protocol (vectors/wire/)

Each vector includes:
- Input description (.json where applicable)
- Expected canonical bytes (.bin)
- Expected SHA-256 (.sha256 or .hex)
- Test keys (keys/ed25519_test_seed.hex)

## Key Design Principles

From Philosophy.md - maintain these axioms during implementation:

1. **Identity equals content** - Content addressing everywhere
2. **Meaning equals canonical bytes** - One deterministic serialization
3. **Power equals capability** - Explicit, bounded authorization
4. **Action equals receipt** - All effects must be auditable
5. **Retries equal ledger** - Idempotency via append-only log
6. **Scale equals caching** - Replication without trust
7. **Autonomy equals gated promotion** - Progressive ring deployment

## Threat Model and Scale

**Threat Model**: Start with "distributed systems are hard + accidental faults", architect for partial adversarial behavior. Assume buggy/misconfigured nodes, nondeterministic tools, and partial compromise is possible. Full Byzantine consensus NOT solved in v0.2.

**Safety Posture**: Receipts, divergence detection, fail-closed ledger semantics, explicit capabilities, ringed promotion gates. This gets 80% of safety value without pretending we solved global adversarial coordination.

**Scale Targets**:
- Phase 1: 1–10 executors, local CAS
- Phase 2: 50–500 executors, shared blob store + ledger
- Phase 3: Multi-cell, multi-region

"Infinite scale" means no centralized bottleneck, replication by hash, horizontal scaling is boring.

## Fail-Closed Philosophy

MYTHOS fails closed on:
- Unknown CodebookID (unless negotiated)
- Budget exhaustion
- Missing capabilities
- Divergent ledger records (conflicting request/response hashes)
- Invalid signatures
- Unverifiable receipts

## Wire Protocol (MYTHOS-X)

Agent-to-agent exchange format:
- Magic: 0x4D594854 ("MYTH")
- Version: 0x0002 (v0.2)
- CodecID: 1=MYTHOS-CAN, 2=zstd(MYTHOS-CAN)
- CodebookID: baseline or negotiated
- Payload: MYTHOS-IR bundle
- SigBlock: Ed25519 signatures

## Module ABI Contract

All tool modules (WASM or process) communicate via:
- Input: ToolRequest (MYTHOS-CAN encoded)
- Output: ToolResponse (MYTHOS-CAN encoded)
- Executor computes request_hash and response_hash
- Modules never access ledger directly

## Security Boundaries

- WASM modules: sandboxed, no filesystem/network unless cap-granted
- Process modules: timeouts, output limits, kill-on-violation
- Capabilities: enforced at executor level before dispatch
- Receipts: signed with Ed25519
- Ledger: append-only, register-before-invoke protocol

## Common Gotchas

1. **Map Ordering**: Map pairs MUST be sorted by canonical encoded key bytes, not by key values directly
2. **Field Numbers**: Structs use integer field numbers, not string keys
3. **Zigzag Encoding**: Signed integers use `(x << 1) ^ (x >> 63)` before varint encoding
4. **CID Computation**: Always hash canonical bytes, never raw input
5. **Receipt ID**: Computed from receipt WITHOUT receipt_id field and signature field
6. **Idempotency**: IdempotencyID = SHA-256(tool_id.bytes || idempotency_key)

## Documentation Standards

When contributing to this repository:
- RFCs use RFC 2119 keywords (MUST, SHOULD, MAY)
- Test vectors are normative - code must match bytes exactly
- Implementation guidance is in separate documents (Implementation.md, DEVELOPER_PLAYBOOK.md)
- Philosophy and rationale in Philosophy.md
- Architecture decisions in ADR-*.md files
- Keep specs machine-verifiable where possible

## Human Readability

**Position**: Humans should NOT hand-author canonical objects as primary workflow.

**Intended Workflow**:
1. Humans author in human-friendly DSL / config / UI (future)
2. Tooling compiles to MYTHOS-IR + canonical objects
3. Humans debug via "disassembly" views, receipts, traces

**Near-Term Strategy**: Build tools that generate canonical objects and let humans inspect them. Human-friendly DSL is NOT required for v0.2.

## Development Workflow

1. **Read DEVLOG.md first** - Understand current status, priorities, and blockers
2. Read relevant RFCs in RFC-MYTHOS-0001.md for normative requirements
3. Check DEVELOPER_PLAYBOOK.md for implementation sequence
4. Consult domain expert agents in .agents/ for specific questions
5. Validate against CTVP vectors in mythos-v0.2-conformance/
6. Follow canonical encoding rules in mythos-v0.2-conformance/SPEC.md
7. Ensure deterministic, byte-for-byte reproducibility
8. Test conformance early and continuously
9. **Update DEVLOG.md** - Document progress, decisions, and blockers

## Development Log (DEVLOG.md)

**Location**: `/DEVLOG.md`

The development log is the single source of truth for project status, decisions, and progress. It MUST be kept up-to-date.

### When to Update DEVLOG.md

**At Session Start**:
- Review "What We've Done" to understand context
- Check "What We're Going To Do" for current priorities
- Read "Questions We Have" and "Blockers" to understand state
- Note any new information from architect/team

**During Work**:
- Add questions as they arise (don't wait)
- Document blockers immediately when encountered
- Log decisions with rationale in "Decision Log"
- Update action items as they're completed

**At Session End**:
- Move completed tasks from "What We're Going To Do" to "What We've Done"
- Archive completed action items with timestamps
- Update "Questions We Have" based on work done
- Add new action items for next session
- Update "Last Updated" timestamp

### What to Document

**Progress (What We've Done)**:
- Features implemented
- Tests passing
- Documentation created
- Problems solved
- Milestones reached

**Plans (What We're Going To Do)**:
- Immediate next steps (this week)
- Short-term goals (2 weeks)
- Medium-term goals (month)
- Dependencies and sequencing

**Questions**:
- Technical ambiguities needing architect input
- Design decisions requiring approval
- Clarifications needed from stakeholders
- Things domain expert agents can't answer

**Blockers**:
- Technical issues preventing progress
- Missing information or specifications
- Resource constraints
- External dependencies

**Decisions**:
- What was decided
- Who decided it
- Why (rationale)
- When (timestamp)
- Impact on project

### DEVLOG Structure

The log maintains these sections:
- **What We've Done**: Completed work by session/date
- **What We're Going To Do**: Prioritized future work
- **Questions We Have**: Organized by stakeholder (Architect, Team, Agents)
- **Blockers**: Current and potential, with severity
- **Assistance Needed**: Specific asks with context
- **Risk Register**: Technical and project risks
- **Success Metrics**: Phase completion criteria
- **Action Items**: Concrete next steps with owners
- **Decision Log**: Timestamped decisions with rationale
- **Change Log**: Session-by-session summary
- **Notes for Future Sessions**: Context maintenance guidance

### Example Updates

**Good Update** (Specific, Actionable):
```markdown
## What We've Done
### Session 2: Core Libraries Start (2026-01-02)
- Implemented mythos-can encoder for UVARINT and IVARINT
- Discovered edge case: zigzag encoding for INT64_MIN
- Added question for architect about overflow handling
- 15/50 CTVP can/ vectors passing

## Questions We Have
### For the Architect
- **Q**: How should zigzag encoding handle INT64_MIN overflow?
  - Context: `(x << 1) ^ (x >> 63)` overflows for INT64_MIN
  - Impact: Affects IVARINT encoding correctness
```

**Bad Update** (Vague, Not Actionable):
```markdown
- Made progress on encoding
- Some tests passing
- Had some questions
```

### Integration with Domain Expert Agents

When consulting agents (see `.agents/README.md`):
1. Document the question in DEVLOG before asking agent
2. Document agent's answer if it resolves the question
3. If agent can't answer, escalate to architect in DEVLOG
4. Track which agent helped with which problem (useful for future)

## Future Work

See Roadmap.md for planned evolution. Key missing pieces:
- RFC-MYTHOS-0006: Package Format + Module Execution Contract (referenced in DEVELOPER_PLAYBOOK.md)
- Runtime implementation (libraries and executor)
- Packaging toolchain (mythos pack CLI)
- Artifact registry for distribution
- Full policy registry and verifier services
