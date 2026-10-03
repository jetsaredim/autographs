---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-03T01:05:09Z
depth: deep
files_reviewed: 11
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/static-admin/index.html
  - controller/tests/admin_workflow.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
findings:
  critical: 1
  warning: 3
  info: 0
  total: 4
status: issues_found
---

# Phase 08: Code Review Report — Round 3 Convergence Checkpoint

**Reviewed:** 2026-10-03T01:05:09Z
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found — reassessment required

## Narrative Findings (AI reviewer)

### Summary

Round 2 repaired the repository baseline snapshot, active-release protection, legacy-release migration state, source-coordinate conversion, clean-state actions, and most request-session ordering. Four actionable issues remain: a detached image element can authorize or invalidate a same-revision preview, perspective handle geometry becomes stale after responsive resizing and clips full-frame hit targets, an in-flight Save/Reset can continue invisibly after the operator confirms Back/discard, and the DOM harness models disabled controls unlike a browser while omitting those interleavings.

This is the mandatory third review. Because one Critical and three Warning findings remain, the repository convergence guard requires a durable reassessment and reviewed revised implementation plan before any further coder work. Do not start another point-fix round from this report alone.

### Round 2 finding lineage

| Round 2 finding | Round 3 disposition | Evidence |
| --- | --- | --- |
| CR-01 displayed-preview revision authority | Incomplete fix / sibling-path miss | Request and draft revisions are checked, but callbacks from detached image elements within the same revision are still authoritative (CR-01). |
| CR-02 in-flight Save baseline | Resolved for returned repository state; sibling action-path miss remains | The submitted snapshot becomes the saved baseline and later drafts stay dirty. Back/discard can still hide a mutation that is already in flight (WR-02). |
| CR-03 promotion cleanup safety | Resolved for the reviewed failure matrix | Pre-promotion recovery is separate from post-promotion pruning; map and release deletion failures leave `current` and its active map intact. |
| CR-04 perspective coordinate parity | Incomplete fix | Initial pointer conversion uses fitted source bounds, but pixel-positioned handles are not recomputed after layout changes and edge targets are clipped at narrow widths (WR-01). |
| CR-05 async review-session authority | Sibling-path miss | Item/image/session checks reject older requests, but they do not distinguish older DOM render instances for the same session and draft revision (CR-01). Back also invalidates the session while the underlying mutation continues (WR-02). |
| CR-06 clean-state action reachability | Resolved | Back and Reset remain visible outside the dirty-only band. |
| WR-01 legacy active-release migration | Resolved | A legacy release without a map reports `comparisonMigrationRequired` and preserves the active artifact until a successful publish. |
| WR-02 executable DOM coverage | Incomplete fix / test weakness | Timers, fetches, and registered listeners execute, but disabled elements still dispatch user input and same-revision render/resize/pending-Back cases are absent (WR-03). |

### Inherited Round 1 lineage

| Round 1 finding | Round 3 disposition |
| --- | --- |
| CR-01 transform/pixel-review parity | Incomplete through CR-01 and WR-01. The server renderer is shared, but displayed-render authority and responsive source geometry are not closed. |
| CR-02 public-current comparison | Resolved for generated maps, active-release safety, and explicit legacy migration state. |
| CR-03 dirty/navigation/publish guards | Incomplete through WR-02: a pending mutation can be hidden by the discard/navigation path. |
| WR-01 canonical no-op adjustment | Resolved in route, memory repository, and Oracle repository mutation boundaries. |
| WR-02 repository failures reported as 404 | Resolved. Repository failures are distinguished from missing images. |
| WR-03 perspective keyboard/pointer interaction | Incomplete through WR-01 after responsive geometry changes. |
| WR-04 preview retry | Resolved for tile and focused-preview failures. |
| WR-05 narrow layout | Layout column collapse is resolved; WR-01 is a new fix regression in handle reachability at that layout. |
| WR-06 comparison selected state | Resolved. |

### Shared-invariant inventory

| Invariant | Producers | Consumers / reporting paths | Mutation boundaries | Tests | Verdict |
| --- | --- | --- | --- | --- | --- |
| Displayed-preview revision authority | Draft-preview route, blob response, image load/error event | Adjusted output panel, Save enablement, retry state | Manual/assist/reset draft changes | Node DOM harness | Open: CR-01 |
| Async review-session authority | Review/session token, request responses | Review view, messages, editor item, navigation | Review, assist, Save, Reset, Back/logout | Deferred fetch harness | Open: CR-01, WR-02 |
| Promotion atomicity | Candidate/map writer, `promote_candidate`, post-promotion prune | `current`, active map, publish status/warning | Symlink switch, map/release deletion | Publisher failure injection | Closed for active-release safety |
| Coordinate parity | Source-guide route and intrinsic dimensions | Handle overlay and pointer conversion | Keyboard/pointer/assist corner updates | Portrait DOM/API tests | Open: WR-01 |
| Focused action reachability | Review controls and dirty band | Back, Reset, Save, navigation | Pending Save/Reset | DOM harness | Open: WR-02 |
| Existing release migration | Active symlink/map lookup | Review state/message and comparison capability | First post-upgrade publish | Admin workflow integration test | Closed |
| Test fidelity | Fake DOM, clock, fetch queue | Registered events and state assertions | Disabled controls, render replacement, resize | `static_admin_behavior.mjs` | Open: WR-03 |

### Critical Issues

