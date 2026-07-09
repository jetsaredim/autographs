---
phase: 07-metadata-taxonomy-and-public-facets
plan: "01"
subsystem: database
tags: [rust, oracle, taxonomy, signer-profiles, edit-history]

requires:
  - phase: 06-admin-collection-workflow
    provides: admin workflow domain model, edit-history behavior, media cleanup tests, and production persistence shape to extend
provides:
  - additive Oracle schema for reusable signers, signer credits, characters, franchises, and item taxonomy fields
  - Rust catalog domain types for signer profiles, signer credits, item origin, taxonomy suggestions, and signer-name normalization
  - in-memory repository validation, signer profile reuse, duplicate-credit rejection, taxonomy defaults, and edit-history diffs
  - Oracle schema preflight checks for Phase 7 tables and required columns
affects: [phase-07-public-facets, phase-07-admin-taxonomy-ui, phase-08-ai-assisted-ingest]

tech-stack:
  added: []
  patterns:
    - additive Oracle update scripts with idempotent PL/SQL guards
    - Rust domain-first taxonomy defaults with legacy signer/category compatibility
    - TDD contract tests for schema artifacts, catalog history behavior, and schema preflight

key-files:
  created:
    - controller/db/updates/07-01-taxonomy-schema.sql
    - .planning/phases/07-metadata-taxonomy-and-public-facets/07-01-SUMMARY.md
  modified:
    - controller/db/schema.sql
    - controller/src/catalog.rs
    - controller/src/oracle_catalog.rs
    - controller/src/oracle_schema.rs
    - controller/tests/admin_workflow.rs
    - controller/tests/media_cleanup.rs
    - controller/tests/publisher.rs
    - controller/tests/seed_content.rs

key-decisions:
  - "Phase 7 schema changes are additive and retain legacy signer, category, and autograph_item_tags through migration."
  - "Memory repository signer credits reuse normalized signer profiles and derive a legacy credit from signer when older inputs omit signerCredits."
  - "Oracle persistence keeps a legacy taxonomy fallback in this plan; later Phase 7 adapter work should wire full signer/taxonomy table reads and writes."

patterns-established:
  - "TDD RED commits capture failing schema/domain/preflight contracts before each implementation commit."
  - "New taxonomy edit-history diffs use exact public/admin field names: signers, characters, franchises, productLine, setName, format, origin, and language."

requirements-completed: [DATA-03, ADMIN-02, ADMIN-03]

duration: 14min
completed: 2026-07-09
---

# Phase 07-01: Metadata Taxonomy Foundation Summary

**Oracle signer/taxonomy schema plus Rust catalog validation and edit-history diffs for reusable signer credits**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-09T13:55:44Z
- **Completed:** 2026-07-09T14:09:20Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Added the Phase 7 Oracle schema foundation: signer profiles, item signer credits, character/franchise join tables, item taxonomy columns, constraints, and indexes.
- Added Rust catalog domain types and in-memory behavior for signer profile reuse, signer credit validation, taxonomy defaults, duplicate-credit rejection, and metadata history diffs.
- Extended Oracle schema preflight so production startup fails closed when Phase 7 taxonomy tables or required columns are missing.

## Task Commits

1. **Task 1 RED: Add failing schema taxonomy contract** - `96b2bbc` (test)
2. **Task 1 GREEN: Add additive taxonomy schema** - `9ec67c6` (feat)
3. **Task 2 RED: Add failing taxonomy history tests** - `0ca8401` (test)
4. **Task 2 GREEN: Extend catalog taxonomy model** - `d869fab` (feat)
5. **Task 3 RED: Add failing schema preflight test** - `88d4e21` (test)
6. **Task 3 GREEN: Extend Oracle schema preflight** - `df0bd2a` (feat)

## Files Created/Modified

