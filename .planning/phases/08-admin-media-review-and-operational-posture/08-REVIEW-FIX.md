---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-03T13:18:13Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 3
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-03T13:18:13Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 3 — post-convergence coherent contract pass

**Summary:**

- Findings in scope: 4
- Fixed: 4
- Skipped: 0
- Convergence evidence: `08-PR-263-CONVERGENCE.md`
- Approved plan: `08-PR-263-ROUND3-REVISED-PLAN.md`
- Independent plan review: `08-PR-263-ROUND3-PLAN-REVIEW.md`
- PR evidence comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5982678671

## Fixed Issues

### CR-01: Detached same-revision image callbacks can authorize Save or erase a newer render state

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Applied fix:** Added a monotonically increasing adjusted-output render generation distinct from preview-request ordering. Every callback must match the current review session, item/image, draft revision, preview revision and blob URL, render generation, and exact mounted image node. Mount, rerender, retry, URL replacement, and teardown invalidate prior authority before DOM replacement. The harness retains detached nodes and proves late load/error events are inert while the current blob remains owned and unrevoked.

### WR-01: Perspective handle geometry is stale after resize and full-frame targets are clipped

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Applied fix:** Normalized source corners remain the sole geometry model. A review-scoped `ResizeObserver` reprojects handles from current intrinsic and frame dimensions, and is invalidated on rerender or teardown. Visual centers clamp 22px inside the frame without changing normalized coordinates or pointer mapping, while CSS guarantees a 44px by 44px target. Portrait and landscape resize cases cover all four edge corners.

### WR-02: Back/discard can hide a Save or Reset that continues mutating persistence

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`
**Applied fix:** Save and Reset now create one review-scoped pending-mutation record containing operation, token, session, item, and image identity. Review Back, tabs, item navigation, signer navigation, publish routes, direct editor rendering, logout, and unload are serialized behind the same handler-level guard while visible controls are disabled. Success reconciles the returned item, saved baseline, draft, dirty state, message, and publish availability before reopening egress; failure retains the review and baseline and reports the error before reopening controls.

### WR-03: The DOM harness dispatches user input on disabled controls and misses browser interleavings

**Status:** Fixed
**Files modified:** `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Applied fix:** User-event dispatch now ignores disabled controls and intentional programmatic dispatch is explicit. The harness controls retained image nodes, resize delivery, deferred preview requests, blob revocation, and deferred Save/Reset responses. It executes old/new load and error ordering, older request completion, portrait and landscape reprojection, disabled edit attempts, Back/tab/item/logout attempts, and success/failure reconciliation.

## Shared-Invariant and Consumer Audit

- **Mounted-output authority:** Audited draft request production, blob creation/revocation, all adjusted-output mounts, load/error reporting, Save enablement, comparison and overlay rerenders, retry, session/item changes, and teardown. Request ordering and DOM generation are separate and jointly required.
- **Normalized geometry authority:** Audited source intrinsic load, fitted source bounds, observer delivery, keyboard and pointer mutation, assist mutation, responsive rerender, labels, and full-size edge targets. Pixel positions are disposable projections only.
- **Mutation/navigation serialization:** Audited Save, Reset, assist, adjustment inputs, pointer drags, comparison and overlays, Back, tabs, item and signer navigation, direct editor rendering, logout, publish, and unload. Presentation disabling is backed by handler-level refusal.
- **Test fidelity:** Audited disabled/enabled transitions, explicit programmatic events, detached nodes, request and blob ownership, mutable rectangles, fake resize observers, deferred mutations, returned repository items, messages, baselines, dirty state, and navigation availability.
- **Preserved closed contracts:** Publisher promotion/cleanup safety, legacy-release migration, Oracle persistence, privacy, and public media behavior were not changed; their existing focused and full-suite regressions remain green.

## Verification

All gates ran in `/tmp/autographs-pr263-review` on `gsd/phase-08-admin-media-review-and-operational-posture`.

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` — passed; 18 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` — passed; 36 tests.
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture` — passed; 4 tests.
- `cargo test --manifest-path controller/Cargo.toml --all-targets` — passed; 163 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-03T13:18:13Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 3_
