---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-21T01:43:34Z
depth: deep
files_reviewed: 36
files_reviewed_list:
  - controller/tests/runtime_kernel_persistence.rs
  - deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml
  - deploy/ansible/playbooks/security-create-issue-status-validate-test.yml
  - deploy/ansible/playbooks/security-finding-classification-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml
  - deploy/ansible/playbooks/security-post-result-status-validate-test.yml
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
  - deploy/ansible/roles/security_patching/tasks/scan.yml
  - deploy/ansible/roles/security_patching/tasks/validate_post_reboot_kernel.yml
  - deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml
  - deploy/ansible/roles/security_patching/tasks/validate_request.yml
  - deploy/ansible/roles/security_patching/templates/security-reboot-result.md.j2
  - deploy/ansible/roles/security_patching/templates/security-report.md.j2
  - deploy/ansible/roles/security_patching/templates/security-update-result.md.j2
  - docs/deployment-runbook.md
  - docs/security-patching.md
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-21T01:43:34Z
**Depth:** deep
**Files Reviewed:** 36
**Status:** issues_found

## Summary

The prior review findings are materially resolved at PR head `9372ddd`: exact live-target metadata now supports mixed clean/finding groups; clean hosts skip update/reboot mutation while finding hosts remain actionable; aggregate drift still prevents partial mutation; running/default UEK files and installed RPM ownership are proved before deployment cleanup and again after reboot; RHCK policy covers the current enabled OL10 BaseOS/AppStream boot and development package families while preserving shared headers/tools; repeated DNF exclusions are retained; commit subjects pass the repository release policy; and all current GitHub checks pass.

The PR is not clean. The reboot path verifies that the booted image equals the configured default UEK image, but never proves that this image is the newest/remediating installed UEK. A valid but stale default can therefore produce a successful no-op reboot and repeat the same reboot classification indefinitely.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: A stale but valid default UEK can cause an endless reboot-approval loop

**Files:** `deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml:259-270`, `deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml:9-41`, `deploy/ansible/roles/security_patching/tasks/validate_post_reboot_kernel.yml:1-24`, `docs/security-patching.md:402`

**Issue:** Reboot preflight proves only that OpenSCAP still classifies the exact approved advisory set as reboot-only and that DNF has no package work. The mutation then reboots whatever `grubby --default-kernel` already selects. Post-reboot validation proves that the running/default files are installed UEK images and are equal, but it does not compare that image with the newest installed bootable UEK (or otherwise prove it is the image that resolves the advisory set). If a host is pinned to an older, still-installed UEK, every safety assertion passes, the host reboots back into that same vulnerable kernel, the post-reboot scan reports the same findings, and the issue offers `approved-production-reboot` again. This contradicts the runbook claim that the workflow boots the newest installed UEK and leaves the issue in a non-convergent loop with repeated downtime.

**Fix:** Before allowing reboot mutation, enumerate installed bootable `kernel-uek*` images using RPM version ordering, resolve the newest supported image, and require (or set) `grubby --default-kernel` to that exact image. Preserve the selected target as an explicit fact and, after reboot, require `/boot/vmlinuz-$(uname -r)` to equal that target in addition to the existing file/ownership checks. Add a fixture where running/default point to an older valid UEK while a newer installed UEK exists and prove the workflow either selects the newer target or reconciles without reboot; it must never reboot the older image and offer the same action again.

---

_Reviewed: 2026-09-21T01:43:34Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
