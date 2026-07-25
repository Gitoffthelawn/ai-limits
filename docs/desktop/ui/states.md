# Tauri UI States

## No Fresh Data State

If checked sources return no fresh usable limit records, the provider block must show an empty state instead of a technical error like `No usable limit records from this source`.

Short copy:

```text
No fresh limits' data. Try another source mode:
```

Show the same source priority segmented control used in settings:

```text
Fast | Full | Best
```

Below the control, show a text button:

```text
More details
```

The button opens the [Help page](help.md) on its Source priority section.

Help section copy:

```text
Source priority

Fast uses quick local/provider-native sources only.

Full checks quick sources first, then uses CLI fallback for Codex and Claude.

Best checks CLI first for Codex and Claude, then falls back to quick sources.

CLI checks may take longer, but usually provide more accurate and current Codex and Claude data.

This setting affects Codex and Claude. Cursor keeps its existing source behavior.

Provider refreshes run asynchronously, so one slower provider should not block the others.

Setup guides:
Claude guide on GitHub
Codex guide on GitHub
```

The setup links must open externally from the Tauri app:

- Claude setup guide: <https://code.claude.com/docs/en/setup>
- Codex CLI guide: <https://developers.openai.com/codex/cli>

Each setup guide opens with helper copy for sharing the guide with an agent.

Backend state:

- selected by `noFreshData: true`.
- shown when `limits` is empty.
- does not use `errorMessage` text in this state.

Frontend-only state:

- inline source priority control state comes from `appSettings.sourcePriority`.
- help view open/closed state and the selected help chapter are frontend state.
