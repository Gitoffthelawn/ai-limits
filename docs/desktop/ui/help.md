# Tauri UI Help Page

## Purpose

The Help page collects short reference sections for the desktop app. Each section fits within the standard window frame without scrolling.

## Layout

The Help page replaces the main content while open. It contains:

- a header with a back button on the left and a centered `Help` title
- a narrow left menu listing the available sections
- a content panel on the right showing the selected section

The back button returns to the main content. `Escape` also closes the page.

On narrow windows the menu stacks above the content panel.

## Sections

Sections are defined once as `HELP_CHAPTERS` in `frontend/index.html`. Each entry provides an `id`, a menu `label`, and rendered content. Adding an entry adds it to the left menu and, on macOS, to the native Help menu.

Current sections, in menu order:

- `about` — what the app does, that it's free, cross-platform, and notification-driven, and where its data comes from.
- `providers` — how each provider (Codex, Claude, Cursor) gets its data, and that visibility is controlled in settings.
- `source-priority` — the Fast, Full, and Best modes, their source chains, the speed/accuracy tradeoff, the provider scope, and the CLI setup guide links. See [controls.md](controls.md).
- `data-errors` — why a provider shows "no fresh data" and what to check, with a link to `source-priority`.
- `notifications` — what triggers a system notification and the current macOS-only limitation.
- `permissions` — the OS-level access the app uses (network, Keychain, local files, notifications, CLI execution) and why.
- `cli-mode` — the tradeoffs of the terminal interface versus the desktop app, the exact command for the running app, and actions to copy or run it in Terminal.
- `limitations` — the current known gaps, mirroring the README limitations list.
- `for-developers` — that the project is MIT-licensed and open source, its stack, and links to GitHub and the license.

Chapters may link to each other via a `data-open-help` button that switches the selected section without leaving the Help page.

Keep this content in sync with the app: when a change affects what a section describes — a setting, a state, a permission, or a link — update the matching chapter in `frontend/index.html` as part of that same change, not as a follow-up.

## Entry Points

The Help page opens from:

- the help button in the [top action row](layout.md), which opens the first section
- the source priority information action in [settings](controls.md) and the no-fresh-data provider state, which open the `source-priority` section
- on macOS, the `Help` menu bar items, which mirror the section list

## macOS Menu Bar

On macOS the native `Help` menu mirrors the section list. See [macos-menu-bar.md](macos-menu-bar.md).

Non-macOS platforms use the in-app entry points only.
