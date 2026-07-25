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

The available sections are listed in [help-sections.md](help-sections.md).

## Entry Points

The Help page opens from:

- the help button in the [top action row](layout.md), which opens the first section
- the source priority information action in [settings](controls.md) and the no-fresh-data provider state, which open the `source-priority` section
- on macOS, the `Help` menu bar items, which mirror the section list

## macOS Menu Bar

On macOS the native `Help` menu mirrors the section list. See [macos-menu-bar.md](macos-menu-bar.md).

Non-macOS platforms use the in-app entry points only.
