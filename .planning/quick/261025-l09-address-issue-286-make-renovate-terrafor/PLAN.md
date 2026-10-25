---
id: 261025-l09
status: implementation_complete
issue: 286
---

# Keep Renovate Terraform CLI updates atomic

1. Give the composite-action Terraform default exclusive custom regex ownership, group all Terraform CLI dependencies across managers, and normalize workflow release tags.
2. Validate the Renovate configuration and extracted dependency metadata, then check the repository Terraform version alignment.
3. Commit the change and open a ready-for-review PR. Reconcile existing Renovate PRs only after the configuration reaches `main`.
