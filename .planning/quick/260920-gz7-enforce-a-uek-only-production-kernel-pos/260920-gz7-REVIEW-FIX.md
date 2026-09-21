---
phase: quick-260920-gz7-enforce-a-uek-only-production-kernel-pos
fixed_at: 2026-09-21T00:42:37Z
review_path: .planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW.md
iteration: 3
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Quick Task 260920-gz7: Code Review Fix Report

**Fixed at:** 2026-09-21T00:42:37Z
**Source review:** `.planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW.md`
**Iteration:** 3

**Summary:**

- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Mixed clean/finding target groups cannot enter either approval workflow

**Files modified:** `deploy/ansible/roles/security_patching/templates/security-report.md.j2`, `deploy/ansible/roles/security_patching/tasks/validate_request.yml`, `deploy/ansible/roles/security_patching/tasks/classify_update_request.yml`, `deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml`, `deploy/ansible/roles/security_patching/tasks/classify_reboot_request.yml`, `deploy/ansible/playbooks/security-reboot.yml`, `deploy/ansible/playbooks/security-report-render-test.yml`, `deploy/ansible/playbooks/security-request-metadata-validate-test.yml`, `deploy/ansible/playbooks/security-update-reconciliation-validate-test.yml`, `deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml`
**Commit:** `d75337c`
**Status:** fixed; requires human verification
**Applied fix:** Finding-bearing reports now retain every live target in hidden metadata and use an empty advisory list for clean hosts. Exact live-target scope validation remains mandatory. Aggregate update and reboot classifiers identify only complete, approved-empty/current-empty `close` hosts as safe skips, retain drift protection for every other state, and expose the remaining actionable host set. The reboot play only mutates hosts that passed per-host preflight. Mixed recovered-clean plus update/reboot fixtures prove exact scope passes while only the finding host reaches mutation.

### CR-02: Installonly cleanup runs before the rebooted kernel is proven safe

**Files modified:** `deploy/ansible/roles/security_patching/tasks/reboot_cleanup.yml`, `deploy/ansible/roles/security_patching/tasks/validate_post_reboot_kernel.yml`, `deploy/ansible/playbooks/security-reboot-preflight-validate-test.yml`, `docs/security-patching.md`
**Commits:** `c859437`, `513cec2`
**Status:** fixed; requires human verification
**Applied fix:** Immediately after reboot, the workflow now resolves the running and default images, requires both regular files to have installed `kernel-uek*` RPM owners, requires the running release to be UEK, and requires the booted image to equal the configured default. An unsafe fallback or rescue boot writes bounded failure context and fails before service checks or `dnf remove --oldinstallonly`. The fixture verifies task ordering and proves an RHCK fallback never reaches simulated cleanup.

### CR-03: The PR merge gate is currently failing

**History rewrite:** `52fcb3f` → `f735ec4` (`fix(08): cover reconciled kernel snapshots`); `13d8bfc` → `a33259f` (`chore(08): wrap kernel reconciliation assertion`); descendant documentation commit `23ede47` → `a10a6f6`.
**Status:** fixed
**Applied fix:** Rewrote only the two rejected subjects to configured release-please types while preserving commit content and order. Local release input validation accepts the PR title and all 17 non-merge commit subjects.

### WR-01: The exact RHCK policy still omits current OL10 RHCK artifacts

**Files modified:** `deploy/ansible/roles/autographs_deploy/defaults/main.yml`, `deploy/ansible/roles/security_patching/defaults/main.yml`, `deploy/ansible/playbooks/runtime-kernel-persistence-validate-test.yml`, `deploy/ansible/playbooks/security-finding-classification-validate-test.yml`, `controller/tests/runtime_kernel_persistence.rs`, `docs/deployment-runbook.md`
**Commit:** `27ce869`
**Status:** fixed
**Applied fix:** The synchronized deployment and scanner policies now remove, exclude, and detect `kernel-abi-stablelists` and `kernel-doc`. Contract tests cover both names while continuing to require preservation of the shared `kernel-headers`, `kernel-tools`, and `kernel-tools-libs` packages.

## Verification

- Syntax checks passed for every playbook under `deploy/ansible/playbooks/` (20 playbooks).
- The full CI Ansible validation fixture sequence passed, including mixed clean/finding update and reboot paths and fallback-kernel cleanup rejection.
- `ansible-lint --profile production deploy/ansible/` passed with zero findings across 67 files.
- Security patching Python tests passed: 27 tests across advisory enrichment, OpenSCAP parsing, and issue task contracts.
- `bash scripts/validate-runtime.sh`, `cargo fmt --check`, and `cargo test --test runtime_kernel_persistence` passed; the Rust contract ran 3 tests.
- `scripts/validate_release_please_inputs.py` accepted the PR title and all 17 non-merge commits using the configured release types.

---

_Fixed: 2026-09-21T00:42:37Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 3_
