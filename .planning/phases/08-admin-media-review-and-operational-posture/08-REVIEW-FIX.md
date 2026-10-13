---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-12T23:44:04Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 1
findings_in_scope: 1
finding_families: 1
fixed: 1
skipped: 0
reassessment_required: false
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-12T23:44:04Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Finding families: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: Sparse weak outer boundaries can still leave an internal region `Confident`

**Finding family:** `auto_assist_outer_boundary_confidence`
**Lineage:** `incomplete_fix`
**Shared invariant:** A `Confident` proposal must remain the same outer item boundary under weaker contrast evidence; material quadrilateral expansion alone invalidates confidence even when component population grows by less than 25%.
**Files modified:** `controller/src/image_adjustments.rs`, `controller/tests/image_adjustments.rs`
**Commit:** `85db903`
**Applied fix:** Changed multi-threshold boundary stability so either excessive component population growth or material quadrilateral-area growth forces safe abstention. Added a non-private 180x140 sparse weak-enclosure fixture with a one-pixel outer rectangle and bridge around dense dark internal content. The regression failed as `Confident` before the production change and now returns `Unavailable` with no corners.

## Boundary Inventory

- **Producer:** `propose_image_adjustment` selects the largest adaptive-threshold component, validates its quadrilateral and area, then delegates confidence to `component_boundary_is_stable`.
- **Threshold sequence:** Relative and fixed probes remain strictly descending through `previous_threshold`; fixed low probes still expose weak enclosures hidden by an inflated adaptive threshold.
- **Anchor continuity:** The original row-major foreground anchor remains the trace seed at every weaker threshold, so the comparison follows mergers of the same selected component rather than selecting an unrelated component.
- **Comparisons:** Population growth and quadrilateral-area growth are now independent instability signals. The prior conjunction was the incomplete fix.
- **Route consumer:** `assist_image_adjustment` serializes the proposal without manufacturing corners and rechecks the media revision after detection.
- **UI consumer:** `detectImageEdges` mutates the draft only for `status === "confident"` with exactly four corners; `unavailable`, `notNeeded`, unexpected, error, and stale results remain non-mutating.
- **Tests:** Existing high-contrast skew, dark-card, reliable outer border, filled enclosure, noisy-border enclosure, low-confidence, near-identity, route, and browser behavior cases remain green alongside the new sparse-enclosure regression.

## Verification

Verification ran in the main checkout on branch `fix/image-review-ui-state` because `.planning/config.json` sets `workflow.use_worktrees` to `false`.

- Pre-fix regression: `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content -- --nocapture` — failed as expected: `Confident` vs `Unavailable`.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture` — passed, 15 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow admin_image_adjustment_routes_save_reset_review_and_assist -- --nocapture` — passed, 1 focused route test.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `node --check controller/static-admin/admin.js` — passed.
- `git diff --check -- controller/src/image_adjustments.rs controller/tests/image_adjustments.rs` — passed.

---

_Fixed: 2026-10-12T23:44:04Z_
_Fixer: Codex (gsd-code-fixer)_
_Iteration: 1_
