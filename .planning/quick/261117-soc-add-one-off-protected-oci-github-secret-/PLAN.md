---
status: in_progress
created: 2026-11-17
branch: gsd/quick-add-one-off-protected-oci-github-secret-
---

# Compare GitHub and OCI Vault secrets once

## Goal

Verify that the GitHub repository secrets for the admin password hash and Oracle wallet password exactly match their OCI Vault CURRENT bundle values without exposing either value.

## Tasks

- Add a manual, main-only GitHub Actions workflow with a dedicated reviewer-gated environment.
- Fetch only the two known Vault bundles and compare decoded bytes to the injected GitHub Secrets, reporting only match/mismatch.
- Validate workflow syntax and the comparison behavior locally.
- Merge and run the workflow once, then remove the temporary workflow after the outcome is recorded.
