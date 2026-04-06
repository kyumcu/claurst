# Query Module Review

Module under review: `src-rust/crates/query`

Why this module first: it is the runtime orchestration layer that sits between the TUI/commands layer and the provider/tools layer. Failures here can corrupt conversation history, misroute requests, or silently degrade agent behavior across the entire application.

## Findings

### 1. Critical: reactive compaction can permanently drop the entire conversation on failure

Files:
- `src-rust/crates/query/src/lib.rs:1482`
- `src-rust/crates/query/src/lib.rs:1506`

In both the `context_collapse()` and `reactive_compact()` paths, the code passes `std::mem::take(messages)` into the compaction call. On success, the replacement messages are restored. On failure, the old messages are not restored, even though the comments acknowledge the issue.

That means a compaction error or cancellation can leave the session history empty or partially destroyed for the rest of the turn and any future turns in the same session.

Impact:
- loss of conversation state
- broken follow-up turns
- high-risk data loss behavior in a core runtime path

### 2. Critical: provider stream errors are silently converted into partial or empty assistant turns

Files:
- `src-rust/crates/query/src/lib.rs:1098`
- `src-rust/crates/query/src/lib.rs:1164`

In the non-Anthropic provider path, `Some(Err(e))` from the provider stream logs the error and `break`s out of the streaming loop. After that, execution continues as if the stream ended normally: the code reconstructs `content_blocks`, builds an assistant message, pushes it into history, and may return `EndTurn`.

This turns transport/provider failures into successful-looking turns. In the worst case, an empty assistant message is committed to history; in the partial case, a truncated response is treated as complete.

Impact:
- silent corruption of assistant output
- misleading success semantics
- hard-to-debug provider failures

### 3. High: `api_base` overrides are ignored when a runtime provider exists in auth storage

Files:
- `src-rust/crates/query/src/lib.rs:941`
- `src-rust/crates/query/src/lib.rs:956`
- `src-rust/crates/query/src/lib.rs:980`

The query loop prefers `runtime_provider_for()` when auth-store credentials exist. However, the custom `api_base` override logic only rewrites `registry_provider`. The final provider selection still uses `runtime_provider.or(registry_provider)`, so the runtime provider wins and the override is discarded.

This is especially dangerous for OpenAI-compatible backends and gateway-style deployments where credentials exist but the endpoint must be changed.

Impact:
- wrong endpoint used at runtime
- local/private deployments silently ignored
- configuration appears to succeed but does not take effect

### 4. High: command queue injection prepends new messages before older conversation history

Files:
- `src-rust/crates/query/src/lib.rs:732`
- `src-rust/crates/query/src/lib.rs:741`
- `src-rust/crates/query/src/command_queue.rs:142`

The command queue drains messages intended for the next turn, but the query loop prepends them ahead of the entire existing message history:

- take the whole history
- append injected messages first
- append the old history second

That reverses chronology at the conversation boundary. A newly injected command should be the most recent user/system instruction, not the oldest message in the entire transcript.

Impact:
- distorted prompt chronology
- model sees late control messages as ancient context
- subtle misbehavior in queued command handling

### 5. Medium: session memory extraction only works with `ANTHROPIC_API_KEY` from the environment

Files:
- `src-rust/crates/query/src/lib.rs:1601`
- `src-rust/crates/query/src/lib.rs:1603`

Session memory extraction builds a fresh `AnthropicClient` only if `ANTHROPIC_API_KEY` exists in the environment. It does not use:

- auth-store credentials added via `/connect`
- provider registry state
- the active provider/model if the session is running on a non-Anthropic backend

So this background feature can silently stop working even though the main session is fully authenticated and running normally.

Impact:
- background memory extraction silently disabled
- behavior depends on env setup instead of active runtime auth
- inconsistent feature behavior across providers

## Fix Plan

### 1. Preserve message history across reactive compaction failures

Approach:
- stop passing `std::mem::take(messages)` directly into compaction calls
- instead, clone or move into a temporary `old_messages` variable
- on success, replace `*messages`
- on failure or cancellation, restore `old_messages`

Implementation target:
- `src-rust/crates/query/src/lib.rs`

Verification:
- add tests for `context_collapse()` failure path
- add tests for `reactive_compact()` cancellation/failure path
- assert that `messages` remains unchanged after failure

### 2. Treat provider stream errors as actual query errors

Approach:
- track an explicit provider stream failure state
- if `Some(Err(e))` occurs, return `QueryOutcome::Error(...)` instead of falling through
- if partial-output recovery is desired later, make it explicit and separate from normal `EndTurn`

Implementation target:
- `src-rust/crates/query/src/lib.rs`

Verification:
- mock a provider stream that emits an error after `MessageStart`
- assert that the loop returns `QueryOutcome::Error`
- assert that no fake successful assistant turn is appended

### 3. Unify provider construction with `api_base` overrides applied last

Approach:
- introduce a helper that resolves the canonical provider and then applies:
  1. canonical provider ID normalization
  2. auth-store credentials
  3. `provider_configs[provider].api_base`
- use that helper instead of selecting `runtime_provider` before override logic

Implementation target:
- `src-rust/crates/query/src/lib.rs`
- possibly `src-rust/crates/api/src/registry.rs` if provider construction should be centralized there

Verification:
- test a credential-backed OpenAI-compatible provider with custom `api_base`
- assert that requests go to the overridden endpoint

### 4. Fix queued message ordering

Approach:
- append drained command-queue messages after the existing history, not before it
- preserve priority order within the drained batch itself

Implementation target:
- `src-rust/crates/query/src/lib.rs`

Verification:
- add a queue-order test showing:
  - old history remains first
  - queued messages appear last
  - within queued messages, higher-priority items still come first

### 5. Decouple session memory extraction from raw env-based Anthropic auth

Approach:
- reuse the active runtime provider/client where possible, or
- resolve auth through the same auth-store/provider-registry path used by the main query loop
- if extraction is Anthropic-only by design, fail explicitly in logs and gate the feature instead of silently skipping it

Implementation target:
- `src-rust/crates/query/src/lib.rs`
- `src-rust/crates/query/src/session_memory.rs`

Verification:
- test with auth-store-only credentials
- test with non-Anthropic provider selected
- confirm expected enable/disable behavior is explicit

## Recommended execution order

1. preserve message history on compaction failure
2. fail hard on provider stream errors
3. fix provider construction and `api_base` override precedence
4. correct queued message ordering
5. refactor session memory auth resolution