- `controller/db/schema.sql` - End-state Oracle schema with signer/taxonomy tables, fields, constraints, and indexes.
- `controller/db/updates/07-01-taxonomy-schema.sql` - Additive live update script for Phase 7 taxonomy storage.
- `controller/src/catalog.rs` - Signer/taxonomy domain types, validation, memory repository behavior, defaults, normalization, and edit-history diffs.
- `controller/src/oracle_catalog.rs` - Legacy-compatible production-persistence fallback for the expanded catalog model.
- `controller/src/oracle_schema.rs` - Phase 7 schema preflight constants, table discovery, and schema/update-script tests.
- `controller/tests/admin_workflow.rs` - Taxonomy deserialization and metadata history tests plus updated fixtures.
- `controller/tests/media_cleanup.rs` - Fixture defaults for expanded catalog item structs.
- `controller/tests/publisher.rs` - Fixture defaults for expanded catalog item structs.
- `controller/tests/seed_content.rs` - Fixture defaults for expanded catalog item structs.

## Decisions Made

- Kept legacy `signer`, `category`, and `autograph_item_tags` intact for Phase 7 migration safety.
- Defaulted legacy item creation into a single derived signer credit when `signerCredits` is omitted, preserving existing admin/API paths while ensuring stored memory items have at least one credit.
- Kept full Oracle taxonomy persistence out of this plan; schema/preflight are ready, while later Phase 7 plans should wire Oracle adapter reads/writes for the new tables.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Kept production-persistence compiling after shared catalog DTO expansion**
- **Found during:** Task 2 (Extend catalog domain and memory repository)
- **Issue:** Adding new `AutographItem` fields and changing `apply_update` broke `controller/src/oracle_catalog.rs` under `production-persistence`.
- **Fix:** Added a minimal legacy signer/taxonomy fallback and passed `None` for unresolved signer-credit updates until later Oracle adapter work wires the Phase 7 tables.
- **Files modified:** `controller/src/oracle_catalog.rs`
- **Verification:** `cargo check --manifest-path controller/Cargo.toml --features production-persistence` passed.
- **Committed in:** `d869fab`

**2. [Rule 3 - Blocking] Updated direct test fixtures for expanded catalog structs**
- **Found during:** Task 2 (Extend catalog domain and memory repository)
- **Issue:** Integration tests outside `admin_workflow` constructed `AutographItemInput` and `AutographItem` directly, so the shared DTO expansion prevented the full controller suite from compiling.
- **Fix:** Added legacy-compatible taxonomy defaults to publisher, media cleanup, and seed content fixtures.
- **Files modified:** `controller/tests/publisher.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/seed_content.rs`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml -- --nocapture` passed.
- **Committed in:** `d869fab`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were compatibility work directly caused by the Task 2 shared model change. No new dependency, endpoint, or public UI scope was added.

## Issues Encountered

- The initial schema test filter did not run Oracle schema unit tests until `--features production-persistence` was included. The RED gate was rerun with the correct feature before committing.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` - passed, 14 tests
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_schema -- --nocapture` - passed, 5 matching tests
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` - passed
- `cargo test --manifest-path controller/Cargo.toml -- --nocapture` - passed before Task 3 preflight-only changes

## Known Stubs

- `controller/src/oracle_catalog.rs` currently maps Oracle-loaded legacy signer/category rows into default in-memory taxonomy values. This is intentional compatibility for Plan 07-01; later Phase 7 Oracle adapter work should persist and load `autograph_signers`, `autograph_item_signers`, `autograph_item_characters`, and `autograph_item_franchises`.

## Threat Flags

None - the new SQL schema, admin-input validation, edit-history diffs, and fail-closed Oracle preflight were all covered by the plan threat model.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The schema and Rust domain foundation are ready for the next Phase 7 plans to wire admin DTO/routes, Oracle persistence joins, static public facets, public detail rendering, and reviewed backfill tooling. Production deploys should apply `controller/db/updates/07-01-taxonomy-schema.sql` before a controller that requires the Phase 7 preflight runs against live Oracle.

## Self-Check: PASSED

- Key files exist: `controller/db/updates/07-01-taxonomy-schema.sql`, `controller/db/schema.sql`, `controller/src/catalog.rs`, `controller/src/oracle_schema.rs`, and this summary.
- Task commits exist: `96b2bbc`, `9ec67c6`, `0ca8401`, `d869fab`, `88d4e21`, and `df0bd2a`.
- Plan-level verification commands passed.
- Stub scan found only the documented Oracle legacy taxonomy fallback; no blocking placeholder UI or mock-data path prevents Plan 07-01 success.
- No accidental file deletions were detected after task commits.

---
*Phase: 07-metadata-taxonomy-and-public-facets*
*Completed: 2026-07-09*
