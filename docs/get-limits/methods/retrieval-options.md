# Data Retrieval Options

## Data retrieval options

| Option | Status | Pros | Cons | Providers |
|---|---|---|---|---|
| Official API | Preferred when available | Stability, clear support, low risk | Often requires a key, enabling the API, or an Enterprise plan | Cursor Enterprise; potentially Codex/Claude/API providers |
| Local transcript/telemetry files | Implemented in PoC | Fast, local, no network requests and no quota consumption | Usually gives usage history, but not always the official remaining limit/reset | Claude (`~/.config/claude/projects`, `~/.claude/projects`, Xcode ClaudeAgentConfig), Codex (`~/.codex`) |
| Local derived DB/cache | Auxiliary layer | Speeds up the dashboard, incremental recalculation, and history after upstream cleanup | Not the primary source of truth; must stay in sync with source files | Claude (`~/.claude/usage.db`, tool-specific caches) |
| Provider CLI | Used in PoC | Works in a minimal user scenario using an already configured CLI | Slow, fragile TUI parsing, requests may consume resources | Codex, Claude |
| Local token/client backend | Used for Cursor | Can work without a separate API key, using an existing login | Cursor's stable internal endpoint has no publicly documented contract and needs careful security review | Cursor |
| Frontend/dashboard API via cookie | Research-only | Often exposes the same data as the web UI | Cookie is a sensitive secret, high security and ToS risk | Potentially Codex, Claude, Cursor |
| Traffic observation | Research-only | Can help understand internal contracts | HTTPS, certificate pinning, high risk of fragility and misuse | Potentially all |

## Current status by provider

| Provider | Primary known option | Status | Documents |
|---|---|---|---|
| Codex | Local JSONL history and `rate_limits` snapshots from `${CODEX_HOME:-~/.codex}` (`codex_local`) | Default implemented source. Provides token usage and current local limit/reset snapshots when present; CLI `/status` (`codex_cli`) remains a broader fallback for `--best`. | [../providers/codex.md](../providers/codex.md) |
| Claude | Local transcript reconstruction (`claude_local`) | Default implemented source. Local transcripts provide usage/history and estimated limits; CLI `/usage` (`claude_cli`) remains a broader fallback for `--best`. | [../providers/claude.md](../providers/claude.md) |
| Cursor | `api2.cursor.sh` `GetCurrentPeriodUsage` via Cursor Agent token (`cursor_api2`) | Only implemented source. Uses Cursor's stable internal endpoint, but its contract is not publicly documented; there is no CLI-based fallback source in code. | [../providers/cursor.md](../providers/cursor.md) |
