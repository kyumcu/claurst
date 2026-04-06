# Provider Rollout Agent

- branch: `refactor/provider-rollout`
- worktree: `.codex/worktrees/provider-rollout`
- ownership area: `query`, `tui`, `commands`, and `cli`
- current task: roll provider-foundation changes into the user-facing and runtime flow
- blockers: depends on provider-foundation output
- files changed: none yet
- tests run: none yet
- last review status: not run yet

Rules:

- consume the provider-foundation contract, do not redefine it
- make `llama.cpp` the clean first-class path
- remove Anthropic-first UX/default behavior
- only generalize internal types if they are blocking provider-neutral behavior
