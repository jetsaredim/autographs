---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: review_required
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

## Media Revision Authority Addendum

### Contract decision

Treat each catalog image UUID as a replaceable slot and bind review work to an immutable opaque `mediaRevision`. Derive the admin-only token by domain-separated SHA-256 over server-private fields that change on replacement, including the image UUID, object key, checksum/ETag when present, content type, and byte size. Return only the digest token; never return the source fields, and never include the token in public static artifacts.

Expose `mediaRevision` in authenticated admin image and review responses. Store it in the review session authority tuple alongside item/image IDs. All review-source URLs and draft/assist/Save/Reset requests carry the expected token through an explicit admin contract. A mismatch is HTTP 409 with a stable redacted response that tells the admin client the media changed; it is not reported as missing, provider failure, or ordinary preview retry.

### Server validation and atomicity

1. Centralize token derivation and constant-time-equivalent exact comparison over the loaded `AutographImage` snapshot.
2. Validate the expected revision before reading private media or doing expensive source/draft/assist work.
3. Re-load and validate the revision in one common terminal finalizer after media read/render/assist work on every success and error branch. Revision mismatch takes precedence and returns the stable 409 conflict even when the old-object read, derivative render, or assist proposal failed. Only when revision is still current may the operation return its original success or redacted provider/render 500.
4. Extend the repository adjustment mutation boundary to accept the server-private expected object key from the validated snapshot. In-memory updates compare it while holding the item lock. Oracle updates add `and object_key = :expected_object_key` to the adjustment SQL predicate. Zero affected rows with an existing image are a media-revision conflict, not not-found.
5. Apply the atomic predicate to both Save and Reset, including canonical no-op paths: a route may return no-op success only after a fresh revision check proves the snapshot is still current.
6. Preserve replacement rollback semantics by restoring the complete original `AutographImage` snapshot, including its saved adjustment. Repository replacement writes the adjustment supplied in `ImageReplacementInput`: the normal replacement route explicitly supplies `None`, while rollback supplies `existing_image.adjustment`. Oracle replacement SQL persists the supplied adjustment JSON rather than unconditionally setting it to null; the in-memory adapter follows the same contract. If complete restoration fails, return error and do not emit a success-shaped token or item response.
7. Define a typed media-revision conflict at the repository/route boundary rather than relying on generic error substring mapping. Its response is HTTP 409 with a stable redacted `{ code: "mediaRevisionConflict", message: "Image media changed. Reopen the review." }` contract. Missing image/item remains 404 and injected repository/provider failure remains 500.

### Client reconciliation

- `beginReviewSession` captures `mediaRevision`; every preview request, output callback, gesture authority check, assist response, Save/Reset completion, and deferred render checks the same value.
- Introduce one `reconcileAdminItemResponse(item)` boundary that runs before any `state.currentItem` assignment or editor render. When review is active, it locates the reviewed image in the response; missing image or mismatched `mediaRevision` both terminate gesture authority, abort/deauthorize preview/assist/mutation work, clear the review, and preserve a stable message that the image changed and must be reopened. Exact presence plus exact revision may continue.
- Route every item-returning client path through that boundary, including upload, set-primary, remove, replace, cleanup retry, Save, Reset, item save/load refresh, and any sibling direct `renderEditor(item)` call. Remove preassignment at call sites so reconciliation always precedes new current-item state and render side effects.
- `replaceImage` records its item/image target before issuing PUT, but uses the same central reconciliation rather than a special-case assignment. This covers replacement begun before review and same-ID response ordering.
- A 409 media-revision conflict from source/draft/assist/Save/Reset invalidates the review rather than retrying against the replacement media with old normalized coordinates.
- Never carry old adjustments across replacement: replacement continues to reset adjustment metadata to `None`, and stale Save/Reset cannot restore or clear it.

### Consumer and mutation inventory

