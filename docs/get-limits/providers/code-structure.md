# Providers Code Structure

## Code

Provider methods start as one file each. When a method grows, nest it as a package with a thin facade and private internals split by responsibility (I/O, parsing, projection).

Example:

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
- `capture.rs` — expect script and process capture (`run_provider`)
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
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

If a single provider grows to many files, you can move to a nested structure by provider. The nested package keeps a thin public facade in `mod.rs`; internal modules stay private to that provider.

## Documentation

Provider documentation is grouped by provider:

```text
docs/get-limits/providers/
  codex.md
  claude.md
  cursor.md
```

Rules:

- one spec file describes one provider
- a spec file may describe multiple provider methods
- provider method sections are named like future code files without `.rs`
- code may be more detailed than the documentation and split provider methods into separate files
- if a spec file becomes too large, it can be split by provider method
