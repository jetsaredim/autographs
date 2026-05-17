#!/usr/bin/env bash

set -euo pipefail

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

DEPLOY_PATH="${DEPLOY_PATH:-/opt/autographs}"
AUTOGRAPHS_SMOKE_BASE_URL="${AUTOGRAPHS_SMOKE_BASE_URL:-https://autographs.jetsaredim.net}"
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
validate_pattern AUTOGRAPHS_SMOKE_BASE_URL "$AUTOGRAPHS_SMOKE_BASE_URL" '^https?://[A-Za-z0-9._:/?=&%-]+$'

if [[ ! "$DEPLOY_PATH" =~ ^/opt/autographs(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$ ]]; then
  echo "DEPLOY_PATH must be /opt/autographs or a safe child path: ${DEPLOY_PATH}" >&2
  exit 1
fi

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

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && \
   sudo podman run --rm \
     --network podman-compose_default \
     --env-file .env \
     -v '${DEPLOY_PATH}/wallet:/opt/autographs/wallet:ro' \
     -v '${DEPLOY_PATH}/secrets:/opt/autographs/secrets:ro' \
     '${AUTOGRAPHS_TOOLS_IMAGE}' \
     sh -lc 'NODE_OPTIONS=\"--max-old-space-size=384\" pnpm --filter app db:migrate'"
     
ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && \
   sudo podman run --rm \
     --network podman-compose_default \
     --env-file .env \
     -v '${DEPLOY_PATH}/wallet:/opt/autographs/wallet:ro' \
     -v '${DEPLOY_PATH}/secrets:/opt/autographs/secrets:ro' \
     '${AUTOGRAPHS_TOOLS_IMAGE}' \
     sh -lc 'NODE_OPTIONS=\"--max-old-space-size=384\" pnpm --filter app db:seed'"

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && \
   sudo podman run --rm \
     --network podman-compose_default \
     --env-file .env \
     -v '${DEPLOY_PATH}/wallet:/opt/autographs/wallet:ro' \
     -v '${DEPLOY_PATH}/secrets:/opt/autographs/secrets:ro' \
     '${AUTOGRAPHS_TOOLS_IMAGE}' \
     sh -lc 'NODE_OPTIONS=\"--max-old-space-size=384\" pnpm --filter app data:smoke'"