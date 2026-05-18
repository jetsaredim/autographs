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
require_env AUTOGRAPHS_TOOLS_IMAGE

DEPLOY_PATH="${DEPLOY_PATH:-/opt/autographs}"
DEPLOY_SSH_READY_TIMEOUT_SECONDS="${DEPLOY_SSH_READY_TIMEOUT_SECONDS:-900}"
DEPLOY_SSH_READY_INTERVAL_SECONDS="${DEPLOY_SSH_READY_INTERVAL_SECONDS:-10}"
DEPLOY_CADDY="${DEPLOY_CADDY:-false}"
AUTOGRAPHS_DOMAIN="${AUTOGRAPHS_DOMAIN:-autographs.jetsaredim.net}"
AUTOGRAPHS_HTTP_PORT="${AUTOGRAPHS_HTTP_PORT:-80}"
AUTOGRAPHS_HTTPS_PORT="${AUTOGRAPHS_HTTPS_PORT:-443}"
AUTOGRAPHS_DB_PROVIDER="${AUTOGRAPHS_DB_PROVIDER:-oracle}"
ORACLE_DB_USER="${ORACLE_DB_USER:-ADMIN}"
ORACLE_DB_PASSWORD="${ORACLE_DB_PASSWORD:-}"
ORACLE_DB_CONNECT_STRING="${ORACLE_DB_CONNECT_STRING:-autographsdb_high}"
ORACLE_DB_WALLET_DIR="${ORACLE_DB_WALLET_DIR:-}"
ORACLE_DB_WALLET_PASSWORD="${ORACLE_DB_WALLET_PASSWORD:-}"
ORACLE_DB_WALLET_ZIP_BASE64="${ORACLE_DB_WALLET_ZIP_BASE64:-}"
AUTOGRAPHS_MEDIA_STORAGE_PROVIDER="${AUTOGRAPHS_MEDIA_STORAGE_PROVIDER:-oci}"
AUTOGRAPHS_OPERATOR_API_TOKEN="${AUTOGRAPHS_OPERATOR_API_TOKEN:-}"
OCI_REGION="${OCI_REGION:-us-ashburn-1}"
OCI_TENANCY_OCID="${OCI_TENANCY_OCID:-}"
OCI_CLI_USER_OCID="${OCI_CLI_USER_OCID:-}"
OCI_FINGERPRINT="${OCI_FINGERPRINT:-}"
OCI_PRIVATE_KEY_PEM="${OCI_PRIVATE_KEY_PEM:-}"
OCI_PRIVATE_KEY_PATH="${OCI_PRIVATE_KEY_PATH:-}"
OCI_MEDIA_BUCKET_NAME="${OCI_MEDIA_BUCKET_NAME:-autographs-media-prod}"
OCI_MEDIA_NAMESPACE="${OCI_MEDIA_NAMESPACE:-}"
OCI_PRIVATE_KEY_CONTAINER_PATH="/opt/autographs/secrets/oci_api_key.pem"
SSH_KEY_FILE="$(mktemp)"
COMPOSE_ENV_FILE="$(mktemp)"
OCI_PRIVATE_KEY_FILE="$(mktemp)"
WALLET_ZIP_FILE="$(mktemp)"
WALLET_TAR_FILE="$(mktemp)"
WALLET_EXTRACT_DIR="$(mktemp -d)"

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
validate_pattern AUTOGRAPHS_APP_IMAGE "$AUTOGRAPHS_APP_IMAGE" '^[A-Za-z0-9._:/@-]+$'
validate_pattern AUTOGRAPHS_DOMAIN "$AUTOGRAPHS_DOMAIN" '^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$'
validate_pattern AUTOGRAPHS_HTTP_PORT "$AUTOGRAPHS_HTTP_PORT" '^[0-9]+$'
validate_pattern AUTOGRAPHS_HTTPS_PORT "$AUTOGRAPHS_HTTPS_PORT" '^[0-9]+$'
validate_pattern AUTOGRAPHS_DB_PROVIDER "$AUTOGRAPHS_DB_PROVIDER" '^[A-Za-z0-9_-]+$'
validate_pattern ORACLE_DB_USER "$ORACLE_DB_USER" '^[A-Za-z][A-Za-z0-9_$#]*$'
validate_pattern ORACLE_DB_CONNECT_STRING "$ORACLE_DB_CONNECT_STRING" '^[A-Za-z0-9._:/?=@+() -]+$'
validate_pattern AUTOGRAPHS_MEDIA_STORAGE_PROVIDER "$AUTOGRAPHS_MEDIA_STORAGE_PROVIDER" '^(oci|local)$'
validate_pattern OCI_REGION "$OCI_REGION" '^[a-z]+-[a-z]+-[0-9]+$'
validate_pattern OCI_MEDIA_BUCKET_NAME "$OCI_MEDIA_BUCKET_NAME" '^[A-Za-z0-9._-]+$'

