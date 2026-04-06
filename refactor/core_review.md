# Core Module Review

Module under review: `src-rust/crates/core`

Why this module next: `core` is the shared foundation for config, settings, provider IDs, auth resolution, persistence, and prompt/runtime defaults. Bugs here fan out into `api`, `query`, `tui`, and `commands`.

## Findings

### 1. High: `Settings::effective_config()` does not actually let top-level `settings.provider` override `settings.config.provider`

Files:
- `src-rust/crates/core/src/lib.rs:1152`
- `src-rust/crates/core/src/lib.rs:1160`

The docstring says:

- `settings.provider` wins over `settings.config.provider`

But the implementation only copies `self.provider` into `config.provider` when `config.provider.is_none()`:

- `if self.provider.is_some() && config.provider.is_none() { ... }`

That means the embedded `config.provider` wins whenever both are set, which directly contradicts the documented behavior.

Impact:
- config precedence is wrong and surprising
- users or higher-level code can believe a provider override is active when it is not
- runtime behavior depends on where the provider was stored, not just its value

### 2. High: `Config::effective_model()` is missing canonical provider IDs and falls back to Anthropic defaults for valid non-Anthropic providers

Files:
- `src-rust/crates/core/src/lib.rs:951`
- `src-rust/crates/core/src/provider_id.rs:45`
- `src-rust/crates/core/src/provider_id.rs:49`
- `src-rust/crates/core/src/provider_id.rs:60`
- `src-rust/crates/core/src/provider_id.rs:61`

`effective_model()` handles several provider aliases, but it still misses canonical provider IDs for some providers defined in `ProviderId`, including:

- `lm-studio`
- `moonshotai`
- `zhipuai`

When one of those canonical IDs is active and no explicit model is set, the function falls through to `DEFAULT_MODEL`, which is Anthropic-specific.

Impact:
- valid provider selections can default to the wrong model family
- provider/model resolution becomes alias-dependent
- downstream UI and query logic can silently build the wrong requests

### 3. High: `AuthStore::api_key_for()` has incomplete canonical-ID coverage

Files:
- `src-rust/crates/core/src/auth_store.rs:80`
- `src-rust/crates/core/src/provider_id.rs:45`
- `src-rust/crates/core/src/provider_id.rs:60`
- `src-rust/crates/core/src/provider_id.rs:61`

`AuthStore::api_key_for()` maps only a subset of provider IDs to environment variables. It handles some aliases like `togetherai | together-ai`, but it does not cover multiple canonical IDs present in `ProviderId`, including:

- `moonshotai`
- `zhipuai`
- several other configured providers visible elsewhere in the repo

This means the auth layer can fail to resolve credentials for a provider that is otherwise supported and canonically named.

Impact:
- env-based auth works only for some spellings of the same provider
- runtime auth behavior diverges from provider registry behavior
- provider support looks partial or flaky depending on the call path

### 4. Medium: provider configuration exists structurally, but core does not provide a canonical provider-aware base URL resolution path

Files:
- `src-rust/crates/core/src/lib.rs:667`
- `src-rust/crates/core/src/lib.rs:738`
- `src-rust/crates/core/src/lib.rs:1088`

`ProviderConfig` contains `api_base`, and `Config` contains `provider_configs`, but `Config::resolve_api_base()` only returns:

- `ANTHROPIC_BASE_URL`, or
- the Anthropic constant

It does not consult:

- the active provider
- `provider_configs[provider].api_base`
- any provider-specific canonicalization

Some higher-level code works around this manually, but the core configuration layer itself does not offer a single correct provider-aware resolution function.

Impact:
- duplicated endpoint-resolution logic in higher layers
- inconsistent behavior between modules
- increased risk of precedence bugs when provider-specific base URLs are added

## Fix Plan

### 1. Make provider precedence match the documented contract

Approach:
- change `Settings::effective_config()` so `self.provider` always overrides `config.provider` when set
- add a regression test that covers:
  - only top-level provider set
  - only embedded provider set
  - both set with conflicting values

Implementation target:
- `src-rust/crates/core/src/lib.rs`

Verification:
- assert that top-level provider wins in all merge/load paths

### 2. Normalize `effective_model()` around canonical provider IDs

Approach:
- add a shared provider canonicalization helper or central alias table
- update `effective_model()` to use canonical IDs rather than scattered alias matches
- include canonical defaults for:
  - `lm-studio`
  - `moonshotai`
  - `zhipuai`
- verify all providers declared in `ProviderId` have either:
  - a default-model mapping, or
  - an explicit fallback policy

Implementation targets:
- `src-rust/crates/core/src/lib.rs`
- optionally shared helper placement in `provider_id.rs`

Verification:
- table-driven tests over canonical provider IDs
- assert no supported provider falls back to Anthropic accidentally

### 3. Expand `AuthStore` credential resolution to canonical provider IDs

Approach:
- centralize provider-to-env-var mapping
- support canonical IDs first, aliases second
- make sure mappings exist for all supported providers that are expected to work from env

Implementation targets:
- `src-rust/crates/core/src/auth_store.rs`
- optionally a shared provider metadata table in core

Verification:
- add tests for canonical and alias IDs
- confirm `api_key_for()` returns the same env var for both canonical and accepted alias spellings

### 4. Add a provider-aware base URL resolver in core

Approach:
- introduce a helper like:
  - `resolve_provider_api_base(&self, provider_id: &str) -> Option<String>`
- resolution order should be explicit:
  1. provider-specific config override
  2. provider-specific env override
  3. provider default
- keep Anthropic-specific `resolve_api_base()` only if it is intentionally Anthropic-only and named accordingly

Implementation targets:
- `src-rust/crates/core/src/lib.rs`
- possibly provider metadata helper in `provider_id.rs` or a new module

Verification:
- tests for provider-specific `api_base` resolution
- tests for canonical IDs and aliases sharing the same override behavior

## Recommended execution order

1. fix provider precedence in `effective_config()`
2. normalize default-model resolution for canonical provider IDs
3. expand `AuthStore` canonical-ID support
4. add a provider-aware base URL resolution helper in core
