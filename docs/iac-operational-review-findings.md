# Terraform and Ansible operational review findings

Status: inventory and contract review in progress for Phase 8 Plan 08-08 Task 5.
This register records source evidence before cleanup changes are selected. A
source inventory is not a final disposition or proof that production state
matches configuration.

## Operating boundary

The tenancy root owns the project compartment, deploy identity and policies,
and the OCI state bucket. The production root owns network, runtime compute,
Vault secrets, the optional Oracle database and private media bucket, the
Porkbun record during migration, and the Cloudflare zone, records, strict TLS,
and cache rules. CI plans against production state. Release deployment saves a
plan and applies that exact file only after rejecting Cloudflare or Porkbun DNS
changes. The separate Cloudflare workflow accepts a reviewed change fingerprint
and applies its own saved plan. Ansible handles deploy freeze, runtime
replacement, health and unfreeze, rollback, security maintenance, and cleanup.

The shared invariant for this review is: an ordinary release must preserve the
existing stateful data and DNS ownership, keep publication frozen while the
controller is replaced, and fail visibly before an unreviewed destructive
change. Production state addresses and a sanitized current-input saved plan
are required to prove that invariant; source defaults alone cannot prove it.

### Terraform stack and module map

```mermaid
flowchart LR
  operator[Operator CLI] --> tenancy
  ci[PR CI plan] --> production
  deploy[Release deploy plan and apply] --> production
  cloudflare_workflow[Cloudflare operator plan and apply] --> production

  subgraph tenancy_stack[Tenancy bootstrap stack]
    tenancy[infra/terraform/tenancy]
    tenancy --> iam[modules/iam: compartment, deploy identity, IAM]
    tenancy --> state_bucket[modules/state_bucket: OCI state bucket]
    tenancy --> tenancy_policies[tenancy/main.tf: runtime and DB secret policies]
  end

  subgraph production_stack[Production runtime stack]
    production[infra/terraform]
    production --> network[modules/network: VCN, subnet, NSG]
    production --> compute[modules/compute: runtime VM]
    production --> data_services[modules/data_services: ADB, private media bucket]
    production --> vault[runtime_secrets.tf: Vault, key, runtime secrets]
    production --> dns[dns.tf: Porkbun record]
    production --> cloudflare_resources[cloudflare.tf: zone, records, TLS, cache]
  end

  state_bucket -. tenancy-bootstrap state key .-> tenancy
  state_bucket -. production state key .-> production
  iam -. compartment and deploy identity inputs .-> production
```

The two roots have separate state keys in the same OCI Object Storage bucket:
`envs/prod/tenancy-bootstrap.tfstate` and `envs/prod/terraform.tfstate`.
PR CI authenticates and plans only the production root; it initializes and
validates the tenancy root without a live plan. The release and Cloudflare
workflows also target the production root. Tenancy changes require a separate
operator plan against the tenancy bootstrap state before apply.

## Production inputs observed

On 2026-11-15, read-only GitHub variable queries returned
`CLOUDFLARE_MODE=proxied`, `OCI_CREATE_AUTONOMOUS_DATABASE=true`,
`OCI_CREATE_MEDIA_BUCKET=true`, and
`OCI_AUTONOMOUS_DATABASE_IS_MTLS_CONNECTION_REQUIRED=true`.
The first three values differ from Terraform's `off`/`false` defaults. This
variable query alone does not establish the corresponding Terraform state or
plan result. Secret values were not read. The mTLS GitHub variable remains in
place during PR #354 review, but the proposed workflow code no longer reads it.
Remove it only after that code reaches `main` and a production no-change plan
passes.

There are two different cases in this review. **Missing configuration with the
current code** means a GitHub Actions variable disappears while the workflow
still supplies its `false` fallback; the existing Terraform `count` expressions
then evaluate to zero. **Deliberately removing a switch from the code** is a
refactor: its workflow input, root and module arguments, conditional resources
and lookups, outputs, tests, and documentation must change together. Keeping a
resource counted at one can preserve its `[0]` state address; removing `count`
also changes that address and needs a reviewed Terraform `moved` transition.
No finding below treats deletion of a variable declaration alone as a complete
or valid refactor.

## Preliminary findings

