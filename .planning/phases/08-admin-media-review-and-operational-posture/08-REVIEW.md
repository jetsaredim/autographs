---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-02T12:14:37Z
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
  critical: 6
  warning: 2
  info: 0
  total: 8
status: issues_found
---

# Phase 08: Code Review Report — Round 2

**Reviewed:** 2026-10-02T12:14:37Z
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

The server now renders draft previews with the publishing derivative function, the image map is outside Caddy's `current` document root, and memory/Oracle adjustment writes share canonical no-op handling. Eight actionable issues remain. Several are request-order failures that the new DOM harness cannot exercise; a release cleanup failure can remove the active release after promotion.

### Round 1 lineage

| Round 1 finding | Round 2 disposition | Evidence |
| --- | --- | --- |
| CR-01 transform parity | Incomplete fix and sibling-path miss | Draft bytes use the publishing renderer, but Save can accept an edit before its preview is shown (CR-01), and source-coordinate handles are drawn over transformed, letterboxed output (CR-04). |
| CR-02 public-current comparison | Incomplete fix and sibling-path miss | New releases have a private map, but promotion cleanup can break the active release (CR-03); releases made before this change have no map (WR-01). |
| CR-03 dirty/navigation/publish guards | Fix regression | Synchronous guards exist, but an in-flight Save can silently mark an unsent draft clean (CR-02), and older review/assist/reset responses can overwrite a newer session (CR-05). |
| WR-01 canonical no-op adjustment | Resolved for the reviewed route and both repositories | `into_canonical` removes exact full-frame corners and identity; the memory and Oracle mutation boundaries skip unchanged values. |
| WR-02 repository failures as 404 | Resolved | `load_admin_image` now distinguishes malformed, missing, and repository-error results. |
| WR-03 perspective keyboard/pointer interaction | Incomplete fix | Focus and pointer handlers exist, but they use the 4:3 frame rather than the fitted source coordinates (CR-04). |
| WR-04 preview retry | Resolved for the direct failure path | Tile and focused failures expose retry actions. |
| WR-05 narrow layout | Resolved at the CSS breakpoint | The review layout changes to one column below 760px. |
| WR-06 comparison selected state | Resolved | Buttons synchronize `aria-pressed`, active styling, and disabled capability. |

The shared review invariant spans the API preview producer, the active-release map, the browser draft and comparison consumers, save/reset/publish mutations, navigation/session actions, and tests. The current test suite checks synchronous helpers and route bytes but does not cover the outstanding asynchronous interleavings or release failure matrix (WR-02).

### Critical Issues

#### CR-01: Save remains enabled while the displayed preview is stale or failed

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1467-1475`
**Related:** `controller/static-admin/admin.js:1425-1433`, `controller/static-admin/admin.js:1555-1585`, `controller/static-admin/admin.js:1792-1802`

**Issue:** An input change queues a draft render for 150 ms and sets `reviewPreviewStatus` to `refreshing`, but retains `reviewPreviewUrl`. `renderImageReview` displays that old URL whenever it exists, and `syncReviewDirtyState` enables Save solely from metadata inequality. A user can move a control and immediately save an adjustment the UI has never rendered. If the new request fails, Save also remains enabled behind the retry state. This breaks the original CR-01 pixel-review invariant even though the server's draft renderer is correct.

**Fix:** Track the draft revision represented by the displayed blob. Mark the stage pending on every edit; enable Save only after the latest revision renders successfully. Keep Save disabled on draft-preview failure and show the pending/error state without presenting old pixels as current.

#### CR-02: An in-flight Save marks later, unsent edits as saved

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1797-1808`
**Related:** `controller/static-admin/admin.js:2362-2370`, `controller/static-admin/admin.js:1827-1842`

**Issue:** `jsonRequest` serializes adjustment A when Save is clicked, then awaits the PATCH. Controls remain editable. If the operator changes the draft to B before the response arrives, the success path copies the *live* B into `reviewSavedAdjustment`, clears `reviewDirty`, and reports success although the repository saved A. Navigation and publishing now proceed while the visible state and persisted derivative disagree. Reset has the same unguarded response timing and can overwrite edits made while DELETE is pending.

**Fix:** Snapshot the submitted adjustment and review-session revision before the request. On success, set the saved baseline from the returned image (or the submitted snapshot); retain any later draft and recompute dirty state. Disable conflicting mutation controls while Save/Reset is pending, and ignore responses for a session that has since closed or switched images.

#### CR-03: Post-promotion cleanup can remove the active static release

**Classification:** BLOCKER
**File:** `controller/src/publisher.rs:829-831`
**Related:** `controller/src/publisher.rs:863-877`, `controller/src/publisher.rs:1965-1970`, `controller/src/publisher.rs:1934-1947`

**Issue:** `promote_candidate` switches `current` to the new release before `prune_promoted_releases` runs. Pruning can fail while removing an old release *or its newly introduced admin image map*. That error enters the generic failure branch, where `retain_failed_candidates` renames the new release directory into `failed`, leaving `current` dangling; `remove_admin_image_map` then deletes its comparison map. Caddy serves `/srv/autographs/static/current`, so a cleanup failure after a successful promotion can take the public catalog offline. The original generic failure path existed, but the new map cleanup introduces another direct trigger and still treats the active release as a candidate.

