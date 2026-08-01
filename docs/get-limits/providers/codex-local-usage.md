# Codex Local Usage

## Provider Method: `codex_local_usage`

Code: `src/providers/codex_local/` (`mod` facade, `raw`, `scan`, `parse`, `project`).

Minimal source:

- root: `${CODEX_HOME:-~/.codex}`
- scanned directories: `sessions/`, `archived_sessions/`
- scanned files: `**/*.jsonl`

What is extracted:

- events with `"type":"token_count"` and `"last_token_usage"`
- totals: input, cached input, output, reasoning output, total
- latest activity timestamp (ISO 8601 UTC from the latest `token_count` event with `rate_limits`)
- `rate_limits` snapshot when present: `primary.used_percent`, `primary.window_minutes`, `primary.resets_at`, `secondary.used_percent`, `secondary.window_minutes`, `secondary.resets_at`, `credits` (`balance` or scalar), `plan_type`

How to get these fields from local files:

1. read `${CODEX_HOME:-~/.codex}/sessions/**/*.jsonl` and `${CODEX_HOME:-~/.codex}/archived_sessions/**/*.jsonl`
2. keep only records where `type = "event_msg"` and `payload.type = "token_count"`
3. for usage, aggregate `payload.info.last_token_usage.*`
4. for limits/reset, read `payload.rate_limits.*` from the latest timestamped event that includes `rate_limits`
5. show `Latest activity` and `resets_at` as ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`)

Behavior:

- if root is missing, returns `not found`
- if no token events are found, returns `token events: not found`
- local Codex JSONL can provide current local snapshot for limit percent and reset time (5h/weekly windows when present)
- local Codex JSONL usually does not provide absolute quota size (`used_tokens`/`max_tokens`), only percent and reset window
- local Codex JSONL and the local Codex state database did not expose manual reset count, type, or expiry during verification; they must not be used as a source for `available_limit_resets`
