MYTHOS Developer Playbook

Building a single distributable package that supports multiple languages at infinite scale

Audience: engineers implementing MYTHOS as a real, deployable substrate
Goal: ship a single distributable “MYTHOS Package” that can contain modules written in multiple languages (WASM, Node, Python, native), and can be replicated and executed safely at massive scale with deterministic verification, receipts, and idempotency.

⸻

0. The core idea you build toward

A “single package” is not “one binary”. It’s a content-addressed bundle with a manifest that maps:
	•	Tasks → IR bundles (or entrypoints)
	•	Tools → modules (WASM primary, sidecars optional)
	•	Policies/Codebooks/CTVP → referenced artifacts
	•	Signatures/Attestations → verification gates

The executor becomes the only universal “loader + verifier + dispatcher”.

⸻

1. Establish the packaging standard first

Before writing runtime code, define the package format you will actually ship.

1.1 Create RFC-MYTHOS-0006: Package Format + Module Execution Contract

Add a new RFC that defines:

PackageManifest (canonical MYTHOS-CAN)
	•	package_id (Hash)
	•	format_version
	•	requires
	•	executor_min_version
	•	codebook_id
	•	ctvp_id (optional but recommended)
	•	artifacts[] (each by kind, cid, media_type, size, sha256)
	•	modules[] (maps tool_id → ModuleRef)
	•	entrypoints[] (task IDs or symbols → IR bundle refs)
	•	signatures[] (Ed25519 now, future expandable)
	•	attestations[] (promotion/eval proofs, optional early)

ModuleRef
	•	kind: wasm | process | oci-image | native
	•	cid: content hash for the module artifact
	•	entry: for wasm (_start), for process (command), for oci-image (image digest + command)
	•	caps_required[]: list of capability types the module expects
	•	timeouts, memory_limit, io_limit

Module ABI (how modules communicate)
	•	Use MYTHOS-CAN messages over:
	•	WASM hostcalls (preferred)
	•	stdin/stdout framed messages (for process modules)
	•	Define two canonical message structs:
	•	ToolRequest
	•	ToolResponse
	•	Require: request_hash/response_hash computed over canonical bytes

This RFC is what makes “multi-language, one package” real.

1.2 Decide your default module strategy

Adopt this default policy:
	•	WASM is the primary module type (portable, sandboxed, replicable).
	•	Process/OCI sidecars exist only for integration-heavy tools (DB drivers, SDKs, vendor APIs) until they get WASM equivalents.

This keeps security and replication sane.

⸻

2. Organize the repo so it scales with the product

Your current repo is docs + CTVP. Keep that, but structure for code and packages now.

2.1 Restructure the repository

Create these top-level folders:
	•	spec/
RFCs, whitepaper, roadmap, CTVP source
	•	ctvp/
the actual conformance vector pack files (what you already have)
	•	runtime/
executor + core libs implementation
	•	packages/
example MYTHOS packages (manifest + modules + IR bundles)
	•	tools/
reference tool modules (WASM-first)
	•	registry/
(optional early) artifact store + signer/verifier CLI
	•	scripts/
packaging/build helpers, CI helpers

Acceptance check: anyone new to the repo can tell where “spec vs runtime vs packages” live instantly.

⸻

3. Implement the minimal runtime spine

Don’t build services yet. Build the part that makes packages executable.

3.1 Implement core libraries (single source of truth)

Create runtime/libs/ and implement these in the same language (recommend Rust):
	•	mythos-can
encoder/decoder exactly per CTVP SPEC.md
	•	mythos-hash
SHA-256, Hash struct helpers
	•	mythos-merkle
RFC 0004 MerkleList + ChunkedBlob support
	•	mythos-receipts
receipt_id rules, signature verification
	•	mythos-ledger
RFC 0005 state machine (start in-memory)
	•	mythos-x
packet framing parser + signature verification
	•	mythos-package
