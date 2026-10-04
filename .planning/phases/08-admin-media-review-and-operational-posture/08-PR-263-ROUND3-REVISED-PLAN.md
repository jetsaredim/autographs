---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: approved
depends_on:
  - 08-REVIEW.md
  - 08-PR-263-CONVERGENCE.md
files_modified:
  - controller/static-admin/admin.js
  - controller/static-admin/admin.css
  - controller/tests/static_admin_behavior.mjs
  - controller/tests/static_admin.rs
  - controller/tests/admin_workflow.rs
---

# PR 263 Round 3 Revised Implementation Plan

## Contract Decisions

### Mounted adjusted-output authority

Introduce a monotonically increasing output-render generation that advances before every adjusted-output mount and every teardown/invalidation. A load/error callback is authoritative only when all of these still match current state: review session, item/image identity, draft revision, preview request/blob identity, output-render generation, and the exact mounted image node. Rerendering comparison or overlay controls creates a new generation even when the draft and preview URL are unchanged. Superseded callbacks do nothing; superseded blob URLs are revoked only when no current render owns them.

Preview status returns to `pending` whenever a new adjusted-output render is mounted. Save is enabled only when the current generation reports `ready` for the current draft revision. A failed current render stays retryable without allowing an older node to clear its error.

### Responsive perspective geometry

Keep normalized source corners as the sole authoritative geometry. Centralize `projectPerspectiveHandles()` so it derives fitted source bounds and visual pixels from current intrinsic dimensions, current source-frame dimensions, and normalized values. Invoke it after source load, every normalized-corner mutation, and every observed source-frame size change. Register one review-scoped `ResizeObserver` (with a window-resize fallback only if required by supported runtime behavior), and disconnect/invalidate it on rerender or teardown.

Clamp only each visual button center by half its interactive diameter within an unclipped overlay region, or provide equivalent overlay gutter, while retaining the exact normalized edge coordinate for requests and labels. Preserve the full 44px hit target for all four corners at narrow layouts.

### Persistence mutation and egress serialization

Represent Save/Reset submission with one review-scoped pending-mutation record containing operation, session, item/image, and request token. Before issuing PATCH/DELETE, enter pending state and disable every review egress surface: Back, tabs, item selection, logout, and any route/navigation action. Each handler must independently refuse egress while pending so programmatic invocation cannot bypass presentation state. Do not show a discard prompt for already submitted work.

Keep the review session alive until the mutation settles. On success, reconcile the returned item and adjustment into `state.currentItem`, the saved baseline, draft/dirty state, publish-boundary reporting, and success message before clearing pending state. On failure, retain the review and unsaved baseline, show the error, and then clear pending state. Only settled state may be left or discarded. Teardown invalidates any remaining observer/render callbacks.

### Browser-faithful executable tests

Split fake-DOM event delivery into a user-event helper that refuses disabled controls and an explicit programmatic-dispatch helper. Add controllable image-node events, retained detached nodes, a fake `ResizeObserver`, mutable bounding rectangles, and deferred mutation responses. Tests must assert both state and visible/control outcomes, and where applicable the repository/API request outcome.

## Coherent Implementation

### 1. Consolidate review lifecycle authority

- Add state/helpers for output-render generation, current mounted adjusted image, preview ownership, resize-observer ownership, and pending persistence mutation.
- Centralize invalidation so review open, rerender, retry, item/image change, Back after settlement, and logout cannot leave authoritative callbacks behind.
- Keep request revision and render generation distinct: request revision orders network results; render generation orders DOM instances created from an accepted result.
- Require the complete authority tuple before changing preview status, displayed revision, Save availability, stage content, or message state.

### 2. Make perspective projection responsive and accessible

- Centralize fitted-bounds calculation and handle projection from normalized corners.
- Observe frame geometry only for the current review/render lifecycle and reproject without changing model values.
- Ensure pointer conversion still uses the true fitted source bounds, not visually inset button centers.
- Preserve keyboard controls and labels after resize.
- Adjust overlay layout/CSS so full-size hit targets remain visible at all edges and the narrow breakpoint.

### 3. Serialize submitted mutations with navigation

- Add one pending-mutation guard used by Save, Reset, Back, tabs, item navigation, logout, and other review egress handlers.
- Keep success/failure reporting in the active review until the response is reconciled.
- Ensure Save advances the baseline only to the submitted/returned adjustment and preserves any legitimate enabled concurrent changes if such a producer exists; otherwise all edit producers remain disabled during submission.
- Ensure Reset success reconciles the no-adjustment baseline and current item before controls or egress reopen.
- Avoid cancellation claims unless the request and persistence layer implement actual cancellation; this plan uses serialization rather than cancellation.

### 4. Replace permissive test simulation with decisive interleavings

