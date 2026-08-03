#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Build a styled macOS disk image without relying on Finder automation.

Finder's "make Finder stuff pretty" AppleScript step (used by Tauri's own
dmg bundler) requires the Automation permission to control Finder. That
permission is not available in GitHub Actions runners, nor is it reliably
available in interactive terminals until manually granted, so it fails
silently and ships an unstyled disk image. This script writes the window
layout directly instead, which needs no such permission.

Usage:
  scripts/build-macos-dmg.sh <path-to.app> <output.dmg>

The window layout, icon positions, and background image are defined in
src-tauri/dmg/settings.py.
EOF
}

if [[ $# -ne 2 || "$1" == "-h" || "$1" == "--help" ]]; then
  usage >&2
  exit 1
fi

APP_PATH="$1"
OUTPUT_PATH="$2"
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SETTINGS="$PROJECT_DIR/src-tauri/dmg/settings.py"

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle not found: $APP_PATH" >&2
  exit 1
fi

if ! python3 -c "import dmgbuild" >/dev/null 2>&1; then
  echo "Installing dmgbuild..." >&2
  python3 -m pip install --user --quiet dmgbuild
fi

rm -f "$OUTPUT_PATH"
mkdir -p "$(dirname "$OUTPUT_PATH")"

DMG_APP_PATH="$APP_PATH" \
DMG_BACKGROUND_PATH="$PROJECT_DIR/src-tauri/dmg/background.png" \
  python3 -m dmgbuild -s "$SETTINGS" "AI Limits" "$OUTPUT_PATH"

echo "Built $OUTPUT_PATH"
