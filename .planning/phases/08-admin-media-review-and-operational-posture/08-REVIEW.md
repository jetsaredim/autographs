---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-04T18:51:44Z
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
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 08: Code Review Report — Pointer Lifecycle Follow-up

**Reviewed:** 2026-10-04T18:51:44Z
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

Commit `7c11380` keeps the captured handle mounted across ordinary pointer movements, rejects retired-node user delivery, settles one preview on pointer up/cancel/lost capture, and preserves the previously closed mutation, publisher, migration, privacy, and public-output contracts. The interaction is not yet authoritative end to end, however. Asynchronous producers that were already active before `pointerdown` can still replace the captured node, and the inset used to keep a 44px edge target visible is not accounted for when converting pointer movement back to normalized source coordinates.

### Finding lineage

| Prior finding | Current disposition | Evidence |
| --- | --- | --- |
| Post-convergence WR-01: first move replaces the pointer-capture owner | Incomplete fix / sibling-path miss | Ordinary moves now retain the same handle, but source-image load and already-running preview/assist work can still rerender or replace it after `pointerdown` (WR-01). |
| Round 3 WR-01: normalized geometry and responsive projection | Incomplete fix | Resize projection and full-size edge targets remain correct, but dragging an inset edge target maps its visual center as a new source coordinate, so the model jumps before tracking the intended delta (WR-02). |
| Round 3 WR-03: browser-faithful DOM coverage | Incomplete fix / test weakness | Disconnected-node delivery is rejected and terminal events are exercised, but the harness starts only after source load, resolves no competing async producer during capture, and moves edge handles directly to the hidden true corner rather than checking a zero/small movement from the visible target (WR-01, WR-02). |
| Round 3 CR-01: mounted adjusted-output authority | Resolved | Request, render generation, exact node, preview URL/revision, draft revision, item/image, and session still gate output callbacks and Save authorization. |
| Round 3 WR-02: pending mutation egress | Resolved | Save/Reset remain serialized with review egress through success and failure reconciliation. |

### Inherited closed-contract check

| Contract | Verdict |
| --- | --- |
| Mounted adjusted-output authority and blob ownership | Closed for output/Save correctness; WR-01 concerns interruption of an active pointer gesture, not stale output authorization. |
| Save/Reset repository reconciliation and navigation serialization | Closed for success, failure, and named egress paths. |
| Publisher promotion and post-promotion cleanup | Closed; full publisher coverage preserves the active release and comparison map. |
| Legacy active-release migration | Closed; the active legacy artifact remains protected until successful comparison metadata exists. |
| Private-original and public-output boundaries | Closed; authenticated previews remain private/no-store and public privacy contracts pass. |

### Warnings

#### WR-01: Pre-existing async producers can still detach the active pointer-capture owner

**Classification:** WARNING
**File:** `controller/static-admin/admin.js:1759-1763`
**Related:** `controller/static-admin/admin.js:1668-1694`, `controller/static-admin/admin.js:1819-1832`, `controller/static-admin/admin.js:2216-2235`, `controller/tests/static_admin_behavior.mjs:482-539`

**Issue:** The new drag record guards synchronous controls and keeps the handle mounted while `pointermove` itself mutates the model, but it does not serialize async work that began before the gesture. Handles are created before the source image loads; if the operator presses one and the image then emits `load`, the callback calls `renderPerspectiveHandles()`, removes the captured button, and creates four replacements without ending or transferring the drag. A draft-preview response or adjusted-output error that resolves after `pointerdown` but before the first move can call `renderImageReview()`, and an already-running edge-assist response can call `markReviewDraftChanged()` and do the same because their completion guards do not reject `state.reviewPerspectiveDrag`. Thus one physical gesture can lose its connected capture owner before its first movement or terminal event. The current harness always fires source load before pointerdown and does not resolve any preview, output error, or assist promise while capture is live, so it cannot catch these interleavings.

**Fix:** Make async render authority aware of the active gesture. Do not replace handles on source load (project the existing connected handles, or do not enable/create them until the source is ready), and either defer or reject/reschedule preview, output-error, and assist completions while `reviewPerspectiveDrag` owns the stage. Whichever policy is chosen must preserve the final normalized model and schedule exactly one authoritative preview at the terminal event. Add controlled tests for `pointerdown` followed by source load, assist completion, preview completion, and output error both before and after the first move; the same capture node must remain connected until pointerup/cancel/lost capture or an explicit lifecycle teardown.

#### WR-02: Dragging a visually inset edge handle jumps the normalized corner

**Classification:** WARNING
**File:** `controller/static-admin/admin.js:2042-2082`
**Related:** `controller/static-admin/admin.js:2156-2168`, `controller/tests/static_admin_behavior.mjs:502-524`

**Issue:** Edge corners retain their true normalized values while `positionPerspectiveHandle()` clamps each 44px button center 22px inside the frame. The move handler nevertheless converts the pointer's absolute client position directly into a source coordinate. For a full-frame landscape corner, the model is `x = 0` while the visible handle center is at `x = 22px`; the first one-pixel movement from that center writes approximately `23 / frameWidth` instead of a small delta from zero. The corner therefore jumps inward solely because its accessible hit target was inset. The test masks this by moving the pointer from the visible inset center to the invisible mathematical edge (`clientY: 0`) and asserting zero, rather than asserting that a stationary or one-pixel move from the visible center preserves/gradually changes the normalized value.

**Fix:** Record the grab relationship at pointerdown and apply movement as a delta from the starting normalized corner (or subtract a maintained grab offset from the current pointer-to-source projection), including when frame geometry changes during capture. A pointermove at the same client coordinates as pointerdown must leave the corner unchanged, and a one-pixel move must change it by one source pixel's normalized fraction without a target-radius jump. Extend the harness across all four full-frame corners and portrait/landscape frames, including resize during capture.

## Verification

- `git diff --check origin/main...HEAD` — passed.
- `node --check controller/static-admin/admin.js` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed, but omits the async-capture and inset-grab interleavings described above.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets` — passed; 163 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.

---

_Reviewed: 2026-10-04T18:51:44Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
