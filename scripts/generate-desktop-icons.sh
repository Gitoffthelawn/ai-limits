#!/bin/sh

set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
icon_dir="$project_dir/src-tauri/icons"
master_icon="$icon_dir/icon-master.svg"
desktop_icon="$icon_dir/icon-desktop.svg"
tauri_cli="$project_dir/node_modules/.bin/tauri"
temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/ai-limits-icons.XXXXXX")

cleanup() {
  rm -rf "$temporary_dir"
}

trap cleanup EXIT HUP INT TERM

if [ ! -x "$tauri_cli" ]; then
  printf '%s\n' "Tauri CLI is unavailable. Run npm install first." >&2
  exit 1
fi

if [ ! -f "$master_icon" ]; then
  printf '%s\n' "Master icon is unavailable: $master_icon" >&2
  exit 1
fi

{
  printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="AI Limits desktop icon source">'
  printf '%s\n' '  <rect width="24" height="24" fill="#000000"/>'
  printf '%s\n' '  <g transform="translate(2.4 2.4) scale(0.8)">'
  sed '1d;$d' "$master_icon" | sed '/^[[:space:]]*$/d; s/[[:space:]]*$//' | sed 's/^/    /'
  printf '%s\n' '  </g>'
  printf '%s\n' '</svg>'
} > "$desktop_icon"

"$tauri_cli" icon "$desktop_icon" --output "$temporary_dir"
find "$temporary_dir" -maxdepth 1 -type f -exec cp {} "$icon_dir" \;
"$tauri_cli" icon "$desktop_icon" --output "$temporary_dir/large" --png 1024
cp "$temporary_dir/large/1024x1024.png" "$icon_dir/icon-1024.png"
