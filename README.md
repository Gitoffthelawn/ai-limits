# ai-limits

English | [Русский](README.ru.md)

Check AI subscription limits easily. Codex, Claude, Cursor.

`ai-limits` is a desktop-first local tool for viewing available AI usage and limit data from supported providers. A stateless command-line interface is also available for automation and diagnostics.

## Interfaces

### Desktop App

The desktop app is the primary product interface for configuration, notifications, and interactive use.

The desktop app is currently in beta.

- macOS works as an app.
- Windows and Linux builds exist and are being tested with real users.
- The interface is functional, but still early.

Download: https://github.com/md2it/ai-limits/releases

### Terminal UI

The terminal interface is a stateless headless interface for automation and diagnostics. It performs one query per invocation, uses built-in defaults unless source flags are provided, and does not maintain a separate configuration.

Run from the repository:

```sh
./bin/ai-limits
```

Show help:

```sh
./bin/ai-limits --help
```

For terminal UI details, see [docs/terminal/interface.md](docs/terminal/interface.md).

## What It Shows

- Current limits for Codex, Claude, and Cursor when available.
- Usage information when a supported source can provide it.
- Results from local files, provider CLIs, and other supported sources.
- Default, structured, and raw output views in the terminal.

## Current Limitations

- No macOS DMG installer yet.
- macOS releases are signed and notarized; Windows and Linux builds are unsigned.
- Desktop notifications currently work on macOS only.
- Windows and Linux desktop builds are still being tested.
- Some local Codex and Claude data sources may not work on Windows and Linux yet.
- CLI-backed data sources are expected to be the most portable option across platforms.

## Documentation

- [Terminal UI](docs/terminal/interface.md)
- [Developer documentation](docs/)

## License

This project is licensed under the [MIT License](LICENSE).
