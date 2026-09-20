# Production security update workflow

This runbook describes the production OS security update scanner and one-click approval workflow.

## Overview

The automation is split into three GitHub Actions workflows:

- `Weekly Security Scan` runs on a weekly schedule or manually and invokes `deploy/ansible/playbooks/security-scan.yml`.
- `Apply Security Updates` runs when the `approved-production-update` label is added to a scanner-created issue and invokes `deploy/ansible/playbooks/security-patch.yml`.
- `Reboot Security Runtime` runs when the `approved-production-reboot` label is added to a scanner-created issue and invokes `deploy/ansible/playbooks/security-reboot.yml`.

The workflows are intentionally thin wrappers around Ansible. Runtime host behavior, GitHub issue rendering, approval validation, drift checks, and update application live in the `security_patching` Ansible role.

The scanner and updater communicate through a GitHub issue. The scanner writes a human-readable report plus a hidden YAML metadata block into the issue body. The updater re-reads that metadata, re-scans the host with the same Oracle Linux OpenSCAP OVAL source, and only acts on the exact advisory IDs that are still pending.

All three workflows use the `production-security-patching` concurrency group. Scanner, update, and reboot jobs therefore cannot race while replacing the same issue body, labels, and state.

## File map

### GitHub workflow files

`.github/workflows/weekly-security-scan.yml`

- Runs every Monday at `08:37 UTC` and also supports `workflow_dispatch`.
- Grants `contents: read` and `issues: write` so the checked-out playbook can create or update GitHub issues.
- Resolves the runtime host through `.github/actions/resolve-runtime-ip`, which reads the Terraform `runtime_public_ip` output and falls back to `VM_PUBLIC_IP` only when the output is unavailable.
- Pins `dawidd6/action-ansible-playbook` by commit SHA.
- Installs `openscap-utils` and `bzip2` on the workflow runner so `oscap-ssh` can run the remote OVAL evaluation.
- Writes the deploy SSH key to `$RUNNER_TEMP` and passes it to `oscap-ssh` through `SSH_ADDITIONAL_OPTIONS`.
- Supplies the production host inventory inline as the alias `production` in the `runtime` group.
- Passes `security_patching_target_group=runtime` to `deploy/ansible/playbooks/security-scan.yml`.
- Exposes `GH_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_RUN_ID`, `GITHUB_RUN_NUMBER`, and `GITHUB_SERVER_URL` to Ansible for issue creation and links.

`.github/workflows/apply-security-updates.yml`

- Runs on `issues.labeled` events only.
- Uses a job-level guard:

  ```yaml
  github.event.label.name == 'approved-production-update' &&
  github.event.issue.pull_request == null
  ```

- Installs `openscap-utils`, `bzip2`, and local Ansible tooling before the third-party Ansible action runs.
- Writes the deploy SSH key to `$RUNNER_TEMP` and passes it to `oscap-ssh` through `SSH_ADDITIONAL_OPTIONS`.
- Resolves the runtime host through `.github/actions/resolve-runtime-ip`, which reads the Terraform `runtime_public_ip` output and falls back to `VM_PUBLIC_IP` only when the output is unavailable.
- Runs `deploy/ansible/playbooks/security-patch.yml` through the pinned Ansible action.
- Uses `continue-on-error: true` on the main update step so the workflow can still run cleanup after validation, SSH, OpenSCAP, or `dnf` failures. Advisory drift is an expected reconciliation outcome, not a failed workflow.
- Passes these extra vars to the apply playbook:

  ```text
  security_patching_issue_number=<labeled issue number>
  security_patching_approver=<label actor>
  security_patching_approval_label=approved-production-update
  security_patching_target_group=runtime
  ```

- Runs `deploy/ansible/playbooks/security-patch-cleanup.yml` with `if: always() && steps.security_update.outcome != 'success'`.
- Fails the workflow after cleanup when the main update step did not succeed.

`.github/workflows/reboot-security-runtime.yml`

- Runs on `issues.labeled` events only.
- Uses a job-level guard:

  ```yaml
  github.event.label.name == 'approved-production-reboot' &&
  github.event.issue.pull_request == null
  ```

