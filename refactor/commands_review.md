# Commands Module Review

Module under review: `src-rust/crates/commands`

Why this module next: `commands` is the control plane above the runtime. It is responsible for mutating config, exposing provider and session operations, and bridging user intent into the TUI and query layers. Bugs here can quietly persist invalid state or present misleading operational status even when the lower layers are working correctly.

## Findings

### 1. High: `/providers` does not report actual live providers or provider status

Files:
- `src-rust/crates/commands/src/lib.rs:7529`
- `src-rust/crates/commands/src/lib.rs:7537`
- `src-rust/crates/api/src/model_registry.rs:63`
- `src-rust/crates/api/src/model_registry.rs:78`

`/providers` claims to list available AI providers and their status, but it builds a fresh `ModelRegistry` and reports whatever models are in that static registry snapshot.

That is not the live provider state. It does not use:

- the runtime `ProviderRegistry`
- auth-store-backed available providers
- provider health checks
- local provider availability

`ModelRegistry::new()` only loads the bundled snapshot for Anthropic, OpenAI, and Google, so the command can omit real configured providers and cannot report their actual status.

Impact:
- misleading operational diagnostics
- local or configured providers can appear missing
- the command contract does not match what it actually measures

### 2. High: `/model` and `/config set model` update the model without synchronizing the active provider

Files:
- `src-rust/crates/commands/src/lib.rs:650`
- `src-rust/crates/commands/src/lib.rs:671`
- `src-rust/crates/commands/src/lib.rs:807`
- `src-rust/crates/cli/src/main.rs:1862`
- `src-rust/crates/tui/src/app.rs:1618`

Both `/model` and `/config set model` write only `config.model`. They do not update `config.provider`, even when the model string is explicitly provider-prefixed, such as `openai/gpt-4o`.

Downstream query dispatch may still recover the provider from the model string in some paths, but the UI and config state are left divergent:

- the stored provider can remain on the old backend
- the displayed model can belong to a different backend
- provider-specific UI flows still read `config.provider`

This creates a split-brain state where some layers follow the explicit model string while others follow the stale provider field.

Impact:
- inconsistent provider/model state across command, TUI, and query layers
- provider-specific UI features can operate on the wrong backend
- config reload behavior becomes harder to reason about

### 3. Medium: command-driven model changes do not validate or canonicalize provider IDs

Files:
- `src-rust/crates/commands/src/lib.rs:658`
- `src-rust/crates/commands/src/lib.rs:672`
- `src-rust/crates/commands/src/lib.rs:809`

The command layer accepts any raw model string and persists it as-is. There is no validation against:

- known providers
- canonical provider IDs
- model-registry entries

That means command paths can still persist legacy aliases or typos such as `lmstudio/default`, `togetherai/...`, or completely unknown provider prefixes. The actual failure is deferred until later runtime paths attempt to resolve the provider or list models.

Impact:
- invalid or non-canonical config can be persisted silently
- command behavior differs from stricter provider-aware UI flows
- user mistakes are discovered late, after state has already been saved

## Fix Plan

### 1. Rework `/providers` around the live provider registry

Approach:
- pass live provider/runtime status into the command context
- build `/providers` output from the runtime `ProviderRegistry`, not a fresh `ModelRegistry`
- report provider health/status explicitly
- keep model-registry metadata only as supplemental detail

Implementation targets:
- `src-rust/crates/commands/src/lib.rs`
- `src-rust/crates/cli/src/main.rs`

Verification:
- confirm configured local providers appear even when not in the bundled model snapshot
- confirm unavailable providers show an actual health/status result

### 2. Synchronize provider state when commands set a provider-prefixed model

Approach:
- when `/model` or `/config set model` receives `provider/model`, parse the provider prefix
- canonicalize the provider ID
- update both `config.model` and `config.provider`
- preserve the Anthropic bare-model behavior only when no provider prefix is supplied

Implementation targets:
- `src-rust/crates/commands/src/lib.rs`

Verification:
- `/model openai/gpt-4o` updates both model and provider
- `/config set model llama-cpp/default` updates both fields consistently
- TUI provider-specific flows stay aligned after command-driven changes

### 3. Validate and canonicalize command-supplied provider/model values before persistence

Approach:
- introduce shared provider canonicalization in the command layer
- reject unknown provider prefixes early with a clear error
- optionally validate the model against the model registry when possible, while still allowing intentional custom models behind supported providers

Implementation targets:
- `src-rust/crates/commands/src/lib.rs`
- shared helper preferably in `claurst-core`

Verification:
- legacy aliases normalize to canonical IDs
- obviously invalid provider prefixes are rejected before config is changed
- valid custom provider-backed model IDs remain allowed

## Recommended execution order

1. rework `/providers` around live runtime provider state
2. synchronize provider state when commands set a provider-prefixed model
3. add provider/model validation and canonicalization for command-driven model changes