- Make disabled user inputs inert in the harness and update existing tests that relied on browser-impossible dispatch.
- Retain the first adjusted image, trigger a same-revision overlay/comparison rerender, then fire old/new load and error events in both orders; only the current node may affect state.
- Resize portrait and landscape source frames after load; prove normalized values stay stable, pixels reproject, and all four edge targets retain 44px reachability at narrow width.
- Defer Save and Reset responses, attempt Back/tab/item/logout, and prove no egress or hidden mutation occurs; then settle success and failure and assert current item, baseline, dirty state, messages, and navigation availability.
- Preserve prior request-order, legacy-release, and publisher failure-matrix coverage.

## Verification

- `node --check controller/static-admin/admin.js`
- `node --check controller/tests/static_admin_behavior.mjs`
- `node controller/tests/static_admin_behavior.mjs`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --all-targets`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
- `git diff --check`

## Non-Goals

- Do not alter publisher promotion, legacy-release migration, Oracle persistence, or public media contracts unless a failing regression proves the coherent lifecycle change affects them.
- Do not add request cancellation semantics.
- Do not replace normalized perspective data with pixel coordinates.
- Do not weaken dirty-state, publish-boundary, privacy, or private-original protections.
- Do not split the four findings into independent point-fix commits that duplicate lifecycle state.

## Done

The plan is implemented only when one shared lifecycle authority closes CR-01 and WR-01–03, all decisive interleavings pass in the executable DOM harness, focused Rust contracts remain green, the full verification set passes, and a lineage-preserving deep review reports zero actionable findings.

## Pointer Lifecycle Reassessment Addendum

### Revised contract decisions

#### Active drag is a render barrier

Treat a live perspective drag as exclusive ownership of the connected source frame and captured handle. Centralize a `renderImageReview` barrier that covers every known stage-replacement producer, not only pointer movement. The inventory must include source-image load callbacks, preview-fetch completion, adjusted-output load/error callbacks, assist completion, retry, comparison/overlay controls, scalar adjustment controls, keyboard perspective actions, Save/Reset, responsive projection callbacks, session/item/image changes, and teardown.

During an authoritative active drag, producers that are not needed for the gesture must be disabled or rejected. Asynchronous completions may update request-local bookkeeping only when their existing authority tuple remains current, but any DOM replacement or status/reporting render must be queued behind the drag barrier. Settlement releases capture/listeners first, applies the final normalized corner as the winning draft, and then performs one authoritative render/preview reconciliation. Cancel, lost capture, session invalidation, or teardown must explicitly discard or reconcile queued work according to the existing request/render authority; no callback may detach the owner mid-gesture or schedule stale work afterward.

Use one centralized helper for checking/queuing stage replacement so a newly added callback cannot bypass the gesture contract through a direct `renderImageReview()` call.

The barrier policy is closed per producer:

| Producer | Policy while drag is active | Settlement behavior |
|---|---|---|
| Source-image first load | A drag cannot start until intrinsic dimensions are non-zero and projection state is `ready`; handles remain disabled beforehand. A redundant authoritative load during drag may only reproject in place. | No queued full render. |
| Source-image error | End the gesture through the single terminal path before replacing frame children; keep the last accepted normalized corner, then show the source error in place. | If the gesture moved, schedule one final draft preview after cleanup; never recreate the stage from the error callback. |
| Preview fetch success | Accept/revoke blobs using the existing request/session/draft authority, record the newest accepted URL/status, and mark output rendering as deferred without calling `renderImageReview()`. | Discard any pre-gesture accepted output as superseded by the final gesture draft; schedule the final draft preview, whose result becomes authoritative. |
| Preview fetch failure | Record the still-authoritative failure as deferred; do not replace the stage or capture owner. | A moved gesture supersedes the old failure with its final preview request. If no move occurred, flush the failure render after cleanup. |
| Adjusted-output load | Update only the already-mounted output's authority/status in place; do not replace the stage. | Preserve the status if still current, unless superseded by the final gesture preview. |
| Adjusted-output error | Record the authoritative error and queue its visual render; do not call `renderImageReview()` during the drag. | A moved gesture supersedes it with the final preview. With no move, flush the error render after cleanup. |
| Assist completion started before drag | Reject the completion when a drag is active; do not apply or queue its proposal or message. | No flush. Operator may request assist again after settlement. |
| Retry, comparison, overlay, scalar, keyboard, Save, Reset, navigation/logout | Disable where represented by a control and reject again in the handler. | No queued action; the operator may retry explicitly afterward. |
| Resize observation | Reproject the same connected handles in place. Rebase the normalized grab offset using the last pointer client coordinates and current fitted bounds so the mathematical corner does not move merely because layout changed. | Continue the same gesture and basis; no render is queued. |
| Session/item/image change or review teardown | Invalidate the gesture and all deferred render state before any replacement. | Discard without preview because the owning review is no longer current. |

#### Preserve source-space grab offset

Handles are non-interactive until the authoritative source image has non-zero intrinsic dimensions and fitted bounds with non-zero width and height. Therefore pointerdown-before-source-load is rejected by construction and cannot create a gesture with a fallback frame coordinate basis.

At pointerdown after that precondition, convert the pointer to normalized fitted-source coordinates and record `grabOffset = authoritativeCorner - pointerPosition`. On each move, convert the new pointer through the current fitted bounds, add the normalized grab offset, clamp the resulting mathematical corner to `[0, 1]`, and store it. The visually inset handle center remains presentation-only.

A pointermove whose coordinates equal pointerdown must leave the normalized model unchanged. Moving an inset edge handle by a source-space delta must move the mathematical corner by the same delta without first jumping from the image edge to the inset center. The gesture stores the last client coordinates. If fitted geometry changes, recompute the normalized grab offset as `currentCorner - normalized(lastPointer)` under the new bounds before accepting another move; this makes resize itself a model no-op and avoids stale pixel geometry.

#### Terminal-state table

| Terminal event | Capture/listener behavior | Model and deferred-work behavior |
|---|---|---|
| `pointerup` | Mark terminal before releasing capture; remove all gesture listeners; release capture only if still held. A resulting `lostpointercapture` observes terminal state and is a no-op. | Commit last accepted corner. If moved, discard pre-gesture deferred preview/error renders and schedule exactly one final preview. If not moved, flush the newest authoritative deferred render once. |
| `pointercancel` | Same idempotent cleanup and release ordering as pointerup. | Commit the last already-accepted corner using the same moved/no-move reconciliation as pointerup; do not invent a rollback that would reuse old revisions. |
| External `lostpointercapture` | Enter the same terminal function once; it must not call release recursively when capture is already absent. | Same commit/reconciliation rule as pointercancel. |
| Source-image error | Enter terminal function before replacing source-frame children. | Commit accepted movement, reconcile preview exactly once, then show the source error in place. |
| Review teardown or session/item/image invalidation | Mark terminal, remove listeners, and release only if safe/current; ignore reentrant lost-capture. | Discard queued render state and do not schedule preview for the retired review. |

### Coherent implementation requirements

1. Inventory every direct `renderImageReview()` caller and every callback that can indirectly replace the source frame; route them through the active-drag render barrier or prove they mutate only connected in-place geometry.
2. Preserve existing request revision, blob ownership, render generation, node identity, review session, item/image, and perspective-generation checks when deferred work is reconciled.
3. Keep the connected handle and frame stable across async source load, preview response, adjusted load/error, and assist response interleavings before the first move and between multiple moves.
4. Record source-space grab offset at pointerdown and apply it to every move, including all four full-frame corners whose visual centers are inset by 22px.
5. Settle or cancel exactly once across pointerup, pointercancel, lostpointercapture, resize invalidation, review teardown, and session/item/image changes; remove listeners and queued work deterministically.
6. Preserve keyboard, assist, overlays, comparison, scalar adjustments, dirty state, Save authorization, responsive projection, and mounted-output authority outside an active drag.
7. Add explicit source projection readiness state. Do not attach enabled pointer/keyboard handlers until source intrinsic/fitted geometry is valid; source error must leave no enabled handles.
8. Make gesture termination idempotent before any `releasePointerCapture()` call so reentrant `lostpointercapture` cannot settle or flush twice.

### Decisive regression matrix

- Before source load, handles are absent or disabled and pointerdown cannot start a gesture. After first authoritative load, begin a drag; a redundant load callback may only reproject the same connected handle in place.
- Begin a drag, then deliver source-image error before the first move and after a move; cleanup occurs once, capture is released safely, the source error appears in place, and preview reconciliation follows the terminal table.
- Begin a drag with an older preview request/output pending, then resolve success and error variants between moves; no stage replacement occurs, stale authority remains rejected, and one final settled preview wins.
- Begin a drag, then resolve an assist request; verify the explicit reject/defer policy and that the active draft remains authoritative.
- For each of four edge handles, pointerdown at the visual center followed by a same-coordinate move is a model no-op.
- For portrait and landscape fitted bounds, two pointer deltas from an inset handle produce matching normalized source deltas without a first-move jump.
- Resize during drag rebases the normalized grab offset from the last pointer coordinates; resize itself is a model no-op and the next delta uses current fitted geometry.
- Pointerup, pointercancel, lost capture, teardown, and session switch each prove exactly-once listener cleanup and preview/deferred-work behavior.
- Pointerup-triggered capture release and its reentrant `lostpointercapture` callback produce only one settlement and one preview/deferred-render reconciliation.
- User dispatch continues to reject disconnected nodes; explicit programmatic dispatch remains limited to intentional stale-callback or handler-guard tests.

### Addendum completion criteria

This reopened plan is ready to resume only after independent review confirms the async producer inventory, centralized render barrier, deferred-work reconciliation, grab-offset coordinate math, failure matrix, and browser-faithful tests. Implementation is complete only when a subsequent lineage-preserving deep source review reports zero Critical, Warning, and Info findings.