- Installs `openscap-utils`, `bzip2`, and local Ansible tooling so the workflow can re-scan after reboot and cleanup.
- Writes the deploy SSH key to `$RUNNER_TEMP` and passes it to `oscap-ssh` through `SSH_ADDITIONAL_OPTIONS`.
- Resolves the runtime host through `.github/actions/resolve-runtime-ip`.
- Runs `deploy/ansible/playbooks/security-reboot.yml` through the pinned Ansible action.
- Uses `continue-on-error: true` on the main reboot step so the workflow can still run cleanup after validation, SSH, reboot, health, installonly cleanup, or OpenSCAP failures. Advisory drift and complete action reclassification are expected reconciliation outcomes, not failed workflows.
- Passes these extra vars to the reboot playbook:

  ```text
  security_patching_issue_number=<labeled issue number>
  security_patching_approver=<label actor>
  security_patching_approval_label=approved-production-reboot
  security_patching_target_group=runtime
  security_patching_reboot_health_domain=<AUTOGRAPHS_DOMAIN>
  ```

- Runs the same cleanup playbook as failed update requests, but removes `approved-production-reboot` instead of `approved-production-update`. Operational preflight failures write a bounded failure-context file so the cleanup comment can include disallowed packages or DNF no-op evidence when available. A complete OpenSCAP scan stays on the normal success path when either the advisory IDs drift or an exact advisory set is reclassified from `reboot` to `update` or `investigate`: no target reboots, the issue is refreshed or closed from the aggregate current scan, and the result comment explains the reconciliation.
- Fails the workflow after cleanup when the main reboot step did not succeed.

`.github/production-patch-approvers.yml`

- Stores the GitHub usernames allowed to approve production updates and reboot cleanup by applying an approval label.
- Current format:

  ```yaml
  production_patch_approvers:
    - jetsaredim
  ```

`.github/CODEOWNERS`

- Requests owner review for the production update workflow files, approver allowlist, security patching playbooks, role files, and this runbook.
- Includes `.github/CODEOWNERS` itself so ownership rules are covered by ownership review.
- CODEOWNERS only requests review unless branch protection requires CODEOWNER approval.

### Ansible playbooks

`deploy/ansible/playbooks/security-scan.yml`

- First play targets `security_patching_target_group`, defaulting to `runtime`.
- Runs with `become: true` and gathers facts on production hosts.
- Imports `security_patching` with `tasks_from: scan`.
- Second play targets `localhost`.
- Imports `security_patching` with `tasks_from: create_issue`.

`deploy/ansible/playbooks/security-patch.yml`

- First play targets `localhost` and imports `tasks_from: validate_request`.
- Second play scans every host in `security_patching_target_group` before any package mutation is allowed.
- Third play aggregates scan completeness, advisory drift, and next-action classification on `localhost`. If any target requires reconciliation, package mutation is disabled for the entire request.
- Fourth play runs `tasks_from: patch` serially. Exact, DNF-applicable requests mutate one host at a time; reconciliation-only requests preserve every host's authoritative scan facts without running DNF.
- A drifted or non-update current state is reconciliation-only: no package command runs, current scan facts are preserved, and the final play refreshes or closes the issue successfully.
- Final play returns to `localhost` and imports `tasks_from: post_result`.
- If an earlier play fails, Ansible will not reach `post_result`; the GitHub Actions cleanup step covers that failure path.

`deploy/ansible/playbooks/security-reboot.yml`

- First play targets `localhost` and imports `tasks_from: validate_request`, using the reboot approval label.
- Second play scans and validates every host in `security_patching_target_group` before any target may be mutated.
- Third play aggregates the preflight on `localhost`. The live inventory host set must exactly match the approved metadata. Any advisory drift, or any complete exact-ID scan whose current action is now `update` or `investigate`, makes the whole request reconciliation-only. A target that is still classified `reboot` must have a complete scan, reboot-eligible packages, and a proven second DNF no-op before the aggregate mutation gate can pass.
- Fourth play runs `tasks_from: reboot_cleanup` serially only when the aggregate mutation gate passes. A complete reconciliation state instead preserves every target's authoritative findings and records reboot/installonly cleanup as not attempted.
- `tasks/validate_reboot_state.yml` compares current OpenSCAP advisory IDs to approved issue metadata, classifies kernel-family eligibility, and records the DNF no-op proof before downtime.
- `tasks/reboot_cleanup.yml` records the running kernel, reboots, waits for SSH, records the new running kernel, waits for Autographs quadlet services, verifies local static Caddy health, verifies Caddy-fronted `/admin/api/health`, and runs:

  ```bash
  dnf -y remove --oldinstallonly --setopt=installonly_limit=2
  ```

