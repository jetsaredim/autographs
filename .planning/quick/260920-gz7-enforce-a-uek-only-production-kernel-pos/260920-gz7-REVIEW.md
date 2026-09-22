---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-22T11:24:46Z
depth: deep
files_reviewed: 43
files_reviewed_list:
  - .github/workflows/ci.yml
  - controller/tests/runtime_kernel_persistence.rs
  - deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  - deploy/ansible/playbooks/security-create-issue-status-validate-test.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-status-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-kernel-selection-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml
  - deploy/ansible/playbooks/security-reboot-result-validate-test.yml
  - deploy/ansible/playbooks/security-reboot.yml
  - deploy/ansible/playbooks/security-report-render-test.yml
  - deploy/ansible/playbooks/security-request-metadata-validate-test.yml
  - deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml
  - deploy/ansible/roles/autographs_deploy/defaults/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_images.yml
  - deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_ownership.yml
  - deploy/ansible/roles/autographs_deploy/tasks/derive_dnf_exclusions.yml
  - deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml
  - deploy/ansible/roles/autographs_deploy/tasks/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/revalidate_uek_boot_state.yml
  - deploy/ansible/roles/autographs_deploy/tasks/validate_kernel_boot_entry.yml
  - deploy/ansible/roles/security_patching/defaults/main.yml
  - deploy/ansible/roles/security_patching/tasks/classify_findings.yml
  - deploy/ansible/roles/security_patching/tasks/classify_reboot_request.yml
  - deploy/ansible/roles/security_patching/tasks/classify_update_request.yml
  - deploy/ansible/roles/security_patching/tasks/create_issue.yml
  - deploy/ansible/roles/security_patching/tasks/patch.yml
  - deploy/ansible/roles/security_patching/tasks/post_reboot_result.yml
  - deploy/ansible/roles/security_patching/tasks/post_result.yml
  - deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml
  - deploy/ansible/roles/security_patching/tasks/resolve_reboot_kernel.yml
  - deploy/ansible/roles/security_patching/tasks/revalidate_reboot_kernel.yml
  - deploy/ansible/roles/security_patching/tasks/scan.yml
  - deploy/ansible/roles/security_patching/tasks/validate_post_reboot_kernel.yml
  - deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml
  - deploy/ansible/roles/security_patching/tasks/validate_request.yml
  - deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2
  - deploy/ansible/roles/security_patching/templates/security-report.md.j2
  - deploy/ansible/roles/security_patching/templates/security-update-result.md.j2
  - docs/deployment-runbook.md
  - docs/security-patching.md
  - scripts/test_security_patching_create_issue_tasks.py
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-22T11:24:46Z
**Depth:** deep
**Files Reviewed:** 43
**Status:** issues_found

## Summary

The deep review covered the complete PR diff at `a98da2c` and rechecked the three findings fixed by `b5e4d3b`, `dc011fc`, and `50724c8`. Those fixes are present and correct: deployment now repeats exact running/default image, RPM-owner, grubby-entry, release-matched initramfs, and multi-component initrd proof immediately before RHCK removal; reboot cleanup repeats the RPM-newest target, exact owner/entry/component proof immediately before default mutation; and the RHCK fallback fixture now isolates the unsafe post-reboot kernel predicate. DNF exclusion preservation, RHCK package policy, approval drift reconciliation, multi-host action precedence, issue refresh/closure, and CI wiring also remain intact. All GitHub CI checks at this head are green.

The PR is not clean. The scanner's definition of a "verified bootable UEK" is now weaker than the deployment gate: it checks only the kernel image and RPM owner, so a missing or mismatched initramfs is advertised as safe `configure` drift even though deployment must reject it. Separately, a failed `grubby --default-kernel` probe aborts the scan before the issue can be refreshed with the intended recovery state.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Scanner can route an unbootable UEK entry into a non-convergent configure loop

**File:** `deploy/ansible/roles/security_patching/tasks/scan.yml:187-204`

**Issue:** `security_patching_running_kernel_is_valid_uek` and `security_patching_default_kernel_is_valid_uek` require only an `el10uek` name, a regular `vmlinuz`, and a `kernel-uek*` RPM owner. They never query the exact grubby entries or validate the release-matched initramfs and every concrete initrd component. After `b5e4d3b`, deployment requires that stronger proof before RHCK removal. Therefore a host with installed RHCK packages plus an owned UEK `vmlinuz` but a missing/mismatched UEK initramfs is classified `configure`; the issue tells the operator to run deployment convergence, deployment fails closed, and every rescan reproduces the same action. The report also labels this weaker state "verified bootable UEK," which is factually incorrect.

**Fix:** Add nonfatal exact-entry probes to `scan.yml`, parse the exact kernel/initrd fields with the same semantics as the deployment helper, and include release matching plus regular-file checks for all concrete components in both UEK-valid facts. Missing/mismatched entry state should classify `investigate`, render recovery guidance, and have a fixture proving RHCK drift cannot select `configure` until the deployment bootability gate can pass.

## Warnings

### WR-01: Missing default-kernel state aborts the scanner instead of updating the issue

**File:** `deploy/ansible/roles/security_patching/tasks/scan.yml:126-155`

**Issue:** `grubby --default-kernel` uses the command module's default fatal behavior, and the following `stat` is unconditional. If grubby returns nonzero because the saved/default entry is absent or corrupt, the scan exits before `classify_findings.yml` can mark the default UEK state invalid and before `create_issue.yml` can publish the documented kernel-recovery action. This leaves the existing ticket stale precisely when operator guidance is needed.

**Fix:** Make the default-kernel probe nonfatal, preserve its rc/stdout as evidence, guard the stat/owner/entry probes on a single valid path, and require `rc == 0` in `security_patching_default_kernel_is_valid_uek`. Add a failed/empty grubby fixture that completes classification as `investigate` with no approval label and an actionable refreshed issue.

---

_Reviewed: 2026-09-22T11:24:46Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
