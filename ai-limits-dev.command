#!/bin/zsh
# macOS Finder launcher: delegates to the documented Tauri development command.
set -euo pipefail

cd "$(dirname "$0")"
exec npm run tauri:dev