| ID | Severity | Finding and evidence | Disposition | Required proof or change |
|---|---|---|---|---|
| IAC-001 | Warning | With the current code unchanged, the deploy, CI, and Cloudflare workflows substitute `false` when either `OCI_CREATE_AUTONOMOUS_DATABASE` or `OCI_CREATE_MEDIA_BUCKET` GitHub Actions value is absent; production currently sets both `true`. The resulting resource `count` values become zero, and the database value also gates Vault lookups and rotation (`infra/terraform/main.tf`, `runtime_secrets.tf`, `modules/data_services/main.tf`). That configuration-loss path could plan removal of managed production data resources. | Investigate before any fix | Capture state and current-input plan; trace all workflow inputs. Evaluate an explicit-value guard separately from any coherent switch-removal refactor. |
| IAC-002 | Warning | CI runs `terraform plan` against production state but does not classify or fail on destructive actions (`.github/workflows/ci.yml`). The release workflow rejects DNS changes, then applies the full remaining saved plan (`.github/workflows/deploy.yml`). | Investigate with IAC-001 | Inspect current plan and supported change policy; decide which stateful changes require an explicit operator gate. |
| IAC-003 | Info | `create_network` and `create_runtime_instance` default `true`; if passed `false` to the current code, their counted network/compute resources would be removed. The release workflow does not set either variable, and runtime recreation is separately requested through a taint on retry. | Retain pending state review | Confirm both state addresses and whether false is supported. Any switch removal must also revise the count/lookup contracts and preserve state addresses or use reviewed `moved` blocks. |
| IAC-004 | Info | `cloudflare_mode` is `proxied` in production. A missing variable becomes `off` in ordinary deploy/CI, but the deploy saved-plan guard rejects changed Cloudflare resources. The operator workflow requires explicit `dns_only` or `proxied` and checks plan scope and fingerprint. | Retain pending plan proof | Confirm a no-change plan and a missing-mode rejection test; preserve proxy persistence and DNS mutation guard. |
| IAC-005 | Info | Full deploy requires `autographs_deploy_publish_frozen` and `autographs_deploy_explicit_cutover`; unfreeze uses `autographs_deploy_allow_unfreeze`, and restart handlers are suppressed during explicit cutover. | Investigate as one contract | Trace preflight, full deploy, unfreeze, failure rescue, rollback, and fixtures before consolidating any flag. |
| IAC-006 | Info | `security_patching_reboot_validate_dnf_noop` defaults true and is set false in reboot-state fixtures; production validation tasks use it to gate DNF no-op checks. | Investigate test seam | Verify both fixture paths and the production fail-closed behavior before changing the switch. |
| IAC-007 | Info | `create_runtime_instance` still describes a “Caddy to Next.js runtime” in `infra/terraform/variables.tf`, although the active runtime is Caddy plus the Rust controller. | Documentation correction candidate | Update the description after the operational contract review; no state effect. |

No Critical or Warning finding is being point-fixed from this preliminary
inventory. IAC-001 and IAC-002 share the stateful-resource preservation
invariant and require a production state/plan audit plus producer and consumer
trace before an implementation plan is selected.

### Candidate resolution for IAC-001

The owner proposed making `true` the code default for the two production
resource-creation switches and then removing their GitHub Actions variables.
That is a coherent simplification if it changes **both** the Terraform root
defaults and the `vars.* || 'false'` fallbacks in CI, release deploy, and the
Cloudflare operator workflow. Changing only Terraform's defaults would have no
effect in those workflows because they always set `TF_VAR_*`. An explicit
`false` remains available for the documented initial bootstrap before Vault
secrets and the Object Storage namespace are ready.

Sequence: change and validate the code while the GitHub variables remain
`true`; require the PR CI production-state plan to show no infrastructure
changes; merge the workflow code to `main`; remove the two GitHub variables;
then run the CI Terraform plan and a read-only operator plan from `main` to
prove the fallbacks keep the ADB and media bucket at their existing state
addresses. The normal release workflow reads its configuration from `main`,
so a controller deployment is unnecessary to validate a Terraform default or
GitHub Actions fallback change. Do not delete the GitHub values before the
new workflow code is on `main`. If the post-removal plan is not clean, restore
both variables to `true` before any apply. This plan handles missing values
under the existing conditional design and keeps the existing `[0]` resource
addresses. A later removal of the Terraform switches themselves must update
their conditional consumers and move each affected counted instance, such as
`module.data_services.oci_database_autonomous_database.catalog[0]` and
`module.data_services.oci_objectstorage_bucket.media[0]`, to its new address
with reviewed `moved` blocks or an equivalent state migration. That later PR
must show a no-destroy, no-recreate production plan before apply. IAC-002's
broader destructive-plan policy remains a separate review decision.

## Switch census and direct boundaries

