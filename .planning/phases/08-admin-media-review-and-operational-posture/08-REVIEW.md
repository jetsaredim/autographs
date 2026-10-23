---
phase: 08-admin-media-review-and-operational-posture
pr: 281
reviewed: 2026-10-23T19:16:16Z
reviewed_head: 14bb74dbb66e9e0a23a046fa14a3a2e61f708a94
depth: deep
review_round: 1
files_reviewed: 12
files_reviewed_list:
  - .planning/spikes/011-local-image-workbench/workbench/src/main.rs
  - .planning/spikes/011-local-image-workbench/workbench/static/app.css
  - .planning/spikes/011-local-image-workbench/workbench/static/app.js
  - .planning/spikes/011-local-image-workbench/workbench/static/index.html
  - controller/image-processing/src/image_adjustments.rs
  - controller/tests/image_adjustments.rs
  - controller/image-processing/src/derivatives.rs
  - controller/image-processing/src/lib.rs
  - controller/src/lib.rs
  - controller/src/routes.rs
  - controller/src/catalog.rs
  - controller/src/oracle_catalog.rs
findings:
  critical: 1
  warning: 5
  info: 0
  total: 6
status: issues_found
convergence:
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: null
  plan_evidence: null
---

# Phase 08: Local Image Workbench Code Review

**Reviewed:** 2026-10-23T19:16:16Z
**Reviewed commit:** `14bb74dbb66e9e0a23a046fa14a3a2e61f708a94`
**Depth:** deep
**Status:** issues_found

## Summary

The shared transform and detector tests pass, and the workbench keeps ordinary
rendered previews and fixture writes outside the source directory. The deep
review nevertheless found one security blocker and five independent warning
families. The loopback listener accepts arbitrary HTTP Host values while serving
raw private originals and mutable fixtures. On the correctness side, async UI
operations have no selected-image ownership token; framing apply/undo does not
round-trip existing transforms and manual measurement state; quadrilateral
validation accepts crossed, concave, and wrongly ordered corners; three-sided
gradient evidence can be promoted to a reliable crop-driving boundary; and
persisted fixtures are associated with transient sorted indexes rather than
stable source identities.

This is review round 1 for the newly pushed workbench-finalization scope. Every
Critical/Warning is therefore classified as `independent`; the third-review
convergence checkpoint is not active.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: The loopback server exposes private originals to arbitrary Host origins

**File:** `.planning/spikes/011-local-image-workbench/workbench/src/main.rs:108`
**Finding family:** `loopback_browser_origin_isolation`
**Lineage:** `independent`
**Shared invariant:** A browser-facing tool that serves private originals and mutates external fixtures must accept requests only from the intended per-process loopback origin.
**Consumers inspected:** Axum router construction; index, asset, image-list, raw-source, render, detector, manual-framing, fixture-load, and fixture-save routes; browser fetch call sites; source response cache headers.
**Evidence level:** `live_executed`
**Issue:** Binding the socket to `127.0.0.1` prevents direct remote TCP access, but the router applies no Host, Origin, or per-process capability check. A live request with `Host: attacker.example:8788` returned `200` from both `/api/images` and `/api/images/0/source`, including the raw private image bytes. This leaves the classic DNS-rebinding boundary open: a hostile browser origin that resolves its own host to loopback can read the same-origin API and can invoke the fixture PUT route. The raw source response at line 177 also omits `Cache-Control: no-store`, unlike the shell and rendered preview responses.
**Fix:** Add an outer middleware that rejects every Host except the configured loopback authority and rejects state-changing requests whose Origin is not that same authority. Prefer a random per-process capability embedded in the printed launch URL and required by every API call, so Host handling is not the only browser boundary. Return `Cache-Control: no-store` on raw sources and fixture/list responses. Add router tests proving an attacker Host and mismatched Origin cannot list/read images or write fixtures while the printed loopback origin can.

## Warnings

### WR-01: Stale async work can overwrite a newly selected image's draft

