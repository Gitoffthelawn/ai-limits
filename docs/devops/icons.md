# Desktop Icon Generation

## Rules

- The master artwork has no internal padding.
- A platform source adds the dark-theme `#111214` background and desktop-safe padding.
- All desktop platform assets must be derived from a platform source, never directly from the master artwork.
- Icon shape follows the platform. Windows and Linux fill the canvas edge to edge. macOS draws its own rounded shape, because the system renders an application icon exactly as authored and does not round it.
- Android and iOS icon folders are outside this desktop application scope.

Run `npm run icons:generate` from the project root after changing [the master source](../../src-tauri/icons/icon-master.svg). The command derives two sources and generates the PNG, ICO, ICNS, and Windows logo assets in `src-tauri/icons`:

- `icon-desktop.svg` — the full-canvas background with a 10% inset, used for every asset;
- `icon-macos.svg` — a rounded plate of 824 on a 1024 canvas with a 185.4 corner radius, following the macOS icon grid, used only for `icon.icns`.

The current packaged icon list is defined in [the Tauri configuration](../../src-tauri/tauri.conf.json).
