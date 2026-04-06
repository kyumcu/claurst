# Tools Module Review

Module under review: `src-rust/crates/tools`

Why this module next: `tools` is the execution boundary between the model/runtime and the local machine. Bugs here directly affect filesystem safety, shell isolation, worktree correctness, and the integrity of tool-mediated state across the rest of the application.

## Findings

### 1. Critical: tool path resolution does not enforce workspace or allowed-directory boundaries

Files:
- `src-rust/crates/tools/src/lib.rs:225`
- `src-rust/crates/tools/src/file_write.rs:56`
- `src-rust/crates/tools/src/file_edit.rs:76`
- `src-rust/crates/tools/src/apply_patch.rs:325`

`ToolContext::resolve_path()` accepts any absolute path unchanged and otherwise joins a relative path onto `working_dir`. It does not verify that the resolved path stays inside the workspace, inside configured writable roots, or inside any explicit allowlist.

The write/edit/apply-patch flows then use that resolved path directly for file mutation. The generic permission hooks only receive descriptive labels and do not themselves constrain filesystem scope at this layer.

That means the model-facing tool layer can be pointed at arbitrary filesystem paths as long as the caller reaches these tools, even though the broader application clearly has a notion of workspace roots and additional directories elsewhere.

Impact:
- arbitrary file mutation outside the project workspace
- inconsistent enforcement between tool types and higher layers
- high-risk safety bug in the main execution surface

### 2. High: worktree session state is process-global instead of session-scoped

Files:
- `src-rust/crates/tools/src/worktree.rs:21`

`WORKTREE_SESSION` is stored as a single global `Option<WorktreeSession>`. The comment says there should be only one active worktree per session, but the implementation is actually one active worktree for the entire process.

If multiple sessions, threads, or user contexts invoke worktree operations concurrently, they can overwrite or observe each other's active worktree state.

Impact:
- cross-session state leakage
- incorrect worktree reuse or cleanup
- race-prone behavior in multi-session or task-heavy usage

### 3. High: `apply_patch` cannot create parent directories for newly added files

Files:
- `src-rust/crates/tools/src/apply_patch.rs:304`
- `src-rust/crates/tools/src/apply_patch.rs:422`
- `src-rust/crates/tools/src/file_write.rs:68`

The patch pipeline supports `*** Add File:` and computes the new file content correctly, but the final write path goes straight to `tokio::fs::write(path, new_content)` without creating parent directories first.

`file_write` already handles this case by calling `create_dir_all(parent)`, so behavior is inconsistent across two write-capable tools. As a result, patches that add a file under a not-yet-existing directory fail even when the patch itself is otherwise valid.

Impact:
- patch application fails for valid new-file operations
- inconsistent behavior between `write_file` and `apply_patch`
- user-visible failures during common code generation or refactor flows

### 4. Medium: Bash tool behavior contradicts its own contract about shell-state persistence

Files:
- `src-rust/crates/tools/src/bash.rs:257`
- `src-rust/crates/tools/src/bash.rs:316`
- `src-rust/crates/tools/src/lib.rs:170`

The Bash tool description says the working directory persists between commands but shell state does not. The implementation, however, persists more than just the working directory:

- environment variable exports are captured and restored
- shell state is reconstructed through wrapper logic and per-session shell state storage

So the model is told one contract while the runtime provides a different one. Even if this was intentional, it is still a logic bug because safety and reasoning depend on accurate tool semantics.

Impact:
- model receives incorrect guarantees about shell isolation
- environment mutations can leak across calls unexpectedly
- debugging tool behavior becomes harder because docs and runtime diverge

## Fix Plan

### 1. Add canonical path authorization in the tool layer

Approach:
- introduce a single helper that:
  - resolves the path
  - canonicalizes it where safe
  - verifies it stays within allowed roots
- make the allowlist explicit from:
  - `working_dir`
  - configured workspace paths
  - configured additional directories where intended
- require all filesystem tools to call this helper before read or write access

Implementation targets:
- `src-rust/crates/tools/src/lib.rs`
- `src-rust/crates/tools/src/file_write.rs`
- `src-rust/crates/tools/src/file_edit.rs`
- `src-rust/crates/tools/src/apply_patch.rs`
- any other filesystem-touching tools in this crate

Verification:
- tests for relative paths inside the workspace
- tests for absolute paths inside allowed roots
- tests for traversal and out-of-workspace absolute paths being rejected

### 2. Replace the global worktree singleton with session-keyed state

Approach:
- store worktree sessions in a map keyed by session ID, mirroring the pattern already used for per-session shell state
- ensure create/read/cleanup paths all operate on the current session entry only
- make cleanup deterministic when a session ends

Implementation target:
- `src-rust/crates/tools/src/worktree.rs`

Verification:
- tests with two concurrent session IDs
- assert one session cannot see or clobber another session's worktree

### 3. Make `apply_patch` create parent directories for new files

Approach:
- before writing patched content, create `path.parent()` when it does not already exist
- align the behavior with `file_write`
- keep this limited to add/update paths that legitimately produce a new file

Implementation target:
- `src-rust/crates/tools/src/apply_patch.rs`

Verification:
- add a patch test that creates `nested/dir/file.rs`
- assert the patch succeeds without precreating directories

### 4. Reconcile Bash tool semantics with actual persisted state

Approach:
- choose one contract and enforce it consistently:
  - either persist only `cwd`, or
  - explicitly document that exported environment state also persists
- if env persistence remains, audit whether all persisted state is safe and intended
- add regression tests for the chosen behavior

Implementation targets:
- `src-rust/crates/tools/src/bash.rs`
- `src-rust/crates/tools/src/lib.rs`

Verification:
- tests for `cd` persistence
- tests for `export FOO=bar` persistence or non-persistence, depending on the chosen contract
- ensure the tool description matches runtime behavior exactly

## Recommended execution order

1. add path-boundary enforcement in the tool layer
2. make worktree session state session-scoped
3. fix parent-directory creation in `apply_patch`
4. reconcile Bash persistence behavior with its documented contract
