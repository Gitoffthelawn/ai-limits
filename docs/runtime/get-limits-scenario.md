# Get Limits Scenario

`get_limits.rs` follows the document [limits/methods/overview.md](limits/methods/overview.md).

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
