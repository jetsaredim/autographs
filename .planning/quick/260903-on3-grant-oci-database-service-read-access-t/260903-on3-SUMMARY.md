---
quick_id: 260903-on3
status: complete
implementation_commit: 01503a2
---

# Summary

Added a dedicated tenancy IAM policy for the Vault secret resource principal that OCI Audit identified as the caller of the rotation requests. The policy grants `use secret-family`, including the `GetSecret` and `UpdateSecret` operations reached during rotation, while restricting callers to Vault secret resource principals in the project compartment and the target to the generated Autonomous Database password secret by stable name. It does not require the deployment-managed secret OCID in tenancy state.

The same policy grants `use autonomous-databases` only for the `adminPassword` action so the resource principal can apply the pending version to ADB. The principal type and compartment conditions remain in place, and neither the secret nor database OCID crosses into tenancy state.

Replaced the ineffective OCI Database service-principal grant and added a regression test that requires both scoped resource-principal statements. The final implementation is commit `01503a2`, building on the secret-update correction in `ea67bc3` and resource-principal correction in `cbeed94`.

## Verification

- `terraform -chdir=infra/terraform fmt -check -recursive`
- `terraform -chdir=infra/terraform/tenancy validate`
- `terraform -chdir=infra/terraform validate`
- `cargo fmt --all --check`
- `cargo test --test terraform_adb_vault_contract` — 7 passed

Live validation completed on 2026-09-04: the rotation work request reached 100% and `SUCCEEDED`, version 6 moved from `PENDING` to `CURRENT`, version 5 became `PREVIOUS`, and the next P90D rotation was scheduled for 2026-12-03. The controller emitted the expected transient ORA-01017 after the ADB password changed; application-level recovery confirmation remains the final runtime observation.
