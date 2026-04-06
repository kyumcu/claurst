# Refactor Completion Plan

Purpose: define the shortest practical path from the current `main` state to a completed refactor, using one worktree at a time and no sub-agents.

Current baseline:

- `main` is at `c466424`
- `bridge`, `acp`, and `buddy` are already removed
- query/tools safety fixes are already merged
- provider foundation in `core` and `api` is already merged
- provider rollout is already merged
- provider breadth reduction is already merged
- remaining prepared worktrees should be aligned to `main` before use

## Working Rules

1. Use one worktree at a time.
2. Make one coherent change set per lane.
3. Run targeted verification before every commit.
4. Merge each completed lane to `main` before starting the next one.
5. Do not reopen removed subsystems.
6. Keep `llama.cpp` first-class and Anthropic best-effort only.

## Phase 1: Provider Rollout

Worktree:
- `.codex/worktrees/provider-rollout`

Goal:
- turn the merged provider foundation into actual `llama.cpp`-first user-facing behavior

Scope:
- `src-rust/crates/tui`
- `src-rust/crates/commands`
- `src-rust/crates/cli`
- minimal `src-rust/crates/query` glue only if strictly required

Required work:
- make `/connect` persist canonical provider IDs only
- make `/model` and model picker behavior consume canonical provider logic
- make startup and resume keep provider/model state synchronized
- remove remaining Anthropic-first defaults from UI and command flows
- ensure `llama.cpp` is the clean default local path

Verification:
- targeted `cargo check` for changed crates
- targeted tests for provider/model selection logic where available
- manual local verification of:
  - `/connect` with `llama.cpp`
  - `/model`
  - startup/resume behavior

Status:
- completed and merged into `main`

## Phase 2: Provider Breadth Reduction

Worktree:
- `.codex/worktrees/provider-breadth`

Goal:
- reduce provider surface area to match the lean local-first product direction

Scope:
- provider lists
- provider registry exposure
- command/UI surfaces that enumerate providers
- related docs or config defaults if they become misleading

Required work:
- remove lower-priority providers that are no longer worth carrying
- keep Anthropic only as best-effort optional support
- ensure remaining provider choices are honest and coherent
- remove stale alias-driven or dead provider paths where they no longer serve the product

Verification:
- targeted `cargo check`
- targeted provider registry and default-model tests
- sanity-check `/connect` and `/providers` style flows after pruning

Status:
- completed and merged into `main`

## Phase 3: Plugins Decision And Cleanup

Worktree:
- `.codex/worktrees/plugins`

Goal:
- either keep plugins as a reduced honest surface or trim them further

Required work:
- compare current plugin behavior to the lean local-first vision
- remove or reduce misleading plugin behavior if necessary
- avoid expanding plugin complexity before the core path is complete

Verification:
- targeted `cargo check`
- targeted plugin lifecycle tests if touched

Status:
- completed and merged into `main`
- live plugin reload removed
- marketplace surface removed
- local plugin install/list/info/enable/disable kept as the honest core

## Phase 4: Final Verification

Worktree:
- main checkout or `.codex/worktrees/integration`

Goal:
- prove the refactor is complete in real product terms

Required verification:
- full `cargo check`
- targeted tests for:
  - provider canonicalization
  - runtime safety fixes
  - provider/model flow correctness
- manual `llama.cpp` happy-path verification:
  - connect
  - model discovery
  - prompt/response loop
  - resume behavior

Also verify:
- no stale product-facing references to `bridge`, `acp`, or `buddy`
- no stale Anthropic-first UX defaults in active flows
- no lower-priority provider surfaces that contradict the new scope

Done when:
- `main` is stable
- the user-facing local path is coherent
- the codebase behavior matches the vision docs

## Phase 5: Final Cleanup

Goal:
- remove remaining confusion after the technical refactor is complete

Required work:
- update or archive refactor docs that no longer reflect the live repo state
- remove stale branches/worktree lanes if no longer needed
- clean up obsolete comments or docs that still describe old behavior

Done when:
- repo docs and branch layout match the finished state

## Immediate Execution Order

1. run final verification
2. perform final cleanup

## Definition Of Completion

The refactor is complete when:

- `llama.cpp` is the clean first-class path end to end
- Anthropic is optional and not central
- lower-priority providers are removed or clearly reduced
- `bridge`, `acp`, and `buddy` remain gone
- query/tools safety fixes remain intact
- `main` passes verification
- the live codebase behavior matches the vision in `refactor/codebase_vision.md`
