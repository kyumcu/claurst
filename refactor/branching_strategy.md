# Branching And Worktree Strategy

Purpose: record the branching/worktree model that was used during the refactor and describe the finished-state cleanup policy now that the main refactor work is merged.

Status:

- historical strategy retained for future large refactors
- topic-branch execution is complete
- local topic worktrees should be removed after merge
- day-to-day work should happen directly on fresh branches from `main`

## Recommendation

For large refactors, use:

- one integration branch
- multiple small topic branches
- one `git worktree` per active topic branch

Do not use one giant refactor branch for the whole effort.

For the current repo state:

- the dedicated refactor worktrees are no longer needed
- merged topic branches can be deleted locally
- new work should start from `main` with fresh short-lived branches only when needed

## Why Not One Huge Branch

This refactor combines several independent but interacting changes:

- immediate removal of `bridge`, `acp`, and `buddy`
- query/runtime safety fixes
- provider foundation cleanup
- provider pruning
- `llama.cpp`-first rollout
- best-effort Anthropic compatibility

If these land in one long-lived branch:

- rebases get painful
- review quality drops
- failures become harder to localize
- rollback gets harder
- parallel work causes avoidable conflicts

## Why Worktrees Fit This Refactor

`git worktree` is a good fit because it lets you:

- keep several active topic branches checked out at once
- avoid stashing while switching streams
- run builds/tests independently per stream
- hand separate directories to sub-agents cleanly
- isolate removal work from runtime work

For this codebase, that is more practical than constantly switching one checkout.

## Local Coordination Layout

Use a gitignored `.codex/` directory inside the repo as the local coordination root.

Recommended structure:

- `.codex/worktrees/` for git worktrees
- `.codex/agents/` for per-agent directives and handoff files
- `.codex/logs/` for optional local notes

This keeps worktrees near the repo while avoiding tracked-file noise.

## Target Model

### 1. Integration branch

Create one long-lived branch that represents the current accepted state of the refactor.

Suggested name:

- `refactor/llamacpp-first`

This branch should only receive:

- completed topic branches
- conflict resolution
- integration verification

It should not become the place where all work happens directly.

### 2. Topic branches

Each major change area should get its own branch with a narrow purpose.

Suggested branches:

- `remove/bridge-acp-buddy`
- `fix/query-safety`
- `refactor/provider-foundation`
- `simplify/provider-breadth`
- `refactor/provider-rollout`
- `fix/plugins-core-only`

Optional follow-up branches:

- `cleanup/provider-runtime-abstractions`
- `fix/mcp-core-scope`

## Historical Worktree Layout

Suggested directories under `.codex/worktrees/`:

- `.codex/worktrees/integration`
- `.codex/worktrees/remove`
- `.codex/worktrees/runtime`
- `.codex/worktrees/provider-foundation`
- `.codex/worktrees/provider-breadth`
- `.codex/worktrees/provider-rollout`
- `.codex/worktrees/plugins`

Map them like this:

- `.codex/worktrees/integration` → `refactor/llamacpp-first`
- `.codex/worktrees/remove` → `remove/bridge-acp-buddy`
- `.codex/worktrees/runtime` → `fix/query-safety`
- `.codex/worktrees/provider-foundation` → `refactor/provider-foundation`
- `.codex/worktrees/provider-breadth` → `simplify/provider-breadth`
- `.codex/worktrees/provider-rollout` → `refactor/provider-rollout`
- `.codex/worktrees/plugins` → `fix/plugins-core-only`

Tracked coordination files that remain as durable records:

- `.codex/agents/integration.md`
- `.codex/agents/remove.md`
- `.codex/agents/runtime.md`
- `.codex/agents/provider-foundation.md`
- `.codex/agents/provider-breadth.md`
- `.codex/agents/provider-rollout.md`
- `.codex/agents/plugins.md`

Tracked `.codex/agents/*.md` files should remain:

- directives
- templates
- durable status records

They should not become:

- noisy local logs
- scratchpads
- frequently changing personal notes

## Ownership Guidance

To reduce conflicts, keep branch ownership aligned with module boundaries.

### Branch: `remove/bridge-acp-buddy`

Primary ownership:

- [`src-rust/crates/bridge`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/bridge)
- [`src-rust/crates/acp`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/acp)
- [`src-rust/crates/buddy`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/buddy)
- entrypoint/config wiring that references them

### Branch: `fix/query-safety`

Primary ownership:

