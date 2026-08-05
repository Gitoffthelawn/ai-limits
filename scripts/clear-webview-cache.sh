#!/usr/bin/env sh
# Clears this app's on-disk WebKit cache before a local dev/debug run.
#
# Unsigned local builds (dev server, `tauri:build:debug`) get a new ad-hoc
# code-signing identity on every rebuild (no stable Team ID), so macOS can't
# reliably key the WebKit website-data store by CFBundleIdentifier the way it
# does for a proper Developer-ID-signed release. In practice that leaves two
# separate cache containers on disk:
#   - ~/Library/WebKit/com.ai-limits.desktop      (CFBundleIdentifier, tauri.conf.json)
#   - ~/Library/WebKit/ai-limits-desktop           (Cargo binary name, no dots)
# and their ~/Library/Caches counterparts. Either can silently serve stale
# HTML/CSS/JS from a previous run — content edits stop appearing in the
# popover/main window until this is cleared. Wiping both up front makes every
# local run start from a clean webview, at the cost of that window's
# localStorage (theme/settings) resetting to defaults.
#
# Best-effort only: never fails the calling npm script. macOS-only; a no-op
# elsewhere, since these paths and the identity-per-rebuild issue are both
# macOS/WebKit-specific.
set -u

if [ "$(uname -s)" != "Darwin" ]; then
  exit 0
fi

BUNDLE_ID="com.ai-limits.desktop"
BINARY_NAME="ai-limits-desktop"

echo "==> Clearing local WebKit cache for $BUNDLE_ID / $BINARY_NAME"

clear_dir() {
  dir="$1"
  if [ ! -e "$dir" ]; then
    echo "    skip (absent): $dir"
    return 0
  fi
  if rm -rf "$dir" 2>/dev/null; then
    echo "    removed: $dir"
  else
    echo "    WARNING: failed to remove $dir (continuing anyway)"
  fi
}

clear_dir "$HOME/Library/WebKit/$BUNDLE_ID"
clear_dir "$HOME/Library/Caches/$BUNDLE_ID"
clear_dir "$HOME/Library/WebKit/$BINARY_NAME"
clear_dir "$HOME/Library/Caches/$BINARY_NAME"

exit 0
