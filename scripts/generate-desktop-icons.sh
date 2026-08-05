#!/bin/sh

set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
icon_dir="$project_dir/src-tauri/icons"
master_icon="$icon_dir/icon-master.svg"
desktop_icon="$icon_dir/icon-desktop.svg"
macos_icon="$icon_dir/icon-macos.svg"
tray_icon="$icon_dir/icon-tray.svg"
tray_png="$icon_dir/icon-tray.png"
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

# Emit an icon source: a background plate, then the master artwork placed on it.
# Arguments: label, plate attributes, artwork transform, output path.
write_icon_source() {
  {
    printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="'"$1"'">'
    printf '%s\n' '  <rect '"$2"' fill="#111214"/>'
    printf '%s\n' '  <g transform="'"$3"'">'
    sed '1d;$d' "$master_icon" | sed '/^[[:space:]]*$/d; s/[[:space:]]*$//' | sed 's/^/    /'
    printf '%s\n' '  </g>'
    printf '%s\n' '</svg>'
  } > "$4"
}

# Windows and Linux fill the icon canvas edge to edge.
write_icon_source \
  "AI Limits desktop icon source" \
  'width="24" height="24"' \
  'translate(2.4 2.4) scale(0.8)' \
  "$desktop_icon"

# macOS shapes the icon itself: a rounded plate on 824 of a 1024 canvas with a
# 185.4 corner radius, expressed here in the 24-unit viewBox. The artwork keeps
# the same 10% inset relative to the plate.
write_icon_source \
  "AI Limits macOS icon source" \
  'x="2.34375" y="2.34375" width="19.3125" height="19.3125" rx="4.3453125"' \
  'translate(4.275 4.275) scale(0.64375)' \
  "$macos_icon"

"$tauri_cli" icon "$desktop_icon" --output "$temporary_dir"
find "$temporary_dir" -maxdepth 1 -type f -exec cp {} "$icon_dir" \;
"$tauri_cli" icon "$desktop_icon" --output "$temporary_dir/large" --png 1024
cp "$temporary_dir/large/1024x1024.png" "$icon_dir/icon-1024.png"

# Only the ICNS carries the macOS shape; every other asset stays edge to edge.
"$tauri_cli" icon "$macos_icon" --output "$temporary_dir/macos"
cp "$temporary_dir/macos/icon.icns" "$icon_dir/icon.icns"

# macOS menu bar icon: a template image, so it must be a single monochrome,
# alpha-only shape that AppKit tints itself for light/dark menu bars and for
# the highlighted (menu-open) state. Built from the same master artwork with
# the per-arc brand colors stripped and one uniform stroke weight; the master's
# fifth path (the doubled accent arc) is dropped because it is invisible once
# the artwork is monochrome at menu bar size.
#
# Geometry: a 24-unit viewBox rendered at 18pt, so 1 unit = 0.75pt. The artwork
# is scaled to 0.80 about the center to leave optical margin, and the 2.35
# stroke lands at ~1.4pt — the SF Symbols-like weight the menu bar expects.
{
  printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2.35" stroke-linecap="round" stroke-linejoin="round" role="img" aria-label="AI Limits menu bar template icon">'
  printf '%s\n' '  <g transform="translate(12 12) scale(0.80) translate(-12 -12)">'
  sed '1d;$d' "$master_icon" |
    grep '<path' |
    sed 's/ stroke="#[0-9a-fA-F]*"//; s/ stroke-width="[0-9.]*"//; s/^[[:space:]]*/    /' |
    sed '5,$d'
  printf '%s\n' '  </g>'
  printf '%s\n' '</svg>'
} > "$tray_icon"

# Rasterized at 36px = 18pt @2x. The tray API takes exactly one image and
# rescales it to an 18pt height, and macOS has no @3x displays, so @2x is the
# only size worth shipping.
node "$project_dir/scripts/render-svg-png.mjs" "$tray_icon" "$tray_png" 36
