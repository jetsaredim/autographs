---
status: in-progress
created: 2026-07-13
issue: 167
---

# Address issue 167: align resolve-runtime-ip Terraform version

## Context

The scheduled Image Cleanup workflow failed because `.github/actions/resolve-runtime-ip/action.yml` defaulted Terraform to `1.15.7`, while `infra/terraform/versions.tf` requires `>= 1.15.8, < 1.16.0`.

## Plan

1. Audit the shared `resolve-runtime-ip` action and all workflow callers.
2. Align the shared action default Terraform version with the production Terraform root.
3. Add lightweight validation so the action default cannot drift below the root minimum version again.
4. Run focused validation and relevant formatting/check commands.
