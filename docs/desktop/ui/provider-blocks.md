# Tauri UI Provider Blocks

## Block Structure

Each provider square contains:

- provider name
- limit rows
- credits line, when available
- manual limit-reset availability, when available
- source line with data origin label and timestamp on one line by default
- update frequency dropdown near the bottom
- provider-specific manual update button at the bottom

Provider names follow the shared [provider naming rules](../../presentation/provider-names.md).

The provider content should roughly match the current terminal output model.

Example data shape:

```text
     --------- CURSOR --------
plan | 54.6% left
■■■■■■■■■■■■□□□
reset Jul 28, 03:00
auto | 63.7% left
■■■■■■■■■■■■■■□□□□
api | 24.5% left
■■■■■□□□□□□□□□□□□□□□
API2, as of Jul 5, 19:28

     --------- CODEX ---------
5h | 92.0% left
■■■■■■■■■■■■■■■■■■■■■■■□□
reset 20:48
7d | 35.0% left
■■■■■■■■■□□□□□□□□□□□□□□□□
reset Jul 10, 03:55
Available credits: 344.2
Local files, as of Jul 5, 19:28

     --------- CLAUDE --------
5h | 100.0% left
■■■■■■■■■■■■■■■■■■■■■■■■■
reset Jul 6, 00:20
7d | 84.0% left
■■■■■■■■■■■■■■■■■■■■□□□□
reset Jul 7, 13:00
CLI, as of Jul 5, 19:29
```

The UI does not need to use terminal-style ASCII rendering. The example defines the information that must be visible.

Accent color rules are documented in [provider-block-colors.md](provider-block-colors.md); limit row, credits, and source line content are documented in [provider-block-content.md](provider-block-content.md).
