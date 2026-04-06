# MCP Module Review

Module under review: `src-rust/crates/mcp`

Why this module next: `mcp` is the integration layer for external tool/resource servers. Bugs here directly affect tool availability, OAuth flows, and connection lifecycle for a large part of the app’s extension surface.

## Findings

### 1. High: the connection manager and reconnect logic only use stdio, despite the crate advertising HTTP/SSE support

Files:
- `src-rust/crates/mcp/src/connection_manager.rs:122`
- `src-rust/crates/mcp/src/connection_manager.rs:142`
- `src-rust/crates/mcp/src/connection_manager.rs:338`
- `src-rust/crates/mcp/src/lib.rs:1045`
- `src-rust/crates/mcp/src/lib.rs:1053`

`McpConnectionManager::connect()` and its reconnect loop always call `McpClient::connect_stdio(&config)`, regardless of `server_type`.

The higher-level `McpManager::connect_all()` also only supports `"stdio"` and marks every other transport as unsupported. That directly contradicts the crate-level documentation claiming HTTP/SSE transport support.

Impact:
- HTTP and SSE MCP servers cannot connect through the main runtime path
- reconnect behavior is wrong for any non-stdio server even if transport support exists elsewhere
- the documented transport surface is wider than the actual production behavior

### 2. High: MCP OAuth initiation generates a PKCE verifier but never preserves it for code exchange

Files:
- `src-rust/crates/mcp/src/lib.rs:1360`
- `src-rust/crates/mcp/src/lib.rs:1427`
- `src-rust/crates/mcp/src/lib.rs:1431`
- `src-rust/crates/mcp/src/oauth.rs:162`

`initiate_auth()` generates a PKCE verifier and derives the challenge, but it only returns the authorization URL. The verifier is not stored anywhere, and it is not actually embedded in the URL despite the comment claiming that it is.

`oauth::exchange_code()` requires the original verifier, so the flow cannot be completed correctly from the information returned by `initiate_auth()` alone.

Impact:
- MCP OAuth login flow is incomplete by construction
- code exchange cannot be performed reliably after browser auth
- users can be sent into an auth flow that cannot finish successfully

### 3. Medium: reconnect-loop bookkeeping is never cleaned up after a successful reconnect

Files:
- `src-rust/crates/mcp/src/connection_manager.rs:255`
- `src-rust/crates/mcp/src/connection_manager.rs:272`
- `src-rust/crates/mcp/src/connection_manager.rs:348`
- `src-rust/crates/mcp/src/connection_manager.rs:349`

`start_reconnect_loop()` inserts a task handle into `reconnect_handles`. When the reconnect loop succeeds, it breaks out of the loop and the task exits, but its handle is never removed from the map.

Future calls to `start_reconnect_loop()` for that server will see the stale map entry and refuse to start a new reconnect loop.

Impact:
- reconnect works at most once per server lifecycle
- later disconnects can leave a server permanently without automatic recovery
- state in the manager no longer reflects the actual running task set

### 4. Medium: prefixed MCP tool routing is ambiguous when server names overlap

Files:
- `src-rust/crates/mcp/src/lib.rs:1166`
- `src-rust/crates/mcp/src/lib.rs:1183`
- `src-rust/crates/mcp/src/lib.rs:1191`

Tool routing uses a string prefix convention:

- tool IDs are exposed as `<server_name>_<tool_name>`
- `call_tool()` finds the first server whose prefix matches

That becomes ambiguous if server names overlap, such as:

- `a`
- `a_b`

In that case, `a_b_tool` can match the shorter `a_` prefix first depending on map iteration order, sending the call to the wrong server with the wrong tool name.

Impact:
- MCP tool dispatch can be nondeterministic for overlapping server names
- the wrong remote tool can be invoked
- error messages can point at the wrong server, making debugging harder

## Fix Plan

### 1. Unify MCP connection handling around real transport dispatch

Approach:
- add a transport-aware connection helper that chooses the correct `McpClient` connect path from `server_type`
- use that helper in:
  - `McpManager::connect_all()`
  - `McpConnectionManager::connect()`
  - the reconnect loop
- make unsupported transports explicit only when there is truly no implementation

Implementation targets:
- `src-rust/crates/mcp/src/lib.rs`
- `src-rust/crates/mcp/src/connection_manager.rs`

Verification:
- connect stdio, HTTP, and SSE servers through the same runtime path
- confirm reconnect uses the same transport as the original connection

### 2. Make the MCP OAuth PKCE flow stateful and completable

Approach:
- have `initiate_auth()` return or persist the verifier and redirect metadata required for `exchange_code()`
- store that state in a short-lived pending-auth structure keyed by server/session
- make the command/UI layer consume that state instead of trying to reconstruct it later

Implementation targets:
- `src-rust/crates/mcp/src/lib.rs`
- `src-rust/crates/mcp/src/oauth.rs`

Verification:
- start auth, complete browser redirect, exchange the code using the original verifier
- confirm the resulting token is stored and `auth_state()` becomes authenticated

### 3. Clean up reconnect task handles when loops exit

Approach:
- remove the server entry from `reconnect_handles` when the reconnect task completes, whether by success or terminal exit
- optionally move the cleanup into a small wrapper task so map maintenance happens in one place

Implementation target:
- `src-rust/crates/mcp/src/connection_manager.rs`

Verification:
- trigger reconnect success, then trigger another disconnect
- confirm a fresh reconnect loop can start again

### 4. Replace prefix-based MCP tool routing with an unambiguous mapping

Approach:
- store an explicit map from exported tool ID to `(server_name, tool_name)`
- avoid deriving routing from string prefix matching over the current client map
- if string encoding is kept, use a delimiter/escaping scheme that cannot collide with server names

Implementation target:
- `src-rust/crates/mcp/src/lib.rs`

Verification:
- add tests with overlapping server names such as `a` and `a_b`
- confirm `a_b_tool` always routes to server `a_b`

## Recommended execution order

1. unify transport-aware connection handling
2. make the OAuth PKCE flow completable
3. fix reconnect-handle cleanup
4. replace ambiguous prefix-based tool routing