| Surface | Required revision behavior |
|---|---|
| Admin item/image response | Includes opaque current `mediaRevision` for private client reconciliation only. |
| Review response/session | Captures current token and uses it in every subsequent authority tuple. |
| Source/private preview GET | Versioned authenticated URL/request validates before read and again before successful response. |
| Draft preview POST | Validates before render and after render; stale result is 409 and cannot create/mount a current blob. |
| Assist POST | Validates before private read/proposal and after proposal; stale result is 409 and cannot change corners/message. |
| Adjusted-output callbacks | Require session media revision in addition to request/blob/render/node authority. |
| Save PATCH / Reset DELETE | Validate token, then repository atomically compares expected private object key during the adjustment update/no-op decision. |
| Replacement PUT success/cleanup rollback | Normal replacement clears adjustment; rollback restores the complete original image including adjustment. Returned token identifies the actually persisted object; client compares before assignment/editor render. |
| All item-returning mutations | Pass through central presence+revision reconciliation before `state.currentItem` assignment; missing reviewed image and changed revision both invalidate. |
| Remove/item/session/logout teardown | Missing-image reconciliation or existing lifecycle invalidation discards revision-bound work. |

### Concurrency and regression matrix

- Start replacement, open review on the old revision, then resolve replacement before source load, before first move, between moves, while preview/assist is pending, and before Save/Reset response; every old review path invalidates and cannot mutate replacement metadata.
- Commit replacement between draft/assist pre-validation and post-validation; old result returns conflict and is never mounted/applied.
- Delete/replace the old object so source read, derivative generation, or assist fails after pre-validation; the common terminal revalidation returns 409 when revision changed and returns redacted 500 only when revision stayed current.
- Commit replacement between Save/Reset route validation and repository update; conditional update returns conflict and leaves replacement adjustment `None`.
- Exercise exact status separation: expected-object-key mismatch is 409 with the stable conflict body, missing image is 404, missing item is 404, and injected repository/provider failure is 500 with no private details.
- Replace with byte-identical media and identical checksum/ETag, content type, byte size, alt text, primary state, and sort order; the new random private object identity alone changes the opaque token. Rollback restores the original token.
- Exercise replacement cleanup success, cleanup warning, and rollback-after-warning-persistence failure from an original image with a non-identity saved adjustment; persisted object, adjustment, returned/error contract, and derived token agree in every branch.
- Exercise upload, set-primary, remove, replace, cleanup retry, Save/Reset, and item refresh responses while review is active; each reconciles before assignment, and missing image or revision mismatch invalidates consistently.
- Exercise another-session/external replacement by feeding a 409 into source, draft, assist, Save, and Reset; client teardown/message is consistent.
- Prove opaque tokens change on replacement, restore on metadata rollback, omit raw object key/checksum, remain absent from public JSON/HTML/manifests, and keep preview responses `no-store`.
- Preserve all gesture barrier, pointer capture, resize/grab-offset, mutation/egress, publisher promotion, legacy migration, privacy, and full-suite regressions.

### Required implementation surfaces

