# Providers Code Structure

## Code

Provider methods start as one file each under `providers/`. A method may grow into a nested package when a single file becomes hard to reason about; other methods stay flat until they need the same treatment.

Example:

```text
providers/
  mod.rs
  claude_cli.rs
  claude_local.rs
  codex_cli.rs
  codex_local/
    mod.rs
    raw.rs
    scan.rs
    parse.rs
    project.rs
  cursor_api2.rs
```

`codex_local/` layout:

- `mod.rs` — public facade (`get_usage` / `collect`) and package tests
- `raw.rs` — raw DTO and internal accumulation model
- `scan.rs` — Codex home resolution and JSONL directory/file scan
- `parse.rs` — token-event / rate-limit / credits parsing
- `project.rs` — structured projection and raw encode/decode

Rules:

- one top-level module describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`
- nested packages keep a thin public facade; internals stay private to the package

If a single provider grows to many files, you can move to a nested structure by provider.

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
