# Terminal Provider Block Format

### Provider Block

Default limits output prints each provider as a separate block.

Block header:

```text
     --------- CODEX ---------
     --------- CLAUDE --------
     --------- CURSOR --------
```

Provider headers are 25 visible characters wide and indented by 5 spaces, matching the start column of the limit bar. An empty line is printed before each provider header except the first header after the top frame. No empty line is printed between a provider header and its body.

Each provider block contains:

- zero or more limit rows;
- credits or balance, when available;
- manual limit-reset rows, when available;
- `Source {source}`, using structured `source` and `data_as_of`.

Limit row format:

Format:

```text
{window:<4} {bar:<25} {left:>11} | reset {reset_at}
```

Example:

```text
     --------- CODEX ---------
5h   ■■□□□□□□□□□□□□□□□□□□□□□□□  8.0% left | reset Jun 30, 21:41
7d   ■■■■■■■■■■■■■■□□□□□□□□□ 54.0% left | reset Jul 3, 21:41
Credits: 344.2
Source codex-cli: Jul 3, 21:41
```

The bar width is `25` characters. Each filled bar character `■` represents `4%`. Remaining limit is rounded to the nearest number of filled characters. Empty bar characters use `□`. ANSI color codes in the bar do not affect column width.

Limit rows use fixed visible column widths: `{window}` is 4 characters, `{bar}` is 25 characters, and `{left}` is 11 characters right-aligned. The ` | reset ` separator starts at the same column on every row.

The `{left}` percentage label is always shown with one decimal place (`8.0% left`, `54.0% left`, `62.5% left`). Structured source data may keep finer precision; presentation normalizes the displayed value and uses the same normalized value for the bar and color thresholds.

User-facing timestamps follow the shared rules in [time-display.md](../runtime/time-display.md): local system timezone, `HH:MM` for today, `MMM D, HH:MM` for another date, and no timezone suffix. Terminal rows keep their own contextual labels, for example `reset {time}` for limit reset time and `Source {source}: {time}` for provider data time. If a source timestamp cannot be parsed reliably, presentation keeps the original source text.

The filled bar characters show available remaining limit, not used limit. The whole filled part uses one color based on remaining limit. The empty bar characters are not colored.

If `data_as_of` is unavailable, print:

```text
Source codex-cli: unknown
```

If the source is unavailable, print the provider block with the status message:

```text
     --------- CLAUDE --------
Unavailable: not logged in
Source claude-cli: unknown
```

If the source is available but has no supported limit data, print the provider block with a short reason:

```text
     --------- CODEX ---------
No usable limit records from this source
Other sources may still provide limit data.
Source codex-cli: Jul 3, 21:41
```

### Manual Limit Resets

When `available_limit_resets` is available, print it after the credits line and before `Source`:

```text
Resets:  1
```

Do not show this section when `available_limit_resets` is `null` or zero. This row is informational only: the terminal interface must not offer reset redemption.
