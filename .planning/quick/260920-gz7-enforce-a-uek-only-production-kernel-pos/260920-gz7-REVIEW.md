---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-20T20:20:33Z
depth: deep
files_reviewed: 25
files_reviewed_list:
  - controller/tests/runtime_kernel_persistence.rs
  - deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  - deploy/ansible/playbooks/security-create-issue-status-validate-test.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/roles/autographs_deploy/defaults/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_images.yml
  - deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_ownership.yml
  - deploy/ansible/roles/autographs_deploy/tasks/derive_dnf_exclusions.yml
  - deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml
  - deploy/ansible/roles/autographs_deploy/tasks/main.yml
  - deploy/ansible/roles/security_patching/defaults/main.yml
  - deploy/ansible/roles/security_patching/tasks/classify_findings.yml
  - deploy/ansible/roles/security_patching/tasks/create_issue.yml
  - deploy/ansible/roles/security_patching/tasks/patch.yml
  - deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml
  - deploy/ansible/roles/security_patching/tasks/post_result.yml
  - deploy/ansible/roles/security_patching/tasks/scan.yml
  - deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml
  - deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2
  - deploy/ansible/roles/security_patching/templates/security-report.md.j2
  - deploy/ansible/roles/security_patching/templates/security-update-result.md.j2
  - docs/deployment-runbook.md
  - docs/security-patching.md
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-20T20:20:33Z
**Depth:** deep
**Files Reviewed:** 25
**Status:** issues_found

## Summary

The three original findings are fixed at their direct sites: destructive RHCK removal now proves both UEK image existence and installed UEK RPM ownership first; unsafe running/default kernels classify as recovery rather than convergence during a scanner run; and DNF exclusions are merged from every active `[main]` declaration without leaking repository-section values. The narrowed Rust contract test correctly permits only the two deliberate non-fatal RPM ownership probes, the four changed Ansible behavior tests pass locally, the Rust contract test passes, and GitHub CI is green.

The full workflow trace still found three actionable gaps. Update/reboot result reconciliation can discard an unsafe kernel-only state, multi-host result aggregation can reintroduce the configure loop that the scanner path fixed, and the explicit RHCK family omits Oracle Linux 10 RHCK packages that are currently shipped.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: Result reconciliation can close an issue while kernel recovery is still required

**Files:** `deploy/ansible/roles/security_patching/tasks/post_result.yml:32-38`, `deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml:34-40`

**Issue:** The initial scanner includes a host when either running/default UEK validation fails, but both result paths decide that a host remains actionable using only OpenSCAP entries and installed RHCK packages. A fresh approval-time scan can therefore classify a host as `investigate` solely because its running or default kernel image is unsafe, preserve an empty finding/RHCK set during no-mutation reconciliation, and then have `post_result` or `post_reboot_result` set `remaining_hosts` to empty and close the issue as clean. This drops the recovery instructions and contradicts the authoritative classification.

**Fix:** Preserve post-update/post-reboot running/default UEK validity and kernel identity facts alongside the existing RHCK facts. Require those snapshot facts to be defined, include either invalid UEK state in `security_patching_remaining_hosts`, mirror them when rendering the refreshed report, and add update and reboot reconciliation fixtures where OpenSCAP/RHCK lists are empty but one kernel validity flag is false.

### WR-02: Multi-host result aggregation lets `configure` override `investigate`

**Files:** `deploy/ansible/roles/security_patching/tasks/post_result.yml:74-85`, `deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml:72-83`

**Issue:** `create_issue.yml` correctly makes `investigate` take precedence over `configure`, but both result aggregators check for `configure` first. With one verified UEK host carrying removable RHCK drift and another host requiring kernel recovery, the refreshed issue is classified `configure`. Its global next action tells the operator to run deployment convergence even though the unsafe host's per-host section says not to do that; deployment then fails its UEK guard and the workflow returns to the same state. This is the non-convergent loop that the WR-01 fix was intended to eliminate.

**Fix:** Apply the same precedence used by `create_issue.yml`: after `close`, select `investigate` whenever any remaining host is `investigate`, then `configure`, then a single common action, otherwise `investigate`. Add mixed `configure` + `investigate` fixtures to both post-update and post-reboot result tests and assert that no approval label or convergence instruction is emitted.

### WR-03: The managed RHCK family omits shipped RHCK kernel packages

**Files:** `deploy/ansible/roles/autographs_deploy/defaults/main.yml:64-78`, `deploy/ansible/roles/security_patching/defaults/main.yml:49-63`

**Issue:** Cleanup, DNF prevention, and scanner detection all depend on these exact lists, but Oracle Linux 10 currently ships additional RHCK packages including `kernel-uki-virt`, `kernel-uki-virt-addons`, `kernel-debug-uki-virt`, and `kernel-debug-devel-matched`. If any is installed, deployment neither removes nor excludes it and the scanner does not report it, so the advertised UEK-only posture can report clean while RHCK kernel artifacts remain and continue updating.

**Fix:** Add every RHCK boot/debug/development package that the policy intends to reject to both synchronized defaults lists (while continuing to preserve shared `kernel-headers`, `kernel-tools`, and `kernel-tools-libs`). Add explicit contract assertions/fixtures for the omitted UKI and matched-debug package names so future Oracle package-family additions cannot silently bypass cleanup and scanning.

---

_Reviewed: 2026-09-20T20:20:33Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
