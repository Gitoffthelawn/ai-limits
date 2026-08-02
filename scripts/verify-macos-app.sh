#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Verify a signed macOS app bundle, release zip, or release disk image.

Usage:
  scripts/verify-macos-app.sh [--notarization MODE] <path-to.app-or.zip-or.dmg>

Modes:
  full         (default) expect stapled notarization ticket
  submit-only  signed; notarization may still be in progress at Apple
  sign-only    signed only; Gatekeeper may warn until notarized

A disk image is verified as a distributable in its own right and the app
bundle it carries is verified from the mounted image.

Examples:
  scripts/verify-macos-app.sh "AI Limits.app"
  scripts/verify-macos-app.sh --notarization full "AI Limits.app.zip"
  scripts/verify-macos-app.sh --notarization full "AI Limits.dmg"
EOF
}

NOTARIZATION_MODE="full"
TARGET=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --notarization)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --notarization" >&2
        usage >&2
        exit 1
      fi
      NOTARIZATION_MODE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ -n "$TARGET" ]]; then
        echo "Unexpected argument: $1" >&2
        usage >&2
        exit 1
      fi
      TARGET="$1"
      shift
      ;;
  esac
done

if [[ -z "$TARGET" ]]; then
  usage >&2
  exit 1
fi

case "$NOTARIZATION_MODE" in
  full|submit-only|sign-only) ;;
  *)
    echo "Unknown notarization mode: $NOTARIZATION_MODE" >&2
    exit 1
    ;;
esac

if [[ ! -e "$TARGET" ]]; then
  echo "Path does not exist: $TARGET" >&2
  exit 1
fi

EXTRACT_DIR=""
MOUNT_POINT=""
cleanup() {
  if [[ -n "$MOUNT_POINT" && -d "$MOUNT_POINT" ]]; then
    hdiutil detach "$MOUNT_POINT" -quiet || hdiutil detach "$MOUNT_POINT" -force -quiet || true
  fi
  if [[ -n "$EXTRACT_DIR" && -d "$EXTRACT_DIR" ]]; then
    rm -rf "$EXTRACT_DIR"
  fi
}
trap cleanup EXIT

verify_app() {
  local app_path="$1"

  echo "Verifying macOS app: $app_path"

  codesign -dv "$app_path"
  codesign -d --entitlements - "$app_path"
  codesign --verify --deep --strict --verbose=4 "$app_path"

  case "$NOTARIZATION_MODE" in
    full)
      spctl --assess --type execute -vv "$app_path"
      xcrun stapler validate "$app_path"
      echo "macOS app is signed, notarized, and stapled."
      ;;
    submit-only)
      echo "Signed app verified. Notarization was submitted without waiting."
      echo "Check status with:"
      echo "  xcrun notarytool history --key ... --key-id ... --issuer ..."
      echo "After Accepted, staple locally or rerun workflow with macos_notarization=full:"
      echo "  xcrun stapler staple \"$app_path\""
      ;;
    sign-only)
      echo "Signed only. Gatekeeper may warn until the app is notarized."
      ;;
  esac
}

verify_dmg() {
  local dmg_path="$1"

  echo "Verifying macOS disk image: $dmg_path"

  codesign -dv "$dmg_path"
  codesign --verify --strict --verbose=4 "$dmg_path"

  case "$NOTARIZATION_MODE" in
    full)
      spctl --assess --type open --context context:primary-signature -vv "$dmg_path"
      xcrun stapler validate "$dmg_path"
      echo "macOS disk image is signed, notarized, and stapled."
      ;;
    submit-only)
      echo "Signed disk image verified. Notarization was submitted without waiting."
      ;;
    sign-only)
      echo "Disk image signed only. Gatekeeper may warn until it is notarized."
      ;;
  esac

  MOUNT_POINT="$(mktemp -d)"
  hdiutil attach "$dmg_path" -nobrowse -readonly -mountpoint "$MOUNT_POINT" -quiet

  local app_path
  app_path="$(find "$MOUNT_POINT" -maxdepth 1 -name '*.app' -print -quit)"
  if [[ -z "$app_path" ]]; then
    echo "No .app bundle found in disk image: $dmg_path" >&2
    exit 1
  fi

  verify_app "$app_path"
}

if [[ "$TARGET" == *.dmg ]]; then
  verify_dmg "$TARGET"
elif [[ "$TARGET" == *.zip ]]; then
  EXTRACT_DIR="$(mktemp -d)"
  ditto -x -k "$TARGET" "$EXTRACT_DIR"
  APP_PATH="$(find "$EXTRACT_DIR" -maxdepth 1 -name '*.app' -print -quit)"
  if [[ -z "$APP_PATH" ]]; then
    echo "No .app bundle found in zip: $TARGET" >&2
    exit 1
  fi
  verify_app "$APP_PATH"
elif [[ "$TARGET" == *.app ]]; then
  verify_app "$TARGET"
else
  echo "Expected a .app bundle, .zip archive, or .dmg disk image: $TARGET" >&2
  exit 1
fi
