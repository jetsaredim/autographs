---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-09-28T13:45:22Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-258-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-28T13:45:22Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-258-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### CR-01: Moving-main status checkout executes code outside the approved workflow revision

**Files modified:** `.github/workflows/deploy.yml`, `scripts/test_release_workflow.py`
**Commit:** e33a942
**Status:** fixed: requires human verification
**Shared invariant:** Production status mutation code must be identical to the code at the immutable workflow revision that passed the release gate and production approval; only the `.release-status.json` data target may advance with `origin/main`.
**Consumers inspected:** The trusted tooling checkout, current-main status checkout, rollback-status and update-status commands, three-attempt fetch/switch/push mutation loop, release publication step, and structural workflow tests.
**Applied fix:** Added a credential-free sparse checkout of `scripts/release.py` pinned to `${{ github.sha }}` and changed both rollback and release status reconciliation to execute that pinned file while retaining `.production-status` as the current-main mutation workspace. Strengthened the workflow contract test to require the immutable ref, exact sparse checkout, checkout ordering, both pinned command paths, and rejection of mutable-workspace execution.
**Verification:** `python3 -m unittest scripts.test_release scripts.test_release_workflow` (47 tests passed); workflow YAML parsed successfully; `git diff --check` passed.

---

_Fixed: 2026-09-28T13:45:22Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
