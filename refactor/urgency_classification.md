# Refactor Findings Classification

Purpose: group all findings from the `refactor/*_review.md` files by urgency and failure mode so it is easier to decide what must be fixed first.

Rule used here: each finding is placed in one primary bucket, even if it could reasonably fit more than one.

## Group Definitions

### Show Stoppers

Issues that can destroy conversation state, break the main execution path, or make a major subsystem fundamentally untrustworthy.

### Vulnerabilities / Safety Risks

Issues that can cross trust boundaries, escape intended filesystem scope, or create high-risk unsafe behavior.

### Data Integrity / Reliability Failures

Issues that silently lose, corrupt, or mis-sequence data, state, events, or persisted artifacts.

### Logical Errors / Inconsistent State

Issues where the code does the wrong thing relative to its intended logic, especially around provider/model/config resolution.

### Contract Gaps / Misleading Interfaces

Issues where a module advertises capabilities, status, or behavior that the runtime does not actually provide.

### Good To Haves / Lower Urgency

Useful improvements, lower-blast-radius correctness fixes, and cleanup that should follow after the higher-risk buckets.

## 1. Show Stoppers

- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): reactive compaction can permanently drop the entire conversation on failure.
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): provider stream errors are silently converted into partial or empty assistant turns.
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md): `session/create` returns a session ID that is not persisted or usable by `session/message`.
- [mcp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/mcp_review.md): connection manager and reconnect logic only use stdio even though the crate advertises HTTP/SSE support.
- [plugins_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/plugins_review.md): plugin reload cannot actually replace the global plugin or hook registries.

## 2. Vulnerabilities / Safety Risks

- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md): tool path resolution does not enforce workspace or allowed-directory boundaries.
- [mcp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/mcp_review.md): MCP OAuth initiation generates a PKCE verifier but never preserves it for code exchange.

## 3. Data Integrity / Reliability Failures

- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): command queue injection prepends new messages before older conversation history.
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): session memory extraction only works with `ANTHROPIC_API_KEY` from the environment.
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md): worktree session state is process-global instead of session-scoped.
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md): `apply_patch` cannot create parent directories for newly added files.
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md): outbound bridge events are silently dropped on upload failures.
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md): the outer bridge loop reports disconnect but never re-registers or recreates the poll loop.
- [plugins_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/plugins_review.md): plugin enabled/disabled state is lost on reload.
- [plugins_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/plugins_review.md): marketplace update/install can leave stale files from previous plugin versions.
- [buddy_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/buddy_review.md): companion soul persistence is not keyed by user identity.
- [buddy_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/buddy_review.md): companion soul writes are not atomic.
- [buddy_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/buddy_review.md): load failures are silently treated the same as “no companion exists yet”.
- [mcp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/mcp_review.md): reconnect-loop bookkeeping is never cleaned up after a successful reconnect.

## 4. Logical Errors / Inconsistent State

- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): `api_base` overrides are ignored when a runtime provider exists in auth storage.
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md): canonical provider IDs for Moonshot and Zhipu do not resolve through `runtime_provider_for`.
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md): local OpenAI-compatible providers are classified as remote unless the base URL literally looks loopback.
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md): `OpenAiCompatProvider::list_models()` fabricates the same context window and max output for every model.
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md): `ApiMessage::from` silently degrades serialization failures into `null` content.
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md): `Settings::effective_config()` does not actually let top-level `settings.provider` override `settings.config.provider`.
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md): `Config::effective_model()` misses canonical provider IDs and can fall back to Anthropic defaults for valid non-Anthropic providers.
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md): `AuthStore::api_key_for()` has incomplete canonical-ID coverage.
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md): core has provider config structure but no canonical provider-aware base URL resolution path.
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md): `/connect` still persists non-canonical provider IDs for `LM Studio` and `Together AI`.
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md): model-registry lookups in the TUI use raw provider IDs without normalization.
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md): async model-fetch results are not tagged with the provider they belong to.
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md): provider inference from model names still has incomplete canonical-ID coverage.
- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md): `/model` and `/config set model` update the model without synchronizing the active provider.
- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md): command-driven model changes do not validate or canonicalize provider IDs.
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md): non-Anthropic startup auth detection is effectively hardcoded true.
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md): session resume restores the model string but not the corresponding provider state.
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md): `--api-base` can be attached to the wrong provider when only `--model provider/model` is supplied.
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md): command-driven config resets can leave the TUI model label stale.
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md): Bash tool behavior contradicts its own contract about shell-state persistence.

