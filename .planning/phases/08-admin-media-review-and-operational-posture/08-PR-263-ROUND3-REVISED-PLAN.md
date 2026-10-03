---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: approved
depends_on:
  - 08-REVIEW.md
  - 08-PR-263-CONVERGENCE.md
files_modified:
  - controller/static-admin/admin.js
  - controller/static-admin/admin.css
  - controller/tests/static_admin_behavior.mjs
  - controller/tests/static_admin.rs
  - controller/tests/admin_workflow.rs
---

# PR 263 Round 3 Revised Implementation Plan

## Contract Decisions

### Mounted adjusted-output authority

Introduce a monotonically increasing output-render generation that advances before every adjusted-output mount and every teardown/invalidation. A load/error callback is authoritative only when all of these still match current state: review session, item/image identity, draft revision, preview request/blob identity, output-render generation, and the exact mounted image node. Rerendering comparison or overlay controls creates a new generation even when the draft and preview URL are unchanged. Superseded callbacks do nothing; superseded blob URLs are revoked only when no current render owns them.

Preview status returns to `pending` whenever a new adjusted-output render is mounted. Save is enabled only when the current generation reports `ready` for the current draft revision. A failed current render stays retryable without allowing an older node to clear its error.

### Responsive perspective geometry

Keep normalized source corners as the sole authoritative geometry. Centralize `projectPerspectiveHandles()` so it derives fitted source bounds and visual pixels from current intrinsic dimensions, current source-frame dimensions, and normalized values. Invoke it after source load, every normalized-corner mutation, and every observed source-frame size change. Register one review-scoped `ResizeObserver` (with a window-resize fallback only if required by supported runtime behavior), and disconnect/invalidate it on rerender or teardown.

Clamp only each visual button center by half its interactive diameter within an unclipped overlay region, or provide equivalent overlay gutter, while retaining the exact normalized edge coordinate for requests and labels. Preserve the full 44px hit target for all four corners at narrow layouts.

### Persistence mutation and egress serialization

Represent Save/Reset submission with one review-scoped pending-mutation record containing operation, session, item/image, and request token. Before issuing PATCH/DELETE, enter pending state and disable every review egress surface: Back, tabs, item selection, logout, and any route/navigation action. Each handler must independently refuse egress while pending so programmatic invocation cannot bypass presentation state. Do not show a discard prompt for already submitted work.

Keep the review session alive until the mutation settles. On success, reconcile the returned item and adjustment into `state.currentItem`, the saved baseline, draft/dirty state, publish-boundary reporting, and success message before clearing pending state. On failure, retain the review and unsaved baseline, show the error, and then clear pending state. Only settled state may be left or discarded. Teardown invalidates any remaining observer/render callbacks.

### Browser-faithful executable tests

Split fake-DOM event delivery into a user-event helper that refuses disabled controls and an explicit programmatic-dispatch helper. Add controllable image-node events, retained detached nodes, a fake `ResizeObserver`, mutable bounding rectangles, and deferred mutation responses. Tests must assert both state and visible/control outcomes, and where applicable the repository/API request outcome.

## Coherent Implementation

### 1. Consolidate review lifecycle authority

- Add state/helpers for output-render generation, current mounted adjusted image, preview ownership, resize-observer ownership, and pending persistence mutation.
- Centralize invalidation so review open, rerender, retry, item/image change, Back after settlement, and logout cannot leave authoritative callbacks behind.
- Keep request revision and render generation distinct: request revision orders network results; render generation orders DOM instances created from an accepted result.
- Require the complete authority tuple before changing preview status, displayed revision, Save availability, stage content, or message state.

### 2. Make perspective projection responsive and accessible

- Centralize fitted-bounds calculation and handle projection from normalized corners.
- Observe frame geometry only for the current review/render lifecycle and reproject without changing model values.
- Ensure pointer conversion still uses the true fitted source bounds, not visually inset button centers.
- Preserve keyboard controls and labels after resize.
- Adjust overlay layout/CSS so full-size hit targets remain visible at all edges and the narrow breakpoint.

### 3. Serialize submitted mutations with navigation

- Add one pending-mutation guard used by Save, Reset, Back, tabs, item navigation, logout, and other review egress handlers.
- Keep success/failure reporting in the active review until the response is reconciled.
- Ensure Save advances the baseline only to the submitted/returned adjustment and preserves any legitimate enabled concurrent changes if such a producer exists; otherwise all edit producers remain disabled during submission.
- Ensure Reset success reconciles the no-adjustment baseline and current item before controls or egress reopen.
- Avoid cancellation claims unless the request and persistence layer implement actual cancellation; this plan uses serialization rather than cancellation.

### 4. Replace permissive test simulation with decisive interleavings

- Make disabled user inputs inert in the harness and update existing tests that relied on browser-impossible dispatch.
- Retain the first adjusted image, trigger a same-revision overlay/comparison rerender, then fire old/new load and error events in both orders; only the current node may affect state.
- Resize portrait and landscape source frames after load; prove normalized values stay stable, pixels reproject, and all four edge targets retain 44px reachability at narrow width.
- Defer Save and Reset responses, attempt Back/tab/item/logout, and prove no egress or hidden mutation occurs; then settle success and failure and assert current item, baseline, dirty state, messages, and navigation availability.
- Preserve prior request-order, legacy-release, and publisher failure-matrix coverage.

## Verification

- `node --check controller/static-admin/admin.js`
- `node --check controller/tests/static_admin_behavior.mjs`
- `node controller/tests/static_admin_behavior.mjs`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --all-targets`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
- `git diff --check`

## Non-Goals

- Do not alter publisher promotion, legacy-release migration, Oracle persistence, or public media contracts unless a failing regression proves the coherent lifecycle change affects them.
- Do not add request cancellation semantics.
- Do not replace normalized perspective data with pixel coordinates.
- Do not weaken dirty-state, publish-boundary, privacy, or private-original protections.
- Do not split the four findings into independent point-fix commits that duplicate lifecycle state.

## Done

The plan is implemented only when one shared lifecycle authority closes CR-01 and WR-01–03, all decisive interleavings pass in the executable DOM harness, focused Rust contracts remain green, the full verification set passes, and a lineage-preserving deep review reports zero actionable findings.
