---
phase: quick-260901-5ex-enable-oci-native-generated-oracle-db-password-rotation
verified: 2026-09-02T00:54:59Z
status: human_needed
score: 4/6 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Review the authenticated deployment-root plan and complete the normal merge deployment"
    expected: "The existing Vault secret and ADB keep their current OCIDs and state addresses; the plan has no replacement/destroy or unrelated ADB diff; apply enables DB-only generation, the ADB target, and the P90D schedule; ordinary DB-backed admin status and an incremental publish still succeed."
    why_human: "Resource identity, OCI control-plane behavior, and the production plan/apply cannot be proven from the local checkout."
  - test: "Run exactly one guarded OCI on-demand database-password rotation without restarting the controller"
    expected: "The OCI work request reaches SUCCEEDED, the CURRENT version increases, rotation metadata is healthy, a fresh DB-backed operation triggers the privacy-safe ORA-01017 refresh log with the new numeric Vault version, incremental publish succeeds, and ActiveEnterTimestamp is unchanged."
    why_human: "This is the plan's explicit post-merge production checkpoint and requires authenticated OCI and VM access."
---

# Quick Task 260901-5ex: OCI-Native Oracle DB Password Rotation Verification Report

**Phase Goal:** Configure OCI Secret Management to generate and rotate the existing Autonomous Database ADMIN password, coordinated with the existing ADB, while preserving resource identities and proving the controller can recover without a restart.
**Verified:** 2026-09-02T00:54:59Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | The deployment state continues to own the existing DB-password secret and ADB without replacement or content exposure. | ? UNCERTAIN | The deployment root still declares the Vault, key, secret, and ADB; `prevent_destroy` remains on Vault/key/secrets; the DB `secret_content` block is omitted; no bundle/content data source or plaintext input was added. However, only the authenticated production plan can prove that the existing remote objects keep their OCIDs and are not replaced. |
| 2 | Only `oracle_db_password` uses the locked OCI generation and ADB rotation settings. | ✓ VERIFIED | `runtime_secrets.tf:80,85-115` scopes auto-generation, omitted manual content, `PASSPHRASE`, `DBAAS_DEFAULT_PASSWORD`, length 30, target type `ADB`, and scheduled `P90D` rotation to the DB secret. Terraform validation and the focused contract test pass. |
| 3 | ADB uses an exact in-root lookup while the managed secret targets the managed ADB without a Terraform cycle. | ✓ VERIFIED | `runtime_secrets.tf:51-64` performs the conditional exact-name lookup and `one(...)`; `main.tf:41` passes the resulting local to the module; `modules/data_services/main.tf:7` supplies it as ADB `secret_id`; `runtime_secrets.tf:104-113` points rotation back to `module.data_services.autonomous_database_id`. Both Terraform roots validate successfully, which rejects graph cycles. |
| 4 | Wallet/admin secrets remain operator-managed and no readiness boolean is introduced. | ✓ VERIFIED | `runtime_secrets.tf:85-102` retains manual bootstrap content only outside the DB branch. Readiness at `runtime_secrets.tf:127-137` is derived from Vault metadata (`>= 1` for generated DB, `> 1` for manual secrets). Contract assertions confirm no root readiness variable/GitHub toggle exists. |
| 5 | Workflow and operator guidance populate only wallet/admin values and never overwrite the OCI-generated DB password. | ✓ VERIFIED | `.github/workflows/deploy.yml:467-475`, `docs/oci-bootstrap.md:108-128`, `docs/configuration-contract.md:93-119,191-193`, and `docs/deployment-runbook.md:115-171` consistently distinguish the generated DB version from the two manual values. The focused contract test enforces the security-critical language. |
| 6 | One coordinated live rotation advances the version and permits refresh/publish without controller restart. | ? UNCERTAIN | `docs/deployment-runbook.md:173-268` contains the guarded acceptance procedure and `controller/src/oracle_connection.rs:173-240` implements exact ORA-01017 refresh/retry logging. The production rotation has intentionally not been run; this is Task 3's blocking human checkpoint. |

