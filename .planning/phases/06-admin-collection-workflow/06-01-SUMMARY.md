---
phase: 06-admin-collection-workflow
plan: 01
type: summary
status: attempted
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

Intended verification commands:

```sh
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
```

The implementation was prepared through the GitHub connector because the execution sandbox could not resolve `github.com`; local cargo/rustfmt were also unavailable in the sandbox, so verification should be run in the normal repository environment.