#### CR-01: Detached same-revision image callbacks can authorize Save or erase a newer render state

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1674-1700`
**Related:** `controller/static-admin/admin.js:1898-1904`, `controller/static-admin/admin.js:2547-2552`

**Issue:** The adjusted-image `load` and `error` handlers capture only the review session and `reviewDraftRevision`. Comparison and overlay changes call `renderImageReview()` without advancing either value. That replacement detaches the old `<img>` and creates a new one for the same blob. If the detached element later fires `load`, its callback marks the revision `ready` and enables Save even though the currently mounted image has not loaded. Conversely, its late `error` can replace a newer successful render with the failure state. The same gap exists whenever the view is rerendered without a draft revision change. The displayed-preview authority fix therefore still equates “same draft” with “current render instance.”

**Fix:** Give every adjusted-output render a monotonically increasing render token (and capture the current preview request revision/blob URL). In each `load`/`error` callback, require the session, draft revision, preview revision/URL, and render token to match the currently mounted output before changing `reviewPreviewStatus` or `reviewDisplayedRevision`. Invalidate the prior render token before replacing the stage. Add a deferred DOM test that retains the first image node, rerenders via comparison/overlay, and proves late events from the detached node cannot enable Save or overwrite the newer node's result.

### Warnings

#### WR-01: Perspective handle geometry is stale after resize and full-frame targets are clipped at narrow widths

**Classification:** WARNING
**File:** `controller/static-admin/admin.js:1817-1833`
**Related:** `controller/static-admin/admin.js:1872-1879`, `controller/static-admin/admin.css:740-762`, `controller/static-admin/admin.css:853-876`

**Issue:** Fitted source bounds are computed correctly when the source loads or a handle moves, but handle positions are stored as absolute pixels and no `ResizeObserver` or window resize path recomputes them. After the responsive layout changes size, the image refits while the four buttons remain at their old coordinates. For full-frame corners, positioning the 44px button center at exact `0` or frame width/height also places half the target outside the source frame; when the source frame fills the `review-stage`, the parent's `overflow: hidden` clips that target. This makes the guide inaccurate after resize and reduces the promised accessible hit area at common narrow layouts.

**Fix:** Observe source-frame size changes and reposition all handles from their normalized corner values after every resize/intrinsic-size change. Keep the coordinate value normalized to the real image edge while clamping only the visual button center far enough inward to preserve its 44px hit target (or provide an unclipped overlay gutter). Test a portrait source before and after changing the frame rectangle, plus all four full-frame corners at the narrow breakpoint.

#### WR-02: Back/discard can hide a Save or Reset that continues mutating the repository

**Classification:** WARNING
**File:** `controller/static-admin/admin.js:1996-2002`
**Related:** `controller/static-admin/admin.js:1445-1451`, `controller/static-admin/admin.js:1954-1993`, `controller/static-admin/admin.js:2005-2040`

**Issue:** Save/Reset disable most edit controls, but Back remains active. Confirming the “Discard unsaved image edits” prompt calls `clearImageReviewState()`, invalidates the session, and returns to the editor while the PATCH/DELETE continues. The completion is then deliberately ignored because its session is stale, so a successful persistent mutation receives no success message and `state.currentItem` stays stale. In the Reset case, a clean review can leave with no confirmation at all. The operator can continue working or publish while an invisible adjustment mutation completes; publish boundary tracking prevents direct release corruption, but the action semantics and on-screen repository state are no longer trustworthy.

**Fix:** Treat mutation pending as a navigation guard. Disable Back and all tab/logout/item navigation while Save/Reset is in flight, or await the mutation and surface its result before leaving. Do not present a discard confirmation for work that has already been submitted unless cancellation is guaranteed end to end. Add tests for both Save→Back and Reset→Back that verify no hidden mutation and no stale editor state.

#### WR-03: The DOM harness dispatches user input on disabled controls and misses the remaining browser interleavings

**Classification:** WARNING
**File:** `controller/tests/static_admin_behavior.mjs:55-61`
**Related:** `controller/tests/static_admin_behavior.mjs:274-303`, `controller/tests/static_admin_behavior.mjs:359-384`

**Issue:** `FakeElement.dispatch` invokes listeners even when the element is disabled. The in-flight Save test explicitly asserts the rotation control is disabled and then dispatches an `input` event on it, a path a browser user cannot take. The harness also treats leaving during a pending Reset as success without asserting what happened to the persistent mutation, and it never retains a detached same-revision image callback or changes source-frame geometry after load. It therefore does not provide the claimed browser-faithful coverage for CR-01, WR-01, or WR-02.

**Fix:** Make activation/input dispatch honor `disabled` for user-event simulations, with a separate explicit primitive only when a programmatic event is intended. Exercise a genuinely enabled concurrent path where required, add detached render-event ordering, add frame resize/reposition assertions, and assert the pending-mutation navigation contract and resulting repository/editor state.

## Mandatory convergence checkpoint

The third review has actionable Critical/Warning findings. Per the repository guard, the next artifact must reassess finding lineage, shared invariants, complete consumer/action inventory, assumptions, failure matrix, revised plan requirements, and explicit resume criteria. Coder work may resume only after those revised assumptions and the implementation plan are reviewed, with both evidence references recorded in the convergence artifact and its status changed from `reassessment_required` to `ready_to_resume`.

## Verification

- `git diff --check 70f6d63..e1c1f0a` — passed.
- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture` — passed; 4 tests.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` — passed; 18 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` — passed; 36 tests.

Passing tests do not cover the event/render and responsive-layout orderings above.

---

_Reviewed: 2026-10-03T01:05:09Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
