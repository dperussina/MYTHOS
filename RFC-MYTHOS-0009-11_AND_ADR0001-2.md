Below are drop-in drafts that “solve” the four open questions at the level you need right now (normative enough to build, flexible enough to evolve), plus the companion ADR.

You can copy/paste these directly into your repo as new files:
	•	RFC-MYTHOS-0008.md (caps + attenuation chain)
	•	RFC-MYTHOS-0009.md (evidence bundles + compression)
	•	RFC-MYTHOS-0010.md (node identity + routing receipts)
	•	RFC-MYTHOS-0011.md (CID propagation gossip, optional)
	•	ADR-002-MCP-GATEWAY-TRUST-BOUNDARY.md (the “front door” rationale)

⸻

RFC-MYTHOS-0008

Capabilities and Attenuation Chain

Status: Draft
Version: 0.8.0-draft
Date: 2026-01-01
Related: RFC-0002 (Core Objects), RFC-0005 (Ledger), RFC-0007 (MCP Gateway)

1. Purpose

Define a cryptographically verifiable capability (Cap) format and an attenuation chain mechanism. Caps are the only way to authorize effects (tool calls, routing, fetch, publish). This RFC makes “power explicit”.

2. Non-goals
	•	Key management/HSM details (deferred)
	•	Full policy language for caps (minimal scopes only)
	•	Byzantine consensus over cap issuance (not required)

3. Design requirements (normative)

A valid cap MUST be:
	•	content-addressable by CID
	•	verifiable offline by nodes
	•	attenuable by downstream holders without issuer participation
	•	time-bounded
	•	scope-bounded (tools/resources)
	•	budget-bounded

Nodes MUST fail closed if:
	•	cap signature invalid
	•	cap expired
	•	cap scope insufficient
	•	cap budget exceeded
	•	attenuation chain invalid

4. Core objects

4.1 CapAuthorityID

Represents the signing authority for caps.

CapAuthorityID (struct):
	•	1: scheme (u8) (v0.2: 1=ed25519)
	•	2: pubkey (bytes)
	•	3: hint (text, optional)

4.2 CapScope

Minimal scope model sufficient for v0.2.

CapScope (struct):
	•	1: tools_allow (list)
	•	2: tools_deny  (list, optional)
	•	3: resources_allow (list, optional)
	•	4: resources_deny  (list, optional)
	•	5: ring_max (u8) (max ring this cap can operate in)
	•	6: cell_allow (list, optional)

Semantics:
	•	Allow lists default to empty meaning “none”.
	•	Deny lists override allow lists.
	•	Ring/cell gates are additional constraints.

4.3 CapBudget

Caps MUST carry a budget ceiling.

CapBudget (struct):
	•	1: cpu_us (u64)
	•	2: wall_us (u64, optional)
	•	3: io_count (u32)
	•	4: state_writes (u32)
	•	5: net_bytes (u64, optional)
	•	6: blob_bytes (u64, optional)

4.4 Capability (Cap)

A capability is a signed, CID-addressable authorization token.

Capability (struct):
	•	1: cap_id (Hash)  // computed
	•	2: version (u16)  // v0.2 => 2
	•	3: issuer (CapAuthorityID)
	•	4: subject (AgentID or NodeID hash) // who can present it
	•	5: scope (CapScope)
	•	6: budget (CapBudget)
	•	7: not_before_us (i64)
	•	8: expires_us (i64)
	•	9: parent_cap (Hash, optional) // attenuation
	•	10: nonce (bytes, optional)
	•	11: sig (Signature)

cap_id rule (normative for v0.2):
cap_id = SHA-256( canonical_bytes(Capability without fields 1 and 11) )

Signature rule:
	•	signature is Ed25519 over raw cap_id (32 bytes)

5. Attenuation chain

5.1 How attenuation works

