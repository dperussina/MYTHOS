# MYTHOS Subject Matter Expert Agents

## Status: Core Foundation Complete ✅

We have successfully created **7 domain expert agents** covering the immediate implementation priority: **Core Libraries + CTVP Runner**.

These agents have deep knowledge of their respective domains and are ready to be consulted throughout MYTHOS development.

---

## Active Expert Agents

### 1. ✅ CTVP & Conformance Expert
**Domain**: Byte-for-byte conformance validation, test vector interpretation, interoperability guarantees

**Knowledge Base**:
- mythos-v0.2-conformance/README.md, SPEC.md, manifest.json
- All test vector categories (can/, merkle/, blob/, receipts/, ledger/, dataset/, codebook/, wire/)
- Known gaps in conformance coverage (from ADR-001)
- Vector-only signing rule for WIRE_001

**Key Expertise**:
- Test vector validation procedures
- Byte-identical output requirements
- Conformance gap identification
- Interoperability requirements

**Ask Me**: "Does my implementation pass CTVP?", "What test vectors should I add?", "Why doesn't my hash match?"

---

### 2. ✅ Canonical Encoding Expert
**Domain**: MYTHOS-CAN encoding, deterministic serialization, type tags, varint/zigzag, map sorting

**Knowledge Base**:
- SPEC.md §1 (MYTHOS-CAN v0.2)
- RFC-0001 §7 (Canonical Encoding)
- vectors/can/ test vectors

**Key Expertise**:
- Type tag encoding (0x00-0x08)
- LEB128 varint encoding
- Zigzag encoding for signed integers: `(x << 1) ^ (x >> 63)`
- MAP canonical ordering: **sort by encoded key bytes** (common mistake!)
- Struct-as-MAP convention with integer field numbers

**Ask Me**: "How do I encode [type]?", "Why is my map ordering wrong?", "What's the zigzag formula?", "What bytes do I output?"

---

### 3. ✅ Content Addressing & Hashing Expert
**Domain**: SHA-256 hashing, ID computation (TypeID, ToolID, CID, etc.), canonicalization requirements

**Knowledge Base**:
- RFC-0001 §6 (Canonical Types and Hashing)
- All ID computation rules
- Hash struct and algorithm IDs

**Key Expertise**:
- Hash struct: `{1: alg (u8), 2: bytes (bytes)}`
- Core IDs: TypeID, ToolID, ContractID, SymbolID, RecipeID, EvalSuiteID
- Receipt ID: `SHA-256(canonical_bytes(receipt_without_fields_1_and_11))`
- IdempotencyID: `SHA-256(tool_id.bytes || idempotency_key)`
- Episode ID: `SHA-256(canonical_bytes(EpisodeRef_without_field_1))`
- **CRITICAL**: ALWAYS canonicalize before hashing

**Ask Me**: "How do I compute [ID type]?", "Why doesn't my CID match?", "Do names affect IDs?", "What bytes get hashed?"

---

### 4. ✅ Merkle Structures Expert
**Domain**: RFC-0004 MerkleList and ChunkedBlob DAG, fanout=1024, deterministic tree construction

**Knowledge Base**:
- RFC-0004 (entire specification)
- vectors/merkle/, vectors/blob/
- Appendix A (Merkle node field numbers)

**Key Expertise**:
- **Global constants**: version=1, fanout=1024 (REQUIRED), SHA-256 (alg=1)
- MerkleList: Ordered Hash lists (episode manifests, dataset manifests)
  - Leaf: 1 to fanout values
  - Internal: 2 to fanout children + count
- ChunkedBlob DAG: Binary blobs split into chunks
  - Default chunk size: 4 MiB (support 256 KiB to 16 MiB)
  - ChunkLeaf: ChunkDesc list + total_size
  - ChunkInternal: children + total_size + chunk_count + chunk_size
- CID = SHA-256(canonical_bytes(MerkleNode))
- **Chunk hashes over STORED bytes** (respects codec/encryption)

**Ask Me**: "How do I build a MerkleList?", "What's the root CID?", "How does fanout work?", "How do I verify chunks?"

**Test Vectors**:
- MERKLE_001: Expected root CID = `af3a30050b542dcba9903f244280ea6c0f3797eb56869f278b2a7df90cf237d4`
- BLOB_001: Expected root CID = `ea8aff585cecd24cc005789a45b07c39550650606c7d18054f4bf4879558bad3`

---

### 5. ✅ Receipt & Ledger Expert
**Domain**: RFC-0005 idempotency semantics, receipt construction, register/commit protocol, divergence handling

**Knowledge Base**:
- RFC-0005 (Receipt Ledger and Idempotent Effects)
- RFC-0001 §14 (Receipts and Attestations)
- vectors/receipts/, vectors/ledger/
- Appendix A.9-A.11 (Receipt, Attestation, Capability structures)

**Key Expertise**:
- Receipt structure (11 fields) and receipt_id computation
- IdempotencyID: `SHA-256(tool_id.bytes || idempotency_key)`
- Three ledger entry types:
  - EffectRegister (Pending): intent + lease
  - EffectCommit (Completed): final result + receipt reference
  - DivergenceMark: conflict record with reason codes
- **Normative execution rules**:
  - Register BEFORE invoke
  - Deduplication on completed
  - Request hash conflict → DivergenceMark + error
  - Response hash conflict → DivergenceMark + fail closed
  - Multiple identical commits OK, divergent ones fail
- Lease rules: expiry allows takeover
- Replay vs Retry semantics
- At-most-once guarantee (not exactly-once end-to-end)

