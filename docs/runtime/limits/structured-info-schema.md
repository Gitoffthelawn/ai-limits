# Structured Info Schema

## Structured format

Structured data should use one common field contract for all providers and sources.

The current minimum structure is described below in a YAML-like schema. The field names, nesting, and meanings are mandatory. The final serialization format may be selected by the implementation as long as it remains machine-readable and preserves this structure.

```yaml
provider: string
source: string
source_link: docs/runtime/limits
status:
  data_available: boolean
  access_available: boolean
  message: string | null
raw_data_available: boolean
collected_at: string | null
data_as_of: string | null
account:
  plan: string | null
  credits_total: number | null
  credits_used: number | null
  credits_remaining: number | null
limits:
  - name: string
    window_label: string | null
    window_minutes: number | null
    resets_at: string | null
    used_percent: number | null
    remaining_percent: number | null
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    amount_unit: string | null
usage:
  tokens:
    input: number | null
    cached_input: number | null
    output: number | null
    reasoning_output: number | null
    cache_read: number | null
    cache_write: number | null
    total: number | null
  money:
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    currency: string | null
  activity:
    events_count: number | null
    files_count: number | null
    sessions_count: number | null
    turns_count: number | null
    latest_activity_at: string | null
  models:
    top_model: string | null
available_limit_resets: number | null
diagnostics:
  - string
```

`available_limit_resets` is the count of manually redeemable resets of provider limits. It is separate from `limits[].resets_at`, which is the automatic reset time of a rate-limit window, and from `usage`, which records consumed tokens, money, and activity.

For Codex CLI, the intended source is the interactive `/usage` view. `usage` is Codex's command and UI terminology; `available_limit_resets` is the ai-limits product field. The raw source is the rendered CLI TUI stream, not a documented local JSON object or array. Sources that do not expose a manual reset count must return `available_limit_resets: null`.

The Rust structured model serializes `available_limit_resets`. Codex CLI populates it from the read-only `/usage` view.
