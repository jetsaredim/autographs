---
quick_id: 260830-hr5
status: complete
implementation_commits:
  - b2612f7
  - 03d53ab
pull_request: 224
completed: 2026-08-30
---

# Quick Task 260830-hr5 Summary: Migrate ADB Terraform Password Source to OCI Vault

Replaced the plaintext GitHub-to-Terraform ADB ADMIN password path with the existing fail-closed Oracle database password Vault secret OCID and proved that the current deploy identity can plan the change without any IAM update.

## Changes

- Wired `local.runtime_secret_id_env_vars["ORACLE_DB_PASSWORD_VAULT_SECRET_ID"]` through the runtime root and data-services module to `oci_database_autonomous_database.catalog.secret_id`.
- Removed the plaintext root/module variable, create-time password precondition, production tfvars example, and both workflow `ADB_ADMIN_PASSWORD` inputs.
- Removed `ADB_ADMIN_PASSWORD` from current GitHub/local examples and operator configuration documentation.
- Documented the distinction between Terraform's control-plane `secret_id` use and the controller's runtime instance-principal secret-bundle retrieval.
- Added a focused Rust structural test enforcing the lookup-to-resource path and rejecting plaintext inputs, secret-bundle data sources, pinned secret versions, GitHub OCID duplication, remote-state coupling, and root IAM/tenancy dependencies.
- Made no changes under `infra/terraform/tenancy/`, `infra/terraform/modules/iam/`, or any IAM/policy resource.

## Validation

- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff` — passed.
- Isolated `terraform -chdir=infra/terraform init -backend=false -input=false` and `terraform -chdir=infra/terraform validate` — passed with OCI provider 8.27.0 accepting `secret_id`.
- `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract` — 2 passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- Plaintext-secret, secret-bundle, pinned-version, GitHub-OCID, cross-stack, IAM-path, and diff checks — passed.
- Ready-for-review PR [#224](https://github.com/jetsaredim/autographs/pull/224) CI run `33323817075` — all seven checks passed.
- The production-state Terraform plan resolved the uniquely named ACTIVE Oracle password secret with the existing deploy identity and planned only `module.data_services.oci_database_autonomous_database.catalog[0]` as an in-place update adding `secret_id`.
- Plan summary: `0 to add, 1 to change, 0 to destroy`; no replacement or destructive action.
- PR #224 remains open, ready for review, and unmerged.

## Authorization Boundary

The green plan proves configuration validity, Vault metadata lookup access, remote-state refresh, and plan-time OCI provider behavior with the current deploy identity. It does not prove apply-time authorization for OCI to consume the secret value while updating ADB. No Terraform apply was run, and the live `ADB_ADMIN_PASSWORD` GitHub secret was not deleted. A later controlled apply plus fresh controller database connectivity must establish that final authorization proof before the live GitHub secret is removed.
