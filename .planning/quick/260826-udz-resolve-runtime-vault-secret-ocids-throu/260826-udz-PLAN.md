---
quick_id: 260826-udz
status: complete
description: Resolve runtime Vault secret OCIDs through OCI Terraform data lookups and pass them directly to deployment without GitHub variables
created: 2026-08-27
completed: 2026-08-27
---

# Quick Task 260826-udz: Resolve Runtime Vault Secret OCIDs Through OCI Data Lookups

## Goal

Make the runtime Terraform root discover the exact ACTIVE controller secrets by deterministic name and pass their OCIDs directly to Ansible, without GitHub Variables or tenancy remote-state coupling.

## Tasks

1. Grant the deploy identity metadata-only secret inspection and add fail-closed runtime Terraform data lookups and outputs.
2. Wire the runtime Terraform outputs into the Ansible environment and add repository contract coverage proving GitHub OCID variables are not used.
3. Update operator documentation, run Terraform/controller validation, and live-plan both Terraform roots without applying them.

## Verification

- `terraform -chdir=infra/terraform fmt -check -recursive -diff`
- Tenancy and runtime Terraform validation
- Live tenancy and runtime plans against production state
- Targeted controller deployment-contract test
- `git diff --check`
