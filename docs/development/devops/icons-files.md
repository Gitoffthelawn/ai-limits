# Desktop Icon Files

Expected desktop icon files:

```text
src-tauri/icons/32x32.png
src-tauri/icons/64x64.png
src-tauri/icons/128x128.png
src-tauri/icons/128x128@2x.png
src-tauri/icons/icon.png
src-tauri/icons/icon-1024.png
src-tauri/icons/icon.ico
src-tauri/icons/icon.icns
```

Windows logo PNG files in `src-tauri/icons/Square*Logo.png` and `src-tauri/icons/StoreLogo.png` are also generated from `icon-desktop.svg`.

Current desktop icon padding:

```text
source: src-tauri/icons/icon-desktop.svg
canvas: 1024x1024
background: black
artwork scale: 80%
internal padding: about 10% per side
```

Verified behavior:

- Local Tauri macOS build copies `src-tauri/icons/icon.icns` into the `.app` bundle without changing it.
- GitHub Actions macOS build also copied `icon.icns` without changing it.
- GitHub Actions Linux `.deb` package copied the checked PNG files without changing them.
- Therefore desktop icon padding is controlled by the source icon files, not by GitHub Actions or Tauri at build time.
