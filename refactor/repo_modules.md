# Repository Modules

This repository's critical infrastructure is centered in the Rust workspace under `src-rust`. The main modules are declared in `src-rust/Cargo.toml`, and the architecture summary is documented in `spec/13_rust_codebase.md`.

## Dependency Graph

```text
claurst (CLI binary)
  -> claurst-core
  -> claurst-api
  -> claurst-acp
  -> claurst-tools
  -> claurst-query
  -> claurst-tui
  -> claurst-commands
  -> claurst-mcp
  -> claurst-bridge
  -> claurst-plugins

claurst-commands
  -> claurst-core
  -> claurst-api
  -> claurst-tools
  -> claurst-query
  -> claurst-mcp
  -> claurst-tui
  -> claurst-plugins
  -> claurst-bridge

claurst-tui
  -> claurst-core
  -> claurst-api
  -> claurst-tools
  -> claurst-query
  -> claurst-mcp

claurst-query
  -> claurst-core
  -> claurst-api
  -> claurst-plugins
  -> claurst-tools

claurst-tools
  -> claurst-core
  -> claurst-api
  -> claurst-mcp

claurst-bridge
  -> claurst-core
  -> claurst-api
  -> claurst-query

claurst-acp
  -> claurst-core
  -> claurst-api

claurst-api
  -> claurst-core

claurst-mcp
  -> claurst-core

claurst-plugins
  -> claurst-core

claurst-buddy
  -> claurst-core
```

## Critical Infrastructure

- `src-rust/crates/cli`
  Binary entrypoint. Starts the app, parses CLI flags, wires config, TUI, query loop, auth, bridge, MCP, and plugins. Main executable is `src-rust/crates/cli/src/main.rs`.

- `src-rust/crates/core`
  Foundation crate used by everything else. This is the most important infrastructure layer.

  Key responsibilities:
  - config and persisted settings
  - auth storage and provider IDs
  - session/history storage
  - system prompt assembly
  - feature flags/gates
  - analytics, snapshots, migrations, remote settings
  - skills, memdir, token budget, voice, IDE/LSP helpers

  Representative files:
  - `src-rust/crates/core/src/lib.rs`
  - `src-rust/crates/core/src/auth_store.rs`
  - `src-rust/crates/core/src/provider_id.rs`
  - `src-rust/crates/core/src/system_prompt.rs`
  - `src-rust/crates/core/src/sqlite_storage.rs`

- `src-rust/crates/api`
  Model/provider abstraction layer. Handles API requests, streaming, model registry, provider registry, provider-specific adapters, request transforms, and error handling.

  This is the second major infrastructure layer.

  Representative files:
  - `src-rust/crates/api/src/lib.rs`
  - `src-rust/crates/api/src/registry.rs`
  - `src-rust/crates/api/src/provider.rs`
  - `src-rust/crates/api/src/model_registry.rs`
  - `src-rust/crates/api/src/providers/openai_compat.rs`

- `src-rust/crates/query`
  Agentic execution engine. Orchestrates turns, tool calls, compaction, coordinator behavior, cron tasks, session memory, away summaries, and command queueing.

  This is the runtime brain of the application.

  Representative files:
  - `src-rust/crates/query/src/lib.rs`
  - `src-rust/crates/query/src/coordinator.rs`
  - `src-rust/crates/query/src/compact.rs`
  - `src-rust/crates/query/src/cron_scheduler.rs`
  - `src-rust/crates/query/src/session_memory.rs`

- `src-rust/crates/tools`
  Tool runtime and concrete tool implementations. This is the main capability surface exposed to the model.

  Includes:
  - shell execution
  - file read/write/edit
  - grep/glob
  - MCP resource access
  - web fetch/search
  - tasks/sub-agents
  - cron/worktree/config/plan-mode tools

  Representative files:
  - `src-rust/crates/tools/src/lib.rs`
  - `src-rust/crates/tools/src/bash.rs`
  - `src-rust/crates/tools/src/file_edit.rs`
  - `src-rust/crates/tools/src/mcp_resources.rs`
  - `src-rust/crates/tools/src/web_search.rs`

## User-Facing Functionality

- `src-rust/crates/tui`
  Terminal UI built on `ratatui`. Manages app state, dialogs, model/provider pickers, transcript rendering, onboarding, MCP views, settings screens, voice UI, tasks overlay, and bridge state.

  Core files:
  - `src-rust/crates/tui/src/app.rs`
  - `src-rust/crates/tui/src/render.rs`
  - `src-rust/crates/tui/src/model_picker.rs`
  - `src-rust/crates/tui/src/prompt_input.rs`

- `src-rust/crates/commands`
  Slash commands and named commands layer. Handles `/connect`, `/model`, `/mcp`, `/status`, `/compact`, `/agent`, and other command workflows.

  Files:
  - `src-rust/crates/commands/src/lib.rs`
  - `src-rust/crates/commands/src/named_commands.rs`

## Integration / Platform Modules

- `src-rust/crates/mcp`
  MCP client infrastructure. Server registry, connection manager, OAuth handling, and server status tracking.

  Files:
  - `src-rust/crates/mcp/src/lib.rs`
  - `src-rust/crates/mcp/src/connection_manager.rs`
  - `src-rust/crates/mcp/src/registry.rs`

- `src-rust/crates/plugins`
  Plugin runtime. Plugin manifests, loading, registry, marketplace, and hooks integration.

  Files:
  - `src-rust/crates/plugins/src/lib.rs`
  - `src-rust/crates/plugins/src/loader.rs`
  - `src-rust/crates/plugins/src/marketplace.rs`

- `src-rust/crates/bridge`
  Bridge to remote/web UI and session-sharing infrastructure. Depends on `core`, `api`, and `query`.

  Main file:
  - `src-rust/crates/bridge/src/lib.rs`

- `src-rust/crates/acp`
  ACP server implementation: JSON-RPC 2.0 over stdio for agent/client integration.

  Main file:
  - `src-rust/crates/acp/src/lib.rs`

## Secondary / Specialized Modules

- `src-rust/crates/buddy`
  Separate buddy/tamagotchi-style subsystem. Peripheral, not core execution infrastructure.

## Practical Module Hierarchy

1. `core`: shared types, config, auth, persistence, prompts, feature gates
2. `api`: provider/model plumbing and streaming
3. `query`: turn orchestration and agent runtime
4. `tools`: executable capabilities used by the model
5. `tui`: user interface and interaction state
6. `commands`: command workflows layered on top of TUI/query/core
7. `mcp`, `plugins`, `bridge`, `acp`: integration infrastructure
8. `cli`: executable composition root
9. `buddy`: optional peripheral subsystem

## Short Architectural Summary

The critical path is:

`cli -> tui/commands -> query -> tools/api -> core`

The main integration subsystems hanging off that path are:

`mcp`, `plugins`, `bridge`, `acp`
