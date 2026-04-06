# Sub-Agent Handover Plan

Purpose: break the refactor effort into bounded workstreams that can be handed to sub-agents with minimal overlap and a clear dependency order.

This document assumes the target direction in:

- [codebase_vision.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/codebase_vision.md)
- [execution_roadmap.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/execution_roadmap.md)
- [urgency_classification.md](/home/manager/Agents/temp/toolsTest/claude/claurst/refactor/urgency_classification.md)

## Handover Principles

1. Assign ownership by module, not by abstract theme alone.
2. Put the highest-risk runtime fixes first.
3. Do foundation work before rollout work.
4. Keep write scopes disjoint where possible.
5. Prefer honest blocking reports over parallel reinvention.

## Agent 1: Runtime Stabilization

### Goal

Remove the most dangerous runtime and safety failures in the core execution path.

### Ownership

- [`src-rust/crates/query`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/query)
- [`src-rust/crates/tools`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tools)

### Primary tasks

- fix compaction-related conversation loss
- treat provider stream failures as real query failures
- enforce filesystem path boundaries in tools
- make worktree state session-scoped

### Why first

These issues create the highest operational risk:

- conversation loss
- fake successful turns
- unsafe filesystem access

### Expected deliverable

- merged runtime safety fixes
- tests for failure paths
- a short note describing remaining edge cases

## Agent 2: Provider Foundation

### Goal

Create one canonical provider system in the foundational layers and remove Anthropic-first assumptions from the shared foundation.

### Ownership

- [`src-rust/crates/core`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/core)
- [`src-rust/crates/api`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/api)

### Primary tasks

- add canonical provider-ID normalization
- unify provider-to-env/auth mapping
- fix provider default-model resolution
- add provider-aware base URL resolution
- normalize runtime provider lookup
- fix local/private endpoint classification for OpenAI-compatible providers
- remove Anthropic as the hidden default fallback for unrelated providers where shared foundation code currently assumes it
- make shared foundation behavior provider-neutral without forcing a total internal type rename

### Why separate

This work defines the contract that `query`, `tui`, `commands`, and `cli` should consume. If every layer invents its own normalization again, the same bugs will return.

### Expected deliverable

- one canonical provider contract in foundational code
- table-driven tests for aliases and canonical IDs
- clear helper APIs for higher layers to consume
- a bounded abstraction strategy that avoids a repo-wide purity refactor

## Agent 3: Provider Rollout

### Goal

Apply the provider foundation consistently across the user-facing and runtime layers, making `llama.cpp` the clean first-class path while keeping Anthropic optional.

### Ownership

- [`src-rust/crates/query`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/query)
- [`src-rust/crates/tui`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tui)
- [`src-rust/crates/commands`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/commands)
- [`src-rust/crates/cli`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/cli)

### Primary tasks

- apply canonical provider logic to `/connect`
- normalize registry and model-registry lookups
- synchronize provider and model updates in commands
- restore provider state on session resume
- resolve `api_base` against the effective provider
- make the `llama.cpp` flow reliable end to end
- remove Anthropic-first onboarding and selection behavior where it still shapes the default UX
- make shared runtime behavior provider-neutral in the user-facing flow, even if some deeper internal types remain transitional

### Dependency

This agent should start only after Agent 2 has defined the shared provider behavior.

### Expected deliverable

- no alias-dependent provider behavior in UI, commands, or runtime
- clean `llama.cpp` connection and model-selection path
- Anthropic support remains available but is no longer the hidden product default
- provider-neutral behavior achieved without unnecessary abstraction churn

## Agent 4: Interface Truthfulness

### Goal

Make exposed interfaces and operational reporting match the actual runtime.

### Ownership

- [`src-rust/crates/acp`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/acp)
- [`src-rust/crates/mcp`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/mcp)
- [`src-rust/crates/commands`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/commands)

### Primary tasks

- remove ACP cleanly
- make `/providers` report live runtime state
- narrow MCP transport claims to real implementation
- make MCP tool routing unambiguous

### Why later

These fixes matter, but they depend on a more stable runtime and provider foundation underneath.

### Expected deliverable

- no ACP surface left in the maintained product path
- external surfaces that reflect live state instead of static guesses

## Agent 5: Scope Reduction

### Goal

Reduce code and maintenance load in non-core areas.

### Ownership

- [`src-rust/crates/plugins`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/plugins)
- [`src-rust/crates/acp`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/acp)
- [`src-rust/crates/bridge`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/bridge)
- [`src-rust/crates/buddy`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/buddy)

### Primary tasks

- reduce plugin complexity to a reliable core if needed
- remove ACP
- remove bridge
- remove `buddy`
- handle any migration or cleanup required by those removals

### Why last

This is easier to do correctly once the team knows what the stable core should look like.

### Expected deliverable

- those three subsystems removed cleanly
- less surface area competing with the local-first core

## Dependency Order

### Can start immediately

- Agent 1: Runtime Stabilization
- Agent 2: Provider Foundation

### Should wait for foundation output

- Agent 3: Provider Rollout

### Should mostly follow stabilization

- Agent 4: Interface Truthfulness

### Should follow the vision and early runtime cleanup

- Agent 5: Scope Reduction

## Recommended Parallel Split

### Workstream A

- Agent 1

Focus:
- runtime safety
- tool safety

### Workstream B

- Agent 2

Focus:
- canonical provider identity
- auth/base URL/default-model foundation
- removing Anthropic-first shared defaults
- bounded provider-neutral behavior in shared foundation code

### Workstream C

- Agent 3

Focus:
- provider rollout into UI, commands, CLI, and runtime
- `llama.cpp` as the clean default local path
- behavioral neutrality first, abstraction cleanup only where it improves shared logic

Note:
- start after Workstream B has produced the provider contract

### Workstream D

- Agent 4

Focus:
- ACP removal, MCP, live reporting, and interface truth

### Workstream E

- Agent 5

Focus:
- removing non-core subsystems

## Handover Template

Each sub-agent should receive:

1. owned modules
2. exact tasks
3. dependencies
4. success criteria
5. forbidden scope

Suggested output format from each sub-agent:

- findings addressed
- files changed
- tests run
- unresolved risks
- whether follow-up from another agent is required

## Coordination Rules

- do not revert unrelated edits
- do not redefine shared provider behavior outside the provider-foundation workstream
- do not introduce new Anthropic-first defaults while fixing provider logic
- do not turn provider cleanup into a repo-wide internal renaming effort unless it directly improves shared runtime behavior
- if blocked by another workstream, stop and report the dependency instead of inventing a parallel contract
- prefer small commits per fix cluster
- keep file ownership clear when touching shared modules like `query` or `commands`

## Suggested First Handoffs

### First handoff

Assign Agent 1:

- fix `query` compaction loss
- fix `query` stream error handling
- enforce `tools` path boundaries

### Second handoff

Assign Agent 2:

- define canonical provider normalization
- fix auth/env/default-model/base URL resolution in `core` and `api`
- remove hidden Anthropic-first shared fallbacks
- keep abstraction cleanup bounded to places where it improves real shared behavior

### Third handoff

Assign Agent 1 or a follow-up stabilization worker:

- finish worktree session isolation

### Fourth handoff

Assign Agent 3:

- roll the provider foundation into `query`, `tui`, `commands`, and `cli`
- verify `llama.cpp` end-to-end behavior
- remove Anthropic-first UX and default-selection behavior
- only generalize internal types if they are actively blocking provider-neutral behavior

### Fifth handoff

Assign Agent 4 and Agent 5 as lower-priority parallel tracks after the core path is stable, with explicit removal of `acp`, `bridge`, and `buddy`.