if [ -n "$ORACLE_DB_WALLET_DIR" ]; then
  validate_pattern ORACLE_DB_WALLET_DIR "$ORACLE_DB_WALLET_DIR" '^/[A-Za-z0-9._/-]+$'
fi

if [ -n "$OCI_TENANCY_OCID" ]; then
  validate_pattern OCI_TENANCY_OCID "$OCI_TENANCY_OCID" '^ocid1\.[A-Za-z0-9._-]+$'
fi

if [ -n "$OCI_CLI_USER_OCID" ]; then
  validate_pattern OCI_CLI_USER_OCID "$OCI_CLI_USER_OCID" '^ocid1\.[A-Za-z0-9._-]+$'
fi

if [ -n "$OCI_FINGERPRINT" ]; then
  validate_pattern OCI_FINGERPRINT "$OCI_FINGERPRINT" '^[A-Fa-f0-9:]+$'
fi

if [ -n "$OCI_PRIVATE_KEY_PATH" ]; then
  validate_pattern OCI_PRIVATE_KEY_PATH "$OCI_PRIVATE_KEY_PATH" '^/[A-Za-z0-9._/-]+$'
fi

if [ -n "$OCI_MEDIA_NAMESPACE" ]; then
  validate_pattern OCI_MEDIA_NAMESPACE "$OCI_MEDIA_NAMESPACE" '^[A-Za-z0-9._-]+$'
fi

