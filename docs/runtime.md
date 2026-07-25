# Runtime

This document describes the main runtime flow shared by the desktop and CLI interfaces: how a limits query moves from provider methods to the user-facing result.

---

## `get_limits` Scenario

`get_limits.rs` follows the document [runtime/limits/methods/overview.md](runtime/limits/methods/overview.md).

Purpose:

- select enabled provider methods
- call provider methods in the right order
- apply provider fallback-chain logic for default and best-source runs
- apply desktop source priority logic for Fast, Full, and Best modes
- assemble a shared result for the desktop and CLI

Boundaries:

- does not contain terminal output
- does not contain low-level process execution
- does not contain low-level HTTP primitives
- does not parse provider-specific output when that is a provider method's responsibility

---

## Presentation

`presentation/` is responsible for the default user-facing output model.

It receives structured data from the shared core and prepares provider blocks for the CLI. The default terminal presentation is documented in [terminal/interface.md](terminal/interface.md).

Responsibilities:

- group source reports into provider blocks;
- choose user-facing provider labels;
- convert limits into fixed-width rows;
- build 25-character remaining-limit bars;
- choose `Source {source}` text from structured `source` and `data_as_of`;
- prepare unavailable or no-data messages from structured status data.
- render the selected source report; fallback order is decided before presentation.

Boundaries:

- does not call providers;
- does not parse raw source data;
- does not own raw or structured serialization;
- does not draw terminal frames or loaders.
