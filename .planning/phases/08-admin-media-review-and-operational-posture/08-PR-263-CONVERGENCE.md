---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: reassessment_required
trigger_review: 08-REVIEW.md
assumption_revision_evidence: 08-PR-263-ROUND3-REVISED-PLAN.md#ambiguous-delete-and-exclusive-recovery-addendum
implementation_plan_review_evidence: pending
implementation_plan_review_comment: pending
---

# PR 263 Review/Fix Convergence Reassessment

## Trigger

The mandatory third review retained one Critical and three Warning findings after two coder passes. The remaining defects share lifecycle and authority boundaries rather than isolated branches: a same-draft render can be superseded without invalidating its callbacks, responsive geometry can change without rerendering normalized guides, a submitted mutation can outlive the review UI that reported it, and the DOM harness permits browser-impossible disabled-control input while omitting those interleavings.

Per the repository convergence guard, no further coder pass may begin until this reassessment and the revised implementation plan are independently reviewed, and both evidence references are recorded here.

## Finding Lineage

| Finding | Classification | Current state |
|---|---|---|
| Round 1 CR-01: transform/pixel-review parity | Incomplete fix across shared preview contract | Server rendering is shared, but Round 2 CR-01 and Round 3 CR-01 show that displayed-output authority is not yet tied to the mounted render instance. Round 2 CR-04 and Round 3 WR-01 show that normalized source geometry is not yet reprojected after layout changes. |
| Round 1 CR-03: dirty/navigation/publish guards | Sibling action-path miss | Dirty-state navigation is guarded, but Round 2 CR-02 and Round 3 WR-02 show that submitted Save/Reset mutations are not serialized with review egress. |
| Round 2 CR-01: displayed-preview revision authority | Incomplete fix / sibling-path miss | Draft and request revisions reject older draft work, but two render instances can share both identities. Round 3 CR-01 remains open. |
| Round 2 CR-02: in-flight Save baseline | Resolved for returned repository state; sibling navigation path remains | Later draft changes are no longer marked saved, but Back/discard can hide an already submitted mutation. Round 3 WR-02 remains open. |
| Round 2 CR-03: promotion cleanup safety | Resolved | The reviewed publisher failure matrix preserves `current` and its active map before and after promotion. |
| Round 2 CR-04: perspective coordinate parity | Incomplete fix / fix regression | Pointer-to-source conversion is correct at initial layout, but absolute handle positions become stale after resize and full-frame hit targets are clipped. Round 3 WR-01 remains open. |
| Round 2 CR-05: async review-session authority | Sibling-path miss | Session and item checks reject older requests, but same-session render instances and submitted mutations need their own authority boundaries. Round 3 CR-01 and WR-02 remain open. |
| Round 2 CR-06: clean-state action reachability | Resolved | Back and Reset remain reachable when clean. |
| Round 2 WR-01: legacy active-release migration | Resolved | The active legacy artifact remains protected until a successful publish supplies its public-current map. |
| Round 2 WR-02: executable DOM coverage | Incomplete fix / test weakness | The harness executes listeners, timers, and fetches, but Round 3 WR-03 shows that it still models disabled controls incorrectly and omits the decisive interleavings. |
| Round 3 CR-01 | Open Critical | Detached same-revision image callbacks can authorize Save or overwrite a newer mounted render. |
| Round 3 WR-01 | Open Warning | Perspective handles are not reprojected after resize and edge hit targets can be clipped. |
| Round 3 WR-02 | Open Warning | Back/discard can hide a Save or Reset that continues mutating persistence. |
| Round 3 WR-03 | Open Warning / test weakness | Browser-impossible disabled-control events and missing interleavings allow the defects above to pass. |

### Post-convergence pointer lineage

| Finding | Classification | Current state |
|---|---|---|
| Post-convergence WR-01: capture owner replaced on first move | Incomplete Round 3 WR-01 fix / sibling-path miss | Commit `7c11380` keeps the stage mounted for ordinary pointer moves and settles one preview at gesture end. |
| Pointer follow-up WR-01: async completion replaces capture owner | Incomplete fix / sibling-path miss | Open: source-image load and pre-existing preview/output-error/assist completions can still rerender while a drag owns pointer capture, especially before the first move. |
| Pointer follow-up WR-02: inset target changes normalized corner | Fix regression / test weakness | Open: mapping the absolute pointer position directly to source coordinates ignores the grab offset introduced by visually inset 44px edge targets, so first movement jumps the mathematical corner. |

