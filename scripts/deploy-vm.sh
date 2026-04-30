#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_env() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "Missing required environment variable: ${name}" >&2
    exit 1
  fi
}

require_env VM_PUBLIC_IP
require_env DEPLOY_SSH_USER
require_env DEPLOY_SSH_PRIVATE_KEY
require_env GHCR_IMAGE_REPOSITORY
require_env AUTOGRAPHS_APP_IMAGE
require_env GITHUB_ACTOR
require_env GHCR_TOKEN

DEPLOY_PATH="${DEPLOY_PATH:-/opt/autographs}"
SSH_KEY_FILE="$(mktemp)"

cleanup() {
  rm -f "$SSH_KEY_FILE"
}

trap cleanup EXIT

printf '%s\n' "$DEPLOY_SSH_PRIVATE_KEY" >"$SSH_KEY_FILE"
chmod 600 "$SSH_KEY_FILE"

SSH_OPTS=(
  -i "$SSH_KEY_FILE"
  -o IdentitiesOnly=yes
  -o StrictHostKeyChecking=accept-new
)

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "mkdir -p '${DEPLOY_PATH}/compose' '${DEPLOY_PATH}/nginx'"
scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/compose/compose.prod.yaml" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:${DEPLOY_PATH}/compose/compose.prod.yaml"
scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/nginx/nginx.conf" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:${DEPLOY_PATH}/nginx/nginx.conf"

printf '%s' "$GHCR_TOKEN" | ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "docker login ghcr.io -u '${GITHUB_ACTOR}' --password-stdin"

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && AUTOGRAPHS_APP_IMAGE='${AUTOGRAPHS_APP_IMAGE}' AUTOGRAPHS_HTTP_PORT='${AUTOGRAPHS_HTTP_PORT:-80}' docker compose -f compose.prod.yaml pull app && AUTOGRAPHS_APP_IMAGE='${AUTOGRAPHS_APP_IMAGE}' AUTOGRAPHS_HTTP_PORT='${AUTOGRAPHS_HTTP_PORT:-80}' docker compose -f compose.prod.yaml up -d"

for _ in $(seq 1 30); do
  if curl --fail --silent "http://${VM_PUBLIC_IP}/health" >/dev/null; then
    curl --fail --silent "http://${VM_PUBLIC_IP}/health"
    exit 0
  fi
  sleep 5
done

echo "Deployment did not pass nginx-fronted /health check" >&2
exit 1
