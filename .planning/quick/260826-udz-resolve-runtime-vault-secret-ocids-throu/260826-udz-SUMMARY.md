---
quick_id: 260826-udz
status: complete
implementation_commit: 8362b67
formatting_commit: 5bcaed7
review_fix_commit: e691b07
completed: 2026-08-27
---

# Quick Task 260826-udz Summary: Resolve Runtime Vault Secret OCIDs Through OCI Data Lookups

Removed the GitHub Variable handoff for runtime Vault secret OCIDs. The runtime Terraform root now discovers all three exact-name ACTIVE secrets and passes their identifiers directly into deployment.

## Changes

- Granted the GitHub deploy identity metadata-only `inspect secrets` access in the project compartment.
- Added exact-name `oci_vault_secrets` data lookups for the database password, wallet password, and admin password hash.
- Required exactly one ACTIVE result per lookup so missing, inactive, or duplicate secrets fail during Terraform plan/refresh before deployment mutation.
- Exported the resolved OCIDs as a Terraform output and wired the deploy workflow to its step outputs instead of GitHub Variables.
- Extracted each output with a standalone `jq -er` assignment so malformed or null Terraform output terminates the workflow under `set -e`.
- Updated operator documentation and repository contract coverage for the new lookup and fail-closed behavior.

## Validation

- Runtime and tenancy Terraform recursive format checks and validation passed.
- Production PR CI Terraform plan passed using the GitHub deploy API identity, proving it can list and resolve all three secret OCIDs.
- All seven CI jobs passed at `e691b07`, including actionlint/ShellCheck, Terraform plan, Rust tests/clippy, Ansible, image build, and secret scan.
- Focused deployment contract suite passed: 8 tests.
- Reviewer warning about zero-match/null handling was fixed and answered inline on PR 222.
- Final review at `e691b07` was clean with no actionable findings.

No runtime Terraform apply or production deployment was run as part of this quick task.