The previously approved plan covered synchronous edit producers but assumed that keeping `pointermove` itself from rerendering was sufficient. The follow-up review disproved that assumption: all asynchronous UI completions capable of stage replacement are also drag-time producers, and the visual hit-target projection needs an inverse mapping that preserves the exact point grabbed.

### Media-replacement lineage

| Finding | Classification | Current state |
|---|---|---|
| Gesture-lifecycle CR-01: same-ID replacement preserves stale review authority | Independent sibling mutation-boundary miss | Open: a replacement PUT started before review can commit new private bytes under the same item/image UUID while the review, gesture, draft previews, and later adjustment save remain bound only to those UUIDs. Old-source geometry can then be persisted for the replacement media. |

The approved gesture plan correctly serialized callbacks inside one review session, but assumed item/image UUIDs identify immutable media. Replacement intentionally preserves the image UUID while changing its object key, checksum, byte size, intrinsic geometry, and adjustment baseline. Review authority must therefore include an opaque media revision and the persistence boundary must enforce it atomically.

### Media failure-recovery lineage

| Finding | Classification | Current state |
|---|---|---|
| Media follow-up CR-01: failed rollback deletes referenced replacement object | Incomplete rollback fix / failure-path sibling miss | Open: restoration failure is logged, but replacement cleanup runs unconditionally and can leave committed metadata pointing at deleted private media. |
| Media follow-up CR-02: unscoped 409 tears down newer review and retains stale reopen token | Conflict-terminal regression / recovery sibling miss | Open: conflict handlers run before operation authority checks; an authoritative conflict clears review without refreshing the item revision, so stale conflicts destroy newer state and current conflicts can loop forever on reopen. |

The media-revision implementation bound normal work to immutable media, but assumed failure cleanup could be best-effort and conflict handling was globally authoritative. Recovery mutations require compare-and-set plus verified postconditions before destructive cleanup, and conflict UI effects require the same per-operation authority tuple as success effects plus an authority-scoped fresh-item recovery.

### Recovery source-review lineage

| Finding | Classification | Current state |
|---|---|---|
| Recovery source CR-01: ambiguous old-object DELETE followed by rollback can restore missing media | Incomplete recovery fix / external-side-effect sibling miss | Open: an OCI delete may apply before returning a transport error. Metadata rollback currently proves only catalog state, not that original bytes still exist and match, then deletes the valid replacement. |
| Recovery source CR-02: refresh can overwrite same-view editor work | Incomplete conflict recovery / sibling action-path miss | Open: conflict recovery renders an actionable blank editor and accepts a late result based on generation/navigation/view only; same-view edits/save/render paths neither block nor invalidate recovery. |

The reviewed recovery plan assumed an error meant the old object remained and that staying on the same editor view meant no newer user authority existed. Both are false. Destructive rollback requires positive media readability/integrity proof, and recovery must exclusively own an inert UI state or be invalidated by any editor mutation/assignment generation.

## Shared Invariants

### 1. Mounted-output authority

Only the currently mounted adjusted-output image may transition preview status or authorize Save. Authority is the conjunction of current review session, current item/image, current draft revision, current preview request/blob identity, current output-render generation, and current DOM node identity. Equal draft content does not make two render instances interchangeable.

### 2. Normalized geometry authority

Perspective corner values remain normalized source coordinates and are the only mutable geometry model. Every intrinsic-size or rendered-frame-size change must reproject that model into visual handle positions. Visual hit-target clamping may move a button center inward, but must never alter the normalized corner value.

### 3. Mutation/navigation serialization

Once Save or Reset crosses the persistence boundary, the review cannot be discarded or left until the mutation settles and its result is reconciled into the current item, baseline, messages, and publish-boundary state. UI disabling is presentation; every egress handler must also enforce the pending-mutation invariant.

### 4. Test-environment fidelity

User-event helpers must obey browser disabled-control behavior. Programmatic dispatch, when intentionally needed, must be explicit. The harness must control image events, resize observation, deferred fetches, detached nodes, and navigation attempts so each authority boundary is executable and asserted.

### 5. Gesture isolation and reversible projection

An active perspective gesture creates a render barrier around its connected capture owner. Every synchronous action and asynchronous completion that could replace the source frame or handle must either be rejected, made in-place, or deferred until settlement. The visual inset used to preserve a 44px target is a presentation transform; pointer dragging must preserve the initial source-to-pointer grab offset so the normalized corner does not jump when the gesture begins.

### 6. Immutable media-revision authority

