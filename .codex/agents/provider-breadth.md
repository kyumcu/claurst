# Provider Breadth Agent

- branch: `simplify/provider-breadth`
- worktree: `.codex/worktrees/provider-breadth`
- ownership area: provider registry/provider list cleanup across `core`, `api`, `commands`, and `tui`
- current task: remove lower-priority providers that complicate the shared runtime
- blockers: should align with provider-foundation decisions
- files changed: none yet
- tests run: none yet
- last review status: not run yet

Rules:

- prune aggressively where maintenance cost is not justified
- do not preserve low-value provider compatibility just for breadth
- coordinate with provider-foundation on canonical provider policy
