# Claude Local Usage

## Provider Method: `claude_local_usage`

Minimal sources:

- `~/.config/claude/projects`
- `~/.claude/projects`
- `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects`

What is extracted:

- `assistant` records with non-zero `message.usage`
- deduplicated turns by `message.id` (latest record wins in file)
- latest server reset anchor found in local JSONL records when a reset timestamp appears inside rate-limit, usage-limit, quota, or 429 payload context
- scope summary: files, sessions, turns
- token totals: input/output/cache-read/cache-write/total
- top model and latest activity timestamp

Behavior:

- if no local roots are present, returns `local transcript roots were not found`
- if roots exist but no token usage is found, returns `no token usage found`
- local transcripts provide usage history; official remaining limit/reset may be unavailable

Detailed reconstruction research notes are documented in [claude-local-usage-research.md](claude-local-usage-research.md).
