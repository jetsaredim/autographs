---
quick_id: 260903-on3
status: complete
implementation_commit: cbeed94
---

# Summary

Added a dedicated tenancy IAM policy for the Vault secret resource principal that OCI Audit identified as the caller of the failed rotation `GetSecret` request. The policy restricts callers to Vault secret resource principals in the project compartment and restricts the target to the generated Autonomous Database password secret by stable name, without requiring the deployment-managed secret OCID in tenancy state.

Replaced the ineffective OCI Database service-principal grant and added a regression test that requires the compartment- and name-scoped resource-principal statement.

The corrected implementation is commit `cbeed94`, which supersedes the original service-principal implementation in `4020df9`.

## Verification

- `terraform -chdir=infra/terraform fmt -check -recursive`
- `terraform -chdir=infra/terraform/tenancy validate`
- `terraform -chdir=infra/terraform validate`
- `cargo fmt --all --check`
- `cargo test --test terraform_adb_vault_contract` — 7 passed

The tenancy plan/apply and live rotation retry remain operator-owned.
