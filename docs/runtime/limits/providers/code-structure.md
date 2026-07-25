# Providers Code Structure

## Code

Initially, `providers/` remains a flat directory.

Example:

```text
providers/
  mod.rs
  codex_cli_usage.rs
  claude_cli_usage.rs
  cursor_api2_usage.rs
```

Rules:

- one file describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`

If a single provider grows to many files, you can move to a nested structure by provider.

## Documentation

Provider documentation is grouped by provider:

```text
docs/runtime/limits/providers/
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
