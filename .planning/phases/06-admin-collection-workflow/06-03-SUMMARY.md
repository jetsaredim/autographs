---
phase: 06-admin-collection-workflow
plan: 03
subsystem: api
tags: [rust, axum, oracle, object-storage, media-cleanup]

requires:
  - phase: 06-02
    provides: private admin item APIs, item detail responses, and edit-history foundations
provides:
  - multi-image upload semantics with explicit primary selection
  - image deletion and replacement APIs that coordinate private media and catalog metadata
  - durable cleanup warnings and retry status for private object deletion failures
  - Oracle schema and preflight coverage for cleanup events
affects: [phase-06-admin-ui, media-cleanup, oracle-schema, live-smoke]

tech-stack:
  added: []
  patterns:
    - controller-coordinated private media cleanup before/after repository metadata transitions
    - redacted cleanup warning DTOs for retryable admin maintenance

key-files:
  created:
    - controller/tests/media_cleanup.rs
  modified:
    - controller/db/schema.sql
    - controller/src/catalog.rs
    - controller/src/oracle_schema.rs
    - controller/src/oracle_catalog.rs
    - controller/src/routes.rs
    - controller/tests/admin_workflow.rs
    - controller/tests/live_persistence_smoke.rs

key-decisions:
  - "Replacement uses a fresh UUID-backed object and metadata row identity, leaving failed old-object cleanup retryable by the old image ID."
  - "Cleanup warnings expose only redacted admin messages and image IDs; private object keys, bucket names, namespaces, and filenames stay out of admin responses."

patterns-established:
  - "Delete path removes private media before metadata; failed media delete keeps metadata attached and records a retryable warning."
  - "Replacement path writes new private media first, swaps metadata, rolls back the new object if metadata update fails, then records cleanup warning only if old-object delete fails."

requirements-completed: [MEDIA-04, ADMIN-02, ADMIN-03, DATA-03]

duration: 45min
completed: 2026-06-26
---

# Phase 06-03: Multi-Image Maintenance and Cleanup Summary

**Retryable admin image cleanup with redacted warnings, primary-image selection, Oracle cleanup events, and live-smoke cleanup guidance**

## Performance

- **Duration:** 45 min
- **Started:** 2026-06-26T20:30:00-04:00
- **Completed:** 2026-06-26T21:15:00-04:00
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added explicit primary-image selection and preserved current primary images when supporting uploads omit `isPrimary`.
- Added image delete, replace, and cleanup retry routes under `/admin/api/items/{id}/images/{image_id}`.
- Added `autograph_cleanup_events` plus memory/Oracle repository support for durable retryable cleanup warnings.
- Extended local media cleanup coverage for delete success, delete failure, replacement rollback, retry idempotence, and redacted warning responses.
- Updated the ignored live persistence smoke to check cleanup-event schema and include cleanup/list guidance for cleanup warning rows.

## Task Commits

1. **Task 1: Add primary-image and multi-upload semantics**
   - `b1b656f` test(06-03): add failing primary image workflow coverage
   - `bf5c45d` feat(06-03): support explicit primary image selection
2. **Task 2: Coordinate image deletion, replacement, cleanup warnings, and retry**
   - `a01821a` test(06-03): add failing media cleanup coverage
   - `69251d5` feat(06-03): add retryable media cleanup operations

## Files Created/Modified

- `controller/tests/media_cleanup.rs` - Focused regression coverage for primary selection, delete, replace rollback, cleanup failure redaction, and retry.
- `controller/db/schema.sql` - Adds `autograph_cleanup_events` and `autograph_cleanup_events_item_status_idx`.
- `controller/src/catalog.rs` - Adds cleanup domain types and repository methods; implements memory metadata cleanup/replacement and warning tracking.
- `controller/src/oracle_schema.rs` - Adds cleanup-event table/column preflight checks.
- `controller/src/oracle_catalog.rs` - Implements Oracle metadata delete/replace, cleanup event insert/query, and retry status updates.
- `controller/src/routes.rs` - Adds primary, delete, replace, and `/cleanup/retry` image routes with redacted cleanup warning responses.
- `controller/tests/live_persistence_smoke.rs` - Extends ignored live smoke schema/list/cleanup checks for cleanup-event rows.
- `controller/tests/admin_workflow.rs` - Uses a generated valid PNG fixture for upload-pending coverage under stricter image validation.

## Decisions Made

- Retry events do not store or return private object keys. Replacement cleanup retries recompute the old private object path from the old image ID and item ID.
- Delete failures return `409 Conflict` with a small `cleanupWarning` envelope and keep image metadata attached so the admin can retry safely.
- Replacement metadata swaps preserve primary/sort semantics while assigning a fresh image/object UUID for the new original.

## Deviations from Plan

### Auto-Fixed Issues

**1. [Rule 3 - Blocking] Existing admin workflow upload fixture failed stricter image validation**
- **Found during:** Plan verification (`cargo test --manifest-path controller/Cargo.toml`)
- **Issue:** `controller/tests/admin_workflow.rs` used a hard-coded PNG byte array that `image` rejected, causing upload-pending coverage to fail with 400.
- **Fix:** Replaced the byte array with the same generated PNG fixture pattern used by media cleanup tests.
- **Files modified:** `controller/tests/admin_workflow.rs`
- **Verification:** Full `cargo test --manifest-path controller/Cargo.toml` passed.
- **Committed in:** `69251d5`

---

**Total deviations:** 1 auto-fixed (1 blocking test fixture)
**Impact on plan:** No scope change; the fix keeps existing admin workflow coverage compatible with the current upload validation path.

## Issues Encountered

- The GSD invocation used `6.3`, which does not resolve as a phase directory in this project. Execution resumed Phase 06 Plan 03 from the handoff state and existing Task 1 commits.
- The safe-resume gate detected production commits for 06-03 without a summary. The handoff explicitly identified Task 1 as complete and Task 2 as remaining, so Task 2 was completed inline and this summary closes the partial-plan state.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed
- `cargo test --manifest-path controller/Cargo.toml --test media_cleanup -- --nocapture` - passed, 5 tests
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` - passed, 8 tests
- `cargo test --manifest-path controller/Cargo.toml` - passed, full controller suite
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` - passed

## User Setup Required

None - no external service configuration required. Live Oracle/Object Storage cleanup verification remains an ignored operator-run smoke path.

## Next Phase Readiness

Plan 06-04 can build publish batching, pending-change diagnostics, and release retention on top of durable image/history events and retryable cleanup warnings. No blockers remain from 06-03.

## Self-Check: PASSED

- Key files exist, including `controller/tests/media_cleanup.rs` and this summary.
- `controller/src/routes.rs` contains `/admin/api/items/{id}/images/{image_id}/cleanup/retry`.
- `controller/db/schema.sql` contains `create table autograph_cleanup_events`.
- Cleanup warning responses are covered by tests that deny private storage identifiers and original filenames.
- All plan verification commands passed.

---
*Phase: 06-admin-collection-workflow*
*Completed: 2026-06-26*
