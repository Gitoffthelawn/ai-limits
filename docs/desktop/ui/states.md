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

The button opens the [Help page](help.md) on its Source priority section, which explains the Fast, Full, and Best modes and links to the Claude and Codex CLI setup guides.

The setup links must open externally from the Tauri app:

- Claude setup guide: <https://code.claude.com/docs/en/setup>
- Codex CLI guide: <https://developers.openai.com/codex/cli>

Backend and frontend state details are documented in [states-data.md](states-data.md).