PackageManifest parsing/verification (RFC 0006)
	•	mythos-ir
IR validator + minimal opcode executor

Acceptance check: you can run CTVP locally and it passes for all implemented categories.

3.2 Build the “Executor Core”

Implement an executable mythos-executor that can:
	1.	Load a PackageManifest (from disk first, CID fetch later)
	2.	Verify:
	•	package signature
	•	codebook compatibility
	•	CTVP compatibility (optional but recommended)
	3.	Load IR bundle(s) referenced by entrypoints
	4.	Execute IR deterministically
	5.	When encountering EFFECT(tool_id, payload, idempotency_key):
	•	dispatch to the module defined in manifest
	•	enforce caps
	•	generate receipt
	•	record ledger state

Acceptance check: you can run a demo package that returns "hello" and can call one tool module.

⸻

4. Make “single package, multiple languages” real

This is the part you care about. Do it by implementing two module runners first.

4.1 Module Runner #1: WASM (primary)

Implement a WASM runner with strict sandbox policy:
	•	no filesystem unless cap-granted
	•	no network unless cap-granted (and ideally proxied through Tool Gateway later)
	•	deterministic time (either denied or provided as a receipt-backed effect)
	•	memory and instruction limits

WASM Tool ABI
	•	input: ToolRequest (MYTHOS-CAN)
	•	output: ToolResponse (MYTHOS-CAN)
	•	executor computes request_hash/response_hash
	•	module never touches ledger directly

Acceptance check: a WASM tool called echo returns the payload unmodified, receipt produced, idempotency enforced.

4.2 Module Runner #2: Process (secondary)

Implement a process runner for Node/Python modules:
	•	executor spawns the process with a command from ModuleRef
	•	communicates over stdin/stdout with a framed protocol:
	•	frame header: length varint
	•	payload: MYTHOS-CAN bytes
	•	enforce:
	•	timeout
	•	max output bytes
	•	kill-on-violation

Acceptance check: same echo tool implemented in Node can be executed under the same ToolRequest/ToolResponse contract and still produces valid receipts.

⸻

5. Implement packaging as a first-class build artifact

Now that the runtime can load packages, build the packaging toolchain.

5.1 Create mythos pack CLI

Implement a CLI that:
	1.	Takes a package folder (packages/demo-001/) with:
	•	manifest.src.json (human-friendly)
	•	ir/ artifacts
	•	modules/ artifacts (wasm, node, python)
	•	policy/, codebook/, ctvp/ refs
	2.	Converts manifest.src.json into canonical MYTHOS-CAN PackageManifest.bin
	3.	Computes CIDs for every artifact
	4.	Writes manifest.json (debug view) and manifest.bin (canonical)
	5.	Signs manifest.bin with dev key for local testing
	6.	Emits a single distributable folder or archive:
	•	package.mythos.tgz (simple)
	•	later: OCI artifact (better)

Acceptance check: pack output is deterministic (repacking yields identical hashes if inputs unchanged).

5.2 Decide the distribution container

Start simple, then graduate:

Phase 1: .tgz (manifest + artifacts)
Phase 2: OCI Artifact (digest-addressed, registry-friendly)

Acceptance check: executor can install and run from either.

⸻

6. Build the replication story (what makes it “infinite”)

“Infinite scale” is replication by hash + verification, not trust-by-network.

6.1 Add a local CAS store (content-addressed storage)

Implement a local store:
	•	keyed by SHA-256 digest
	•	stores: package manifests, wasm modules, IR bundles, policies, CTVP packs
	•	exposes:
	•	put(bytes)->cid
	•	get(cid)->bytes

Acceptance check: executor never needs “file paths”, it can load by CID.

6.2 Add remote distribution (artifact registry)

Implement the simplest registry you can:
	•	HTTP API:
	•	PUT /objects/{cid}
	•	GET /objects/{cid}
	•	store objects in S3/Azure Blob
	•	metadata stored in a KV or DB
	•	add optional “index” for package IDs → manifest CID