Item ID plus image ID identifies a catalog slot, not the bytes currently occupying it. Every review response, source/draft preview, assist proposal, adjusted-output callback, Save/Reset mutation, and client reconciliation must bind to one opaque media revision derived from server-private media metadata. The opaque token must not reveal an Object Storage key, namespace, bucket, or raw checksum. Adjustment persistence must compare the expected private object key atomically in the repository update so a replacement cannot race between route validation and write.

### 7. Failure recovery preserves the last committed authority

Rollback is a conditional state transition, not a logging side effect. Initial replacement and restoration both compare the object key expected at their mutation boundary. No object may be deleted until a reload proves catalog metadata no longer references it and the intended snapshot is authoritative. A failed or lost rollback preserves all possibly referenced objects and returns a redacted error for manual recovery.

A media-conflict response is authoritative only for the exact session, request/draft revision, assist request, mutation token, item/image, and media revision that issued it. Stale conflict responses are inert. A current conflict invalidates once, then refreshes and reconciles the admin item under a recovery generation before inviting reopen; the old `state.currentItem` token is never reused as recovery state.

### 8. External deletion proof and exclusive recovery ownership

An Object Storage delete error is outcome-ambiguous: the object may still exist or may already be gone. Because the replacement is already committed and known readable at that point, it remains authoritative on every delete-error path. The route must not restore original metadata or delete the replacement after an ambiguous delete result.

While conflict refresh is pending or retryable, recovery exclusively owns an inert recovery surface. No editor form input, Save, image action, direct editor render, or item assignment may proceed without first invalidating recovery. A recovery response is current only when its recovery generation, navigation context, editor-authority generation, and inert recovery state all still match.

## Complete Consumer and Action Inventory

| Invariant | Producers | Consumers and reporting paths | Mutation / invalidation boundaries | Required tests |
|---|---|---|---|---|
| Mounted-output authority | Draft preview fetch, blob URL, `renderImageReview`, adjusted `<img>` creation | Output stage, loading/error text, Save enabled state, displayed revision | Review open/close, item or image switch, assist/manual/reset draft, overlay/comparison rerender, retry, blob replacement/revocation | Detached old image load/error after same-revision rerender; older request completion; session/item switch; retry |
| Normalized geometry authority | Intrinsic image dimensions, normalized corners, fitted source bounds, frame observer | Handle positions, labels, pointer and keyboard updates, preview request payload | Image load, responsive resize, breakpoint/layout change, manual/assist corner change, review teardown | Portrait and landscape resize; all four full-frame corners; 44px hit targets; pointer/keyboard parity |
| Mutation/navigation serialization | Save PATCH, Reset DELETE, mutation-pending state and token | Back, tabs, item selection, logout, messages, current item, baseline, dirty state, publish controls | Mutation submit, success/failure settlement, session teardown | Save→Back, Reset→Back, tab/item/logout during pending mutation, success/failure reconciliation |
| Test fidelity | Fake elements, user-event helper, timers, deferred fetch queue, fake image events, fake resize observer | All DOM behavior assertions | Disabled/enabled transition, node detach, render replacement, resize delivery, deferred mutation completion | Browser-impossible events rejected; explicit programmatic events supported; all three open behavioral findings reproduced before fix and closed after fix |
| Gesture isolation | Pointerdown/capture, source-image load, preview fetch response, adjusted-image load/error, assist completion, retry/comparison/overlay/control actions, resize observer | Connected capture owner, source frame, handle projection, preview status/message, settled preview scheduling | Pointer move/up/cancel/lost capture, session/item/image/generation change, stage rerender, async completion, teardown | Async source/preview/error/assist completions before first move and between moves; capture owner stays connected; settlement/teardown flushes or discards deferred work exactly once |
| Reversible visual projection | Normalized corner, fitted source bounds, 22px visual inset, pointerdown position | Pointer-to-source conversion, handle pixels, labels, preview payload | Edge/corner pointerdown, multiple moves, resize during gesture, cancel/settlement | Grab centered on all four inset edge handles; zero-distance move is a no-op; subsequent deltas change the normalized corner by the same source-space delta without a jump |
| Immutable media revision | Replacement upload/metadata commit, opaque revision derivation, review response, admin item response | Review session, preview URLs/requests, assist, output callbacks, Save/Reset, editor reconciliation, dirty/publish reporting | Replacement started before/during review, route pre/post validation, atomic repository update, cleanup-warning rollback, review teardown | Same-ID replacement resolves before first move, between moves, during preview/assist, before Save/Reset repository write, and after route validation; stale work returns conflict/invalidates and cannot persist |
| Rollback object safety | Initial replacement CAS, cleanup warning persistence, rollback CAS, metadata reload, object deletion | Active catalog object, original/replacement/newer-concurrent objects, cleanup warning/manual recovery | Initial replacement race, rollback success/failure/race, verification failure, delete failure | Restoration failure and concurrent replacement preserve every possibly referenced object; deletion follows verified restoration only |
| Scoped conflict recovery | Captured preview/session/draft/assist/mutation authority, conflict handler, recovery generation, fresh item load/reconciliation | Current review, newer review, current item token, editor message/reopen action | Old 409 after review switch, current 409, recovery request race/navigation/auth failure | Stale conflicts cannot alter newer review; current conflict refreshes token and immediate reopen succeeds; stale recovery result cannot overwrite navigation |
| Ambiguous delete safety | Old-object delete result, committed replacement snapshot, cleanup-warning persistence, recovery evidence | Catalog snapshot, original and replacement objects, manual cleanup | Delete applied-then-error, delete error-with-object-intact, warning persistence success/failure | Every delete-error path keeps the readable replacement authoritative and never deletes it; warning failure preserves both possible objects and returns recovery evidence |
| Exclusive recovery ownership | Recovery generation, editor-authority generation, recovery view/state, disabled controls, retry action | Form inputs, Save, renderEditor/reconcile, item/image actions, navigation | Pending refresh, retryable failure, same-view edit/save/render, navigation/logout/new review | Recovery surface is inert; programmatic/user edits cannot proceed; any newer editor assignment invalidates recovery before late response can reconcile |