**Score:** 4/6 truths fully verified; the two remaining truths require live production evidence and are not implementation defects.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `infra/terraform/runtime_secrets.tf` | DB-only generation/rotation, exact lookup, metadata readiness | ✓ VERIFIED | Exists, substantive, provider-valid, and wired to the ADB module, runtime outputs, and Vault metadata. |
| `infra/terraform/main.tf` | ADB input uses lookup-derived secret OCID | ✓ VERIFIED | `module.data_services.autonomous_database_admin_password_secret_id` receives the lookup-derived local, with no direct managed-secret reference. |
| `controller/tests/terraform_adb_vault_contract.rs` | Regression contract for scope, safety, readiness, wiring, and docs | ✓ VERIFIED | Six tests execute and pass; they cover positive configuration and important forbidden patterns. |
| `.github/workflows/deploy.yml` | Fail-closed generated-vs-manual readiness guidance | ✓ VERIFIED | Terraform output is checked before Ansible; the failure message directs population of wallet/admin only. Workflow YAML parses successfully. |
| `docs/oci-bootstrap.md` | Fresh-environment generated/manual ordering | ✓ VERIFIED | Provides the two-apply sequence, exact-name lookup, version rules, and no-content boundary. |
| `docs/configuration-contract.md` | Normative generated/manual secret ownership contract | ✓ VERIFIED | Describes generation parameters, readiness, deployment-state ownership, runtime retrieval, P90D coordination, and no-plaintext boundary. |
| `docs/deployment-runbook.md` | Guarded rollout and live rotation acceptance | ✓ VERIFIED | Provides pre/post metadata checks, overlap guards, the exact waiter command, version assertion, log check, publish requirement, and no-restart assertion. |
| `infra/terraform/outputs.tf` | Withhold secret OCIDs until generated/manual readiness holds | ✓ VERIFIED | Both secret-ID outputs are filtered by `local.runtime_secret_values_ready` and descriptions match the new semantics. |

`gsd-tools query verify.artifacts` passed all 7 PLAN-declared artifacts. The output file is an additional implementation artifact verified manually.

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `infra/terraform/main.tf` | `data.oci_vault_secrets.oracle_db_password` | Lookup-derived local passed to ADB module | ✓ WIRED | `main.tf:41` consumes `local.autonomous_database_admin_password_secret_id`, defined from `one(data...secrets).id` at `runtime_secrets.tf:64`. |
| `oci_vault_secret.runtime["oracle_db_password"]` | `module.data_services.autonomous_database_id` | `rotation_config.target_system_details.adb_id` | ✓ WIRED | The DB-only dynamic rotation block uses the module ADB output at `runtime_secrets.tf:104-113`. |
| `local.runtime_secret_values_ready` | `.github/workflows/deploy.yml` | Filtered Terraform output plus pre-Ansible `jq` gate | ✓ WIRED | `outputs.tf:21-44` withholds IDs until ready; deploy checks for exactly three non-empty IDs and exits before Ansible. |
| `infra/terraform/runtime_secrets.tf` | `docs/configuration-contract.md` | Normative semantics mirror executable configuration | ✓ WIRED | Generation template/length, version thresholds, P90D scope, stable OCID, and manual-secret ownership agree. |
| `docs/deployment-runbook.md` | `controller/src/oracle_connection.rs` | Acceptance searches for existing exact refresh log and verifies no restart | ✓ WIRED | Runbook grep text matches log messages at `oracle_connection.rs:202-228`; retry occurs once after refresh at lines 209-236. |

The generic key-link query reported four false negatives because these links are indirect or semantic rather than literal file-to-file references; each was traced manually above.

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Consumer | Status |
|---|---|---|---|---|
| `runtime_secrets.tf` / `main.tf` | Existing DB-secret OCID | Exact-name `oci_vault_secrets` lookup plus `one(...)` | ADB resource `secret_id` | ✓ FLOWING |
| `runtime_secrets.tf` | Managed ADB OCID | `module.data_services.autonomous_database_id` | Secret `rotation_config.target_system_details.adb_id` | ✓ FLOWING |
| `runtime_secrets.tf` / `outputs.tf` / deploy workflow | Non-secret current-version metadata | `data.oci_vault_secrets.runtime_readiness` | Readiness local → filtered output → workflow gate | ✓ FLOWING |
| Deployment runbook / controller | Rotated CURRENT version | OCI rotation → controller Vault refresh source | One ORA-01017 retry and privacy-safe version log | ? LIVE PROOF NEEDED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Terraform is formatted | `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff` | Exit 0 | ✓ PASS |
| Deployment graph/configuration validates | `terraform -chdir=infra/terraform validate` | `Success! The configuration is valid.` | ✓ PASS |
| Tenancy boundary still validates | `terraform -chdir=infra/terraform/tenancy validate` | `Success! The configuration is valid.` | ✓ PASS |
| Rotation contract is enforced | `cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract` | 6 passed, 0 failed | ✓ PASS |
| Runtime contract has no regression | `bash scripts/validate-runtime.sh` | 3 passed, 0 failed | ✓ PASS |
| Repository hygiene remains valid | `python3 scripts/validate_repo_hygiene.py` and its unit tests | Validator passed; 5 tests passed | ✓ PASS |
| Workflow files are parseable | PyYAML parse of every `.github/workflows/*.yml` | All six parsed | ✓ PASS |
| Formatting/diff cleanliness | `cargo fmt --check`; `git diff --check 922848e..HEAD` | Exit 0 | ✓ PASS |