- [`src-rust/crates/query`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/query)
- [`src-rust/crates/tools`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tools)

### Branch: `refactor/provider-foundation`

Primary ownership:

- [`src-rust/crates/core`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/core)
- [`src-rust/crates/api`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/api)

### Branch: `simplify/provider-breadth`

Primary ownership:

- provider registry and provider list cleanup in:
  - [`src-rust/crates/core`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/core)
  - [`src-rust/crates/api`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/api)
  - [`src-rust/crates/commands`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/commands)
  - [`src-rust/crates/tui`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tui)

### Branch: `refactor/provider-rollout`

Primary ownership:

- [`src-rust/crates/query`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/query)
- [`src-rust/crates/tui`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tui)
- [`src-rust/crates/commands`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/commands)
- [`src-rust/crates/cli`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/cli)

### Branch: `fix/plugins-core-only`

Primary ownership:

- [`src-rust/crates/plugins`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/plugins)

## Historical Merge Order

The refactor was merged in this order into `refactor/llamacpp-first`:

1. `remove/bridge-acp-buddy`
2. `fix/query-safety`
3. `refactor/provider-foundation`
4. `simplify/provider-breadth`
5. `refactor/provider-rollout`
6. `fix/plugins-core-only`

Optional later:

7. `fix/mcp-core-scope`
8. `cleanup/provider-runtime-abstractions`

## Why This Order

### 1. Remove dead surface first

This immediately lowers maintenance load and prevents wasted effort on deleted modules.

### 2. Stabilize runtime safety early

No point doing elegant provider work if query/tool behavior can still corrupt sessions.

### 3. Build provider foundation before UI rollout

The rollout work should consume one shared provider contract, not invent a second one.

### 4. Prune providers before finishing rollout

If lower-priority providers are going away, remove them before polishing UI and command behavior around them.

### 5. Roll out `llama.cpp`-first behavior after the foundation is stable

This keeps the user-facing changes aligned with the new core behavior.

## Rebase And Integration Policy

### Topic branches

Topic branches should:

- rebase regularly onto `refactor/llamacpp-first`
- stay focused
- avoid unrelated cleanup

### Integration branch

The integration branch should:

- absorb one topic branch at a time
- run verification after each merge
- resolve cross-branch conflicts centrally

## Finished-State Cleanup Policy

After a refactor wave is merged:

- remove the dedicated worktrees with `git worktree remove`
- delete merged topic branches locally
- keep tracked `.codex/agents/*.md` only as durable coordination records
- avoid leaving `.codex/worktrees/` populated with stale historical checkouts

## Conflict Hotspots

These files or crates are likely to see overlap and should be coordinated carefully:

- [`src-rust/crates/query`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/query)
- [`src-rust/crates/core`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/core)
- [`src-rust/crates/api`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/api)
- [`src-rust/crates/tui`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/tui)
- [`src-rust/crates/commands`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/commands)
- [`src-rust/crates/cli`](/home/manager/Agents/temp/toolsTest/claude/claurst/src-rust/crates/cli)

Rules:

- avoid parallel edits to `core` and `api` across multiple branches where possible
- let `provider-foundation` define the provider contract
- let `provider-rollout` consume it rather than reshape it

## Worktree Operating Rules

1. One branch per worktree.
2. One main responsibility per worktree.
3. Do not mix unrelated fixes in a worktree just because the files are open.
4. Run verification inside the worktree that owns the change.
5. Delete worktrees once their branches are merged.

## Suggested Commands

Example setup:

```bash
git switch -c refactor/llamacpp-first
mkdir -p .codex/worktrees .codex/agents .codex/logs
git worktree add .codex/worktrees/integration refactor/llamacpp-first

git worktree add -b remove/bridge-acp-buddy .codex/worktrees/remove HEAD
git worktree add -b fix/query-safety .codex/worktrees/runtime HEAD
git worktree add -b refactor/provider-foundation .codex/worktrees/provider-foundation HEAD
git worktree add -b simplify/provider-breadth .codex/worktrees/provider-breadth HEAD
git worktree add -b refactor/provider-rollout .codex/worktrees/provider-rollout HEAD
git worktree add -b fix/plugins-core-only .codex/worktrees/plugins HEAD
```

## Decision Summary

Best strategy for this refactor:

- not one giant branch
- not many unrelated edits in one checkout
- yes to multiple small branches
- yes to worktrees
- yes to one integration branch

Short version:

Use topic branches with worktrees, and merge them into one staged integration branch in dependency order.
