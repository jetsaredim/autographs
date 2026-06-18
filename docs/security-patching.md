# Production security update workflow

This runbook describes the production OS security update scanner and one-click approval workflow.

## Overview

The automation is split into two GitHub Actions workflows:

- `Weekly Security Scan` runs on a weekly schedule or manually and invokes `deploy/ansible/playbooks/security-scan.yml`.
- `Apply Security Updates` runs when the `approved-production-update` label is added to a scanner-created issue and invokes `deploy/ansible/playbooks/security-patch.yml`.

The workflows are intentionally thin wrappers around Ansible. Runtime host behavior, GitHub issue rendering, approval validation, drift checks, and update application live in the `security_patching` Ansible role.

The scanner and updater communicate through a GitHub issue. The scanner writes a human-readable report plus a hidden YAML metadata block into the issue body. The updater re-reads that metadata, re-scans the host, and only applies the exact package specs that are still pending.

## File map

### GitHub workflow files

`.github/workflows/weekly-security-scan.yml`

- Runs every Monday at `08:37 UTC` and also supports `workflow_dispatch`.
- Grants `contents: read` and `issues: write` so the checked-out playbook can create or update GitHub issues.
- Pins `dawidd6/action-ansible-playbook` by commit SHA.
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

- Installs local Ansible tooling for the cleanup path before the third-party Ansible action runs.
- Runs `deploy/ansible/playbooks/security-patch.yml` through the pinned Ansible action.
- Uses `continue-on-error: true` on the main update step so the workflow can still run cleanup after validation, SSH, drift, or `dnf` failures.
- Passes these extra vars to the apply playbook:

  ```text
  security_patching_issue_number=<labeled issue number>
  security_patching_approver=<label actor>
  security_patching_approval_label=approved-production-update
  security_patching_target_group=runtime
  ```

- Runs `deploy/ansible/playbooks/security-patch-cleanup.yml` with `if: always() && steps.security_update.outcome != 'success'`.
- Fails the workflow after cleanup when the main update step did not succeed.

`.github/production-patch-approvers.yml`

- Stores the GitHub usernames allowed to approve production updates by applying the approval label.
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
- Second play targets `security_patching_target_group`, runs with `become: true`, gathers facts, and uses `serial: 1`.
- On each runtime host, it imports `tasks_from: scan` to capture the current update set, then imports `tasks_from: patch`.
- Final play returns to `localhost` and imports `tasks_from: post_result`.
- If an earlier play fails, Ansible will not reach `post_result`; the GitHub Actions cleanup step covers that failure path.

`deploy/ansible/playbooks/security-patch-cleanup.yml`

- Runs only on `localhost`.
- Imports `security_patching` with `tasks_from: cleanup_failed_request`.
- Comments on the issue and removes the approval label after failed update workflows.

### Ansible role defaults and templates

`deploy/ansible/roles/security_patching/defaults/main.yml`

- Defines the GitHub API URL, repository, token, request headers, scan ID, timestamp, issue number, approver, temp file paths, default approval label, and default target group.
- Defines labels managed by the scanner:

  ```text
  security-patching
  production
  patch-scan-open
  approved-production-update
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
- Then renders a public Markdown report with scan ID, timestamp, target group, host summary, package table, review checklist, and approval instructions.

`deploy/ansible/roles/security_patching/templates/security-update-result.md.j2`

- Renders the success or partial-success result comment after the apply playbook reaches `post_result`.
- Includes approver, scan ID, target group, optional workflow run URL, per-host update counts, and remaining findings.

## Approval model

The scanner creates an issue when production hosts report available security updates from `dnf updateinfo`.

To approve the proposed update set, apply this label to the issue:

```text
approved-production-update
```

The apply workflow validates that:

1. the actor is listed in `.github/production-patch-approvers.yml`,
2. the issue is open,
3. the issue has the scanner label `security-patching`,
4. the issue has the approval label,
5. the issue contains the scanner metadata block,
6. the target group matches the workflow target, and
7. the fresh pre-apply scan exactly matches the package specs embedded in the issue.

If the package set has drifted, the workflow refuses to apply updates. Run the scanner again to generate a fresh issue.

## Scan flow

The scan path starts in `.github/workflows/weekly-security-scan.yml`, then runs `deploy/ansible/playbooks/security-scan.yml`.

`tasks/scan.yml` runs on each runtime host:

1. Verifies `dnf` is available with `dnf --version`.
2. Refreshes metadata with `dnf -q makecache --refresh`.
3. Lists available security advisories:

   ```text
   dnf -q updateinfo list --security --available
   ```

4. Accepts return code `0` or `100`.
5. Normalizes output by trimming blank lines and filtering common metadata/plugin lines.
6. Parses advisory rows into raw entries:

   ```yaml
   advisory_id: <first column>
   severity: <second column with /Sec. or /Bugfix. suffix removed>
   package_spec: <last column>
   ```

7. Reads advisory details for each unique advisory:

   ```text
   dnf -q updateinfo info <advisory_id>
   ```

8. Extracts CVE identifiers with the `CVE-YYYY-ID` regex.
9. Builds these host facts:

   ```yaml
   security_patching_update_entries:
     - advisory_id: ELSA-...
       severity: Important
       package_spec: package-version.arch
       cves:
         - CVE-...
   security_patching_update_package_specs:
     - package-version.arch
   security_patching_update_advisory_ids:
     - ELSA-...
   ```

`tasks/create_issue.yml` runs on `localhost` after all hosts have scan facts:

1. Requires `GITHUB_REPOSITORY` and `GH_TOKEN`.
2. Builds `security_patching_hosts_with_findings` from hosts whose `security_patching_update_entries` list is non-empty.
3. Stops with a debug message if no hosts have findings.
4. Ensures the managed GitHub labels exist. GitHub `422` is accepted so already-existing labels do not fail the scan.
5. Renders `security-report.md.j2` to a private temp file.
6. Searches open issues labeled `security-patching` and `patch-scan-open`.
7. Filters out pull requests and selects existing issues whose body contains the same target group marker.
8. Updates the first matching open issue if present, otherwise creates a new issue.

This means weekly scans converge on one open issue per target group instead of creating duplicate open scan issues.

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
instances:
  production:
    package_specs:
      - "package-version.arch"
-->
```