- `controller/src/routes.rs` for opaque revision derivation, authenticated response/request contracts, pre/post validation, conflict response, and client-visible revision fields.
- `controller/src/catalog.rs` and `controller/src/oracle_catalog.rs` for atomic expected-object-key adjustment updates and conflict classification.
- `controller/static-admin/admin.js` for revision-bound review authority and same-ID replacement reconciliation.
- `controller/tests/admin_workflow.rs`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`, Oracle source/unit coverage, and live-smoke compile coverage as appropriate.

### Completion criteria

This addendum may resume only after an independent reviewer approves the opaque token/privacy contract, all route/repository race closures, rollback semantics, client invalidation, and concurrency tests with zero findings. Implementation is complete only when full verification passes and a subsequent lineage-preserving deep source review reports zero Critical, Warning, and Info findings.

## Media Failure Recovery Authority Addendum

### Replacement and rollback state machine

Make image metadata replacement a typed expected-object-key compare-and-set for both initial replacement and recovery restoration.

| Transition | Expected key | New snapshot | Cleanup rule |
|---|---|---|---|
| Initial replacement | Original key loaded before upload | Replacement image with adjustment `None` | If CAS loses, reload metadata. Delete the uploaded candidate only when reload proves no catalog row references it; otherwise preserve it and return error/conflict. |
| Rollback after cleanup-warning persistence failure | Replacement key committed by this request | Complete original `AutographImage`, including saved adjustment | Reload and verify exact original key, revision, and adjustment. Delete replacement object only after verification succeeds. |
| Rollback failure or CAS loss | Replacement key expected, but repository fails or current key differs | No forced overwrite | Preserve original, replacement, and any newer candidate objects. Return redacted 500 or typed conflict, record manual-cleanup evidence, and never delete an object metadata may reference. |

Extend in-memory and Oracle replacement updates with an expected current object key and typed outcomes: `Updated`, `Conflict`, `NotFound`, and `Repository`. Oracle replacement SQL includes `object_key = :expected_object_key`; zero rows require a reliable reload to distinguish missing from conflict. The route verifies the postcondition after successful restoration before deletion.

Map every transition outcome exactly:

| Outcome | HTTP/body contract | Object/evidence behavior |
|---|---|---|
| Initial or rollback `Conflict` | 409 `{ code: "mediaRevisionConflict", message: "Image media changed. Retry from the current item." }` | Preserve all candidates until reload proves a candidate unreferenced. Never overwrite the winner. |
| `NotFound` item/image | 404 `{ code: "imageNotFound", message: "Image was not found." }` | Preserve candidates unless a reload proves they are unreferenced. |
| Repository failure | 500 `{ code: "imageRecoveryRequired", message: "Image recovery needs operator attention.", recoveryId }` | Preserve all candidates and emit structured private recovery evidence. |
| Restoration postcondition mismatch | Same redacted 500 recovery body | Treat as integrity/repository failure; delete nothing. |
| Replacement-object delete failure after verified restoration | Same redacted 500 recovery body | Original snapshot remains authoritative; replacement becomes an orphan candidate for manual cleanup. |
| Verified restoration and successful replacement deletion | Existing failure response for the original warning-persistence failure; no success-shaped replacement response | Reload has proved original key/revision/adjustment authoritative and replacement unreferenced. |

For every `imageRecoveryRequired` response, generate a UUID recovery ID and emit one structured `image_replacement_manual_recovery_required` error event containing the recovery ID, item/image IDs, transition, typed outcome, and SHA-256 fingerprints of candidate keys—not raw object keys, bucket, namespace, or media checksum. The HTTP body exposes only the recovery ID and redacted message. Source-contract/log-capture tests assert the event name and correlation fields; route tests assert status/body/recovery-ID shape and that candidates remain readable as required.

Tests must use a repository that succeeds initial replacement and then fails restoration, proving persisted metadata still references a readable replacement object. Add a concurrent newer-replacement fixture proving rollback CAS cannot overwrite the winner and no possibly active object is deleted. Preserve the successful restoration test with non-identity adjustment and exact revision restoration.

### Scoped conflict authority and recovery

Replace global unconditional conflict teardown with an async handler that accepts an explicit operation authority record and target item ID. Each caller evaluates stale/abort rules before allowing recovery:

| Operation | Required current authority before 409 may affect UI |
|---|---|
| Draft preview | Captured review session, item/image/media revision, preview request revision, draft revision, and abort-controller ownership. |
| Review open/source initialization | Captured open-review session plus target item/image and originating current-item revision. |
| Assist | Captured review session, item/image/media revision, draft revision, and assist request generation. |
| Save/Reset | Current mutation token, review session, item/image/media revision, and submitted draft/baseline authority. |
| Source/output callbacks | Existing session/request/blob/render/node/media-revision tuple; stale node callbacks remain inert. |

The source guide must not use a direct `<img src>` request for revision-bound acquisition. Add a source request revision/abort controller and fetch the authenticated source URL to a blob first. The fetch path can inspect 409 JSON, prove the full open-review session/item/image/media/source-request tuple, and invoke scoped recovery only when current. On 200 it creates an owned blob URL and mounts an `<img>` whose later load/error callbacks remain guarded by source request, blob URL, node, session, and media revision. Superseded source requests abort and revoke their blobs. Output node load/error remains ordinary blob-render authority because draft preview status was already observed by its fetch.

For an authoritative conflict:

1. Capture item ID and increment a dedicated conflict-recovery generation.
2. Invalidate review, gesture, preview, assist, and mutation state exactly once without rendering stale `state.currentItem` as recovered truth.
3. Fetch the current admin item from the server.
4. Before reconciliation, require the recovery generation and intended item/navigation context still match; otherwise discard the response.
5. Reconcile the fresh item through `reconcileAdminItemResponse`, then show stable “image changed; reopen review” guidance. A fresh Review action uses the refreshed revision and succeeds.
6. If refresh fails, follow the explicit recovery-state table below without restoring stale review authority.

Stale conflicts perform none of these steps. The handler must not clear a newer review, overwrite a later item/navigation choice, or issue a recovery fetch.

| Refresh result | Required state and action |
|---|---|
| 200 current item | Reconcile before assignment, render editor with current image revision, show reopen guidance, clear recovery state. |
| 404 | Keep `state.currentItem = null`, clear all review/item-action authority, return to collection view, show non-retryable “item no longer exists” guidance, clear recovery state. |
| 500/network | Keep `state.currentItem = null`, review cleared, and item/review controls unavailable. Store only `{ itemId, generation, status: "retryable" }` and show a dedicated “Retry item refresh” action. |
| Retry action | Increment recovery generation, capture item ID plus current navigation context, disable itself while pending, and run the same fetch/reconcile pipeline. Late results from older attempts are inert. |
| 401/403 | Invalidate recovery generation/state and execute existing logout/session-expired flow; no retry control or stale item remains. |
| Navigation/new review/logout while recovery pending | Increment/clear recovery generation before changing context; pending result is discarded. |

### Required regression matrix

- Deferred draft and assist requests from review A return 409 after review B mounts; B's session, nodes, message, and Save state are unchanged and no refresh occurs.
- Stale Save and Reset conflicts after mutation/session replacement are inert.
- Current draft, assist, Save, and Reset conflicts invalidate once, fetch/reconcile the current item token, show guidance, and immediate reopen succeeds.
- Current review-open and fetched-source conflicts follow the same exactly-once refresh/reconcile path; stale open/source 409s cannot affect a newer review. Output blob-node errors remain node-authority errors and never masquerade as 409.
- Recovery fetch resolves after navigation, different-item load, logout, or newer review open; generation/context checks discard it.
- Recovery 404 clears current item and item actions; recovery 500 exposes only a generation-bound retry with no stale controls; retry success restores fresh item/reopen; auth failure logs out. Late retry results are inert.
- Initial replacement CAS conflict, restoration success, restoration failure, restoration CAS loss to a newer replacement, verification mismatch, and replacement-object delete failure each assert catalog key/adjustment, object existence, response type, and cleanup evidence.
- No test permits deletion of `replacement_key` unless a reload first proves metadata points at the exact restored original snapshot.
- Route tests assert exact 409/404/500 bodies and recovery ID shape. Structured-evidence tests assert the private event/correlation fields and prohibit raw keys/checksums in HTTP responses.
- Preserve complete media-revision, gesture, output, mutation/egress, publisher, migration, privacy, and public-output suites.

### Completion criteria

Resume only after an independent reviewer approves the typed replacement state machine, verified deletion proof, per-operation conflict authority predicates, recovery-generation reconciliation, and failure tests with zero findings. Close only after implementation passes full all-feature verification and the next lineage-preserving deep review reports zero Critical, Warning, and Info findings.
