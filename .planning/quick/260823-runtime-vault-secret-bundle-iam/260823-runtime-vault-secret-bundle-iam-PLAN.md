---
quick_id: 260823-runtime-vault-secret-bundle-iam
status: executing
created: 2026-08-23
---

# Runtime Vault Secret-Bundle IAM

## Objective

Update the tenancy Terraform IAM domain so the runtime instance-principal
dynamic group can retrieve approved OCI Vault secret bundles for the next
configuration-hardening proof.

## Scope

- Add runtime Vault/key/proof-secret resources in the tenancy root.
- Add a runtime Vault secret-bundle policy beside the existing runtime Object
  Storage policy.
- Scope access to the Autographs project compartment, relying on the current
  single-purpose compartment layout for Autographs-only Vault secrets.
- Validate Terraform formatting and tenancy configuration.
- Open a PR and run review before applying the tenancy root.

## Verification

- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff`
- `terraform -chdir=infra/terraform/tenancy validate`
- `terraform -chdir=infra/terraform/tenancy plan -var-file=environments/prod/terraform.tfvars -out=/tmp/autographs-runtime-vault-iam.tfplan`
- Apply only after PR review completes.
