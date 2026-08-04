# Tauri UI Provider Blocks

## Block Structure

Each provider square contains, top to bottom:

- provider name
- **Limits section**
- **Subscription section**
- **Usage section**
- source line with data origin label, timestamp, and next scheduled update
- provider-specific manual update button and a settings gear at the bottom, side by side

The three sections are the desktop rendering of the product's three output kinds, defined independent of any interface in [product/output-kinds.md](../../product/output-kinds.md). Each can be hidden independently through the [display toggles](settings.md#display). A display toggle turns the matching section slot on or off for every provider card at once. Heading and lines appear only when that card has data for the section; when the toggle is on and the card has no data, the slot stays as reserved empty space (see [Section Slot Alignment](#section-slot-alignment)).

Provider names follow the shared [provider naming rules](../../presentation/provider-names.md).

## Section Headings

Each section opens with a horizontal divider carrying the section name centered in it:

```text
──────── LIMITS ────────
───── SUBSCRIPTION ─────
──────── USAGE ─────────
```

The heading names the numbers below it, so the user does not have to infer what a bare figure means. The divider and its label occupy the same vertical space an unlabelled divider would.

A heading is rendered only when its section has content. An empty reserved slot has neither heading nor divider, so a card never shows a heading over nothing. A section whose display toggle is off contributes no slot at all.

Divider and label styling is documented in [provider-block-colors.md](provider-block-colors.md).

## Line Budget

Sections are deliberately short so cards stay compact:

| Section | Lines | Content |
| --- | --- | --- |
| Limits | one group per limit window, plus optional credits and manual-reset lines | unchanged from current behavior |
| Subscription | at most 3 | plan with price, renewal, management links |
| Usage | at most 4 | one metric per line |

These budgets are content caps. A source with less data produces fewer lines; no placeholder or dash is shown for a missing value. Equal height across cards comes from [section slot alignment](#section-slot-alignment), not from padding each section up to its line-budget cap.

## Section Slot Alignment

Every visible provider card uses the same section-slot structure: one slot per section whose display toggle is on, in the Limits → Subscription → Usage order. Each card owns its own layout; the row is not one shared grid spanning all cards.

For each section type that is toggled on, the height of that slot is the maximum content height of that same slot among all currently visible provider cards. Cards hidden by provider visibility do not take part in the maximum. When a card has less content in a slot than that maximum — including no content at all — the leftover height is empty space (air). Missing values are never filled with placeholders or dashes.

Turning a display toggle off removes that slot from every card, so there is nothing to measure for that section type. Turning it back on restores the slot on every card and recalculates heights from the visible cards only.

Content inside each equalized slot is top-packed: the heading and any body stay at the top edge of the slot, and the leftover air stays below.

Cards in a row still stretch to the height of the tallest card so source line, update controls, and footers stay aligned at the bottom.

## Target Card

The complete card, with every supported field populated and all display toggles on. This is a layout reference, not a portrait of any real provider: no source currently supplies all of these values at once, and the provider name below is illustrative. What each source actually reports is shown in "Partial Coverage" further down.

```text
Cursor

──────── LIMITS ────────
plan | 54.6% left
▇▇▇▇▇▇▇▇▇▇▇▇░░░
reset Jul 28, 03:00
auto | 63.7% left
▇▇▇▇▇▇▇▇▇▇▇▇▇▇░░░░
Available credits: 344.2

───── SUBSCRIPTION ─────
Pro ≈ $20.00 /mo
renews Sep 3, 2026
Manage · Billing

──────── USAGE ─────────
Tokens 711.2M
Sessions 92
Turns 6,394
Files 223

API2, as of Jul 5, 19:28. Next upd 19:33
───────────────────────
[     UPDATE NOW     ] [⚙]
```

The UI does not use terminal-style ASCII rendering. The example defines the information that must be visible and its order.

## Partial Coverage

Sources differ in what they expose, and partial coverage is the normal case rather than an error state. A value that is absent contributes no line. A section with no values at all has no heading and no lines, but when its display toggle is on the card still keeps a reserved empty slot whose height matches the same slot on the other visible cards.

```text
Claude                          Codex

──────── LIMITS ────────        ──────── LIMITS ────────
5h | 100.0% left                5h | 92.0% left
▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇         ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇░░
reset Jul 6, 00:20              reset 20:48
7d | 84.0% left                 Available credits: 344.2
▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇░░░░
                                ───── SUBSCRIPTION ─────
───── SUBSCRIPTION ─────        Plus
Pro                             renews Sep 3, 2026

──────── USAGE ─────────        ──────── USAGE ─────────
Tokens 711.2M                   Tokens 1.5B
Sessions 92                     Sessions 148
Turns 6,394                     Turns 2,910
                                Files 921

CLI, as of Jul 5, 19:29         Local files, as of Jul 5, 19:28
```

Claude reports a plan name but no renewal date, no price, and no management links, so its subscription section is one line. It also reports no changed-file count, so the `Files` line is absent from its Usage section. Codex reports a plan name and a renewal date but no price and no management links, so its subscription section is two lines instead of three. The side-by-side above shows content only; on screen the matching section slots share equalized heights with empty space where a card has fewer lines — see [Section Slot Alignment](#section-slot-alignment).

Section content rules are documented in [provider-block-content.md](provider-block-content.md); accent and divider colors in [provider-block-colors.md](provider-block-colors.md).
