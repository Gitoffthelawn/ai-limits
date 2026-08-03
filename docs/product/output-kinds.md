# User-Facing Output Kinds

## Purpose

This document defines the output kinds the product presents to the user, independent of which interface renders them and independent of how the underlying data was collected.

It applies to every current and future interface built on the shared core: the desktop app, the terminal CLI, macOS widgets, the menu bar, and any interface added later. It is not scoped to any single interface's implementation.

## The three output kinds

The system presents normalized structured data (see [get-limits/structured-info.md](../get-limits/structured-info.md)) to the user as three output kinds:

1. **Limits** — the remaining and used share of each rate-limit window that applies to a provider, account, or plan, with its reset time. Sourced from `limits[]` and `available_limit_resets`.
2. **Plan** — the user's subscription/tariff context: which plan they are on, when it started, when it renews, and what it costs. Sourced from `account`.
3. **Usage** — how much of the provider's tracked consumption has been used, in whatever shape that provider reports it (tokens, money, activity, model mix). Sourced from `usage`.

These are the three highest-value ways to answer "where do I stand with this provider," in that order of everyday usefulness: limits tell the user if they are about to be blocked, plan tells the user what they are paying for and when it changes, usage gives supporting detail.

## Cross-interface expectation

Any interface may implement any subset of these three output kinds, and may implement none of them for a given provider if the provider's structured data does not support it. Showing all three is not a requirement placed on every interface.

What is fixed is the categorization itself: an interface that shows subscription context does so as **Plan**, an interface that shows consumption detail does so as **Usage**, and an interface that shows quota headroom does so as **Limits**. Interfaces do not invent a fourth category or blend these three into an undifferentiated feed, so that the same mental model transfers between the terminal, the desktop app, and any interface added later.

The Tauri desktop app is the first interface implementing all three output kinds side by side, one per provider block. Its concrete layout, rendering rules, and visibility toggles are documented in [desktop/ui/provider-blocks.md](../desktop/ui/provider-blocks.md), [desktop/ui/provider-block-content.md](../desktop/ui/provider-block-content.md), and [desktop/ui/settings.md](../desktop/ui/settings.md).

## Plan output: goal and content

The goal of the Plan output is to give the user a compact, at-a-glance way to keep subscriptions under control, without needing to open the provider's own billing page.

When available for a source, Plan output shows:

- the tariff/plan name
- when the current subscription started
- when it next renews
- what it costs

Price is shown with the understanding that it can vary by currency, country, region, or promotional terms; a source that cannot guarantee one universal price value carries a disclaimer alongside the price rather than presenting it as an unconditional fact. See `account.price_note` in [get-limits/structured-info-schema.md](../get-limits/structured-info-schema.md).

Direct links into the provider's own plan-change or billing-management pages are an optional addition to the Plan output, not a required part of it.

### Current source coverage

Plan output degrades per source, and partial coverage is the normal case rather than an error state:

| Source | Plan name | Started | Renews | Price | Management links |
| --- | --- | --- | --- | --- | --- |
| `codex_local` | yes | yes | yes | no | no |
| `codex_cli` | no | no | no | no | no |
| `claude_cli` | no | no | no | no | no |
| `claude_local` | no | no | no | no | no |
| `cursor_api2` | no | no | yes | no | no |

No current source exposes price or management links. Those fields are specified so an interface knows how to render them and so a future source can populate them, but they are `null` everywhere today and no interface displays them.

A renewal date is only shown when it is still in the future. Sources that read a cached local credential can carry a subscription window that has already elapsed; per the "no weak assumptions" rule in [get-limits/structured-info-rules.md](../get-limits/structured-info-rules.md), an elapsed renewal date is reported as unknown with a diagnostic rather than presented as an upcoming charge.

## Usage output: goal and content

Usage shape differs sharply between providers: one provider reports tokens, another reports money, another reports session/turn counts, another reports a model mix. The Usage output does not force these into one fixed layout. An interface renders whichever `usage.*` fields are non-null for a given source, in human-readable form, and omits the rest. There is no expectation that two providers' Usage output look alike.
