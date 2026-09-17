---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-17T09:42:14Z
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
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-17T09:42:14Z
**Depth:** deep
**Files Reviewed:** 36
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

All five blocker findings from iterations 1 and 2 are resolved. Target scope fails closed, advisory drift and complete action reclassification reconcile successfully without mutation, reboot mutation has an all-host gate, and scanner/update/reboot report paths now publish the resolved approval label only after Ansible has assigned it. The full configured Python and Ansible validation suites pass. Two non-blocking but actionable quality defects remain: the runbook still describes the superseded reboot failure behavior, and one validation play depends on facts leaked by earlier plays rather than defining an isolated fixture.

## Warnings

### WR-01: Reboot runbook still documents reclassification as a failed workflow

**File:** `docs/security-patching.md:126-141,218,403-415,490-500`

**Issue:** The implementation now treats a complete exact-advisory scan whose action changes from `reboot` to `update` or `investigate` as successful no-mutation reconciliation. The runbook still says every non-drifted target must remain reboot-eligible, and explicitly says that if DNF would apply package updates the reboot workflow fails and enters cleanup. It only identifies advisory-ID drift as an expected success path. Operators following this text will expect a red run and manual rescan when the actual workflow refreshes the issue successfully with a new action and approval label.

**Fix:** Document both reconciliation triggers: advisory-set drift and complete action reclassification. State that either disables reboot/installonly mutation for the full target group and reaches `post_reboot_result`; distinguish these from incomplete scans or a failed second safety proof after a target is still classified `reboot`, which remain operational failures. Update the playbook overview, approval model, reboot flow, and failure-cleanup sections consistently.

### WR-02: Non-kernel reboot validation test passes through leaked cross-play facts

**File:** `deploy/ansible/playbooks/security-reboot-state-validate-test.yml:551-578`

**Issue:** The final non-kernel fixture does not set `security_patching_scan_status`, `security_patching_next_action`, or its action label. Since the new implementation gates package eligibility on `next_action == 'reboot'`, this play passes in the normal full-file run only because earlier localhost plays leave `security_patching_next_action: reboot` and related facts behind. Running the play in isolation with `--start-at-task 'Record non-kernel approved reboot issue metadata'` deterministically fails because `security_patching_reboot_disallowed_packages` is undefined. This hides fixture-order regressions and means the assertion is not independently testing the intended safety condition.

**Fix:** Make the fixture self-contained by explicitly setting a complete scan and the `reboot` action/label when testing the defense-in-depth package guard, or change it into an explicit `investigate` reclassification fixture with matching assertions. Ensure each play initializes every role input it relies on, then add an isolated execution check or remove assertions that depend on prior-play state.

---

_Reviewed: 2026-09-17T09:42:14Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
