# Quick Task: OCI-Native Generated Oracle DB Password Rotation - Research

**Researched:** 2026-09-01
**Domain:** OCI Secret Management automatic generation and Autonomous Database coordinated rotation
**Confidence:** HIGH for the pinned provider schema and repository design; MEDIUM for first-scheduled-rotation timing because Oracle does not document that timing precisely

## Summary

OCI Secret Management can automatically generate a database-compatible passphrase and coordinate its rotation with an Autonomous Database target. The repository already has the two runtime prerequisites: the ADB resource consumes the Oracle DB password secret OCID, and the controller refreshes `CURRENT` after an exact `ORA-01017` and retries one connection. [VERIFIED: repository grep] [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Concepts/manage-secrets.htm]

Provider `oracle/oci` v8.27.0 supports the required settings as in-place `oci_vault_secret` updates. The database-password secret should use `PASSPHRASE`, `DBAAS_DEFAULT_PASSWORD`, a length of 30, an `ADB` target, and a scheduled ISO-8601 interval. It must not send `secret_content` while auto-generation is enabled. [VERIFIED: https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/internal/service/vault/vault_secret_resource.go] [VERIFIED: https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/vendor/github.com/oracle/oci-go-sdk/v65/vault/passphrase_generation_context.go]

**Primary recommendation:** Configure only `oracle_db_password` for OCI generation and ADB rotation; break the otherwise circular Terraform graph by letting ADB look up the already-created deterministic secret while the managed secret references the ADB resource as its rotation target. Merge/deploy the configuration first, then invoke `oci vault secret rotate` once and prove version advancement, controller refresh without restart, and incremental publish success.

## Project Constraints (from AGENTS.md)

- Keep generated static public artifacts plus one private Rust controller; do not introduce a split-service architecture.
- Prefer OCI Always Free and Oracle Autonomous Database Free.
- Keep secret handling explicit and OCI access least-privileged; routine deployment must not require tenancy-wide admin power.
- Auto-deploy from GitHub Actions on merge to `main`.
- Use `cargo fmt`, tests, production-persistence checks, Clippy, Terraform validation, and relevant Ansible checks in proportion to changed files.
- Keep operator documentation procedural and explicit about prerequisites and live-smoke steps.
- Start repository edits through the active GSD workflow, work on a dedicated branch, and merge only through a ready-for-review PR.
- For a review/coder cycle, put all actionable findings and the clean confirmation on the GitHub PR; prefer inline comments.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Generate the ADMIN password | OCI Secret Management | KMS | Vault generates the passphrase and encrypts each version with the existing symmetric key. [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Concepts/manage-secrets.htm] |
| Coordinate password rotation | OCI Secret Management | Autonomous Database | `rotation_config.target_system_details` identifies the ADB target; the service rotation updates the target and promotes the new secret version. [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/create-secret.htm] |
| Declare desired generation and schedule | Terraform runtime root | OCI provider v8.27.0 | `oci_vault_secret` exposes the generation and rotation blocks as updatable fields. [VERIFIED: official provider v8.27.0 source] |
| Recover application connections | Rust controller | OCI Secret Retrieval | On `ORA-01017`, the controller reloads `CURRENT` once and retries one connection. [VERIFIED: `controller/src/oracle_connection.rs`] |
| Prove a live rotation | Operator CLI | Controller logs/admin publish | `RotateSecret` is asynchronous; live validation must wait for the work request and then exercise a fresh DB connection. [CITED: https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/vault/secret/rotate.html] |

## Exact Provider v8.27.0 Contract

The local lock file pins `oracle/oci` `8.27.0`, and `terraform providers schema -json` against that installed provider confirmed this schema. [VERIFIED: `infra/terraform/.terraform.lock.hcl` and local provider schema]

