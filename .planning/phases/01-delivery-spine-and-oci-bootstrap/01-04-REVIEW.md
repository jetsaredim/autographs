---
phase: 01-delivery-spine-and-oci-bootstrap
reviewed: 2026-05-12T20:27:54Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - .env.example
  - .github/.env.github.example
  - .github/actions/setup-runtime/action.yml
  - .github/workflows/ci.yml
  - .github/workflows/deploy.yml
  - app/next-env.d.ts
  - app/tsconfig.json
  - docs/configuration-contract.md
  - docs/deployment-runbook.md
  - scripts/deploy-vm.sh
  - scripts/validate-ci.sh
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
resolution_status: resolved
resolved: 2026-05-12T20:45:00Z
---

# Phase 1: Code Review Report

**Reviewed:** 2026-05-12T20:27:54Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Reviewed the committed PR diff from `origin/main...HEAD`, excluding planning artifacts and the lockfile per review-scope rules. Local validation passed with `bash scripts/validate-ci.sh` after lint, typecheck, Next.js build, Terraform fmt, `terraform init -backend=false`, and `terraform validate`.

No critical blockers were found. The main risks are deployment/operations risks: OCI backend auth may not have the identity fields Terraform backend initialization needs, remote SSH commands are assembled through shell interpolation, and the production image path can drift from the lockfile-validated CI path.

## Resolution

All three warnings were addressed after review:

- `deploy.yml` now passes OCI API-key identity fields directly to `terraform init` backend configuration.
- `deploy-vm.sh` now validates deploy inputs against strict shell-safe patterns before interpolating them into remote SSH commands.
- `app/Dockerfile` now copies `pnpm-lock.yaml` and installs with `pnpm install --frozen-lockfile`.

Post-fix verification passed:

- `bash -n scripts/validate-ci.sh scripts/deploy-vm.sh`
- `git diff --check`
- `bash scripts/validate-ci.sh`
- `bash scripts/validate-runtime.sh`

## Warnings

### WR-01: Terraform backend init may not receive OCI API-key identity

**File:** `.github/workflows/deploy.yml:59`
**Issue:** The deploy job sets `OCI_CLI_USER_OCID` and `TF_VAR_user_ocid`, then runs `terraform init` with `auth=APIKey` backend config at lines 82-88. Backend initialization happens before Terraform provider variables are evaluated, and the OCI backend example explicitly says API-key auth also needs `tenancy_ocid`, `user_ocid`, `fingerprint`, and `private_key_path`. As written, the backend may not see the user OCID/private key inputs and fail before `terraform apply` can run.
**Fix:**
```yaml
env:
  OCI_TENANCY_OCID: ${{ secrets.OCI_TENANCY_OCID }}
  OCI_USER_OCID: ${{ secrets.OCI_CLI_USER_OCID }}
  OCI_FINGERPRINT: ${{ secrets.OCI_FINGERPRINT }}
  OCI_PRIVATE_KEY_PATH: ${{ runner.temp }}/oci/api_key.pem
run: |
  terraform init \
    -backend-config="bucket=${{ vars.OCI_STATE_BUCKET_NAME || 'autographs-tf-state' }}" \
    -backend-config="namespace=${{ vars.OCI_OBJECT_STORAGE_NAMESPACE }}" \
    -backend-config="region=${{ vars.OCI_REGION }}" \
    -backend-config="key=${{ vars.OCI_STATE_OBJECT_KEY || 'envs/prod/terraform.tfstate' }}" \
    -backend-config="workspace_key_prefix=envs" \
    -backend-config="auth=APIKey"
```
Alternatively, pass the API-key fields as explicit `-backend-config` values or document the exact backend-supported environment names and mirror them in `.github/.env.github.example`.

### WR-02: Remote deploy commands are vulnerable to shell breakage/injection from config values

**File:** `scripts/deploy-vm.sh:41`
**Issue:** `DEPLOY_PATH`, `GITHUB_ACTOR`, `AUTOGRAPHS_APP_IMAGE`, and `AUTOGRAPHS_HTTP_PORT` are interpolated into remote shell strings inside single quotes on lines 41, 45, and 48. If any repo variable contains a single quote, whitespace-sensitive shell content, or command substitution syntax, the remote command can fail or execute unintended shell. These values are operator-controlled, but they cross the GitHub Variables to remote shell boundary and should be treated as untrusted input.
**Fix:**
```bash
remote_script='
  set -euo pipefail
  mkdir -p "$DEPLOY_PATH/compose" "$DEPLOY_PATH/nginx"
  docker login ghcr.io -u "$GITHUB_ACTOR" --password-stdin
  cd "$DEPLOY_PATH/compose"
  docker compose -f compose.prod.yaml pull app
  docker compose -f compose.prod.yaml up -d
'

printf '%s' "$GHCR_TOKEN" | ssh "${SSH_OPTS[@]}" "${DEPLOY_SSH_USER}@${VM_PUBLIC_IP}" \
  "DEPLOY_PATH=$(printf '%q' "$DEPLOY_PATH") GITHUB_ACTOR=$(printf '%q' "$GITHUB_ACTOR") AUTOGRAPHS_APP_IMAGE=$(printf '%q' "$AUTOGRAPHS_APP_IMAGE") AUTOGRAPHS_HTTP_PORT=$(printf '%q' "${AUTOGRAPHS_HTTP_PORT:-80}") bash -s" \
  <<<"$remote_script"
```
Any equivalent approach is fine: validate these values against strict patterns before use, or send a quoted environment/script over SSH rather than concatenating command strings.

### WR-03: Published production image is not tied to the frozen lockfile path validated by CI

**File:** `.github/workflows/deploy.yml:38`
**Issue:** The deploy workflow publishes the production image directly from `app/Dockerfile`, while CI only validates the workspace install with `corepack pnpm install --frozen-lockfile`. The current image build path does not prove that the published artifact used the committed lockfile, so deployment can succeed with dependency resolution different from the tested CI path.
**Fix:** Make the image build consume the committed lockfile and fail on drift, then keep the deploy workflow publishing only that validated image. For example, update the Dockerfile path used here to copy the workspace lockfile and install with `pnpm install --frozen-lockfile`, or add a CI/deploy smoke step that builds the same Docker image before publication.

---

_Reviewed: 2026-05-12T20:27:54Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