- The next play re-scans hosts after an actual reboot. Reconciliation-only requests retain the already-complete preflight scan instead.
- Final play returns to `localhost` and imports `tasks_from: post_reboot_result`.
- Complete advisory drift and complete action reclassification reach `post_reboot_result` successfully. Incomplete scans, target-scope mismatches, or a failed package-family/DNF safety proof on a target that remains classified `reboot` are operational failures; Ansible will not reach `post_reboot_result`, and the GitHub Actions cleanup step covers that failure path.

`deploy/ansible/playbooks/security-patch-cleanup.yml`

- Runs only on `localhost`.
- Imports `security_patching` with `tasks_from: cleanup_failed_request`.
- Comments on the issue and removes the approval label after failed update or reboot workflows.

### Ansible role defaults and templates

`deploy/ansible/roles/security_patching/defaults/main.yml`

- Defines the GitHub API URL, repository, token, request headers, scan ID, timestamp, issue number, approver, temp file paths, Oracle OVAL URL, OpenSCAP result paths, default approval label, and default target group.
- Defines the reboot follow-up guard defaults, including `security_patching_reboot_validate_dnf_noop` and `security_patching_reboot_allowed_package_regex`.
- Defines the update and reboot approval labels plus the DNF output patterns and UEK-only package inventory used to classify each complete scan as `close`, `configure`, `update`, `reboot`, or `investigate`.
- Defines labels managed by the scanner:

  ```text
  security-patching
  production
  patch-scan-open
  approved-production-update
  approved-production-reboot
  ```

- Defines the labels applied to scanner-created or scanner-updated issues:

  ```text
  security-patching
  production
  patch-scan-open
  ```

`deploy/ansible/roles/security_patching/templates/security-report.md.j2`

- Renders the scanner issue body.
- Starts with a hidden metadata block consumed by the apply workflow.
- Then renders a public Markdown report with scan ID, timestamp, target group, host summary, a combined advisory/package summary, review checklist, and approval instructions.

`deploy/ansible/roles/security_patching/templates/security-update-result.md.j2`

- Renders the success or partial-success result comment after the apply playbook reaches `post_result`.
- Includes approver, scan ID, target group, optional workflow run URL, per-host update counts, and remaining findings.

`deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2`

- Renders the success or follow-up result comment after the reboot playbook reaches `post_reboot_result`.
- Includes approver, scan ID, target group, optional workflow run URL, kernel before/after reboot, installonly cleanup status, and remaining findings.

## Approval model

The scanner creates an issue when production hosts fail Oracle Linux OVAL definitions during an OpenSCAP scan.

To approve the proposed update set, apply this label to the issue:

```text
approved-production-update
```

To approve a production reboot and old installonly kernel cleanup after reviewing a patch result that indicates the remaining findings need reboot follow-up, apply this label to the issue:

```text
approved-production-reboot
```

The apply workflow validates that:

1. the actor is listed in `.github/production-patch-approvers.yml`,
2. the issue is open,
3. the issue has the scanner label `security-patching`,
4. the issue has the approval label,
5. the issue contains the scanner metadata block,
6. the target group matches the workflow target, and
7. current complete scan facts are available before package mutation is considered.

If the advisory set has drifted, the workflow applies no packages. It treats the fresh OpenSCAP scan as authoritative, refreshes or closes the same issue, consumes the stale approval through the desired label set, posts the classified next action, and completes successfully when reconciliation succeeds.

