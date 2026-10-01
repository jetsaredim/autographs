---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-30T10:11:28Z
depth: deep
files_reviewed: 4
files_reviewed_list:
  - controller/db/schema.sql
  - controller/db/updates/08-01-image-adjustments.sql
  - controller/src/oracle_schema.rs
  - controller/src/oracle_catalog.rs
findings:
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 08 Plan 05: Code Review Report

**Reviewed:** 2026-09-30T10:11:28Z
**Depth:** deep
**Files Reviewed:** 4
**Status:** issues_found

## Summary

The Oracle adjustment implementation is not shippable yet. Every save or reset attempts to append an `imageAdjustmentChanged` edit event, but both the fresh-schema constraint and the live migration reject that value, so the transaction cannot complete. In addition, several image mutation methods commit before they deserialize and return the owning item; because persisted adjustment JSON is explicitly untrusted, a malformed sibling image can make those methods report failure after their metadata and history changes have already committed. The current tests pass because the Oracle-specific mutation test only searches SQL strings and the workflow tests exercise the memory repository.

Finding lineage starts here as review round 1: CR-01, CR-02, and WR-01 are independent findings. No prior PR #261 review finding was supplied for recurrence classification.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Oracle rejects every image-adjustment history event

**Classification:** BLOCKER (Critical tier)
**Lineage:** Round 1 — independent finding.
**Shared invariant:** Every `EditEventKind` emitted by a production mutation must be admitted by the end-state Oracle schema, installed by the live migration, and required by schema preflight; otherwise the mutation and its audit record cannot commit atomically.
**Coverage inspected:** The producer is `EditEventKind::ImageAdjustmentChanged` and `update_image_adjustment`; the mutation boundary is `insert_edit_event` followed by the transaction commit; consumers are history, pending-change, and publish-boundary queries; the fresh-schema and live-migration paths are `schema.sql` and `08-01-image-adjustments.sql`; preflight currently checks only `cleanupChanged`; the relevant Oracle tests assert SQL fragments while the workflow tests use `MemoryCatalogRepository`.
**Files:** `controller/db/schema.sql:158-168`, `controller/db/updates/08-01-image-adjustments.sql:8-24`, `controller/src/oracle_schema.rs:46-52`, `controller/src/oracle_catalog.rs:629-639`
**Issue:** `update_image_adjustment` serializes and updates the CLOB, then inserts an event whose type is `imageAdjustmentChanged`. `AUTOGRAPH_EDIT_EVENTS_TYPE_CK` does not include that value in the canonical schema, and the Phase 8 migration only adds `adjustment_json`; it never replaces the constraint. Oracle therefore rejects the event insert with a check-constraint violation on both fresh and upgraded databases, preventing save/reset from reaching `commit()`. Preflight still passes because it only looks for `cleanupChanged`, so deployment does not fail closed before serving the broken repository method.
**Fix:** Add the new event type to the canonical constraint, make the Phase 8 migration idempotently replace the existing constraint with the full value set, and require the new value in `REQUIRED_CHECK_CONSTRAINTS` with the Phase 8 migration as its remediation script. For example:

```sql
alter table autograph_edit_events drop constraint autograph_edit_events_type_ck;

alter table autograph_edit_events add constraint autograph_edit_events_type_ck
  check (event_type in (
    'created', 'metadataUpdated', 'imageAdded', 'imageRemoved',
    'imageReplaced', 'imageAdjustmentChanged', 'primaryImageChanged',
    'publicationChanged', 'cleanupChanged'
  ));
```

The migration should retain the existing existence guard before dropping the constraint, and schema/preflight tests should assert `imageAdjustmentChanged`, not just the new column.

### CR-02: Invalid persisted adjustment JSON can produce a committed write reported as failure

