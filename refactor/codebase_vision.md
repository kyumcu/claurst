# Codebase Vision

Purpose: define the target shape of the codebase so refactoring decisions are made against a clear standard instead of one-off bug fixing.

## Vision Statement

Build a lean, local-first coding agent centered on `llama.cpp`, with safe tools, predictable runtime behavior, and a compact feature set that is fully trustworthy.

## Core Product Direction

The product should optimize for:

- first-class `llama.cpp` support
- reliable local usage
- a small but high-quality feature set
- tools that work consistently
- clear runtime and provider state
- low operational surprise

The goal is not broad feature sprawl or equal optimization for every provider.

The goal is:

- one excellent local path
- one understandable execution model
- one trustworthy basic workflow

## Primary User Promise

The user should be able to:

1. connect a local `llama.cpp` server
2. select a model cleanly
3. run a stable chat/session loop
4. use a core set of tools safely
5. trust that the UI, commands, and runtime all describe the same state

## Design Principles

### 1. Local-first

`llama.cpp` should be treated as a first-class provider, not an edge integration.

That means:

- `/connect` must work cleanly
- local model discovery must be reliable
- base URL overrides must be easy and correct
- local providers must not be penalized by remote-provider assumptions

### 2. Correctness before breadth

Do fewer things, but make them solid.

Prefer:

- fewer providers with clean support
- fewer commands with correct behavior
- fewer integrations with honest capability reporting

Over:

- broad provider matrices with alias drift
- half-working remote features
- many commands that expose inconsistent runtime state

### 3. Runtime truth over surface polish

Commands, TUI, ACP, MCP, and bridge surfaces must reflect actual runtime behavior.

The system should not:

- advertise capabilities it does not implement
- report static snapshots as live state
- present fallback behavior as real provider support

### 4. Safety at the tool boundary

The tool layer is the trust boundary between the model and the machine.

It must guarantee:

- bounded filesystem access
- session-scoped execution state
- explicit and understandable shell semantics
- consistent behavior across editing tools

### 5. Canonical provider identity everywhere

Provider alias handling should exist only at input edges.

Inside the system:

- provider IDs should be canonical
- auth lookup should be canonical
- default model logic should be canonical
- base URL resolution should be canonical
- UI and commands should not persist aliases

### 6. Explicit failure over silent corruption

The runtime should fail clearly instead of pretending success.

This applies especially to:

- stream failures
- compaction failures
- session restore mismatches
- provider resolution failures
- serialization failures

## Minimum High-Quality Feature Set

These are the features that should be excellent before expanding scope.

### 1. Provider connection

- `llama.cpp` connect flow
- endpoint override support
- model discovery
- canonical provider persistence

### 2. Session runtime

- stable streaming
- safe compaction
- reliable history ordering
- consistent session resume
- explicit error handling

### 3. Tools

Core tool set only:

- shell
- file read
- file edit
- patch apply
- search / glob

Optional tools should earn their place by reliability, not novelty.

### 4. Control surface

The following should be accurate and dependable:

- `/connect`
- `/model`
- `/providers`
- `/status`
- `/config`

## Non-Goals

These should not drive the codebase until the core is solid:

- maximizing provider count
- keeping every integration surface equally feature-rich
- adding more commands before fixing state correctness
- polishing advanced remote features before local reliability is solved
- maintaining misleading compatibility layers indefinitely

## Scope Discipline

To keep the codebase lean, every subsystem should be classified as one of:

- core
- secondary
- optional

### Core

These should receive the highest engineering attention and define the product:

- `core`
- `api`
- `query`
- `tools`
- `tui`
- `commands`
- `cli`

### Secondary

These may remain, but they should not complicate the core path and should be reduced to honest, maintainable scope:

- `mcp`
- `plugins`

### Optional

These should be treated as removable, disableable, or explicitly out of the main quality target unless they are strategically required:

- `bridge`
- `acp`
- `buddy`

## Leaning Strategy

The codebase should be made leaner by:

- removing duplicate provider logic in favor of one canonical provider system
- shrinking partially implemented integration surfaces to the features they truly support
- treating remote-control and peripheral features as optional until the local core is solid
- preferring removal or disablement over carrying misleading half-implemented behavior

## Keep / Shrink / Defer Guidance

### Keep and strengthen

- local `llama.cpp` flow
- session runtime correctness
- bounded core tools
- canonical provider handling
- the small control surface around `/connect`, `/model`, `/providers`, `/status`, and `/config`

### Shrink aggressively

- provider alias branches spread across multiple crates
- plugin reload and marketplace complexity unless it is made trustworthy
- MCP breadth beyond the transport and routing modes actually used
- command and reporting surfaces that claim more than they can verify live

### Defer or make optional

- remote bridge sophistication
- ACP session workflows unless they are made real
- peripheral subsystems like `buddy`
- feature breadth for many remote providers before `llama.cpp` is excellent

## Target Architecture

The clean architecture target is:

- `core`
  provider identity, config, auth, base URL resolution, defaults
- `api`
  provider adapters, transport, model listing, runtime provider construction
- `query`
  the single trustworthy turn/runtime engine
- `tools`
  bounded machine capabilities
- `tui`
  user-facing state and interaction over runtime truth
- `commands`
  thin control layer over the same canonical runtime behavior

Everything else should be secondary to this path:

- `mcp`
- `plugins`
- `bridge`
- `acp`
- `buddy`

## Refactor Priorities Implied By This Vision

### Priority 1: runtime and tool safety

- prevent conversation loss
- prevent fake successful turns
- enforce filesystem boundaries
- fix bridge event loss and reconnect behavior

### Priority 2: provider coherence

- canonical provider IDs across all layers
- correct `api_base` resolution
- synchronized provider/model state
- reliable `llama.cpp` local flow

### Priority 3: truthful interfaces

- ACP should only claim what it actually supports
- commands should report live runtime state
- MCP transport claims should match implementation
- plugin reload should change actual runtime behavior

### Priority 4: lower-risk cleanup

- metadata quality
- stale label cleanup
- peripheral persistence hardening

### Priority 5: reduce unnecessary surface area

- simplify or disable optional modules that are not part of the local-first core
- remove misleading compatibility and reporting layers
- cut breadth that does not raise the quality of the basic workflow

## Definition Of Done For “Lean And Solid”

The codebase is meaningfully aligned with this vision when:

1. `llama.cpp` works end to end through `/connect`, `/model`, and live session usage.
2. No core runtime path can silently destroy or corrupt conversation state.
3. No tool can mutate files outside approved roots.
4. Provider identity is canonical across config, auth, UI, runtime, and commands.
5. Commands and external interfaces report actual runtime truth.
6. The supported feature set is smaller, clearer, and more reliable than before.

## Practical Decision Rule

When deciding whether to keep, refactor, or remove something, ask:

Does this make the local `llama.cpp` workflow safer, clearer, or more reliable?

If not, it is probably not a priority.