if [[ ! "$DEPLOY_PATH" =~ ^/opt/autographs(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$ ]]; then
  echo "DEPLOY_PATH must be /opt/autographs or a safe child path: ${DEPLOY_PATH}" >&2
  exit 1
fi

if [ "$DEPLOY_SSH_READY_TIMEOUT_SECONDS" -lt 1 ] || [ "$DEPLOY_SSH_READY_INTERVAL_SECONDS" -lt 1 ]; then
  echo "DEPLOY_SSH_READY_TIMEOUT_SECONDS and DEPLOY_SSH_READY_INTERVAL_SECONDS must be positive integers" >&2
  exit 1
fi

cleanup() {
  rm -f "$SSH_KEY_FILE" "$COMPOSE_ENV_FILE" "$OCI_PRIVATE_KEY_FILE" "$WALLET_ZIP_FILE" "$WALLET_TAR_FILE"
  rm -rf "$WALLET_EXTRACT_DIR"
}

trap cleanup EXIT

printf '%s\n' "$DEPLOY_SSH_PRIVATE_KEY" >"$SSH_KEY_FILE"
chmod 600 "$SSH_KEY_FILE"

if [ -n "$OCI_PRIVATE_KEY_PEM" ]; then
  printf '%s\n' "$OCI_PRIVATE_KEY_PEM" >"$OCI_PRIVATE_KEY_FILE"
  chmod 600 "$OCI_PRIVATE_KEY_FILE"
  OCI_PRIVATE_KEY_PATH="$OCI_PRIVATE_KEY_CONTAINER_PATH"
fi

write_compose_env() {
  local name="$1"
  local value="$2"

  value="${value//$'\n'/}"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//\$/\$\$}"
  printf '%s="%s"\n' "$name" "$value" >>"$COMPOSE_ENV_FILE"
}

write_compose_env AUTOGRAPHS_APP_IMAGE "$AUTOGRAPHS_APP_IMAGE"
write_compose_env AUTOGRAPHS_DOMAIN "$AUTOGRAPHS_DOMAIN"
write_compose_env AUTOGRAPHS_HTTP_PORT "$AUTOGRAPHS_HTTP_PORT"
write_compose_env AUTOGRAPHS_HTTPS_PORT "$AUTOGRAPHS_HTTPS_PORT"
write_compose_env AUTOGRAPHS_DB_PROVIDER "$AUTOGRAPHS_DB_PROVIDER"
write_compose_env AUTOGRAPHS_COMPOSE_NETWORK "${AUTOGRAPHS_COMPOSE_NETWORK:-autographs_runtime}"
write_compose_env ORACLE_DB_USER "$ORACLE_DB_USER"
write_compose_env ORACLE_DB_PASSWORD "$ORACLE_DB_PASSWORD"
write_compose_env ORACLE_DB_CONNECT_STRING "$ORACLE_DB_CONNECT_STRING"
write_compose_env ORACLE_DB_WALLET_DIR "$ORACLE_DB_WALLET_DIR"
write_compose_env ORACLE_DB_WALLET_PASSWORD "$ORACLE_DB_WALLET_PASSWORD"
write_compose_env AUTOGRAPHS_MEDIA_STORAGE_PROVIDER "$AUTOGRAPHS_MEDIA_STORAGE_PROVIDER"
write_compose_env AUTOGRAPHS_OPERATOR_API_TOKEN "$AUTOGRAPHS_OPERATOR_API_TOKEN"
write_compose_env OCI_REGION "$OCI_REGION"
write_compose_env OCI_TENANCY_OCID "$OCI_TENANCY_OCID"
write_compose_env OCI_CLI_USER_OCID "$OCI_CLI_USER_OCID"
write_compose_env OCI_FINGERPRINT "$OCI_FINGERPRINT"
write_compose_env OCI_PRIVATE_KEY_PATH "$OCI_PRIVATE_KEY_PATH"
write_compose_env OCI_MEDIA_BUCKET_NAME "$OCI_MEDIA_BUCKET_NAME"
write_compose_env OCI_MEDIA_NAMESPACE "$OCI_MEDIA_NAMESPACE"
chmod 600 "$COMPOSE_ENV_FILE"

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

if [ -n "$OCI_PRIVATE_KEY_PEM" ]; then
  scp "${SSH_OPTS[@]}" "$OCI_PRIVATE_KEY_FILE" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:/tmp/autographs-oci-api-key.pem"
  ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
    "trap 'sudo rm -f /tmp/autographs-oci-api-key.pem' EXIT; sudo install -d -o '${DEPLOY_SSH_USER}' -m 0700 '${DEPLOY_PATH}/secrets' && sudo install -o '${DEPLOY_SSH_USER}' -m 0600 /tmp/autographs-oci-api-key.pem '${DEPLOY_PATH}/secrets/oci_api_key.pem'"
fi

if [ -n "$ORACLE_DB_WALLET_ZIP_BASE64" ]; then
  printf '%s' "$ORACLE_DB_WALLET_ZIP_BASE64" | base64 -d >"$WALLET_ZIP_FILE"
  unzip -q "$WALLET_ZIP_FILE" -d "$WALLET_EXTRACT_DIR"
  tar -C "$WALLET_EXTRACT_DIR" -czf "$WALLET_TAR_FILE" .
  scp "${SSH_OPTS[@]}" "$WALLET_TAR_FILE" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}:/tmp/autographs-wallet.tgz"
  ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
    "sudo install -d -o '${DEPLOY_SSH_USER}' -m 0700 '${DEPLOY_PATH}/wallet' && sudo tar -xzf /tmp/autographs-wallet.tgz -C '${DEPLOY_PATH}/wallet' && sudo chown -R '${DEPLOY_SSH_USER}:${DEPLOY_SSH_USER}' '${DEPLOY_PATH}/wallet' && sudo chmod -R go-rwx '${DEPLOY_PATH}/wallet'"
fi

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