**Classification:** BLOCKER (Critical tier)
**Lineage:** Round 1 — independent finding.
**Shared invariant:** A repository mutation that returns an error must not have committed its metadata, item timestamp, or edit-history event; untrusted persisted CLOB data must be validated before the commit boundary that makes a mutation externally durable.
**Coverage inspected:** Adjustment JSON is produced by the typed serializer or can already exist in Oracle; every image row is consumed by `image_from_row`/`deserialize_image_adjustment`; `create`, item update, and attach validate the item before mutation, while set-primary, remove, replace, and adjustment-update commit and only then reload the complete item; the returned error is propagated to action callers even though pending-change/history rows are already durable; current tests cover pure JSON helpers and memory behavior but no malformed Oracle sibling row or transaction outcome.
**File:** `controller/src/oracle_catalog.rs:487-640` (post-commit reloads), `controller/src/oracle_catalog.rs:1846-1881` (fallible CLOB deserialization)
**Issue:** `update_image_adjustment` commits at lines 637-639 and calls `load_item` at line 640. `load_item` deserializes every image belonging to the item, so malformed `adjustment_json` on a different image returns the redacted adapter error after the target update, parent timestamp, and `ImageAdjustmentChanged` event are already committed. The same ordering affects set-primary, remove, and replacement. A caller sees failure and can retry an operation that already happened, creating duplicate audit events or additional state changes. This is a direct consequence of treating Oracle CLOB contents as untrusted but validating them only after commit.
**Fix:** Reload and validate the complete result inside the transaction before committing, then return the already-loaded value. Explicitly roll back on reload/validation failure (or use a transaction helper that guarantees rollback). Apply the same ordering to all four affected image mutation paths. For example:

```rust
insert_edit_event(&connection, &event)?;
let updated = load_item(&connection, item_id)?
    .ok_or_else(|| "autograph item was not found".to_owned())?;
connection
    .commit()
    .map_err(|error| format!("commit Oracle image adjustment metadata: {error}"))?;
Ok(updated)
```

Add a regression case with a malformed adjustment CLOB on a sibling image and prove that the target row, item timestamp, and history count remain unchanged when the method returns an error.

## Warnings

### WR-01: Oracle persistence tests can pass without exercising Oracle persistence

**Classification:** WARNING
**Lineage:** Round 1 — independent test-reliability finding that masks CR-01 and CR-02.
**Shared invariant:** Tests presented as proof of Oracle save/reset/replacement behavior must cover the Oracle-specific contract: actual schema constraints, bind/read behavior, item/image existence, transaction outcome, history insertion, and replacement clearing.
**Coverage inspected:** `oracle_adjustment_json_round_trips_and_rejects_invalid_values` invokes only Rust serialization helpers; `oracle_adjustment_mutations_persist_updates_and_clear_replacements` performs two substring checks; the save/reset/history and replacement tests instantiate `MemoryCatalogRepository`; the ignored live persistence smoke does not read or write adjustment metadata or call the Oracle repository method.
**File:** `controller/src/oracle_catalog.rs:2539-2567`
**Issue:** The tests pass even though production Oracle rejects the event type and even though no test observes a real CLOB bind/read or transaction. The targeted production-feature test reports two passing Oracle adjustment tests because neither opens Oracle nor executes the mutation. This makes the summary's Oracle behavior claims materially stronger than the evidence and leaves bind type, SQL NULL reset, constraint, rollback, existence, timestamp, and history regressions undetectable.
**Fix:** Add credential-free schema-contract tests that compare all Rust event strings with the canonical/migration constraint and assert Phase 8 preflight requires `imageAdjustmentChanged`. Extend the ignored live Oracle persistence smoke (or an Oracle integration fixture) to call `OracleCatalogRepository::update_image_adjustment` for save and reset, read the CLOB back through `load_item`, verify exactly one history event per call and the item timestamp, exercise wrong item/image IDs, replace the original and verify SQL NULL, and prove rollback on an induced history/deserialize failure.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_adjustment -- --nocapture` passed two tests, confirming WR-01: both are credential-free helper/string assertions and do not execute Oracle.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` passed; compilation does not validate Oracle DDL constraints or transaction behavior.
- The event producer/consumer inventory found `imageAdjustmentChanged` in the Rust enum and adapter but not in `schema.sql`, the Phase 8 migration, or schema preflight.

---

_Reviewed: 2026-09-30T10:11:28Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