Acceptance check: any node can reconstruct the package from just the manifest CID.

⸻

7. Add safety gates so autonomy doesn’t destroy you

Once packages can distribute, they can also propagate bad behavior. Gates are non-negotiable.

7.1 Ring deployment gates

Implement ring enforcement in executor:
	•	ring0: allow running new packages/modules with limited caps
	•	ring1: broader caps, but promotion requires attestations
	•	ring2: production caps only for promoted artifacts

Acceptance check: unpromoted modules cannot acquire high-risk caps.

7.2 Capability enforcement

Make caps unavoidable:
	•	every tool invocation must specify cap proof
	•	executor validates cap scope before dispatch
	•	caps are attenuable and chain-verifiable

Acceptance check: effects without caps fail closed.

7.3 Ledger as the idempotency firewall

Implement RFC 0005 ledger rules:
	•	register before invoke
	•	commit after receipt
	•	retries return existing receipt instead of re-invoking tool
	•	divergence fails closed

Acceptance check: the same idempotency key never triggers multiple real effects.

⸻

8. Make conformance automatic in CI

This ensures “portable and replicable” stays true as you iterate.

8.1 Add a conformance runner

Build mythos ctvp verify that:
	•	loads your ctvp/ directory
	•	validates all *.bin hash files
	•	re-encodes expected objects and matches exact bytes where applicable
	•	verifies signatures
	•	recomputes merkle CIDs
	•	validates wire packet parsing

Acceptance check: a clean checkout passes CTVP in CI on every merge.

8.2 Add cross-language conformance

If you implement encoder/decoder in multiple languages (Rust + TS), require both to pass the same vectors.

Acceptance check: Rust and TS produce byte-identical encodings for the same objects.

⸻

9. Deliver the first real end-to-end package

This is the “prove it works” milestone.

9.1 Build packages/demo-001

Include:
	•	IR bundle: returns “hello”, then calls tool echo, then returns response
	•	WASM module: echo.wasm
	•	PackageManifest with tool mapping and entrypoint
	•	Signed manifest

Acceptance check: one command runs the package and emits:
	•	return value
	•	receipts
	•	ledger entries

9.2 Build packages/demo-002 (multi-language)

Include:
	•	same IR
	•	tool echo implemented in both:
	•	wasm (primary)
	•	node process runner (fallback)
	•	manifest chooses wasm by default, node behind a feature flag

Acceptance check: switching runner still yields receipts and idempotency behavior identical.

⸻

10. What you intentionally do not build yet

To avoid wasting time early:
	•	don’t build a full orchestration scheduler yet
	•	don’t build multi-region ledger sharding yet
	•	don’t build full policy registry UI yet
	•	don’t build fancy human syntax yet

You build:
	•	canonical bytes
	•	package manifest
	•	module runners
	•	receipts + ledger
	•	CAS + distribution by CID

That’s the foundation for “infinite replication”.

⸻

Output artifacts a developer should be able to produce

By the end of this playbook, a developer can produce these exact deliverables:
	1.	mythos-executor binary
	2.	mythos pack tool
	3.	mythos ctvp verify tool
	4.	package.mythos.tgz (or OCI artifact) containing:
	•	manifest.bin (canonical)
	•	manifest.sig (signature)
	•	ir/*.bin
	•	modules/*.wasm plus optional node/python modules
	•	policy/codebook refs
	5.	Execution outputs:
	•	receipts
	•	ledger entries
	•	logs/traces (optional)

⸻

If you want the next document

The next “missing spec” is what I’d write next as a formal doc for your repo:

RFC-MYTHOS-0006: Package Format + Module Execution Contract
It becomes the dev team’s exact blueprint for implementing the manifest + runners + signing rules, so multi-language packaging isn’t ad hoc.
