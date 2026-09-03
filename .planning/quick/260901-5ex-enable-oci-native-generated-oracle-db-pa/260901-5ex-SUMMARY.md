---
phase: quick-260901-5ex-enable-oci-native-generated-oracle-db-password-rotation
plan: 01
status: human_needed
subsystem: infra
tags: [oci, vault, autonomous-database, terraform, secret-rotation]

requires:
  - phase: quick-migrate-regional-vault-resources
    provides: Deployment-state ownership of the existing Vault, key, runtime secrets, and ADB
provides:
  - OCI-generated Oracle database password with ADB-coordinated P90D rotation configuration
  - Exact-name lookup and metadata-readiness pattern that avoids a Terraform graph cycle
  - Fail-closed bootstrap, deployment, and live-rotation acceptance guidance
affects: [terraform-deploy, oracle-persistence, runtime-secret-operations]

tech-stack:
  added: []
  patterns: [exact-name OCI lookup for an existing same-state dependency, metadata-only secret readiness]

key-files:
  created: []
  modified:
    - infra/terraform/runtime_secrets.tf
    - infra/terraform/main.tf
    - infra/terraform/outputs.tf
    - controller/tests/terraform_adb_vault_contract.rs
    - .github/workflows/deploy.yml
    - docs/oci-bootstrap.md
    - docs/configuration-contract.md
    - docs/deployment-runbook.md

key-decisions:
  - "Only oracle_db_password is OCI-generated and rotated; wallet and admin-hash values remain operator-managed."
  - "ADB consumes the existing secret through an exact-name lookup, while the managed secret targets the managed ADB."
  - "Live validation is deferred until after merge/deploy and must prove one rotation without fetching content or restarting the controller."

patterns-established:
  - "Generated-secret readiness accepts version 1; manually bootstrapped secrets require version 2 or later."
  - "Production rotation evidence uses metadata, numeric versions, controller refresh logs, and an unchanged service start timestamp."

requirements-completed: []

duration: 33min
completed: 2026-09-01
---

# Quick Task 260901-5ex: OCI-Native Oracle DB Password Rotation Summary

**OCI-generated ADB password management with scheduled P90D coordination, cycle-free Terraform wiring, and a guarded no-restart live acceptance procedure**

## Performance

- **Duration:** 33 min
- **Started:** 2026-09-02T00:17:16Z
- **Implementation checkpoint:** 2026-09-02T00:49:50Z
- **Tasks:** 2 of 3 complete; live production verification remains
- **Files modified:** 8

## Accomplishments

- Configured only the Oracle database password for OCI `PASSPHRASE` generation with `DBAAS_DEFAULT_PASSWORD`, length 30, an ADB target, and scheduled `P90D` rotation.
- Broke both the direct secret/ADB dependency and the hidden readiness dependency by using exact-name and metadata-only Vault lookups while retaining both resources in deployment state.
- Updated the workflow and normative/operator documentation so only wallet/admin values are populated manually and the generated database password is never overwritten out of band.
- Added a content-free production procedure that stops on destructive/unrelated plans or unhealthy rotation state and proves version advance, controller credential refresh, incremental publish, and no restart.

## Task Commits

1. **Task 1: Configure database-only OCI generation and coordinated ADB rotation** - `a394a58`
2. **Task 2: Document staged rollout and guarded rotation acceptance** - `a013740`
3. **Task 3: Prove one coordinated live rotation without restarting the controller** - blocked until PR merge and normal deployment

## Files Created/Modified

- `infra/terraform/runtime_secrets.tf` - DB-only generation/rotation, exact lookup, metadata readiness, and cycle-free ADB target wiring.
- `infra/terraform/main.tf` - supplies ADB with the exact-name lookup OCID.
- `infra/terraform/outputs.tf` - describes generated-DB versus manual-secret readiness accurately.
- `controller/tests/terraform_adb_vault_contract.rs` - enforces scope, safety, readiness, rollout, and no-content boundaries.
- `.github/workflows/deploy.yml` - reports generated DB versus manual wallet/admin readiness correctly.
- `docs/oci-bootstrap.md` - documents fresh-environment ordering and generated version-1 readiness.
- `docs/configuration-contract.md` - establishes the normative ownership, generation, and manual-secret contract.
- `docs/deployment-runbook.md` - provides guarded plan/deploy and one-rotation acceptance commands.

## Decisions Made

- Followed the locked plan decisions: both resources remain in deployment state, no new readiness boolean exists, and Terraform configures but never invokes rotation.
- Readiness is derived from a Vault metadata listing rather than the managed secret resource because the latter would recreate the ADB/readiness graph cycle.
- The live checkpoint must occur only after the implementation PR is merged and deployed successfully.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed a hidden readiness dependency cycle**
- **Found during:** Task 1 Terraform validation
- **Issue:** `runtime_secret_values_ready` still read the managed secret resource, so ADB depended on the secret while the secret's rotation target depended on ADB.
- **Fix:** Derived readiness from non-secret `oci_vault_secrets` list metadata while retaining the exact-name DB lookup for the ADB input.
- **Files modified:** `infra/terraform/runtime_secrets.tf`, `controller/tests/terraform_adb_vault_contract.rs`
- **Verification:** Both Terraform roots validate; the focused contract passes.
- **Committed in:** `a394a58`

---

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Necessary to achieve the locked same-state, cycle-free architecture; no scope expansion.

## Issues Encountered

- The repository does not contain the plan-referenced `scripts/validate-ci.sh`. Validation used workflow YAML parsing, repository hygiene checks, `scripts/validate-runtime.sh`, focused Rust tests, Terraform validation for both roots, and `git diff --check` instead.
- The checkout's configured backend could not authenticate during local `terraform init -backend=false`; isolated `TF_DATA_DIR` directories provided backend-free provider initialization and successful validation without touching production state.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract` - 6 passed.
- `bash scripts/validate-runtime.sh` - 3 passed.
- `python3 scripts/validate_repo_hygiene.py` - passed.
- `python3 -m unittest scripts/test_validate_repo_hygiene.py` - 5 passed.
- Parsed every `.github/workflows/*.yml` file with PyYAML - passed.
- Deployment and tenancy Terraform `fmt -check` / `validate` - passed.
- `git diff --check` - passed.

## User Setup Required

No new variables or plaintext inputs are required. Task 3 requires an authenticated OCI operator and VM access after the PR is merged and the normal deployment succeeds.

## Next Phase Readiness

- Implementation and rollout documentation are ready for review and PR plan validation.
- Stop before apply if the authenticated plan proposes replacement/destroy, a secret OCID change, an unexpected ADB update, or an unrelated database diff.
- After merge/deploy, perform exactly one guarded live rotation only when OCI rotation metadata is idle/healthy, then verify the controller refresh and incremental publish without restart.

## Known Stubs

None.

## Self-Check: PASSED

- Task 1 commit `a394a58` exists.
- Task 2 commit `a013740` exists.
- All eight implementation/documentation files exist.
- Task 3 is intentionally incomplete and is not claimed in `requirements-completed`.

---
*Phase: quick-260901-5ex-enable-oci-native-generated-oracle-db-password-rotation*
*Implementation checkpoint: 2026-09-01*
