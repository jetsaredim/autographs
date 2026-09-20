---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-20T16:38:22Z
depth: deep
files_reviewed: 21
files_reviewed_list:
  - deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  - deploy/ansible/playbooks/security-create-issue-status-validate-test.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/roles/autographs_deploy/defaults/main.yml
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
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-20T16:38:22Z
**Depth:** deep
**Files Reviewed:** 21
**Status:** issues_found

## Summary

The UEK/RHCK classification and no-mutation reconciliation paths are internally consistent, and the revised reboot allowlist prevents RHCK findings from entering the reboot-cleanup path. The destructive deployment path still has one bootability hole, however, and two edge cases can leave either an unhelpful non-convergent issue state or silently discard pre-existing DNF policy.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: RHCK removal trusts a default UEK pathname without proving a bootable UEK image exists

**File:** `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml:16-24`

**Issue:** The only precondition for destructive RHCK package removal is that `uname -r` and `grubby --default-kernel` contain `el10uek`. `grubby` can retain entries whose referenced `/boot/vmlinuz-*` file no longer exists—the same task file explicitly cleans such stale entries later. If the default UEK entry is stale, and the running UEK image/package was also removed after boot, this assertion passes and the following DNF transaction can delete the remaining bootable RHCK fallback. The later stale-entry cleanup then removes the missing UEK entry, leaving the host with no proven bootable non-rescue kernel.

**Fix:** Before the DNF removal, derive `/boot/vmlinuz-{{ uname_release }}`, stat both that image and the exact path returned by `grubby --default-kernel`, and fail unless both are regular files. Prefer also verifying RPM ownership by an installed `kernel-uek*` package. Add a behavior test for a stale default UEK BLS entry and prove that no package mutation occurs.

## Warnings

### WR-01: The scanner emits a non-convergent `configure` action when RHCK is active or selected for next boot

**File:** `deploy/ansible/roles/security_patching/tasks/classify_findings.yml:85-111`

**Issue:** Any installed RHCK package forces `configure`, but the scanner never records whether the running and default kernels are UEK. The issue therefore always instructs the operator to run deployment convergence. That deployment deliberately refuses mutation when either kernel is RHCK, so a host with RHCK selected as default—or actually running RHCK—will repeatedly retain the same `configure` issue while the prescribed action fails. The issue does not expose the kernel state or the recovery needed to make convergence safe.

**Fix:** Inventory the running and default kernel in the scan. Emit `configure` only when both are valid UEK; otherwise emit `investigate` (or a dedicated recovery action) and render explicit steps to install/select a UEK kernel and reboot if required before deployment convergence. Cover both default-RHCK and running-RHCK fixtures.

### WR-02: DNF exclusion persistence preserves only the first existing `exclude=` declaration

**File:** `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml:41-66`

**Issue:** The slurped configuration is searched globally and then piped through `first`. `community.general.ini_file` uses `exclusive: true` by default, so it replaces/removes every matching `exclude` option in `[main]` with the single computed value. If `[main]` contains multiple active declarations, exclusions from every declaration after the first are silently lost. The global regex can also select an `exclude` from a different section if it appears first. That contradicts the task's preservation contract and can re-enable packages intentionally held back by an operator.

**Fix:** Parse only the `[main]` section and merge tokens from every active `exclude` declaration before writing the canonical line, or manage the RHCK policy in a dedicated DNF configuration drop-in that does not rewrite unrelated operator exclusions. Add a functional fixture containing multiple `[main]` exclusions and another section.

---

_Reviewed: 2026-09-20T16:38:22Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
