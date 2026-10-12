---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: 2e003dfd604b7b0ca7ed53c9e0ae11b74ef041fd
reviewed: 2026-10-04
verdict: revisions_required
blockers: 2
warnings: 1
info: 0
---

# PR 263 Pointer Lifecycle Plan Review

## Verdict

**REVISIONS REQUIRED — two blockers, one warning, and zero advisories.**

The reopened reassessment correctly preserves lineage: the surviving async capture-owner defect is an incomplete fix/sibling-path miss, while the inset-handle jump is a fix regression exposed by a test weakness. The addendum also keeps the work bounded to one review-lifecycle authority change and preserves the already-closed mounted-output, persistence-egress, publisher, migration, privacy, and public-output contracts.

Coder work must not resume yet. The gesture contract still leaves authority and coordinate decisions unresolved at the exact interleavings that triggered this reassessment.

## Blockers

### 1. Active-drag producers need one deterministic policy and settlement contract

**Severity:** BLOCKER  
**Dimension:** key links / convergence readiness  
**Required property:** Every synchronous or asynchronous producer capable of replacing the review stage, changing the drag-owned draft, or changing preview/reporting authority must have one declared behavior while a drag is active and one declared flush/discard behavior for each terminal path.

**Evidence:** The addendum establishes a centralized barrier, but it still offers materially different outcomes instead of selecting the contract. It says producers may be disabled or rejected, async completions may update bookkeeping while renders are queued, assist may be rejected or deferred, and resize may either cancel or rebase. It also groups pointerup, pointercancel, lost capture, invalidation, and teardown under “settle or cancel exactly once” without stating whether the last accepted geometry is committed or reverted and whether accepted preview/blob/assist work is flushed, superseded, or discarded on each path. Those choices affect draft revision, blob ownership, preview status, dirty state, and the single settled preview. A coder can satisfy the prose locally while producing conflicting behavior between source load, preview success/error, adjusted-output error, assist completion, and cancellation.

**Example fix (non-binding):** Add a producer/terminal policy table. For each source load/error, preview success/error, adjusted-output load/error, assist success/error, retry, comparison/overlay/scalar/keyboard action, mutation/navigation/session transition, and resize callback, name `in-place`, `reject`, `defer`, or `teardown`; then state what pointerup, pointercancel, lost capture, resize invalidation, session/item/image invalidation, and teardown do with geometry, queued render intent, accepted blobs, messages/status, and preview scheduling.

### 2. Grab-offset authority is undefined when capture begins before intrinsic source geometry exists

**Severity:** BLOCKER  
**Dimension:** cross-contract correctness  
**Required property:** The coordinate contract must define an available, stable basis for the initial grab offset and its transformation whenever intrinsic or rendered fitted bounds change, including the required source-load-after-pointerdown interleaving.

**Evidence:** The plan requires pointerdown to compute the pointer in “true fitted-source coordinates,” while its decisive matrix also requires a drag to begin before the source image `load` event and continue on the same capture node after load. In the current seam, handles are mounted before load and `sourceRenderedBounds()` falls back to frame coordinates when `naturalWidth`/`naturalHeight` are unavailable. The later load may introduce different intrinsic dimensions and letterboxed fitted bounds. A source-space offset computed before that event therefore has no defined conversion into the new source basis. The plan’s resize alternative (“terminate” or “recompute from a stable normalized grab offset”) does not resolve this case, and termination would contradict the required test that the original captured node remains connected and receives multiple moves.

**Example fix (non-binding):** Define a pre-load gesture basis and a mandatory rebase rule that preserves the authoritative normalized corner and the last pointer client position when intrinsic/fitted geometry becomes available, or revise the interaction contract so capture cannot begin until intrinsic geometry exists and adjust the source-load regression consistently. In either case specify zero-distance, clamping, portrait/landscape letterboxing, and rendered resize behavior in the same coordinate equations.

## Warning

### 3. The executable matrix omits source-image error and real pointer-capture release semantics

**Severity:** WARNING  
**Dimension:** verification derivation  
**Required property:** Browser-faithful connected-node tests must exercise every callback that can detach the capture owner and must prove capture/listener release and queued-work disposition across every termination path.

**Evidence:** The explicit inventory and regression matrix name source-image load but not the sibling source-image error callback. The current error callback calls `disconnectPerspectiveProjection()` and `sourceFrame.replaceChildren(...)`, so it directly terminates capture and detaches the handle. The harness also implements `setPointerCapture()` only; it has no `releasePointerCapture()`/`hasPointerCapture()` behavior or assertion that capture is released before the one authoritative reconciliation. The plan names pointerup, pointercancel, lost capture, teardown, and session switch, but does not require source-error delivery or item/image invalidation as distinct connected-node cases.

**Example fix (non-binding):** Add controlled source-error-during-drag coverage and a minimal fake pointer-capture lifecycle that can assert exactly-once release/listener removal and re-entrant lost-capture safety. Cover pointerup, pointercancel, lost capture, resize policy, source error, review teardown, and session/item/image invalidation with the policy selected for Blocker 1.

## Goal-Backward Coverage That Passes

- Finding lineage is correctly classified and attributable.
- The addendum inventories the principal direct and indirect `renderImageReview()` producers and requires one centralized stage-replacement barrier.
- Request revision, blob identity, output-render generation, mounted-node identity, review session, item/image, and perspective generation remain required authority inputs.
- The intended grab-offset equation covers inset handles, zero-distance moves, clamping, and source-space deltas once a stable fitted-source basis exists.
- The regression matrix uses connected nodes and user-event delivery, includes async interleavings before and between moves, and preserves explicit programmatic dispatch for intentional stale-callback tests.
- The proposed change remains one coherent admin lifecycle/test-harness change and retains the full verification suite and already-closed contracts.

## Resume Condition

Keep `08-PR-263-CONVERGENCE.md` at `reassessment_required`. Revise the addendum to select the producer and terminal policies, define grab-offset rebasing across the pre-load/intrinsic-geometry transition, and extend the executable matrix for source error and pointer-capture release. Then run another independent plan review. Only an approval with zero blockers, warnings, and advisories may be recorded as implementation-plan evidence before coder work resumes.
