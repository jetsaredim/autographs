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
require_env GITHUB_ACTOR
require_env GHCR_TOKEN

DEPLOY_PATH="${DEPLOY_PATH:-/opt/autographs}"
DEPLOY_SSH_READY_TIMEOUT_SECONDS="${DEPLOY_SSH_READY_TIMEOUT_SECONDS:-900}"
DEPLOY_SSH_READY_INTERVAL_SECONDS="${DEPLOY_SSH_READY_INTERVAL_SECONDS:-10}"
DEPLOY_CADDY="${DEPLOY_CADDY:-false}"
AUTOGRAPHS_DOMAIN="${AUTOGRAPHS_DOMAIN:-autographs.jetsaredim.net}"
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
validate_pattern DEPLOY_SSH_READY_TIMEOUT_SECONDS "$DEPLOY_SSH_READY_TIMEOUT_SECONDS" '^[0-9]+$'
validate_pattern DEPLOY_SSH_READY_INTERVAL_SECONDS "$DEPLOY_SSH_READY_INTERVAL_SECONDS" '^[0-9]+$'
validate_pattern GITHUB_ACTOR "$GITHUB_ACTOR" '^[A-Za-z0-9][A-Za-z0-9-]*$'
validate_pattern AUTOGRAPHS_DOMAIN "$AUTOGRAPHS_DOMAIN" '^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$'

if [[ ! "$DEPLOY_PATH" =~ ^/opt/autographs(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$ ]]; then
  echo "DEPLOY_PATH must be /opt/autographs or a safe child path: ${DEPLOY_PATH}" >&2
  exit 1
fi

if [ "$DEPLOY_SSH_READY_TIMEOUT_SECONDS" -lt 1 ] || [ "$DEPLOY_SSH_READY_INTERVAL_SECONDS" -lt 1 ]; then
  echo "DEPLOY_SSH_READY_TIMEOUT_SECONDS and DEPLOY_SSH_READY_INTERVAL_SECONDS must be positive integers" >&2
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
  -o BatchMode=yes
  -o ConnectTimeout=10
  -o IdentitiesOnly=yes
  -o StrictHostKeyChecking=accept-new
)

wait_for_ssh() {
  local elapsed=0

  echo "Waiting up to ${DEPLOY_SSH_READY_TIMEOUT_SECONDS}s for SSH on ${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}..."

  while [ "$elapsed" -lt "$DEPLOY_SSH_READY_TIMEOUT_SECONDS" ]; do
    if ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "true" >/dev/null 2>&1; then
      echo "SSH is ready on ${VM_PUBLIC_IP}."
      return
    fi

    sleep "$DEPLOY_SSH_READY_INTERVAL_SECONDS"
    elapsed=$((elapsed + DEPLOY_SSH_READY_INTERVAL_SECONDS))
  done

  echo "SSH did not become ready on ${VM_PUBLIC_IP} within ${DEPLOY_SSH_READY_TIMEOUT_SECONDS}s" >&2
  exit 1
}

wait_for_ssh

scp "${SSH_OPTS[@]}" "$ROOT_DIR/deploy/scripts/bootstrap-runtime.sh" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:/tmp/autographs-bootstrap-runtime.sh"
ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "sudo DEPLOY_USER='${DEPLOY_SSH_USER}' DEPLOY_PATH='${DEPLOY_PATH}' bash /tmp/autographs-bootstrap-runtime.sh"

printf '%s' "$GHCR_TOKEN" | ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" "sudo podman login ghcr.io -u '${GITHUB_ACTOR}' --password-stdin"

ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "cd '${DEPLOY_PATH}/compose' && \
   sudo podman-compose -f compose.prod.yaml pull app && \
   sudo podman-compose -f compose.prod.yaml up -d --no-deps app && \
   if [ '${DEPLOY_CADDY}' = 'true' ] ; then \
     sudo podman-compose -f compose.prod.yaml pull caddy && \
     sudo podman-compose -f compose.prod.yaml up -d caddy; \
   fi"

for _ in $(seq 1 30); do
  if curl --fail --silent "http://${VM_PUBLIC_IP}/health" >/dev/null; then
    break
  fi
  sleep 5
done

if ! curl --fail --silent "http://${VM_PUBLIC_IP}/health" >/dev/null; then
  echo "Deployment did not pass Caddy-fronted HTTP /health readiness check" >&2
  exit 1
fi

for _ in $(seq 1 30); do
  if curl --fail --silent "https://${AUTOGRAPHS_DOMAIN}/health" >/dev/null; then
    curl --fail --silent "https://${AUTOGRAPHS_DOMAIN}/health"
    exit 0
  fi
  sleep 5
done

echo "Deployment did not pass Caddy-fronted HTTPS /health check for ${AUTOGRAPHS_DOMAIN}" >&2
exit 1
