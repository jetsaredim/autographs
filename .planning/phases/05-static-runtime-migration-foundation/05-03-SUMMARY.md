---
phase: 05-static-runtime-migration-foundation
plan: 03
subsystem: private-content-seed
tags: [rust, axum, oracle, odpi-c, instant-client, object-storage, s3, podman]
requires:
  - phase: 05-02
    provides: authenticated Rust controller boundary
provides:
  - Protected minimal item create, update, publication, and private original upload APIs
  - UUID-only private original object keys with private original filename metadata
  - Static-runtime Oracle migration and credential-gated live persistence smoke
  - Temporary Oracle Instant Client smoke container for VM-local verification
affects: [05-04-static-publisher, 05-05-static-admin, 05-06-runtime-cutover]
tech-stack:
  added: [oracle, odpi-c, oracle-instant-client, rust-s3]
  patterns: [uuid-only original keys, redacted admin responses, vm-local credential-gated smoke, schema preflight before mutation]
key-files:
  created:
    - app/db/migrations/002_static_runtime_foundation.sql
    - controller/src/catalog.rs
    - controller/src/media.rs
    - controller/src/storage_keys.rs
    - controller/tests/live_persistence_smoke.rs
    - controller/Dockerfile.smoke
  modified:
    - controller/src/routes.rs
    - docs/static-runtime-runbook.md
key-decisions:
  - "Use UUID-only originals/{item_id}/{image_id} private object keys and store source filenames only as private metadata."
  - "Use the native oracle crate with ODPI-C and Oracle Instant Client after the pure-Rust oracle-rs spike failed against ADB."
  - "Run the live smoke inside a temporary VM-local Podman container so Oracle wallet and private network behavior match production."
patterns-established:
  - "Private seed APIs return redacted item/image responses without bucket, namespace, object key, Object Storage URL, or original filename leakage."
  - "Live persistence verification checks schema readiness before mutation and logs generated cleanup coordinates before writing."
requirements-completed: [STATIC-02, STATIC-03, STATIC-05]
duration: 1h 45m
completed: 2026-06-02
---

# Phase 05 Plan 03: Private Content Seed Path Summary

**Authenticated Rust seed APIs with UUID-only private media keys and a VM-proven Oracle Instant Client/Object Storage persistence path**

## Performance

- **Duration:** 1h 45m
- **Tasks:** 4
- **Files modified:** 13

## Accomplishments

- Added protected item create, update, publication, and private image upload routes backed by local repository/media abstractions.
- Added schema migration `002_static_runtime_foundation.sql` for private original filenames, publish jobs, and public derivative accounting.
- Proved the mandatory live path on the OCI VM: Oracle connect, draft item insert/read/delete, private Object Storage upload/read/delete, and UUID-only key behavior.

## Task Commits

1. **Task 1: Add schema migration for static foundation metadata** - `70533c5`
2. **Task 2: Add local catalog and private media abstractions** - `93ebeb7`
3. **Task 3: Add protected private seed endpoints** - `f3b4b44`
4. **Task 4: Add and prove credential-gated live persistence smoke** - `535ae1c`, `9328c75`, `16b7009`, `f10ad43`, `698de80`

## Decisions Made

- Replaced the experimental pure-Rust `oracle-rs` path with the native `oracle` crate after a real ADB listener refusal showed the thin implementation was not a viable production foundation.
- Package Oracle Instant Client in the controller smoke container and use the same `ORACLE_DB_CONNECT_STRING=autographsdb_medium` wallet alias as the deployed app.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced incompatible pure-Rust Oracle spike**
- **Found during:** Task 4 live VM proof
- **Issue:** `oracle-rs` reached the ADB listener but received a TNS refusal with reduced diagnostic detail.
- **Fix:** Switched the spike to `oracle`, ODPI-C, and Oracle Instant Client with mounted-wallet alias resolution.
- **Verification:** VM live smoke passed against ADB and OCI Object Storage.
- **Committed in:** `f10ad43`

**2. [Rule 3 - Blocking] Selected an explicit rustls provider**
- **Found during:** Task 4 live VM proof
- **Issue:** S3 dependency paths enabled multiple rustls crypto providers.
- **Fix:** Explicitly install the `aws-lc-rs` provider before live network work.
- **Verification:** Container guard path and live Object Storage upload/read/delete passed.
- **Committed in:** `16b7009`, retained by `f10ad43`

**3. [Rule 2 - Missing Critical] Added stale-schema preflight and cleanup logging**
- **Found during:** Task 4 live VM proof
- **Issue:** A missing `002` migration allowed mutation to begin before the image metadata insert failed.
- **Fix:** Verify `ORIGINAL_FILENAME` exists before mutation and log generated item/object coordinates.
- **Verification:** Controller test suite and rebuilt smoke-container guard path pass.
- **Committed in:** `698de80`

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical).
**Impact on plan:** The durable Rust direction remains intact and now uses the mature Oracle native client path.

## Issues Encountered

- The downloaded ADB wallet `sqlnet.ora` needed its wallet directory placeholder replaced with `/opt/autographs/wallet` for the native smoke container.
- Live ADB migration `002 static_runtime_foundation` was applied before the successful proof.

## User Setup Required

- Revoke the temporary OCI S3 Customer Secret Key after the live checkpoint work is complete.
- Clean the one orphaned draft item/object from the pre-migration failed run if it has not already been removed.

## Next Phase Readiness

- `05-04` can build static candidates and derivatives on top of the proven private source-of-truth write path.
- The future deployed controller container must include Oracle Instant Client and wrap blocking Oracle operations appropriately.

## Self-Check: PASSED

- `cargo test --manifest-path controller/Cargo.toml`
- VM-local `AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true` Podman smoke passed on 2026-06-02.

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-02*
