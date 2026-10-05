---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-05T01:05:27Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 5
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-05T01:05:27Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 5 — approved pointer-lifecycle addendum

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0
- Implementation commit: `0559c82`
- PR fix comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5986381118

## Fixed Issues

### WR-01: Pre-existing async producers can still detach the active pointer-capture owner

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Commit:** `0559c82`
**Applied fix:** Added one stage-render barrier around every `renderImageReview()` call. While a current gesture owns the connected source frame, preview success/failure and adjusted-output failure may update authoritative state but defer stage replacement. Accepted no-move work flushes once after capture cleanup; moved gestures supersede pre-gesture output/error state and schedule one final preview. Source load only establishes readiness or reprojects in place, source error terminates once before changing the connected source frame, and output-only deferred reconciliation preserves that error frame. Assist completions and synchronous retry/comparison/overlay/scalar/keyboard/Save/Reset/navigation/logout producers are rejected or disabled during the gesture. Session, item/image, login, and review teardown discard queued work and release capture without scheduling retired requests.

### WR-02: Dragging a visually inset edge handle jumps the normalized corner

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Commit:** `0559c82`
**Applied fix:** Added an explicit source-projection readiness gate and normalized fitted-source grab offset. Pointerdown is rejected until intrinsic and rendered geometry are non-zero. Movement applies `currentPointer + grabOffset`, so a move at the visually inset handle center is a model no-op and later pixel deltas produce matching normalized source deltas. Resize and redundant source-load callbacks rebase the offset from the current mathematical corner and last client coordinates before reprojecting the same connected handles.

## Direct and Indirect Producer Audit

| Producer | Active-gesture policy | Terminal reconciliation |
|---|---|---|
| Source first/redundant load | Pre-readiness handles are disabled; authoritative load enables or reprojects the existing handles in place and rebases an active grab. | No queued stage render. |
| Source error | Enters the idempotent terminal path before projection teardown and replaces only source-frame children. | No-move deferred output renders in place once; moved geometry schedules one final preview without an immediate stage replacement. |
| Preview success | Request/session/draft checks remain authoritative; accepted blob/status is recorded and the centralized stage render is deferred. | No-move flushes once; moved geometry revokes/supersedes it and schedules the final preview. |
| Preview failure | Authoritative error/status is recorded and its stage render is deferred. | No-move flushes once; moved geometry restores the pre-gesture message and supersedes the failure with the final preview. |
| Adjusted-output load | Updates only the already-mounted authoritative output and Save state in place. | Preserved unless a moved gesture supersedes its draft. |
| Adjusted-output error | Records the authoritative error and defers the full render. | No-move flushes once; moved geometry supersedes it with the final preview. |
| Assist completion | A completion that observes an active drag is rejected without proposal or message mutation. | No deferred work. |
| Retry, comparison, overlays, scalar controls, keyboard, Save, Reset | Presentation controls are disabled where applicable and every handler independently rejects the action. | Operator may retry after settlement. |
| Resize observer | Rebase grab offset from the last client position under the new fitted bounds, then reproject the same handles. | Model remains unchanged by resize itself. |
| Navigation/logout/session/item/image/teardown | Handler guard blocks ordinary egress; explicit lifecycle invalidation marks terminal, removes listeners, conditionally releases capture, and discards deferred render state. | No preview is scheduled for a retired review. |

Every direct `renderImageReview()` caller was audited: preview scheduling, preview success/failure, initial review open, adjusted-output failure, terminal deferred flush, comparison mode, and overlay changes now pass through the centralized barrier or execute only after gesture authority has ended. The only review-stage replacements are the barrier-controlled full render and source-error's terminal in-place source-frame replacement. Handle recreation occurs only during a full stage render; source load and resize are in-place projections.

## Terminal-State Audit

- Pointer up and pointer cancel mark the drag terminal before listener removal and conditional capture release. A synchronous/reentrant lost-capture callback sees terminal state and cannot settle twice.
- External lost capture uses the same terminal path without recursively releasing already-lost capture.
- Source error uses the same terminal path, preserving accepted geometry and reconciling deferred output/final preview exactly once before or after its in-place error presentation as required.
- Review/session/item/image/login teardown discards deferred work and never schedules a preview for the retired authority.
- Exact authority remains the conjunction of review session, item/image, perspective generation, connected frame/image/handle, and pointer identity. Existing output authority additionally retains request revision, draft revision, preview URL, render generation, and exact mounted node.

## Executable Regression Coverage

- Pre-load handles are disabled and cannot capture; first load enables them without replacement.
- All four full-frame inset corner centers accept a same-coordinate move as a no-op.
- Portrait and landscape gestures apply two source-space deltas without a first-move jump.
- Resize and redundant load during capture preserve the normalized model, rebase the grab offset, and retain the same connected handle.
- Accepted preview success, preview failure, stale success/failure between moves, adjusted-output load/error, and an in-flight assist completion are exercised while capture is live.
- Source error is exercised before and after movement, including output-only no-move deferred reconciliation.
- Pointer up, cancel, external lost capture, reentrant release-triggered lost capture, projection teardown, and full review/session teardown prove exactly-once cleanup and scheduling/discard behavior.
- Browser-user delivery rejects disconnected nodes; explicit programmatic delivery is reserved for stale-callback and handler-guard tests.

## Verification

All gates ran in `/tmp/autographs-pr263-review` on `gsd/phase-08-admin-media-review-and-operational-posture` at implementation commit `0559c82`.

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

_Fixed: 2026-10-05T01:05:27Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 5_
