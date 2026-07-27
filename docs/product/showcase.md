# Browser Screenshot Showcase

The browser showcase renders the production desktop frontend with fixed demonstration data and a decorative macOS Tahoe, Windows, or Linux window frame. It is for creating screenshots only; it does not connect to Tauri, Rust, provider APIs, or local accounts.

From the project root, run `python3 -m http.server 1420 --bind 127.0.0.1 --directory frontend`, then open `http://127.0.0.1:1420/?showcase=macos`. Replace `macos` with `windows` or `linux`, or use the platform controls beside the frame.

The fixed data is defined with the frontend so its shape follows the rendered provider cards. Update it only when a new screenshot needs different content or the displayed data model changes.

The frame starts at a compact size and can be resized by dragging its invisible bottom-right corner. The three platform controls change only the decorative window chrome.