The reboot workflow uses the same approver allowlist and scanner metadata validation but a separate label so package remediation approval never implies downtime. It performs a fresh all-host pre-reboot OpenSCAP check and requires the live target set to match the approved instance set. Advisory-set drift and a complete exact-ID scan reclassified to `update` or `investigate` are successful reconciliation outcomes: the workflow reboots no hosts, refreshes or closes the issue from the aggregate current scan, and publishes the current action and approval label. Only targets still classified `reboot` must pass the kernel-family package guard and a second DNF no-op proof before downtime. The reboot path is intended for cases where DNF has already installed all applicable updates, but OpenSCAP still reports installed stale installonly kernel packages or the system must boot into a newly installed kernel before old kernel cleanup can finish.

## Scan flow

The scan path starts in `.github/workflows/weekly-security-scan.yml`, then runs `deploy/ansible/playbooks/security-scan.yml`.

`tasks/scan.yml` runs on each runtime host through delegated runner-side OpenSCAP tooling:

1. Verifies `oscap-ssh` is available on the workflow runner.
2. Downloads Oracle's OL10 ELSA OVAL file from `https://linux.oracle.com/security/oval/com.oracle.elsa-ol10.xml.bz2`. This release-scoped feed is intentionally used instead of `com.oracle.elsa-all.xml.bz2`; the all-releases file is over 200 MB uncompressed, while the OL10 feed is about 20 MB and contains only the definitions relevant to the production OS major release.
3. Decompresses the OVAL XML on the runner.
4. Runs `oscap-ssh --sudo <user>@<host> <port> oval eval --skip-validation --results <xml> --report <html> <oval xml>`.
5. Accepts return code `0` or `2`; OpenSCAP returns `2` when evaluated definitions report findings.
6. Parses the OpenSCAP result XML with `scripts/oracle_linux_oscap_results.py`.
7. Builds these host facts:

   ```yaml
   security_patching_update_entries:
     - advisory_id: ELSA-...
       severity: Important
       cves:
         - CVE-...
       packages:
         - glibc
       package_count: 1
       ksplice_aware: true
   security_patching_update_advisory_ids:
     - ELSA-...
   security_patching_update_ksplice_aware_advisory_ids:
     - ELSA-...
   security_patching_scan_source: openscap-oval
   ```
8. Probes the current advisory IDs with non-mutating `dnf --assumeno upgrade-minimal --security --advisories=...` and classifies the next action:

   | Classification | Evidence | Issue action |
   |---|---|---|
   | `close` | Complete OpenSCAP scan has no findings | Close an existing scanner issue |
   | `configure` | One or more managed RHCK packages are installed on the UEK-only runtime | Offer no approval label; run deployment convergence, then re-scan |
   | `update` | DNF reports an advisory-scoped package transaction | Offer `approved-production-update` |
   | `reboot` | DNF proves a no-op and every package is in the UEK reboot/installonly family allowlist | Offer `approved-production-reboot` |
   | `investigate` | Missing package metadata, mixed no-op findings, failed/unrecognized DNF evidence, or conflicting host classifications | Offer no approval label |

The runtime host must have `openscap-scanner` installed so `oscap-ssh` can execute `oscap` remotely. The base deployment role installs that package during instance setup. The workflow inventory supplies `ansible_user`, and the workflow passes a temp deploy key through `SSH_ADDITIONAL_OPTIONS`; local runs can omit `ansible_user` and let SSH config provide `User` and `IdentityFile` for the production IP.

`tasks/create_issue.yml` runs on `localhost` after all hosts have scan facts:

1. Requires `GITHUB_REPOSITORY` and `GH_TOKEN`.
2. Builds `security_patching_hosts_with_findings` from hosts whose `security_patching_update_entries` list is non-empty.
3. Records a debug message if no hosts have findings, then still searches for an existing scanner issue so a stale issue can be refreshed to an empty metadata set and closed.
4. Records OpenSCAP advisory detail from the host facts.
5. Keeps only complete parser output. If OpenSCAP produces unmapped true definitions, unknown/error definition states, missing result states, or no evaluated definition results, the parser exits non-zero and no approval issue is published.
6. Ensures the managed GitHub labels exist. GitHub `422` is accepted so already-existing labels do not fail the scan.
7. Renders `security-report.md.j2` to a private temp file.
8. Searches open issues labeled `security-patching` and `patch-scan-open`.
9. Filters out pull requests and selects existing issues whose body contains the same target group marker.
10. Updates the first matching open issue if findings remain, creates a new issue when needed, or closes the matching issue when the complete scan is clean.

