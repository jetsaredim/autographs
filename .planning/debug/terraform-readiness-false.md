---
status: resolved
trigger: "PR #231 Terraform workflow plan fails because runtime_secret_values_ready is false."
created: 2026-09-02
updated: 2026-09-02
---

# Debug Session: Terraform Secret Readiness False

## Symptoms

- Expected behavior: the authenticated production Terraform plan recognizes the three existing populated runtime secrets and shows only the intended in-place OCI-generated database-password rotation configuration.
- Actual behavior: the production plan aborts at the ADB lifecycle precondition because `runtime_secret_values_ready` evaluates false.
- Error messages: `Resource precondition failed` at `modules/data_services/main.tf:17`; `var.create_autonomous_database is true`; `var.runtime_secret_values_ready is false`; `All managed runtime secrets must have a non-bootstrap CURRENT version before create_autonomous_database can create or update ADB.`
- Timeline: first observed in PR #231 CI after adding metadata-derived runtime secret readiness.
- Reproduction: run the PR CI Terraform plan against the production deployment state and OCI account.

## Current Focus

- hypothesis: OCI provider v8.27.0's `oci_vault_secrets` flattening omits `current_version_number`, so the readiness expression converts every missing summary version to zero even though all three secrets are populated.
- test: verify the authenticated PR Terraform plan uses the singular metadata reads and passes the ADB readiness precondition.
- expecting: the plan preserves the existing Vault, secret, and ADB identities and contains only the intended in-place rotation configuration.
- next_action: push commit `83e5cda` and rerun the PR checks; treat the authenticated Terraform plan as the final environment-backed verification.

reasoning_checkpoint:
  hypothesis: "The readiness gate is false because OCI provider v8.27.0 declares but never sets `current_version_number` in `oci_vault_secrets.SetData()`, and `try(tonumber(missing), 0)` therefore maps all three existing secrets to zero."
  confirming_evidence:
    - "The September 1 main deploy refreshed the same stable secret OCIDs, passed the same thresholds using managed-resource metadata, and emitted all three gated outputs."
    - "The PR #231 job reads both list data sources successfully and then receives a concrete false readiness value."
    - "Oracle provider v8.27.0 source shows `VaultSecretsDataSourceCrud.SetData()` never assigns `current_version_number`, while `VaultSecretDataSourceCrud.SetData()` explicitly assigns it from `GetSecretResponse.CurrentVersionNumber`."
  falsification_test: "The hypothesis would be false if the v8.27.0 list data-source `SetData()` assigned `current_version_number`, or if switching readiness to exact-ID singular metadata still produced zero for the known populated secrets."
  fix_rationale: "Retain list lookups only for content-free stable-OCID discovery, then read each secret's metadata through exact-ID `oci_vault_secret` GetSecret calls and derive the existing readiness thresholds from those populated values."
  blind_spots: "Local OCI credentials cannot reproduce the production plan; after local structural/validation tests, the unchanged PR workflow must confirm the authenticated production plan."

fault_tree:
  - readiness input shape: list-secret summary omits or renames version metadata
  - identity correlation: readiness lookup compares the wrong identifier/name
  - readiness semantics: expression treats unknown/null values as false despite populated secrets
  - external data: one or more production secrets genuinely has only a bootstrap version
  - graph timing: readiness evaluates before a resolvable detail lookup is available

## Evidence

