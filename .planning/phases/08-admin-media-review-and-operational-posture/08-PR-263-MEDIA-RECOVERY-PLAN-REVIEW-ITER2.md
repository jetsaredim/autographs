---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: 1176b81da1c9b863f64ed088182d5d72057ddfe3
reviewed: 2026-10-05
review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6007104847
verdict: revisions_required
blockers: 1
warnings: 0
info: 0
---

# PR 263 Media Failure Recovery Authority Plan Review — Iteration 2

## Verdict

**REVISIONS REQUIRED — one blocker, zero warnings, and zero advisories.**

The revision closes the source-transport and failed-refresh blockers and supplies private UUID-correlated recovery evidence. The exact route contract is still internally contradictory and incomplete, so `08-PR-263-CONVERGENCE.md` must remain `reassessment_required`.

## Prior Finding Closure

| Prior finding | Revised-plan evidence | Verdict |
|---|---|---|
| BL-01: exact typed route mappings and manual-recovery evidence | Lines 255-266 add typed 409/404/500 rows, UUID `recoveryId`, and one structured private `image_replacement_manual_recovery_required` event with item/image IDs, transition, outcome, and key fingerprints. | Partially closed. The exact 409 body contradicts the earlier stable contract, and the verified-restoration failure row still names no exact status/body (current blocker). |
| BL-02: direct source `<img>` cannot expose 409 | Line 282 replaces revision-bound direct image acquisition with an authenticated fetch-to-blob request carrying source request/session/item/image/media authority, then separately guards blob-node callbacks. Lines 309 and 315 add stale/current open/source and retained output authority tests. | Closed. |
| BL-03: recovery 404/500/auth state is undefined | Lines 295-302 define 200, 404, 500/network, retry, 401/403, and context-change transitions. Stale item/actions are cleared, retry carries a new generation/context, and late results are rejected. Lines 310-311 require the decisive races. | Closed. |

## Full Contract Recheck

| Required property | Evidence | Verdict |
|---|---|---|
| Initial replacement and rollback are expected-object-key CAS transitions | Both transitions name their expected key; memory and Oracle return typed `Updated`, `Conflict`, `NotFound`, or `Repository`; Oracle predicates on expected object key. | Covered. |
| Rollback deletion follows exact restoration proof | Reload must prove original key, opaque revision, and saved adjustment; verification mismatch deletes nothing and becomes a recovery error. | Covered. |
| Failed/lost rollback preserves every possibly referenced object and cannot overwrite a winner | Conflict/failure rows preserve candidates; concurrent-newer replacement tests require no overwrite or deletion. | Covered. |
| Manual recovery is correlatable without public/private leakage | 500 body exposes only UUID recovery ID and redacted copy; structured private diagnostics use key fingerprints and exclude raw key, bucket, namespace, and checksum. Tests assert both sides. | Covered. |
| Every conflict is scoped before UI effects | Draft, open/source, assist, Save/Reset, and output callbacks each have a complete authority tuple; stale conflicts issue no refresh or state mutation. | Covered. |
| Current conflict refreshes once and immediate reopen succeeds | Dedicated generation invalidates once, fresh item reconciles before assignment, and current open/source/draft/assist/Save/Reset cases require successful immediate reopen. | Covered. |
| Failed/late recovery cannot restore stale authority | 404 clears the item, 500/network exposes only a generation-bound retry, auth logs out, and navigation/new review/logout invalidates pending generations. | Covered. |
| Prior media, gesture, output, mutation, publisher, migration, privacy, and public contracts remain required | The regression matrix retains the complete prior suites and keeps the opaque revision/private-source boundaries unchanged. | Covered. |

## Blocker

### BL-01 — The plan still has two exact 409 bodies and one non-exact failure response

**Dimension:** task completeness / cross-contract consistency  
**Required property:** Each typed replacement/rollback outcome must have one unambiguous HTTP status and body that is consistent with the phase-wide media-revision contract; every transition row, including successful restoration after warning-persistence failure, must identify the response rather than refer to unspecified existing behavior.

**Evidence:** The already-approved media-revision contract at line 190 declares the stable 409 body as `{ code: "mediaRevisionConflict", message: "Image media changed. Reopen the review." }`. The new outcome table at line 259 assigns the same status/code a different exact message: `"Image media changed. Retry from the current item."` Both cannot be the single exact body asserted by route tests. Separately, line 264 maps verified restoration plus successful replacement deletion only to “Existing failure response for the original warning-persistence failure,” without naming its status/body. That does not satisfy the addendum's own “Map every transition outcome exactly” requirement and leaves the executor/test author unable to know whether this coherent-but-failed operation returns the redacted recovery body, an empty 500, or another contract. The response discrepancy is the remaining portion of prior BL-01, not a style preference.

**Example revision (non-binding):** Choose one stable 409 body for every `mediaRevisionConflict` route (or define a different typed code if replacement conflicts intentionally require distinct copy), and replace “existing failure response” with its exact status/body. If successful restoration needs no manual recovery, state the exact redacted non-recovery error contract and assert it separately from UUID-bearing `imageRecoveryRequired` branches.

## Structured Issues

```yaml
issues:
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "cross_plan_data_contracts"
    severity: "blocker"
    required_property: "Every typed replacement/rollback outcome has one exact status/body consistent with the stable media-revision contract"
    description: "Line 190 fixes mediaRevisionConflict to 'Image media changed. Reopen the review.', while line 259 assigns the same status/code 'Image media changed. Retry from the current item.'; line 264 refers only to an unspecified existing failure response. Exact route tests and implementation therefore have mutually incompatible or absent expected bodies."
    fix_hint: "Unify the mediaRevisionConflict body (or use a distinct typed code) and spell out the exact verified-restoration failure status/body."
```

## Required Re-review Evidence

Resolve the single response-contract inconsistency, then obtain a third independent media-recovery plan review with zero Critical, Warning, and Info findings. Only after that artifact and PR comment are recorded may convergence become `ready_to_resume`.

No implementation or tests were changed or executed during this plan-only review.
