RFC-MYTHOS-0007

MCP Gateway and Node Control Plane

Status: Draft
Version: 0.7.0-draft
Date: 2026-01-01
Related: RFC-MYTHOS-0001 (Wire), RFC-MYTHOS-0002 (Core objects), RFC-MYTHOS-0004 (Merkle/Blobs), RFC-MYTHOS-0005 (Ledger), RFC-MYTHOS-0006 (Package + Module Execution Contract, draft)

⸻

1. Purpose

This RFC defines the standard control plane for operating MYTHOS at scale:
	•	an MCP Gateway that exposes a small, stable API surface to LLMs and human operators
	•	a MYTHOS Node API that allows nodes to execute tasks, fetch/verify artifacts, route work to other nodes, and return verifiable evidence (receipts, ledger state, trace refs)

The goal is to allow an LLM to talk to one MCP server and indirectly interact with many nodes without the LLM being in the trust boundary for effects, promotion, or privilege.

⸻

2. Non-goals

This RFC does not define:
	•	the full package format (RFC-0006)
	•	consensus algorithms or global byzantine safety
	•	the internals of learning/training loops
	•	UI/UX or operator dashboards
	•	full mesh network protocols (gossip, QUIC, etc.)

It defines a minimum viable, stable contract between:
	•	MCP client (LLM)
	•	MCP Gateway
	•	Node(s)

⸻

3. Definitions

MCP Gateway: An MCP server presenting “mythos.*” tools/resources. It authenticates callers, enforces policy, and proxies to nodes.

Node: A runtime that can verify artifacts, execute tasks (IR), dispatch tool modules, enforce caps/budgets, and emit receipts/ledger entries.

Cell: A set of nodes sharing the same policy/ring configuration and usually the same ledger boundary.

CAS: Content Addressed Storage keyed by digest/CID (SHA-256 in v0.2). Stores artifacts, blobs, manifests, receipts, traces.

TaskRef: Stable identifier for a submitted task, typically a Hash/CID of the canonical Task object or submission envelope.

ReceiptRef: Stable identifier for receipts emitted by tool/effect execution.

Ring: Promotion / risk tier boundary (e.g., ring0 canary, ring1 limited, ring2 production).

⸻

4. Design principles

4.1 LLM out of the trust boundary

The LLM must never be able to:
	•	mint unbounded capabilities
	•	bypass idempotency/ledger constraints
	•	promote packages or policies without attestations
	•	submit unsigned artifacts directly into production rings

4.2 CID-first operation

All durable objects are referenced by content hash:
	•	Packages, IR bundles, blobs, receipts, traces, datasets
Replication and distribution are “boring” (cache by CID).

4.3 Evidence-first execution

Anything that touches the world produces:
	•	receipts
	•	ledger entries
	•	trace references
These are first-class outputs of the control plane, not logs.

4.4 Small surface area

Expose a tiny set of tools/resources and keep them stable. Add capability via composition, not sprawl.

⸻

5. Actors and trust boundaries

5.1 Actors
	•	Caller: LLM or human tool client using MCP
	•	Gateway: Policy-enforcing MCP server
	•	Node: Execution engine
	•	Registry/CAS: Artifact store (may be local or remote)
	•	Ledger service: Idempotency and effect history boundary (embedded or external)

5.2 Trust boundaries
	•	Caller is untrusted
	•	Gateway is semi-trusted (it enforces policy; should be audited)
	•	Node is trusted for enforcement, but must verify everything (defense in depth)
	•	CAS is untrusted storage (verification by hash/signature required)

⸻

6. Control plane layers

6.1 MCP Gateway responsibilities
	•	Authenticate and rate limit callers
	•	Translate MCP tool calls into Node API calls
	•	Enforce:
	•	ring policy
	•	cap issuance/attenuation rules
	•	budget ceilings
	•	package allowlists/denylists
	•	Provide streaming status/results
	•	Keep caller UX simple: “submit/status/results/publish/fetch”

6.2 Node responsibilities
	•	Verify packages/artifacts by CID + signature + attestations
	•	Execute tasks deterministically (except effects)
	•	Dispatch tool modules per RFC-0006
	•	Enforce:
	•	caps
	•	budgets
	•	idempotency ledger
	•	Emit receipts and trace refs
	•	Route/delegate tasks to other nodes if allowed

⸻

7. MCP Gateway API (Normative)

The Gateway MUST expose the following MCP tools (names are normative).

7.1 mythos.submit

Submit a task or package for execution.

Input (conceptual):
	•	package_ref: CID or inline package bundle (inline allowed only in ring0)
	•	entrypoint: task id or symbol
	•	inputs: MYTHOS-CAN bytes or JSON (gateway converts)
	•	ring: requested ring (default ring0)
	•	budget: requested budget (bounded by policy)
	•	caps_request: requested caps (bounded by policy)
	•	idempotency_key: optional explicit key; otherwise derived deterministically

Output:
	•	task_ref (CID)
	•	admission (accepted/rejected + reason)
	•	stream_token (optional for incremental updates)

Normative behavior:
	•	Gateway MUST validate ring/caps/budget policy before forwarding.
	•	Gateway MUST attach caller identity to an audit trail object (CID) if enabled.

7.2 mythos.status

Return status/progress for a task.

Input:
	•	task_ref

Output:
	•	state: queued | running | waiting_on_effect | complete | failed | rejected
	•	node: executing node id
	•	progress: optional structured progress
	•	latest_receipts: list of ReceiptRefs
	•	logs: optional, bounded, non-authoritative

7.3 mythos.results

Return final result and evidence bundle.

Input:
	•	task_ref