To attenuate a cap, the holder creates a new Capability where:
	•	parent_cap references the parent’s cap_id
	•	scope is a subset of parent scope
	•	budget is <= parent budget
	•	time window is within parent time window
	•	subject may be narrowed (subset, usually a single node/agent)
	•	issuer MAY remain the same or MAY change depending on policy

v0.2 rule (simple, strong):
Attenuated caps MUST be signed by the same issuer as the root cap.

This avoids complex delegated signing in early versions. You can later add “delegated attenuation keys” as an extension.

5.2 Subset validation (node-side)

Node MUST validate:
	•	tools_allow(child) ⊆ tools_allow(parent)
	•	tools_deny(child) ⊇ tools_deny(parent) (child can deny more)
	•	budgets child <= parent per field
	•	not_before and expires are within parent window
	•	ring_max(child) <= ring_max(parent)
	•	cell_allow(child) ⊆ cell_allow(parent) if present

If parent_cap is missing, cap is a root cap.

5.3 Chain verification

Nodes SHOULD verify the full chain to a trusted issuer key.
Nodes MAY cache verified chains by cap_id.

6. Presentation and binding to effects

Every effect invocation MUST include:
	•	cap_ref (cap_id)
	•	cap_bytes (optional if node can fetch by ref)
	•	effect_budget_claim (budget to spend, must be <= cap_budget remaining)

Nodes MUST bind effect authorization to:
	•	tool_id/resource_id
	•	ring/cell context
	•	idempotency key (via ledger)

7. Revocation (v0.2 minimal)

Revocation is optional in v0.2.

If implemented, use:
	•	RevocationListRef (CID) distributed to nodes
	•	nodes check cap_id membership

8. Implementation notes (informative)
	•	Keep cap objects small. Put large policy in referenced objects.
	•	Cache validation results by cap_id.
	•	Tie caps to ledger spending to prevent budget abuse.

⸻

RFC-MYTHOS-0009

Evidence Bundles and Compression

Status: Draft
Version: 0.9.0-draft
Date: 2026-01-01
Related: RFC-0002, RFC-0005, RFC-0007

1. Purpose

Define a standard schema for bundling execution evidence:
	•	receipts
	•	ledger entries
	•	trace refs
	•	policy decisions
and a standard compression approach so evidence remains cheap to store and replicate.

2. EvidenceBundle object (normative)

2.1 EvidenceBundle

EvidenceBundle (struct):
	•	1: bundle_id (Hash) // computed
	•	2: version (u16) // v0.2 => 2
	•	3: task_ref (Hash)
	•	4: node_id (Hash NodeID)
	•	5: created_us (i64)
	•	6: receipts (list)
	•	7: ledger_entries (list, optional)
	•	8: traces (list, optional)
	•	9: artifacts (list, optional) // e.g., codebook, manifest, policies used
	•	10: metrics (map<text, i64/text/bytes>, optional)
	•	11: sig (Signature, optional)

bundle_id rule:
bundle_id = SHA-256( canonical_bytes(EvidenceBundle without fields 1 and 11) )

2.2 EvidenceBundleBlob (compressed transport)

EvidenceBundle references IDs. To ship “the actual payloads” efficiently, define a blob wrapper:

EvidenceBundleBlob (struct):
	•	1: version (u16)
	•	2: bundle (EvidenceBundle canonical bytes, uncompressed)
	•	3: compression (u8) // 0=none, 1=zstd
	•	4: packed_objects (bytes) // compressed or raw
	•	5: index (bytes) // small uncompressed index for random access
	•	6: packed_sha256 (Hash) // integrity

2.3 Packed object format (normative)

packed_objects contains a concatenation of:
	•	obj_len (uvarint)
	•	obj_bytes (MYTHOS-CAN bytes of the object)

index contains:
	•	list of (obj_id_hash32, offset_uvarint, len_uvarint)

Nodes/gateways can:
	•	validate packed_sha256
	•	decompress (if zstd)
	•	fetch a specific object by offset without parsing everything (optional)

