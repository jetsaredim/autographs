---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-04T18:32:21Z
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
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 08: Code Review Report — Post-Convergence Review

**Reviewed:** 2026-10-04T18:32:21Z
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

The coherent post-convergence pass closes the mounted-output authority, persistence-mutation serialization, responsive resize projection, disabled-control simulation, publisher promotion, legacy migration, privacy, and public-output findings. One actionable interaction defect remains in the perspective producer: the first pointer movement schedules a preview and synchronously replaces the entire review stage, detaching the element that owns pointer capture. The test then continues dispatching pointer events directly to that detached element, so it cannot demonstrate the real browser drag lifecycle.

### Round 3 finding lineage

| Round 3 finding | Post-convergence disposition | Evidence |
| --- | --- | --- |
| CR-01 detached same-revision callbacks | Resolved | Network request revision, preview URL, output render generation, exact mounted node, draft revision, item/image, and review session must all remain current before load/error callbacks can update preview state or authorize Save. Detached-node load/error ordering is exercised. |
| WR-01 responsive perspective geometry | Incomplete fix / sibling-path miss | Resize reprojection and 44px edge-target clamping are repaired, but a pointer producer replaces its own captured handle on the first movement, interrupting the continuous drag lifecycle (WR-01). |
| WR-02 pending mutation egress | Resolved | Save and Reset use one identity-scoped pending mutation; Back, tabs, item/signer navigation, direct editor rendering, publish, logout, and unload paths are guarded until success/failure reconciliation. |
| WR-03 browser-faithful DOM coverage | Incomplete fix / test weakness | Disabled user controls, detached image callbacks, resize delivery, and deferred mutations are modeled, but the perspective test manually sends a second user pointer movement to a handle already detached by the first movement (WR-01). |

### Inherited closed-contract check

| Contract | Verdict |
| --- | --- |
| Mounted adjusted-output authority and blob ownership | Closed; stale requests and stale DOM instances cannot update the current output or Save state. |
| Save/Reset repository reconciliation and navigation serialization | Closed for success, failure, and named egress paths. |
| Publisher promotion and post-promotion cleanup | Closed; focused and full publisher tests preserve the active release and public-current map across cleanup failures. |
| Legacy active-release migration | Closed; the active legacy artifact remains protected until a successful publish establishes comparison metadata. |
| Private-original and public-output boundaries | Closed; authenticated previews stay `no-store`, errors remain redacted, and public privacy tests pass. |

### Warnings

#### WR-01: Perspective dragging replaces the pointer-capture owner after the first move

**Classification:** WARNING
**File:** `controller/static-admin/admin.js:2046-2058`
**Related:** `controller/static-admin/admin.js:2070-2078`, `controller/static-admin/admin.js:1612-1631`, `controller/tests/static_admin_behavior.mjs:495-499`

**Issue:** `beginPerspectiveDrag()` captures the pointer on the current corner button. Its first `pointermove` calls `setPerspectiveCorner()`, which calls `markReviewDraftChanged()`. `scheduleDraftPreview()` then synchronously calls `renderImageReview()`, replacing `imageReviewStage` and detaching that captured button and its source frame. A browser will no longer deliver a continuous user drag to the newly created handle without a new pointer-down; any later event retained by the old listener also uses the detached frame's geometry. The DOM test masks this by invoking a second `dispatch("pointermove")` directly on the old `handle` variable after the first dispatch has already rerendered the stage. That is programmatic delivery to a detached node, not browser-faithful user behavior.

**Fix:** Keep the pointer-capture owner mounted for the duration of the drag. Separate source-guide/handle rendering from adjusted-output status rendering, or defer the full stage rerender/preview scheduling until `pointerup`/`pointercancel` while projecting the active handle in place during movement. Track the active drag by review/perspective generation and terminate it on teardown or `lostpointercapture`. Update the harness so user-event dispatch refuses disconnected nodes, assert that multiple physical drag movements update the current normalized corner without replacing the active handle, and reserve explicit detached-node dispatch only for testing stale-callback rejection.

## Verification

- `git diff --check origin/main...HEAD` — passed.
- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed, but the perspective sequence contains the detached-node fidelity gap described above.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` — passed; 18 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` — passed; 36 tests.
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture` — passed; 4 tests.
- `cargo test --manifest-path controller/Cargo.toml --all-targets` — passed; 163 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.

---

_Reviewed: 2026-10-04T18:32:21Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
