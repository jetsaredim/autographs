---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-16T09:57:23Z
depth: deep
files_reviewed: 30
files_reviewed_list:
  - .github/workflows/apply-security-updates.yml
  - .github/workflows/ci.yml
  - .github/workflows/reboot-security-runtime.yml
  - .github/workflows/weekly-security-scan.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-patch.yml
  - deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-status-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-result-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-state-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml
  - deploy/ansible/roles/security_patching/defaults/main.yml
  - deploy/ansible/roles/security_patching/tasks/classify_findings.yml
  - deploy/ansible/roles/security_patching/tasks/classify_update_request.yml
  - deploy/ansible/roles/security_patching/tasks/cleanup_failed_request.yml
  - deploy/ansible/roles/security_patching/tasks/create_issue.yml
  - deploy/ansible/roles/security_patching/tasks/patch.yml
  - deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml
  - deploy/ansible/roles/security_patching/tasks/post_result.yml
  - deploy/ansible/roles/security_patching/tasks/scan.yml
  - deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml
  - deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2
  - deploy/ansible/roles/security_patching/templates/security-report.md.j2
  - deploy/ansible/roles/security_patching/templates/security-update-result.md.j2
  - docs/security-patching.md
  - scripts/oracle_linux_oscap_results.py
  - scripts/test_oracle_linux_oscap_results.py
  - scripts/test_security_patching_create_issue_tasks.py
findings:
  critical: 3
  warning: 0
  info: 0
  total: 3
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-16T09:57:23Z
**Depth:** deep
**Files Reviewed:** 30
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

The update path now reconciles drift without package mutation and the shared concurrency group prevents issue writers from racing. The reboot path, however, still treats advisory drift as a failed workflow and validates each host immediately before mutating it. In addition, all issue-reconciliation paths treat an absent or empty target group as a clean scan. These defects can produce a red expected-drift run, partial multi-host downtime, or a falsely closed security issue.

## Critical Issues

### CR-01: Empty target groups are treated as authoritative clean scans

**Severity:** BLOCKER

**File:** `deploy/ansible/roles/security_patching/tasks/create_issue.yml:8-39`

**Issue:** Every host loop uses `groups[security_patching_target_group] | default([])`, but no task first requires that the target group exists and contains at least one host. With a missing or empty inventory group, `security_patching_incomplete_scan_hosts` and `security_patching_hosts_with_findings` both remain empty, so `create_issue.yml` proceeds down the clean path and closes an existing scanner issue. The same fail-open pattern appears in `classify_update_request.yml:5-27`, `post_result.yml:1-31`, and `post_reboot_result.yml:1-31`; an update or reboot request can therefore render an empty metadata set and close the issue without any current OpenSCAP result. `validate_request.yml` only proves that issue metadata has instances, not that those instances match the live inventory group.

**Fix:** Add a shared fail-closed target-scope assertion before scan classification or publication. Require the target group to exist, require at least one live target, and for approval workflows require the sorted inventory host set to equal the metadata instance-key set. Add fixtures proving that missing, empty, and host-mismatched target groups fail before any GitHub issue `PATCH`.

### CR-02: Reboot advisory drift still deliberately fails the workflow

**Severity:** BLOCKER

**File:** `deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml:64-98`

**Issue:** The reboot validator writes a refresh payload and then unconditionally asserts that the current and approved advisory sets match. GitHub Actions subsequently forces a failed conclusion at `.github/workflows/reboot-security-runtime.yml:109-111`. Cleanup may refresh the issue, but the requested contract is that an authoritative OpenSCAP drift is a successful reconciliation outcome rather than a failed workflow. The issue template also states broadly that an approval workflow reconciles drift without package changes, which is false for reboot approvals.

**Fix:** Model reboot drift as a reconciliation-only state, analogous to `classify_update_request.yml`: preserve the current complete OpenSCAP facts, set reboot/cleanup facts to not attempted, skip all downtime and package cleanup, reconcile the issue body and labels through the normal result publisher, post the added/removed advisory details, and let the workflow succeed. Reserve the failure cleanup path for incomplete scans and operational errors. Replace the current rejection fixture with success-path drift and clean-drift reconciliation tests.

### CR-03: Reboot validation and mutation are interleaved per host

**Severity:** BLOCKER

**File:** `deploy/ansible/playbooks/security-reboot.yml:12-31`

**Issue:** The serial reboot play scans and validates one host and immediately reboots it before examining the next host. If a later target has drift, incomplete package metadata, DNF work, or another validation failure, earlier targets have already incurred downtime and installonly cleanup. This violates the all-host mutation gate implemented for updates. The failure refresh payload is also a single file populated by the failing host, so cleanup can replace the issue with only that host's findings after earlier hosts were already mutated.

**Fix:** Split reboot processing into an all-host preflight play, a localhost aggregate decision, and a separate serial mutation play. No host may reboot unless every target has a complete scan, an exact approved advisory set, reboot-eligible packages, and a proven DNF no-op. If any target needs reconciliation, skip mutation for the whole group and publish a complete aggregate current-state report. Add a two-host regression fixture where the first host is eligible and the second drifts, asserting that neither host reaches `reboot_cleanup` and both hosts remain represented in refreshed metadata.

---

_Reviewed: 2026-09-16T09:57:23Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
