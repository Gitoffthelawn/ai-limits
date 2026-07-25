# Desktop Icon Generation

Status: active.

Icon source files:

```text
src-tauri/icons/icon-master.svg
src-tauri/icons/icon-desktop.svg
```

Rules:

- `icon-master.svg` is the master artwork. It intentionally has no internal padding.
- `icon-desktop.svg` is the desktop icon source derived from the master artwork. It keeps the black background and adds desktop-safe internal padding.
- Desktop PNG, `.ico`, and `.icns` files must be generated from `icon-desktop.svg`, not directly from `icon-master.svg`.
- Android and iOS icon folders are not part of this desktop application.

Required local tools:

```text
qlmanage
magick
iconutil
sips
```

Tool roles:

- Use macOS QuickLook through `qlmanage` to render SVG into PNG. This is the confirmed renderer for the current SVG artwork.
- Do not use ImageMagick as the SVG renderer for this icon. During verification, ImageMagick rendered the small sparkles but dropped the main logo shape.
- Use ImageMagick only to assemble `icon.ico` from already rendered PNG files.
- Use a direct ICNS container build from already rendered PNG files, then verify the result with `iconutil`.
- Use `sips` for dimension checks.

Expected output files, padding configuration, and verified build behavior are documented in [icons-files.md](icons-files.md).
