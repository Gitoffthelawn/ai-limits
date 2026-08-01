# Providers Code Structure

## Code

Current provider methods are nested packages under `providers/`. A new method may start as one file and nest later when a single file becomes hard to reason about. Nested packages keep a thin public facade in `mod.rs` and private internals split by responsibility (I/O, parsing, projection).

Current layout:

```text
providers/
  mod.rs
  claude_cli/
    mod.rs
    capture.rs
    parse.rs
    project.rs
  claude_local/
    mod.rs
    io.rs
    parse.rs
    model.rs
    project.rs
  codex_cli/
    mod.rs
    process.rs
    parse.rs
    project.rs
  codex_local/
    mod.rs
    raw.rs
    scan.rs
    parse.rs
    project.rs
  cursor_api2/
    mod.rs
    fetch.rs
    parse.rs
    helpers.rs
    project.rs
```

Rules:

- one package or file describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- the package root keeps the public facade; internals stay private
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`

`claude_cli/` layout:

- `mod.rs` — public facade (`get_usage` / `collect_usage`) and source identity constants
- `capture.rs` — expect script and process capture (`capture_provider_run`)
- `parse.rs` — TUI line normalization (including CR/LF) and parsing into an internal model
- `project.rs` — projection to `SourceData` / `StructuredSourceInfo`

`claude_local/` layout:

- `mod.rs` — thin `collect()` orchestration
- `io.rs` — transcript root discovery and recursive JSONL scan
- `parse.rs` — turn usage and server reset anchors from JSON
- `model.rs` — usage accumulation and 5h session-limit math
- `project.rs` — raw JSON and structured `SourceData` projection

`codex_cli/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_structured`
- `process.rs` — PTY expect script and `codex login status` checks
- `parse.rs` — TUI line normalization and limit/credits/reset parsing
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

`codex_local/` layout:

- `mod.rs` — public facade (`get_usage` / `collect`) and package tests
- `raw.rs` — raw DTO and internal accumulation model
- `scan.rs` — Codex home resolution and JSONL directory/file scan
- `parse.rs` — token-event / rate-limit / credits parsing
- `project.rs` — structured projection and raw encode/decode

`cursor_api2/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_source_data`
- `fetch.rs` — Keychain token and HTTP request via `infra/os_access`
- `parse.rs` — scrape helpers and internal `CursorApiFields` model
- `helpers.rs` — private date, amount, and billing helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

## Documentation

Provider documentation is grouped by provider; large providers split method details into separate files:

```text
docs/get-limits/providers/
  code-structure.md
  contract.md
  claude.md
  claude-cli-usage.md
  claude-local-usage.md
  codex.md
  codex-cli-usage.md
  codex-local-usage.md
  cursor.md
  cursor-options.md
```

Rules:

- one top-level spec file describes one provider
- a spec file may describe multiple provider methods
- provider method docs name the method like the source id (for example `claude_cli_usage`)
- code may be more detailed than the documentation and split a method into private modules
- if a spec file becomes too large, it can be split by provider method
