---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-09-30T23:21:32Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-261-REVIEW-ITER2.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-30T23:21:32Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-261-REVIEW-ITER2.md`
**Iteration:** 2

**Summary:**

- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-02: The constraint replacement can commit the drop before the replacement exists

**Files modified:** `controller/db/updates/08-01-image-adjustments.sql`, `controller/src/oracle_schema.rs`, `controller/tests/live_persistence_smoke.rs`
**Commit:** 105d890
**Status:** fixed: requires human verification
**Applied fix:** Changed the migration to stage and validate the expanded constraint under a temporary name before removing the legacy canonical constraint, recover the old-only, both, temporary-only, and canonical-new states, and exercise those state transitions in the credentialed live migration fixture.

### WR-03: Runtime preflight accepts an edit-event constraint missing seven supported values

**Files modified:** `controller/db/updates/08-01-image-adjustments.sql`, `controller/src/oracle_schema.rs`, `controller/tests/live_persistence_smoke.rs`
**Commit:** c312dd2
**Status:** fixed
**Applied fix:** Made production schema preflight validate every value in `EditEventKind::ALL` against the enabled canonical Oracle constraint, retained remediation through the Phase 8 migration, and added partial-constraint rejection coverage while keeping the migration/live fixtures aligned with the complete event set.

### WR-04: A passing live smoke can silently leave production fixtures behind

**Files modified:** `controller/tests/live_persistence_smoke.rs`
**Commit:** 668a828
**Status:** fixed
**Applied fix:** Made explicit async cleanup authoritative on the successful smoke path. Cleanup now requires OCI deletion and verified 404 absence, requires the Oracle item delete and commit to succeed, and confirms item, image, and edit-event rows are absent. The `Drop` path remains best-effort for panic recovery and emits a stable `LIVE_PERSISTENCE_RECOVERY_REQUIRED` message containing both the item ID and object key when fallback cleanup fails. Manual cleanup now also distinguishes a confirmed 404 from unrelated OCI read failures.

## Verification

Verification ran in the main checkout on `gsd/phase-08-admin-media-review-and-operational-posture` because `workflow.use_worktrees=false`.

- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed; the credentialed Oracle/OCI smoke compiled but was not executed locally.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_ -- --nocapture` — passed: 44 tests total, including the matching deployment contract test.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-09-30T23:21:32Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
