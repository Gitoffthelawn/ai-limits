# Architecture

This document defines the target structure of `src/` after moving from a PoC monolith to a maintainable application.

---

## Goal

The code supports:

- a primary desktop interface
- a stateless CLI for automation and diagnostics
- multiple providers
- multiple ways to fetch data for a single provider
- small files with a clear area of responsibility

The desktop and CLI share a common core, not separate business logic.

---

## `src/` Structure

Target structure for the near term:

```text
src/
  cli/
  infra/
  notifications/
  presentation/
  providers/
  get_limits.rs
  lib.rs
  types.rs
```

Purpose:

- `cli/` — terminal interface, arguments, retrieval scenario flags, output, exit codes
- `infra/` — technical primitives for processes, HTTP, and timeouts
- `notifications/` — shared notification service with platform adapters
- `presentation/` — user-facing display models built from structured data
- `providers/` — ways to fetch usage/limits from providers
- `get_limits.rs` — limits-fetching scenario and provider method integration
- `lib.rs` — shared core available to different interfaces
- `types.rs` — shared types and the application's internal language

---

## Boundaries

Module rules:

- `cli/` does not fetch data from providers directly
- `cli/` calls the shared core and is responsible only for terminal behavior
- `cli/` parses `--best`/`-b` and passes the selected retrieval scenario to the shared core
- `cli/` renders presentation output but does not decide user-facing limit semantics
- `presentation/` converts structured data into user-facing provider blocks
- `presentation/` selects display labels, limit rows, bar values, and fallback messages
- `presentation/` renders the selected source report and does not decide fallback order
- `presentation/` does not fetch data, read files, run commands, or call provider methods
- `get_limits.rs` coordinates provider method selection and fallback logic
- `get_limits.rs` owns provider fallback chains for default and best-source runs
- `get_limits.rs` owns desktop source priority chains for Fast, Full, and Best modes
- `get_limits.rs` does not run processes or HTTP directly when that can be delegated to provider/infra
- `get_limits.rs` does not format terminal output
- `providers/` does not format terminal output
- `providers/` returns normalized types from `types.rs`
- `providers/` follows [runtime/limits/providers/contract.md](runtime/limits/providers/contract.md)
- `infra/` does not know the business meaning of usage/limits
- `infra/` is responsible only for technical interaction with the outside world
- `types.rs` must not depend on CLI, desktop, the file system, or external commands

Provider code and spec-doc structure is documented in [runtime/limits/providers/code-structure.md](runtime/limits/providers/code-structure.md). Desktop-specific architecture (settings, Tauri) is documented in [desktop/architecture.md](desktop/architecture.md).

---

The main runtime flow across `get_limits.rs` and `presentation/` is documented in [runtime/get-limits-scenario.md](runtime/get-limits-scenario.md) and [runtime/presentation.md](runtime/presentation.md).

---

## Rule for Agents

When making changes, first identify the business area of the task:

- terminal behavior — `cli/`
- desktop settings — Tauri frontend state
- data fetching — `providers/`
- presentation — `presentation/`
- limits-fetching scenario — `get_limits.rs`
- process execution, HTTP, timeouts — `infra/`
- shared data structures — `types.rs`

If a task spans more than one area, describe the overlap explicitly before making changes.
