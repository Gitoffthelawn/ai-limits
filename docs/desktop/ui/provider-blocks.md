# Tauri UI Provider Blocks

## Block Structure

Each provider square contains, top to bottom:

- provider name
- **Limits section** — limit rows, credits line when available, manual limit-reset availability when available
- **Plan section** — subscription/tariff details, when available
- **Usage section** — human-readable consumption details, when available
- source line with data origin label and timestamp on one line by default
- update frequency dropdown near the bottom
- provider-specific manual update button at the bottom

The Limits, Plan, and Usage sections are the desktop rendering of the product's three output kinds, defined independent of any interface in [product/output-kinds.md](../../product/output-kinds.md). Each section is shown only when the source has data for it, and each can be hidden independently through the [display toggles](settings.md#display).

The three sections carry no section title or other explicit label. They are separated from each other only by the same horizontal divider already used inside a provider block to separate metadata from controls, styled per [provider-block-colors.md](provider-block-colors.md). A provider block with only one section populated shows no dividers, since there is nothing to separate.

Provider names follow the shared [provider naming rules](../../presentation/provider-names.md).

The provider content should roughly match the current terminal output model.

Example data shape:

```text
     --------- CURSOR --------
plan | 54.6% left
■■■■■■■■■■■■□□□
reset Jul 28, 03:00
auto | 63.7% left
■■■■■■■■■■■■■■□□□□
api | 24.5% left
■■■■■□□□□□□□□□□□□□□□
──────────────────────────
Renews Jul 28, 2026
──────────────────────────
Spend this period: $20.00
API2, as of Jul 5, 19:28

     --------- CODEX ---------
5h | 92.0% left
■■■■■■■■■■■■■■■■■■■■■■■□□
reset 20:48
7d | 35.0% left
■■■■■■■■■□□□□□□□□□□□□□□□□
reset Jul 10, 03:55
Available credits: 344.2
──────────────────────────
Plan: Plus
Started Jan 12, 2026 · renews Aug 8, 2026
──────────────────────────
Tokens: 1.5B total
Files: 921 · Events: 22,545
Local files, as of Jul 5, 19:28

     --------- CLAUDE --------
5h | 100.0% left
■■■■■■■■■■■■■■■■■■■■■■■■■
reset Jul 6, 00:20
7d | 84.0% left
■■■■■■■■■■■■■■■■■■■■□□□□
reset Jul 7, 13:00
──────────────────────────
Tokens: 711.2M total
Sessions: 92 · Turns: 6,394 · Files: 223
Top model: claude-sonnet-5
CLI, as of Jul 5, 19:29
```

The three providers above deliberately show different section coverage: Cursor has a renewal date but no plan name, Codex has both plus credits, and Claude has no subscription data at all and therefore no Plan section and only one divider. This is the normal case, not an edge case.

The UI does not need to use terminal-style ASCII rendering. The example defines the information that must be visible; the `──────` lines stand in for the horizontal section divider.

Accent color rules are documented in [provider-block-colors.md](provider-block-colors.md); limit row, credits, plan, usage, and source line content are documented in [provider-block-content.md](provider-block-content.md).
