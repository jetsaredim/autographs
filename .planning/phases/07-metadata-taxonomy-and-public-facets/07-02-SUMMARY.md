---
phase: 07-metadata-taxonomy-and-public-facets
plan: "02"
subsystem: database
tags: [rust, oracle, taxonomy, signer-profiles, backfill, plsql]

requires:
  - phase: 07-metadata-taxonomy-and-public-facets
    provides: 07-01 additive signer/taxonomy schema, domain model, memory repository behavior, and schema preflight
provides:
  - Oracle persistence for signer profiles, item signer credits, characters, franchises, item taxonomy fields, signer suggestions, profile edits, merge repair, and taxonomy suggestions
  - repository-level signer duplicate warnings, profile edit history, and merge history behavior
  - repeatable Phase 7 taxonomy mapping, legacy export fixture, review SQL, generated PL/SQL, and no-new-dependency backfill CLI
affects: [phase-07-admin-taxonomy-ui, phase-07-public-facets, phase-08-ai-assisted-ingest]

tech-stack:
  added: []
  patterns:
    - TDD contracts for Oracle persistence, signer repair behavior, and taxonomy backfill artifacts
    - repository-level signer repair APIs shared by memory and Oracle adapters
    - no-new-dependency Rust CLI using std::env::args with serde_json parsing

key-files:
  created:
    - controller/src/taxonomy_migration.rs
    - controller/src/bin/taxonomy_backfill.rs
    - controller/db/updates/07-02-taxonomy-backfill-review.sql
    - controller/db/updates/07-03-taxonomy-backfill-apply.sql
    - controller/fixtures/taxonomy-legacy-export.json
    - controller/tests/taxonomy_migration.rs
    - .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json
    - .planning/phases/07-metadata-taxonomy-and-public-facets/07-02-SUMMARY.md
  modified:
    - controller/src/catalog.rs
    - controller/src/oracle_catalog.rs
    - controller/src/lib.rs
    - controller/tests/admin_workflow.rs

key-decisions:
  - "Oracle now writes legacy signer as compact signer text and legacy category as format while Phase 7 keeps rollback/reference fields."
  - "Signer profile edits and merges record item-level metadataUpdated events for every linked item."
  - "Likely duplicate physical items stay report-only in backfill output and are not auto-merged by generated PL/SQL."

patterns-established:
  - "Signer suggestion ranking uses exact match, prefix match, then near match while keeping deliberate new signer creation unblocked."
  - "Backfill artifacts are generated from a committed temporary mapping file and scanned for private storage/credential terms."

requirements-completed: [DATA-03, ADMIN-02, ADMIN-03]

duration: 15min
completed: 2026-07-09
---

# Phase 07-02: Oracle Taxonomy Persistence and Backfill Summary

**Oracle signer/taxonomy persistence with duplicate-aware signer repair and repeatable reviewed backfill artifacts**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-09T14:14:02Z
- **Completed:** 2026-07-09T14:29:08Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Wired Oracle create/update/load/list paths to persist signer profiles, item signer credits, characters, franchises, format, origin, language, product line, and set name.
- Added repository APIs for signer suggestions, taxonomy suggestions, profile edits, and merge repair with DATA-03 metadata history for linked items.
- Added a deterministic Phase 7 backfill mapping, sanitized legacy fixture, read-only review SQL, generated apply PL/SQL, and `taxonomy_backfill` CLI.

## Task Commits

1. **Task 1 RED: Oracle taxonomy persistence contract** - `f0b2db8` (test)
2. **Task 1 GREEN: Persist Oracle signer taxonomy joins** - `25d8473` (feat)
3. **Task 2 RED: Signer repair repository tests** - `f5f0efb` (test)
4. **Task 2 GREEN: Signer suggestions and merge repair** - `da37966` (feat)
5. **Task 3 RED: Taxonomy backfill tests** - `d8d835c` (test)
6. **Task 3 GREEN: Taxonomy backfill artifacts** - `67a559d` (feat)

## Files Created/Modified

