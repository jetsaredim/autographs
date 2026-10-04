---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-04T18:38:30Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 4
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-04T18:38:30Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 4 — post-convergence pointer-lifecycle correction

**Summary:**

- Findings in scope: 1
- Fixed: 1
- Skipped: 0
- Implementation commit: `7c11380`
- PR fix comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5983155034

## Fixed Issues

### WR-01: Perspective dragging replaces the pointer-capture owner after the first move

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`
**Commit:** `7c11380`
**Applied fix:** Pointer movement now mutates the normalized perspective model and reprojects the mounted handles in place without scheduling a preview or replacing the captured node. The active drag records review-session, item/image, perspective-generation, frame, image, handle, and pointer identities; every movement revalidates that authority. Pointer up, cancel, or lost capture removes all gesture listeners and schedules exactly one settled preview. Projection teardown or a session/render invalidation terminates the gesture without scheduling stale work. Concurrent keyboard, assist, comparison, overlay, adjustment, Save, and Reset actions cannot supersede an active drag.

The executable fake DOM now distinguishes connected browser-user delivery from explicit programmatic delivery. User dispatch to a disconnected node is rejected. Tests prove two physical moves reach the same connected captured handle, normalized geometry changes in place, no request or preview revision advances before settlement, exactly one settled preview contains the final corner, and cancel, lost capture, teardown, and deliberate stale-node delivery cannot mutate retired gesture state.

## Shared-Invariant and Consumer Audit

- **Pointer producer authority:** Pointer down establishes one capture owner and immutable session/item/image/perspective-generation identity. Pointer move validates that complete tuple before changing normalized coordinates. A second pointer cannot steal the active gesture.
- **Sibling producers:** Keyboard movement retains the existing immediate preview path when no drag is active. Auto-assist, scalar adjustment controls, overlays, comparison controls, Save, and Reset refuse concurrent action while pointer authority is live, preventing an unrelated full render from detaching the capture owner.
- **Consumers:** Handle pixels remain projections of normalized corners; each accepted movement reprojects all current handles without replacing them. Dirty state and Save authorization update immediately, while adjusted output remains non-authoritative until the settled preview loads.
- **Mutation and invalidation boundaries:** Pointer up, pointer cancel, and lost capture settle once. Render/projection teardown, review-session change, item/image change, pending persistence mutation, disconnected frame/handle, or generation mismatch remove gesture listeners without scheduling stale preview work.
- **Reporting/action paths:** Dirty reporting and Save disabling occur on the first accepted move. The settled preview owns the final draft revision and restores Save authority only after its current output node loads.
- **Test fidelity:** Browser-user dispatch rejects disconnected nodes. `dispatchProgrammatic` remains explicit and is used only where tests intentionally exercise stale callbacks or handler-level guards.

## Verification

All gates ran in `/tmp/autographs-pr263-review` on `gsd/phase-08-admin-media-review-and-operational-posture` at implementation commit `7c11380`.

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

_Fixed: 2026-10-04T18:38:30Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 4_
