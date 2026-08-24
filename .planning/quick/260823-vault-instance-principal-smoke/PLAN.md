---
status: complete
created: 2026-08-23
task: vault-instance-principal-smoke
---

# Quick Task: Start controller Vault runtime secret migration

## Goal

Use the completed temporary-container Vault proof to start the real controller/runtime migration: add controller startup support for resolving selected runtime secrets from OCI Vault with instance principals, and update Terraform/deploy wiring for Terraform-managed runtime secret shells with secret-OCID-scoped bundle access.

## Plan

1. Extract the existing OCI instance-principal federation/request-signing logic into a reusable production-OCI helper without changing Object Storage behavior.
2. Add a controller startup resolver that reads `*_VAULT_SECRET_ID` runtime coordinates, retrieves current OCI Vault secret bundle contents, and populates the existing runtime secret env names before controller configuration loads.
3. Update deploy workflow/Ansible env rendering to pass Vault secret OCID coordinates as transition inputs while direct GitHub secrets remain supported.
4. Update tenancy Terraform so it creates the expected runtime secret shells and derives secret-bundle IAM policy statements from those same Terraform-managed secret OCIDs, avoiding cross-stack secret-ID inputs while keeping read access tightly scoped.
5. Treat the legacy plaintext admin password sink as eliminated from production rather than moved to Vault; production admin authentication is hash-only.
6. Remove the temporary smoke container/test files from the PR while preserving the successful live proof as evidence in the summary.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml --features production-oci runtime_secrets`
- `cargo test --manifest-path controller/Cargo.toml --features production-oci oci_secrets`
- `cargo check --manifest-path controller/Cargo.toml --features production-oci`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `terraform -chdir=infra/terraform/tenancy validate`
- `terraform -chdir=infra/terraform/tenancy plan -var-file=environments/prod/terraform.tfvars`
- `git diff --check`
