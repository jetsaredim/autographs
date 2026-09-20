---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
fixed_at: 2026-09-20T16:55:33Z
review_path: .planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Quick Task 260920-gz7: Code Review Fix Report

**Fixed at:** 2026-09-20T16:55:33Z
**Source review:** `.planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: RHCK removal trusts a default UEK pathname without proving a bootable UEK image exists

**Files modified:** `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml`, `deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_images.yml`, `deploy/ansible/roles/autographs_deploy/tasks/assert_kernel_ownership.yml`, `deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml`
**Commit:** `0eefb9c`
**Status:** fixed; requires human verification
**Applied fix:** The deployment now proves the running and default UEK paths are regular files and that each is owned by an installed `kernel-uek*` RPM before the RHCK removal task can run. Executable fail-closed fixtures cover missing running/default images and invalid RPM ownership without reaching simulated mutation.

### WR-01: The scanner emits a non-convergent configure action when RHCK is active or selected for next boot

**Files modified:** `deploy/ansible/roles/security_patching/tasks/scan.yml`, `deploy/ansible/roles/security_patching/tasks/classify_findings.yml`, `deploy/ansible/roles/security_patching/tasks/create_issue.yml`, `deploy/ansible/roles/security_patching/templates/security-report.md.j2`, `deploy/ansible/playbooks/security-finding-classification-validate-test.yml`, `deploy/ansible/playbooks/security-report-render-test.yml`, `deploy/ansible/playbooks/security-create-issue-status-validate-test.yml`, `docs/deployment-runbook.md`, `docs/security-patching.md`
**Commit:** `2534d43`
**Status:** fixed; requires human verification
**Applied fix:** Scans now inventory and verify running/default UEK images and RPM ownership. Unsafe kernel state classifies as `investigate`, takes aggregate precedence, offers no approval label, remains reportable even without package findings, and renders install/select/reboot/converge/rescan guidance. Running-RHCK and default-RHCK fixtures prove neither state enters configure, update, or reboot approval loops.

### WR-02: DNF exclusion persistence preserves only the first existing exclude declaration

**Files modified:** `deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml`, `deploy/ansible/roles/autographs_deploy/tasks/derive_dnf_exclusions.yml`, `deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml`
**Commit:** `553b456`
**Status:** fixed; requires human verification
**Applied fix:** The deployment extracts only the active `[main]` section, merges tokens from every active `exclude=` declaration with the managed RHCK exclusions, and writes a sorted unique canonical value. The executable fixture proves multiple main exclusions survive, another section does not leak into policy, and repeated evaluation is idempotent.

---

_Fixed: 2026-09-20T16:55:33Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