3. Compression standard

v0.2 normative compression:
	•	Zstandard (zstd) level 3 default
	•	dictionary not required

Gateways/nodes MUST support:
	•	compression=0 (none)
	•	compression=1 (zstd)

4. Evidence integrity rules
	•	Every packed object MUST be verifiable by its own CID/hash if referenced.
	•	Bundle IDs are stable and content-addressable.
	•	Signature optional in v0.2; recommended for higher rings.

⸻

RFC-MYTHOS-0010

Node Identity and Routing Receipts

Status: Draft
Version: 1.0.0-draft
Date: 2026-01-01
Related: RFC-0002, RFC-0005, RFC-0007

1. Purpose

Standardize:
	•	Node identity format (NodeID)
	•	Cell identity (CellID)
	•	Routing receipts and delegation evidence

2. Node identity

2.1 NodeID

NodeID is a stable identifier derived from a public key.

NodeID (struct):
	•	1: scheme (u8) // 1=ed25519
	•	2: pubkey (bytes)
	•	3: hint (text, optional)

node_id_hash rule:
node_id_hash = SHA-256( canonical_bytes(NodeID) )

Nodes MUST present their NodeID during handshake, and peers refer to the node by node_id_hash.

2.2 CellID

A Cell is an administrative/security domain.

CellID (struct):
	•	1: version (u16)
	•	2: name (text)
	•	3: ring (u8)
	•	4: policy_root (Hash CID)
	•	5: registry_root (Hash CID, optional)
	•	6: created_us (i64)

cell_id_hash = SHA-256(canonical_bytes(CellID))

3. Routing as an auditable effect

Routing/delegation is represented as an effect/tool call:
	•	tool_id = SHA-256("tool:route.submit")

It MUST produce a receipt.

4. RoutingReceipt

RoutingReceipt (struct):
	•	1: receipt_id (Hash) // computed
	•	2: kind (u8) // 1=route_submit
	•	3: parent_task (Hash TaskRef)
	•	4: child_task (Hash TaskRef)
	•	5: from_node (Hash NodeID)
	•	6: to_node (Hash NodeID or Hash CellID)
	•	7: package_ref (Hash CID, optional)
	•	8: reason (text, optional)
	•	9: time_us (i64)
	•	10: status (u16) // 200 accepted, 409 duplicate, 403 denied, etc.
	•	11: sig (Signature, optional)

receipt_id rule:
receipt_id = SHA-256( canonical_bytes(RoutingReceipt without fields 1 and 11) )

5. Verification rules
	•	If sig present, signature verifies over receipt_id.
	•	Nodes MUST record route_submit in ledger under idempotency semantics like any other effect.
	•	Gateways SHOULD include RoutingReceipts in EvidenceBundles.

⸻

RFC-MYTHOS-0011

CID Propagation Gossip Protocol (Optional)

Status: Draft
Version: 1.1.0-draft
Date: 2026-01-01
Related: RFC-0004, RFC-0007, RFC-0010

1. Purpose

Define an optional, minimal protocol for propagating awareness of new CIDs across nodes so caches warm quickly and tasks can be routed based on object locality.

This is not required for correctness. Only for performance and convergence.

2. Design goals
	•	low overhead
	•	robust under partitions
	•	resistant to spam (bounded)
	•	CID-first, content-addressed

3. Message types (normative)

All messages are MYTHOS-CAN objects, carried over any transport (HTTP, QUIC, libp2p, etc.)

3.1 Have

Advertise possession of objects.

Have (struct):
	•	1: kind (u8) // 1=have
	•	2: from_node (Hash NodeID)
	•	3: cids (list)
	•	4: ttl_hops (u8) // default 3
	•	5: time_us (i64)
	•	6: sig (Signature, optional)

3.2 Want

Request objects.

