# Claude

## Current Status

PoC uses two Claude sources:

- `claude_cli_usage`: launches `claude --no-chrome`, sends `/usage`, parses TUI lines.
- `claude_local_usage`: scans local transcript JSONL files and aggregates token usage history.

Provider method details are documented in [claude-cli-usage.md](claude-cli-usage.md) and [claude-local-usage.md](claude-local-usage.md).

---

## Limitations

- for `claude_cli_usage`, full output remains a TUI stream and depends on current CLI text/layout
- for `claude_cli_usage`, request/parse can take noticeable time
- for `claude_local_usage`, reset remains an estimate unless a server reset anchor is available
- for Claude Desktop and browser-extension flows, local-file coverage remains unverified separately from Claude Code behavior

---

## Other Options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | May apply to API accounts, but not necessarily to Claude Code subscription limits |
| Local transcript JSONL (`claude_local_usage`) | Implemented in PoC | Scans local transcript roots and aggregates token usage history by assistant turns; official remaining limit/reset may be unavailable |
| Local SQLite/cache | Auxiliary layer | e.g. `~/.claude/usage.db` from `claude-usage`: convenient for dashboard and incremental scanning, but this is derived data, not a primary source |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe way to handle cookie/session tokens |
| Traffic observation | Research-only | Not to be considered as a product mechanism |
