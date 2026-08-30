---
phase: quick-260830-mvw-grant-deploy-identity-least-privilege-re
reviewed: 2026-08-30T20:48:33Z
depth: standard
reviewed_head: 10bf7aa56923950e03c85206571a2f4f42873b40
files_reviewed: 17
files_reviewed_list:
  - .env.example
  - .github/.env.github.example
  - .github/workflows/ci.yml
  - .github/workflows/deploy.yml
  - controller/tests/terraform_adb_vault_contract.rs
  - docs/configuration-contract.md
  - docs/deployment-runbook.md
  - docs/oci-bootstrap.md
  - infra/terraform/environments/prod/terraform.tfvars.example
  - infra/terraform/main.tf
  - infra/terraform/modules/data_services/main.tf
  - infra/terraform/modules/data_services/variables.tf
  - infra/terraform/tenancy/main.tf
  - infra/terraform/tenancy/outputs.tf
  - infra/terraform/variables.tf
  - infra/terraform/runtime_secrets.tf
  - infra/terraform/modules/iam/main.tf
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Quick Task 260830-mvw: Code Re-review Report

**Reviewed:** 2026-08-30T20:48:33Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** clean

## Narrative Findings (AI reviewer)

All reviewed files meet quality standards. No actionable issues remain.

CR-01 from the initial review is resolved in implementation commit `c3c77bd`.
The tenancy root now creates one dedicated policy with one statement granting the
deploy group `read secret-bundles` only for
`oci_vault_secret.runtime["oracle_db_password"].id`. The statement targets the
project compartment by OCID and is attached in its parent compartment through
the home-region OCI provider, matching OCI IAM policy syntax and inheritance.
No deploy bundle access is added for `oracle_db_wallet_password` or
`admin_password_hash`.

The new policy depends on the existing IAM module outputs and the existing
Oracle database password secret resource. Neither dependency points back to the
policy, so the tenancy graph remains acyclic. The existing deploy policy retains
`inspect secrets` for the three metadata lookups and compartment-scoped
`manage autonomous-database-family` for the ADB update. The runtime root still
passes only the discovered secret OCID to `secret_id`; it does not read a secret
bundle, accept a plaintext ADB password, or consume cross-stack state.

The documentation consistently requires the tenancy-root policy to be planned
and applied before the runtime ADB update. It also correctly distinguishes the
green runtime plan from the remaining live proof: an authenticated tenancy
plan/apply, the controlled ADB update, and a fresh controller database
connection. Absence of that external operator evidence is not a source-code
defect.

The structural regression tests verify the intended exact-secret statement,
exclude equivalent grants for the wallet password and admin hash, preserve the
metadata-only runtime lookup, and reject the retired plaintext/cross-stack
inputs. No plaintext password or remote-state coupling was reintroduced.

## Verification Performed

- Confirmed PR #224 and the checkout are both at
  `10bf7aa56923950e03c85206571a2f4f42873b40`.
- Reviewed the complete `origin/main...HEAD` source diff and the focused
  `e26ec15...10bf7aa` remediation diff.
- Verified the current PR description records the required tenancy-first
  deployment order and live-evidence boundary.
- Verified all seven checks in CI run `33333938755` completed successfully.
- `terraform -chdir=infra/terraform/tenancy validate` passed.
- `terraform -chdir=infra/terraform validate` passed.
- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff`
  passed.
- `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract`
  passed (3 tests).
- `git diff --check c3c77bd^..c3c77bd` passed for the implementation fix.
- Rechecked OCI's official policy syntax, compartment location/inheritance, and
  `target.secret.id` scoping documentation.

## PR Audit Trail

- CR-01 resolution reply:
  <https://github.com/jetsaredim/autographs/pull/224#discussion_r3890492003>
- Clean re-review confirmation:
  <https://github.com/jetsaredim/autographs/pull/224#issuecomment-5471171075>

---

_Reviewed: 2026-08-30T20:48:33Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
