---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-16T21:25:11Z
depth: deep
files_reviewed: 36
files_reviewed_list:
  - .github/workflows/apply-security-updates.yml
  - .github/workflows/ci.yml
  - .github/workflows/reboot-security-runtime.yml
  - .github/workflows/weekly-security-scan.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-patch.yml
  - deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-status-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-result-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-state-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/playbooks/security-target-scope-validate-test.yml
  - deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml
  - deploy/ansible/roles/security_patching/defaults/main.yml
  - deploy/ansible/roles/security_patching/tasks/classify_findings.yml
  - deploy/ansible/roles/security_patching/tasks/classify_reboot_request.yml
  - deploy/ansible/roles/security_patching/tasks/classify_update_request.yml
  - deploy/ansible/roles/security_patching/tasks/cleanup_failed_request.yml
  - deploy/ansible/roles/security_patching/tasks/create_issue.yml
  - deploy/ansible/roles/security_patching/tasks/patch.yml
  - deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml
  - deploy/ansible/roles/security_patching/tasks/post_result.yml
  - deploy/ansible/roles/security_patching/tasks/scan.yml
  - deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml
  - deploy/ansible/roles/security_patching/tasks/validate_request.yml
  - deploy/ansible/roles/security_patching/tasks/validate_target_scope.yml
  - deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2
  - deploy/ansible/roles/security_patching/templates/security-report.md.j2
  - deploy/ansible/roles/security_patching/templates/security-update-result.md.j2
  - deploy/ansible/test-fixtures/security-target-scope-inventory.yml
  - docs/security-patching.md
  - scripts/oracle_linux_oscap_results.py
  - scripts/test_oracle_linux_oscap_results.py
  - scripts/test_security_patching_create_issue_tasks.py
findings:
  critical: 2
  warning: 0
  info: 0
  total: 2
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-16T21:25:11Z
**Depth:** deep
**Files Reviewed:** 36
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

The three iteration-1 blockers are materially resolved: target scope now fails closed, advisory-ID drift on the reboot path is a successful no-mutation reconciliation, and reboot mutation is gated behind an all-host preflight. Two blocker-level state defects remain. Regenerated issue bodies lose their actionable approval label because several `set_fact` tasks read a sibling fact before Ansible assigns it, and the reboot aggregate still fails complete authoritative scans when the advisory IDs are unchanged but their current action has changed.

## Critical Issues

### CR-01: Regenerated reports publish an empty approval label

**Severity:** BLOCKER

**Files:** `deploy/ansible/roles/security_patching/tasks/create_issue.yml:72-81`, `deploy/ansible/roles/security_patching/tasks/post_result.yml:83-92`, `deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml:73-82`

**Issue:** Each cited `set_fact` task assigns `security_patching_next_action_label` and, in the same task, assigns `security_patching_report_approval_label` from `security_patching_next_action_label`. Ansible templates all `set_fact` arguments against the pre-task variable context; sibling assignments are not visible during that task. On these localhost paths the pre-task value comes from the role default and is empty, so a scan or reconciliation whose aggregate action is `update` or `reboot` renders hidden metadata with `approval_label: ""` and visible guidance to apply an empty label. The result comment happens to use the newly assigned next-action label later, but the regenerated issue itself is not independently actionable and the next approval metadata is wrong. Existing tests only assert the advisory set and prose, so they do not catch the empty parsed approval label.

**Fix:** Split label resolution into two tasks: first set `security_patching_next_action_label`, then set `security_patching_report_approval_label` (and the reconciled labels) from that established fact. Alternatively, compute both directly from the action-to-label mapping without a sibling reference. Extend scanner, post-update, and post-reboot render fixtures to parse the hidden metadata and assert the exact expected approval label, and assert the visible next-action line contains that label.

### CR-02: Complete reboot reclassification still takes the failure path when advisory IDs are unchanged

**Severity:** BLOCKER

**Files:** `deploy/ansible/roles/security_patching/tasks/classify_reboot_request.yml:18-45`, `deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml:219-230`

**Issue:** The reboot aggregate treats only advisory-ID drift as reconciliation. Any exact-ID host whose fresh complete scan is no longer classified as `reboot` has `security_patching_reboot_preflight_ready: false`, is added to `security_patching_reboot_blocked_hosts`, and fails the playbook assertion. This occurs when the same advisories now have DNF package work (`next_action: update`), have newly incomplete/disallowed package metadata (`investigate`), or otherwise no longer qualify for reboot cleanup. Those are authoritative, complete, no-mutation scan outcomes, just like ID drift, but the workflow goes red and cleanup merely removes the trigger label. It never reaches `post_reboot_result`, so the issue keeps the stale reboot guidance instead of being regenerated with the current update/investigate action. The update workflow already handles the symmetric exact-ID action change as successful reconciliation.

**Fix:** Track complete exact-ID hosts whose current `security_patching_next_action` is not `reboot` as reconciliation hosts, not operationally blocked hosts. Disable mutation for the entire group, retain true failures for missing/incomplete scans or a failed second safety proof after a `reboot` classification, and let the aggregate facts reach `post_reboot_result`. Render an explicit no-reboot/no-cleanup reconciliation message for this case. Add fixtures for an unchanged advisory set changing from `reboot` to `update` and to `investigate`; both should succeed, mutate no host, refresh the issue action/label, and include every target in the aggregate report.

---

_Reviewed: 2026-09-16T21:25:11Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
