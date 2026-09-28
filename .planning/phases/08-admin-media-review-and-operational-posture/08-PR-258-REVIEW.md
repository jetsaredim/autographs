---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-28T13:31:35Z
depth: deep
files_reviewed: 2
files_reviewed_list:
  - .github/workflows/deploy.yml
  - scripts/test_release_workflow.py
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-28T13:31:35Z
**Depth:** deep
**Files Reviewed:** 2
**Status:** issues_found

## Summary

The separate production-status checkout fixes the dirty immutable-tag checkout failure, but it crosses the production approval boundary by executing `scripts/release.py` from a moving `main` ref. A commit that reaches `main` after this workflow was dispatched or approved can therefore execute inside the already-running production job and mutate release status. The new structural test verifies workspace isolation but does not pin or test the provenance of the executable used in that workspace.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Moving-main status checkout executes code outside the approved workflow revision

**File:** `.github/workflows/deploy.yml:824-864`
**Issue:** `Checkout current main for production status` resolves `ref: main` when that late step runs, and the next step invokes `python3 scripts/release.py` from that checkout. A push or merge after the run's `${{ github.sha }}` was selected can replace `scripts/release.py` before this production job reaches the status step. That later code has not been bound to the workflow revision that passed the release gate or the production-environment approval, yet it runs with the checkout's write-capable GitHub credential and while production credentials/artifacts remain on the runner. It can also reinterpret the verified manifest/audit or mutate more than `.release-status.json`. The same exposure exists in automatic, retry, and rollback paths. `scripts/test_release_workflow.py:289-314` only asserts that status runs in `.production-status`; it does not assert that the status executable is pinned, so all tests pass while this trust-boundary regression remains.

**Fix:** Keep `.production-status` as the moving/current-main mutation target, but execute status reconciliation from a separate sparse checkout pinned to the run's immutable `${{ github.sha }}`. For example:

```yaml
- name: Checkout trusted production status tooling
  if: steps.request.outputs.operation == 'rollback' || steps.release_gate.outputs.ready == 'true'
  uses: actions/checkout@v7
  with:
    fetch-depth: "1"
    ref: ${{ github.sha }}
    path: .status-automation
    sparse-checkout: scripts/release.py

# In the status step, while working-directory remains .production-status:
python3 "$GITHUB_WORKSPACE/.status-automation/scripts/release.py" rollback-status ...
python3 "$GITHUB_WORKSPACE/.status-automation/scripts/release.py" update-status ...
```

Extend the workflow contract test to require the tooling checkout's `ref` to equal `${{ github.sha }}`, require both status commands to use that pinned absolute path, and reject `python3 scripts/release.py` in the mutable status workspace. Preserve the existing current-main checkout and retry loop so only the status data target, not executable code, follows `origin/main`.

---

_Reviewed: 2026-09-28T13:31:35Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
