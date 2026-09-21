---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-21T00:14:51Z
depth: deep
files_reviewed: 29
files_reviewed_list:
  - controller/tests/runtime_kernel_persistence.rs
  - deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  - deploy/ansible/playbooks/security-create-issue-status-validate-test.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-status-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-result-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml
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
  critical: 3
  warning: 1
  info: 0
  total: 4
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-21T00:14:51Z
**Depth:** deep
**Files Reviewed:** 29
**Status:** issues_found

## Summary

The first- and second-round direct fixes are present: deployment proves both UEK images and RPM ownership before RHCK removal; kernel-only recovery facts survive result reconciliation; `investigate` outranks `configure`; and the DNF exclusion merge preserves repeated active `[main]` declarations. The current controller, Ansible, image-build, and workflow checks pass.

The PR is not clean. Multi-host approval metadata is incompatible with the exact target-scope guard, post-reboot installonly cleanup can run after booting an unverified fallback kernel, the current Oracle Linux 10 RHCK package set is still incomplete, and the conventional-commit merge gate is failing on two commits.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Mixed clean/finding target groups cannot enter either approval workflow

**Files:** `deploy/ansible/roles/security_patching/tasks/create_issue.yml:33-41`, `deploy/ansible/roles/security_patching/templates/security-report.md.j2:9-16`, `deploy/ansible/roles/security_patching/tasks/validate_target_scope.yml:36-44`

**Issue:** The scanner puts only `security_patching_hosts_with_findings` into hidden issue metadata, while both approval paths require metadata instance keys to exactly equal every live host in the target group. A two-host group with one clean host and one host requiring update/reboot therefore produces metadata for only the finding host, and approval fails before reconciliation because the clean host is absent. This is especially likely after the new configure/recovery flow cleans one host while another still needs patching, so the advertised multi-host convergence can become non-actionable.

**Fix:** Preserve every `security_patching_target_host` in hidden metadata, using an empty advisory list for clean hosts, then make update/reboot preflight explicitly treat an empty approved/current clean host as a safe skipped target. Add clean+update, clean+reboot, and recovered+remaining-finding multi-host fixtures that reach normal reconciliation/mutation instead of failing exact scope validation.

### CR-02: Installonly cleanup runs before the rebooted kernel is proven safe

**Files:** `deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml:17-23,72-84`, `deploy/ansible/playbooks/security-reboot.yml:43-59`

**Issue:** Preflight proves the running/default images before reboot, but after reboot the workflow only records `uname -r`, checks application health, and immediately runs destructive `dnf remove --oldinstallonly`. A boot can land on a preserved RHCK rescue image or another fallback despite the pre-reboot default. In that state cleanup can discard older known-good UEK packages before the later OpenSCAP scan notices that the running kernel is not verified UEK. Application health does not prove the boot target or RPM ownership.

**Fix:** Before installonly cleanup, stat `/boot/vmlinuz-$(uname -r)`, verify its installed RPM owner is `kernel-uek*`, and verify the running release/default state is the intended safe UEK posture. Fail closed without cleanup if any proof fails, persist bounded failure context, and add a fallback/rescue fixture proving `dnf remove --oldinstallonly` is never reached.

### CR-03: The PR merge gate is currently failing

**File:** Git commit history at `52fcb3f` and `13d8bfc`

**Issue:** The required Conventional commits check rejects `test(08): cover reconciled kernel snapshots` and `style(08): wrap kernel reconciliation assertion`; this repository permits only `feat`, `fix`, `perf`, `revert`, `docs`, and `chore`. The current PR therefore cannot produce a clean review/merge state even though the implementation checks pass.

**Fix:** Rewrite those two commit subjects to allowed types (for example `fix(08): cover reconciled kernel snapshots` and `chore(08): wrap kernel reconciliation assertion`) and force-push the reviewed branch, then rerun CI and review the rewritten head.

## Warnings

### WR-01: The exact RHCK policy still omits current OL10 RHCK artifacts

**Files:** `deploy/ansible/roles/autographs_deploy/defaults/main.yml:64-82`, `deploy/ansible/roles/security_patching/defaults/main.yml:49-67`

**Issue:** Oracle's current OL10 repositories also ship `kernel-abi-stablelists` and `kernel-doc`; the latter has a distinct UEK counterpart (`kernel-uek-doc`). Neither is in the synchronized removal/exclusion/detection lists. If installed, these version-coupled RHCK artifacts remain patchable while the scanner reports the host as UEK-only. This leaves the second-round “complete RHCK family” fix incomplete; the tests only assert the four names added in that round rather than the complete deliberate policy boundary.

**Fix:** Add the RHCK-specific ABI/doc artifacts to both synchronized lists and contract tests, or explicitly define and test a narrower policy that explains why each current OL10 `kernel*` package is either rejected or deliberately preserved alongside `kernel-headers`, `kernel-tools`, and `kernel-tools-libs`.

---

_Reviewed: 2026-09-21T00:14:51Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
