# Refactor Execution Roadmap

Purpose: convert the findings in the `refactor/` review set into an execution order that reduces real risk first, avoids thrash, and respects cross-module dependencies.

This is not a design doc for every fix. It is a sequencing document.

## Planning Rules

1. Fix data-loss and trust-boundary issues before UX correctness.
2. Fix shared provider canonicalization before patching individual provider-specific symptoms.
3. Fix runtime truth mismatches before improving reporting surfaces.
4. Prefer coordinated cross-module sweeps where one root cause appears in many crates.
5. Prefer shrinking or disabling misleading optional surfaces over investing in broad partial support.
6. Remove Anthropic-first defaults before investing in new provider breadth.
7. Make shared runtime behavior provider-neutral before attempting full internal type/generalization cleanup.

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

Goal: make runtime state reliable across queueing, worktree, and plugin flows while preparing `bridge` for removal.

### 1.1 Bridge containment before removal

Source:
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md)

Fix set:
- decide whether any behavior must be preserved elsewhere before removal
- contain the most dangerous failure modes only if the module must survive temporarily
- stop investing in deeper bridge behavior beyond shutdown or migration needs

Why now:
- bridge currently creates operational risk while no longer fitting the target product scope
- the goal is to avoid spending large effort on a subsystem that should be deleted

Expected outcome:
- either a safe temporary bridge state or a clear path to removal

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

Decision gate after 1.4:
- if plugins are not core to the product, stop after making the current behavior honest and stable
- do not expand marketplace or reload breadth before the core local workflow is finished

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

## Phase 2.5: Remove Anthropic Centrality

Goal: keep Anthropic support, but stop treating it as the implicit product default in runtime behavior, UX, and architecture.

Sources:
- [core_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/core_review.md)
- [cli_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/cli_review.md)
- [commands_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/commands_review.md)
- [tui_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/tui_review.md)
- [api_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/api_review.md)

### 2.5.1 Make `llama.cpp` the clean default local path

Fix set:
- ensure onboarding and `/connect` emphasize `llama.cpp`
- ensure local model discovery works without Anthropic-first fallback behavior
- ensure `api_base` and model selection behave correctly for `llama.cpp` without extra config gymnastics

Primary modules:
- `tui`
- `commands`
- `cli`

### 2.5.2 Make Anthropic explicit rather than implicit

Fix set:
- remove control-flow assumptions that default to `anthropic` unless a provider is explicitly chosen
- stop using Anthropic model families as unrelated-provider fallbacks
- reduce Anthropic-specific bootstrap assumptions in auth and resume paths

Primary modules:
- `core`
- `cli`
- `query`

### 2.5.3 Move toward provider-neutral runtime abstractions

Fix set:
- reduce reliance on Anthropic-shaped naming and fallback semantics in generic runtime code
- prefer generic provider/client/event concepts where practical
- keep Anthropic-specific code inside the Anthropic provider path rather than in shared layers

Primary modules:
- `api`
- `query`
- `tui`

Expected phase outcome:
- Anthropic remains supported
- Anthropic is no longer the hidden product identity
- `llama.cpp` becomes the clean first-class path in the actual behavior of the app

### 2.5.4 Hold the abstraction line

Rule:
- make shared runtime behavior provider-neutral now
- generalize internal abstractions only where they simplify real shared logic
- defer repo-wide naming cleanup and abstraction purity work that does not materially improve the local-first core

Examples of must-fix now:
- provider selection and fallback logic
- auth and base URL resolution
- shared query dispatch behavior
- shared model/provider persistence and resume logic

Examples that can wait:
- renaming every Anthropic-flavored type immediately
- genericizing peripheral code that no longer shapes product behavior
- broad internal cleanup done only for conceptual neatness

## Phase 3: Make Reported Capabilities Match Runtime Truth

Goal: stop interfaces from claiming support they do not actually provide.

### 3.1 ACP removal plan

Source:
- [acp_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/acp_review.md)

Fix set:
- identify callers and codepaths that still depend on ACP
- remove the fake session lifecycle instead of broadening it
- remove misleading capability/reporting surface alongside the module
- preserve only any thin compatibility layer that is absolutely required during transition

Why here:
- ACP is externally consumed, so removal needs to happen deliberately rather than by leaving a broken stub
- it should be handled after the core runtime/provider layer is stabilized

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

