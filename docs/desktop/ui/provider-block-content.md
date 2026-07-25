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

## Credits Line

When the provider has remaining credits, show one text line directly below the limit rows:

```text
Credits: 344.2
```

The line is hidden when credits are unavailable.

---

## Manual Limit Resets

When `availableLimitResets` is greater than zero, show an informational section after credits and before the source line:

```text
Resets: 1
```

This section shows availability only; it must not contain a control that redeems a reset.

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
