# Fix Plan

## 1. High: fix `api_base` override precedence for runtime providers

Add a canonical provider-construction path that applies auth-store credentials and `provider_configs[provider].api_base` in one place, instead of choosing `runtime_provider` before override handling. The simplest safe change is in `src-rust/crates/query/src/lib.rs`: if an override base exists, rebuild the provider from the canonical provider ID regardless of whether it came from the registry or `runtime_provider_for()`. Then add coverage for a keyed OpenAI-compatible provider plus custom `api_base`.

## 2. High: fix `LM Studio` provider ID normalization

Normalize `lmstudio` to `lm-studio` at the `/connect` persistence boundary, the same way `llama.cpp` was fixed. Update alias handling so old configs still work in `src-rust/crates/tui/src/app.rs`, `src-rust/crates/tui/src/model_picker.rs`, and `src-rust/crates/core/src/lib.rs`. Then verify live `/model` fetch works through the registry path.

## 3. Medium: normalize all remaining UI aliases to canonical backend IDs

Audit picker IDs against `src-rust/crates/core/src/provider_id.rs` and convert the remaining mismatches, especially `togetherai` vs `together-ai`. Do not keep fixing these one by one in scattered matches. Introduce a shared normalization helper like `canonical_provider_id(&str) -> &str` and use it in `/connect`, model inference, registry lookups, auth lookup, and query dispatch. Add a small table-driven test set for known aliases.

## 4. Follow-up verification

Run targeted checks:

- `/connect` -> `/model` for `llama.cpp` and `LM Studio`
- a keyed OpenAI-compatible provider with stored auth plus custom `api_base`
- alias backward compatibility for existing saved configs such as `llamacpp`, `lmstudio`, and `togetherai`
