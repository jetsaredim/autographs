---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-09-30T10:31:30Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-261-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-30T10:31:30Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-261-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: Oracle rejects every image-adjustment history event

**Files modified:** `controller/db/schema.sql`, `controller/db/updates/08-01-image-adjustments.sql`, `controller/src/oracle_schema.rs`
**Commit:** 186b7c5
**Status:** fixed
**Applied fix:** Added `imageAdjustmentChanged` to the canonical Oracle check constraint, made the Phase 8 migration idempotently replace that constraint with the complete event set, and made schema preflight require the Phase 8 event value and remediation script.

### CR-02: Invalid persisted adjustment JSON can produce a committed write reported as failure

**Files modified:** `controller/src/oracle_catalog.rs`, `controller/tests/live_persistence_smoke.rs`
**Commit:** 855a87e
**Status:** fixed: requires human verification
**Applied fix:** Moved full-item readback before commit for every Oracle mutation that returns `load_item` data—create, item update, attach, set-primary, remove, replace, and adjustment update—and added explicit rollback when that validation fails. Added a live Oracle regression that induces malformed sibling adjustment JSON and verifies the target row, item timestamp, and history count remain unchanged.

### WR-01: Oracle persistence tests can pass without exercising Oracle persistence

**Files modified:** `controller/src/catalog.rs`, `controller/src/oracle_schema.rs`, `controller/tests/live_persistence_smoke.rs`
**Commit:** 89f8ca6
**Status:** fixed
**Applied fix:** Added a credential-free schema contract that checks every Rust edit-event value against both canonical and migration DDL. Expanded the ignored live Oracle smoke to cover schema preflight, save/reset CLOB round trips, item timestamp changes, exact history counts, wrong item/image IDs, replacement clearing, and failed-readback rollback.

## Verification

Verification ran in the main checkout on `gsd/phase-08-admin-media-review-and-operational-posture` because `workflow.use_worktrees=false`.

- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` — 29 passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_ -- --nocapture` — 43 matching tests passed, including the Phase 8 schema contract.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed; the credentialed live smoke compiled but was not executed locally.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-09-30T10:31:30Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
