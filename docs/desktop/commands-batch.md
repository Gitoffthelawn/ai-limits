# Tauri Batch Provider Limits Command

## `get_provider_limits`

Input:

```json
{
  "query": {
    "enabledCodex": true,
    "enabledClaude": true,
    "enabledCursor": true,
    "sourcePriority": "full",
    "notificationsEnabled": true
  }
}
```

Rust input type: `ProviderLimitsQuery`.

Response on success:

```json
[
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
]
```

The command returns `Result<Vec<ProviderLimits>, String>`.

Frontend status: currently not called by `frontend/index.html`; normal refresh uses `get_single_provider_limits` per enabled provider, documented in [commands.md](commands.md).
