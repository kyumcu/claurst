# Refactor Execution Roadmap

Purpose: convert the findings in the `refactor/` review set into an execution order that reduces real risk first, avoids thrash, and respects cross-module dependencies.

This is not a design doc for every fix. It is a sequencing document.

## Planning Rules

1. Fix data-loss and trust-boundary issues before UX correctness.
2. Fix shared provider canonicalization before patching individual provider-specific symptoms.
3. Fix runtime truth mismatches before improving reporting surfaces.
4. Prefer coordinated cross-module sweeps where one root cause appears in many crates.

## Phase 0: Stabilize the Safety Boundary

Goal: remove the highest-risk failure modes that can damage state or escape intended scope.

### 0.1 Tools path-boundary enforcement

Source:
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md)

Why first:
- this is the clearest trust-boundary issue in the repo
- it affects the model-to-machine execution surface directly
- later tool improvements are lower value if filesystem scope is still unbounded

Expected outcome:
- all filesystem tools enforce a single allowed-root policy
- absolute path handling becomes explicit and auditable

### 0.2 Query compaction failure preservation

Source:
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md)

Why here:
- it is a direct conversation-loss bug in the main runtime path
- fixes to later session behavior are lower value if turns can still erase history

Expected outcome:
- compaction failure cannot destroy or truncate active conversation history

### 0.3 Query provider-stream error hardening

Source:
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md)

Why here:
- fake successful turns poison history and make debugging almost impossible
- this is a core runtime correctness fix, not a UX nicety

Expected outcome:
- provider transport failures become explicit query failures
- no more empty or truncated assistant turns committed as success

## Phase 1: Stop Silent Loss and State Drift

Goal: make runtime state reliable across bridge, queueing, worktree, and plugin flows.

### 1.1 Bridge outbound event durability

Source:
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md)

Fix set:
- retry-safe buffering for upload failures
- outer reconnect lifecycle that actually recreates the bridge session

Why now:
- bridge failures currently lose session events permanently
- this is an operational reliability problem, not just a reporting issue

### 1.2 Session-scoped worktree state

Source:
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md)

Why now:
- process-global worktree state can leak across sessions
- it undermines correctness in multi-session/task-heavy usage

### 1.3 Query message-order and memory-auth consistency

Source:
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md)

Fix set:
- queued message ordering
- session memory auth resolution

Why now:
- both are correctness bugs in turn construction and background runtime behavior
- both affect trust in session behavior over time

### 1.4 Plugin reload integrity

Source:
- [plugins_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/plugins_review.md)

Fix set:
- replace global `OnceLock` registry state
- preserve enabled/disabled state
- clean update/install replacement behavior

Why now:
- reported plugin state currently diverges from active runtime behavior
- reload is a central operational workflow for extensibility

## Phase 2: Canonicalize Provider Identity Everywhere

Goal: eliminate the largest cross-cutting source of inconsistent behavior in the codebase.

Sources:
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md)
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md)
- [query_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/query_review.md)
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md)
- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md)
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md)
- [fix_plan_1.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/fix_plan_1.md)

Why this is a separate coordinated phase:
- the same root bug appears in every layer
- one-off fixes will keep reintroducing alias drift
- provider canonicalization and provider-aware config resolution need one shared contract

### 2.1 Establish shared provider canonicalization in core

Fix set:
- canonical provider normalization helper
- canonical provider-to-env mapping
- canonical default-model resolution
- provider-aware base URL resolver

Primary modules:
- `core`

### 2.2 Make API/runtime lookup canonical

Fix set:
- normalize runtime provider lookup
- fix canonical Moonshot/Zhipu paths
- resolve local/private endpoint classification correctly

Primary modules:
- `api`

### 2.3 Make query provider construction canonical and override-aware

Fix set:
- unify auth-backed provider construction
- apply `api_base` overrides after canonical provider resolution

Primary modules:
- `query`

### 2.4 Make user-facing provider state canonical

Fix set:
- `/connect` canonical IDs
- registry/model-registry lookup normalization
- async provider-scoped model fetches
- synchronized provider/model updates from commands
- CLI resume and `--api-base` effective-provider handling

Primary modules:
- `tui`
- `commands`
- `cli`

