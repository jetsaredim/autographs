---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-21T15:51:53Z
depth: deep
files_reviewed: 40
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
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-21T15:51:53Z
**Depth:** deep
**Files Reviewed:** 40
**Status:** issues_found

## Summary

The review covered the complete PR diff at `ba3fa7e`, including the latest RPM-ordered UEK target selection and all earlier deployment, scanner, approval reconciliation, issue refresh, and post-reboot safety changes. The previous findings are resolved: DNF supplies RPM-aware EVR ordering, the exact installed `kernel-uek-core` candidate is mapped to an owned kernel image and a `grubby` entry, stale defaults are advanced and read back before downtime, the expected target survives across the reboot and must equal both running and default state before cleanup, missing candidates fail closed, aggregate multi-host drift blocks mutation, kernel-only recovery state remains actionable, and RHCK removal/exclusion/reconciliation behavior remains fail-closed.

The PR is not yet clean. The new resolver calls a target "bootable" after validating only the kernel image, RPM owner, and existence of a `grubby` record; it never validates the boot entry's initramfs. A broken newest entry can therefore be selected and rebooted even though the required initramfs is absent.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Reboot preflight accepts a UEK boot entry whose initramfs is missing

**Files:** `deploy/ansible/roles/security_patching/tasks/resolve_reboot_kernel.yml:113-147`, `deploy/ansible/playbooks/security-reboot-kernel-selection-validate-test.yml:10-13`

**Issue:** The newest-candidate proof checks that the `vmlinuz` file is regular, owned by `kernel-uek-core`, and mentioned by successful `grubby --info` output. A BLS/grubby entry can remain present while its `initrd=/boot/initramfs-<release>.img` file is missing; in fact stale boot artifacts are one of the states this PR is designed to handle. The current fixture demonstrates the gap by declaring a candidate valid with boot-entry output containing only a `kernel=` line and no initrd at all. `reboot_cleanup.yml` then makes this entry the default and reboots. On this OCI host, whose root path depends on early-boot storage/network support, a missing initramfs can prevent the host from returning, so the later post-reboot equality checks never get a chance to fail safely.

**Fix:** Parse the exact selected entry returned by `grubby --info`, require its `kernel=` value to equal the selected image, extract the concrete primary `initrd=` path (ignoring variables such as `$tuned_initrd`), and `stat` it as a regular file before setting `security_patching_reboot_target_kernel_valid`. Fail closed with the image/initrd/entry evidence in the existing failure context. Extend the selection fixture with a real initramfs file for valid cases plus a missing-initramfs case that proves aggregate mutation is denied.

---

_Reviewed: 2026-09-21T15:51:53Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