Preferred lean outcome:
- support only the MCP transports and routing patterns that are actually used
- defer broader MCP capability breadth until the local-first core is complete

### 3.4 Bridge removal plan

Source:
- [bridge_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/bridge_review.md)

Fix set:
- identify whether any user-visible flow still depends on bridge
- remove bridge-specific config and API hooks that are no longer needed
- avoid deep cleanup work that only matters if bridge remains a supported subsystem

Why here:
- bridge should be removed cleanly after the core path is stable enough that its absence is acceptable

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

### 4.3a Internal abstraction cleanup that still matters

Fix set:
- replace Anthropic-shaped shared abstractions only where they still create real coupling in `api`, `query`, or `tui`
- keep this tightly scoped to code that distorts behavior or blocks maintainability
- avoid turning this into a full internal-renaming sweep

### 4.4 Buddy removal plan

Source:
- [buddy_review.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/buddy_review.md)

Fix set:
- identify any remaining compile-time or runtime dependencies
- remove buddy from the maintained product surface
- only do persistence cleanup if required to support safe removal or migration

## Phase 5: Scope Reduction and Simplification

Goal: remove or reduce parts of the codebase that add complexity without strengthening the local `llama.cpp` workflow.

### 5.1 Reduce provider sprawl pressure

Targets:
- alias handling duplicated across many crates
- remote-provider-specific branches that do not meet the core quality bar

Outcome:
- one canonical provider system
- fewer ad hoc compatibility branches

### 5.2 Remove non-core subsystems

Targets:
- `acp`
- `bridge`
- `buddy`

Outcome:
- removed from the maintained product surface
- no partially implemented contracts competing with the core runtime

### 5.3 Keep plugins and MCP on a strict scope budget

Targets:
- plugin marketplace and reload behavior
- MCP transport and routing breadth

Outcome:
- only keep the portions that are reliable and actively needed
- defer the rest instead of maintaining misleading partial behavior

## Suggested Workstream Split

If multiple engineers are working in parallel, this split minimizes overlap:

### Workstream A: runtime safety and session correctness

Modules:
- `query`
- `tools`

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
- `mcp`
- `commands`

### Workstream D: extensibility and peripheral integrity

Modules:
- `plugins`

## Dependency Notes

- `core` provider canonicalization should land before broad `tui` and `commands` cleanup.
- `api` runtime lookup fixes should land before `query` provider-construction cleanup is considered complete.
- `query` compaction and stream-error fixes do not depend on provider canonicalization and should be done immediately.
- `commands /providers` should wait until runtime provider truth is fixed in `core` and `api`.
- ACP runtime-backed model/tool discovery should ideally wait until provider/tool truth surfaces are stabilized.
- removal planning for `acp`, `bridge`, and `buddy` should happen before investing in any further feature work inside them.
- removing Anthropic centrality should begin only after canonical provider behavior is defined, otherwise the fallback logic will just move around.
- full internal provider-neutral abstraction cleanup should wait until the behavioral provider-neutral pass is complete, otherwise the refactor will balloon.

## Fastest Risk-Reduction Order

If only a small number of fixes can be done now, use this order:

1. `tools`: path-boundary enforcement.
2. `query`: compaction failure preservation.
3. `query`: stream error handling.
4. contain `bridge` only enough to remove it safely.
5. `core` + `api`: shared provider canonicalization foundation.
6. `query` + `tui` + `commands` + `cli`: provider canonicalization rollout.
7. remove Anthropic-first defaults and make `llama.cpp` the clean first-class path.
8. make shared runtime behavior provider-neutral where it affects real execution.
9. `plugins`: real reload semantics.
10. remove `acp` cleanly instead of broadening it.
11. `mcp`: real transport support and OAuth completion correctness.
12. remove `bridge` and `buddy` from the maintained product surface.

## Completion Criterion

The refactor/reliability wave is in good shape when:

- no core execution path can silently drop conversation state
- no tool can mutate files outside approved roots
- provider identity is canonical across config, runtime, UI, and auth
- Anthropic support is explicit and optional rather than the hidden default path
- `llama.cpp` is the clean default local workflow
- operational surfaces report actual runtime truth
- plugin state survives transient failures without silent drift
- `bridge`, `acp`, and `buddy` are removed from the maintained product surface
