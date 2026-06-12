---
status: complete
quick_id: 260611-kh0
slug: remove-obsolete-runtime-vault-s3-credent
date: 2026-06-11
---

# Quick Task 260611-kh0 Summary

Removed the runtime Terraform Vault/KMS/secret resources and their root/module
variables and outputs. Updated runtime documentation so S3 Customer Secret
values are no longer described as being mirrored into Terraform-managed Vault
secrets.

Current deploy-time S3 environment wiring remains in place intentionally until
the controller media adapter moves to OCI instance-principal Object Storage.

Verification:

- `terraform -chdir=infra/terraform fmt -recursive`
- `terraform -chdir=infra/terraform validate` not run to completion locally; provider initialization requires registry access and CI/deploy will validate the runtime path.
