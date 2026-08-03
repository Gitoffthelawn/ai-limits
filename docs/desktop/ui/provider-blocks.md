# Tauri UI Provider Blocks

## Block Structure

Each provider square contains, top to bottom:

- provider name
- **Limits section**
- **Subscription section**
- **Usage section**
- source line with data origin label and timestamp
- update frequency dropdown near the bottom
- provider-specific manual update button at the bottom

The three sections are the desktop rendering of the product's three output kinds, defined independent of any interface in [product/output-kinds.md](../../product/output-kinds.md). Each is shown only when the source has data for it, and each can be hidden independently through the [display toggles](settings.md#display).

Provider names follow the shared [provider naming rules](../../presentation/provider-names.md).

## Section Headings

Each section opens with a horizontal divider carrying the section name centered in it:

```text
──────── LIMITS ────────
───── SUBSCRIPTION ─────
──────── USAGE ─────────
```

The heading names the numbers below it, so the user does not have to infer what a bare figure means. The divider and its label occupy the same vertical space an unlabelled divider would.

A heading is rendered only when its section has content. A hidden or empty section contributes neither heading nor divider, so a card never shows a heading over nothing.

Divider and label styling is documented in [provider-block-colors.md](provider-block-colors.md).

## Line Budget

Sections are deliberately short so that provider cards stay close to the same height and the row looks orderly:

| Section | Lines | Content |
| --- | --- | --- |
| Limits | one group per limit window, plus optional credits and manual-reset lines | unchanged from current behavior |
| Subscription | at most 3 | plan with price, renewal, management links |
| Usage | at most 4 | one metric per line |

These budgets are caps, not padding. A source with less data produces a shorter section; the card is not padded to reach the cap, and no placeholder or dash is shown for a missing value.

Cards in a row stretch to the height of the tallest card. Because every section has the same small line budget for every provider, natural heights stay close and that stretch stays visually minor.

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

API2, as of Jul 5, 19:28
───────────────────────
Upd every [ 5 min ▾ ]
[     UPDATE NOW     ]
```

The UI does not use terminal-style ASCII rendering. The example defines the information that must be visible and its order.

## Partial Coverage

Sources differ in what they expose, and partial coverage is the normal case rather than an error state. A value that is absent contributes no line, and a section with no values at all is omitted with its heading.

```text
Claude                          Codex

──────── LIMITS ────────        ──────── LIMITS ────────
5h | 100.0% left                5h | 92.0% left
▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇         ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇░░
reset Jul 6, 00:20              reset 20:48
7d | 84.0% left                 Available credits: 344.2
▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇░░░░
                                ───── SUBSCRIPTION ─────
──────── USAGE ─────────        Plus
Tokens 711.2M                   renews Sep 3, 2026
Sessions 92
Turns 6,394                     ──────── USAGE ─────────
Files 223                       Tokens 1.5B
                                Files 921
CLI, as of Jul 5, 19:29
                                Local files, as of Jul 5, 19:28
```

Claude has no subscription data, so that section and its heading are absent entirely. Codex reports a plan name but no price and no management links, so its subscription section is two lines instead of three, and it reports no session or turn counts, so its usage section is two lines instead of four.

Section content rules are documented in [provider-block-content.md](provider-block-content.md); accent and divider colors in [provider-block-colors.md](provider-block-colors.md).