## Assumption Audit

- Rejected: matching review session and draft revision is sufficient callback authority.
- Revised: every mounted adjusted-output render needs a unique generation and node identity; callbacks from superseded nodes are stale even when URL and draft content are unchanged.
- Rejected: computing perspective handle pixels on load or interaction is sufficient.
- Revised: normalized corners are durable state, while pixels are disposable projections recomputed whenever intrinsic or rendered geometry changes.
- Rejected: disabling edit controls is sufficient during persistence mutation.
- Revised: all review egress actions must be blocked in handlers as well as visually disabled until the submitted mutation settles; submitted work cannot be described as discardable without cancellation semantics.
- Rejected: a synthetic DOM event helper may invoke listeners on disabled controls and still demonstrate browser user behavior.
- Revised: user-event dispatch must honor `disabled`; programmatic events require a distinct explicit primitive, and correctness claims require controlled detached-node, resize, and deferred-request interleavings.
- Retained: publisher promotion/cleanup atomicity and legacy-release migration are closed unless the coherent change touches those contracts.
- Rejected: preventing `pointermove` from rerendering is enough to keep the capture owner mounted.
- Revised: active drag is a render barrier covering every synchronous producer and asynchronous completion that can replace the stage; deferred effects must be explicitly flushed or discarded at settlement/teardown.
- Rejected: absolute pointer-to-source mapping remains correct when an edge handle's visual center is inset from its normalized corner.
- Revised: pointerdown records the source-space grab offset between the mathematical corner and pointer; every move applies that offset before normalization, so a zero-distance move cannot change the model.
- Rejected: stable item/image UUIDs are sufficient authority for a review or adjustment mutation.
- Revised: the image UUID is a replaceable slot. An opaque media revision binds client work to a specific private-object snapshot, and the repository mutation condition uses the server-private expected object key to close the validation/write race.
- Rejected: invalidating the UI when the replacement response arrives is sufficient.
- Revised: client invalidation is required for prompt UX, while server pre/post validation and atomic compare-and-set remain authoritative for concurrent requests, other sessions, and response-order races.
- Rejected: rollback can attempt restoration, log failure, and still delete the replacement object.
- Revised: restoration is expected-key CAS followed by exact snapshot reload/verification; destructive cleanup is permitted only after proof that the object is unreferenced. Failure/race preserves objects and returns error.
- Rejected: every media-revision 409 is globally authoritative and clearing review is sufficient recovery.
- Revised: conflict handling first proves the originating operation tuple is current. Authoritative conflict recovery invalidates once and fetches/reconciles a fresh item under its own generation before showing reopen guidance.
- Rejected: an Object Storage delete error proves the old object remains available for rollback.
- Revised: delete errors are outcome-ambiguous. Do not roll back metadata after any old-object delete error. Keep the already-committed readable replacement authoritative; persist a cleanup warning when possible, otherwise preserve every candidate and return correlated recovery-required evidence.
- Rejected: recovery generation plus same route/view is sufficient to protect a pending refresh from editor work.
- Revised: recovery owns a dedicated inert state and captures editor-authority generation. All edit/save/render/assignment paths block or invalidate recovery, and acceptance requires both generations plus the inert state.

