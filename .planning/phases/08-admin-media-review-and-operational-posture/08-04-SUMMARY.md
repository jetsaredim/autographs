---
phase: 08-admin-media-review-and-operational-posture
plan: "04"
subsystem: media
tags: [rust, image-adjustments, imageproc, derivatives, catalog, edit-history]

requires:
  - phase: 08-admin-media-review-and-operational-posture
    provides: Plan 08-03 established the CDN/cache contract before media adjustment work
provides:
  - Validated non-destructive image adjustment DTOs with canonical cache key serialization
  - Imageproc-backed transform helper and adjusted derivative wrapper that preserves existing derivative callers
  - Catalog adjustment field, repository update contract, memory save/reset behavior, and edit-history event
affects: [phase-08, media, catalog, derivatives, publisher, oracle-persistence, admin-media-review]

tech-stack:
  added: [imageproc-0.27.0]
  patterns:
    - Adjustment metadata is private catalog state and validates before persistence
    - Existing derivative generation remains a no-adjustment wrapper around the adjusted path
    - Replacement of a private original clears saved adjustment metadata while preserving the admin-facing image id

key-files:
  created:
    - controller/src/image_adjustments.rs
    - controller/tests/image_adjustments.rs
    - .planning/phases/08-admin-media-review-and-operational-posture/08-04-SUMMARY.md
  modified:
    - controller/Cargo.toml
    - controller/Cargo.lock
    - controller/src/lib.rs
    - controller/src/derivatives.rs
    - controller/src/catalog.rs
    - controller/src/routes.rs
    - controller/src/oracle_catalog.rs
    - controller/tests/admin_workflow.rs
    - controller/tests/media_cleanup.rs
    - controller/tests/publisher.rs
    - controller/tests/seed_content.rs

key-decisions:
  - "Use the 08-RESEARCH.md package-legitimacy approval for imageproc 0.27.0 and resolve Cargo.lock through the first compile/test gate."
  - "Keep public static JSON/HTML contracts unchanged in Plan 08-04; Plan 08-07 owns adjustment-aware public publisher/cache behavior."
  - "Expose adjustment metadata through the private admin item response while keeping public static artifacts untouched."

patterns-established:
  - "ImageAdjustment::canonical_cache_key returns adjustment:none for identity transforms and includes IMAGE_ADJUSTMENT_TRANSFORM_VERSION for non-identity transforms."
  - "CatalogRepository::update_image_adjustment validates adjustment metadata, updates item timestamps, records ImageAdjustmentChanged, and participates in pending-change counts."
  - "Memory image replacement always clears replacement.adjustment so stale transforms do not carry across new private originals."

requirements-completed: [MEDIA-06]

duration: 13min
completed: 2026-08-17
---

# Phase 08 Plan 04: Image Adjustment Foundation Summary

**Validated Rust image adjustment metadata, transform helpers, and memory repository save/reset behavior now anchor Phase 8 admin media correction work.**

## Performance

- **Duration:** 13min
- **Started:** 2026-08-17T12:42:35Z
- **Completed:** 2026-08-17T12:55:20Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- Added `controller/src/image_adjustments.rs` with bounded rotation, zoom, pan, crop, perspective validation, canonical JSON/cache-key serialization, transform helpers, and deterministic auto-assist proposals.
- Added `imageproc = "0.27.0"` and resolved `Cargo.lock` after confirming `08-RESEARCH.md` marked the package legitimate for Phase 8 projective transforms.
- Kept `generate_derivative(source, variant)` as the existing no-adjustment API while adding `generate_adjusted_derivative` for adjusted preview/public derivative callers.
- Added `AutographImage.adjustment`, `EditEventKind::ImageAdjustmentChanged`, and `CatalogRepository::update_image_adjustment`.
- Implemented memory repository save/reset behavior with validation, timestamp updates, pending-change/history participation, and replacement clearing stale adjustment metadata.
- Added focused tests for validation, canonicalization, derivative generation, auto-assist confidence/unavailable behavior, adjustment history, reset, and replacement clearing.

## Task Commits

1. **Task 1 RED: Add failing image adjustment foundation tests** - `97e2be3` (test)
2. **Task 1 GREEN: Add image adjustment transform foundation** - `cdda67d` (feat)
3. **Task 2 RED: Add failing catalog image adjustment tests** - `706ab34` (test)
4. **Task 2 GREEN: Add catalog image adjustment metadata** - `0e2d135` (feat)
5. **Task 3: Format adjustment foundation files** - `7c042ff` (style)

**Plan metadata:** Pending closeout commit.

## Files Created/Modified

