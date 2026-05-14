#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/deploy/compose/compose.prod.yaml"
PROJECT_NAME="autographs-runtime"
HEALTH_URL="http://127.0.0.1:${AUTOGRAPHS_HTTP_PORT:-8080}/health"

export AUTOGRAPHS_DOMAIN="${AUTOGRAPHS_DOMAIN:-localhost}"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required to validate the runtime topology" >&2
  exit 1
fi

cleanup() {
  docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" down --remove-orphans >/dev/null 2>&1 || true
}

trap cleanup EXIT

docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" up -d --build

for _ in $(seq 1 30); do
  if curl --fail --silent "$HEALTH_URL" >/dev/null; then
    curl --fail --silent "$HEALTH_URL"
    exit 0
  fi
  sleep 2
done

echo "runtime health check did not succeed via Caddy" >&2
docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" logs >&2 || true
exit 1
