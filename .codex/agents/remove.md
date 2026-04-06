# Removal Agent

- branch: `remove/bridge-acp-buddy`
- worktree: `.codex/worktrees/remove`
- ownership area: `bridge`, `acp`, `buddy`, and wiring that references them
- current task: remove `bridge`, `acp`, and `buddy` while keeping the repo buildable
- blockers: none
- files changed: none yet
- tests run: none yet
- last review status: not run yet

Rules:

- remove code cleanly rather than stubbing it
- do not add replacement complexity
- if a dependency must move elsewhere, record it explicitly
