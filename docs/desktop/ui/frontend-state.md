# Tauri UI Frontend State

The current frontend calls:

- `get_single_provider_limits` to run an actual collection for one provider (starts a source-chain walk, or joins one already in flight for that provider — see [Shared Structured Data Cache](#shared-structured-data-cache)).
- `get_cached_provider_limits` to read the shared snapshot for one provider without starting or joining a collection; resolves to `null` if nothing has been collected for that provider yet this session.
- `open_external_url` for allowlisted setup guide links.
- `start_provider_cli_login` only when the user selects the provider Sign in action.

Each provider is requested independently; there is no aggregate limits-fetch command in the desktop IPC contract.

## Shared Structured Data Cache

The application process keeps one current `StructuredSourceInfo` snapshot for each provider, held in `StructuredInfoCache` (`src-tauri/src/commands/structured_cache.rs`). The snapshot is written only after the provider source chain has selected its result and applied account-field backfill.

Main Window and Menu Bar Popover read the same snapshot. Each surface independently projects the unchanged structured data into its own provider-card state and keeps only presentation state locally.

The shared cache contains no raw provider output, stderr, UI-specific `ProviderLimits` model, rendered markup, or cached transport failures. A failed collection does not replace the last successful structured snapshot.

`collected_at` and `data_as_of` remain fields of the unchanged structured snapshot. `collected_at` is the time of the actual collection and is the common basis for update scheduling; `data_as_of` remains the freshness timestamp of the provider data and is displayed to the user according to the presentation rules. `collected_at` also reaches the frontend as `ProviderLimits.collectedAt` (raw ISO-8601, unformatted — unlike `dataTimestamp`, which arrives pre-formatted for display) — see [Provider Fields Used](#provider-fields-used) and [refresh.md](refresh.md#shared-refresh-schedule).

Concurrent requests for the same provider must share one collection operation: `CollectionCoordinator` (same file) tracks in-flight collections per provider id, and a second `get_single_provider_limits` call for a provider already being collected joins that call's result instead of starting a second one. On a successful collection, notifications are evaluated once from the selected structured snapshot, the shared cache is updated, and only then is a `provider-updated` Tauri app event emitted (payload: the same `ProviderLimits` shape a direct response carries) — every open surface listens for it and updates its own mounted card from the payload, without starting a collection of its own. The Popover, which is not a Tauri-managed window, receives it via the same native forwarding `popover_panel::install_event_forwarding` already used for `settings-changed`/`theme-changed` (see [mac-popover.md](../mac-popover.md#cross-window-sync)). A failed collection updates neither the shared cache nor emits this event.

On initializing its provider list (first load, or a provider newly re-enabled), a surface calls `get_cached_provider_limits` for each enabled provider before deciding whether to collect: a provider with an existing shared snapshot renders it immediately and only re-collects if its own update-frequency schedule (recomputed from that snapshot's `collectedAt`) says a refresh is already due; a provider with no shared snapshot yet collects normally. This is what lets a surface opened after the other has already collected data show it immediately, with no collection of its own.

User-facing problem and recovery rules are documented in [problems.md](problems.md).

## Provider Fields Used

| Backend field | Frontend usage |
| --- | --- |
| `id` | provider block identity, DOM `data-provider-id`, timer maps |
| `label` | provider heading, accessibility labels |
| `limits` | rendered limit rows; empty array selects empty/error state |
| `limits[].label` | row label before `% left` |
| `limits[].remainingPercentage` | displayed percent, bar width, bar color |
| `limits[].resetTime` | optional reset line |
| `plan` | Plan section content, always an object `{ lines: string[], links: { label: string, url: string }[] }`, never `null`. `lines` are ready-to-render text lines built by the backend from `account.plan`, `account.subscription_started_at`, `account.renewal_at`, `account.price_amount`, `account.price_currency`, and `account.price_note`; the frontend renders them verbatim in order. `links` carries at most `Manage plan` (from `account.plan_management_url`) and `Manage billing` (from `account.billing_management_url`); missing URLs are omitted by the backend. When both `lines` and `links` are empty, the section has no heading and no lines; if Show plan is on, the card still reserves the equalized slot — see [provider-blocks.md](provider-blocks.md#section-slot-alignment) |
| `sourceId` | origin label in `{label},`; possible values: `Local files`, `CLI`, `API2`, `Unknown` |
| `dataTimestamp` | `as of {timestamp}`; missing value displays `unknown` |
| `collectedAt` | raw ISO-8601 instant of the actual collection, not displayed; seeds the shared refresh schedule — see [refresh.md](refresh.md#shared-refresh-schedule) |
| `selectedUpdateFrequency` | fallback default for provider interval if no local value exists |
| `errorMessage` | marks refresh as failed and supplies fallback message outside no-fresh-data and CLI authorization states |
| `noFreshData` | renders the no-fresh-data empty state with a link to data-availability help |
| `authorizationRequired` | when `codex` or `claude`, renders the CLI authorization problem with Sign in and the manual login command |

## Frontend-Only Fields And State

These values are not returned by the backend:

- `pending`, added by `createEmptyProvider` before the first response.
- `appSettings.notifications`.
- `appSettings.cursor`.
- `appSettings.cloud`.
- `appSettings.codex`.
- `appSettings.showLimits`.
- `appSettings.showPlan`.
- `appTheme`, persisted separately from app settings.
- provider update interval selected in the dropdown after local initialization.
- provider refresh timers.
- provider refresh in-flight markers.
- settings dropdown open/closed state.
- help view open/closed state and the selected help chapter.

`selectedUpdateFrequency` exists in the backend response and is currently always `"5 min"`, but persisted frontend intervals override it after the user changes a provider dropdown.

`appSettings.showLimits` and `appSettings.showPlan` are `true` by default. They toggle the Limits and Plan sections of every provider block in place, without triggering a refresh; see [settings.md](settings.md#display) and [provider-blocks.md](provider-blocks.md).
