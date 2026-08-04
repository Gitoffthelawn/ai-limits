# Tauri UI Frontend State

The current frontend calls:

- `get_single_provider_limits` for provider refreshes.
- `open_external_url` for allowlisted setup guide links.
- `start_provider_cli_login` only when the user selects the provider Sign in action.

Each provider is requested independently; there is no aggregate limits-fetch command in the desktop IPC contract.

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
| `selectedUpdateFrequency` | fallback default for provider interval if no local value exists |
| `errorMessage` | marks refresh as failed and supplies fallback message outside no-fresh-data and CLI authorization states |
| `noFreshData` | renders no-fresh-data empty state with source priority controls |
| `authorizationRequired` | when `codex` or `claude`, renders the CLI authorization problem with Sign in and the manual login command |

## Frontend-Only Fields And State

These values are not returned by the backend:

- `pending`, added by `createEmptyProvider` before the first response.
- `appSettings.notifications`.
- `appSettings.cursor`.
- `appSettings.cloud`.
- `appSettings.codex`.
- `appSettings.sourcePriority`.
- `appSettings.showLimits`.
- `appSettings.showPlan`.
- `appTheme`, persisted separately from app settings.
- provider update interval selected in the dropdown after local initialization.
- provider refresh timers.
- provider refresh in-flight markers.
- settings dropdown open/closed state.
- help view open/closed state and the selected help chapter.

`selectedUpdateFrequency` exists in the backend response and is currently always `"5 min"`, but persisted frontend intervals override it after the user changes a provider dropdown.

`appSettings.sourcePriority` is `"full"` by default. Fast maps to `fast_free`, Full maps to `cli_fallback`, and Best maps to `cli_first`. Source chain order is defined in [../../get-limits/source-chains.md](../../get-limits/source-chains.md).

`appSettings.showLimits` and `appSettings.showPlan` are `true` by default. They toggle the Limits and Plan sections of every provider block in place, without triggering a refresh; see [settings.md](settings.md#display) and [provider-blocks.md](provider-blocks.md).
