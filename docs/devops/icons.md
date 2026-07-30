# Desktop Icon Generation

## Rules

- The master artwork has no internal padding.
- The desktop source adds the dark-theme `#111214` background and desktop-safe padding.
- All desktop platform assets must be derived from the desktop source, never directly from the master artwork.
- Android and iOS icon folders are outside this desktop application scope.

Run `npm run icons:generate` from the project root after changing [the master source](../../src-tauri/icons/icon-master.svg). The command derives `icon-desktop.svg` with the dark-theme `#111214` background and a 10% inset, then generates the PNG, ICO, ICNS, and Windows logo assets in `src-tauri/icons`.

The current packaged icon list is defined in [the Tauri configuration](../../src-tauri/tauri.conf.json).