**File:** `.planning/spikes/011-local-image-workbench/workbench/static/app.js:119`
**Finding family:** `workbench_async_authority`
**Lineage:** `independent`
**Shared invariant:** Only an operation started for the current image and current draft revision may publish preview state, detector results, fixture state, or alignment mutations.
**Consumers inspected:** `selectImage`, `loadFixture`, delayed `scheduleRender`, `renderPreview`, `detect`, `saveFixture`, `assessManualFraming`, `applySuggestedAlignment`, all image-navigation handlers, and the server's id-scoped endpoints.
**Evidence level:** `source_only`
**Issue:** These async functions read and later mutate shared globals such as `selectedId` and `adjustment` without capturing an operation owner. For example, selecting A starts `loadFixture`, selecting B changes `selectedId`, and a late A response can assign A's fixture to B's global draft before `syncAll`. A late detector response has the same problem. `saveFixture` sends to the old URL but, after awaiting, marks and rerenders whichever `selectedId` is current. Auto-alignment leaves navigation and draft controls active and can restore its `previousAdjustment` over an edit or selection made while a render is pending. `renderGeneration` does not fully close this gap because it increments only when the delayed replacement render actually begins, not synchronously when the draft changes.
**Fix:** Introduce a monotonically increasing selection/draft operation generation. Capture `{imageId, generation, adjustmentSignature}` before every await and commit results only if all still match. Increment/invalidate it synchronously in `selectImage` and `draftChanged`, abort superseded fetches, and cancel auto-alignment on any navigation or edit (or disable those controls for the operation). Make `loadFixture` return a value rather than mutating globals. Add deterministic deferred-fetch tests that complete A after B for load, detect, save, render, and auto-align paths.

### WR-02: Framing apply and undo do not round-trip the previewed state

**File:** `.planning/spikes/011-local-image-workbench/workbench/static/app.js:883`
**Finding family:** `framing_transform_round_trip`
**Lineage:** `independent`
**Shared invariant:** Applying a displayed crop must render exactly the displayed rectangle, and Undo must restore every draft-local geometry input that Apply changed.
**Consumers inspected:** server transform order (`perspective -> rotation -> crop -> zoom/pan`); `framingAssessment`; `syncFramingActions`; `suggestedFramingCrop`; automatic and confirmed-manual support; `applySuggestedFraming`; `undoSuggestedFraming`; manual-boundary remapping and reassessment.
**Evidence level:** `source_only`
**Issue:** The proposal rectangle is normalized in the current final preview, but line 889 writes it directly into `adjustment.crop`, whose coordinates are evaluated before the existing crop/zoom/pan. When an existing crop, zoom, or pan still leaves the requested band available, `framingAssessment` returns `Adjustable` before checking `limitedByExistingCrop`, so Apply is enabled; it then overwrites the old crop and resets zoom/pan. The rerender therefore need not match the cyan preview rectangle. On the manual path, Apply also remaps `measurementCorners`, but Undo restores only `adjustment`, leaving those handles in cropped coordinates while claiming the prior draft was restored.
**Fix:** Either suppress Apply whenever `limitedByExistingCrop` is true until the operator resets those controls, or explicitly compose the final-output rectangle through the existing crop/zoom/pan into the transform's crop coordinate space. Store one undo snapshot containing the adjustment, measurement corners, touched set, origin, and confirmation state, and restore it atomically. Add behavioral tests for an existing crop, zoom/pan, a confirmed manual boundary, Apply, rerender equivalence with the overlay, and Undo equality with the complete pre-Apply state.

### WR-03: Perspective and manual-framing validators accept invalid quadrilateral topology

**File:** `controller/image-processing/src/image_adjustments.rs:211`
**Finding family:** `ordered_convex_quadrilateral_validation`
**Lineage:** `independent`
**Shared invariant:** Every four-corner transform or framing boundary must be a normalized, non-self-intersecting, convex TL/TR/BR/BL quadrilateral with meaningful area and edge lengths.
**Consumers inspected:** source and measurement drag producers; JSON apply; `ImagePerspective::validate`; `apply_perspective`; workbench render/save endpoints; production draft-preview/save routes; in-memory and Oracle persistence; manual-framing endpoint; framing analysis and crop actions; validation tests.
**Evidence level:** `deterministic_test`
**Issue:** `ImagePerspective::validate` bounds each scalar and then delegates only to `Projection::from_control_points`; that algebraic solve is not a topology validator. A focused executable against the reviewed crate returned `Ok(())` for a bow-tie, a concave quad, and the four valid corners in the wrong winding. The workbench UI can create these shapes by dragging handles across each other, and the production preview/save routes trust the same validator before persistence. The manual-framing endpoint is weaker still: it checks only four finite points, not normalization, ordering, convexity, or area, before using their bounding box to authorize framing actions.
**Fix:** Add one shared ordered-convex-quad validator that checks finite normalized coordinates, minimum edge/area thresholds, consistent winding, and non-intersection before creating a projection or measuring framing. Reuse it for `ImagePerspective` and manual boundaries. Add unit and endpoint cases for bow-tie, concave, reversed/wrong order, near-zero-area, out-of-range, and a valid skewed quad.