**Ask Me**: "How does register-commit work?", "When is an effect divergent?", "How do leases work?", "Replay vs retry?"

---

### 6. ✅ Wire Protocol Expert
**Domain**: MYTHOS-X packet framing, signature verification, codebooks, agent-to-agent exchange

**Knowledge Base**:
- RFC-0001 §8 (MYTHOS-X Wire Protocol)
- RFC-0001 §9 (Codebooks)
- RFC-0001 Appendix B (Baseline Codebook, normative)
- vectors/wire/, vectors/codebook/

**Key Expertise**:
- **Packet structure** (byte-level):
  1. Magic: `0x4D594854` ("MYTH")
  2. Version: u16 network order (v0.2 = `0x0002`)
  3. Flags: u16 bitfield
  4. CodecID: u8 (1=MYTHOS-CAN, 2=zstd)
  5. CodebookID: Hash
  6. PayloadLen: u32 network order
  7. Payload: bytes
  8. SigBlockLen: u32 network order
  9. SigBlock: MYTHOS-CAN list of SigBlockEntry
- **Signature block**: Ed25519 over (magic through payload)
- **Baseline codebook** (REQUIRED):
  - Codeword ranges: 0x0100-0x01FF (opcodes), 0x0200-0x02FF (small ints), 0x0300-0x03FF (field nums), 0x0400-0x04FF (capscope kinds)
  - CodebookID = SHA-256(canonical_bytes(sorted entry list))
- **Negotiated codebooks** (OPTIONAL): proposal + acceptance + fallback
- **Vector-only signing rule** (WIRE_001 test vector only)

**Ask Me**: "How do I parse MYTHOS-X packets?", "What's in the signature?", "How do codebooks work?", "CodecID vs CodebookID?"

---

### 7. ✅ Capabilities & Security Expert
**Domain**: Capability structure, scope descriptors, enforcement, attenuation chains, fail-closed security

**Knowledge Base**:
- RFC-0001 §12 (Capabilities)
- RFC-0001 §19 (Security Considerations)
- Appendix A.11-A.12 (Capability and CapScope structs)
- ADR-001 (Threat model and safety posture)

**Key Expertise**:
- Capability structure (10 fields) and cap_id computation
- **CapScope**: Structured, not free text (11 scope kinds in v0.2)
  - Blob: Read, Write, Pin, Delete
  - Tool: Call
  - Training: Run
  - Eval: Run
  - Deploy: Promote
  - Compute: Spawn
  - Cell: Read, Write
- **REQUIRED enforcement checks** (BEFORE effect execution):
  - Subject match (subject == executing agent)
  - Time bounds (not_before ≤ now < expires_at)
  - Scope constraints
  - Attenuation chain validation
- **Attenuation**: Delegation with reduced authority
  - Child cap references parent via attenuation_parent
  - Must be same/stricter bounds and scope
- **Threat model** (ADR-001): Distributed systems + bugs + partial adversarial
  - NOT full Byzantine consensus in v0.2
  - Safety via receipts, divergence detection, fail-closed ledger, explicit caps, ring gates
- **WASM security**: No filesystem/network unless cap-granted, memory limits
- **Process security**: Timeouts, output limits, kill-on-violation

**Ask Me**: "How do I validate a cap?", "What's the correct scope?", "How does attenuation work?", "When to fail closed?"

---

## How to Consult These Agents

When implementing MYTHOS components, these agents are your SME team. Here's how to use them:

**1. Identify Your Domain**
- Working on encoding? → Canonical Encoding Expert
- Computing hashes or IDs? → Content Addressing & Hashing Expert
- Building Merkle trees? → Merkle Structures Expert
- Implementing receipts/ledger? → Receipt & Ledger Expert
- Parsing MYTHOS-X packets? → Wire Protocol Expert
- Enforcing capabilities? → Capabilities & Security Expert
- Validating implementation? → CTVP & Conformance Expert

**2. Ask Specific Questions**
- Reference RFC sections, test vectors, or specific structs
- Provide context about what you're implementing
- Ask about edge cases and gotchas

**3. Request Validation**
- "Does this design match the spec?"
- "Why doesn't my output match the test vector?"
- "What am I missing in this implementation?"

**4. Get Implementation Guidance**
- "What's the recommended approach for [component]?"
- "What are the critical invariants I must maintain?"
- "How do I test this correctly?"

---

## Phase 2 Agents (Planned)

See [AGENT-ROADMAP.md](./AGENT-ROADMAP.md) for the 5 additional agents we'll create:

8. 🔄 **Blob Subsystem Expert** (NEXT) - Blob operations, streaming, encryption, provenance
9. 🔄 **MYTHOS-IR Expert** (HIGH PRIORITY) - IR bundle, opcode execution, determinism
10. 🔄 **Learning Substrate Expert** (MEDIUM) - Episodes, signals, promotion records
11. 🔄 **Dataset & Query Expert** (MEDIUM) - RFC-0003, deterministic queries, sampling
12. 🔄 **Package & Module Expert** (HIGH PRIORITY) - RFC-0006, PackageManifest, module ABI

These will be created when we move beyond core libs into execution, learning, and packaging phases.

---

## Success Metrics

**Goal**: Build **mythos-can**, **mythos-hash**, **mythos-merkle**, **mythos-receipts**, **mythos-ledger**, **mythos-x** libraries + **CTVP conformance runner** that pass all test vectors byte-for-byte.

**These 7 agents provide the domain expertise needed to achieve that goal.**

Ready to build MYTHOS? Consult your expert team!