Expected phase outcome:
- provider ID aliases are accepted only at input edges
- the stored/runtime provider identity is canonical everywhere else
- provider-specific auth, default models, base URLs, and registry lookups behave consistently

## Phase 3: Make Reported Capabilities Match Runtime Truth

Goal: stop interfaces from claiming support they do not actually provide.

### 3.1 ACP contract correction

Source:
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md)

Fix set:
- make sessions real or remove the fake session lifecycle
- align advertised capabilities with implemented behavior
- replace static discovery with runtime-backed discovery
- enforce JSON-RPC validation

Why here:
- ACP is externally consumed, so misleading behavior creates integration churn
- it should be corrected after the core runtime/provider layer is stabilized

### 3.2 Commands reporting truthfulness

Source:
- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md)

Fix set:
- rework `/providers` around live runtime provider state

Why here:
- this depends on provider canonicalization and runtime truth already being fixed

### 3.3 MCP transport and routing truthfulness

Source:
- [mcp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/mcp_review.md)

Fix set:
- real transport dispatch for stdio vs HTTP/SSE
- unambiguous tool routing
- reconnect bookkeeping cleanup

Why here:
- some pieces are close to show-stopper severity, but they still depend on a clear runtime contract
- this should follow the earlier stabilization work unless MCP is your immediate product priority

### 3.4 Bridge config/API contract cleanup

Source:
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md)

Fix set:
- either implement or remove `session_timeout_ms`
- honor injected HTTP clients

Why here:
- these are contract cleanups after event durability and reconnect correctness are fixed

## Phase 4: Metadata Quality and Lower-Risk Correctness

Goal: address issues that matter, but are less urgent than the earlier correctness and safety work.

### 4.1 OpenAI-compatible metadata honesty

Source:
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md)

Fix set:
- stop inventing exact model limits
- surface serialization failures explicitly

### 4.2 Tool behavior contract alignment

Source:
- [tools_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tools_review.md)

Fix set:
- decide and document Bash persistence semantics accurately
- align runtime behavior with that contract

### 4.3 CLI/TUI state polish

Sources:
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md)
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md)

Fix set:
- stale model label recomputation
- remaining provider inference edge cases

### 4.4 Buddy persistence hardening

Source:
- [buddy_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/buddy_review.md)

Fix set:
- per-user persistence scoping
- atomic writes
- explicit load-failure handling

## Suggested Workstream Split

If multiple engineers are working in parallel, this split minimizes overlap:

### Workstream A: runtime safety and session correctness

Modules:
- `query`
- `tools`
- `bridge`

### Workstream B: provider identity and config resolution

Modules:
- `core`
- `api`
- `query`
- `tui`
- `commands`
- `cli`

Note:
- `query` overlaps both A and B, so it should be sequenced carefully or owned by one person.

### Workstream C: external contract surfaces

Modules:
- `acp`
- `mcp`
- `commands`
- `bridge`

### Workstream D: extensibility and peripheral integrity

Modules:
- `plugins`
- `buddy`

## Dependency Notes

- `core` provider canonicalization should land before broad `tui` and `commands` cleanup.
- `api` runtime lookup fixes should land before `query` provider-construction cleanup is considered complete.
- `query` compaction and stream-error fixes do not depend on provider canonicalization and should be done immediately.
- `commands /providers` should wait until runtime provider truth is fixed in `core` and `api`.
- ACP runtime-backed model/tool discovery should ideally wait until provider/tool truth surfaces are stabilized.

## Fastest Risk-Reduction Order

If only a small number of fixes can be done now, use this order:

1. `tools`: path-boundary enforcement.
2. `query`: compaction failure preservation.
3. `query`: stream error handling.
4. `bridge`: outbound event durability and outer reconnect.
5. `core` + `api`: shared provider canonicalization foundation.
6. `query` + `tui` + `commands` + `cli`: provider canonicalization rollout.
7. `plugins`: real reload semantics.
8. `acp`: remove or implement fake session behavior.
9. `mcp`: real transport support and OAuth completion correctness.

## Completion Criterion

The refactor/reliability wave is in good shape when:

- no core execution path can silently drop conversation state
- no tool can mutate files outside approved roots
- provider identity is canonical across config, runtime, UI, and auth
- operational surfaces report actual runtime truth
- plugin and bridge state survive transient failures without silent drift
