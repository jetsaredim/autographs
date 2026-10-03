---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-03T00:55:30Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 2
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-03T00:55:30Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 2

**Summary:**

- Findings in scope: 8
- Fixed: 8
- Skipped: 0
- Coherent source/test commit: `e1c1f0a`

## Fixed Issues

### CR-01: Save remains enabled while the displayed preview is stale or failed

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `e1c1f0a`
**Applied fix:** Draft edits now advance an explicit draft revision, immediately remove stale adjusted pixels from the authoritative output panel, and disable Save until the matching blob has both returned and fired its image `load` event. Failed or pending renders keep Save disabled and expose only loading/error UI.

### CR-02: An in-flight Save marks later, unsent edits as saved

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `e1c1f0a`
**Applied fix:** Save snapshots the submitted adjustment, draft revision, and review session. Its response advances only the saved baseline, preserving a later draft as dirty; mutation controls are disabled while Save/Reset is pending. The deferred-fetch harness verifies the PATCH body contains revision A while a later revision B remains dirty after A succeeds.

### CR-03: Post-promotion cleanup can remove the active static release

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/publisher.rs`, `controller/src/routes.rs`, `controller/static-admin/admin.js`
**Commit:** `e1c1f0a`
**Applied fix:** Pre-promotion failures retain failed candidates, while post-promotion retention is a separate housekeeping result. Once `current` switches, pruning failure returns a successful publish with a redacted `cleanupWarning`; the active release and its private comparison map are never moved or deleted. Inactive maps are removed before inactive releases so a failure remains retryable on the next publish.

### CR-04: Perspective handles use frame coordinates instead of source-image coordinates

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/routes.rs`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/admin_workflow.rs`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `e1c1f0a`
**Applied fix:** Added an authenticated, no-store sanitized unadjusted source-guide endpoint. Perspective handles now live on that guide and calculate fitted image bounds from its intrinsic aspect ratio, while the canonical adjusted derivative remains a separate output preview. Tests cover portrait letterboxing, normalized pointer coordinates, a non-identity saved adjustment, and distinct source/output pixels.

### CR-05: Older async review actions can overwrite a newer item or review session

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `e1c1f0a`
**Applied fix:** Review open, draft render, assist, Save, and Reset now carry one item/image/session identity and the relevant draft revision across every await. Closing, navigating, switching item/image, or beginning a newer review invalidates older responses. Deferred A→B, assist-after-close, reset-after-reopen, and review→navigation interleavings execute in the harness.

### CR-06: Clean reviews hide the only Discard and Reset actions

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `e1c1f0a`
**Applied fix:** Back to item editor and Reset are now outside the conditional dirty band and remain reachable in clean and dirty states. Only Save depends on dirty state and latest-preview authority. The DOM harness opens a clean review, invokes Reset without an edit, and uses Back while a mutation is pending.

### WR-01: Existing active releases cannot provide public-current comparison

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/publisher.rs`, `controller/src/routes.rs`, `controller/tests/admin_workflow.rs`
**Commit:** `e1c1f0a`
**Applied fix:** Active-release lookup now distinguishes no release, a mapped release, and a pre-map release. A published item on a legacy release reports `comparisonMigrationRequired` with the safe rollout contract: run one full publish after upgrade; the current release remains unchanged unless that publish succeeds. An integration test constructs a legacy active release without a map and verifies the explicit migration state and unchanged public artifact.

### WR-02: The DOM harness does not execute draft requests or user event paths

**Status:** Fixed — requires human verification
**Files modified:** `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_admin.rs`, `controller/tests/admin_workflow.rs`, `controller/src/publisher.rs`
**Commit:** `e1c1f0a`
**Applied fix:** Replaced the non-executing timer/fetch stubs with a flushable fake clock, controllable/deferred fetch queue, response bodies/blobs, and registered-event dispatch. Assertions cover serialized request bodies, blob render revisions, image load success/failure, dirty state after deferred Save, A→B/close/navigation response ordering, clean Reset/Back, publish refusal, portrait source bounds, and promotion map/release deletion failures.

## Lineage and Shared-Invariant Coverage

- **Incomplete Round 1 fixes completed:** Round 1 CR-01 now includes displayed-pixel revision authority and source-coordinate parity; Round 1 CR-02 now includes migration and post-promotion atomicity.
- **Fix regressions repaired:** Round 2 CR-02 and CR-05 close the asynchronous dirty/session regressions introduced around the Round 1 synchronous navigation guards.
- **Sibling-path misses repaired:** The same session/revision checks cover review, draft preview, assist, Save, Reset, item/image switching, Back, top-level navigation, and logout invalidation. Promotion housekeeping covers both inactive-map and inactive-release deletion failures without touching the active pair.
- **Independent workflow reachability:** CR-06 separates always-reachable Back/Reset actions from the dirty-only message and preview-authorized Save.
- **Test weaknesses repaired:** WR-02 now executes the actual registered event paths and request interleavings rather than calling only synchronous helpers.
- **Producer/consumer/mutation inventory:** Producers are the unadjusted source route, draft renderer, active-release map, and publisher; consumers are the source guide, adjusted output/comparison panes, and review response; mutations are Save/Reset/assist/manual edits and release promotion; reporting/action paths are publish status, cleanup warning, migration state, navigation, publish refusal, and retry UI.

## Verification

All gates ran in the provided isolated checkout `/tmp/autographs-pr263-review` on branch `gsd/phase-08-admin-media-review-and-operational-posture`.

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed after formatting.
- `cargo test --manifest-path controller/Cargo.toml --lib publisher::tests -- --nocapture` — passed; 4 publisher unit tests, including map and release deletion failure injection.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` — passed; 18 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` — passed; 36 tests after adding migration and portrait source/output coverage.
- `cargo test --manifest-path controller/Cargo.toml` — passed; 163 non-ignored tests passed and 2 credential-gated live tests remained ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-03T00:55:30Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
