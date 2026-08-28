---
quick_id: 260826-u0r
status: complete
implementation_commit: 719b55b
completed: 2026-08-27
---

# Quick Task 260826-u0r Summary: Preserve Operator-Managed Vault Secret Content

Converted the Terraform-managed runtime Vault secrets from automatically generated placeholders to manual, intentionally invalid bootstrap content whose subsequent versions remain operator-managed.

## Changes

- Set `enable_auto_generation = false` for every runtime controller secret.
- Added a non-sensitive, per-secret base64 bootstrap marker used only during resource creation.
- Ignored `secret_content` updates and legacy `secret_generation_context` drift so Terraform cannot overwrite OCI CLI rotations.
- Updated the configuration contract and deployment runbook to describe the manual bootstrap and rotation boundary.

## Validation

- `terraform -chdir=infra/terraform/tenancy fmt -check -diff`
- `terraform -chdir=infra/terraform/tenancy validate`
- Real production-tenancy plan against `envs/prod/tenancy-bootstrap.tfstate`: `0 to add, 3 to change, 0 to destroy`
- Machine-readable plan confirmed all three secrets remain on current version `2`, `secret_content` is unchanged, OCI already reports manual generation, and the only planned update is Terraform's stored `enable_auto_generation` value from `true` to `false`.
- `git diff --check`

No Terraform apply was run.