That block is the contract between scanner and updater. The apply workflow uses it to identify the approved scan, target group, host list, and exact package specs.

The visible issue body contains:

- `# Production security update report`
- `Scan ID`, `Generated`, and `Target group`
- a summary table:

  ```markdown
  | Instance | Proposed security updates | Advisories |
  |---|---:|---:|
  | `production` | 3 | 2 |
  ```

- a per-host package table:

  ```markdown
  | Package spec | Advisory | Severity | CVEs |
  |---|---|---|---|
  | `package-version.arch` | [ELSA-...](...) | Important | [CVE-...](...) |
  ```

- a review checklist
- one-click approval instructions for the `approved-production-update` label

## Host inventory

Both workflows use the same host alias to avoid exposing the raw production IP address in GitHub issues:

```ini
[runtime]
production ansible_host=<VM_PUBLIC_IP> ansible_user=<DEPLOY_SSH_USER or opc>
```

The workflows expect `VM_PUBLIC_IP` to exist as a repository variable unless the inventory is later replaced by a Terraform-output or OCI-inventory step.

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

After validation, `security-patch.yml` scans each runtime host again by importing `tasks/scan.yml`. That fresh scan produces the current package specs used for drift detection.

`tasks/patch.yml` then runs per host:

1. Looks up approved package specs from the scanner metadata for the current `inventory_hostname`.
2. Builds current package specs from the fresh scan.
3. If the host was not present in the approved metadata, records skipped state and preserves current findings for the final report.
4. If the host has approved specs, asserts the fresh package specs exactly match the approved package specs.
5. Preserves pre-update entries.
6. Applies only the approved specs with:

   ```yaml
   ansible.builtin.dnf:
     name: "{{ security_patching_approved_package_specs }}"
     state: latest
     security: true
     update_only: true
   ```

7. Re-scans the host after updates.
8. Preserves post-update entries and post-update package specs.

## Update behavior

The apply playbook uses `ansible.builtin.dnf` with:

- `security: true`
- `update_only: true`
- `state: latest`
- the exact package specs captured by the scanner issue

The workflow runs hosts serially and re-scans after applying updates. It removes the approval label after the run starts, comments the result back to the issue, and closes the issue only when the post-update scan has no remaining findings.

## Result comment format

When the apply playbook reaches `tasks/post_result.yml`, it:

1. Builds `security_patching_remaining_hosts` from hosts with non-empty `security_patching_post_update_entries`.
2. Renders `security-update-result.md.j2`.
3. Posts the rendered file as an issue comment.
4. Removes the approval label.
5. Closes the issue with `state_reason: completed` if no hosts still have findings.

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
| Instance | Updated | Approved package specs | Remaining security updates |
|---|---:|---:|---:|
| `production` | true | 3 | 0 |
```

If the post-update scan is clean, the comment says:

```text
Post-update scan is clean for the approved target group. This issue will be closed automatically.
```

If findings remain, the comment lists the hosts and leaves the issue open:

```markdown
Post-update scan still reports findings on:

- `production`: 1 remaining security update(s)

This issue is intentionally left open for follow-up review.
```

## Failure cleanup behavior

If validation, drift detection, SSH, `dnf`, or another update step fails, Ansible may not reach `post_result`. The GitHub Actions workflow handles that with an `always()` cleanup step.

`tasks/cleanup_failed_request.yml`:

1. Requires repository, token, and issue number.
2. Builds the issue URL, workflow run URL, and approval label URL.
3. Comments on the issue.
4. Removes the approval label so the failed request cannot be retried accidentally by a stale label.

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

Only the first three labels are applied by the scanner to report issues. The approval label is applied manually by an allowed operator.

## Control-plane files

The sensitive control-plane files are CODEOWNED:

- `.github/workflows/weekly-security-scan.yml`
- `.github/workflows/apply-security-updates.yml`
- `.github/production-patch-approvers.yml`
- `deploy/ansible/playbooks/security-scan.yml`
- `deploy/ansible/playbooks/security-patch.yml`
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
  deploy/ansible/playbooks/security-patch-cleanup.yml
```

CI also runs actionlint against workflows and Ansible syntax/lint checks through `.github/workflows/ci.yml`.