### WR-04: Three-sided gradient evidence is promoted to a reliable crop-driving boundary

**File:** `controller/image-processing/src/image_adjustments.rs:925`
**Finding family:** `boundary_confidence_preservation`
**Lineage:** `independent`
**Shared invariant:** Evidence strong enough only to veto automatic correction must not be promoted to a reliable boundary that can authorize automatic alignment or framing mutations.
**Consumers inspected:** component and gradient boundary producers; `propose_image_adjustment`; `detect_image_boundary`; refinement fallback; render response headers and framing support; `fittedEdgesAreStable`; reliability gating; automatic alignment and framing actions; boundary tests.
**Evidence level:** `source_only`
**Issue:** `strongest_gradient_boundary` returns a candidate when only three of four sides pass localized support/contrast/coherence. That threshold is conservative when the result merely sets `larger_gradient_corners` and causes auto-correction to abstain. The same result is also returned by `locate_gradient_image_boundary`, however, and both consumers in `detect_image_boundary` label it `Reliable`. The workbench then treats reliable automatic support as eligible for suggestions. A missing refinement yields `edges: []`, which bypasses the unstable-four-edge warning; a straight but low-contrast/incoherent fourth side can also pass `fittedEdgesAreStable`, which checks only coverage and straightness. Thus the weaker veto evidence can drive a crop.
**Fix:** Preserve the side-evidence count/confidence in the gradient result. Require all four localized sides for `Reliable`; classify a three-sided candidate as `Tentative` (or unavailable) and ensure it cannot drive alignment/framing actions. Keep the three-side threshold only for the conservative auto-correction veto if desired. Add a deterministic three-valid-side fixture and assert that it still vetoes correction but never produces crop/actionable reliability.

### WR-05: Fixture files are attached to transient sort indexes instead of source identity

**File:** `.planning/spikes/011-local-image-workbench/workbench/src/main.rs:585`
**Finding family:** `stable_fixture_source_identity`
**Lineage:** `independent`
**Shared invariant:** A persisted adjustment fixture must reload only for the same source image it was saved against across workbench restarts and directory changes.
**Consumers inspected:** image discovery and sorting; image summaries; fixture load/save paths; browser selection and fixture markers; query-index behavior; discovery test.
**Evidence level:** `source_only`
**Issue:** `discover_images` sorts paths and names fixtures solely as `0000-adjustment.json`, `0001-adjustment.json`, and so on. Adding, removing, or renaming an earlier-sorting file changes every later id, so an existing valid fixture is silently loaded for a different private original and the UI reports that wrong image as having a fixture. The current test proves deterministic ordering for one scan but does not exercise rescan identity.
**Fix:** Derive a stable, collision-resistant fixture key from source identity (for example a manifest entry containing normalized relative path plus a source fingerprint) and verify that identity before load. Keep transient numeric ids only for HTTP routing if useful. Add a restart test that saves fixtures, inserts/removes an earlier-sorting image, rescans, and proves every fixture remains attached to its original or fails closed on an identity mismatch.

## Verification

- `cargo test --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml` — passed: 8 tests.
- `cargo test --manifest-path controller/image-processing/Cargo.toml` — passed: 1 unit test plus doc tests.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments` — passed: 22 tests.
- `node --check .planning/spikes/011-local-image-workbench/workbench/static/app.js` — passed.
- Focused executable linked against the reviewed image-processing crate — bow-tie, concave, and wrong-winding perspective cases all returned `Ok(())`, confirming WR-03.
- Live loopback probe — requests with `Host: attacker.example:8788` returned `200` for the image list and raw source bytes, confirming CR-01's missing authority check.
- `git status --short` remained clean after review diagnostics; no source file was modified.

---

_Reviewed: 2026-10-23T19:16:16Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep_