This means weekly scans converge on one open issue per target group instead of creating duplicate open scan issues.
Existing issue updates replace the full body and label set. The replacement labels are only `security-patching`, `production`, and `patch-scan-open`, so a stale `approved-production-update` label is removed whenever a scan rewrites the issue.

## Scanner issue format

Scanner-created issues are titled:

```text
Production security update report - <UTC timestamp>
```

The body begins with a hidden YAML metadata block:

```markdown
<!-- autographs-security-patch-metadata
scan_id: "security-scan-<github run id>"
created_at: "<YYYY-MM-DDTHH:MM:SSZ>"
target_group: "runtime"
approval_label: "approved-production-update"
next_action: "update"
instances:
  production:
    advisory_ids:
      - "ELSA-2026-500006"
-->
```

That block is the contract between scanner and updater. The apply workflow uses it to identify the approved scan, target group, host list, and exact advisory IDs.

The visible issue body contains:

- `# Production security update report`
- `Scan ID`, `Generated`, and `Target group`
- an OpenSCAP advisory status of `complete`; degraded parser output fails the scan before issue creation
- a summary table:

  ```markdown
  | Instance | OpenSCAP findings | Advisories | Ksplice-specific OVAL findings |
  |---|---:|---:|---:|
  | `production` | 3 | 3 | 1 |
  ```

- a per-host advisory summary with affected package count and package sample:

  ```markdown
  | Advisory | Severity | CVEs | Ksplice-specific OVAL | Affected packages | Package sample | Summary |
  |---|---|---|---:|---:|---|---|
  | [ELSA-...](...) | Important | [CVE-...](...) | true | 2 | `glibc`, `glibc-common` | ELSA-... security update |
  ```

- a review checklist
- a classified next action: normal update approval, separate reboot approval, investigation with no approval label, or automatic clean closure

Long CVE and package lists are sampled in the visible table so large advisory sets fit inside GitHub issue body limits. The complete approved advisory ID set is preserved in hidden metadata for drift protection.

## Host inventory

Both workflows use the same host alias to avoid exposing the raw production IP address in GitHub issues:

```ini
[runtime]
production ansible_host=<runtime_public_ip from Terraform, or VM_PUBLIC_IP fallback> ansible_user=<DEPLOY_SSH_USER or opc>
```

The workflows first read Terraform state and use the `runtime_public_ip` output for this inventory host. `VM_PUBLIC_IP` remains a fallback repository variable for unusual cases where Terraform state is unavailable or does not yet expose the output.

## Apply flow

The apply path starts when an allowed operator applies `approved-production-update` to a scanner issue.

`tasks/validate_request.yml` runs first on `localhost`:

1. Requires repository, token, issue number, and approver.
2. Loads `.github/production-patch-approvers.yml`.
3. Asserts the label actor is in `production_patch_approvers`.
4. Reads the GitHub issue through the API.
5. Extracts issue label names.
6. Requires the issue to be open, labeled `security-patching`, and labeled with the approval label.
7. Extracts exactly one hidden metadata block matching:

   ```text
   <!-- autographs-security-patch-metadata
   ...
   -->
   ```

8. Parses the metadata block with `from_yaml`.
9. Records `security_patching_request_scan_id`, `security_patching_request_instances`, and `security_patching_request_target_group`.
10. Requires the metadata target group to match the workflow target group.
11. Requires at least one instance in metadata.
12. Comments that the update request was accepted.

The accepted-request comment format is:

```markdown
Production security update request accepted.

- Approved by: `<actor>`
- Scan ID: `<scan id>`
- Target group: `<target group>`
```

After validation, `security-patch.yml` scans each runtime host again by importing `tasks/scan.yml`. That fresh scan produces the current OpenSCAP advisory IDs used for drift detection.

`tasks/patch.yml` then runs per host:

1. Looks up approved advisory IDs from the scanner metadata for the current `inventory_hostname`.
2. Builds current advisory IDs from the fresh OpenSCAP scan.
3. Compares current and approved advisory IDs and reads the current scan classification.
4. If the sets drifted, or the current action is not `update`, marks the host reconciliation-only and preserves current entries, advisory details, scan status, next action, and approval label without running DNF.
5. For an exact, DNF-applicable match, preserves pre-update entries.
6. Checks whether the Oracle Ksplice client is present and records that availability for the result comment. Ksplice remains report-only.
7. Applies only approved advisories through DNF's advisory-scoped update path with:

   ```bash
   dnf -y upgrade-minimal --security --advisories=<comma-separated ELSA IDs>
   ```

