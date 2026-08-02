#!/usr/bin/env sh
# Regenerate README showcase screenshots under docs/readmes/screenshots/, then stop the local server.
set -eu

PORT="${AI_LIMITS_TAURI_DEV_PORT:-1420}"
HOST="${AI_LIMITS_TAURI_DEV_HOST:-127.0.0.1}"
PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
SERVER_PID=""

cleanup() {
  if [ -n "${SERVER_PID}" ]; then
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
    SERVER_PID=""
  fi
  sh "$PROJECT_ROOT/scripts/stop-frontend-http-server.sh" || true
}

trap cleanup EXIT INT TERM

sh "$PROJECT_ROOT/scripts/stop-frontend-http-server.sh"
python3 -m http.server "$PORT" --bind "$HOST" --directory "$PROJECT_ROOT/frontend" &
SERVER_PID=$!
sleep 0.2
if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
  SERVER_PID=""
  echo "Failed to start showcase server on http://${HOST}:${PORT}/" >&2
  exit 1
fi

ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if python3 - "$HOST" "$PORT" <<'PY'
import socket, sys
host, port = sys.argv[1], int(sys.argv[2])
try:
    with socket.create_connection((host, port), timeout=0.25):
        raise SystemExit(0)
except OSError:
    raise SystemExit(1)
PY
  then
    ready=1
    break
  fi
  sleep 0.15
done

if [ "$ready" -ne 1 ]; then
  echo "Showcase server did not become ready on http://${HOST}:${PORT}/" >&2
  exit 1
fi

cd "$PROJECT_ROOT"
if [ ! -d "$PROJECT_ROOT/node_modules/playwright" ]; then
  npm install
fi
npx playwright install chromium
node "$PROJECT_ROOT/scripts/update-showcase-screenshots.mjs"
