# Source Chains

This document is the source of truth for provider source order.

A source chain is an ordered list of provider methods. The app tries the next method only when the current method does not provide usable limit data.

Usable limit data means:

- `access_available = true`
- `data_available = true`
- at least one limit record is present

Which interface mode uses which chain is documented in [source-chain-mapping.md](source-chain-mapping.md).

## Chains

### `fast_free`

Fast local/provider-native chain. It avoids provider CLI checks.

```text
Codex: codex_local
Claude: claude_local
Cursor: cursor_api2
```

### `cli_fallback`

Local/provider-native chain with CLI fallback for Codex and Claude.

```text
Codex: codex_local -> codex_cli
Claude: claude_local -> claude_cli
Cursor: cursor_api2
```

### `cli_first`

CLI-first chain for more accurate and current Codex and Claude data. CLI checks may take longer.

```text
Codex: codex_cli -> codex_local
Claude: claude_cli -> claude_local
Cursor: cursor_api2
```
