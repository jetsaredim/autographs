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
AUTOGRAPHS_HTTP_PORT="${AUTOGRAPHS_HTTP_PORT:-80}"
SSH_KEY_FILE="$(mktemp)"

validate_pattern() {
  local name="$1"
  local value="$2"
  local pattern="$3"

  if [[ ! "$value" =~ $pattern ]]; then
    echo "Invalid ${name}: ${value}" >&2
    exit 1
  fi
}

validate_pattern VM_PUBLIC_IP "$VM_PUBLIC_IP" '^[A-Za-z0-9._:-]+$'
validate_pattern DEPLOY_SSH_USER "$DEPLOY_SSH_USER" '^[A-Za-z_][A-Za-z0-9_-]*$'
validate_pattern DEPLOY_PATH "$DEPLOY_PATH" '^/[A-Za-z0-9._/-]+$'
validate_pattern GITHUB_ACTOR "$GITHUB_ACTOR" '^[A-Za-z0-9][A-Za-z0-9-]*$'
validate_pattern AUTOGRAPHS_APP_IMAGE "$AUTOGRAPHS_APP_IMAGE" '^[A-Za-z0-9._:/@-]+$'
validate_pattern AUTOGRAPHS_HTTP_PORT "$AUTOGRAPHS_HTTP_PORT" '^[0-9]+$'

case "$DEPLOY_PATH" in
  /opt/autographs | /opt/autographs/*) ;;
  *)
    echo "DEPLOY_PATH must be /opt/autographs or a child path: ${DEPLOY_PATH}" >&2
    exit 1
    ;;
esac

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

scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/scripts/bootstrap-runtime.sh" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:/tmp/autographs-bootstrap-runtime.sh"
ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "sudo DEPLOY_USER='${DEPLOY_SSH_USER}' DEPLOY_PATH='${DEPLOY_PATH}' bash /tmp/autographs-bootstrap-runtime.sh"

scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/compose/compose.prod.yaml" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:${DEPLOY_PATH}/compose/compose.prod.yaml"
scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/nginx/nginx.conf" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:${DEPLOY_PATH}/nginx/nginx.conf"

printf '%s' "$GHCR_TOKEN" | ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "sudo podman login ghcr.io -u '${GITHUB_ACTOR}' --password-stdin"

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && sudo env AUTOGRAPHS_APP_IMAGE='${AUTOGRAPHS_APP_IMAGE}' AUTOGRAPHS_HTTP_PORT='${AUTOGRAPHS_HTTP_PORT}' podman-compose -f compose.prod.yaml pull && sudo env AUTOGRAPHS_APP_IMAGE='${AUTOGRAPHS_APP_IMAGE}' AUTOGRAPHS_HTTP_PORT='${AUTOGRAPHS_HTTP_PORT}' podman-compose -f compose.prod.yaml up -d"

for _ in $(seq 1 30); do
  if curl --fail --silent "http://${VM_PUBLIC_IP}/health" >/dev/null; then
    curl --fail --silent "http://${VM_PUBLIC_IP}/health"
    exit 0
  fi
  sleep 5
done

echo "Deployment did not pass nginx-fronted /health check" >&2
exit 1
