---
phase: quick-260830-hr5-migrate-adb-terraform-from-the-github-ad
reviewed: 2026-08-30T17:22:00Z
depth: standard
reviewed_head: e26ec15af6c77827c41eefb74bf747b0a6486c0c
files_reviewed: 13
files_reviewed_list:
  - .env.example
  - .github/.env.github.example
  - .github/workflows/ci.yml
  - .github/workflows/deploy.yml
  - controller/tests/terraform_adb_vault_contract.rs
  - docs/configuration-contract.md
  - docs/deployment-runbook.md
  - infra/terraform/environments/prod/terraform.tfvars.example
  - infra/terraform/main.tf
  - infra/terraform/modules/data_services/main.tf
  - infra/terraform/modules/data_services/variables.tf
  - infra/terraform/runtime_secrets.tf
  - infra/terraform/variables.tf
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Quick Task 260830-hr5: Code Review Report

**Reviewed:** 2026-08-30T17:22:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Narrative Findings (AI reviewer)

The reviewed configuration correctly replaces the Terraform plaintext ADMIN
password argument with the existing, uniquely discovered ACTIVE Vault secret
OCID. The provider supports an updatable `secret_id`, disallows using it with
`admin_password`, and uses the latest secret version when no version is supplied.
The successful production-backed PR plan verifies the metadata lookup, state
refresh, and in-place provider planning path (`0 to add, 1 to change, 0 to
destroy`). It does not verify the separate Vault bundle permission required by
the update call, and the repository's current deploy policy does not grant that
permission.

## Critical Issues

### CR-01 [BLOCKER]: Merge-triggered Terraform apply lacks permission to consume the Vault secret

**File:** `infra/terraform/modules/data_services/main.tf:7`

**Issue:** Adding `secret_id` causes OCI provider 8.27.0 to include `SecretId` in
`UpdateAutonomousDatabase` when Terraform applies this change. Oracle's
Autonomous Database documentation requires the calling user group to have
`READ` access to the referenced secret when a Vault secret is used to set or
reset the ADMIN password. The repository's deploy group currently has only
`inspect secrets` in `infra/terraform/modules/iam/main.tf`, which is sufficient
for the metadata-only `oci_vault_secrets` lookup and explains why the PR plan
succeeded, but it does not grant `read secret-bundles`. The merge-triggered
production apply is therefore expected to fail authorization before Ansible
runs.

**Fix:** Before merging this runtime change, add and apply a tenancy policy that
grants the deploy group `read secret-bundles` for only the Oracle database
password secret OCID, or run this one database update with a separately
authorized identity. Keep the permission scoped to the exact secret rather than
all three runtime secrets. Then rerun the production-backed plan and perform a
controlled apply followed by a fresh controller database connection.

Example policy shape:

```hcl
"Allow group id ${module.iam.deploy_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.id = '${oci_vault_secret.runtime["oracle_db_password"].id}'"
```

Oracle reference: <https://docs.oracle.com/en/cloud/paas/autonomous-database/serverless/adbsb/manage-users-create.html#GUID-B5846072-995B-4B81-BDCB-AF530BC42847>

PR audit-trail comment:
<https://github.com/jetsaredim/autographs/pull/224#discussion_r3890026250>

## Verification Performed

- Confirmed the checked-out head is
  `e26ec15af6c77827c41eefb74bf747b0a6486c0c`, matching PR #224.
- Read all 13 scoped files and the complete `origin/main...HEAD` diff.
- Inspected OCI provider 8.27.0 resource behavior for `secret_id` and
  `secret_version_number`.
- Confirmed CI run `33324095587` planned one in-place ADB update and did not run
  an apply.
- `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract`
  passed (2 tests).
- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff`
  passed.
- `terraform -chdir=infra/terraform validate` passed.
- `git diff --check origin/main...HEAD` passed.

---

_Reviewed: 2026-08-30T17:22:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
