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

## 5-Hour Limit Reconstruction

`claude_local_usage` also reconstructs a single active 5-hour limit record from local transcripts:

- the numerator is `input_tokens + output_tokens` summed over turns in the active window
- the denominator is a fixed local estimate of `88,000` tokens for the Max5 plan; this is a community-derived approximation, not a value read from an official Claude API
- if a server reset anchor was found in local data (a reset timestamp nested under a rate-limit, usage-limit, quota, or 429 payload) and it is in the future, the window is `[anchor - 5h, anchor)` and the reset source is reported as `server reset anchor`
- otherwise the window is reconstructed from transcript timing: a new window starts at the first turn after the previous window elapsed or after a gap of 5 hours or more since the last turn, and the reset source is reported as `estimated reset`
- the resulting limit record includes `used_percent`, `remaining_percent`, `used_amount`, `remaining_amount`, `total_amount` (tokens), and `resets_at`
- because the `88,000` denominator is an approximation, reported usage can diverge from the account's actual 5-hour limit, especially at high usage