## 5. Contract Gaps / Misleading Interfaces

- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md): `/providers` does not report actual live providers or provider status.
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md): bridge session timeout is configured but unused.
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md): `start_bridge_with_client()` accepts an HTTP client parameter but ignores it.
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md): ACP capability and method surface overstates what the server actually implements.
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md): `tool/list` and `model/list` return static or snapshot data instead of live runtime state.
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md): ACP accepts malformed JSON-RPC envelopes without validating protocol version or request shape.
- [plugins_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/plugins_review.md): marketplace install/list paths do not match the plugin loader’s manifest conventions.
- [mcp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/mcp_review.md): prefixed MCP tool routing is ambiguous when server names overlap.

## 6. Good To Haves / Lower Urgency

- [fix_plan_1.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/fix_plan_1.md): follow-up verification across `llama.cpp`, `LM Studio`, and other canonicalized provider flows.
- [review_coverage.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/review_coverage.md): coverage tracking is complete and should be kept current if new modules are added.

## Cross-Cutting Themes

### Provider canonicalization is a major urgency multiplier

The same root problem appears in `core`, `api`, `query`, `tui`, `commands`, `cli`, and `fix_plan_1.md`. This is not a one-off bug. It is a systemic normalization failure affecting auth lookup, default model choice, registry lookup, `/connect`, `/model`, resume flows, and `api_base` overrides.

### Runtime truth vs advertised truth is the second major cluster

`commands`, `acp`, `mcp`, `plugins`, and `bridge` all have places where the interface claims more than the runtime actually provides. These are dangerous because they create false confidence and make debugging much slower.

### Data loss and state drift are concentrated in core execution surfaces

The highest-risk operational issues are in `query`, `tools`, and `bridge`. Those should be treated as the first stabilization wave before doing lower-risk cleanup.

### Optional-surface complexity is the main code-size pressure

The biggest non-core complexity comes from `acp`, `bridge`, `mcp`, `plugins`, and `buddy`. These modules are not equally valuable to the local `llama.cpp` workflow, so they should not all be expanded by default. For these areas, shrinking scope is often better than implementing every advertised path.

## Leaning Guidance By Module Group

### Keep and invest

- `query`
- `tools`
- `core`
- `api`
- `tui`
- `commands`
- `cli`

These modules define the trustworthy local-first path and should be improved, not reduced.

### Keep but constrain

- `plugins`
- `mcp`

These should be kept only within honest, reliable scope. Their complexity should be capped until the core path is stable.

### Reduce, isolate, or make optional

- `acp`
- `bridge`
- `buddy`

These have the weakest connection to the lean local-first core. They should be minimized, made optional, or explicitly deprioritized unless they are a strategic product requirement.

## Recommended Urgency Order

1. Show Stoppers
2. Vulnerabilities / Safety Risks
3. Data Integrity / Reliability Failures
4. Logical Errors / Inconsistent State
5. Contract Gaps / Misleading Interfaces
6. Good To Haves / Lower Urgency

## Practical First Wave

If the goal is to reduce the most real risk quickly, the first wave should be:

1. [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): compaction data loss.
2. [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md): stream errors becoming fake successful turns.
3. [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md): filesystem path-boundary enforcement.
4. [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md): outbound event loss and missing outer reconnect.
5. [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md), [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md), [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md), [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md), [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md), [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md): unify provider canonicalization and provider-aware config resolution as a coordinated sweep.

## Practical Reduction Candidates

If the aim is to reduce code and maintenance load, the strongest candidates are:

- `acp`: reduce to a minimal honest surface unless real editor-session support is required.
- `bridge`: keep only if remote/web control is a real product need.
- `buddy`: isolate as optional or remove from the core refactor path.
- `plugins`: cap scope at reliable loading and runtime truth; defer marketplace sophistication.
- `mcp`: support only the transports and routing modes that are truly needed.
