---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-11T14:14:52Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 9
findings_in_scope: 3
finding_families: 3
fixed: 3
skipped: 0
reassessment_required: false
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-11T14:14:52Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Convergence evidence:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CONVERGENCE.md` at `fa67260`
**Approved plan:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-RECOVERY-SURFACE-CLEANUP-REVISED-PLAN.md`
**Plan review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-RECOVERY-SURFACE-CLEANUP-PLAN-REVIEW.md`
**Iteration:** 9 — single authorized post-convergence coder pass

## Outcome

The three in-scope findings were implemented as one recovery-authority contract change. Exclusive recovery now remains visible and operable while ordinary editor authority is inactive; every live-smoke cleanup path discovers object identities before destructive row cleanup; and all cleanup evidence uses domain-separated fingerprints and safe status categories rather than raw private identifiers or provider-controlled text.

- Findings in scope: 3
- Fixed: 3
- Skipped: 0
- Commit: `d61a38e`
- Status: `all_fixed`

## Finding Lineage and Fixes

### CR-01: Recovery status and retry are descendants of the hidden/inert editor

**Finding family:** `exclusive_conflict_recovery_surface`
**Lineage:** `sibling_path_miss`
**Status:** Fixed
**Commit:** `d61a38e`
**Files:** `controller/static-admin/index.html`, `controller/static-admin/admin.css`, `controller/static-admin/admin.js`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`

**Shared invariant:** Conflict recovery may revoke ordinary editor authority, but pending status and the generation-bound retry must remain perceivable and user-operable outside every hidden or inert ancestor.

**Applied fix:** Added a dedicated live status/action container in `#add-item-view` before and outside `#item-form`. Pending and retryable recovery render only into that container. Existing recovery generation, request owner, editor-authority generation, navigation revision, and current-view checks remain authoritative; invalidation retires and clears the container. The DOM harness now models the real ancestor topology and suppresses user events through disconnected, hidden, inert, and disabled ancestors. Executable cases prove the retry remains visible and operable while editor descendants are inactive and that render, navigation, and new-review authority changes retire late recovery work.

### CR-02: Post-commit replacement failure can leave committed B outside cleanup authority

**Finding family:** `live_smoke_cleanup_discovery`
**Lineage:** `incomplete_fix`
**Status:** Fixed
**Commit:** `d61a38e`
**Files:** `controller/tests/live_persistence_smoke.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/logging_contract.rs`, `docs/static-runtime-runbook.md`

**Shared invariant:** Object identity discovery must complete before any Oracle row capable of revealing a current image key or cleanup-event target is deleted.

**Applied fix:** The live media-revision smoke refreshes cleanup inventory immediately after the replacement response and before asserting its status. Explicit cleanup, the normal guard, and fallback `Drop` now merge and deduplicate locally known candidates, current `autograph_images.object_key` values, and `autograph_cleanup_events.target_object_key` values. They delete and verify every candidate first, then delete the item and verify item, image, edit-event, and cleanup-event counts are zero. Discovery, deletion, or absence-verification failure returns before row deletion so Oracle discovery metadata remains available. Deterministic fixtures cover committed B, duplicate candidates, discovery failure, delete failure, verification failure, already-absent retry, and object-verification-before-row-deletion ordering.

### WR-01: Cleanup transcripts can expose raw private object identities

**Finding family:** `live_smoke_cleanup_evidence_privacy`
**Lineage:** `sibling_path_miss`
**Status:** Fixed
**Commit:** `d61a38e`
**Files:** `controller/tests/live_persistence_smoke.rs`, `controller/tests/logging_contract.rs`, `docs/static-runtime-runbook.md`

**Shared invariant:** Normal, timeout, panic, provider-failure, verification, and recovery-required cleanup evidence may expose only safe counts, item IDs, status categories, and domain-separated fingerprints.

**Applied fix:** Introduced the same length-prefixed `autographs-image-recovery-key-v1` SHA-256 domain separation used by application recovery evidence. Cleanup progress, timeout, provider failure, absence verification, row verification, panic wrappers, and fallback recovery reports no longer interpolate raw keys, checksums, namespace/bucket values, credentials, or provider text. Tests exercise the actual message builders for success, already absent, timeout, provider failure, verification failure, panic, and fallback-thread-panic output with distinctive forbidden markers, and source contracts enforce discovery/deletion order across manual, normal-guard, and `Drop` paths. The runbook now prefers item-ID discovery and states that privately supplied candidates are consumed but never echoed.

## Consumer and Failure-Path Audit

| Invariant | Producers and mutation boundaries | Consumers / reporting paths | Verified failure behavior |
|---|---|---|---|
| Exclusive usable recovery | Current media conflicts; recovery generation/owner; editor and navigation revisions | Pending status, retry, editor form, render/reconcile, navigation, logout, new review | Hidden/inert ancestors block ordinary user events; the external retry is operable; late recovery loses authority |
| Discovery before deletion | Replacement response, current image rows, cleanup-event targets, local A/B candidates | Explicit cleanup, normal guard, fallback `Drop`, OCI delete/read, Oracle cascade and count verification | Discovery/delete/verification failure preserves rows; retry is idempotent; committed B is included before response assertion |
| Opaque evidence | Fingerprint/message builders, provider classification, recovery reporter | Normal progress, already absent, timeout, provider failure, verification, panic, runbook | Forbidden raw markers and provider text are absent; stable fingerprint/status correlation remains |

## Verification

Verification ran in `/tmp/autographs-pr263-review` on branch `gsd/phase-08-admin-media-review-and-operational-posture` at reviewed base `fa6726015a76aaa4dfbae6b168160739a13eb923` plus these uncommitted fixes. Credential-gated Oracle/OCI tests were compiled but not executed.

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed (`static admin behavior tests passed`).
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin` — passed, 19 tests.
- `cargo test --manifest-path controller/Cargo.toml --test media_cleanup` — passed, 20 tests.
- `cargo test --manifest-path controller/Cargo.toml --test logging_contract` — passed, 3 tests.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed; credential-gated test compiled only.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 2 credential-gated live tests ignored as designed, all executed tests green.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-11T14:14:52Z_
_Fixer: gsd-code-fixer_
_Iteration: 9_
_Commit: d61a38e_