- 2026-09-02: PR #231 job `100445679459` successfully reads both `data.oci_vault_secrets` data sources before the ADB precondition evaluates false.
- 2026-09-02: the same CI log shows the existing Vault, key, ADB, and three secrets refreshing from their stable production OCIDs.
- 2026-09-02: the plan failure is isolated to `runtime_secret_values_ready`; initialization, validation, state refresh, and OCI authentication all succeed.
- 2026-09-02: no `.planning/debug/knowledge-base.md` exists, so there is no prior resolved-session candidate to test; no project-defined skill files exist under `.codex/skills/` or `.agents/skills/`.
- 2026-09-02: `infra/terraform/runtime_secrets.tf` indexes `current_version_number` on elements returned by `data.oci_vault_secrets.runtime_readiness.secrets`, with `try(..., 0)` converting any absent attribute to zero; all three terms therefore become false if the list summary omits that field.
- 2026-09-02: the existing focused Rust test only asserts the readiness expression as source text and does not validate the OCI provider data-source schema or model the expression against list-secret summary shape.
- 2026-09-02: a direct `terraform providers schema -json` in `infra/terraform` did not reach schema output because the configured OCI Object Storage backend attempted `ListObjects` and returned `401 NotAuthenticated`; this is a local backend-auth limitation, not evidence about the readiness hypothesis.
- 2026-09-02: a backend-free Terraform configuration pinned to OCI provider v8.27.0 proves `data.oci_vault_secrets.secrets` includes computed string attribute `current_version_number`; the per-secret `oci_vault_secret` data source exposes the same attribute. The assumed provider-schema mismatch is false.
- 2026-09-02: branch diff from `origin/main` shows the old readiness source was each managed `oci_vault_secret.runtime` resource's `current_version_number`; this branch replaced it with the list data source while retaining `> 1` for the two manual secrets. The threshold itself predates the branch, but its metadata source changed.
- 2026-09-02: `gh run view 100445679459` returned HTTP 404 because `100445679459` is a job identifier rather than a workflow-run identifier; GitHub connectivity and authentication were otherwise sufficient to return the API response.
- 2026-09-02: GitHub job `100445679459` is the PR-head Terraform check at commit `ee6b85d`; both list data sources complete in about 0.2 seconds, the stable ADB refreshes, and the precondition then receives a concrete false readiness value. The log does not expose individual secret version fields.
- 2026-09-02: main deploy run `33484241937` refreshed the same three stable secret OCIDs on OCI provider v8.27.0, passed the old managed-resource-derived `> 1` readiness thresholds, applied successfully, and emitted all three gated secret-ID outputs. Therefore the two manual secrets were above version 1 immediately before PR #231 failed.
- 2026-09-02: official Oracle provider v8.27.0 source confirms `VaultSecretsDataSourceCrud.SetData()` builds each list element without ever assigning `current_version_number`, although the list schema inherits that computed attribute from `VaultSecretResource()`; the singular `VaultSecretDataSourceCrud.SetData()` explicitly assigns `current_version_number` from `GetSecretResponse.CurrentVersionNumber`.
- 2026-09-02: the new focused regression assertion fails before the HCL fix at `terraform_adb_vault_contract.rs:110`, because the configuration still reads `current_version_number` from list summaries; this confirms the test detects the exact faulty source.
- 2026-09-02: after switching readiness to conditional exact-ID `oci_vault_secret` metadata reads, the previously failing focused regression passes (1/1), `terraform fmt -check` passes, and `terraform validate` reports success.
- 2026-09-02: `terraform graph -type=plan` in the production root attempted the configured OCI Object Storage backend and returned the same local `401 NotAuthenticated`; graph construction therefore still needs a backend-free temporary data directory, while validation itself succeeded.
- 2026-09-02: the complete `terraform_adb_vault_contract` integration-test binary passes 6/6; `terraform fmt -check`, `terraform validate`, and `git diff --check` pass.
- 2026-09-02: a second backend-free graph attempt initialized both pinned providers but Terraform still requires initialization of the declared OCI backend before producing a plan graph. This local credential limitation remains; successful `terraform validate` and the authenticated PR plan after commit are the available graph checks.
- 2026-09-02: final scope inspection shows only `infra/terraform/runtime_secrets.tf` and `controller/tests/terraform_adb_vault_contract.rs` modified for the fix; stable `oci_vault_secret.runtime` resource identities are untouched and the unrelated untracked quick directory remains untouched.
- 2026-09-02: project-standard `cargo check --features production-persistence` and `cargo clippy --all-targets --features production-persistence` pass; Terraform formatting/validation and `git diff --check` pass.
- 2026-09-02: full `cargo test` reaches `caddy_static_routes` and has one pre-existing feature-branch assertion failure because that test expects the retired phrase `only its version-1 bootstrap placeholder` while the already-committed PR workflow now distinguishes the OCI-generated DB password from the two manual secrets. The readiness fix changes neither file; all preceding unit/admin/auth tests and the complete 6-test Vault/ADB contract pass.

## Eliminated

- hypothesis: the `oci_vault_secrets` list schema omits `current_version_number` entirely.
  reason: the locally queried OCI provider v8.27.0 schema declares computed `current_version_number`; the defect is instead that list-result flattening never populates the declared field.
- hypothesis: the two existing manual secrets contain legitimate real values at version 1 and are rejected only because the threshold assumes placeholder-first creation.
  reason: the authenticated main deploy one day earlier used the same `> 1` threshold against managed-resource metadata, succeeded, and emitted all three readiness-gated secret OCIDs for the same stable identities.
- hypothesis: the workflow lacks OCI credentials or permission to list the secrets.
  reason: both Vault secret data sources complete successfully in the authenticated job.
- hypothesis: Terraform has a graph cycle between the secret and ADB.
  reason: configuration validation succeeds and the plan reaches state refresh and lifecycle precondition evaluation.

## Resolution

- root_cause: OCI provider v8.27.0 advertises `current_version_number` on `oci_vault_secrets.secrets` because the list data source reuses the resource schema, but its flattening implementation never populates that field. PR #231 switched the readiness gate from managed-resource metadata to this empty list-summary field, and `try(..., 0)` consequently classified every populated production secret as version zero.
- fix: Retained list-secret lookups for stable secret OCID discovery, added ADB-enabled exact-ID `oci_vault_secret` metadata reads, and derived the unchanged generated/manual version thresholds from those populated `GetSecret` results. Added focused structural regression assertions rejecting list-summary version use.
- verification: Focused Vault/ADB contract tests pass 6/6; Terraform formatting and validation pass; `git diff --check`, production-feature `cargo check`, and production-feature `cargo clippy` pass. The authenticated PR Terraform plan remains the final environment-backed check. The stale `caddy_static_routes` wording assertion exposed by the broad suite was updated to enforce the generated-database/manual-wallet-and-admin boundary.
- files_changed: [infra/terraform/runtime_secrets.tf, controller/tests/terraform_adb_vault_contract.rs, controller/tests/caddy_static_routes.rs]
