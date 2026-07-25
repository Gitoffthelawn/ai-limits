# Codex CLI Usage

## Provider Method: `codex_cli_usage`

Minimum commands:

- verify CLI availability: `command -v codex`
- verify CLI version: `codex --version`
- official website: https://openai.com/codex
- CLI documentation: https://developers.openai.com/codex/cli

Verified PoC details:

- launches the standard `codex` command without a custom path to the CLI
- Codex CLI refuses to launch the interactive TUI if `stdin`/`stderr` are not TTYs
- for PoC, the system `expect` command is used as a minimal PTY adapter
- runtime sets `TERM=xterm-256color`, `COLUMNS=120`, `LINES=40` and runs `stty cols 120 rows 40`
- PoC sends `/status` via bracketed paste
- the first `/status` call sometimes triggers a limit refresh
- a second `/status` call returns the actual breakdown
- the parser waits for response indicators: startup screen, `refresh requested`, limit lines, or `Credits`
- user-facing output shows only the found summary: `5h limit`, `Weekly limit`, and `Credits`

### Manual Limit Resets

Verified CLI behavior:

- `/usage` reports the number of available manual resets, for example `You have 1 usage limit reset available`;
The collector reads `/usage` after `/status` in the same PTY session, then interrupts the CLI. It never confirms or redeems a reset. The source is the rendered `/usage` TUI stream; no local JSON object or array for these records has been verified. The implementation extracts only `available_limit_resets`; it never enters the redemption path. The normalized field is defined in [structured-info.md](../structured-info.md).
