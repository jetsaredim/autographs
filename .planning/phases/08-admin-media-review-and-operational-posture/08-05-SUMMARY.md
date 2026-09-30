---
phase: 08-admin-media-review-and-operational-posture
plan: "05"
subsystem: database
tags: [rust, oracle, image-adjustments, catalog, edit-history]

requires:
  - phase: 08-admin-media-review-and-operational-posture
    provides: Plan 08-04 established validated adjustment DTOs and the catalog repository contract
provides:
  - Additive Oracle adjustment_json schema, idempotent migration, and preflight coverage
  - Typed Oracle image adjustment reads, writes, resets, and edit-history events
  - Replacement semantics that clear stale adjustment metadata for a new private original
affects: [phase-08, oracle-persistence, admin-media-review, publisher, media]

actuals:
  tokens: 3060
  tasks: 2
  commits: 4

tech-stack:
  added: []
  patterns:
    - Persist private image adjustment metadata as validated typed JSON in a nullable Oracle CLOB
    - Redact invalid persisted adjustment JSON at the adapter boundary
    - Clear adjustment metadata whenever private original metadata is replaced

key-files:
  created:
    - controller/db/updates/08-01-image-adjustments.sql
    - .planning/phases/08-admin-media-review-and-operational-posture/08-05-SUMMARY.md
  modified:
    - controller/db/schema.sql
    - controller/src/oracle_schema.rs
    - controller/src/oracle_catalog.rs

key-decisions:
  - "Use a nullable Oracle CLOB for adjustment_json and bind Option<String> so reset writes SQL NULL."
  - "Parse persisted adjustment JSON through ImageAdjustment::from_json and return a stable redacted adapter error for malformed values."
  - "Set adjustment_json to NULL in the same replacement metadata statement so new originals cannot inherit stale corrections."

patterns-established:
  - "Oracle image reads deserialize adjustment_json through the validated ImageAdjustment boundary."
  - "Oracle adjustment updates touch the owning item, append ImageAdjustmentChanged, commit, and reload the item."

requirements-completed: [MEDIA-06]

coverage:
  - id: D1
    description: "Oracle schema, migration, and preflight require autograph_images.adjustment_json."
    requirement: MEDIA-06
    verification:
      - kind: unit
        ref: "controller/src/oracle_schema.rs#phase8_preflight_requires_private_image_adjustment_metadata"
        status: pass
      - kind: other
        ref: "rg -n adjustment_json controller/db/schema.sql controller/db/updates/08-01-image-adjustments.sql controller/src/oracle_schema.rs"
        status: pass
    human_judgment: false
  - id: D2
    description: "Oracle persistence reads validated adjustment JSON and writes or resets it with item timestamp and edit-history semantics."
    requirement: MEDIA-06
    verification:
      - kind: unit
        ref: "controller/src/oracle_catalog.rs#oracle_adjustment_json_round_trips_and_rejects_invalid_values"
        status: pass
      - kind: integration
        ref: "controller/tests/admin_workflow.rs#memory_repository_saves_and_resets_image_adjustment_history"
        status: pass
      - kind: other
        ref: "cargo check --manifest-path controller/Cargo.toml --features production-persistence"
        status: pass
    human_judgment: false
  - id: D3
    description: "Replacing a private original clears persisted adjustment metadata."
    requirement: MEDIA-06
    verification:
      - kind: unit
        ref: "controller/src/oracle_catalog.rs#oracle_adjustment_mutations_persist_updates_and_clear_replacements"
        status: pass
      - kind: integration
        ref: "controller/tests/admin_workflow.rs#replacing_image_preserves_id_and_clears_stale_adjustment"
        status: pass
    human_judgment: false

duration: 7min
completed: 2026-09-30
status: complete
---

# Phase 08 Plan 05: Oracle Image Adjustment Persistence Summary

**Validated private image adjustments now persist through Oracle with additive schema migration, edit-history tracking, reset support, and stale-correction clearing on replacement.**

## Performance

- **Duration:** 7min active retry execution
- **Started:** 2026-09-30T01:40:31Z
- **Completed:** 2026-09-30T01:47:30Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `autograph_images.adjustment_json` to the end-state schema, an idempotent live update, and production schema preflight.
- Implemented typed Oracle adjustment JSON serialization/deserialization, save/reset behavior, item timestamp updates, and `ImageAdjustmentChanged` history events.
- Made private-original replacement clear `adjustment_json` atomically so prior corrections cannot affect new media.

## Task Commits

1. **Task 1 RED: Add failing Oracle adjustment schema test** - `3344ce7` (test)
2. **Task 1 GREEN: Add Oracle adjustment schema support** - `315317f` (feat)
3. **Task 2 RED: Define Oracle adjustment persistence contract** - `aac2f84` (test)
4. **Task 2 GREEN: Persist Oracle image adjustments** - `09bdb6a` (feat)

**Plan metadata:** Pending closeout commit.

## Files Created/Modified

- `controller/db/updates/08-01-image-adjustments.sql` - Adds the nullable adjustment CLOB only when absent.
- `controller/db/schema.sql` - Declares the end-state image adjustment column.
- `controller/src/oracle_schema.rs` - Requires the Phase 8 column during production preflight and tests the schema contract.
- `controller/src/oracle_catalog.rs` - Maps, validates, saves, resets, and clears Oracle adjustment metadata.

## Decisions Made

- Used a nullable CLOB with `Option<String>` binding to represent reset as SQL `NULL` without a separate sentinel value.
- Kept malformed persisted JSON errors stable and redacted while retaining validation at both serialization and deserialization boundaries.
- Included `adjustment_json = null` directly in replacement SQL to make stale-state clearing part of the replacement transaction.

## Deviations from Plan

None - plan scope and persistence contract were implemented as specified. The already-established admin workflow tests from Plan 08-04 supplied the requested credential-free save/reset/replacement behavior coverage; Plan 08-05 added Oracle-specific unit contract tests alongside them.

## Issues Encountered

- Two delegated executor attempts stopped after creating partial RED-test work. The valid Task 1 RED commit was retained, the duplicate partial edit was reconciled, and execution completed inline without losing task lineage.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence phase8_preflight_requires_private_image_adjustment_metadata -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_adjustment -- --nocapture` passed (2 tests).
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` passed (29 tests).
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` passed.
- Adjustment schema/source assertions and `git diff --check` passed.
- The generic post-merge build/test detector found no root-level configured build or test command; the plan-specific Rust gates above were used.

## Known Stubs

None.

## Threat Flags

None. Adjustment payloads cross Oracle boundaries only through validated typed serialization; malformed persisted values return a redacted error, and save/reset actions are recorded in edit history.

## User Setup Required

None - the migration must be applied through the existing database update procedure when the phase is deployed.

## Next Phase Readiness

Plan 08-06 can build the private admin preview/adjustment interface on the now-persistent Oracle contract. Phase-level verification remains deferred because this wave-filtered run leaves Plans 08-06 through 08-08 incomplete.

## Self-Check: PASSED

- The migration file exists and all four implementation commits are present in git history.
- Schema, preflight, typed JSON mapping, update/reset, history, and replacement-clearing criteria passed.
- Production-persistence compilation and the credential-free admin workflow suite passed.

---
*Phase: 08-admin-media-review-and-operational-posture*
*Completed: 2026-09-30*
