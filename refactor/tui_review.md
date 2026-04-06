# TUI Module Review

Module under review: `src-rust/crates/tui`

Why this module next: `tui` is the primary user-facing state machine for provider selection, model selection, auth entry, and async UI updates. Bugs here can persist incorrect configuration, hide provider failures behind fallback behavior, or present the wrong runtime state to the user.

## Findings

### 1. High: `/connect` still persists non-canonical provider IDs for `LM Studio` and `Together AI`

Files:
- `src-rust/crates/tui/src/app.rs:220`
- `src-rust/crates/tui/src/app.rs:224`
- `src-rust/crates/tui/src/app.rs:2649`
- `src-rust/crates/tui/src/app.rs:2613`
- `src-rust/crates/core/src/provider_id.rs:45`
- `src-rust/crates/core/src/provider_id.rs:49`

The provider picker still emits UI aliases:

- `lmstudio`
- `togetherai`

Those IDs are then used directly in both auth-store writes and provider activation. The canonical backend IDs are:

- `lm-studio`
- `together-ai`

This is the same class of bug that was already fixed for `llama.cpp`. The TUI still persists provider IDs that do not match the canonical registry/provider ID layer, so behavior becomes alias-dependent across `/connect`, `/model`, auth lookup, and provider dispatch.

Impact:
- `LM Studio` and `Together AI` can still fail model discovery or runtime lookup depending on the call path
- auth-store and config data continue to accumulate non-canonical IDs
- behavior differs between newly connected providers and the backend registry contract

### 2. High: model-registry lookups in the TUI use raw provider IDs without normalization

Files:
- `src-rust/crates/tui/src/model_picker.rs:244`
- `src-rust/crates/tui/src/app.rs:1362`
- `src-rust/crates/tui/src/app.rs:4990`

The TUI uses provider IDs directly for:

- `model_registry.best_model_for_provider(provider_id)`
- `models_for_provider_from_registry(provider_id, ...)`
- `ProviderRegistry::get(ProviderId::new(&provider_id_str))`

None of those paths normalize aliases first. So even where the model picker has compatibility branches for `lmstudio`, `llamacpp`, and `togetherai`, the registry-backed path still depends on the exact spelling stored in config.

This means the UI can appear to work through hardcoded fallback models while the dynamic provider-backed model flow quietly fails.

Impact:
- model discovery remains inconsistent across alias vs canonical provider IDs
- live provider-backed `/model` results can fail while fallback entries still display
- the user sees partial success even when the real registry path is broken

### 3. Medium: async model-fetch results are not tagged with the provider they belong to

Files:
- `src-rust/crates/tui/src/app.rs:4957`
- `src-rust/crates/tui/src/app.rs:4960`
- `src-rust/crates/tui/src/app.rs:4988`

The async model fetch channel only returns `Vec<ModelEntry>` or `Err(())`. It does not include the provider ID that the request was launched for.

When results are drained, the TUI derives the provider from the current config state at receive time, not the provider that was active when the fetch started. If the user changes provider or reopens the picker while a previous fetch is in flight, stale results can be applied to the wrong provider context.

Impact:
- wrong model list can populate the picker after a fast provider switch
- current-model highlighting can be computed against the wrong provider prefix
- racey, hard-to-reproduce UI inconsistency in a core interaction path

### 4. Medium: provider inference from model names still has incomplete canonical-ID coverage

Files:
- `src-rust/crates/tui/src/app.rs:1430`
- `src-rust/crates/tui/src/app.rs:1450`
- `src-rust/crates/core/src/provider_id.rs:49`

`infer_provider_from_model()` knows about:

- `lmstudio`
- `llamacpp`
- `llama-cpp`
- `togetherai`
- `together-ai`

But it still does not recognize the canonical `lm-studio` provider ID. That leaves the TUI with another alias-specific branch where persisted canonical values and inferred provider values can diverge.

This is especially risky because the function is used to recover provider identity from model strings, which means the bug can show up after config reloads or partial state reconstruction rather than only in `/connect`.

Impact:
- provider inference remains inconsistent after canonicalization elsewhere
- model-to-provider recovery depends on legacy spellings
- the UI state machine can drift from persisted config state

## Fix Plan

### 1. Normalize provider IDs at the TUI persistence boundary

Approach:
- introduce a shared provider-ID canonicalization helper
- apply it before:
  - writing auth-store credentials
  - activating a provider from `/connect`
  - persisting provider/model settings
- update the picker items so displayed labels stay the same but stored IDs are canonical

Implementation targets:
- `src-rust/crates/tui/src/app.rs`
- shared helper placement preferably in `claurst-core`

Verification:
- `/connect` for `LM Studio` stores `lm-studio`
- `/connect` for `Together AI` stores `together-ai`
- old saved aliases still load and behave correctly

### 2. Normalize provider IDs before all registry and model-registry lookups

Approach:
- canonicalize provider IDs before calling:
  - `best_model_for_provider`
  - `list_by_provider`
  - `ProviderRegistry::get`
- keep compatibility aliases only at input edges, not throughout the lookup logic

Implementation targets:
- `src-rust/crates/tui/src/app.rs`
- `src-rust/crates/tui/src/model_picker.rs`

Verification:
- registry-backed `/model` fetch works for both legacy aliases and canonical IDs
- hardcoded fallback is used only when the provider really has no registry data

### 3. Make async model fetches provider-scoped

Approach:
- include the requested provider ID in the async fetch result message
- on receipt, discard results that do not match the currently open picker context
- compute current-model highlighting against the provider that the result belongs to

Implementation target:
- `src-rust/crates/tui/src/app.rs`

Verification:
- start a model fetch, switch providers before it completes, and confirm stale results are ignored
- ensure current highlighting remains correct after rapid provider changes

### 4. Consolidate provider inference around canonical IDs

Approach:
- replace the ad hoc known-provider string list with canonical IDs plus alias normalization
- ensure `infer_provider_from_model()` recognizes canonical local-provider IDs such as `lm-studio`
- keep the inference table aligned with `ProviderId`

Implementation targets:
- `src-rust/crates/tui/src/app.rs`
- optionally shared metadata/helper in `claurst-core`

Verification:
- table-driven tests over model strings using both canonical and legacy provider prefixes
- confirm inferred provider matches the persisted canonical provider ID

## Recommended execution order

1. normalize provider IDs at `/connect` and auth/config persistence boundaries
2. normalize all TUI registry/model-registry lookups
3. make async model fetches provider-scoped
4. consolidate provider inference around canonical IDs
