# Tauri UI Provider Block Content

Every section opens with a labelled divider when it has content, as documented in [provider-blocks.md](provider-blocks.md). A value that is absent contributes no line. A section with no values at all has no heading and no lines; when its display toggle is on, the card still reserves an empty equalized slot — see [section slot alignment](provider-blocks.md#section-slot-alignment). Nothing is ever rendered as an empty value, a dash, or a placeholder.

---

## Limits Section

Heading: `LIMITS`.

### Limit Rows

Each limit row is rendered as a vertical group:

1. Top text line above the bar: `{window} | {remaining}% left`, for example `5h | 59.0% left`.
2. Full-width remaining bar.
3. Reset text line below the bar: `reset {time}`, for example `reset Jul 6, 01:49`.

The limit type, such as `5h`, `7d`, `Cursor Models`, or `Other Models`, must not consume a separate left column. This lets every bar use 100% of the provider block content width.

Cursor surfaces only the two usage pools (`Cursor Models` from structured `auto`, `Other Models` from `api_models`). Structured `plan_usage` and `included_spend` stay in the data model and are not rendered on the card.

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

### Available Credits Line

When the provider has remaining credits, show one text line directly below the limit rows:

```text
Available credits: 344.2
```

The line is hidden when credits are unavailable.

### Manual Limit Resets

When `availableLimitResets` is greater than zero, show an informational line after credits. It uses the same visual style as the available credits line:

```text
Available resets: 1
```

This line shows availability only; it must not contain a control that redeems a reset.

---

## Subscription Section

Heading: `SUBSCRIPTION`. At most three lines, in this order.

```text
Pro ≈ $20.00 /mo
renews Sep 3, 2026
Manage · Billing
```

**Line 1 — plan and price.** Composed from `account.plan`, `account.price_amount`, `account.price_currency`, and `account.price_period`:

- all present: `{plan} ≈ {price} /{period}`
- plan only: `{plan}`
- price only: `≈ {price} /{period}`
- neither: the line is omitted

The price itself is rendered as the currency symbol immediately followed by the amount with two decimals, then a space, then `/` and the period token: `$20.00 /mo`. The full line for a Cursor Pro account reads exactly `Pro ≈ $20.00 /mo`.

The `≈` sign is mandatory whenever a price is shown. It is the compact form of the price disclaimer: a plan's real cost varies by country, currency, tax, and promotional terms, and the product does not claim to know the exact amount charged. It replaces a separate disclaimer line, which the line budget does not allow — the disclaimer text is never printed on the card. `account.price_note` remains in the structured data as the long-form explanation and may be surfaced as a tooltip, but it does not occupy a line. The card never omits the `≈` on the grounds that the note is absent from the card.

The amount and the currency are always shown together or not at all. A price whose currency the source did not state is not displayed as a bare number, and a plan's publicly known list price is never substituted for a price the source did not report; see [get-limits/structured-info-rules.md](../../get-limits/structured-info-rules.md).

**Line 2 — renewal.** From `account.renewal_at`, rendered as `renews {date}` in the date-only form documented in [presentation/time-display.md](../../presentation/time-display.md). A renewal date is only ever shown when it is still in the future; see [get-limits/structured-info-rules.md](../../get-limits/structured-info-rules.md).

**Line 3 — management links.** From `account.plan_management_url` and `account.billing_management_url`, rendered as `Manage · Billing` where each word is an external-style link (see [styleguide.md](styleguide.md)). When only one URL is present, only that link is shown. When neither is present, the line is omitted.

`account.subscription_started_at` is collected into structured data but is not displayed. It does not fit the line budget and is the least actionable of the subscription facts for day-to-day cost control.

---

## Usage Section

Heading: `USAGE`. At most four lines, one metric per line, always in this order:

```text
Tokens 711.2M
Sessions 92
Turns 6,394
Files 223
```

| Line | Structured source |
| --- | --- |
| `Tokens {total}` | `usage.tokens.total` |
| `Sessions {count}` | `usage.activity.sessions_count` |
| `Turns {count}` | `usage.activity.turns_count` |
| `Files {count}` | `usage.activity.files_count` |

This is the standard set for every provider and every source. The order is fixed so that the same metric sits on the same line across cards and the eye can compare providers vertically.

Counts use thousands separators (`6,394`). Token totals use a compact suffix (`711.2M`, `1.5B`) because they routinely reach nine and ten digits and would otherwise dominate the card.

A known zero renders as a line. `Sessions 0` states that the source reports no sessions yet; omitting the line would instead state that the source does not report sessions at all, which is a different fact.

The remaining `usage` fields — the token breakdown, money, events, latest activity, and top model — stay in structured data and are not displayed. Keeping the set to four metrics is what makes cards comparable and keeps them short; a fifth line would be spent differently by each source and the vertical alignment between cards would be lost.

A source that reports none of these four metrics shows no Usage heading and no Usage lines; when Show usage is on, the card still reserves the equalized Usage slot — see [section slot alignment](provider-blocks.md#section-slot-alignment).

Only Codex reports a `Files` count. Both Claude sources and `cursor_api2` leave `usage.activity.files_count` `null` as a confirmed source limit, so their Usage sections are three lines; a scanned-file or event count is never substituted for it.

`cursor_api2` reports `Tokens`, `Sessions`, and `Turns`, and no `Files` line. Its `Turns` figure counts billable usage events rather than interface turns; the difference is recorded in [get-limits/providers/cursor-api2-usage.md](../../get-limits/providers/cursor-api2-usage.md) and does not change how the line is rendered.

---

## Source Line

Provider source information is shown on one line, after all sections:

```text
API2, as of Jul 5, 22:12. Next upd 22:17
```

The line has two parts:

- source status: origin label, timestamp from `dataTimestamp`, rendered as `{origin label}, as of {timestamp}.`
- next update: `Next upd {time}`, the next time this provider's own update-frequency timer (see [controls.md](controls.md#update-frequency)) will fire, or `Manual upd only` when that provider's frequency is set to `Manual only`

Possible origin labels: `Local files`, `CLI`, `API2`, `Unknown`.

Each part is a non-breaking unit: `{origin label}, as of {timestamp}.` and `Next upd {time}` (or `Manual upd only`) must not wrap in the middle. If the provider block is too narrow for the full line, the line may break only between these two units.
