---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-27T17:12:01Z
depth: deep
files_reviewed: 7
files_reviewed_list:
  - .github/workflows/deploy.yml
  - scripts/release.py
  - scripts/retry_recovery.py
  - scripts/retry-recovery-contracts.json
  - scripts/test_release.py
  - scripts/test_release_workflow.py
  - scripts/test-fixtures/release-workflow/cases.json
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-09-27T17:12:01Z
**Depth:** deep
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The iteration-1 payload-boundary fix correctly keeps controller and Terraform inputs on the selected tag, compares the complete Ansible delta to a release-specific allowlist, verifies every approved recovery blob hash before overlay, validates all files before writing any of them, and runs Ansible from the resulting tag workspace. The guard and contract are also checked out at the immutable workflow-dispatch `${{ github.sha }}`, rather than from a moving branch reference.

The retry lifecycle is not converged, however. The fixed-name audit asset includes the current dispatch commit, is uploaded before any production work, and is compared byte-for-byte on every later attempt. Any post-upload failure that requires a new commit on `main` produces a different audit and blocks the repaired retry. The workflow's own successful status commit also advances `main`, so a fresh dispatch after a transient final-publication failure hits the same conflict (although re-running the original workflow run retains its original SHA). The status schema additionally discards or overwrites deployed recovery provenance when a later repo-only release performs no production mutation.

## Critical Issues

### CR-01: A post-audit failure blocks any repaired retry from a new main commit

**File:** `/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.github/workflows/deploy.yml:224-253`
**Lineage:** `fix_regression:CR-01`
**Issue:** The audit bytes include `automationRevision=${{ github.sha }}` (lines 224 and 238), and the workflow uploads those bytes under the single immutable name `retry-recovery-audit.json` before image reconciliation, Terraform, Ansible, health verification, the status commit, or publication (lines 241-253). A subsequent attempt accepts an existing asset only when it is byte-identical. If any later step fails and the repair lands as another commit on `main`, the next dispatch has a different `github.sha`, generates a different audit, and fails at `reconcile-asset` before it can retry production. There is also a self-induced fresh-dispatch case: the status step pushes a new commit to `main` at lines 811-839 before `gh release edit --draft=false`; if publication then fails transiently, a new dispatch runs from that status commit, so its `automationRevision` differs even though the recovery revision and every approved byte are unchanged. Re-running the original GitHub Actions run can retain the old event SHA, but that cannot consume a code or contract repair. The committed status also embeds the old audit, making the documented new-dispatch "partial failure" path non-idempotent. This defeats the recovery workflow whenever the post-upload failure itself needs a reviewed fix, unless an operator manually deletes or replaces the audit asset.

**Fix:** Make the canonical per-release recovery evidence stable across attempts. For example, derive `retry-recovery-audit.json` only from the release source, pinned recovery revision, and approved path hashes, while recording workflow-attempt/automation identity in a separate append-only or content-addressed attempt record. When the fixed asset already exists, validate its payload against the current contract and the bytes about to be executed, then reuse it if that recovery payload is unchanged. If automation provenance must be part of the canonical object, version/name assets per attempt and have the status/publication gate explicitly select the successful audit instead of byte-comparing a new attempt against one fixed filename. Preserve fail-closed rejection when the pinned recovery revision or approved hashes actually change.

## Warnings

### WR-01: The partial-failure test proves only pure-function idempotence with an unchanged audit

**File:** `/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/scripts/test_release.py:410-420`
**Lineage:** `test_weakness:CR-01`
**Issue:** `test_retry_transition_is_idempotent_after_partial_failure` passes the exact same in-memory audit object to both calls. It never models the workflow facts that the first attempt uploads a fixed-name asset, the status write advances `main`, and the next dispatch therefore supplies a different `automationRevision`. The structural workflow tests likewise assert only that `reconcile-asset` is present; they do not execute the existing-asset path across two automation revisions. As a result, the suite calls this lifecycle idempotent while the real retry fails before status reconciliation.

**Fix:** Add a two-attempt integration fixture: generate/upload audit A, apply the status transition, advance the automation commit without changing the recovery contract or approved blobs, and assert that attempt B can validate/reuse the canonical recovery evidence and reach publication without changing deployed status. Add the complementary case where the recovery revision or an approved hash changes and assert that the second attempt fails closed before Terraform or Ansible.

### WR-02: Repo-only releases erase provenance for runtime bytes that remain deployed

**File:** `/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/scripts/release.py:452-475`
**Lineage:** `fix_regression:CR-01`
**Issue:** `retryRecovery` is the only status field that records the recovery revision and approved hashes used to produce the active runtime, but every automatic transition unconditionally removes it at line 475, and every retry unconditionally replaces it at line 473. For a later `repo-only` release, lines 454-457 intentionally leave `deployedRepositoryVersion` and `sourceRevision` unchanged because production was not mutated, yet line 475 deletes the recovery provenance for those still-deployed bytes. A repo-only manual retry similarly replaces it with that non-mutating release's tag-only audit. `.release-status.json` can therefore claim that the recovered release remains deployed while omitting or misattributing the overlay that is actually running.

**Fix:** Define separate deployed-runtime provenance and last-attempt provenance, or treat `retryRecovery` as deployed state: preserve it across repo-only automatic/retry transitions, replace it only when a production-mutating retry installs a new recovery payload, and clear it only when a later production-mutating tagged release supersedes the recovered runtime. Add status-transition tests beginning from a recovered status for repo-only automatic, repo-only retry, runtime-config/controller automatic, and rollback cases.

---

_Reviewed: 2026-09-27T17:12:01Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