## Failure Matrix

| Scenario | Required behavior |
|---|---|
| Old adjusted image loads after a same-draft overlay/comparison rerender | Callback is ignored; it cannot enable Save, clear an error, or replace the current stage. |
| Old adjusted image errors after the replacement loads | Callback is ignored; successful current output stays authoritative. |
| Older preview request resolves after a newer request/render | Older response and all later node callbacks are ignored and its blob is safely revoked. |
| Review/item/session changes before response or image event | Result is ignored and cannot mutate the new review. |
| Source frame resizes after image load | All handles are reprojected from normalized values; stored corners remain unchanged. |
| Corner is exactly on any source edge at narrow width | Normalized value remains at the edge; the full 44px interactive target remains reachable and visible. |
| Save or Reset is pending and Back/tab/item/logout is attempted | Egress is blocked without a discard prompt; the visible review remains authoritative until settlement. |
| Pending mutation succeeds | Current item, baseline, dirty state, publish boundary, messages, and controls reconcile before egress is re-enabled. |
| Pending mutation fails | Error remains visible in the same review, no baseline is advanced, and egress is re-enabled consistently. |
| User-event helper targets a disabled control | No browser-user listener executes. |
| Test intentionally needs programmatic dispatch | A separate explicit helper dispatches it and the test states why. |
| Source-image load fires after pointer capture but before first move | Capture owner remains connected; projection may update in place, but no full stage replacement occurs. |
| Pointerdown is attempted before authoritative source intrinsic/fitted geometry exists | No gesture starts; handles are absent or disabled until projection readiness is established. |
| Source-image error fires during drag | Gesture terminates exactly once before the source frame changes; accepted movement is reconciled once and the error is shown in place. |
| Preview response or adjusted-image load/error arrives during drag | State may be recorded, but stage replacement/reporting that detaches the owner is deferred until settlement and remains subject to request/render authority. |
| Assist completion arrives during drag | It cannot replace or mutate the active gesture; it is rejected as stale or deferred under an explicit single-owner policy. |
| Drag settles normally with deferred work | Capture releases, final normalized geometry wins, and exactly one authoritative render/preview sequence reconciles deferred status. |
| Drag is cancelled, loses capture, or is torn down | Listeners/capture/deferred work are cleared according to the documented cancel contract; no stale render or preview is scheduled. |
| Pointerup releases capture and synchronously/reentrantly emits `lostpointercapture` | Terminal state is already marked, so the second callback is a no-op and no duplicate preview/render flush occurs. |
| Pointerdown occurs at the visual center of an inset edge handle, followed by zero movement | Normalized corner is unchanged. |
| Pointer moves from an inset edge handle | The initial grab offset is preserved and the normalized corner changes only by the pointer's source-space delta. |
| Replacement begins before review, then returns while review/drag is active | Returned item carries a different opaque media revision; review/gesture/requests are invalidated before editor reconciliation, and old geometry cannot remain savable. |
| Replacement commits after stale draft preview or assist starts | Route revalidation rejects/suppresses the old-media result; the client treats revision conflict as review invalidation, not a retryable same-media preview error. |
| Replacement deletes/replaces the old object while source/draft/assist work fails | Final authority revalidation runs on the error path; revision mismatch returns the stable 409 conflict instead of provider/render 500. The original operation error is returned only if revision is still current. |
| Replacement commits after Save/Reset route validation but before repository write | Atomic expected-object-key predicate updates zero rows and returns conflict; replacement adjustment baseline remains unchanged. |
| Replacement cleanup warning persistence fails and metadata rolls back to original object | Repository restoration writes the complete original image snapshot, including its saved adjustment. The original opaque revision and adjustment baseline are restored together; partial rollback is an error and no success response is emitted. |
| Returned item omits the actively reviewed image | Central reconciliation invalidates review/gesture/async authority before assigning `state.currentItem` or rendering the item, exactly as for revision mismatch. |
| Any item-returning mutation responds while review is active | Upload, set-primary, remove, replace, cleanup retry, Save/Reset, and other item responses pass one reconciliation boundary before current-item assignment. |
| Expected object key mismatches / image is absent / item is absent / repository fails | Responses are respectively stable redacted 409 / 404 / 404 / 500; tests prove the classes do not collapse into the string-error fallback. |
| Replacement uploads byte-identical media with identical public/basic metadata | New private object identity alone changes `mediaRevision`; rolling metadata back to the complete original snapshot restores the original token and adjustment. |
| Opaque revision is exposed through admin API | Token cannot be reversed into or used as an OCI namespace, bucket, object key, signed URL, or raw media checksum; no token enters public static output. |
| Initial replacement metadata CAS loses to another replacement | Candidate cannot overwrite the winner; it is deleted only after reload proves it is unreferenced, otherwise it is preserved. |
| Cleanup-warning persistence fails and restoration succeeds | Reload proves original object plus adjustment are authoritative; only then may replacement object be deleted. |
| Original-snapshot restoration fails | Replacement and original objects are retained because metadata may reference either; redacted 500 and manual-cleanup evidence are recorded. |
| Concurrent newer replacement wins before rollback CAS | Rollback cannot overwrite the winner and cannot delete the prior replacement candidate or newer active object. |
| Old draft/assist/mutation 409 arrives after a newer review opens | Origin authority check fails; no teardown, message, item refresh, or state mutation occurs. |
| Current authoritative 409 occurs | Review invalidates once, fresh item is loaded/reconciled under a recovery token, stale revision is replaced, and reopening succeeds immediately. |
| Conflict recovery fetch resolves after navigation or newer review | Recovery generation/context check rejects the result; current item/review is not overwritten. |
| Replacement/rollback returns Conflict, NotFound, Repository failure, verification mismatch, or delete failure | Exact redacted mappings are respectively stable media-conflict 409, image-not-found 404, and recovery-required 500 for the last three. Recovery-required responses include a UUID correlated to structured private diagnostics without raw keys in HTTP. Fully verified restoration plus successful cleanup returns the distinct redacted `imageReplacementFailed` 500 without recovery ID. |
| Source preview returns 409 | Status-bearing fetch-to-blob path checks full source request/session/media authority before recovery; stale response is inert, current response refreshes item. Direct image load/error never has to infer HTTP status. |
| Conflict refresh returns 404 | Current item and review actions remain cleared; collection view shows non-retryable missing-item guidance. |
| Conflict refresh returns 500/network error | No stale item/review controls are actionable; only a generation-bound item refresh retry is exposed. |
| Conflict refresh returns 401/403 | Recovery state invalidates and existing logout/session-expired behavior runs with no retry or stale item. |
| Recovery retry succeeds or resolves late | Current generation reconciles fresh item and permits reopen; older retry/navigation results are inert. |
| Old-object delete applies but returns error | No rollback occurs; replacement metadata and readable replacement object remain authoritative, and cleanup warning/recovery evidence records the ambiguous result. |
| Old-object delete errors without applying | No rollback occurs; replacement remains authoritative and the old object is retained as a cleanup candidate. |
| Cleanup-warning persistence also fails after ambiguous delete | Preserve replacement authority and every possibly existing object; return redacted `imageRecoveryRequired` with structured private evidence. |
| User types, saves, invokes image action, or direct render occurs during recovery | Controls/handlers are inert or the action explicitly invalidates recovery and establishes a newer editor generation before proceeding; pending recovery cannot overwrite it. |
| Recovery response arrives after same-view editor generation changes | Acceptance fails despite identical route/view/item ID; response is discarded with no form/item mutation. |

## Revised Plan Requirements

The historical addenda remain approved evidence, but the latest source review exposed ambiguous external delete outcome and same-view recovery ownership gaps. The plan now contains an `Ambiguous Delete and Exclusive Recovery Addendum`. No coder work may resume until an independent reviewer approves replacement-authoritative ambiguous-delete handling, inert recovery UI, editor-generation invalidation, the production-safe live validation boundary, and the expanded failure matrix with zero findings; record that artifact/comment in frontmatter first.

## Resume Criteria

1. This reassessment and its revised assumptions are committed on the PR branch.
2. An independent plan reviewer verifies the mounted-render generation contract, resize projection lifecycle, mutation/egress serialization, and browser-faithful regression matrix.
3. Any plan-review findings are incorporated and the plan is re-reviewed until approved.
4. The final plan-review evidence path and assumption-revision evidence path are recorded in this file's frontmatter.
5. Only then may `status` change from `reassessment_required` to `ready_to_resume` and one coherent coder pass begin.
6. A subsequent deep source review must preserve Round 1–3 lineage and return zero actionable findings before the loop closes.
