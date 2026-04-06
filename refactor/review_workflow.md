# Review Workflow

Purpose: define how to use review effectively during the refactor so risks are caught at topic-branch boundaries instead of only at the end.

## Core Rule

Use review on each bounded branch or worktree, not just on the final integrated result.

Review is most useful when the scope is:

- small enough to reason about
- behaviorally meaningful
- ready for commit or merge

## Recommended Review Cycle

1. finish one bounded change in a topic branch
2. ask for review on the current diff
3. fix actionable findings
4. re-run review if the change is risky or the fixes were substantial
5. commit
6. merge into the integration branch
7. optionally review the integration diff again after merge

## Best Times To Use Review

### Before commit

Use review before committing when the change touches:

- runtime logic
- provider selection
- config resolution
- tool safety
- module removal

### Before merge

Use review before merging a topic branch into the integration branch when:

- the branch changes shared code
- the branch deletes code
- the branch rewires defaults or provider behavior

### After conflict resolution

If a rebase or merge required manual conflict resolution, run review again.

## Best Prompt Patterns

Use short, explicit prompts.

General:

- `review the current diff`
- `review this branch`
- `review my current changes`

Risk-focused:

- `review this with focus on regressions`
- `review this with focus on runtime safety`
- `review this with focus on provider logic`
- `review this deletion for hidden dependencies`
- `review this with focus on config and state consistency`

## Review Guidance By Planned Branch

### Branch: `remove/bridge-acp-buddy`

Ask:

- `review the current diff with focus on hidden dependencies from removing bridge, acp, and buddy`
- `review this deletion for missed wiring, imports, and build regressions`

Focus:

- dead references
- build/config breakage
- command or UI paths still assuming removed modules exist

### Branch: `fix/query-safety`

Ask:

- `review the current diff with focus on runtime safety and regressions`
- `review this query/tools change for state corruption and failure-path bugs`

Focus:

- message loss
- fake successful turns
- path-boundary enforcement
- worktree state leakage

### Branch: `refactor/provider-foundation`

Ask:

- `review the current diff with focus on provider canonicalization and default resolution`
- `review this foundation change for config, auth, and api_base regressions`

Focus:

- canonical provider identity
- default model logic
- auth lookup
- base URL precedence
- provider pruning side effects

### Branch: `simplify/provider-breadth`

Ask:

- `review this diff with focus on provider removal regressions`
- `review this provider-pruning change for leftover references and inconsistent fallbacks`

Focus:

- stale provider references
- broken UI/model picker assumptions
- dead config paths

### Branch: `refactor/provider-rollout`

Ask:

- `review the current diff with focus on provider state consistency`
- `review this llama.cpp-first rollout for UI, CLI, and command regressions`

Focus:

- `/connect`
- `/model`
- session resume
- provider/model synchronization
- Anthropic no longer being implicit

### Branch: `fix/plugins-core-only`

Ask:

- `review this diff with focus on plugin reload correctness and scope reduction`

Focus:

- stale plugin state
- registry reload correctness
- reduced scope not leaving broken marketplace paths

### Branch: `fix/mcp-core-scope`

Ask:

- `review this diff with focus on MCP transport truthfulness and reduced-scope regressions`

Focus:

- transport claims vs implementation
- routing correctness
- removed paths still referenced elsewhere

## When Review Is Less Valuable

Skip or de-emphasize review for:

- pure documentation edits
- trivial wording changes
- mechanical renames with no behavior change

## Worktree-Specific Guidance

For worktrees:

- review inside the worktree that owns the topic branch
- do not wait until all worktrees are merged
- treat each worktree review as a quality gate before integration

## Integration Branch Review

After merging a topic branch into the integration branch, ask for another review if:

- the merge involved conflicts
- the topic touched shared runtime modules
- behavior changed across module boundaries

Good prompt:

- `review the integration diff with focus on cross-module regressions`

## Decision Rule

If a branch changes:

- `core`
- `api`
- `query`
- `cli`
- `tui`

then it should almost always be reviewed before merge.

## Short Version

- review every meaningful topic branch
- review before merge
- review after conflict resolution
- use small scopes
- ask for risk-focused review prompts, not just generic review
