# Changelog

This file records user-visible changes. Its version sections are used automatically for Git tags and GitHub Release notes.

## Unreleased

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
