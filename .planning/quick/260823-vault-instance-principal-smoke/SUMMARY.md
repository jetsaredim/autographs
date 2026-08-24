---
status: complete
completed: 2026-08-24
task: vault-instance-principal-smoke
---

# Quick Task Summary: Controller Vault runtime secret migration

## Outcome

Consumed the successful temporary-container Vault proof and pivoted the PR toward the durable controller/runtime migration path. The throwaway smoke container and live smoke test were removed from the branch; the controller now has startup support for resolving selected runtime secret values from OCI Vault with instance-principal authentication.

## Changes

- Extracted OCI instance-principal metadata/federation/request-signing behavior from `oci_media` into a reusable `oci_auth` helper.
- Kept `OciInstancePrincipalMediaStore` behavior focused on Object Storage URL construction and delegated signing/session management to the shared helper.
- Added `oci_secrets`, a narrow OCI Vault Secret Retrieval client that reads current base64 secret bundles by OCID through the shared instance-principal signer.
- Added `runtime_secrets`, a controller startup resolver for `ORACLE_DB_PASSWORD_VAULT_SECRET_ID`, `ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID`, `AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID`, and `AUTOGRAPHS_OPERATOR_API_TOKEN_VAULT_SECRET_ID`.
- Updated the controller entrypoint so startup Vault resolution runs before the normal config/router initialization path.
- Updated deploy workflow and Ansible `app.env` rendering so Vault secret OCID coordinates can be deployed while direct secret values remain a transition fallback.
- Updated tenancy Terraform so the tenancy root creates four runtime secret shells and derives the runtime secret-bundle policy statements from those same Terraform-managed secret OCIDs, avoiding cross-stack secret-ID inputs while keeping access tightly scoped.
- Explicitly chose the production hash-only admin path: `AUTOGRAPHS_ADMIN_PASSWORD_HASH` moves to Vault, while legacy `AUTOGRAPHS_ADMIN_PASSWORD` is eliminated from production rather than stored as another Vault secret.
- Removed the temporary Vault smoke Dockerfile/test/runbook surface from the branch.
- Updated configuration/deployment docs with the real Vault migration contract.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed
- `cargo test --manifest-path controller/Cargo.toml --features production-oci runtime_secrets` — passed
- `cargo test --manifest-path controller/Cargo.toml --features production-oci oci_secrets` — passed
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oci_auth` — passed
- `cargo test --manifest-path controller/Cargo.toml controller_dockerfile_copies_compile_time_static_assets` — passed
- `cargo check --manifest-path controller/Cargo.toml --features production-oci` — passed
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed
- `terraform -chdir=infra/terraform/tenancy fmt -check -diff` — passed
- `terraform -chdir=infra/terraform/tenancy validate` — passed
- `terraform -chdir=infra/terraform/tenancy plan -var-file=environments/prod/terraform.tfvars` — blocked by expired/unusable OCI authentication while reading the Object Storage remote state bucket (`401 NotAuthenticated` on `ListObjects`)
- `git diff --check` — passed

Ansible syntax check was attempted, but the local sandbox could not create Ansible's default temp directory under `/home/jgreenwa/.ansible/tmp` because that path is read-only in this environment.

## Live proof

The temporary smoke container was transferred to the OCI runtime VM and run with:

- `OCI_AUTH_MODE=instance_principal`
- `OCI_REGION=us-ashburn-1`
- `AUTOGRAPHS_VAULT_PROOF_SECRET_ID=[redacted proof secret OCID]`

Operator-reported output:

```text
running 1 test
Vault proof secret id: [redacted proof secret OCID]
Vault proof secret version: 1
Vault proof secret stages: CURRENT,LATEST
Vault proof decoded byte length: 32
Vault proof decoded sha256 prefix: [redacted]
test live::live_vault_secret_smoke_reads_generated_proof_secret ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
```

This proves a containerized process on the runtime VM can retrieve the scoped proof secret through OCI instance principals without user API keys, GitHub secrets, mounted key files, or Oracle wallet material. The proof container itself is not retained in the branch.

## Next step

Review the controller/Terraform/deploy scaffold, then rotate/populate the four Terraform-managed OCI Vault secret shells outside Terraform and set the matching GitHub repository variables from the `runtime_secret_id_env_vars` output for deploy.
