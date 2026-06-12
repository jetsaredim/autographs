---
status: complete
quick_id: 260611-tq0
slug: tighten-tenancy-iam-for-instance-princip
date: 2026-06-12
---

# Quick Task 260611-tq0 Summary

Updated tenancy IAM for the instance-principal Object Storage direction:

- Removed the admin-runtime IAM user/group/membership path.
- Removed the runtime Vault secret-reader policy and secret-name variables.
- Added runtime dynamic-group Object Storage access for the private media bucket.
- Scoped deploy object access to the Terraform state bucket and removed deploy Vault/key/secret permissions.
- Scoped operator object access to the private media bucket while preserving the operator group.
- Added CI tenancy Terraform validation with backend disabled so PRs catch syntax
  and provider-schema issues in `infra/terraform/tenancy`.
- Removed no-longer-useful create toggles for deploy/operator groups and the
  deploy user; those identities are now end-state managed by the tenancy root.
- Removed the remaining create toggles for the project compartment and state
  bucket, with Terraform moved blocks to preserve state addresses.
- Updated the OCI bootstrap runbook to apply the tenancy root before the
  runtime root and to use the tenancy state backend key.
- Reordered bootstrap import guidance before tenancy plan/apply and documented
  why PR CI validates tenancy Terraform without running a live tenancy plan.
- Documented the intentional compartment-scoped runtime dynamic group decision
  and added missing import examples for unconditional tenancy resources.

Verification:

- `terraform -chdir=infra/terraform/tenancy fmt -recursive`
- `terraform -chdir=infra/terraform/modules/iam fmt`
- `git diff --check`
- `terraform -chdir=infra/terraform/tenancy validate` attempted locally but blocked by local OCI provider plugin startup failure; CI now runs tenancy validation from a fresh runner.
