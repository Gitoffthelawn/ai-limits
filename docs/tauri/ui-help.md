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

Current sections:

- `source-priority` — the Fast, Full, and Best modes, their source chains, the speed/accuracy tradeoff, the provider scope, and the CLI setup guide links. See [ui-controls.md](ui-controls.md).

## Entry Points

The Help page opens from:

- the help button in the [top action row](ui-layout.md), which opens the first section
- the source priority information action in [settings](ui-controls.md) and the no-fresh-data provider state, which open the `source-priority` section
- on macOS, the `Help` menu bar items, which mirror the section list

## macOS Menu Bar

On macOS the native `Help` menu mirrors the section list. See [ui-macos-menu-bar.md](ui-macos-menu-bar.md).

Non-macOS platforms use the in-app entry points only.
