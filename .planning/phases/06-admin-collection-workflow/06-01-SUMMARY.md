---
phase: 06-admin-collection-workflow
plan: 01
type: summary
status: complete
---

# Phase 6 Plan 01 Summary

Implemented the durable catalog-change foundation for the admin workflow.

## Completed

- Added `FieldPatch<T>` so admin metadata patches can distinguish missing fields, explicit JSON null clears, and concrete set values.
- Added edit-history domain contracts: `FieldDiff`, `EditEventKind`, `AutographEditEvent`, and `PendingChangeSummary`.
- Added `created_at_epoch_seconds` and `updated_at_epoch_seconds` to `AutographItem`.
- Extended `CatalogRepository` with `history`, `pending_changes`, and `record_event`.
- Updated the memory repository to emit create, metadata, publication, and image-add events.
- Added field-level diffs for metadata updates, including nullable-field clears and publication-status changes.
- Added the canonical Oracle `autograph_edit_events` table and item-created index.
- Updated Oracle schema preflight to require `AUTOGRAPH_EDIT_EVENTS`, `EVENT_TYPE`, and `FIELD_DIFFS_JSON`.
- Updated the Oracle catalog adapter to insert/query edit events and report pending changes relative to the latest succeeded publish job.
- Added `controller/tests/admin_workflow.rs` coverage for nullable clears, metadata diffs, publication diffs, and pending private changes.

## Verification

Verified on 2026-06-22 after PR #139 was merged to `main`:

```sh
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml
```

Results:

- `cargo fmt --check`: passed.
- `admin_workflow`: 4 passed, 0 failed.
- Full controller test suite: passed; live persistence/static publish smoke tests remain ignored unless compiled with live credentials.
- `production-persistence` check: passed.
- `cargo clippy`: passed.

## Rollout

Operator confirmed on 2026-06-22 that the live Oracle schema changes were applied, the deploy workflow was rerun, and a full content generation was triggered.
