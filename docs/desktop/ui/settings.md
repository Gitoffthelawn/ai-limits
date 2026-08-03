# Tauri UI Settings

## Settings

The settings button opens a dropdown grouped into behavior, provider visibility, display, and other sections.

The behavior section is first.

It has a source priority three-option segmented control:

- Fast
- Full
- Best

Default value:

- Full

Source priority behavior:

- Fast uses the `fast_free` source chain from [../../get-limits/source-chains.md](../../get-limits/source-chains.md).
- Full uses the `cli_fallback` source chain.
- Best uses the `cli_first` source chain.
- Full and Best may take longer than Fast because they can run provider CLI checks.
- Best usually provides more accurate and current Codex and Claude data because it starts provider CLI checks first.
- Cursor uses its existing Cursor source and is not affected by source priority.

The behavior section includes an information action. It opens the [Help page](help.md) on its Source priority section, which explains the Fast, Full, and Best modes, their source chains, the speed/accuracy tradeoff, and the provider scope. The no-fresh-data provider state links to the same Help section.

The provider visibility section has toggles:

- Cursor
- Cloud
- Codex

## Display

The display section has toggles:

- Show limits
- Show plan
- Show usage

These control the Limits, Plan, and Usage sections of every provider block, defined in [provider-blocks.md](provider-blocks.md) and [provider-block-content.md](provider-block-content.md). They apply to all provider blocks at once; there is no per-provider override.

Defaults:

- Show limits, Show plan, and Show usage are all on.

User experience:

- Turning a display toggle off hides the matching section in every provider block immediately, without waiting for a refresh.
- Turning a display toggle back on immediately shows the matching section again using the data already held for each provider, without triggering a refresh.
- This differs from the provider visibility toggles below, whose effect on the next limits request only takes effect on the next refresh; display toggles never change what data is requested, only what is rendered from data already on hand.

## Other

The other section has toggles:

- Notifications
- Dark theme

Defaults:

- Notifications, Cursor, Cloud, and Codex are on
- Dark theme follows the system theme until the user changes it manually

User experience:

- Notifications controls whether the app sends system limit alerts for every notification type in [../../notifications/content.md](../../notifications/content.md), including low remaining and 100% again
- Cursor, Cloud, and Codex control which provider blocks are shown and which providers are included in the next limits request
- Cloud corresponds to Claude
- Changing a toggle saves the choice and hides disabled provider blocks, but does not start a refresh
- Saved choices apply on the next manual refresh or scheduled provider update

Settings storage:

- settings are saved in `localStorage` under `ai-limits-settings`.
- theme preference is saved in `localStorage` under `ai-limits-theme`.
- per-provider update intervals are saved in `localStorage` under `ai-limits-provider-intervals`.
- these saved settings are frontend state; they are not returned by the backend.
- Show limits, Show plan, and Show usage are saved in `ai-limits-settings` alongside the other toggles; they are purely a rendering choice and are never sent to the backend as part of the limits request.

Settings request mapping:

| UI setting | Command query field |
| --- | --- |
| Notifications | `notificationsEnabled` |
| Cursor | `enabledCursor` |
| Cloud | `enabledClaude` |
| Codex | `enabledCodex` |
| Source priority | `sourcePriority` |

Show limits, Show plan, and Show usage have no command query field: they never affect what is requested from the backend, only what the frontend renders from the response it already has.