Output:
	•	result: canonical result object (or CID)
	•	receipts: list (or CID to receipt bundle)
	•	trace_refs: list (or CID)
	•	ledger_refs: list (or CID)
	•	metrics: optional

Normative behavior:
	•	If result is large, gateway SHOULD return a CID and require mythos.fetch.

7.4 mythos.publish

Publish an artifact/package to CAS/registry.

Input:
	•	artifact_bytes OR artifact_path (path only in trusted gateway deployments)
	•	media_type
	•	signing_policy: dev | prod (prod requires key availability)
	•	ring_target: ring0 default

Output:
	•	cid
	•	signature_ref (optional)
	•	promotion_state: pending | accepted | rejected

Normative behavior:
	•	Gateway MUST refuse publish to higher rings without required signatures/attestations.

7.5 mythos.fetch

Fetch an object by CID.

Input:
	•	cid
	•	as: bytes | decoded | json (decoded/json are best-effort views)

Output:
	•	raw bytes OR decoded representation
	•	verification status (hash verified, sig verified, attested?)

Normative behavior:
	•	Gateway MUST verify CID matches content before returning bytes if it fetched remotely.

⸻

8. Node API (Normative)

The node exposes a non-MCP API that the gateway calls. This can be HTTP, gRPC, or QUIC. Protocol is out of scope, but semantics are normative.

8.1 SubmitTask
	•	Accepts a submission envelope.
	•	Returns TaskRef or rejection reason.

8.2 GetTaskStatus
	•	Returns state, receipts so far, where it is running.

8.3 GetTaskResult
	•	Returns final result CID + evidence bundle CID.

8.4 FetchObject
	•	Returns object bytes for a CID (only if present or fetchable by policy).

8.5 VerifyObject
	•	Returns verification result: hash ok, signature ok, attestation ok.

8.6 RouteTask (Optional)
	•	Allows delegating a task to another node/cell.
	•	Must be policy gated.

⸻

9. Routing and delegation model

9.1 Delegation as an effect

Delegation SHOULD be represented as a tool effect:
	•	EFFECT(route.submit, payload={task_ref, target_policy})

This ensures:
	•	delegation produces receipts
	•	delegation is idempotent
	•	delegation is auditable

9.2 Routing policies

Routing decisions may use:
	•	ring compatibility
	•	data locality (CAS proximity)
	•	capability availability
	•	budget limits
	•	load

Routing policy is intentionally pluggable.

⸻

10. Capabilities and budgets at the control plane

10.1 Caps issuance

Gateway MAY issue caps to callers, but MUST attenuate:
	•	scope (which tools)
	•	rate (max calls)
	•	budget (cpu/io/state)
	•	duration (expiry)

10.2 Caps verification

Node MUST treat caps as untrusted input and verify:
	•	signature
	•	expiry
	•	scope
	•	attenuation chain

10.3 Budgets

Budgets MUST be enforced at node:
	•	cpu time
	•	memory
	•	io calls
	•	state writes
	•	wall clock timeouts

Gateway enforces ceilings, node enforces actuals.

⸻

11. Evidence model

11.1 Receipts

Any external effect MUST yield a ReceiptRef.

11.2 Trace refs

Nodes SHOULD emit trace references for:
	•	task execution
	•	module dispatch
	•	policy decisions
	•	routing events

11.3 Evidence bundles

To reduce chattiness, nodes SHOULD aggregate evidence into bundles:
	•	EvidenceBundleRef containing:
	•	receipts
	•	ledger entries
	•	trace refs
	•	metrics

Gateway returns bundle CID by default.

⸻

12. Failure modes and required behavior

12.1 Partial results

If execution fails after some effects:
	•	node MUST return receipts and ledger entries that occurred
	•	node MUST mark task as failed with a canonical error object

12.2 Idempotency collisions

If a submission repeats with the same idempotency key:
	•	node MUST return the existing result/receipt bundle and MUST NOT re-execute effects.

12.3 Non-canonical inputs

Gateway SHOULD reject malformed inputs early, but node MUST reject:
	•	non-canonical MYTHOS-CAN objects
	•	invalid signatures
	•	invalid CIDs

Fail closed.

⸻

13. Minimal implementation roadmap (informative)

Phase 0:
	•	Gateway exposes submit/status/results/fetch
	•	One node behind it
	•	Local CAS

Phase 1:
	•	Multiple nodes in one cell
	•	Routing by load
	•	Shared CAS

Phase 2:
	•	Multiple cells (rings)
	•	Promotion gates enforced by attestations
	•	Registry + index service

⸻

14. Security considerations
	•	Gateway is a high-value target: require auth, rate limits, audit logs.
	•	Nodes must verify everything, assume gateway compromise is possible.
	•	Don’t allow raw network/file/time access to modules without caps.
	•	Treat CAS as untrusted storage: verify hashes/signatures always.
	•	Keep “publish/promote” separate from “submit/execute”.

⸻

15. Open questions
	•	Standardize cap format and attenuation chain (likely RFC-0008).
	•	Standardize evidence bundle schema and compression.
	•	Define routing receipts and node identity format.
	•	Define optional gossip protocols for CID propagation.

⸻

Appendix A: Recommended tool/resource naming
	•	Tools: mythos.submit, mythos.status, mythos.results, mythos.publish, mythos.fetch
	•	Resources (optional): mythos/cells, mythos/nodes, mythos/packages/{cid}, mythos/tasks/{task_ref}

⸻

If you want, I can also produce a companion ADR (“Why MCP Gateway is the front door and why the LLM is not trusted”) that you can drop into the repo alongside this RFC.
