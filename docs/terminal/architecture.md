# Terminal Architecture

## Parallel Model

Selected sources are started in parallel.

Execution model:

```text
provider worker threads
        ↓
channel events
        ↓
cli event loop
        ↓
terminal renderer
```

If multiple sources are waiting at the same time, multiple loader lines are displayed.

Format:

```text
⠋ waiting codex-cli
⠙ waiting claude-cli
```

When a source finishes, its loader is cleared and the result is printed as soon as it is ready.

---

## Architectural Boundaries

Layout:

```text
src/cli/mod.rs
  - parses arguments
  - starts provider worker threads
  - receives events via channel
  - passes state to terminal renderer
  - formats and prints provider blocks from structured source reports

src/infra/loader.rs
  - selects unicode/ascii spinner
  - draws loader lines
  - clears loader lines
  - prints frames and headers

src/get_limits.rs
  - calls provider methods
  - selects fallback chains for default and best-source runs
  - returns normalized SourceReport

src/providers/*
  - fetches source data
  - returns raw and structured data
  - does not render terminal UI
```
