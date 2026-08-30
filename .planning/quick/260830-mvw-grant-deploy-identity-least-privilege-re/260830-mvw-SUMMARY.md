---
quick_id: 260830-mvw
status: complete
implementation_commits:
  - c3c77bd
pull_request: 224
completed: 2026-08-30
---

# Quick Task 260830-mvw Summary: Grant Deploy Identity Scoped ADB Secret Access

Resolved PR #224 review finding CR-01 by adding a dedicated tenancy policy that lets the GitHub deploy group read only the Oracle database password secret bundle required by the Autonomous Database `secret_id` update.

## Changes

- Added `oci_identity_policy.deploy_database_password_secret_bundle_access` in the tenancy root.
- Scoped `read secret-bundles` with `target.secret.id` to `oci_vault_secret.runtime["oracle_db_password"].id`.
- Left the wallet-password and admin-password-hash bundles outside the deploy identity boundary.
- Exported the policy name for operator inspection.
- Added executable structural coverage for the exact allowed and denied secret boundaries.
- Documented the required deployment order: apply the tenancy root before allowing the runtime ADB update.

## Validation

- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff` — passed.
- `terraform -chdir=infra/terraform/tenancy validate` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract` — 3 passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `git diff --check` — passed.
- A live tenancy plan was attempted but the local OCI Object Storage backend request returned `401 NotAuthenticated`; no alternate credentials or apply were attempted. The operator must run the authenticated plan/apply.

## Deployment Boundary

Apply the tenancy root from PR #224 first and confirm it creates only the dedicated policy plus its output. Merge or run the runtime deployment only after that policy is active. The first runtime apply and a fresh controller database connection remain the live proof that OCI consumed the secret successfully.

