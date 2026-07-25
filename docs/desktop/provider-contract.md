# Tauri Provider Contract

## Query Fields

`ProviderLimitsQuery` uses camelCase JSON fields:

| Field | Type | Frontend source | Backend effect |
| --- | --- | --- | --- |
| `enabledCodex` | boolean | `appSettings.codex` | includes/excludes Codex source plan |
| `enabledClaude` | boolean | `appSettings.cloud` | includes/excludes Claude source plan |
| `enabledCursor` | boolean | `appSettings.cursor` | includes/excludes Cursor source plan |
| `sourcePriority` | `"fast"`, `"full"`, or `"best"` | `appSettings.sourcePriority` | selects source chain |
| `notificationsEnabled` | boolean | `appSettings.notifications` | allows notification checks for successful reports |

The Rust type has defaults, but the current frontend passes all fields explicitly.

Source priority values:

- `fast` uses `fast_free`.
- `full` uses `cli_fallback`.
- `best` uses `cli_first`.
- Cursor source selection is unchanged by `sourcePriority`.

Source chain order is defined in [../runtime/limits/source-chains.md](../runtime/limits/source-chains.md).

The provider response field contract is documented in [provider-response-contract.md](provider-response-contract.md).
