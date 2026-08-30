---
quick_id: 260830-mvw
status: in_progress
mode: execute
description: Grant deploy identity least-privilege read access to the Oracle database password secret bundle
---

# Quick Task 260830-mvw Plan

## Goal

Resolve PR #224 review finding CR-01 by granting the GitHub deploy group only the Vault bundle permission needed for the Autonomous Database password update, while preserving metadata-only discovery for the other runtime secrets.

## Task 1: Add the scoped tenancy policy and executable contract

**Files:** `infra/terraform/tenancy/main.tf`, `infra/terraform/tenancy/outputs.tf`, `controller/tests/terraform_adb_vault_contract.rs`

**Action:** Add a tenancy-root IAM policy granting `read secret-bundles` to `module.iam.deploy_group_id`, constrained by `target.secret.id` to `oci_vault_secret.runtime["oracle_db_password"].id`. Keep the runtime dynamic-group policy unchanged and do not grant the deploy identity access to the wallet-password or admin-password-hash bundles. Export the policy name for operator inspection and add a structural regression test for the exact boundary.

**Verify:** Run Terraform formatting and validation, the focused Rust contract test, Rust formatting, and `git diff --check`. If live tenancy configuration is available, run a read-only tenancy plan and require only the intended policy/output changes with no destruction.

## Task 2: Document deployment order and update the PR audit trail

**Files:** `docs/configuration-contract.md`, `docs/deployment-runbook.md`, `docs/oci-bootstrap.md`, PR #224 description and review thread

**Action:** Document that `inspect secrets` remains sufficient for metadata lookup, while ADB control-plane use of `secret_id` additionally requires the deploy group to read only the database-password bundle. State that the tenancy policy must be applied before merging or running the runtime apply. Update PR #224 to remove the obsolete no-IAM experiment boundary and include the prerequisite and validation. Reply directly to CR-01 with the fix commit and validation evidence.

**Verify:** Review the rendered PR body and inline reply, search current docs for contradictory metadata-only claims, and confirm no plaintext secret material or additional secret OCIDs were introduced.