The PLAN names `scripts/validate-ci.sh`, but that file does not exist in the repository. Equivalent available checks were run directly (workflow parsing, focused contract, runtime validation, hygiene validation, Terraform validation, and diff checks). This is a plan-command mismatch, not missing product behavior.

### Probe Execution

No probe scripts are declared by this quick task and no conventional `scripts/**/tests/probe-*.sh` files exist. Probe execution is not applicable.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| `QUICK-260901-5EX` | `260901-5ex-PLAN.md` | OCI-generated, ADB-coordinated password rotation with safe no-restart proof | ? NEEDS HUMAN | Tasks 1 and 2 are implemented and locally validated. The requirement's production proof remains Task 3. This quick-task ID is not mapped in `.planning/REQUIREMENTS.md`, as the PLAN's source audit explicitly records. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `infra/terraform/runtime_secrets.tf` | 4, 14, 89-90 | `placeholder` | ℹ️ Info | Intentional non-sensitive bootstrap content for the two operator-managed secrets; it is guarded by version-based readiness and ignored after operators create real versions. Not a stub. |
| Controller build output | — | Existing `oracle_wallet_dir` dead-code warning | ℹ️ Info | Pre-existing warning unrelated to this Terraform/docs task; tests still pass. |

No unreferenced `TBD`, `FIXME`, or `XXX` markers, empty implementations, plaintext password inputs, pinned secret versions, remote-state links, or new readiness toggles were found in the eight changed files.

### Disconfirmation Pass

- **Partially met requirement:** Stable in-place production identity is strongly supported by same addresses and `prevent_destroy`, but it is not proven until the authenticated plan is reviewed.
- **Test limitation:** The focused Rust test is intentionally structural; passing string assertions cannot prove OCI accepts the update or coordinates ADB rotation.
- **Uncovered external path:** The timing/status behavior of the first scheduled rotation cannot be simulated locally. The runbook mitigates it by forbidding overlap and treating an early advance as the acceptance candidate, but production evidence is still required.

These limitations map directly to the human checkpoint and do not reveal a local implementation gap.

### Human Verification Required

#### 1. Guarded plan and normal deployment

**Test:** Review the authenticated deployment-root plan, merge, and let the normal deployment apply run.

**Expected:** No replacement/destroy, secret OCID change, unrelated ADB diff, or state-address change; generation/ADB target/P90D schedule appear on the existing resources; ordinary DB-backed status and incremental publish succeed.

**Why human:** The local checkout has no authority to inspect or mutate the production OCI control plane.

#### 2. One coordinated no-restart rotation

**Test:** Follow `docs/deployment-runbook.md:173-268` when rotation metadata is idle/healthy. Issue exactly one `oci vault secret rotate`, wait for `SUCCEEDED`, trigger a fresh DB-backed request, inspect the safe refresh log, run incremental publish, and compare controller start timestamps.

**Expected:** `after_version > before_version`; the refresh log names the new numeric Vault version; DB-backed work and incremental publish succeed; `ActiveEnterTimestamp` is unchanged; no secret content is fetched or printed.

**Why human:** This requires authenticated OCI CLI access, the production controller VM, and live ADB behavior.

### Gaps Summary

No local implementation blockers were found. Tasks 1 and 2 exist in the current branch, both implementation commits (`a394a58` and `a013740`) are present, and the available local validation passes. The quick task remains `human_needed` solely because its stated goal includes the explicit post-merge production plan/apply and live rotation proof in Task 3.

---

_Verified: 2026-09-02T00:54:59Z_
_Verifier: the agent (gsd-verifier)_
