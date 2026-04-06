# Bridge Module Review

Module under review: `src-rust/crates/bridge`

Why this module next: `bridge` is the remote-control/session-sharing path between the local CLI and the remote web UI. Bugs here affect registration, remote prompts, permission responses, and the integrity of bidirectional session state.

## Findings

### 1. High: outbound bridge events are silently dropped on upload failures

Files:
- `src-rust/crates/bridge/src/lib.rs:678`
- `src-rust/crates/bridge/src/lib.rs:683`
- `src-rust/crates/bridge/src/lib.rs:684`

`run_poll_loop()` drains all pending outbound events from `event_rx` into a local `Vec`, then calls `upload_events(events).await`. If upload fails, the error is only logged and the drained events are discarded.

There is no retry queue, no reinsert into the channel, and no durable buffering. Any transient network failure therefore loses:

- text deltas
- tool start/end events
- turn completion events
- error/session metadata events

Impact:
- remote web UI can miss chunks of a session permanently
- tool traces and turn boundaries can become incomplete or misleading
- transient bridge/network failures cause irreversible state loss

### 2. High: the outer bridge loop reports disconnect but never re-registers or recreates the poll loop

Files:
- `src-rust/crates/bridge/src/lib.rs:1436`
- `src-rust/crates/bridge/src/lib.rs:1458`
- `src-rust/crates/bridge/src/lib.rs:1465`

`run_bridge_loop()` performs registration once, spawns `session.run_poll_loop(...)` once, then treats `msg_rx.recv() == None` as terminal by sending `Disconnected` and breaking.

If the inner poll loop exits because of repeated poll errors or reconnect exhaustion, the outer loop never attempts:

- re-registration
- poll-loop restart
- session replacement

So the bridge announces a disconnect to the TUI but does not actually implement the higher-level recovery flow implied by its reconnect events and bridge-state model.

Impact:
- remote control dies permanently after poll-loop failure
- reconnect UX is only partial and can stop at a disconnected state
- runtime behavior does not match the module’s stated reconnect semantics

### 3. Medium: bridge session timeout is configured but unused

Files:
- `src-rust/crates/bridge/src/lib.rs:169`
- `src-rust/crates/bridge/src/lib.rs:183`
- `src-rust/crates/bridge/src/lib.rs:649`

`BridgeConfig` exposes `session_timeout_ms`, but the value is never referenced in the polling or session lifecycle logic.

That means the module advertises per-session inactivity timeout control while the runtime ignores it entirely.

Impact:
- configuration surface is misleading
- long-lived stale sessions are not governed by the documented timeout setting
- operators cannot rely on timeout tuning to control bridge behavior

### 4. Medium: `start_bridge_with_client()` accepts an HTTP client parameter but ignores it

Files:
- `src-rust/crates/bridge/src/lib.rs:834`
- `src-rust/crates/bridge/src/lib.rs:836`
- `src-rust/crates/bridge/src/lib.rs:850`

`start_bridge_with_client()` takes `_http: reqwest::Client`, but it immediately constructs a fresh `BridgeSession::new(config)` instead of using the supplied client.

That makes the parameter misleading and prevents callers from controlling:

- custom client settings
- shared connection pools
- injected test clients

Impact:
- helper API contract is misleading
- client injection is ineffective
- tests and runtime customization are harder than the signature suggests

## Fix Plan

### 1. Add retry-safe buffering for outbound bridge events

Approach:
- do not discard drained events when `upload_events()` fails
- keep a pending outbound queue inside the poll loop and retry it on the next iteration
- preserve event ordering across retries
- consider bounded/durable overflow handling if the bridge stays down for a long time

Implementation target:
- `src-rust/crates/bridge/src/lib.rs`

Verification:
- inject a temporary upload failure and confirm previously drained events are uploaded later
- verify text/tool/turn events remain in order after retry

### 2. Add a real outer reconnect lifecycle for the bridge worker

Approach:
- when the inner poll loop terminates unexpectedly, treat that as a recoverable bridge failure
- recreate or re-register the bridge session and start a fresh poll loop with backoff
- surface `Reconnecting` and final `Disconnected` only from this outer recovery controller

Implementation target:
- `src-rust/crates/bridge/src/lib.rs`

Verification:
- simulate repeated poll failures and confirm the outer bridge worker retries
- confirm a recovered bridge emits a fresh `Connected` event and resumes traffic

### 3. Either implement `session_timeout_ms` or remove it from the runtime config

Approach:
- if the timeout is intended, track bridge activity and terminate/recycle inactive sessions after the configured interval
- otherwise remove the dead config field and related documentation to avoid false expectations

Implementation target:
- `src-rust/crates/bridge/src/lib.rs`

Verification:
- test inactivity timeout behavior explicitly if kept
- otherwise ensure no stale docs/config mention the unused setting

### 4. Honor injected HTTP clients in the bridge startup helpers

Approach:
- thread the provided `reqwest::Client` into `BridgeSession`
- stop rebuilding an internal client when a caller already supplied one
- keep `BridgeSession::new` and `start_bridge_with_client` contracts aligned

Implementation target:
- `src-rust/crates/bridge/src/lib.rs`

Verification:
- add a test/startup path that uses a custom client and confirm it is actually exercised

## Recommended execution order

1. add retry-safe buffering for outbound events
2. implement a real outer reconnect lifecycle
3. decide and enforce the `session_timeout_ms` contract
4. honor injected HTTP clients in bridge startup helpers