Want (struct):
	•	1: kind (u8) // 2=want
	•	2: from_node (Hash NodeID)
	•	3: cids (list)
	•	4: max_bytes (u64)
	•	5: time_us (i64)
	•	6: sig (Signature, optional)

3.3 Offer

Offer objects or redirect to another node/cell.

Offer (struct):
	•	1: kind (u8) // 3=offer
	•	2: from_node (Hash NodeID)
	•	3: cid (Hash CID)
	•	4: size (u64)
	•	5: available (bool)
	•	6: redirect_node (Hash NodeID, optional)
	•	7: time_us (i64)

4. Propagation rules
	•	Nodes maintain a bounded LRU “seen” set of (cid, time).
	•	On receiving Have:
	•	store knowledge “node X has CID Y”
	•	forward Have to peers if ttl_hops > 0 and cid not recently seen
	•	Want is sent only when policy allows and bandwidth budget permits.
	•	Objects are fetched through the existing CAS/object fetch API, not through gossip payloads (keeps gossip small).

5. Spam resistance (normative)

Nodes MUST enforce:
	•	max cids per Have/Wan (e.g., 512)
	•	rate limits per peer
	•	optional signature requirement in higher rings
	•	denylist peers that exceed budgets

⸻

ADR-002

MCP Gateway is the Front Door, and the LLM is Not Trusted

Status: Accepted
Date: 2026-01-01
Related: RFC-0007

Context

MYTHOS aims to support execution across many nodes with:
	•	deterministic verification of artifacts
	•	auditable, idempotent real-world effects
	•	autonomous distribution and scaling

LLMs are powerful operators, but they are:
	•	non-deterministic
	•	prompt-injectable
	•	not inherently policy-compliant
	•	not a stable security principal

We need a system where LLMs can drive work, but cannot accidentally or maliciously bypass safety boundaries.

Decision

We adopt the pattern:

LLM/Caller → MCP Gateway → MYTHOS Node(s)

The MCP Gateway is the only externally exposed control plane, and it enforces:
	•	authentication and rate limiting
	•	ring policy (promotion boundaries)
	•	capability issuance/attenuation policies
	•	budget ceilings
	•	artifact publishing rules
	•	audit logging / evidence aggregation

MYTHOS Nodes treat the gateway as helpful, not authoritative:
	•	they verify all CIDs and signatures themselves
	•	they enforce caps and budgets locally
	•	they enforce ledger/idempotency locally
	•	they fail closed if inputs are invalid

Why the LLM is not trusted (rationale)

1) Prompt injection is inevitable

Any tool exposed to untrusted input can be manipulated. If the LLM can directly invoke high-risk effects, an attacker can route around policy by crafting input.

2) LLMs cannot be security principals

They don’t own keys, they don’t prove identity, and their output is probabilistic.

3) Determinism requires a stable substrate

If the same intent can produce different actions, verification must happen below the LLM layer.

4) Autonomy must be gated

Promotion, capability expansion, and broad effects require attestations and policy checks, not “model confidence”.

Consequences

Positive
	•	Clear trust boundary
	•	Easier audits (one choke point)
	•	Safer scaling (caps + budgets enforced centrally and locally)
	•	Better interoperability (stable MCP tools)

Negative
	•	Gateway becomes a high-value target
	•	Additional hop adds latency
	•	Requires careful policy configuration

Mitigations
	•	Gateway hardened: auth, rate limits, audit logs, least privilege
	•	Nodes verify everything; assume gateway compromise is possible
	•	Separate “publish/promote” from “execute/submit”
	•	Use ring deployment and capability attenuation

Guardrails (non-negotiable)
	•	No raw effect execution without caps
	•	All effects produce receipts
	•	Ledger enforces idempotency
	•	Higher rings require attestations for promotion
	•	Nodes fail closed on any verification failure

⸻

If you want, I can also amend RFC-0007’s “Open questions” section to reference these RFCs explicitly (0008–0011) so the doc links are clean and navigable.
