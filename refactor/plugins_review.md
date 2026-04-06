# Plugins Module Review

Module under review: `src-rust/crates/plugins`

Why this module next: `plugins` controls extensibility across commands, hooks, skills, agents, and MCP/LSP contributions. Bugs here can make reloads misleading, cause installed plugins to vanish from discovery, or silently change the active capability surface of the whole app.

## Findings

### 1. High: plugin reload cannot actually replace the global plugin or hook registries

Files:
- `src-rust/crates/plugins/src/lib.rs:76`
- `src-rust/crates/plugins/src/lib.rs:82`
- `src-rust/crates/plugins/src/lib.rs:87`
- `src-rust/crates/plugins/src/lib.rs:100`
- `src-rust/crates/plugins/src/lib.rs:241`

The global plugin and hook registries are stored in `OnceLock`s. `set_global_registry()` and `set_global_hooks()` call `.set(...)` and explicitly ignore failure if the lock was already initialized.

That means the first loaded registry wins for the lifetime of the process. A later plugin reload can compute a new `PluginRegistry`, but the global registry and global hooks used by commands/tools will stay pinned to the old state.

Impact:
- `/reload-plugins` can report success while commands/hooks still use stale plugin state
- newly added or removed plugin hooks do not actually take effect in the running process
- the runtime capability surface diverges from the reported registry contents

### 2. High: marketplace install/list paths do not match the plugin loader’s manifest conventions

Files:
- `src-rust/crates/plugins/src/loader.rs:76`
- `src-rust/crates/plugins/src/loader.rs:101`
- `src-rust/crates/plugins/src/marketplace.rs:163`
- `src-rust/crates/plugins/src/marketplace.rs:165`
- `src-rust/crates/plugins/src/marketplace.rs:228`

The loader only recognizes plugin manifests named:

- `plugin.json`
- `plugin.toml`

But the marketplace code falls back to writing:

- `manifest.yaml`

and `list_installed()` looks for:

- `manifest.yaml`
- `manifest.json`

So the marketplace and loader disagree on the on-disk plugin format. A plugin installed via the fallback manifest path can show up as "installed" to the marketplace code but still be invisible to actual plugin discovery.

Impact:
- marketplace-installed plugins may not load at runtime
- install/list UX can claim a plugin exists even though the loader ignores it
- plugin management state becomes split between two incompatible file conventions

### 3. Medium: plugin enabled/disabled state is lost on reload

Files:
- `src-rust/crates/plugins/src/loader.rs:143`
- `src-rust/crates/plugins/src/registry.rs:35`
- `src-rust/crates/plugins/src/registry.rs:95`
- `src-rust/crates/plugins/src/lib.rs:241`

Every plugin loaded from disk is created with `enabled: true`. `reload_plugins()` builds a completely new registry from disk and does not carry forward enable/disable state from the old registry.

That means a user-disabled plugin becomes enabled again after a reload unless disable state is persisted elsewhere and re-applied separately.

Impact:
- `/plugin disable` is not stable across reload
- reloading can silently reactivate commands, hooks, MCP servers, or styles the user had turned off
- runtime behavior changes unexpectedly after plugin refresh

### 4. Medium: marketplace update/install can leave stale files from previous plugin versions

Files:
- `src-rust/crates/plugins/src/marketplace.rs:137`
- `src-rust/crates/plugins/src/marketplace.rs:138`
- `src-rust/crates/plugins/src/marketplace.rs:163`
- `src-rust/crates/plugins/src/marketplace.rs:205`

`marketplace_install()` creates the install directory if needed and then extracts/copies new content into it, but it does not clean the directory first. `marketplace_update()` simply calls `marketplace_install(name).await?`.

If an older plugin version had files that are removed in the new version, those stale files remain on disk and can continue to affect discovery or command loading.

Impact:
- plugin updates can leave dead commands/assets/hooks behind
- runtime behavior may reflect a mixture of old and new plugin versions
- update results become nondeterministic depending on leftover files

## Fix Plan

### 1. Replace process-global `OnceLock` plugin state with replaceable runtime storage

Approach:
- store the global plugin and hook registries in a replaceable container such as `RwLock` or `ArcSwap`
- make reload update the active global registries atomically
- ensure commands/tools/hooks always read the latest active registry

Implementation targets:
- `src-rust/crates/plugins/src/lib.rs`

Verification:
- reload plugins after adding/removing a command or hook and confirm behavior changes immediately in the same process

### 2. Unify marketplace install formats with loader expectations

Approach:
- make marketplace installs produce the same manifest/file layout the loader expects
- standardize on one manifest convention, preferably `plugin.json` or `plugin.toml`
- update `list_installed()` to inspect the same manifest filenames the loader uses

Implementation targets:
- `src-rust/crates/plugins/src/loader.rs`
- `src-rust/crates/plugins/src/marketplace.rs`

Verification:
- install a plugin through the marketplace path and confirm it is discovered by `load_plugins()`
- ensure installed/listed/loaded plugin counts agree

### 3. Preserve enabled/disabled state across reload

Approach:
- when reloading, carry over prior enabled/disabled state by plugin name where the plugin still exists
- only default to enabled for newly discovered plugins
- consider persisting disable state if that is intended to survive process restarts as well

Implementation targets:
- `src-rust/crates/plugins/src/lib.rs`
- `src-rust/crates/plugins/src/registry.rs`

Verification:
- disable a plugin, reload, and confirm it remains disabled
- add a new plugin and confirm only the new one defaults to enabled

### 4. Make marketplace update/install replace plugin contents cleanly

Approach:
- install into a temporary directory, validate contents, then swap into place
- or remove the existing install directory before unpacking the new version
- avoid partial updates leaving mixed-version files behind

Implementation target:
- `src-rust/crates/plugins/src/marketplace.rs`

Verification:
- update a plugin where the new version deletes files from the old version
- confirm the removed files no longer exist after update

## Recommended execution order

1. make global plugin/hook state replaceable so reload is real
2. unify marketplace install formats with loader expectations
3. preserve enabled/disabled state across reload
4. make updates replace plugin contents cleanly
