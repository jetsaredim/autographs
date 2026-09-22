---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
reviewed: 2026-09-22T01:36:42Z
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
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Quick Task 260920-gz7: Code Review Report

**Reviewed:** 2026-09-22T01:36:42Z
**Depth:** deep
**Files Reviewed:** 40
**Status:** issues_found

## Summary

The deep review covered the complete PR diff at `fb7d37f`, rechecked every prior review finding, traced the update/reboot reconciliation paths, and focused on the `e6df9ea` initramfs fix. The resolver now correctly parses exact quoted `kernel=` and `initrd=` fields, handles multiple concrete initrd components, requires the release-matched initramfs and every concrete component to be regular files, selects the RPM-ordered newest installed `kernel-uek-core`, preserves the target through reboot, and requires post-reboot running/default equality before cleanup. Multi-host drift, mixed clean/actionable approval scope, RHCK inventory, DNF exclusion preservation, issue refresh, action precedence, and CI wiring remain correct. Focused Ansible, Python, Rust, and all GitHub CI checks pass.

The PR is not clean. The deployment path still removes RHCK packages without proving the running/default UEK boot entries have usable initramfs files. In the reboot path, the new immediate revalidation stats only the paths remembered from preflight; it does not re-read the selected grubby entry, kernel image, or RPM owner, so a changed boot entry can pass the check and be rebooted. The fallback-boot fixture also no longer isolates its intended post-reboot guard after the new initramfs predicates were added.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Deployment removes RHCK before proving a bootable UEK initramfs exists

**File:** `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml:23-83`

**Issue:** The destructive RHCK removal gate proves only that the running/default paths contain `el10uek`, that both `vmlinuz` paths are regular files, and that RPM reports a `kernel-uek*` owner. It never inspects either exact `grubby` entry or its initramfs. A stale default UEK entry can therefore retain a valid package-owned kernel image while `/boot/initramfs-<release>.img` is missing. The following DNF transaction removes the ordinary RHCK packages that still provide a bootable fallback, leaving the configured default unable to boot. This is the same bootability gap fixed by `e6df9ea` in the reboot workflow, but the deployment cleanup path still has it.

**Fix:** Before `Remove managed RHCK kernel packages from UEK runtime`, resolve the exact running and default UEK entries, parse their concrete initrd components, require the release-matched initramfs and every concrete component to be regular files, and revalidate that proof immediately before the DNF removal. Add missing/mismatched-initramfs deployment fixtures proving RHCK removal is never reached.

### CR-02: Pre-mutation revalidation does not prove the grubby entry still references the validated initramfs

**File:** `deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml:1-31`

**Issue:** Preflight correctly parses the selected grubby entry, but the later task named "Evaluate selected UEK initramfs proof immediately before boot mutation" only stats the list of paths captured earlier. It does not re-run `grubby --info=<target>`, reparse the exact `kernel=`/`initrd=` fields, or recheck the kernel image and RPM owner. If a kernel transaction or operator changes the BLS/grubby entry after the all-host preflight while the old files remain, this check stays true; `grubby --set-default` selects the changed entry, the readback verifies only the kernel path, and the workflow can reboot into a missing or mismatched initramfs. The security workflows serialize with each other, but production deployment uses a different concurrency group, and multi-host preflight/serial reboot also creates a nontrivial gap.

**Fix:** Immediately before default selection, rerun the complete target proof: stat the kernel image, verify `kernel-uek-core` ownership, query the exact grubby entry, reparse its kernel and concrete initrd fields, require exact equality with the preserved target/component set, and stat the current component set. Add a fixture that changes the effective boot entry after resolver preflight while leaving the originally validated initramfs files present; selection/reboot/cleanup must remain unreachable.

## Warnings

### WR-01: The RHCK fallback fixture is now satisfied by an unrelated missing-initramfs predicate

**File:** `deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml:98-140`

**Issue:** The fallback-RHCK fixture invokes `validate_post_reboot_kernel.yml` without setting `security_patching_reboot_target_kernel_valid`, `security_patching_reboot_target_kernel_image`, or the new target initramfs facts. The guard therefore fails even if all running/default RHCK ownership and equality predicates regress, because the target/initramfs defaults alone evaluate false. The test still passes but no longer proves its stated property: that a fallback RHCK boot cannot reach installonly cleanup.

**Fix:** Populate a fully valid expected target and initramfs proof in this fixture, then vary only the post-reboot running/default release, image, and owner to RHCK. Assert the failure context identifies the unsafe post-reboot kernel and cleanup remains unreachable.

---

_Reviewed: 2026-09-22T01:36:42Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
