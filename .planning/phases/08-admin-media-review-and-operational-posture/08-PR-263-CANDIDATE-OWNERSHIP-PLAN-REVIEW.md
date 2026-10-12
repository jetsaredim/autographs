---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: e114fb3
verdict: changes_required
blockers: 1
warnings: 0
info: 0
---

# PR 263 Candidate Ownership Plan Review

## ISSUES FOUND

**Plan:** `08-PR-263-CANDIDATE-OWNERSHIP-REVISED-PLAN.md`
**Reviewed commit:** `e114fb3`
**Verdict:** **CHANGES REQUIRED — 1 Blocker, 0 Warning, 0 Info**

## Blocker

### 1. Local-only candidates must remain recoverable after the final `Drop` discovery failure

**Dimension:** Key links / test fidelity

**Required property:** Every locally known candidate remains actionable until
its object is verified absent, including when the last inventory attempt occurs
inside `Drop`.

**Evidence:** The plan makes `refresh_inventory` transactional and tests
persistent failure while the state object still exists. However, the current
`Drop` path returns immediately when inventory fails and then destroys the
guard. A retained local-only A candidate therefore disappears at guard
destruction. The emitted item/step evidence is intentionally fingerprint-only
and cannot reconstruct A when it has no Oracle image or cleanup-event row—the
exact motivating assumption recorded in `08-REVIEW.md`.

**Source boundary:** `controller/tests/live_persistence_smoke.rs:1419-1428`

**Why this remains open:** An implementation can remove `mem::take`, pass the
proposed helper tests, and still leave `Drop` as “failed refresh → report →
return.” That closes ownership during the call but not through the guard's
terminal lifecycle. A subsequent item-ID cleanup can discover B, remove B and
the Oracle rows, and still orphan local-only A.

**Required plan change:** Define and test a terminal ownership mechanism. On a
final discovery failure, either delete and verify all retained local candidates
while preserving Oracle rows, or durably preserve their private identities
before `Drop` destroys the guard. Raw identities must remain out of output.

## Coverage audit

| Boundary | Assessment |
| --- | --- |
| Initial A registration | Covered by transactional pre-call candidate ownership |
| Post-replacement refresh | Covered for failure followed by successful retry |
| Partial query/row/decode failure | Covered; temporary union is not installed |
| Normal `cleanup_and_verify` | Covered; union remains installed through delete and verification failures |
| Fallback `Drop` with successful retry | Covered; A/B/event union precedes row deletion |
| Fallback `Drop` with repeated discovery failure | **Not covered; guard destruction loses local-only identities** |
| Manual cleanup | Existing private environment input and Oracle rows remain available; no new privacy regression planned |
| Destructive SQL ordering | Covered; remains after all candidate absence proofs |
| Duplicate/already-absent candidates | Covered through deduplication and exact 404 behavior |
| CR-01 recovery surface | Remains closed; plan does not modify its files or authority contract |
| WR-01 evidence privacy | Remains closed; raw identities and provider text stay out of output |
| Verification commands | Runnable and targeted, but cannot compensate for the missing terminal-lifecycle assertion |

## Recommendation

Revise the plan to define and test ownership at guard destruction, then
re-review. Do not change convergence status to `ready_to_resume` yet.

No source files were reviewed as changed, and no repository state was modified
by the independent plan reviewer.