**Fix:** Split pre-promotion failure recovery from post-promotion housekeeping. Once the pointer has switched, never move or delete the active release or its map because pruning failed. Report cleanup failure separately and retry pruning later. Add a failure-injection test for old-release/map deletion that asserts `current` remains readable and its map remains available.

#### CR-04: Perspective handles use frame coordinates instead of source-image coordinates

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1645-1655`
**Related:** `controller/static-admin/admin.js:1692-1719`, `controller/static-admin/admin.css:725-735`, `controller/src/image_adjustments.rs:278-298`

**Issue:** The server interprets perspective corners as normalized coordinates of the *private original before transformation*. The browser draws and drags them across a fixed 4:3 `.review-frame`, while `<img>` uses `object-fit: contain` and can occupy only part of that frame. For a portrait image, the full-frame handles sit in the side matte; pointer positions are divided by the full frame width, so a handle placed on a visible source corner sends a different source coordinate. With a saved rotation/crop/perspective, the preview is already transformed, so its visible corners cannot serve as the original-coordinate guide either. The auto-assist proposal uses original coordinates and is likewise overlaid in the wrong place.

**Fix:** Provide a sanitized unadjusted source guide for perspective editing, fit the handles to that image's actual rendered bounds, and convert pointer coordinates relative to those bounds. Keep the canonical adjusted derivative as a separate output preview. Test portrait and non-identity saved adjustments with pointer coordinates and resulting pixels.

#### CR-05: Older async review actions can overwrite a newer item or review session

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1524-1541`
**Related:** `controller/static-admin/admin.js:1768-1779`, `controller/static-admin/admin.js:1827-1842`, `controller/static-admin/admin.js:2210-2229`

**Issue:** `openImageReview` does not verify that the item/image is still selected when its GET resolves. The operator can click Review for image A, switch tabs or items while it is pending, and the late response forces the UI back to A's review while `currentItem` refers to the newer item. Similarly, an assist response for A applies corners to whatever `state.reviewDraftAdjustment` exists when it resolves; after discarding A and opening B, it can silently apply A's corners to B. A late reset response can replace B's draft/current item as well. The navigation guard only considers an already-open dirty draft, so it cannot prevent these in-flight cases.

**Fix:** Give each review session an identity/revision token, increment it on item change or close, and check it after every awaited review/assist/save/reset response before mutating state. Cancel requests where practical. Add deferred-response DOM tests for A→B and review→navigation transitions.

#### CR-06: Clean reviews hide the only Discard and Reset actions

**Classification:** BLOCKER
**File:** `controller/static-admin/index.html:396-404`
**Related:** `controller/static-admin/admin.js:1425-1432`, `controller/static-admin/admin.js:1818-1842`, `controller/static-admin/admin.css:16-18`

**Issue:** The Save, Discard, and Reset buttons all live inside `image-review-dirty-band`, which `syncReviewDirtyState` hides whenever the draft matches the saved adjustment. On opening an image with a saved correction, the operator cannot invoke Reset until first changing an unrelated control. A clean review also has no in-view way to return to the item editor; it requires a top-level tab. This removes the requested reset/discard workflow in its normal starting state.

**Fix:** Keep Discard/Back and Reset visible in the focused review; disable only Save when the draft is clean. Keep the dirty message conditional as a separate element. Add a DOM test that opens a saved adjustment and invokes Reset without first editing it.

### Warnings

#### WR-01: Existing active releases cannot provide public-current comparison

**Classification:** WARNING
**File:** `controller/src/publisher.rs:753-764`
**Related:** `controller/src/routes.rs:873-925`, `controller/src/publisher.rs:960-967`

**Issue:** The admin image map is created only by publishes with this new code. After deploying the controller over an existing active static release, `public_image_preview` returns `None` for every image because the old release has no map; the review route labels published images `privateOnly` and disables comparison. The release remains valid and public, but the new feature is unavailable until an operator performs a fresh publish.

**Fix:** Backfill the active release's private map from a verified repository-to-public-artifact association during rollout, or explicitly require and verify an initial publish before enabling the review UI. Do not report a known published image as private-only solely because migration metadata is absent.

#### WR-02: The DOM harness does not execute draft requests or user event paths

**Classification:** WARNING
**File:** `controller/tests/static_admin_behavior.mjs:139-174`
**Related:** `controller/tests/static_admin_behavior.mjs:185-284`, `controller/tests/static_admin.rs:691-705`

**Issue:** `window.setTimeout` only returns a number, so `scheduleDraftPreview` never calls `loadDraftPreview`; the global `fetch` always throws if invoked. Most assertions call internal functions directly, and the fake DOM never dispatches a real input, Save, tab, or reset event sequence. The harness therefore passes while CR-01, CR-02, and CR-05 remain possible, and its comparison assertion only checks assigned `src` strings rather than successful image loads.

**Fix:** Use a fake clock that flushes scheduled callbacks, a controllable fetch/response queue, and event dispatch through the registered UI listeners. Assert serialized request bodies, rendered blob revisions, dirty state after deferred responses, and navigation/publish actions.

---

_Reviewed: 2026-10-02T12:14:37Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
