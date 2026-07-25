# Presentation

`presentation/` is responsible for the default user-facing output model.

It receives structured data from the shared core and prepares provider blocks for the CLI. The default terminal presentation is documented in [../terminal/interface.md](../terminal/interface.md).

Responsibilities:

- group source reports into provider blocks;
- choose user-facing provider labels;
- convert limits into fixed-width rows;
- build 25-character remaining-limit bars;
- choose `Source {source}` text from structured `source` and `data_as_of`;
- prepare unavailable or no-data messages from structured status data.
- render the selected source report; fallback order is decided before presentation;
- format the raw and structured output modes (`--raw`, `--structured`) from a source report.

Boundaries:

- does not call providers;
- does not parse raw source data;
- does not draw terminal frames or loaders.