| HCL path | Exact value/constraint | Notes |
|---|---|---|
| `enable_auto_generation` | `true` | Optional, defaults false, updatable. The service response is also exposed separately as computed `is_auto_generation_enabled`. [VERIFIED: official provider v8.27.0 source] |
| `secret_generation_context.generation_type` | `PASSPHRASE` | Provider accepts `BYTES`, `PASSPHRASE`, or `SSH_KEY`; use passphrase for ADB. [VERIFIED: official provider v8.27.0 source] |
| `secret_generation_context.generation_template` | `DBAAS_DEFAULT_PASSWORD` | Exact passphrase templates are `SECRETS_DEFAULT_PASSWORD` and `DBAAS_DEFAULT_PASSWORD`; use the database-specific one. [VERIFIED: official OCI Go SDK vendored by provider v8.27.0] |
| `secret_generation_context.passphrase_length` | `30` | Oracle documents passphrases up to 32, but OCI Database default passwords at most 30; ADB passwords must be 12-30 characters. [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Concepts/manage-secrets.htm] [CITED: https://docs.oracle.com/en-us/iaas/autonomous-database-serverless/doc/manage-users-create.html] |
| `secret_generation_context.secret_template` | omit | The SDK says passphrase secrets have no structure by default and are stored Base64-encoded; adding a JSON wrapper would make the decoded bundle something other than the password ADB expects. [VERIFIED: official OCI Go SDK vendored by provider v8.27.0] |
| `rotation_config.target_system_details.target_system_type` | `ADB` | Provider accepts `ADB` or `FUNCTION`. [VERIFIED: official provider v8.27.0 source] |
| `rotation_config.target_system_details.adb_id` | managed ADB OCID | Required for target type `ADB`. [VERIFIED: official provider v8.27.0 source] |
| `rotation_config.is_scheduled_rotation_enabled` | `true` | When true, `rotation_interval` is required. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/vault_secret.html] |
| `rotation_config.rotation_interval` | `P90D` recommended | Service accepts ISO-8601 durations from 1 through 360 days. Ninety days is a project recommendation, not an Oracle requirement. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/vault_secret.html] [ASSUMED] |

The v8.27.0 provider's `Update()` builds one `UpdateSecret` request. It includes `secret_content` only when that block changed, and it does not invoke `RotateSecret`. Therefore enabling generation/rotation in Terraform does not itself constitute the controlled on-demand rotation test. [VERIFIED: https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/internal/service/vault/vault_secret_resource.go]

Oracle's service rejects a request that combines enabled auto-generation with supplied content. The database-password branch must omit its `secret_content` block, including for fresh creation; leaving the block in configuration is unsafe even if current imported state happens to suppress an update through `ignore_changes`. [VERIFIED: observed OCI `CannotParseRequest` in project operations] [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/create-secret.htm]

## Recommended Terraform Pattern

### Avoid the direct resource cycle

A direct design creates a cycle: ADB needs the secret ID, while the secret rotation target needs the ADB ID. Break it without a GitHub OCID variable by querying the deterministic existing secret for the ADB input. The repository already stages fresh setup with `create_autonomous_database=false`, so the secret exists before the later ADB-creation apply. [VERIFIED: repository Terraform and deployment runbook]

1. Add a conditional `data "oci_vault_secrets"` lookup for the database password using the existing compartment, vault, and unique secret name. Query it only when ADB creation is enabled.
2. Require exactly one result with `one(...)`; an empty or ambiguous lookup must fail closed.
3. Pass that looked-up ID to `module.data_services.autonomous_database_admin_password_secret_id` instead of referencing `oci_vault_secret.runtime["oracle_db_password"].id` directly.
4. On the managed secret, include the ADB rotation block when `each.key == "oracle_db_password" && var.create_autonomous_database`; its `adb_id` can then reference `module.data_services.autonomous_database_id` without a graph cycle.
5. For fresh bootstrap: first apply with ADB disabled creates the generated database password and the two manual secret shells; after populating the wallet/admin-hash shells, the ADB-enabled apply looks up the existing password secret, creates ADB from it, then configures that ADB as the rotation target.

The secret name is unique within a vault by OCI contract, making compartment + vault + exact name a stable lookup boundary. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/vault_secret.html]

### HCL shape

