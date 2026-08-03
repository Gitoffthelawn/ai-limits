# Tauri UI Provider Block Content

## Limit Rows

Each limit row is rendered as a vertical group:

1. Top text line above the bar: `{window} | {remaining}% left`, for example `5h | 59.0% left`.
2. Full-width remaining bar.
3. Reset text line below the bar: `reset {time}`, for example `reset Jul 6, 01:49`.

The limit type, such as `5h`, `7d`, `plan`, `auto`, or `api`, must not consume a separate left column. This lets every bar use 100% of the provider block content width.

The remaining-limit value follows the shared [limit display rules](../../presentation/limit-display.md).

The remaining bar shows:

- filled segment width equal to remaining percentage
- unfilled spent segment in white or another very light neutral color
- one solid color for the whole filled segment

The filled segment color is calculated from remaining percentage:

- `100%` is green
- `50%` is yellow
- `1%` is red
- intermediate values are interpolated between these anchors

The bar must not use a left-to-right rainbow gradient inside the filled segment. For example, if `10%` remains, the filled 10% segment is a near-red color and the spent 90% segment stays light.

## Available Credits Line

When the provider has remaining credits, show one text line directly below the limit rows:

```text
Available credits: 344.2
```

The line is hidden when credits are unavailable.

---

## Manual Limit Resets

When `availableLimitResets` is greater than zero, show an informational line after credits and before the source line. It uses the same visual style as the available credits line:

```text
Available resets: 1
```

This line shows availability only; it must not contain a control that redeems a reset.

---

## Plan Section

The Plan section shows the source's `account` subscription fields, defined in [get-limits/structured-info-schema.md](../../get-limits/structured-info-schema.md). It is the desktop rendering of the **Plan** output kind from [product/output-kinds.md](../../product/output-kinds.md).

When present, shown as plain text lines, one fact per line or combined onto one line where it reads naturally:

```text
Plan: Plus
Started Jan 12, 2026 · renews Aug 8, 2026
```

- `plan` renders as `Plan: {plan}`.
- `subscription_started_at` and `renewal_at` render together as `Started {date} · renews {date}` when both are present; either one alone still renders on its own line.
- Subscription dates render in the date-only form documented in [presentation/time-display.md](../../presentation/time-display.md). They always carry the year and never a time of day: a subscription can have started years ago, and an annual plan can renew up to a year out.
- `price_amount` and `price_currency` render as a formatted price line. When `price_note` is present, it is appended in parentheses on the same line, for example `$20.00 (may vary by country/currency)`. No billing period is appended, because no schema field carries one.
- `plan_management_url` and `billing_management_url`, when present, render as external-style links (see [styleguide.md](styleguide.md)) below the price line, using generic labels such as "Manage plan" and "Manage billing".
- Any field that is `null` contributes no line; the section is omitted entirely when every subscription field is `null`.

Source coverage is uneven and the section is built to degrade. No source currently exposes `price_amount`, `price_currency`, `price_note`, `plan_management_url`, or `billing_management_url`, so the price line and the management links are specified but not rendered for any provider today. Of the sources that do report subscription data, `codex_local` supplies plan name plus both dates, and `cursor_api2` supplies a renewal date only.

## Usage Section

The Usage section shows the source's `usage` fields, defined in [get-limits/structured-info-schema.md](../../get-limits/structured-info-schema.md). It is the desktop rendering of the **Usage** output kind from [product/output-kinds.md](../../product/output-kinds.md).

Usage shape varies sharply between sources: one reports tokens, another reports money, another reports session/turn counts and a top model. The section does not use a fixed layout. It renders one short human-readable line per non-null `usage.*` group that has at least one non-null field, and omits groups that are entirely `null`:

```text
Tokens: 711.2M total
Sessions: 92 · Turns: 6,394
Top model: claude-sonnet-5
```

- `usage.tokens` renders as a token summary line, leading with `total` when available, otherwise composing from whichever of `input`/`output`/`cached_input`/`cache_read`/`cache_write`/`reasoning_output` are present.
- `usage.money` renders as a spend line, for example `Spend this period: {used_amount} {currency}`, falling back to `total_amount` when `used_amount` is absent.
- `usage.activity` renders as one line combining whichever of `events_count`, `files_count`, `sessions_count`, `turns_count` are present; `latest_activity_at` is not repeated here since it already backs the source timestamp when applicable (see [structured-info-rules.md](../../get-limits/structured-info-rules.md)).
- `usage.models` renders as `Top model: {top_model}`.
- Large integer counters are formatted with a compact human suffix (`1.5B`, `22,545`) consistent with the rest of the provider block; exact formatting follows the shared number-formatting conventions used elsewhere in the frontend.
- The section is omitted entirely when every `usage.*` group is entirely `null`.

---

## Source Line

Provider source information is shown on one line by default:

```text
Local files, as of Jul 5, 22:12
```

The line has two parts:

- origin label, for example `Local files`, `CLI`, or `API2`
- timestamp from `dataTimestamp`, rendered as `as of {timestamp}`

Possible origin labels: `Local files`, `CLI`, `API2`, `Unknown`.

Each part is a non-breaking unit: `{origin label},` and `as of {timestamp}` must not wrap in the middle. If the provider block is too narrow for the full line, the line may break only between these two units.
