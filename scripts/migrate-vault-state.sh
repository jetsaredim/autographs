#!/usr/bin/env bash

set -Eeuo pipefail
umask 077

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
readonly TENANCY_ROOT="${REPO_ROOT}/infra/terraform/tenancy"
readonly RUNTIME_ROOT="${REPO_ROOT}/infra/terraform"
readonly BACKEND_CONFIG="${RUNTIME_ROOT}/bootstrap/backend.hcl"
readonly TENANCY_TFVARS="${TENANCY_ROOT}/environments/prod/terraform.tfvars"
readonly RUNTIME_TFVARS="${RUNTIME_ROOT}/environments/prod/terraform.tfvars"
readonly TERRAFORM_BIN="${TERRAFORM_BIN:-terraform}"

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

prompt_secret() {
  local variable_name="$1"
  local prompt="$2"
  local value="${!variable_name:-}"

  if [[ -z "${value}" ]]; then
    IFS= read -r -s -p "${prompt}" value
    printf '\n'
  fi
  [[ -n "${value}" ]] || die "${variable_name} must not be empty"
  printf -v "${variable_name}" '%s' "${value}"
  export "${variable_name}"
}

state_has_address() {
  local root="$1"
  local address="$2"

  "${TERRAFORM_BIN}" -chdir="${root}" state list | grep -Fqx -- "${address}"
}

state_resource_id() {
  local root="$1"
  local address="$2"

  "${TERRAFORM_BIN}" -chdir="${root}" show -json |
    jq -er --arg address "${address}" '
      first(
        ..
        | objects
        | select(.address? == $address)
        | .values.id
      )
    '
}

import_if_missing() {
  local address="$1"
  local import_id="$2"
  local expected_state_id="$3"
  local actual_state_id

  if state_has_address "${RUNTIME_ROOT}" "${address}"; then
    actual_state_id="$(state_resource_id "${RUNTIME_ROOT}" "${address}")"
    [[ "${actual_state_id}" == "${expected_state_id}" ]] ||
      die "${address} already exists in runtime state with an unexpected OCID"
    printf 'Already imported with the expected OCID: %s\n' "${address}"
    return
  fi

  "${TERRAFORM_BIN}" -chdir="${RUNTIME_ROOT}" import \
    -input=false \
    -var-file="${RUNTIME_TFVARS}" \
    -var="autographs_dns_record_id=${dns_record_id}" \
    -var="autographs_dns_ttl=${deployed_dns_ttl}" \
    "${address}" \
    "${import_id}"

  actual_state_id="$(state_resource_id "${RUNTIME_ROOT}" "${address}")"
  [[ "${actual_state_id}" == "${expected_state_id}" ]] ||
    die "${address} import completed with an unexpected OCID"
}

require_command "${TERRAFORM_BIN}"
require_command jq
[[ -f "${BACKEND_CONFIG}" ]] || die "missing local backend config: ${BACKEND_CONFIG}"
[[ -f "${TENANCY_TFVARS}" ]] || die "missing tenancy tfvars: ${TENANCY_TFVARS}"
[[ -f "${RUNTIME_TFVARS}" ]] || die "missing runtime tfvars: ${RUNTIME_TFVARS}"

prompt_secret TF_VAR_porkbun_api_key 'Porkbun API key: '
prompt_secret TF_VAR_porkbun_secret_key 'Porkbun secret key: '
trap 'unset TF_VAR_porkbun_api_key TF_VAR_porkbun_secret_key' EXIT

readonly MIGRATION_DIR="${MIGRATION_DIR:-$(mktemp -d /tmp/autographs-vault-state-migration.XXXXXX)}"
readonly TENANCY_BACKUP="${MIGRATION_DIR}/tenancy-before.json"
readonly RUNTIME_BACKUP="${MIGRATION_DIR}/runtime-before.json"
readonly RUNTIME_PLAN="${MIGRATION_DIR}/runtime-after-import.tfplan"

mkdir -p -- "${MIGRATION_DIR}"
chmod 700 "${MIGRATION_DIR}"
printf 'Migration artifacts: %s\n' "${MIGRATION_DIR}"

"${TERRAFORM_BIN}" -chdir="${TENANCY_ROOT}" init \
  -reconfigure \
  -backend-config="${BACKEND_CONFIG}" \
  -backend-config=key=envs/prod/tenancy-bootstrap.tfstate

"${TERRAFORM_BIN}" -chdir="${RUNTIME_ROOT}" init \
  -reconfigure \
  -backend-config="${BACKEND_CONFIG}" \
  -backend-config=key=envs/prod/terraform.tfstate

"${TERRAFORM_BIN}" -chdir="${TENANCY_ROOT}" state pull >"${TENANCY_BACKUP}"
"${TERRAFORM_BIN}" -chdir="${RUNTIME_ROOT}" state pull >"${RUNTIME_BACKUP}"