8. Re-scans the host after remediation.
9. Preserves post-update entries, advisory details, advisory IDs, scan status, and the newly classified next action only after the post-update scan completes.

## Update behavior

The apply playbook treats OpenSCAP as the authority for detection and closure. DNF is the only mutating remediation engine in the approval workflow because `ksplice all upgrade` is not scoped to the approved advisory ID set. The workflow still reports Ksplice availability and Ksplice-specific OVAL findings so a future Ksplice-specific approval path can be added without weakening the current approval boundary.

The workflow runs hosts serially and re-scans after applying updates. It reconciles the full issue body, labels, and open/closed state with one idempotent GitHub `PATCH`, so the triggering approval label is consumed by the desired label set. It then comments the result. If findings remain, the same issue contains only the authoritative remaining advisory set and its classified next action.

When the remaining findings are UEK installonly findings and DNF proves there is no package work, the refreshed issue offers only the separate `approved-production-reboot` label. Installed RHCK packages take precedence over DNF work and produce the `configure` action with no approval label; deployment removes that drift without rebooting, after which a fresh scan can classify the remaining UEK state normally. Mixed, incomplete, or unrecognized states offer no approval label and require investigation. The reboot workflow independently rechecks that DNF is a no-op for the approved advisories, boots the instance into the newest installed UEK, waits for Autographs health, removes old installonly kernels, re-runs OpenSCAP, and refreshes or closes the same issue.

## Reboot flow

The reboot path starts when an allowed operator applies `approved-production-reboot` to a scanner issue.

`tasks/validate_request.yml` runs first on `localhost` with the reboot approval label and the same scanner metadata contract used by the update workflow. It confirms the actor is allowed, the issue is open, the issue is scanner-created, the reboot approval label is present, and metadata targets the requested group.

The reboot playbook scans each runtime host again before downtime. `tasks/validate_reboot_state.yml` first preserves that authoritative scan. Advisory-ID drift, or an exact advisory set whose complete scan is now classified `configure`, `update`, or `investigate`, marks the request for successful no-mutation reconciliation. For an exact advisory set that is still classified `reboot`, the advisory package names must match the configured UEK package-family regex and this second DNF check must report no remaining package work:

```bash
dnf --assumeno upgrade-minimal --security --advisories=<comma-separated ELSA IDs>
```

If the complete scan's DNF classification now reports package work, the current action becomes `update`; if its evidence is incomplete, mixed, or unrecognized, the action becomes `investigate`. Either complete action change disables reboot and installonly cleanup for the full target group and reaches `post_reboot_result`, which refreshes the issue with the current action and label. Incomplete scans, or a failed package-family or second DNF no-op safety proof after a target remains classified `reboot`, fail before downtime and use cleanup to remove the reboot approval label. If a host in the target group has no approved findings and remains clean, it records a skipped reboot state instead of rebooting.

`tasks/reboot_cleanup.yml` then runs per runtime host that still has approved findings:

1. Records the running kernel before reboot.
2. Reboots the host and waits for SSH to return.
3. Records the running kernel after reboot.
4. Waits for `autographs-controller.service` and `autographs-caddy.service`.
5. Verifies `http://127.0.0.1:8081/manifest.json`.
6. Verifies Caddy-fronted `https://<AUTOGRAPHS_DOMAIN>/admin/api/health` by resolving the configured domain to `127.0.0.1` on the host.
7. Removes old installonly kernel packages with:

   ```bash
   dnf -y remove --oldinstallonly --setopt=installonly_limit=2
   ```

After cleanup, the playbook re-runs the OpenSCAP scan. `tasks/post_reboot_result.yml` refuses to publish a result unless all hosts have complete post-reboot scan facts. It applies the same five-way next-action classifier and desired-state issue `PATCH`; remaining findings or RHCK drift stay open with accurate configure/update/reboot/investigate guidance, while a clean UEK-only scan closes the issue.

