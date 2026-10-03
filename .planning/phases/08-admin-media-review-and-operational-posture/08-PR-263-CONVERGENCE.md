---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: ready_to_resume
trigger_review: 08-REVIEW.md
assumption_revision_evidence: 08-PR-263-ROUND3-REVISED-PLAN.md#contract-decisions
implementation_plan_review_evidence: 08-PR-263-ROUND3-PLAN-REVIEW.md
implementation_plan_review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5963986246
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

## Shared Invariants

### 1. Mounted-output authority

Only the currently mounted adjusted-output image may transition preview status or authorize Save. Authority is the conjunction of current review session, current item/image, current draft revision, current preview request/blob identity, current output-render generation, and current DOM node identity. Equal draft content does not make two render instances interchangeable.

### 2. Normalized geometry authority

Perspective corner values remain normalized source coordinates and are the only mutable geometry model. Every intrinsic-size or rendered-frame-size change must reproject that model into visual handle positions. Visual hit-target clamping may move a button center inward, but must never alter the normalized corner value.

### 3. Mutation/navigation serialization

Once Save or Reset crosses the persistence boundary, the review cannot be discarded or left until the mutation settles and its result is reconciled into the current item, baseline, messages, and publish-boundary state. UI disabling is presentation; every egress handler must also enforce the pending-mutation invariant.

### 4. Test-environment fidelity

User-event helpers must obey browser disabled-control behavior. Programmatic dispatch, when intentionally needed, must be explicit. The harness must control image events, resize observation, deferred fetches, detached nodes, and navigation attempts so each authority boundary is executable and asserted.

## Complete Consumer and Action Inventory

| Invariant | Producers | Consumers and reporting paths | Mutation / invalidation boundaries | Required tests |
|---|---|---|---|---|
| Mounted-output authority | Draft preview fetch, blob URL, `renderImageReview`, adjusted `<img>` creation | Output stage, loading/error text, Save enabled state, displayed revision | Review open/close, item or image switch, assist/manual/reset draft, overlay/comparison rerender, retry, blob replacement/revocation | Detached old image load/error after same-revision rerender; older request completion; session/item switch; retry |
| Normalized geometry authority | Intrinsic image dimensions, normalized corners, fitted source bounds, frame observer | Handle positions, labels, pointer and keyboard updates, preview request payload | Image load, responsive resize, breakpoint/layout change, manual/assist corner change, review teardown | Portrait and landscape resize; all four full-frame corners; 44px hit targets; pointer/keyboard parity |
| Mutation/navigation serialization | Save PATCH, Reset DELETE, mutation-pending state and token | Back, tabs, item selection, logout, messages, current item, baseline, dirty state, publish controls | Mutation submit, success/failure settlement, session teardown | Save→Back, Reset→Back, tab/item/logout during pending mutation, success/failure reconciliation |
| Test fidelity | Fake elements, user-event helper, timers, deferred fetch queue, fake image events, fake resize observer | All DOM behavior assertions | Disabled/enabled transition, node detach, render replacement, resize delivery, deferred mutation completion | Browser-impossible events rejected; explicit programmatic events supported; all three open behavioral findings reproduced before fix and closed after fix |

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

## Revised Plan Requirements

The coherent implementation is specified in `08-PR-263-ROUND3-REVISED-PLAN.md`. It must close all four Round 3 findings as one lifecycle contract change rather than separate point fixes. `08-PR-263-ROUND3-PLAN-REVIEW.md` independently approved the plan with zero blockers, warnings, or advisories; the approval is also preserved in the PR comment referenced in frontmatter.

## Resume Criteria

1. This reassessment and its revised assumptions are committed on the PR branch.
2. An independent plan reviewer verifies the mounted-render generation contract, resize projection lifecycle, mutation/egress serialization, and browser-faithful regression matrix.
3. Any plan-review findings are incorporated and the plan is re-reviewed until approved.
4. The final plan-review evidence path and assumption-revision evidence path are recorded in this file's frontmatter.
5. Only then may `status` change from `reassessment_required` to `ready_to_resume` and one coherent coder pass begin.
6. A subsequent deep source review must preserve Round 1–3 lineage and return zero actionable findings before the loop closes.