readonly vault_address='oci_kms_vault.runtime_secrets'
readonly key_address='oci_kms_key.runtime_secrets'
readonly admin_hash_address='oci_vault_secret.runtime["admin_password_hash"]'
readonly db_password_address='oci_vault_secret.runtime["oracle_db_password"]'
readonly wallet_password_address='oci_vault_secret.runtime["oracle_db_wallet_password"]'
readonly source_addresses=(
  "${vault_address}"
  "${key_address}"
  "${admin_hash_address}"
  "${db_password_address}"
  "${wallet_password_address}"
)

for address in "${source_addresses[@]}"; do
  state_has_address "${TENANCY_ROOT}" "${address}" ||
    die "tenancy state does not contain expected source address: ${address}"
done

readonly vault_id="$("${TERRAFORM_BIN}" -chdir="${TENANCY_ROOT}" output -raw runtime_secrets_vault_id)"
readonly key_id="$("${TERRAFORM_BIN}" -chdir="${TENANCY_ROOT}" output -raw runtime_secrets_key_id)"
readonly secret_ids="$("${TERRAFORM_BIN}" -chdir="${TENANCY_ROOT}" output -json runtime_secret_ids)"
readonly management_endpoint="$(
  jq -er '
    .resources[]
    | select(.type == "oci_kms_vault" and .name == "runtime_secrets")
    | .instances[0].attributes.management_endpoint
  ' "${TENANCY_BACKUP}"
)"
readonly dns_record_id="$(
  jq -er '
    .resources[]
    | select(.type == "porkbun_dns_record" and .name == "autographs")
    | .instances[0].attributes.id
  ' "${RUNTIME_BACKUP}"
)"
readonly deployed_dns_ttl="$(
  jq -er '
    .resources[]
    | select(.type == "porkbun_dns_record" and .name == "autographs")
    | .instances[0].attributes.ttl
  ' "${RUNTIME_BACKUP}"
)"
deployed_owner_email="$(
  jq -r '
    first(
      .resources[]
      | select(.type == "oci_kms_vault" and .name == "runtime_secrets")
      | .instances[0].attributes.freeform_tags.owner
    ) // ""
  ' "${TENANCY_BACKUP}"
)"
if [[ "${deployed_owner_email}" == "unset" ]]; then
  deployed_owner_email=""
fi
readonly deployed_owner_email
readonly db_password_id="$(jq -er '.oracle_db_password' <<<"${secret_ids}")"
readonly wallet_password_id="$(jq -er '.oracle_db_wallet_password' <<<"${secret_ids}")"
readonly admin_hash_id="$(jq -er '.admin_password_hash' <<<"${secret_ids}")"

import_if_missing "${vault_address}" "${vault_id}" "${vault_id}"
import_if_missing \
  "${key_address}" \
  "managementEndpoint/${management_endpoint}/keys/${key_id}" \
  "${key_id}"
import_if_missing "${db_password_address}" "${db_password_id}" "${db_password_id}"
import_if_missing "${wallet_password_address}" "${wallet_password_id}" "${wallet_password_id}"
import_if_missing "${admin_hash_address}" "${admin_hash_id}" "${admin_hash_id}"

"${TERRAFORM_BIN}" -chdir="${RUNTIME_ROOT}" plan \
  -input=false \
  -var-file="${RUNTIME_TFVARS}" \
  -var="autographs_dns_record_id=${dns_record_id}" \
  -var="autographs_dns_ttl=${deployed_dns_ttl}" \
  -var="owner_email=${deployed_owner_email}" \
  -target="${vault_address}" \
  -target="${key_address}" \
  -target="${admin_hash_address}" \
  -target="${db_password_address}" \
  -target="${wallet_password_address}" \
  -out="${RUNTIME_PLAN}"

vault_changes="$(
  "${TERRAFORM_BIN}" -chdir="${RUNTIME_ROOT}" show -json "${RUNTIME_PLAN}" |
    jq -r '
      .resource_changes[]?
      | select(.address | test("^oci_(kms_vault|kms_key|vault_secret)\\."))
      | select(.change.actions != ["no-op"])
      | "\(.address): \(.change.actions | join(","))"
    '
)"

if [[ -n "${vault_changes}" ]]; then
  printf 'Unexpected Vault resource changes:\n%s\n' "${vault_changes}" >&2
  die "do not remove the tenancy state addresses; inspect ${RUNTIME_PLAN}"
fi

printf '\nImport checkpoint complete.\n'
printf 'All five runtime resources match their original OCI OCIDs.\n'
printf 'The deployment plan contains no Vault, key, or secret changes.\n'
printf 'Saved plan: %s\n' "${RUNTIME_PLAN}"
printf 'Inspect it with: terraform -chdir=infra/terraform show %q\n' "${RUNTIME_PLAN}"
printf 'Tenancy state is unchanged. Do not run state rm until this plan is reviewed.\n'