The reboot result comment includes the kernel before reboot, kernel after reboot, whether installonly cleanup changed anything, and remaining OpenSCAP finding counts.

## Result comment format

When the apply playbook reaches `tasks/post_result.yml`, it:

1. Refuses to publish a result unless every target host has complete post-update OpenSCAP scan facts.
2. Builds `security_patching_remaining_hosts` from hosts with non-empty `security_patching_post_update_entries`.
3. Aggregates host classifications; conflicting host actions become `investigate`.
4. Renders `security-report.md.j2` from the authoritative post-update facts, including an empty metadata set for a clean scan.
5. Reconciles title, body, labels, and open/closed state with one idempotent `PATCH`.
6. Renders and posts `security-update-result.md.j2` with the classified follow-up.

The result comment begins:

```markdown
## Production security update result

- Approved by: `<actor>`
- Scan ID: `<scan id>`
- Target group: `<target group>`
- Workflow run: <actions run url>
```

It then includes a per-host table:

```markdown
| Instance | Ksplice available | Ksplice apply mode | DNF updated | Approved advisories | Remaining OpenSCAP findings |
|---|---:|---|---:|---:|---:|
| `production` | true | report_only | true | 3 | 0 |
```

If the post-update scan is clean, the comment says:

```text
Post-update scan is clean for the approved target group. This issue will be closed automatically.
```

If findings remain, the comment lists the hosts, notes that the issue body was refreshed with the remaining advisory set, and leaves the issue open:

```markdown
Post-update scan still reports findings on:

- `production`: 1 remaining security update(s)

This issue has been refreshed with the remaining findings and is intentionally left open for follow-up review.
Advisory-scoped DNF still reports package work. Apply `approved-production-update` after reviewing the refreshed advisory set.
```

When the reboot playbook reaches `tasks/post_reboot_result.yml`, it follows the same refresh-or-close issue behavior and posts a reboot-specific table:

```markdown
| Instance | Rebooted | Kernel before | Kernel after | Installonly cleanup | Remaining OpenSCAP findings |
|---|---:|---|---|---:|---:|
| `production` | true | `6.12.0-204.92.4.3.el10uek.x86_64` | `6.12.0-205.92.4.2.el10uek.x86_64` | true | 0 |
```

## Failure cleanup behavior

If validation, SSH, an incomplete OpenSCAP scan, DNF mutation, reboot, health checks, installonly cleanup, or another operational step fails, Ansible may not reach `post_result` or `post_reboot_result`. The GitHub Actions workflows handle that with an `always()` cleanup step. Expected update reconciliation and reboot reconciliation do not use this failure path: complete advisory drift and complete exact-ID action reclassification preserve current facts, suppress mutation across the target group, and publish through the normal result task. A failed package-family or second DNF no-op safety proof on a target that is still classified `reboot` remains an operational failure. Reboot operational failures persist bounded operator-facing context without stopping approval-label removal.

`tasks/cleanup_failed_request.yml`:

1. Requires repository, token, and issue number.
2. Builds the issue URL, workflow run URL, and approval label URL.
3. Removes or attempts to remove the approval label so the failed request cannot be retried accidentally by a stale label.
4. Comments on the issue with bounded failure details and the label cleanup outcome.

Failure cleanup comment format:

```markdown
Production security update workflow did not complete successfully.

- Workflow outcome: `<outcome>`
- Workflow run: <actions run url>

The approval label has been removed so this request cannot be retried accidentally.
Re-run the scanner or re-apply the approval label after reviewing the workflow logs.
```

## GitHub labels

The scanner ensures these labels exist before creating or updating a scan issue:

| Label | Purpose |
|---|---|
| `security-patching` | Identifies scanner-created security patching issues. |
| `production` | Marks production runtime maintenance issues. |
| `patch-scan-open` | Marks an open scan finding that can be updated by later scans. |
| `approved-production-update` | Triggers the apply workflow when added by an allowed actor. |
| `approved-production-reboot` | Triggers the reboot/installonly cleanup workflow when added by an allowed actor. |

Only the first three labels are applied by the scanner to report issues. The approval labels are applied manually by an allowed operator.

## Control-plane files

