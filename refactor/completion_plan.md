# Remaining Refactor To-Do

Purpose: define the small amount of cleanup that remains after the completed `llama.cpp`-first refactor.

Current baseline:

- `llama.cpp` is the first-class path
- Anthropic is optional best-effort support
- `bridge`, `acp`, and `buddy` are removed
- provider breadth was reduced
- plugins were reduced to an honest local-only core
- final verification already passed on the active product path

This file is intentionally forward-looking. Completed execution history was removed from here to keep the remaining work clear.

## Remaining Work

### 1. Dead-Code Purge

Goal:
- remove internal provider leftovers and dead branches that no longer affect active behavior

Focus:
- `src-rust/crates/api`
- `src-rust/crates/core`
- `src-rust/crates/tui`

Examples:
- unused provider constants
- unused provider adapters
- stale helper branches that mention removed providers

Done when:
- active code paths no longer carry obviously dead provider-specific scaffolding
- the remaining provider surface in code matches the supported product surface

### 2. Refactor Doc Cleanup

Goal:
- keep only useful current-state docs in `refactor/`

Required work:
- remove planning docs that only describe already-finished execution
- keep architectural references and review docs only if they still provide value
- ensure surviving docs match the current codebase

Done when:
- `refactor/` is easier to scan
- there is one clear current to-do plan instead of many completed plans

### 3. Secondary-Scope Confirmation

Goal:
- confirm that remaining secondary surfaces stay honest and minimal

Focus:
- `mcp`
- remaining `plugins` surface

Required work:
- check that current behavior matches current product claims
- avoid reopening feature breadth unless it directly supports the local-first core

Done when:
- no remaining secondary surface overclaims runtime capability

### 4. Selective Internal Cleanup

Goal:
- improve maintainability only where it still helps the lean local-first core

Allowed scope:
- shared abstractions that still create real confusion or coupling
- narrow cleanup that reduces maintenance cost

Not in scope:
- broad renaming for aesthetics
- provider-neutral abstraction work that does not improve behavior or maintenance

Done when:
- remaining cleanup improves clarity or code size without reopening the refactor

## Working Rules

1. Keep `llama.cpp` first-class and Anthropic best-effort only.
2. Do not reopen removed subsystems.
3. Prefer deletion over carrying dead compatibility branches.
4. Treat this as cleanup work, not a new architectural rewrite.

## Recommended Order

1. dead-code purge
2. refactor doc cleanup
3. secondary-scope confirmation
4. selective internal cleanup only if still justified

## Definition Of Done

The remaining refactor work is done when:

- no misleading active-path code remains for removed providers or subsystems
- `refactor/` contains only useful current-state docs plus any deliberately kept review archive
- secondary surfaces stay minimal and truthful
- the codebase remains aligned with `refactor/codebase_vision.md`
