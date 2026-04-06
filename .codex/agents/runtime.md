# Runtime Agent

- branch: `fix/query-safety`
- worktree: `.codex/worktrees/runtime`
- ownership area: `query` and `tools`
- current task: fix runtime safety issues in query and tools
- blockers: none
- files changed: none yet
- tests run: none yet
- last review status: not run yet

Rules:

- prioritize correctness over cleanup
- do not redesign provider abstractions here
- focus on compaction safety, stream failure handling, path boundaries, and worktree isolation