- `controller/src/image_adjustments.rs` - Defines adjustment DTOs, validation, canonical serialization, transform helper, auto-assist proposal, and wrapper helper.
- `controller/tests/image_adjustments.rs` - Covers identity cache key, validation rejection, adjusted derivative output, and auto-assist confident/unavailable fixtures.
- `controller/Cargo.toml` - Adds `imageproc = "0.27.0"`.
- `controller/Cargo.lock` - Resolves `imageproc` and transitive Rust image-processing dependencies.
- `controller/src/lib.rs` - Exports the `image_adjustments` module.
- `controller/src/derivatives.rs` - Adds adjusted derivative generation while preserving the existing wrapper.
- `controller/src/catalog.rs` - Adds private adjustment metadata, repository contract, edit event kind, memory save/reset behavior, and replacement clearing.
- `controller/src/routes.rs` - Sets uploaded/replaced image adjustments to `None` and includes adjustment metadata in private admin item responses.
- `controller/src/oracle_catalog.rs` - Initializes the adjustment field as `None` until Plan 08-05 adds Oracle persistence.
- `controller/tests/admin_workflow.rs` - Adds memory repository adjustment save/reset/replacement tests.
- `controller/tests/media_cleanup.rs`, `controller/tests/publisher.rs`, `controller/tests/seed_content.rs` - Add explicit `adjustment: None` to existing test image fixtures.

## Decisions Made

- Used the approved `imageproc 0.27.0` dependency rather than hand-rolled homography, rotation, sampling, or warp logic.
- Kept adjustment persistence private and additive in the Rust catalog model; Oracle schema/persistence remains Plan 08-05.
- Did not change public static JSON/HTML contracts or publisher cache-key behavior in this plan because Plan 08-07 owns adjustment-aware public publishing/cache behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated existing image constructors for the new adjustment field**
- **Found during:** Task 2 (Add the catalog adjustment field, repository contract, and memory behavior)
- **Issue:** Adding `AutographImage.adjustment` made existing route, Oracle row, and test fixture constructors incomplete.
- **Fix:** Added explicit `adjustment: None` at existing construction sites and preserved private admin response serialization for the new field.
- **Files modified:** `controller/src/routes.rs`, `controller/src/oracle_catalog.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/publisher.rs`, `controller/tests/seed_content.rs`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`, `cargo test --manifest-path controller/Cargo.toml`, and `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- **Committed in:** `0e2d135`

---

**Total deviations:** 1 auto-fixed (Rule 3).
**Impact on plan:** No scope creep. The extra edits were compile/correctness fallout from the planned catalog field addition and did not modify public static contracts.

## Issues Encountered

- `cargo update --manifest-path controller/Cargo.toml -p imageproc` could not target `imageproc` before it appeared in the resolved dependency graph. The first compile/test gate resolved `Cargo.lock` successfully after the dependency was added to `Cargo.toml`.
- Initial `cargo fmt --check` found formatting-only diffs in plan files. `cargo fmt --manifest-path controller/Cargo.toml` fixed them, and the final format check passed.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` passed.
- `cargo test --manifest-path controller/Cargo.toml` passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` passed.
- `git diff --check` passed.
- Source assertions for `pub mod image_adjustments;`, `IMAGE_ADJUSTMENT_TRANSFORM_VERSION`, `ImageAdjustment`, `ImagePerspective`, `canonical_cache_key`, `apply_image_adjustment`, `propose_image_adjustment`, `adjustment: Option<ImageAdjustment>`, `ImageAdjustmentChanged`, and `update_image_adjustment` passed.
- No public static JSON/HTML contract files were modified.

## Known Stubs

None. Stub scan found only an existing test format string in `controller/tests/publisher.rs`, not placeholder behavior.

## Threat Flags

None. The new surfaces are covered by the plan threat model: adjustment payload validation, private media transform bounds, private adjustment DTO exposure, and Cargo dependency legitimacy.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 08-05 can add Oracle persistence and schema preflight against the established `CatalogRepository::update_image_adjustment` contract. Plan 08-07 still owns adjustment-aware publisher/cache behavior and public derivative cache-key verification.

## Self-Check: PASSED

- Created files `controller/src/image_adjustments.rs` and `controller/tests/image_adjustments.rs` exist on disk.
- Task commits `97e2be3`, `cdda67d`, `706ab34`, `0e2d135`, and `7c042ff` exist in git history.
- No public static JSON/HTML contract files changed in this plan.
- Stub scan found no blocking stubs.

---
*Phase: 08-admin-media-review-and-operational-posture*
*Completed: 2026-08-17*
