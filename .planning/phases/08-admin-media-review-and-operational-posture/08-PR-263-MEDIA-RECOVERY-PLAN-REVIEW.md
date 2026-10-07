---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: fbdfdef65496a5a591cae0a92422384cf1cbc5e2
reviewed: 2026-10-05
review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6007037754
verdict: revisions_required
blockers: 3
warnings: 0
info: 0
---

# PR 263 Media Failure Recovery Authority Plan Review

## Verdict

**REVISIONS REQUIRED — three blockers, zero warnings, and zero advisories.**

The addendum correctly changes initial replacement and rollback into expected-object-key compare-and-set operations, requires verified restoration before replacement deletion, and scopes ordinary draft/assist/mutation conflicts to their originating authority. It does not yet define an executable recovery contract for source-image conflicts, failed item refreshes, or the route/evidence outcomes of replacement and rollback failures. Keep `08-PR-263-CONVERGENCE.md` at `reassessment_required`; no coder pass should start from this revision.

## Goal-Backward Coverage

| Required property | Current plan evidence | Verdict |
|---|---|---|
| Initial replacement and rollback are typed expected-key CAS transitions | The state table supplies original/replacement expected keys; memory and Oracle gain `Updated`, `Conflict`, `NotFound`, and `Repository` outcomes. | Covered structurally, but route classifications and recovery evidence are ambiguous (BL-01). |
| Successful rollback is reloaded and the original key/revision/adjustment are verified before replacement deletion | The rollback row and regression matrix require exact restoration and forbid deletion before reload proof. | Covered. |
| Failed/lost rollback preserves all potentially referenced objects and cannot overwrite a newer winner | The failure row preserves original, replacement, and newer candidates; the concurrent fixture requires CAS loss without overwrite/deletion. | Covered structurally, but response/evidence behavior is unresolved (BL-01). |
| Every conflict proves its captured authority before UI effects | Draft, open, assist, Save/Reset, and node callbacks have authority predicates; stale conflicts are intended to be inert. | Not executable for the source-image 409 path (BL-02). |
| Current conflict invalidates once, refreshes the exact item, replaces the stale token, and permits immediate reopen | The recovery generation and fresh-item reconciliation sequence covers successful JSON conflict paths. | Covered on refresh success; refresh failure leaves stale UI authority undefined (BL-03). |
| Delayed recovery cannot overwrite navigation, another item, logout, or a newer review | Generation plus intended navigation context and the race matrix require late results to be discarded. | Covered. |
| Regression evidence covers repositories, concurrency, object existence, messages/state, and prior contracts | The matrix names CAS conflict, restoration success/failure/race, verification mismatch, delete failure, stale/current client conflicts, recovery races/failures, and prior suites. | Incomplete until BL-01 through BL-03 specify assertions that distinguish the required outcomes. |

## Blockers

### BL-01 — Replacement and rollback failure outcomes need exact route mappings and durable manual-recovery evidence

**Dimension:** task completeness / failure-matrix derivation  
**Required property:** Every typed replacement transition must map deterministically to an HTTP/result contract and an inspectable cleanup-recovery record: expected-key loss is conflict, missing item/image is not-found, repository failure is internal error, and any preserved orphan candidate is identified through a defined operator-visible or structured internal evidence path without leaking private keys in the client response.

**Evidence:** The addendum defines repository outcomes at lines 245-253, but the route rule says only “return error/conflict” for initial CAS loss and “return redacted 500 or typed conflict, record manual-cleanup evidence” for rollback failure/CAS loss. It does not state which outcome produces which status/body, whether a verification mismatch is conflict or repository failure, or what “manual-cleanup evidence” concretely is when cleanup-warning persistence itself has failed. The regression matrix at line 287 asks tests to assert response type and cleanup evidence, but no response/evidence contract exists for those tests to assert. This ambiguity is material: an executor can map a concurrent winner to 500, collapse a missing image into conflict, or log an untraceable generic warning while still claiming the prose is satisfied.

**Example revision (non-binding):** Add a transition/output table mapping `Conflict`, `NotFound`, `Repository`, postcondition mismatch, and replacement-object delete failure to exact redacted statuses/bodies. Define the evidence sink and required correlation fields (for example, structured private server diagnostics keyed by request/item/image and cleanup operation, while keeping object keys out of the HTTP body), then require tests to assert each mapping and evidence event.

### BL-02 — The source-preview 409 cannot enter the planned conflict-recovery handler

**Dimension:** key links planned  
**Required property:** A current source-preview media-revision conflict must be distinguishable from an ordinary image load failure, prove the complete source authority tuple before UI mutation, and enter the same fresh-item recovery flow; stale source/output completions must remain inert.

