# Buddy Module Review

Module under review: `src-rust/crates/buddy`

Why this module now: `buddy` is a peripheral subsystem rather than core infrastructure, but it still owns deterministic identity derivation, rendering, and on-disk persistence. The meaningful risks here are data integrity and state semantics rather than runtime control flow.

## Findings

### 1. Medium: companion soul persistence is not keyed by user identity

Files:
- `src-rust/crates/buddy/src/lib.rs:377`
- `src-rust/crates/buddy/src/lib.rs:953`
- `src-rust/crates/buddy/src/lib.rs:978`

`CompanionBones` are derived from `user_id`, but the persisted soul is always loaded from a single file:

- `{config_dir}/companion.json`

There is no user-scoping in the persistence path. If multiple user identities share the same config directory, they will all load the same persisted soul while deriving different bones.

That creates a mixed identity model where the visual/body traits come from one user ID and the persisted name/personality come from whichever user last wrote the shared file.

Impact:
- companion identity can bleed across users or profiles sharing the same config dir
- rendered companion state becomes internally inconsistent across soul vs bones
- the persistence model does not match the user-scoped derivation model

### 2. Medium: companion soul writes are not atomic

Files:
- `src-rust/crates/buddy/src/lib.rs:963`
- `src-rust/crates/buddy/src/lib.rs:967`

`save_companion_soul()` writes directly to `companion.json` with `std::fs::write`. If the process crashes or the filesystem write is interrupted mid-write, the persisted file can be left truncated or partially written.

Since `load_companion_soul()` treats parse failure as `None`, a corrupted write can silently look like an unhatched companion on the next load.

Impact:
- partial writes can corrupt the only persisted companion soul
- corruption is silently downgraded into “no soul loaded”
- users can lose companion identity without a visible error

### 3. Low: load failures are silently treated the same as “no companion exists yet”

Files:
- `src-rust/crates/buddy/src/lib.rs:953`
- `src-rust/crates/buddy/src/lib.rs:957`
- `src-rust/crates/buddy/src/lib.rs:958`

`load_companion_soul()` returns `None` for all of these cases:

- file does not exist
- file cannot be read
- JSON cannot be parsed

That keeps callers simple, but it also erases the distinction between first-run state and data corruption or I/O problems.

Impact:
- corrupted or unreadable companion data fails silently
- debugging persistence problems is harder than necessary
- users may see their companion “reset” with no explanation

## Fix Plan

### 1. Scope persisted companion souls by user identity

Approach:
- include a stable user-derived identifier in the persisted file name or directory layout
- alternatively store the `user_id` or its hash in the file and reject mismatches on load
- keep the persistence model aligned with deterministic bone derivation

Implementation target:
- `src-rust/crates/buddy/src/lib.rs`

Verification:
- two different user IDs sharing the same config root should no longer load the same soul accidentally

### 2. Make soul persistence atomic

Approach:
- write to a temporary file in the same directory and rename it into place
- optionally fsync if strong durability is important for this subsystem

Implementation target:
- `src-rust/crates/buddy/src/lib.rs`

Verification:
- interrupted or repeated writes should never leave a partially written `companion.json`

### 3. Distinguish “missing companion” from “failed to load companion”

Approach:
- return a richer result type from `load_companion_soul()` or log parse/read failures explicitly
- preserve the simple caller flow while surfacing corruption and I/O issues somewhere observable

Implementation target:
- `src-rust/crates/buddy/src/lib.rs`

Verification:
- missing file remains a normal first-run case
- malformed JSON and read failures produce a distinct log or error path

## Recommended execution order

1. scope persisted souls by user identity
2. make soul persistence atomic
3. distinguish missing-state from load failure
