# Tauri UI Controls

## Provider Actions

Each provider block ends with a row of two controls, side by side:

```text
[     UPDATE NOW     ] [⚙]
```

`UPDATE NOW` takes the remaining row width and refreshes only that provider block.

The gear button opens that provider's own settings dropdown, in the same visual style as the gear button next to `UPDATE ALL NOW` (see [settings.md](settings.md)) — but scoped to this one card instead of the whole app. The dropdown opens upward and left, anchored to the gear button's top-right corner, so it stays within the bounds of its own card instead of spilling into the row below or past the card's edge.

## Update Frequency

The per-provider settings dropdown holds a single group, headed `UPDATE FREQUENCY`, with the options listed in a column below it:

- Manual only
- 1 min
- 5 min
- 10 min
- 30 min
- 1 hour

Default value:

- 5 min

The selected option is highlighted in green. Choosing another option applies immediately and updates the source line's next-update time (see [provider-block-content.md](provider-block-content.md#source-line)).