- `controller/src/catalog.rs` - Added signer suggestion/profile update/merge/taxonomy suggestion contracts and memory behavior.
- `controller/src/oracle_catalog.rs` - Added Oracle join-table persistence, signer suggestions, profile edits, merge repair, and taxonomy suggestions.
- `controller/src/taxonomy_migration.rs` - Backfill mapping/report/PLSQL generation logic.
- `controller/src/bin/taxonomy_backfill.rs` - CLI for `report` and `plsql` modes.
- `controller/db/updates/07-02-taxonomy-backfill-review.sql` - Read-only SQL Developer review artifact.
- `controller/db/updates/07-03-taxonomy-backfill-apply.sql` - Operator-reviewable generated PL/SQL artifact.
- `controller/fixtures/taxonomy-legacy-export.json` - Sanitized representative legacy export fixture.
- `controller/tests/admin_workflow.rs` - Signer suggestions, profile edit history, merge, and taxonomy suggestion tests.
- `controller/tests/taxonomy_migration.rs` - Backfill report, PL/SQL mapping/privacy, and read-only review SQL tests.
- `controller/src/lib.rs` - Exports the taxonomy migration module.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json` - Temporary Phase 7 mapping artifact.

## Decisions Made

- Kept legacy `autograph_items.signer` and `category` populated during Oracle writes as Phase 7 rollback/reference fields, with `category` mirroring `format`.
- Used repository-level signer repair APIs rather than route-specific logic so Phase 7 admin UI and future ingest work share the same behavior.
- Kept duplicate physical-item handling report-only in the backfill pipeline per D-07-19.

## Deviations from Plan

None - plan executed exactly as written.

**Total deviations:** 0 auto-fixed.
**Impact on plan:** No scope changes or unplanned dependency additions.

## Issues Encountered

- `cargo fmt --check` found wrapping-only formatting changes after implementation; rustfmt was applied and checks passed.
- Privacy/read-only scans use `rg` no-match exit code `1` as the expected successful result for forbidden-token absence.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed.
- `cargo test --manifest-path controller/Cargo.toml --test taxonomy_migration -- --nocapture` - passed, 3 tests.
- `cargo test --manifest-path controller/Cargo.toml signer -- --nocapture` - passed, 3 matching signer tests.
- `cargo test --manifest-path controller/Cargo.toml profile -- --nocapture` - passed during Task 2 verification.
- `cargo test --manifest-path controller/Cargo.toml merge -- --nocapture` - passed during Task 2 verification.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` - passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_catalog -- --nocapture` - passed during Task 1 verification.
- `cargo run --manifest-path controller/Cargo.toml --bin taxonomy_backfill -- report --mapping .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json --input controller/fixtures/taxonomy-legacy-export.json --out /tmp/taxonomy-backfill-report.md` - passed.
- `cargo run --manifest-path controller/Cargo.toml --bin taxonomy_backfill -- plsql --mapping .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json --input controller/fixtures/taxonomy-legacy-export.json --out /tmp/taxonomy-backfill.sql` - passed.
- `/tmp/taxonomy-backfill-report.md` contains `Mapped`, `Needs review`, and `Report only` sections.
- `/tmp/taxonomy-backfill.sql` maps `custom` to `origin = 'Custom'` and maps `Tr`, `Tra`, and `Trading Card` to `format = 'Trading Card'`.
- Generated SQL scans found no Object Storage URLs, bucket names, object keys, Oracle connection strings, credential-like tokens, or private keys.
- `07-02-taxonomy-backfill-review.sql` contains no mutating `update`, `insert`, `merge`, or `delete` statements outside comments.

## Known Stubs

None - stub scan found no TODO/FIXME/placeholder or hardcoded-empty UI data paths in created/modified files.

## Threat Flags

None - the new Oracle mutation paths, signer merge/profile history, and generated SQL surfaces are covered by the plan threat model and mitigations.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 7 admin and public-surface plans can now rely on repository-level signer reuse, duplicate suggestions, profile update/merge repair, taxonomy suggestions, and reviewed backfill artifacts. Live rollout should still follow the planned schema/data-first flow: review report and PL/SQL, apply intentionally in SQL Developer when appropriate, deploy app changes, then run a full static publish.

## Self-Check: PASSED

- Key files exist: `controller/src/taxonomy_migration.rs`, `controller/src/bin/taxonomy_backfill.rs`, `controller/db/updates/07-02-taxonomy-backfill-review.sql`, `controller/db/updates/07-03-taxonomy-backfill-apply.sql`, `controller/fixtures/taxonomy-legacy-export.json`, `.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json`, and this summary.
- Task commits exist: `f0b2db8`, `25d8473`, `f5f0efb`, `da37966`, `d8d835c`, and `67a559d`.
- Plan-level verification commands passed.
- No accidental file deletions were detected across the task commits.

---
*Phase: 07-metadata-taxonomy-and-public-facets*
*Completed: 2026-07-09*