This census covers root Terraform Boolean/mode inputs, production Ansible
Boolean/mode inputs found in role defaults or workflow calls, and maintenance
workflow dispatch modes. Module inputs that simply forward a root value are
traced with the root input. Fixture probe overrides remain in the role-task
inventory until their supported test paths are classified. Effects described
for `false` or missing inputs refer to the code as it exists today, not to a
proposed refactor that removes the input and its conditional consumers.

| Switch or mode | Producer and current/default value | Direct consumer and unsafe transition | Review state |
|---|---|---|---|
| `create_network` | Terraform default `true`; deploy/CI do not override | `module.network` counts VCN, subnet, gateway, routing, NSG and rules; false would remove managed network. | Both paths and state effect to verify. |
| `create_runtime_instance` | Terraform default `true`; deploy/CI do not override | `module.compute` and image lookup; false would remove VM. Separate `recreate_runtime_instance` workflow input taints existing VM on retry. | Both paths and retry fixture to verify. |
| `create_autonomous_database` | GitHub variable `true`, Terraform/workflow fallback `false` | ADB count, Vault lookups, readiness check, password rotation target. Missing variable risks database removal. | IAC-001; Warning. |
| `create_media_bucket` | GitHub variable `true`, Terraform/workflow fallback `false` | Private bucket count and namespace precondition. Missing variable risks bucket removal. | IAC-001; Warning. |
| `assign_public_ip` | Fixed `true` in the compute resource; root and module inputs removed | VM VNIC retains its public IP. | PR production plan to verify. |
| ADB `is_free_tier` | Hardcoded `true` in the data services resource; former root/module input removed | Fixes the database to Oracle Always Free. The existing counted resource address is unchanged. | PR CI production plan returned no changes. |
| ADB `is_mtls_connection_required` | Hardcoded `true` in the data services resource; former GitHub/workflow and Terraform inputs removed in PR #354 | Requires wallet-based connections. Existing counted resource address is unchanged. | Local validation and PR CI production no-change plan passed. Remove the now-unused GitHub variable after merge. |
| Media bucket `versioning` | Hardcoded `Enabled` in the data services resource in PR #354 | Preserves private Object Storage recovery posture. Existing counted resource address is unchanged. | Local validation and PR CI production no-change plan passed. |
| Root and tenancy OCI `auth` | Terraform default `APIKey`; accepts `APIKey`/`SecurityToken` | Selects OCI provider authentication for production or bootstrap state. | Confirm each root's operator entry point and credential source. |
| State bucket `storage_tier` | Hardcoded `Standard` in the state bucket module in PR #354 | Keeps Terraform state promptly retrievable. Existing uncounted resource address is unchanged. | Local validation and authenticated tenancy-root no-change plan passed on 2026-11-15; no apply. |
| `cloudflare_mode` | GitHub variable `proxied`; root default and deploy/CI fallback `off` | Counts zone, records, TLS, rules; `dns_only` preserves zone but changes proxy flag; ordinary deploy DNS guard rejects changes. | IAC-004; preserve controlled transitions. |
| `runtime_secret_values_ready` | Derived from the three Vault CURRENT-version checks | Blocks ADB creation/update until the generated DB password and manually populated secrets are ready. | Preserve fail-closed precondition; verify missing and bootstrap versions. |
| `autographs_deploy_publish_frozen` | Role default and deploy input `true`; unfreeze sets `false` | Full-deploy assertion and exact Caddy publish gate. | IAC-005; trace as one cutover contract. |
| `autographs_deploy_explicit_cutover` | Role default `true` | Full-deploy assertion and restart-handler suppression. | IAC-005; trace role notifications and fixtures. |
| `autographs_deploy_allow_unfreeze` | Absent/false by default; unfreeze sets `true` | Allows removal of publish gate when frozen flag becomes false. | IAC-005; verify failure rescue and no bypass. |
| `autographs_publish_unfreeze_verify_only` | Unfreeze default `false`; deploy invokes true then false | First invocation checks origin readiness without removing gate; second removes it after health. | Trace both paths and failed second step. |
| `autographs_system_cleanup_dry_run` | Role default string `"false"`; image-cleanup workflow input | Controls Podman cleanup mutation. | Verify preview and execute paths. |
| `security_patching_reboot_validate_dnf_noop` | Role default `true`; fixture overrides true/false | Gates DNF no-op proof before reboot cleanup. | IAC-006; verify production and test seam. |
| Security update/reboot approval labels | GitHub issue labels and approved workflow events | Separate update and reboot paths validate current scanner state before mutation. | Trace issue update, approval removal, cleanup, and retry. |
| `recreate_runtime_instance` | Deploy workflow dispatch Boolean, default false | Retry-only taint of tracked compute address before saved plan. | Verify replacement gate and rollback path. |
| Release `operation` | Manual deploy workflow choice `retry`/`rollback`; push to `main` follows the release path | Selects retry or controller rollback on manual runs while normal release processing follows the push event. | Trace every branch through status reporting and failure cleanup. |
| Cloudflare `operation` | Operator workflow choice `plan`/`apply` | Apply requires a matching nonempty plan fingerprint. | Current plan-only run passed; retain. |
| Image cleanup `dry_run` / `remote_operation` | Scheduled or manual `image-cleanup.yml`; manual options `inventory`/`delete` | Scheduled local Podman cleanup gets `dry_run=false` and may prune; scheduled GHCR remains dry-run. Manual GHCR deletion requires explicit `delete`; the manual local dry-run input is independent. | Trace both local and remote actions and protected tags. |

