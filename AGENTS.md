# Agent Directives

Purpose: define required rules for sub-agents working on this repo during the `llama.cpp`-first refactor.

## Working Model

Sub-agents should use:

- one worktree each under `.codex/worktrees/`
- one directive/status file each under `.codex/agents/`

Recommended layout:

- `.codex/worktrees/integration`
- `.codex/worktrees/remove`
- `.codex/worktrees/runtime`
- `.codex/worktrees/provider-foundation`
- `.codex/worktrees/provider-breadth`
- `.codex/worktrees/provider-rollout`
- `.codex/worktrees/plugins`

- `.codex/agents/integration.md`
- `.codex/agents/remove.md`
- `.codex/agents/runtime.md`
- `.codex/agents/provider-foundation.md`
- `.codex/agents/provider-breadth.md`
- `.codex/agents/provider-rollout.md`
- `.codex/agents/plugins.md`

## Required Directives

1. One branch per worktree.
2. One main responsibility per worktree.
3. Do not switch branches inside a shared worktree.
4. Do not edit files outside your assigned ownership area unless the handoff explicitly requires it.
5. Do not revert unrelated edits.
6. Keep commits small and focused.
7. Record status in your matching `.codex/agents/<name>.md` file.

## Execution Protocol

Before responding, state the intended end state or user objective in one clear sentence.

Then:

1. Identify the key requirements, constraints, and assumptions.
2. Create a short plan before acting.
3. Validate the plan for completeness and correctness before execution.
4. Update the plan as the task progresses.

Use available tools deliberately:

- keep progress structured and updated
- request user input only when necessary
- for file or code tasks, read files before modifying them
- write carefully and use shell commands when appropriate

Execution standard:

- execute systematically and efficiently
- after execution, verify the result against the request, the intended end state, and internal correctness
- if anything is missing, inconsistent, or uncertain, revise before finalizing
- output only the final validated answer unless intermediate steps are required
- always double-check your work

## Refactor Priorities

The current refactor direction is:

- `llama.cpp` is the first-class path
- Anthropic is best-effort compatibility only
- lower-priority providers may be removed
- `bridge`, `acp`, and `buddy` are removal targets
- provider-neutral behavior matters more than internal naming purity

## Sub-Agent Rules

### Runtime agents

Applies to:

- `.codex/worktrees/runtime`

Focus:

- query safety
- tool safety
- failure-path correctness

Rules:

- do not redesign provider abstractions
- do not introduce new provider defaults
- prioritize correctness over cleanup

### Removal agents

Applies to:

- `.codex/worktrees/remove`

Focus:

- remove `bridge`
- remove `acp`
- remove `buddy`

Rules:

- remove wiring cleanly
- keep the repo buildable
- avoid replacing removed features with new complexity

### Provider foundation agents

Applies to:

- `.codex/worktrees/provider-foundation`
- `.codex/worktrees/provider-breadth`

Focus:

- canonical provider identity
- auth/base URL/default-model behavior
- provider pruning

Rules:

- remove Anthropic-first defaults
- keep Anthropic only on a best-effort basis
- drop lower-priority providers if they materially simplify the shared runtime
- do not turn this into a repo-wide renaming exercise

### Provider rollout agents

Applies to:

- `.codex/worktrees/provider-rollout`

Focus:

- `/connect`
- `/model`
- CLI/TUI/provider state
- `llama.cpp`-first user flow

Rules:

- consume the provider-foundation contract rather than redefining it
- prefer behaviorally provider-neutral fixes
- only generalize deeper internal types if they are blocking shared behavior

### Plugins and secondary-scope agents

Applies to:

- `.codex/worktrees/plugins`

Focus:

- reliable plugin core only

Rules:

- do not expand marketplace complexity
- keep scope minimal and honest

## Review Requirement

Sub-agents should request review:

- before merging their topic branch
- after large conflict resolutions
- after risky shared-runtime changes

Good prompts:

- `review the current diff`
- `review this with focus on regressions`
- `review this with focus on provider logic`
- `review this deletion for hidden dependencies`

## Required Status Format

Each `.codex/agents/<name>.md` file should track:

- branch name
- worktree path
- ownership area
- current task
- blockers
- files changed
- tests run
- last review status

## Conflict Rule

If a sub-agent is blocked by another workstream:

- stop
- record the dependency in its `.codex/agents/<name>.md`
- do not invent a parallel contract

## Final Rule

If a change improves naming aesthetics but does not improve the `llama.cpp`-first core path, defer it.
