# Tauri Commands

The current desktop UI does not depend on one combined provider response for normal refresh behavior.

Tauri commands support fetching one provider at a time, for example Codex, Claude, or Cursor. The frontend starts provider refreshes in parallel and updates each provider block as soon as that provider result is available.

This keeps slow sources from blocking unrelated provider blocks and matches the planned per-provider update frequency controls.

This document describes the factual command/response contract implemented in `src-tauri/src/commands.rs` and used by `frontend/index.html`.

The `open_external_url`, `get_cli_command`, and `run_cli_in_terminal` commands are documented in [commands-utility.md](commands-utility.md). The batch `get_provider_limits` command is documented in [commands-batch.md](commands-batch.md).

## `get_single_provider_limits`

Input:

```json
{
  "providerId": "codex",
  "query": {
    "enabledCodex": true,
    "enabledClaude": true,
    "enabledCursor": true,
    "sourcePriority": "full",
    "notificationsEnabled": true
  }
}
```

Rust inputs:

- `provider_id: String`
- `query: ProviderLimitsQuery`

Response on success:

```json
{
  "id": "codex",
  "label": "Codex",
  "sourceId": "codex-local",
  "dataTimestamp": "Jul 5, 19:28",
  "selectedUpdateFrequency": "5 min",
  "limits": [
    {
      "label": "5h",
      "remainingPercentage": 92.0,
      "resetTime": "20:48"
    }
  ],
  "creditsRemaining": null,
  "availableLimitResets": 1,
  "errorMessage": null,
  "noFreshData": false
}
```

The command returns `Result<ProviderLimits, String>`.

If the provider is disabled or unknown for the passed query, the command returns an error string:

```text
Provider '{provider_id}' is disabled or unknown
```

Frontend usage:

- called as `invoke("get_single_provider_limits", { providerId, query: settingsToQuery() })`
- one call is started per enabled provider.
- successful provider responses mount or update one provider block.
- thrown command errors are caught by the frontend and shown only as a transient provider `Failed` status.
