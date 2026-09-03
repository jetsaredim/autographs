---
quick_id: 260903-on3
status: complete
implementation_commit: 4020df9
---

# Summary

Added a dedicated tenancy IAM policy that lets the OCI Database service read only the generated Autonomous Database password secret by stable secret name. This supplies the `GetSecret` permission needed by coordinated Vault-to-ADB rotation without expanding deploy-user or runtime-instance access.

Added a regression test that requires the exact name-scoped policy statement and prevents duplicate Database service secret grants.

## Verification

- `terraform -chdir=infra/terraform fmt -check -recursive`
- `terraform -chdir=infra/terraform/tenancy validate`
- `terraform -chdir=infra/terraform validate`
- `cargo fmt --all --check`
- `cargo test --test terraform_adb_vault_contract` — 7 passed

The tenancy plan/apply and live rotation retry remain operator-owned.

