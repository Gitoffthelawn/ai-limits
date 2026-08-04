# Changelog

This file records user-visible changes. Its version sections are used automatically for Git tags and GitHub Release notes.

## Unreleased

- Removed the Fast/Full/Best source-priority setting: the app always queries providers over RPC first now, falling back to local data only when RPC isn't available. The setting existed to trade freshness for speed against the old CLI text-interface path, but RPC made that path fast enough that the tradeoff no longer bought anything.
- Split each provider card's source line into two: a `Source` line (for example `RPC, as of 10:10.`) and an update-time line (`Last upd hh:mm, next hh:mm`, or `Manual only` when a provider has no scheduled refresh). Both are new Display settings — `Source` is off by default, `Update time` is on by default.
- Renamed the Display settings `Show limits` and `Show plan` to `Limits` and `Subscription`.
- Moved each provider card's update-frequency picker into its own gear-button dropdown next to `UPDATE NOW`, opening upward within the card instead of a permanent `Upd every` row.
- Removed the transient provider refresh status overlay from the desktop cards, and kept the Limits and Plan slots aligned across the currently visible cards with top-packed content and reserved blank space for empty sections.
- Added a "100% again" notification: alerts when a provider's remaining limit returns to exactly 100% after having been lower, in addition to the existing low-remaining alerts. A first reading or a partial rise (for example 40% to 97%) does not notify.
- Codex limits now come from the Codex CLI's app-server, which answers in seconds instead of driving the CLI's text interface. Codex reports exact reset times, the plan tier, the credit balance, available limit resets, and the lifetime token total.
- Claude limits read through the CLI now come from a direct usage request instead of driving the CLI's text interface: they arrive in about two seconds, without a terminal emulator, and without using up any of the account's quota. Claude now reports the plan tier, exact reset times, the extra usage allowance, and the spend for the current period.
- Cursor now reports the plan name and price, the renewal date, the included spend allowance, the token breakdown and total, and the session, turn, and event counts for the current billing cycle, alongside the usage percentages it already showed.
- Claude limits read without the CLI now come from the snapshot the Claude app saves when you open `/usage`, so the 5-hour and 7-day percentages and reset times are the ones Claude itself reports instead of a local estimate. The snapshot is shown with the time it was taken, because it only refreshes when `/usage` is opened. Claude also reports the plan, the subscription start date, and the usage credit spend, and falls back to Claude's own usage totals when no transcripts are left on the machine.
- Paths shown in output, raw data, and the Help panel's command now start at `~` instead of spelling out your home directory.
- Simplified the macOS menu bar: dropped the unused File and View menus, and replaced the app menu's Services item with a Settings… item that opens the desktop app's settings panel.
- Added the app version to Help > About.
- Fixed the macOS install disk image shipping without its styled window: the background and Applications-folder shortcut were silently missing since the first release with a disk image, because arranging them needed a Finder permission unavailable in GitHub Actions.

## [v0.2.0](https://github.com/md2it/ai-limits/releases/tag/v0.2.0) — 2026-08-03

- macOS installations now update themselves: the app checks for a new release at startup and every 12 hours, downloads it in the background, and offers a restart once it is ready. Automatic updates can be turned off in Settings.
- Gave the macOS application icon the rounded shape the platform expects, leaving the Windows and Linux icons edge to edge.

## [v0.1.0](https://github.com/md2it/ai-limits/releases/tag/v0.1.0) — 2026-08-02

- macOS is now installed from a signed and notarized disk image; the application bundle archive is no longer published as a separate download.
- Added a thin gray border to browser showcase window frames so README screenshots stay visible on light and dark backgrounds.
- Refreshed README showcase screenshots from the current UI.

## [v0.0.15](https://github.com/md2it/ai-limits/releases/tag/v0.0.15) — 2026-08-02

- Fixed Codex CLI authorization detection when the CLI writes its signed-in status to diagnostic output.
- Enabled a restrictive Content Security Policy for the desktop app.

## [v0.0.14](https://github.com/md2it/ai-limits/releases/tag/v0.0.14) — 2026-07-30

- Clarified desktop Help copy about privacy, source priority, and unavailable limit data.
- When Codex CLI or Claude CLI is installed but not signed in, show a clear authorization message with the manual login command; the desktop app also offers an explicit Sign in action that starts login only after the user chooses it.
- Renamed desktop credit and reset availability labels and gave both lines consistent styling.
- Added a browser-only screenshot showcase with macOS, Windows, and Linux window frames.
- Rejects outdated local Codex and Claude limit snapshots after their expected reset time and uses the configured fallback source when available.
- Added the exact local CLI command to Help → CLI mode, with Copy and Run in Terminal actions.
- Disabled automatic desktop app rebuilds after code changes; development builds now run only on explicit command.
- Added a desktop Help page with a sidebar of sections, reached from a new info button in the top action row and, on macOS, from the Help menu bar; moved the source priority explanation there and removed its modal.
- Made the CLI stateless: one query per run, built-in defaults, no config file or watch mode.
- Exposed available Codex limit resets in the terminal and desktop UI.
- Removed the unsupported Claude statusline source and setup.

## [v0.0.13](https://github.com/md2it/ai-limits/releases/tag/v0.0.13) — 2026-07-08

- add desktop source priority modes Fast, Full, and Best
- UI improvements: rewritings, relayout modal window items, grouped the settings
- preserve macOS signing through release zip round-trip
- enforce local commit prefixes
- add contributor release tooling

## [v0.0.12](https://github.com/md2it/ai-limits/releases/tag/v0.0.12) — 2026-07-08

Highlights:
- Fixed macOS signing and notarization preservation through the release zip round-trip: removed `ditto --sequesterRsrc`, added shared verification script, and verify the archived artifact after extract so stapled tickets survive download.
- Added contributor release tooling: semver-based pre-releases, commit message checks, and contributor setup scripts.
- Centralized desktop OS permission handling and tightened macOS entitlements (removed JIT/unsigned-memory/library-validation allowances).
- Decoupled release version from the Tauri app version; this release is tagged `v0.0.12`.
- Updated the provider status table and refreshed product documentation.

## [desktop-unstable-8-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-8-1) — 2026-07-07

Desktop build, notification, and UI polish update.

- Added Apple notarization support for macOS desktop builds.
- Improved macOS build scripts and release workflow documentation.
- Routed desktop notifications through Tauri.
- Unified user-facing time display across the desktop UI.
- Improved provider card layout and source labels.
- Updated the provider status table.
- Refined DevOps and desktop build documentation.

## [desktop-unstable-5-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-5-1) — 2026-07-06

Desktop beta usability and provider display update.

- Improved the Tauri desktop UI theme.
- Added remaining credits to provider cards.
- Improved local CLI discovery through configured PATH handling.

## [desktop-unstable-4-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-4-1) — 2026-07-06

Initial unstable desktop pre-release of AI Limits.

- Added the first Tauri desktop app connected to the core limits engine.
- Shows usage limits, reset times, provider status, and data sources in a desktop UI.
- Added manual refresh, loading states, and per-provider refresh controls.
- Added provider settings, notification settings, and CLI fallback controls.
- Added system notifications for limit updates.
- Added local provider integrations for Codex, Claude, and Cursor usage data.
- Added CLI watch mode.
- Added desktop app icons, application logo assets, and the initial desktop build workflow.
- Updated documentation for desktop builds, release flow, smoke testing, and beta downloads.