**Evidence:** The addendum lists “Review open/source initialization” and “Source/output callbacks” at lines 264 and 267, but gives no transport/wiring that delivers the 409 status/body to the async conflict handler. The current source guide assigns the authenticated preview URL directly to an `<img>` (`admin.js:1844-1847`); its `error` event (`:1865-1878`) exposes neither HTTP status nor the JSON `mediaRevisionConflict` body, so replacement between review-open and source load is indistinguishable from a provider/render error. The decisive current-conflict matrix at line 284 covers only draft, assist, Save, and Reset, omitting review-open/source/output conflicts and immediate reopen. Authority-checking the node prevents stale UI mutation but does not refresh the current token for an authoritative source conflict.

**Example revision (non-binding):** Specify a status-bearing source acquisition path (such as authenticated fetch-to-blob with the full session/request/media authority tuple) or an equivalent server/client contract that can identify 409 before mounting the image. Add stale and current review-open/source conflict cases, asserting no effect for stale work and exactly-once refresh/reconcile/immediate reopen for current work. Keep output-node load/error authority tests even where output bytes already came from a status-bearing draft fetch.

### BL-03 — Recovery 404/500 behavior does not prevent stale-item actions or define retry authority

**Dimension:** key links planned / context recovery  
**Required property:** When authoritative conflict recovery cannot obtain a fresh item, stale `state.currentItem` and its review controls must not become actionable recovery truth; 404, transient 500, and auth failure need distinct state transitions, and any retry must carry a fresh recovery generation/context that cannot overwrite later navigation or review state.

**Evidence:** The success sequence at lines 269-275 safely reconciles a fresh item. Line 276 says only to preserve “redacted error/retry guidance” on refresh failure, and line 286 says 404/500/auth follow “explicit redacted behavior” without defining that behavior. After step 2 invalidates the review, the current implementation still has a stale `state.currentItem` and existing rendered image controls; unless the plan explicitly clears/disables or replaces them, a 404 or 500 can expose Review again with the obsolete revision and recreate the conflict loop. The plan also does not define which failures are retryable, what action triggers retry, or which generation/context a retry captures. Auth logout is referenced, but 404 item disappearance and transient 500 are materially different authority states.

**Example revision (non-binding):** Define a recovery-state table: 404 clears or deauthorizes the missing current item and returns to an appropriate non-item view; 500 retains no actionable stale review controls and exposes a generation-bound refresh retry; 401/403 executes logout and invalidates recovery. Test message/view/control state, zero stale reopen, retry success, and retry results arriving after navigation/new review/logout.

## Structured Issues

```yaml
issues:
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "task_completeness"
    severity: "blocker"
    required_property: "Every replacement/rollback outcome has an exact route classification and defined manual-recovery evidence"
    description: "The addendum defines typed repository outcomes but says only error/conflict or 500/conflict at the route and never defines the cleanup-evidence sink or fields, so tests and implementation cannot distinguish CAS loss, missing resource, repository failure, verification mismatch, and orphan cleanup consistently."
    fix_hint: "Add an outcome table with exact redacted status/body mappings and a structured private evidence contract, then assert every branch."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "key_links_planned"
    severity: "blocker"
    required_property: "A source-preview 409 reaches authority-scoped fresh-item recovery rather than collapsing into a generic image error"
    description: "The plan inventories source callbacks but does not bridge the server's 409 JSON contract to the direct img element error seam; current-conflict tests omit review-open/source/output, so authoritative source conflicts cannot refresh the stale token or prove immediate reopen."
    fix_hint: "Use a status-bearing source acquisition/recovery contract and add stale/current source and open conflict interleavings."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "key_links_planned"
    severity: "blocker"
    required_property: "Failed conflict refreshes leave no actionable stale item and have distinct generation-safe 404, 500, and auth transitions"
    description: "The plan specifies only redacted error/retry guidance. It does not say what happens to stale currentItem controls, how 404 differs from transient 500, or how retry authority is captured, allowing the obsolete revision to be reopened after failed recovery."
    fix_hint: "Define and test view/item/control/message/retry state for 404, 500, and auth failure, including late retry results."
```

## Required Re-review Evidence

Revise the failure-recovery addendum and convergence matrix to close BL-01 through BL-03, then obtain a new independent plan review with zero Critical, Warning, and Info findings. Record the final artifact path and PR comment URL in `08-PR-263-CONVERGENCE.md` before changing its status to `ready_to_resume`.

No implementation or tests were changed or executed during this plan-only review.
