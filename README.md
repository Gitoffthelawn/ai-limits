# ai-limits

English | [Русский](README.ru.md)

A local app for tracking AI subscription limits and usage across Codex, Claude, and Cursor.

Benefits:
- Works without an API subscription,
- No sign-ins,
- All providers in one place,
- Completely free,
- Private: no third-party services, proxies, or registrations,
- Lightweight desktop app for macOS, Windows, and Linux,
- Limit notifications,
- Open source.

## Download

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-macos-app.zip"><img src="https://shieldcn.dev/badge/macOS-v0.0.13-grey.svg?logo=apple" alt="Download for macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.0.13-blue.svg?logo=windows" alt="Download for Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.0.13/AI-Limits-v0.0.13-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.0.13-yellow.svg?logo=linux" alt="Download for Linux"></a>
</p>

The buttons download the latest desktop build in one click.

A CLI is also available for terminal users: `./bin/ai-limits`

## Features

- Shows limits, reset date and time, available tokens, and available manual resets,
- Works with Codex, Claude, and Cursor,
- Retrieves data from local files, provider CLIs, and APIs,
- Falls back to another source when one is unavailable,
- Lightweight desktop app for macOS, Windows, and Linux,
- CLI with several output formats,
- Native system notifications when limits reach configured thresholds,
- Manual and flexible automatic data refresh.

## Alternatives

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| Desktop app and CLI | ✅ | ✅ | ❌ |
| Codex, Claude, and Cursor | ✅ | ✅ | ✅ |
| macOS, Windows, and Linux | ✅ | ❌ | ❌ |
| No intermediary service | ✅ | ✅ | ✅ |

The full comparison covers 18 alternatives and 16 criteria: [alternatives catalog](docs/product/analogues.tsv).

## Platform support and limitations

- macOS: the app is signed and notarized; notifications work,
- Windows and Linux: builds are available; support evolves based on user feedback,
- Desktop notifications are currently available only on macOS,
- Some local Codex and Claude sources may not work on Windows and Linux yet; CLI sources work everywhere.

## License

[MIT License](LICENSE)
