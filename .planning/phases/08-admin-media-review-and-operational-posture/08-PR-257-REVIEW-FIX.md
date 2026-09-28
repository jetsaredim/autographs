---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-09-27T17:06:33Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-257-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-27T17:06:33Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-257-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### CR-01: Retry can silently publish later runtime configuration as the older release

**Files modified:** `.github/workflows/deploy.yml`, `scripts/retry_recovery.py`, `scripts/retry-recovery-contracts.json`, `scripts/release.py`, `scripts/test_release.py`, `scripts/test_release_workflow.py`
**Commit:** 4cb9782
**Applied fix:** Replaced the mutable `deploy/ansible` checkout with a release-specific recovery contract. The v0.2.4 contract pins recovery revision `75bb0a9979639a32a7bb3d8da2507bc33f5c38f9`, requires the complete Ansible delta to contain only the two approved boot-entry validation files, verifies their exact blob hashes before mutation, and overlays only those approved bytes onto the tag workspace. Controller and Terraform inputs remain tag-bound, and the accepted recovery payload is recorded in release audit/status data.

### WR-01: The regression test proves path routing but not the claimed release-content isolation

**Files modified:** `scripts/test_release_workflow.py`, `scripts/test-fixtures/release-workflow/cases.json`
**Commit:** 6a2dbe8
**Applied fix:** Added two-revision recovery-boundary coverage. The tests accept the exact reviewed v0.2.4 recovery delta, reject representative non-allowlisted Ansible changes before production mutation, verify the pinned file hashes, and assert that recovery provenance participates in the publication/status boundary.

## Verification

- `scripts/test_release.py`: 22 tests passed.
- `scripts/test_release_workflow.py`: 20 tests passed.
- Repository automation suite: 95 tests passed.
- The actual `v0.2.4..75bb0a9` Ansible delta and both pinned blob hashes match the committed recovery contract.
- Workflow YAML, recovery contract JSON, and the guarded overlay/diff checks passed.

---

_Fixed: 2026-09-27T17:06:33Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