```hcl
# Illustrative shape; preserve current names and module boundaries.
data "oci_vault_secrets" "oracle_db_password" {
  count          = var.create_autonomous_database ? 1 : 0
  compartment_id = var.compartment_ocid
  vault_id       = oci_kms_vault.runtime_secrets.id
  name           = local.runtime_controller_secret_definitions.oracle_db_password.secret_name
}

locals {
  autonomous_database_admin_password_secret_id = var.create_autonomous_database ? one(
    data.oci_vault_secrets.oracle_db_password[0].secrets
  ).id : null
}

resource "oci_vault_secret" "runtime" {
  for_each = local.runtime_controller_secret_definitions

  enable_auto_generation = each.key == "oracle_db_password"

  dynamic "secret_content" {
    for_each = each.key == "oracle_db_password" ? [] : [1]
    content {
      content_type = "BASE64"
      content      = base64encode("AUTOGRAPHS_UNCONFIGURED_${upper(each.key)}")
      name         = "terraform-bootstrap"
      stage        = "CURRENT"
    }
  }

  dynamic "secret_generation_context" {
    for_each = each.key == "oracle_db_password" ? [1] : []
    content {
      generation_type     = "PASSPHRASE"
      generation_template = "DBAAS_DEFAULT_PASSWORD"
      passphrase_length   = 30
    }
  }

  dynamic "rotation_config" {
    for_each = each.key == "oracle_db_password" && var.create_autonomous_database ? [1] : []
    content {
      is_scheduled_rotation_enabled = true
      rotation_interval             = "P90D"
      target_system_details {
        target_system_type = "ADB"
        adb_id             = module.data_services.autonomous_database_id
      }
    }
  }
}
```

Update `runtime_secret_values_ready`: an auto-generated database-password version 1 is usable and must not be mistaken for a manual bootstrap placeholder. For `oracle_db_password`, use the planned `enable_auto_generation` configuration as readiness; do not depend solely on computed `is_auto_generation_enabled`, because that live response can still be false during the first plan that enables generation. Continue requiring `current_version_number > 1` for the wallet password and admin hash. [VERIFIED: current readiness implementation and provider v8.27.0 schema] [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/create-secret.htm]

Do not set ADB `secret_version_number`: Oracle documents that omitting it uses the latest version, while pinning it would undermine rotation. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/database_autonomous_database.html]

## Scope Boundary

### This PR

- Configure automatic generation only for `oracle_db_password`; keep wallet and admin-hash generation/rotation unchanged.
- Omit manual content for the generated database-password secret, but keep `ignore_changes = [secret_content]` protection for operator-managed manual secrets.
- Add the deterministic secret lookup that breaks the ADB/secret dependency cycle.
- Configure the ADB target and scheduled interval.
- Correct readiness logic for generated version 1.
- Update Terraform contract tests and operator documentation.
- Validate that the plan contains no replacement, destroy, plaintext secret, or unexpected ADB password update.

### Live post-deploy validation

- Record the secret's current version and controller service/container start time.
- Confirm the configuration apply left the secret active, auto-generation enabled, rotation target set to the expected ADB, and a schedule/next rotation visible.
- Invoke one on-demand `RotateSecret` and wait for its work request to succeed.
- Verify a larger current version, updated last-rotation time, and healthy rotation status.
- Exercise a DB-backed operation until the controller logs its `ORA-01017` refresh to the new Vault version without restarting.
- Complete an incremental publish.

Oracle documents `next_rotation_time` but does not precisely specify when the first scheduled rotation occurs after enabling a schedule on an existing manual secret. Compare the version immediately before and after Terraform apply; if it already advanced, treat that as the live rotation event and validate before issuing another rotate. [VERIFIED: official OCI SDK secret model] [ASSUMED]

## Safe On-Demand Validation Path

Run locally with an authenticated OCI CLI profile that has the project permissions. The commands store the OCID in a shell variable; no secret contents are requested or printed.

```bash
secret_id="$(terraform -chdir=infra/terraform output -json runtime_secret_ids |
  jq -er '.oracle_db_password')"

before_version="$(oci vault secret get \
  --secret-id "$secret_id" \
  --query 'data."current-version-number"' \
  --raw-output)"

oci vault secret get \
  --secret-id "$secret_id" \
  --query 'data.{autoGeneration:"is-auto-generation-enabled",rotationStatus:"rotation-status",nextRotation:"next-rotation-time",currentVersion:"current-version-number"}'

oci vault secret rotate \
  --secret-id "$secret_id" \
  --wait-for-state SUCCEEDED \
  --max-wait-seconds 1200

after_version="$(oci vault secret get \
  --secret-id "$secret_id" \
  --query 'data."current-version-number"' \
  --raw-output)"

test "$after_version" -gt "$before_version"

oci vault secret get \
  --secret-id "$secret_id" \
  --query 'data.{currentVersion:"current-version-number",lastRotation:"last-rotation-time",nextRotation:"next-rotation-time",rotationStatus:"rotation-status"}'
```

`oci vault secret rotate` forces rotation in Vault and the configured target system, requires a valid target, and returns an asynchronous work request. Its waiter accepts `SUCCEEDED` and defaults to a 1200-second maximum. [CITED: https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/vault/secret/rotate.html]

