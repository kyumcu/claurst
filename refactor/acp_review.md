# ACP Module Review

Module under review: `src-rust/crates/acp`

Why this module next: `acp` is the editor-facing JSON-RPC integration surface. Bugs here do not just affect one internal feature; they affect how external clients understand the server’s capabilities and whether session, tool, and model operations are trustworthy.

## Findings

### 1. High: `session/create` returns a session ID that is not persisted or usable by `session/message`

Files:
- `src-rust/crates/acp/src/lib.rs:184`
- `src-rust/crates/acp/src/lib.rs:199`

`session/create` fabricates a timestamp-based `session_id` and returns `"status": "created"`, but it does not create any real session record in history or keep any in-memory session registry.

`session/message` then accepts any request and returns `"status": "accepted"` without:

- verifying the session exists
- storing the message
- invoking the query loop
- producing a response tied to that session

So the ACP session API exposes identifiers that do not correspond to any actual session lifecycle.

Impact:
- ACP clients can believe they created a usable session when none exists
- follow-up messages succeed syntactically but do nothing
- editor integrations cannot rely on ACP sessions as a real conversation primitive

### 2. High: ACP capability and method surface overstates what the server actually implements

Files:
- `src-rust/crates/acp/src/lib.rs:88`
- `src-rust/crates/acp/src/lib.rs:149`
- `src-rust/crates/acp/src/lib.rs:199`

The server announces:

- `sessions.create = true`
- `sessions.list = true`
- `tools.list = true`
- `models.list = true`

and documents `session/message` as a supported method.

But the only session mutation method is an explicit placeholder, and the server does not provide any way to get an actual assistant turn back from a sent message. The ACP surface therefore reads like a working backend, while a key method is only a stub.

Impact:
- ACP clients are encouraged to depend on behaviors that are not implemented
- integration failures show up late at runtime rather than during capability negotiation
- the protocol contract is misleading for editor/plugin authors

### 3. Medium: `tool/list` and `model/list` return static or snapshot data instead of live runtime state

Files:
- `src-rust/crates/acp/src/lib.rs:165`
- `src-rust/crates/acp/src/lib.rs:218`

`tool/list` returns a hardcoded list of tool names/descriptions instead of the real configured tool set. That means it cannot reflect:

- disabled tools
- agent-mode filtering
- plugin-added tools
- MCP-exposed tools

`model/list` constructs a fresh `ModelRegistry::new()` and reports its bundled snapshot, not the live provider/model environment for the running process.

Impact:
- ACP clients see capabilities that may not match the active runtime
- external integrations cannot trust ACP discovery to mirror the actual server configuration
- local/provider-backed dynamic model availability is hidden behind a static snapshot

### 4. Medium: ACP accepts malformed JSON-RPC envelopes without validating protocol version or request shape

Files:
- `src-rust/crates/acp/src/lib.rs:29`
- `src-rust/crates/acp/src/lib.rs:114`
- `src-rust/crates/acp/src/lib.rs:143`

Requests are deserialized into `JsonRpcRequest`, but the server never checks:

- `jsonrpc == "2.0"`
- whether notifications should suppress responses
- whether `id` is present for request methods that expect replies

Everything that parses into the struct is dispatched as if it were a valid request.

Impact:
- protocol compliance is looser than JSON-RPC 2.0 clients may expect
- malformed client messages can receive misleading normal responses
- interoperability/debugging becomes harder when the server accepts invalid envelopes

## Fix Plan

### 1. Make ACP sessions real or remove the fake session lifecycle

Approach:
- either back `session/create` and `session/message` with real session storage/query execution
- or explicitly downgrade ACP to a stateless request surface until session support exists
- do not return synthetic session IDs unless they are actually usable

Implementation target:
- `src-rust/crates/acp/src/lib.rs`

Verification:
- create a session, send a message, and confirm the session exists in history and produces a real model turn
- if stateless mode is chosen, ensure unsupported session methods return clear method errors instead

### 2. Align advertised ACP capabilities with implemented behavior

Approach:
- only advertise capabilities that are fully supported
- either implement `session/message` end-to-end or remove it from the supported method surface
- make the startup `server/ready` payload match the initialize response and real runtime

Implementation target:
- `src-rust/crates/acp/src/lib.rs`

Verification:
- capability negotiation and actual method behavior stay consistent
- unsupported methods are rejected explicitly rather than acknowledged as placeholders

### 3. Replace static discovery responses with runtime-backed discovery

Approach:
- build `tool/list` from the actual configured tool registry
- build `model/list` from the live provider/model registry or explicitly mark it as a bundled snapshot if that is all ACP can offer
- keep ACP discovery aligned with the running process configuration

Implementation target:
- `src-rust/crates/acp/src/lib.rs`

Verification:
- disabled/plugin/MCP tools are reflected accurately in `tool/list`
- provider-backed model availability is reflected accurately in `model/list`

### 4. Enforce basic JSON-RPC 2.0 request validation

Approach:
- validate `jsonrpc == "2.0"`
- distinguish requests from notifications based on `id`
- return proper protocol errors for invalid envelopes

Implementation target:
- `src-rust/crates/acp/src/lib.rs`

Verification:
- malformed or non-2.0 envelopes receive the correct JSON-RPC error behavior
- notifications no longer receive inappropriate normal responses

## Recommended execution order

1. make ACP sessions real or remove the fake session contract
2. align advertised capabilities with implemented behavior
3. replace static discovery responses with runtime-backed discovery
4. enforce basic JSON-RPC 2.0 validation
