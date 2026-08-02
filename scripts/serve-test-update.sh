#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Serve a locally built update so the auto-updater can be exercised without
publishing a GitHub release.

Usage:
  scripts/serve-test-update.sh <version> [port]

Arguments:
  version  Version the served update announces, for example 0.3.0. It must be
           higher than the version of the app you start, or nothing is offered.
  port     Port for the local manifest server (default 8787).

Before running this, build the newer version with the updater artifacts and the
same signing key the app was built against:

  TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/ai-limits-updater.key)" \
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
    npm exec tauri -- build --bundles app --target universal-apple-darwin

Then point the app you want to update at this server by temporarily replacing
the updater endpoint in src-tauri/tauri.conf.json with:

  http://127.0.0.1:8787/latest.json

and rebuild that older app. Its plugins.updater.pubkey must stay the public key
matching the signing key above, or the download is rejected.
EOF
}

if [[ $# -lt 1 || "${1:-}" == "--help" ]]; then
  usage
  exit 1
fi

version="$1"
port="${2:-8787}"

project_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$project_root"

bundle_dir="target/universal-apple-darwin/release/bundle/macos"
archive="$(find "$bundle_dir" -maxdepth 1 -name '*.app.tar.gz' -print -quit 2>/dev/null || true)"
signature="$(find "$bundle_dir" -maxdepth 1 -name '*.app.tar.gz.sig' -print -quit 2>/dev/null || true)"

if [[ -z "$archive" || -z "$signature" ]]; then
  echo "No updater artifacts in $bundle_dir. Build them first; see --help." >&2
  exit 1
fi

serve_dir="$(mktemp -d)"
trap 'rm -rf "$serve_dir"' EXIT

cp "$archive" "$serve_dir/update.app.tar.gz"

VERSION="$version" PORT="$port" SIGNATURE_PATH="$signature" SERVE_DIR="$serve_dir" python3 - <<'PY'
from pathlib import Path
import json
import os

signature = Path(os.environ["SIGNATURE_PATH"]).read_text().strip()
macos = {
    "signature": signature,
    "url": f"http://127.0.0.1:{os.environ['PORT']}/update.app.tar.gz",
}
manifest = {
    "version": os.environ["VERSION"],
    "notes": "Local test update.",
    "pub_date": "2000-01-01T00:00:00Z",
    "platforms": {
        "darwin-universal": macos,
        "darwin-aarch64": macos,
        "darwin-x86_64": macos,
    },
}
Path(os.environ["SERVE_DIR"], "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")
PY

echo "Serving update $version on http://127.0.0.1:$port/latest.json (Ctrl+C to stop)."
python3 -m http.server "$port" --bind 127.0.0.1 --directory "$serve_dir"
