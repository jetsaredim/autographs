---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-09-27T19:09:05Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-257-REVIEW-ITER2.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-27T19:09:05Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-257-REVIEW-ITER2.md`
**Iteration:** 2

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: A post-audit failure blocks any repaired retry from a new main commit

**Files modified:** `.github/workflows/deploy.yml`, `scripts/retry_recovery.py`, `scripts/release.py`, `scripts/test_release.py`, `scripts/test_release_workflow.py`
**Commit:** 3949ce1
**Applied fix:** Removed per-dispatch automation identity from the stable `retry-recovery-audit.json` payload and added separately named, append-only attempt assets keyed by workflow run ID and attempt. Each attempt record captures the automation revision and hashes the stable recovery audit, while the canonical asset remains byte-identical whenever the release source, pinned recovery revision, and approved file hashes are unchanged. Existing canonical and attempt assets are reconciled fail-closed before production mutation.

### WR-01: The partial-failure test proves only pure-function idempotence with an unchanged audit

**Files modified:** `scripts/test_release_workflow.py`
**Commit:** 0874fdd
**Applied fix:** Added a two-dispatch lifecycle test that reuses an existing canonical audit across different automation revisions and workflow run IDs, verifies distinct attempt evidence binds to the same payload hash, and proves that a changed pinned recovery payload conflicts before Terraform or Ansible execution.

### WR-02: Repo-only releases erase provenance for runtime bytes that remain deployed

**Files modified:** `scripts/release.py`, `scripts/test_release.py`
**Commit:** 684dbfe
**Applied fix:** Treated `retryRecovery` as active deployed-runtime provenance. Repo-only automatic and retry transitions now preserve it, production-mutating automatic releases clear it, production-mutating retries replace it only when a recovery overlay is applied, and controller-only rollback leaves it intact. Added coverage for every transition class.

---

_Fixed: 2026-09-27T19:09:05Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