On the VM, capture service start time before the test and confirm it does not change:

```bash
sudo systemctl show autographs-controller.service \
  --property=ActiveEnterTimestamp --value

sudo journalctl -u autographs-controller.service --since '15 minutes ago' \
  --grep='ORA-01017\|refreshed Oracle database credential\|reusing concurrently refreshed'
```

Success requires the refresh log to report the new Vault version, a subsequent DB-backed admin request to succeed, an incremental publish to succeed, and an unchanged service start timestamp. The existing controller performs one serialized refresh per failed credential generation and one retry only for exact `ORA-01017`. [VERIFIED: `controller/src/oracle_connection.rs`]

## Permissions

No new tenancy IAM policy is indicated. The deploy group already has compartment-scoped `manage secret-family` and `manage autonomous-database-family`. Oracle's permission table says `RotateSecret` needs `SECRET_ROTATE` (included in `manage secrets`/the aggregate family), while using a Vault secret to set ADB ADMIN requires secret-bundle read access plus authority to manage the database operation. [VERIFIED: `infra/terraform/modules/iam/main.tf`] [CITED: https://docs.oracle.com/iaas/Content/Identity/Reference/keypolicyreference.htm] [CITED: https://docs.oracle.com/en-us/iaas/autonomous-database-serverless/doc/manage-users-create.html]

The runtime dynamic group still needs only its existing read access to the three named secret bundles. It does not rotate secrets or update ADB; it retrieves `CURRENT` after the old credential fails. [VERIFIED: repository IAM and controller source]

## Runtime State Inventory

| Category | Items Found | Action Required |
|---|---|---|
| Stored data | Runtime Terraform state tracks the Vault secret; OCI stores its current/previous versions; ADB stores the active ADMIN credential. | In-place configuration only; never state-remove/import/replace these resources. Live rotation creates a new OCI secret version and updates ADB. |
| Live service config | OCI holds generation context, rotation target/status, and schedule outside Git. GitHub supplies `OCI_CREATE_AUTONOMOUS_DATABASE`. | After deploy, inspect the live secret fields and work request before declaring success. |
| OS-registered state | The controller systemd/Podman unit consumes the same stable Vault secret OCID from `/opt/autographs/env/app.env`. | No env-file or service restart change; explicitly prove the start timestamp is unchanged. |
| Secrets/env vars | `ORACLE_DB_PASSWORD_VAULT_SECRET_ID` remains unchanged; plaintext remains absent; only bundle version/content changes. | No GitHub secret or environment-coordinate rotation. Never fetch/print content in validation. |
| Build artifacts | No new package, binary, image tag, or generated artifact is required. | None; run existing Terraform/Rust structural tests. |

## Common Pitfalls

1. **Content plus auto-generation:** OCI rejects the request. Use a conditional `secret_content` block, not merely `ignore_changes`. [CITED: https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/create-secret.htm]
2. **Mutual Terraform references:** Direct secret-to-ADB and ADB-to-secret resource references produce a graph cycle. Use the deterministic list lookup for ADB's input and the resource output for the rotation target.
3. **Version-1/read-after-apply readiness regression:** A freshly auto-generated password is real at version 1, while computed `is_auto_generation_enabled` can still describe the pre-apply object during the enabling plan. Use configured `enable_auto_generation` for the DB branch and keep the `> 1` bootstrap rule only for manually bootstrapped secrets.
4. **Pinning a secret version:** Setting `secret_version_number` prevents normal latest-version consumption and creates drift after rotations. Omit it. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/database_autonomous_database.html]
5. **Assuming apply proves rotation:** Provider `UpdateSecret` configures the feature but does not call `RotateSecret`; the on-demand work request and an application DB operation are required evidence. [VERIFIED: official provider v8.27.0 source]
6. **Assuming an existing session must fail:** Already-established DB sessions may survive a password change. Trigger a fresh DB-backed request and wait for the known refresh log; do not restart the controller.
7. **Import drift:** Provider v8.27.0's acceptance test ignores `enable_auto_generation` and `secret_content` during import verification, and content is intentionally not round-tripped. Preserve the current state address and explicit generation configuration; do not re-import as part of this task. [VERIFIED: https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/internal/integrationtest/vault_secret_test.go]
8. **Overlapping changes:** ADB documentation warns the secret/password update cannot run in parallel with several other ADB changes. Keep this PR focused and stop on any unrelated ADB diff. [CITED: https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/database_autonomous_database.html]

