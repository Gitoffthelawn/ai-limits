# Providers Code Structure

## Code

Provider methods start as one file each. When a method grows, nest it as a package with a thin facade and private internals split by responsibility (I/O, parsing, projection).

Example:

```text
providers/
  mod.rs
  claude_cli.rs
  claude_local.rs
  codex_cli/
    mod.rs      # facade: collect_usage orchestration
    process.rs  # CLI I/O: expect script, login status
    parse.rs    # line/percent/reset/credits parsing
    project.rs  # StructuredSourceInfo / SourceData projection
  codex_local.rs
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

`codex_cli/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_structured`
- `process.rs` — PTY expect script and `codex login status` checks
- `parse.rs` — TUI line normalization and limit/credits/reset parsing
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

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
