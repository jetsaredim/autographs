---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-12T00:44:13Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 11
findings_in_scope: 1
finding_families: 1
fixed: 1
skipped: 0
reassessment_required: false
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-12T00:44:13Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Convergence evidence:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CONVERGENCE.md` at `602a882`
**Approved plan:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CANDIDATE-OWNERSHIP-REVISED-PLAN.md`
**Plan review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CANDIDATE-OWNERSHIP-PLAN-REVIEW.md`, iteration 3 approved with zero findings
**Iteration:** 11 — single authorized post-reassessment coder pass

## Outcome

The remaining CR-02 finding was implemented as one terminal candidate-ownership contract. The live smoke now commits private cleanup authority for initial candidate A before the object can be written, candidate refresh cannot consume the guard's existing set on discovery failure, and the reserved authority operation cannot enter ordinary admin cleanup actions.

- Findings in scope: 1
- Finding families: 1
- Fixed: 1
- Skipped: 0
- Commit: `c356600`
- Status: `all_fixed`

## Finding Lineage and Fix

### CR-02: Failed inventory consumes the guard's only copy of locally known cleanup candidates

**Finding family:** `live_smoke_cleanup_discovery`
**Lineage:** `incomplete_fix`
**Status:** Fixed
**Commit:** `c356600`
**Files:** `controller/tests/live_persistence_smoke.rs`, `controller/tests/media_cleanup.rs`, `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, `controller/src/routes.rs`

**Shared invariant:** Every smoke-created private object has durable Oracle cleanup authority before creation; fallible inventory never mutates the installed candidate set; object absence is verified before discovery rows are deleted; and only application operations `delete` and `replace` may enter admin warning/retry paths.

**Applied fix:** The initial A key is registered and committed as a private `livePersistenceSmokeGuard` cleanup event after the item exists and before `media.write`. Registration or commit failure aborts before object creation. The authority row remains available to the inclusive item-ID inventory and is removed only by the final item cascade after all candidates are verified absent. `refresh_inventory` now calls a production helper that copies the installed candidates into a fallible discovery producer and assigns the returned union only on success, so query, row-read, or decode failure cannot empty or partially replace guard state.

One shared actionability predicate permits only `delete` and `replace`. The Memory and Oracle cleanup-warning loaders both apply it, and `retry_image_cleanup` checks it again before media deletion, metadata removal, or event resolution. The reserved smoke authority remains discoverable by private item-ID cleanup while staying absent from item/status warning responses and unreachable through cleanup retry.

## Finding Lineage Preserved

- Iteration 9 closed CR-01 (`exclusive_conflict_recovery_surface`) and WR-01 (`live_smoke_cleanup_evidence_privacy`); this pass does not modify their UI or evidence surfaces.
- Iteration 9's CR-02 discovery-first change remains intact: local candidates, current image keys, and every cleanup-event target are inventoried before object deletion and destructive SQL.
- Iteration 10 reclassified CR-02 as an `incomplete_fix` because `mem::take` erased local candidates on failed discovery. The iteration-11 helper removes that ownership transfer.
- Plan-review iteration 1's terminal-lifecycle blocker is closed by register-before-create durable authority and fresh item-ID recovery after guard destruction.
- Plan-review iteration 2's actionability blocker is closed by the shared allowlist at both warning loaders and defense-in-depth at the route.

## Executable Evidence

| Boundary | Evidence |
| --- | --- |
| Register before create | Lifecycle fixture proves authority registration and commit precede object write; forced registration failure performs no write. A source-order assertion binds the fixture to the live smoke call sequence. |
| Atomic refresh | Pure tests invoke the same helper as `LivePersistenceSmokeCleanup::refresh_inventory`; partial failure preserves the original set and retry installs a deduplicated A/B union. |
| Terminal recovery | Repeated discovery failure retains rows; a fresh cleanup with no local guard state rediscovers A from cleanup events and B from current-image rows, verifies both absent, then removes rows. |
| Repository filtering | Memory behavior test omits the reserved event; Oracle source/query contract proves it applies the shared predicate while item-ID inventory remains operation-inclusive. |
| Route defense | An injected repository deliberately returns a reserved warning; the route returns conflict while the object, image metadata, and unresolved event remain unchanged. |
| Allowed operations | Shared predicate tests accept exactly `delete` and `replace`; existing delete and replacement retry integration tests remain green. |
| Privacy | Registration errors contain item/step context without the raw key; existing cleanup evidence tests retain domain-separated fingerprint-only output and reject provider-controlled text. |

## Verification

Verification ran in `/tmp/autographs-pr263-review` on branch `gsd/phase-08-admin-media-review-and-operational-posture` at `602a882` plus these uncommitted fixes. Credential-gated Oracle/OCI smoke cases were compiled but not executed.

- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test media_cleanup` — passed, 26 tests.
- `cargo test --manifest-path controller/Cargo.toml --test logging_contract` — passed, 3 tests.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke` — passed, 4 executed and 1 credential-gated test ignored.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 2 credential-gated live tests ignored as designed, all executed tests green.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-12T00:44:13Z_
_Fixer: gsd-code-fixer_
_Iteration: 11_
_Commit: c356600_
