---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-08-21T00:21:59Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - controller/Cargo.toml
  - controller/Cargo.lock
  - controller/src/lib.rs
  - controller/src/image_adjustments.rs
  - controller/src/derivatives.rs
  - controller/src/catalog.rs
  - controller/src/routes.rs
  - controller/src/oracle_catalog.rs
  - controller/tests/image_adjustments.rs
  - controller/tests/admin_workflow.rs
  - controller/tests/media_cleanup.rs
  - controller/tests/publisher.rs
  - controller/tests/seed_content.rs
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Phase 08 Plan 04: Code Review Report

**Reviewed:** 2026-08-21T00:21:59Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Reviewed the Wave 4 image-adjustment model, transform helper, repository contract, route serialization, Oracle placeholder behavior, dependency changes, and focused tests. The focused `image_adjustments` test suite passes, but the perspective transform is wired backward, invalid perspective metadata can still be saved, and the new dependency re-enables image codecs outside the project upload/runtime contract.

## Critical Issues

### CR-01: Perspective Warp Uses the Inverse Control-Point Direction

**Severity:** BLOCKER
**File:** `controller/src/image_adjustments.rs:272`
**Impact:** `imageproc::warp` expects a projection from input image coordinates to output coordinates and inverts it internally while sampling. The implementation builds `Projection::from_control_points(target, source)`, so a saved perspective correction maps the output rectangle back to the selected source quadrilateral before `warp` inverts it again. Real perspective corrections will produce the wrong crop/warp even though validation and tests pass.
**Suggested Fix:**

```rust
let projection = Projection::from_control_points(source, target)
    .ok_or_else(|| "perspective corners do not form a valid projection".to_owned())?;
```

Add a regression that applies a known trapezoid-to-rectangle perspective correction and asserts visible pixels land at the expected output corners.

## Warnings

### WR-01: Degenerate Perspective Metadata Passes Repository Validation

**Severity:** WARNING
**File:** `controller/src/image_adjustments.rs:141`
**Impact:** `ImagePerspective::validate` only checks that corner coordinates are finite and normalized. Duplicate, collinear, or otherwise non-invertible corner sets pass `ImageAdjustment::validate`, so `CatalogRepository::update_image_adjustment` can persist metadata that later fails in `apply_perspective` during preview/publish derivative generation.
**Suggested Fix:** Extend perspective validation to construct the same source-to-target projection used by the transform and reject `None`; add tests for duplicate and collinear corners.

### WR-02: `imageproc` Default Features Re-enable Unapproved Image Decoders

**Severity:** WARNING
**File:** `controller/Cargo.toml:23`
**Impact:** The controller pins `image` to JPEG/PNG/WebP at line 22, but `imageproc = "0.27.0"` enables `imageproc` defaults, including `image/default`. `cargo tree --edges features` shows this activates AVIF, BMP, DDS, EXR, GIF, HDR, ICO, PNM, QOI, TGA, TIFF, and related transitive crates such as `ravif`. That expands the private-media decoding surface beyond the project’s accepted upload formats and adds unnecessary dependency risk.
**Suggested Fix:**

```toml
imageproc = { version = "0.27.0", default-features = false }
```

Keep only explicitly needed `imageproc` features if a transform path requires them, and add a dependency/feature check or comment documenting why the image codec set remains restricted.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture` passed.
- `git diff --check origin/main...65fd32a4a629582881e80868701aac239e487718 -- <scoped files>` passed.

---

_Reviewed: 2026-08-21T00:21:59Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
