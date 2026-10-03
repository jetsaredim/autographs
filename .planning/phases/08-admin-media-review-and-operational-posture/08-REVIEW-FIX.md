---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-02T01:46:12Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 1
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-02T01:46:12Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 9
- Fixed: 9
- Skipped: 0

## Fixed Issues

### CR-01: Focused review double-applies saved transforms and never previews perspective edits

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/routes.rs`, `controller/src/image_adjustments.rs`, `controller/static-admin/admin.js`, `controller/tests/admin_workflow.rs`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commits:** `a93ce9e`, `d8c82e7`
**Applied fix:** Added an authenticated, no-store draft-preview endpoint that renders the canonical draft through `generate_adjusted_derivative`. The browser debounces and cancels draft requests, displays the returned WebP without CSS geometry, and therefore uses the same perspective/rotation/crop/zoom/pan order as persistence and publishing. Integration coverage proves saved and draft endpoints return identical bytes for the same non-identity adjustment; the DOM harness proves the focused image has no second client transform.

### CR-02: Before/after and split comparison controls are placeholders

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/publisher.rs`, `controller/src/routes.rs`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/admin_workflow.rs`, `controller/tests/static_admin_behavior.mjs`
**Commits:** `a93ce9e`, `d8c82e7`
**Applied fix:** The publisher now records an admin-private item/image-to-detail-artifact map beside each promoted release. Review responses resolve the active release's actual public derivative, report published/unpublished/private-only state accurately, and expose comparison only when that artifact is available. Before/after and split modes now render distinct public-current and private-latest sources. Release-map cleanup follows failed and pruned releases.

### CR-03: Review drafts bypass dirty-state, navigation, and publish guards

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/static-admin/index.html`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `d8c82e7`
**Applied fix:** Added canonical `reviewDirty` derivation and one discard-confirmation path used by top-level navigation, item switching, image actions, signer management, and logout. Publish controls are disabled while either item or image drafts are dirty, the shared publish path refuses stale publication, session expiry preserves the in-page draft, and `beforeunload` guards page closure. Save/reset clear the flag only after successful server mutation.

### WR-01: Saving an untouched image creates a non-identity perspective adjustment

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/image_adjustments.rs`, `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, `controller/src/routes.rs`, `controller/static-admin/admin.js`, `controller/tests/admin_workflow.rs`, `controller/tests/static_admin_behavior.mjs`
**Commits:** `a93ce9e`, `d8c82e7`
**Applied fix:** Identity drafts now use `perspective: null`; full-frame corners canonicalize back to null on both client and server. Route and repository mutation boundaries skip unchanged canonical adjustments, so no-op saves do not touch timestamps, pending state, or history.

### WR-02: Repository failures are misreported as missing images

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/routes.rs`, `controller/tests/admin_workflow.rs`
**Commit:** `a93ce9e`
**Applied fix:** Replaced the lossy optional loader with an explicit malformed/not-found/repository-error result. Preview, review, draft preview, save/reset, and assist now distinguish 400, 404, and safely logged 500 outcomes. A failing repository double verifies 500 behavior across the three read callers named by the review.

### WR-03: Perspective handles lose focus after one keypress and have no pointer interaction

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `d8c82e7`
**Applied fix:** Keyboard movement updates the existing focused handle and restores its corner focus after async preview refresh. Pointer capture/drag updates normalized corners continuously. Each handle now has a clamped 44px hit target with a 24px visual marker and touch-safe dragging. The DOM harness exercises repeated arrow keys and pointer movement.

### WR-04: Preview failure states provide no retry action

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `d8c82e7`
**Applied fix:** Tile and focused failure states now render in-place `Retry preview` actions. Tile retries recreate the same-origin image with a request token; focused retries rerun the authenticated draft renderer while preserving the draft and surrounding review context. The DOM harness executes error-to-retry behavior.

### WR-05: Focused review layout does not collapse on mobile

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.css`, `controller/tests/static_admin.rs`
**Commit:** `d8c82e7`
**Applied fix:** The existing narrow breakpoint now collapses `.review-layout` to one column and reduces the stage minimum height to 18rem. A CSS regression test verifies both responsive declarations.

### WR-06: Comparison buttons expose no selected state to sighted or assistive-tech users

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `d8c82e7`
**Applied fix:** Comparison buttons synchronize exactly one `aria-pressed`/active state after every mode change. Before/after and split are disabled with `aria-disabled` when no public-current artifact exists. The DOM harness exercises both private-only and comparison-capable state.

## Lineage and Sibling-Path Classification

- **Incomplete-fix lineage:** CR-01 and CR-02 were incomplete original implementations, not regressions from an earlier repair round. They are resolved as one rendering/comparison contract rather than sibling point fixes.
- **Sibling-path misses repaired:** Direct memory and Oracle adjustment mutations now share canonical no-op behavior; dirty-draft handling covers tabs, status links, item changes, image actions, signer management, logout, session failure, page close, and both publish entry points.
- **Fix regressions:** None discovered.
- **Independent findings:** WR-02 and WR-05 were independent of the rendering authority but were fixed in the same round.
- **Test weaknesses repaired:** The prior source-string-only browser coverage is supplemented by an executable DOM harness for canonical dirty state, repeated keyboard movement, pointer drag, navigation confirmation, publish refusal, retry recovery, comparison capability, distinct comparison sources, and absence of a second CSS transform.

## Verification

All gates ran in the provided isolated checkout `/tmp/autographs-pr263-review` on branch `gsd/phase-08-admin-media-review-and-operational-posture`.

- `node --check controller/static-admin/admin.js` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml` — passed; 159 tests passed and 2 credential-gated live tests remained ignored by their existing contract.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-02T01:46:12Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