The sensitive control-plane files are CODEOWNED:

- `.github/workflows/weekly-security-scan.yml`
- `.github/workflows/apply-security-updates.yml`
- `.github/workflows/reboot-security-runtime.yml`
- `.github/production-patch-approvers.yml`
- `deploy/ansible/playbooks/security-scan.yml`
- `deploy/ansible/playbooks/security-patch.yml`
- `deploy/ansible/playbooks/security-reboot.yml`
- `deploy/ansible/roles/security_patching/`

CODEOWNERS only requests ownership by default. Require CODEOWNER review through branch protection if this should be enforced before merging future changes.

## Validation commands

Use the same temp overrides as the deployment runbook when running Ansible locally from restricted shells:

```bash
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local \
ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg \
ansible-playbook --syntax-check \
  deploy/ansible/playbooks/security-scan.yml \
  deploy/ansible/playbooks/security-patch.yml \
  deploy/ansible/playbooks/security-reboot.yml \
  deploy/ansible/playbooks/security-patch-cleanup.yml
```

Run Ansible lint for the security patching surface:

```bash
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local \
ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg \
ansible-lint \
  deploy/ansible/roles/security_patching \
  deploy/ansible/playbooks/security-scan.yml \
  deploy/ansible/playbooks/security-patch.yml \
  deploy/ansible/playbooks/security-reboot.yml \
  deploy/ansible/playbooks/security-patch-cleanup.yml
```

CI also runs actionlint against workflows and Ansible syntax/lint checks through `.github/workflows/ci.yml`.

## Phase 8 scanner repair verification

Use this section after deploying the scanner repair. Do not install production updates unless a real scanner issue contains an approved advisory set that you intend to apply.

### workflow_dispatch scan proof

Trigger a manual scan and capture its run ID:

```bash
gh workflow run weekly-security-scan.yml --ref main
gh run list --workflow weekly-security-scan.yml --limit 5
gh run view <scan-run-id> --json conclusion,jobs,url
```

Expected result: the `Resolve runtime VM IP` step succeeds, the `Run security scan playbook` step succeeds, and a scanner issue is created or updated when package inventory exists.

### OpenSCAP parser review

Exercise the OpenSCAP parser locally:

```bash
python3 -m unittest scripts/test_oracle_linux_oscap_results.py
```

Expected result: the parser maps true OpenSCAP OVAL definitions to ELSA advisory IDs, CVEs, affected package names, errata links, and Ksplice-specific OVAL status.

### same-issue update review

Run the scanner twice while an open scanner issue exists:

```bash
gh workflow run weekly-security-scan.yml --ref main
gh run list --workflow weekly-security-scan.yml --limit 5
gh issue list --state open --label security-patching --label patch-scan-open
gh issue view <scanner-issue-number> --json title,labels,body,updatedAt,url
```

Expected result: the same open scanner issue for `target_group: "runtime"` is updated instead of creating duplicate open scan issues.

### stale approval removal

Only perform this proof in production when the issue represents a real advisory set and an allowed operator intentionally wants to exercise the approval path. Adding `approved-production-update` to a real scanner issue triggers the apply workflow.

```bash
gh issue edit <scanner-issue-number> --add-label approved-production-update
gh workflow run weekly-security-scan.yml --ref main
gh issue view <scanner-issue-number> --json labels,body
```

Expected result after the scanner update: labels are reset to `security-patching`, `production`, and `patch-scan-open`; stale `approved-production-update` and `approved-production-reboot` labels are absent from the open scanner issue.

### dry-run apply-path exercise

Use check mode only, and only with a real scanner issue whose advisory set you reviewed:

```bash
GH_TOKEN=<token> \
GITHUB_REPOSITORY=jetsaredim/autographs \
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local \
ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote \
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg \
ansible-playbook --check \
  deploy/ansible/playbooks/security-patch.yml \
  --extra-vars security_patching_issue_number=<scanner-issue-number> \
  --extra-vars security_patching_approver=<github-username> \
  --extra-vars security_patching_target_group=runtime
```

Expected result: validation reads the advisory metadata, the runtime host is re-scanned, and drift checks compare the current OpenSCAP advisory IDs to the approved advisory IDs before any update task can proceed.
