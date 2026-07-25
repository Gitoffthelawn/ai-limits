# Tauri Utility Commands

## `open_external_url`

Input:

```json
{
  "url": "https://code.claude.com/docs/en/setup"
}
```

Rust input:

- `url: String`

Response on success:

```json
null
```

The command returns `Result<(), String>`.

Allowed URLs:

- `https://code.claude.com/docs/en/setup`
- `https://developers.openai.com/codex/cli`
- `https://github.com/md2it/ai-limits`
- `https://github.com/md2it/ai-limits/blob/main/LICENSE`

Any other URL returns:

```text
External URL is not allowed
```

Frontend usage:

- called from CLI setup guide buttons on the Help page source priority section.
- if Tauri is unavailable, the frontend falls back to `window.open`.

## `get_cli_command`

Response on success is the shell-quoted command for the running desktop executable with `--cli`. The command remains valid if the macOS app bundle has been moved.

## `run_cli_in_terminal`

Opens macOS Terminal and executes the same command returned by `get_cli_command`. Other platforms return an unsupported-operation error.
