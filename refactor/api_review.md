# API Module Review

Module under review: `src-rust/crates/api`

Why this module next: it owns provider construction, model listing, request translation, streaming transport, and runtime provider lookup. Any bug here affects every model call path, especially multi-provider support.

## Findings

### 1. High: canonical provider IDs for Moonshot and Zhipu do not resolve through `runtime_provider_for`

Files:
- `src-rust/crates/api/src/registry.rs:26`
- `src-rust/crates/api/src/registry.rs:68`
- `src-rust/crates/core/src/provider_id.rs:60`
- `src-rust/crates/core/src/provider_id.rs:61`
- `src-rust/crates/api/src/providers/openai_compat_providers.rs:281`
- `src-rust/crates/api/src/providers/openai_compat_providers.rs:292`

`provider_from_key()` and therefore `runtime_provider_for()` match `"moonshot"` and `"zhipu"`, but the canonical provider IDs defined in core are `"moonshotai"` and `"zhipuai"`. The actual provider factories also construct providers using those canonical IDs via `ProviderId::MOONSHOT` and `ProviderId::ZHIPU`.

That means runtime auth-store lookup can fail for the canonical IDs even though the provider itself is registered under those canonical IDs elsewhere. In practice, auth-store-backed runtime provider resolution is inconsistent depending on whether the caller uses the canonical ID or the short alias.

Impact:
- `/connect` or saved settings can use a canonical provider ID that fails runtime provider resolution
- auth-store credentials may appear to exist but not produce a runtime provider
- request dispatch behavior becomes alias-dependent

### 2. High: local OpenAI-compatible providers are classified as “remote” unless the base URL literally contains `localhost`, `127.0.0.1`, or `::1`

Files:
- `src-rust/crates/api/src/providers/openai_compat.rs:739`
- `src-rust/crates/api/src/providers/openai_compat.rs:748`

`health_check()` for `OpenAiCompatProvider` allows keyless operation only when the base URL string contains `localhost`, `127.0.0.1`, or `::1`. That is too narrow for real local deployments, such as:

- `http://host.docker.internal:1234`
- `http://192.168.1.50:8080`
- `http://my-workstation.local:8080`

Those endpoints can be perfectly valid local/private deployments, but the provider will be marked unavailable with “No API key configured”.

Impact:
- false negative health status for local or LAN-hosted providers
- broken `/connect` or model-picker UX for legitimate self-hosted setups
- environment-dependent behavior that is hard to diagnose

### 3. Medium: `OpenAiCompatProvider::list_models()` fabricates the same context window and max output for every model

Files:
- `src-rust/crates/api/src/providers/openai_compat.rs:683`
- `src-rust/crates/api/src/providers/openai_compat.rs:730`
- `src-rust/crates/api/src/providers/openai_compat.rs:731`

The OpenAI-compatible `/models` path returns every discovered model with:

- `context_window: 128_000`
- `max_output_tokens: 16_384`

regardless of the actual model or provider. That keeps the UI functioning, but it pollutes downstream logic that depends on model metadata:

- context warnings
- compaction thresholds
- model picker descriptions
- provider capability assumptions

The bug is not that metadata is incomplete; the bug is that incorrect metadata is presented as fact.

Impact:
- misleading model picker data
- wrong context-management decisions
- hidden provider-specific limitations and overestimation of capabilities

### 4. Medium: `ApiMessage::from` silently degrades serialization failures into `null` content

Files:
- `src-rust/crates/api/src/lib.rs:185`
- `src-rust/crates/api/src/lib.rs:194`

When block serialization fails, `ApiMessage::from` substitutes `Value::Null` using `unwrap_or(Value::Null)`. That converts a structural serialization failure into a syntactically valid but semantically broken request body.

This is dangerous because upstream callers cannot distinguish:

- a correct empty message
- a serializer failure that erased content

The likely runtime effect is an opaque API error or model behavior that no longer corresponds to the real conversation history.

Impact:
- silent request corruption
- harder debugging because the original serialization failure is lost
- broken message payloads can reach the provider layer

## Fix Plan

### 1. Normalize provider IDs at the API registry boundary

Approach:
- add a shared canonicalization helper for provider IDs
- apply it inside:
  - `provider_from_key()`
  - `runtime_provider_for()`
  - auth-store-driven registration paths
- ensure aliases like `moonshot` and `zhipu` resolve to `moonshotai` and `zhipuai`

Implementation targets:
- `src-rust/crates/api/src/registry.rs`
- possibly shared helper placement in `claurst-core`

Verification:
- add table-driven tests for canonical and alias IDs
- verify both auth-store and env-driven registration produce the same provider IDs

### 2. Replace string-substring “is local” detection with explicit endpoint classification

Approach:
- parse the provider base URL with `url::Url`
- classify local/private addresses using:
  - loopback IPs
  - RFC1918 private ranges
  - link-local/private hostnames where appropriate
  - optionally explicit provider-local flags for known local providers
- do not rely on raw substring matching

Implementation target:
- `src-rust/crates/api/src/providers/openai_compat.rs`

Verification:
- test `localhost`, `127.0.0.1`, `::1`
- test LAN/private endpoints like `192.168.x.x`
- test `host.docker.internal`
- confirm remote endpoints still require keys

### 3. Stop inventing exact model limits in `list_models()`

Approach:
- if the provider does not expose real limits, return conservative/unknown metadata instead of fabricated exact values
- optionally enrich metadata from the model registry when available
- separate “discovered model exists” from “exact context/max_output known”

Implementation targets:
- `src-rust/crates/api/src/providers/openai_compat.rs`
- potentially `src-rust/crates/api/src/model_registry.rs`

Verification:
- confirm model picker can still render unknown or partial metadata
- confirm context-window logic does not treat fabricated values as authoritative

### 4. Make message serialization failures explicit

Approach:
- remove silent `Value::Null` fallback in `ApiMessage::from`
- either:
  - make conversion fallible, or
  - log and surface a structured error before request dispatch

Implementation targets:
- `src-rust/crates/api/src/lib.rs`
- any callers that currently assume infallible conversion

Verification:
- add tests with intentionally invalid block payloads
- assert failures are surfaced, not converted into `null`

## Recommended execution order

1. normalize provider IDs in the API registry/runtime lookup path
2. fix local endpoint classification in `health_check()`
3. stop fabricating model limits in OpenAI-compatible `/models`
4. make message serialization failures explicit
