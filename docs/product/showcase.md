# Browser Screenshot Showcase

The browser showcase renders the production desktop frontend with fixed demonstration data and a decorative macOS Tahoe, Windows, or Linux window frame. It is for creating screenshots only; it does not connect to Tauri, Rust, provider APIs, or local accounts.

From the project root, run `python3 -m http.server 1420 --bind 127.0.0.1 --directory frontend`, then open `http://127.0.0.1:1420/?showcase=macos`. Replace `macos` with `windows` or `linux`, or use the platform controls beside the frame.

To replace the README images in `docs/readmes/screenshots/` in one step, run `npm run screenshots:update` (or `sh scripts/update-showcase-screenshots.sh`). The script starts the local frontend server, captures the five showcase shots (including settings and help), overwrites those PNG files, and stops the server when finished.

The fixed data is defined in `frontend/modules/showcase.js` so its shape follows the rendered provider cards. Update it only when a new screenshot needs different content or the displayed data model changes.

The frame starts at a compact size and can be resized by dragging its invisible bottom-right corner. The three platform controls change only the decorative window chrome. Showcase windows use a 1px gray border so README screenshots stay visible on light and dark GitHub backgrounds.

## Screenshot Requirements

Fixed rules for `scripts/update-showcase-screenshots.mjs`, so future updates don't have to re-derive them:

- Display toggles: `showPlan`, `showSource`, and `showUpdateTime` are OFF (only `showLimits` is on). Cards must show limit bars only, no PLAN section, no `Source ...` line, no `Last upd ...` line.
- Viewport `1400x900`, `deviceScaleFactor: 1`.
- Theme: `dark` for `macos.png`, `windows.png`, `linux.png`, `macos-help.png`; `light` for `macos-light-settings.png`.
- Window frame is resized to `1010px` wide with `box-shadow: none` before capture; page background is made transparent.
- The capture target is `.showcase-window` with `omitBackground: true` (PNG keeps transparency outside the frame).
- Five fixed shots: `macos`, `windows`, `linux` (plain), `macos-light-settings` (settings dropdown open), `macos-help` (help view open).
- Mock data always comes from `frontend/modules/showcase.js`; never hand-edit a screenshot or hardcode different numbers per shot.