## Validation Architecture

Skipped because `.planning/config.json` explicitly sets `workflow.nyquist_validation` to `false`. Still run the existing focused contract test, Terraform format/validate for both roots, `git diff --check`, and the normal CI workflow.

Recommended focused command:

```bash
cargo test --manifest-path controller/Cargo.toml --test terraform_adb_vault_contract
```

The structural contract should assert the database-only generation settings, absence of its manual content, exact ADB target, schedule, lookup-based cycle break, generated-secret readiness, no plaintext password, no pinned version, and unchanged manual handling for the wallet/admin-hash secrets.

## Security Domain

| ASVS Category | Applies | Standard control |
|---|---|---|
| V2 Authentication | Yes | OCI-generated database credential with coordinated rotation; controller refresh is limited to exact invalid-credential failures. |
| V3 Session Management | No | This task does not modify browser/admin sessions. |
| V4 Access Control | Yes | Existing compartment-scoped deploy permissions and name-scoped runtime bundle reads; no tenancy-wide grant. |
| V5 Input Validation | Yes | Provider enum validation, ADB password template/length, exact one-result secret lookup, and destructive-plan guard. |
| V6 Cryptography | Yes | OCI Secret Management plus the existing AES software key; never generate or encrypt password material in Terraform. |

Threats and controls:

- **Disclosure:** never place generated content in HCL, Terraform variables/state, GitHub variables, shell history, or logs; validate metadata/version only.
- **Tampering:** wait for the signed OCI `RotateSecret` work request and confirm its `SUCCEEDED` terminal state.
- **Denial of service:** prove ADB target coordination and controller retry before relying on the schedule; stop on failed rotation rather than restarting repeatedly.
- **Privilege expansion:** reuse current compartment policies; do not add tenancy-wide Vault or Database permissions.

## Assumptions and Open Question

| # | Claim/decision | Risk if wrong |
|---|---|---|
| A1 | Use `P90D` as the scheduled interval. | Operational cadence differs from user intent; confirm before locking the PR. |
| A2 | The first scheduled rotation will not race the immediate post-deploy check. Oracle does not specify the initial schedule timing precisely. | An earlier-than-expected version advance; mitigate by recording the version before apply and validating any advance before forcing another rotation. |

## Environment Availability

| Dependency | Available | Version | Use |
|---|---:|---:|---|
| Terraform | Yes | 1.15.8 | Plan/apply and output lookup |
| OCI provider | Yes | 8.27.0 | Managed secret and ADB resources |
| OCI CLI | Yes | 3.79.0 | On-demand rotate and metadata checks |
| jq | Yes | 1.8.1 | Extract non-secret OCIDs/version metadata |
| Live OCI credentials | Operator-dependent | — | Required only for plan/apply/rotation validation |

## Sources

### Official Oracle service/API documentation

- https://docs.oracle.com/en-us/iaas/Content/secret-management/Concepts/manage-secrets.htm
- https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/create-secret.htm
- https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/update-secret.htm
- https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/vault/secret/rotate.html
- https://docs.oracle.com/iaas/Content/Identity/Reference/keypolicyreference.htm
- https://docs.oracle.com/en-us/iaas/autonomous-database-serverless/doc/manage-users-create.html

### Official Oracle Terraform provider/source

- https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/vault_secret.html
- https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/database_autonomous_database.html
- https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/internal/service/vault/vault_secret_resource.go
- https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/internal/integrationtest/vault_secret_test.go
- https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/vendor/github.com/oracle/oci-go-sdk/v65/vault/passphrase_generation_context.go
- https://github.com/oracle/terraform-provider-oci/blob/v8.27.0/vendor/github.com/oracle/oci-go-sdk/v65/vault/rotation_config.go
- Local `terraform providers schema -json` using the locked v8.27.0 binary

## Metadata

**Confidence breakdown:**
- Provider schema and update behavior: HIGH — pinned source and installed schema agree.
- Architecture: HIGH — derived from the repository graph and staged bootstrap contract.
- OCI service behavior and permissions: HIGH — official service, CLI, IAM, and ADB documentation.
- First scheduled-run timing: LOW — not stated explicitly; live guard handles uncertainty.

**Valid until:** 2026-10-01, or until the OCI provider version changes.
