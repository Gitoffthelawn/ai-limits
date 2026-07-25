# Codex

## Current status

PoC uses two Codex sources:

- `codex_cli_usage`: launches `codex`, sends `/status`, parses TUI limit lines.
- `codex_local_usage`: scans local JSONL history in `${CODEX_HOME:-~/.codex}`, aggregates token usage, and reads local rate-limit snapshots.

Codex CLI also exposes manually redeemable limit resets through `/usage`. `/usage` is Codex terminology; in ai-limits these records are part of limits, not token usage. They are separate from the automatic `resets_at` time shown for a rate-limit window.

Provider method details are documented in [codex-cli-usage.md](codex-cli-usage.md) and [codex-local-usage.md](codex-local-usage.md).

---

## Limitations

- full output remains a TUI stream and may contain terminal control sequences
- the approach depends on the current CLI behavior and TUI text
- CLI requests can take a noticeable amount of time
- needs verification of whether such requests consume user limits

---

## Other options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | Requires separate verification of usage/limits availability for a Codex subscription |
| Local telemetry files (`codex_local_usage`) | Implemented in PoC | Reads `${CODEX_HOME:-~/.codex}` JSONL history, aggregates usage, and reads local `rate_limits` snapshots (`used_percent`, `resets_at`, windows, optional credits) |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe approach to session data |
| Traffic observation | Research-only | Do not consider as a product mechanism |