### Fixed-policy input validation

The Always Free change removes one root/module pass-through and fixes the
existing resource argument to `true`. It does not change the `count` expression
or resource address. The production tfvars example no longer advertises a paid
tier override. Each additional input below was changed and validated locally
one at a time. None of the changes alters a resource `count`, `for_each`, or
Terraform address. The PR CI authenticated production plan on
[run 43746652748](https://github.com/jetsaredim/autographs/actions/runs/43746652748)
returned no changes. Local validation used the pinned providers with an
isolated `TF_DATA_DIR` and `-backend=false`; that local validation did not read
production or tenancy state.

| Former input | Fixed resource value and local check | Live proof / follow-up |
|---|---|---|
| `autonomous_database_is_mtls_connection_required` | ADB `is_mtls_connection_required = true`; production-root `terraform validate` passed after removing the three workflow inputs and updating wallet docs. | PR CI production plan returned no changes. Remove unused GitHub variable only after merge and recheck. |
| `autonomous_database_db_workload` | ADB `db_workload = "OLTP"`; production-root `terraform validate` passed. | PR CI production plan returned no changes. |
| `autonomous_database_license_model` | ADB `license_model = "LICENSE_INCLUDED"`; production-root `terraform validate` passed. | PR CI production plan returned no changes for this billing-related argument. |
| `autonomous_database_data_storage_size_in_tbs` | ADB `data_storage_size_in_tbs = 1`; production-root `terraform validate` passed. OCI ignores this sizing request on Always Free databases; it is not the effective storage size. | PR CI production plan returned no changes. |
| `media_bucket_versioning` | Private bucket `versioning = "Enabled"`; production-root `terraform validate` passed. | PR CI production plan returned no changes. |
| `state_bucket_storage_tier` | State bucket `storage_tier = "Standard"`; tenancy-root `terraform validate` passed. | Authenticated plan against `envs/prod/tenancy-bootstrap.tfstate` returned no changes on 2026-11-15; no apply. |

The resource-presence switches (`create_autonomous_database`,
`create_media_bucket`, `create_network`, and `create_runtime_instance`) are a
different class. Their conditionals determine resource addresses and supported
bootstrap paths. Do not hardcode or remove them as a simple pass-through
cleanup; use the separate state-preservation sequence above.

### Remaining defaults and current GitHub overrides

On 2026-11-15, the two Terraform root variable files, local operator tfvars,
three production Terraform workflow inputs, and current GitHub Actions variable
names/values were compared. "Same" below means the current explicit value
equals the code fallback; it does not prove the value was never changed
historically or that other operators have no private overrides.

| Input group | Current observation | Contract and disposition |
|---|---|---|
| `create_network`, `create_runtime_instance` | Production defaults and local tfvars are `true`; no GitHub workflow override. | Both drive resource `count` expressions and related lookups/outputs. Retain until a full consumer inventory and reviewed `moved` transitions or equivalent state update make a switch-removal plan safe. |
| `assign_public_ip` | Fixed `true` in the compute resource; no GitHub workflow override. | Root and module inputs were removed; confirm the existing VM has no production plan change. |
| `runtime_boot_volume_size_gbs` | Production default, local tfvars, and example are `50`; no GitHub Actions variable or workflow override exists. | Keep the Terraform input configurable and retain the `50` default. No boot-volume sizing cleanup is proposed. |
| `project_name`, `environment`, `name_prefix` | Defaults and local tfvars match in both roots; no production workflow override. | Feed tags, Vault secret names, IAM names, and runtime resource names. Preserve cross-root alignment; a harmless-looking name change can affect existing resources and secret lookups. |
| `region`, `home_region`, network CIDRs and ingress CIDRs | Current local values match their defaults; no production workflow override. | These are location and network policy inputs. Retain operator control rather than treating the current values as proof of fixed policy. |
| Tenancy identity and state-bucket names | Most local values match defaults or rely on defaults; the tenancy root has no GitHub workflow input. | They identify existing IAM and state resources. Review each name against tenancy state before any hardcoding. |
| `runtime_image_ocid` | Default empty; no current GitHub variable, but workflows still pass the empty value. | Empty selects the Oracle Linux image lookup. It is a meaningful lookup mode, not a fixed image identifier. |

Eight remaining GitHub variables duplicate every matching workflow fallback:
`DEPLOY_SSH_USER`, `DEPLOY_PATH`, `OCI_AUTONOMOUS_DATABASE_NAME`,
`OCI_AUTONOMOUS_DATABASE_DISPLAY_NAME`, `OCI_MEDIA_BUCKET_NAME`,
`AUTOGRAPHS_DNS_DOMAIN`, `AUTOGRAPHS_DNS_SUBDOMAIN`, and
`AUTOGRAPHS_DNS_TTL`. The deploy SSH/path and media bucket names also have
non-Terraform workflow consumers with the same fallbacks. Their removal from
GitHub is a configuration cleanup candidate, followed by CI and read-only
production plan verification; no Terraform code change or state move is needed
to test the current fallback behavior. The three runtime sizing variables
`OCI_RUNTIME_SHAPE`, `OCI_RUNTIME_OCPUS`, and `OCI_RUNTIME_MEMORY_GBS` matched
the Terraform defaults and are being removed from GitHub after the PR plan
confirms no changes. The mTLS GitHub variable is now unused
by the PR code and should be removed only after merge. In contrast, the two
`OCI_CREATE_*` values are `true` while workflow fallbacks remain `false`, and
`CLOUDFLARE_MODE` is `proxied` while its fallback is `off`; those are not
redundant and remain under IAC-001/IAC-004 review.

## Production state and plan evidence

The [read-only operator plan run 43678520459](https://github.com/jetsaredim/autographs/actions/runs/43678520459)
used current `main` (`363f3c6742045f32ec2f0fb52d35be4163224fbb`)
and the production GitHub variables on 2026-11-15. It completed successfully
with **No changes**, Cloudflare change count `0`, change-set fingerprint
`37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`,
and the explicit `Plan only: no Terraform apply was run` result. Thus the
saved plan proposed no create, update, replacement, or destruction at that
point. The plan refreshed the Oracle database, private media bucket, runtime
VM, Vault, network, Porkbun record, Cloudflare zone, three DNS records, strict
TLS, and cache rules. The Oracle image and Vault data lookups completed.

The log exposed 30 distinct refresh/read addresses, including these sensitive
contract anchors (identifiers and values omitted):

| Boundary | Observed address |
|---|---|
| Database | `module.data_services.oci_database_autonomous_database.catalog[0]` |
| Private media | `module.data_services.oci_objectstorage_bucket.media[0]` |
| Runtime VM | `module.compute.oci_core_instance.runtime[0]` |
| Vault secrets | `oci_vault_secret.runtime["admin_password_hash"]`, `oci_vault_secret.runtime["oracle_db_password"]`, `oci_vault_secret.runtime["oracle_db_wallet_password"]` |
| Database secret lookup | `data.oci_vault_secrets.oracle_db_password[0]` |
| Secret readiness lookups | `data.oci_vault_secrets.runtime_readiness`, `data.oci_vault_secret.runtime_readiness["admin_password_hash"]`, `data.oci_vault_secret.runtime_readiness["oracle_db_password"]`, `data.oci_vault_secret.runtime_readiness["oracle_db_wallet_password"]` |
| DNS and cache | `porkbun_dns_record.autographs`, `cloudflare_zone.domain[0]`, `cloudflare_dns_record.autographs[0]`, `cloudflare_dns_record.mail[0]`, `cloudflare_dns_record.spf[0]`, `cloudflare_zone_setting.strict_tls[0]`, `cloudflare_ruleset.autographs_cache[0]` |

The workflow fingerprints the sorted changed-resource JSON; that value is not
a digest of the complete saved plan. It does not print `terraform state list`
or publish the saved plan JSON, so the refresh lines are evidence for those
addresses, not an exhaustive state inventory. A final review must capture the
full address list and a sanitized action summary from the saved plan. The
no-change result supports the current configuration only; it does not resolve
the missing-variable failure paths in IAC-001 and IAC-002.

### Authenticated tenancy-state plan

On 2026-11-15, with the current PR head `c8ebd75` and a freshly authenticated
local OCI `SecurityToken` session, the tenancy root was initialized against
`envs/prod/tenancy-bootstrap.tfstate` in a separate local Terraform data
directory. The read-only `terraform plan -detailed-exitcode` exited `0` and
reported **No changes. Your infrastructure matches the configuration.** A
read-only state address listing included
`module.state_bucket.oci_objectstorage_bucket.this`, the existing uncounted
state bucket resource. No Terraform apply ran. This supplies the live-state
check missing from PR CI's tenancy syntax validation; the plan did not cover
the separate production state key.

## Review and implementation sequence

1. Complete the source inventory below and trace each switch through direct
   producers, consumers, mutation boundaries, reporting paths, and tests.
   Treat missing workflow configuration and deliberate switch removal as
   separate scenarios. A removal plan must rewrite every dependent branch and
   show the resulting state-address effect.
2. Capture production state and a sanitized current-input plan; review any
   proposed action with the owner before selecting code changes.
3. Implement coherent contract changes in focused PRs. Post findings and
   resolution evidence to each PR before merging.
4. Record final dispositions, merged PRs, CI results, and production no-change
   plan evidence here. Only then close P8-POSTURE-006 and Plan 08-08 Task 5.

## Production execution paths traced so far

| Entry point | Mutation boundary | Failure and recovery boundary | Validation still needed |
|---|---|---|---|
| `deploy.yml` release/retry | Saved Terraform plan, DNS-change rejection, then exact-file apply; publisher freeze preflight precedes full Ansible deploy. | Health and frozen-origin checks precede explicit unfreeze and release status. Retry recovery and controller-only rollback are separate paths. | Verify a failed non-DNS plan cannot remove ADB, bucket, or Vault rotation; exercise freeze/unfreeze and rollback fixtures. |
| `publish-freeze-preflight.yml` | Installs an exact Caddy publish gate and stops the old publisher before replacing it. | Durable freeze record and old-process checks block an unsafe handoff. | Confirm interrupted preflight leaves a recoverable frozen state. |
| `deploy.yml` Ansible role | Installs runtime packages, secret-backed environments, quadlets, Caddy, and controller under the frozen gate. | Full role asserts frozen publication and explicit cutover; restart handlers are suppressed in this mode. | Trace every handler notification and both flag values. |
| `publish-unfreeze.yml` | Verify-only invocation checks origin readiness; second invocation removes the gate after health. | Failed unfreeze restores the frozen Caddyfile and retains the durable record. | Run existing role/fixture tests and inspect partial-failure evidence. |
| `controller-rollback.yml` | Replaces only the controller image after digest and mapping checks. | Refuses an old publisher behind shared-media routing without a verified digest record. | Verify rollback cannot bypass a frozen or incompatible media contract. |
| `weekly-security-scan.yml` / `security-scan.yml` | Read-only OpenSCAP scan followed by issue creation/update. | An incomplete host scan prevents an actionable approval issue. | Check degraded scan, stale-label removal, and issue-render fixtures. |
| `apply-security-updates.yml` / `security-patch.yml` | Re-scan and approval validation precede advisory-scoped DNF mutation. | Failure workflow invokes `security-patch-cleanup.yml` to clear stale approval. | Verify drift and partial-apply paths against current fixtures. |
| `reboot-security-runtime.yml` / `security-reboot.yml` | Re-scan and kernel/approval checks precede reboot and installonly cleanup. | Post-reboot scan reports remaining findings; failed workflow invokes cleanup. | Verify both DNF no-op flag values and target-scope fixtures. |
| `image-cleanup.yml` / `system-cleanup.yml` | Podman and GHCR image pruning use independent dry-run/operation inputs. | Protected tags and retained counts guard active versions. | Verify scheduled and explicit-delete paths. |
| `cloudflare-terraform.yml` | Plan-only or fingerprint-gated apply of an exact saved plan containing Cloudflare changes only. | Required state addresses and zone identity are checked before planning; non-Cloudflare changes abort. | Capture full state list and sanitized action summary without values. |

## Source inventory

This list enumerates Terraform blocks, Ansible entry points and role task files, and workflow callers. The disposition for each element remains **trace pending** until its supported states, failure path, and production plan effect are checked. Paths are relative to the repository root.

### Terraform resources, lookups, modules, and providers

| Block | Source | State owner / caller | Inputs, failure path, test, disposition |
|---|---|---|---|
| resource `cloudflare_zone.domain` | `infra/terraform/cloudflare.tf:6` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `cloudflare_dns_record.autographs` | `infra/terraform/cloudflare.tf:16` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `cloudflare_dns_record.mail` | `infra/terraform/cloudflare.tf:27` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `cloudflare_dns_record.spf` | `infra/terraform/cloudflare.tf:39` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `cloudflare_zone_setting.strict_tls` | `infra/terraform/cloudflare.tf:50` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `cloudflare_ruleset.autographs_cache` | `infra/terraform/cloudflare.tf:58` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `porkbun_dns_record.autographs` | `infra/terraform/dns.tf:1` | Production root / controlled DNS apply; routine deploy guarded | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| data `oci_core_images.oracle_linux_10` | `infra/terraform/images.tf:1` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| module `network` | `infra/terraform/main.tf:1` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| module `compute` | `infra/terraform/main.tf:15` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| module `data_services` | `infra/terraform/main.tf:34` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_instance.runtime` | `infra/terraform/modules/compute/main.tf:1` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_database_autonomous_database.catalog` | `infra/terraform/modules/data_services/main.tf:1` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_objectstorage_bucket.media` | `infra/terraform/modules/data_services/main.tf:24` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_compartment.project` | `infra/terraform/modules/iam/main.tf:26` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_group.deploy` | `infra/terraform/modules/iam/main.tf:36` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_group.operator` | `infra/terraform/modules/iam/main.tf:45` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_user.deploy` | `infra/terraform/modules/iam/main.tf:54` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_user_group_membership.deploy` | `infra/terraform/modules/iam/main.tf:64` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_dynamic_group.runtime_instances` | `infra/terraform/modules/iam/main.tf:71` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_api_key.deploy` | `infra/terraform/modules/iam/main.tf:83` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_policy.deploy` | `infra/terraform/modules/iam/main.tf:130` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_policy.operator` | `infra/terraform/modules/iam/main.tf:140` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_policy.runtime_object_access` | `infra/terraform/modules/iam/main.tf:150` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_vcn.main` | `infra/terraform/modules/network/main.tf:1` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_internet_gateway.main` | `infra/terraform/modules/network/main.tf:11` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_route_table.public` | `infra/terraform/modules/network/main.tf:21` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_network_security_group.runtime` | `infra/terraform/modules/network/main.tf:36` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_network_security_group_security_rule.ssh_ingress` | `infra/terraform/modules/network/main.tf:45` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_network_security_group_security_rule.http_ingress` | `infra/terraform/modules/network/main.tf:61` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_network_security_group_security_rule.https_ingress` | `infra/terraform/modules/network/main.tf:77` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_network_security_group_security_rule.egress_all` | `infra/terraform/modules/network/main.tf:93` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_core_subnet.public` | `infra/terraform/modules/network/main.tf:102` | Production root / release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_objectstorage_bucket.this` | `infra/terraform/modules/state_bucket/main.tf:6` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| provider `oci` | `infra/terraform/providers.tf:1` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| provider `porkbun` | `infra/terraform/providers.tf:11` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_kms_vault.runtime_secrets` | `infra/terraform/runtime_secrets.tf:21` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_kms_key.runtime_secrets` | `infra/terraform/runtime_secrets.tf:33` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| data `oci_vault_secrets.oracle_db_password` | `infra/terraform/runtime_secrets.tf:51` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| data `oci_vault_secrets.runtime_readiness` | `infra/terraform/runtime_secrets.tf:58` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| data `oci_vault_secret.runtime_readiness` | `infra/terraform/runtime_secrets.tf:75` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_vault_secret.runtime` | `infra/terraform/runtime_secrets.tf:81` | Production root / CI plan and release deploy | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| data `oci_objectstorage_namespace.ns` | `infra/terraform/tenancy/main.tf:1` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| module `iam` | `infra/terraform/tenancy/main.tf:5` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| module `state_bucket` | `infra/terraform/tenancy/main.tf:28` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_policy.runtime_secret_bundle_access` | `infra/terraform/tenancy/main.tf:38` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| resource `oci_identity_policy.database_secret_access` | `infra/terraform/tenancy/main.tf:51` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| provider `oci` | `infra/terraform/tenancy/providers.tf:1` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |
| provider `oci` | `infra/terraform/tenancy/providers.tf:11` | Tenancy bootstrap / operator | Trace pending: inputs, state address, failure/recovery, validation, final disposition. |

Terraform `moved` blocks in `modules/iam/main.tf` and `modules/state_bucket/main.tf`, plus the Porkbun `import` block in `dns.tf`, remain part of the state-transition review. The `required_providers` declarations in both roots and each module are included in the provider/version validation gate.

### Ansible playbooks

| Playbook | Caller / purpose | Supported states, failure/recovery, validation, disposition |
|---|---|---|
| `deploy/ansible/playbooks/controller-rollback.yml` | `deploy.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/deploy-admin-credentials-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/deploy.yml` | `deploy.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/publish-freeze-deploy-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/publish-freeze-preflight.yml` | `deploy.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/publish-freeze-render-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/publish-freeze-role-reaudit-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/publish-unfreeze.yml` | `deploy.yml`, `ci.yml`, `complete-v0.5.0.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-create-issue-status-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-finding-classification-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-kernel-posture-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-patch-cleanup.yml` | `apply-security-updates.yml`, `ci.yml`, `reboot-security-runtime.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-patch.yml` | `apply-security-updates.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-post-result-status-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-reboot-kernel-selection-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-reboot-result-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-reboot-state-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-reboot.yml` | `ci.yml`, `reboot-security-runtime.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-report-render-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-request-metadata-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-scan.yml` | `weekly-security-scan.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-target-scope-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml` | `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |
| `deploy/ansible/playbooks/system-cleanup.yml` | `image-cleanup.yml`, `ci.yml` | Trace pending: variables, safe states, failed-run cleanup, test, final disposition. |

### Ansible roles, task files, and handlers

| Element | Caller / owner | Supported states, failure/recovery, validation, disposition |
|---|---|---|
| role `autographs_deploy` | Playbook imports / production runtime or maintenance | Trace pending: defaults, imports, privilege, check mode, final disposition. |
| task file `tasks/admin_credentials.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/assert_kernel_images.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/assert_kernel_ownership.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/derive_dnf_exclusions.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/kernel_persistence.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/main.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/publish_freeze.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/revalidate_uek_boot_state.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/validate_kernel_boot_entry.yml` | `autographs_deploy` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| handler `Reload systemd` | `deploy/ansible/roles/autographs_deploy/handlers/main.yml` | Trace pending: notifications, cutover state, recovery, test, final disposition. |
| handler `Restart controller container` | `deploy/ansible/roles/autographs_deploy/handlers/main.yml` | Trace pending: notifications, cutover state, recovery, test, final disposition. |
| handler `Restart Caddy container` | `deploy/ansible/roles/autographs_deploy/handlers/main.yml` | Trace pending: notifications, cutover state, recovery, test, final disposition. |
| role `autographs_system_cleanup` | Playbook imports / production runtime or maintenance | Trace pending: defaults, imports, privilege, check mode, final disposition. |
| task file `tasks/main.yml` | `autographs_system_cleanup` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| role `security_patching` | Playbook imports / production runtime or maintenance | Trace pending: defaults, imports, privilege, check mode, final disposition. |
| task file `tasks/classify_findings.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/classify_reboot_request.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/classify_update_request.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/cleanup_failed_request.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/create_issue.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/extract_issue_metadata.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/inspect_kernel_boot_entry.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/kernel_posture.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/patch.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/post_reboot_result.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/post_result.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/reboot_cleanup.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/resolve_reboot_kernel.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/revalidate_reboot_kernel.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/scan.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/validate_post_reboot_kernel.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/validate_reboot_state.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/validate_request.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |
| task file `tasks/validate_target_scope.yml` | `security_patching` | Trace pending: direct producers/consumers, failure recovery, fixtures, final disposition. |

### GitHub workflows and local actions

| Entry point | Owned operation | Inputs, failure/recovery, test, disposition |
|---|---|---|
| `.github/workflows/apply-security-updates.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/ci.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/cloudflare-terraform.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/complete-v0.5.0.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/conventional-commits.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/deploy.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/image-cleanup.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/reboot-security-runtime.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/workflows/weekly-security-scan.yml` | Workflow event and job trace pending | Trace pending: event/input/secret sources, mutation gate, recovery, CI, final disposition. |
| `.github/actions/resolve-runtime-ip/action.yml` | Called by workflow steps | Trace pending: inputs/outputs, failure behavior, test, final disposition. |
