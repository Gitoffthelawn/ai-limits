#!/usr/bin/env sh
set -eu

PORT="${AI_LIMITS_TAURI_DEV_PORT:-1420}"
HOST="${AI_LIMITS_TAURI_DEV_HOST:-127.0.0.1}"
PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

sh "$PROJECT_ROOT/scripts/stop-frontend-http-server.sh"

exec python3 -m http.server "$PORT" --bind "$HOST" --directory "$PROJECT_ROOT/frontend"
