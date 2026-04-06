# CLI Module Review

Module under review: `src-rust/crates/cli`

Why this module next: `cli` is the composition root for the entire application. It loads config, builds provider and model registries, wires the TUI and query runtime together, and applies command-side mutations back into live state. Bugs here can leave the whole application in an inconsistent or misleading runtime configuration.

## Findings

### 1. High: non-Anthropic startup auth detection is effectively hardcoded “true”

Files:
- `src-rust/crates/cli/src/main.rs:538`
- `src-rust/crates/cli/src/main.rs:560`
- `src-rust/crates/cli/src/main.rs:567`

The `other_provider_configured` check includes:

- `|| true; // Ollama/LM Studio don't require keys`

That makes `has_non_anthropic_env` always true, which in turn makes `other_provider_configured` always true regardless of the actual environment.

As a result, when a non-Anthropic provider is selected and no credentials are available, the CLI suppresses the normal missing-auth failure path and proceeds with an empty API key. That may be acceptable for local providers, but it is wrong for remote providers such as OpenAI, Google, Groq, Together AI, and others.

Impact:
- missing credentials for remote non-Anthropic providers are not caught at startup
- the user gets deferred runtime failures instead of an immediate actionable error
- auth/bootstrap behavior depends on a placeholder shortcut rather than real provider type

### 2. High: session resume restores the model string but not the corresponding provider state

Files:
- `src-rust/crates/cli/src/main.rs:1328`
- `src-rust/crates/cli/src/main.rs:1750`
- `src-rust/crates/cli/src/main.rs:1754`

When a session is loaded at startup or via `CommandResult::ResumeSession`, the CLI copies `session.model` into config and UI state, but it does not restore or infer the matching provider into `config.provider`.

That leaves the runtime in a split state:

- `model` may be `openai/gpt-4o`
- `provider` may still be the previous provider or `None`

Some lower layers can recover the provider from the model string, but the TUI and provider-specific flows still read `config.provider` directly in several places. So a resumed session can look partially correct while still carrying stale provider state.

Impact:
- resumed sessions can reopen with the wrong active provider state
- provider-specific TUI features can target the wrong backend after resume
- persisted session identity and live config drift apart

### 3. Medium: `--api-base` can be attached to the wrong provider when only `--model provider/model` is supplied

Files:
- `src-rust/crates/cli/src/main.rs:453`
- `src-rust/crates/cli/src/main.rs:485`
- `src-rust/crates/cli/src/main.rs:488`

CLI argument handling sets `config.model` from `--model`, but `--api-base` is stored under `provider_configs[provider_id]` using `config.provider` or a default of `anthropic`.

If the user runs something like:

- `--model openai/gpt-4o --api-base http://localhost:1234`

without also passing `--provider openai`, the base URL override is written under the Anthropic provider entry instead of the provider implied by the model string.

Impact:
- CLI provider override behavior is order/flag-shape dependent
- endpoint overrides can silently attach to the wrong provider
- OpenAI-compatible custom backends are easy to misconfigure from the CLI

### 4. Medium: command-driven config resets can leave the TUI model label stale

Files:
- `src-rust/crates/cli/src/main.rs:1862`
- `src-rust/crates/cli/src/main.rs:1883`

When the command layer returns `ConfigChange` or `ConfigChangeMessage`, the CLI only updates `app.model_name` if `new_cfg.model` is `Some(...)`.

If a command clears the explicit model override, such as `/config unset model`, the config is updated but the UI model label is not recomputed from the effective default model. The TUI can continue showing the old model even though the live config no longer points to it.

Impact:
- stale model label in the interactive UI
- visible state diverges from effective runtime configuration
- follow-up provider/model interactions become harder to reason about

## Fix Plan

### 1. Replace the placeholder non-Anthropic auth shortcut with explicit provider classification

Approach:
- remove the unconditional `|| true`
- classify providers into:
  - local/keyless-capable providers
  - remote/key-required providers
- only allow empty-auth startup bypass for providers that are explicitly local or otherwise keyless

Implementation target:
- `src-rust/crates/cli/src/main.rs`

Verification:
- remote providers without credentials fail fast with a clear message
- local providers such as Ollama, LM Studio, and llama.cpp still start without keys

### 2. Restore canonical provider state when loading or resuming sessions

Approach:
- when `session.model` is loaded, infer and canonicalize its provider
- update `config.provider` alongside `config.model`
- reuse the same provider inference/canonicalization logic used elsewhere in the stack

Implementation targets:
- `src-rust/crates/cli/src/main.rs`
- shared helper preferably in `claurst-core`

Verification:
- resume a session saved with `openai/gpt-4o` and confirm both model and provider restore correctly
- confirm provider-specific TUI flows behave correctly immediately after resume

### 3. Resolve `--api-base` against the effective provider, not just `config.provider`

Approach:
- determine the effective provider from:
  1. explicit `--provider`
  2. provider prefix embedded in `--model`
  3. final config fallback
- attach `--api-base` to that effective provider ID after canonicalization

Implementation target:
- `src-rust/crates/cli/src/main.rs`

Verification:
- `--model openai/gpt-4o --api-base ...` writes the override under OpenAI
- canonical and alias provider IDs map to the same provider-config entry

### 4. Recompute the TUI model label after config mutations that clear the explicit model

Approach:
- when handling `ConfigChange` and `ConfigChangeMessage`, update `app.model_name` from the effective model, not only from `new_cfg.model`
- keep the fast-mode indicator derived from the effective model as well

Implementation targets:
- `src-rust/crates/cli/src/main.rs`
- possibly shared helper usage from `claurst_api::effective_model_for_config`

Verification:
- `/config unset model` immediately updates the visible model label to the provider default
- fast-mode indicator stays aligned with the actual effective model

## Recommended execution order

1. fix non-Anthropic auth bootstrap classification
2. restore provider state correctly on session load and resume
3. resolve `--api-base` against the effective provider
4. recompute the TUI model label after config mutations
