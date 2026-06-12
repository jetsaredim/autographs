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

Verification:

- `terraform -chdir=infra/terraform/tenancy fmt -recursive`
- `terraform -chdir=infra/terraform/modules/iam fmt`
- `git diff --check`
- `terraform -chdir=infra/terraform/tenancy validate` attempted but blocked by local OCI provider plugin startup failure; leave full validation to PR checks/operator apply.
