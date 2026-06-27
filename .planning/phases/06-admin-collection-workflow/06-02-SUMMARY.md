---
phase: 06-admin-collection-workflow
plan: 02
type: summary
status: implemented
branch: task-06-02-admin-api
pr: 141
merged_at: 2026-06-26T20:29:58Z
merge_commit: 8bc8302333a5537acf7cb9bbac59e5789729076a
---

# Phase 06-02 Summary

## Implemented

- Added authenticated admin item read APIs:
  - `GET /admin/api/items`
  - `GET /admin/api/items/{id}`
  - `GET /admin/api/items/{id}/history`
- Added admin finder filtering for `query`, `signer`, `title`, `category`, `tag`, and `publicationStatus`.
- Added redacted admin item summary, item history, edit-event, field-diff, and pending-marker response DTOs.
- Extended image responses with `altText` while continuing to omit private storage fields such as object keys, bucket names, namespaces, and original filenames.
- Added pending-change metadata to item save responses so create/update remain explicit save-only operations.
- Kept create/update/image-upload save paths separate from static publish execution; publish still only runs through explicit publish endpoints.

## Verification coverage added

- `admin_can_list_get_update_and_read_history`
  - exercises list, detail, patch, and history admin APIs
  - checks finder filters
  - checks pending markers
  - checks `altText`
  - asserts private storage details are not serialized
- `save_does_not_publish`
  - creates and patches an item
  - confirms pending changes are returned
  - confirms publish status remains `idle`
- `image_upload_response_includes_pending_changes`
  - uploads an image through the admin API
  - confirms the mutation response includes `pendingChanges`
  - confirms uploaded image `altText` is serialized

## Notes

The admin finder is implemented as a repository extension that composes with the existing `CatalogRepository::list()` method, so both the in-memory repository and Oracle repository get the same case-insensitive filtering behavior without duplicating predicate logic in each adapter.

The admin list/detail pending marker is intentionally provisional for this phase: it means the item has recorded admin edit history, not that the item differs from the last completed static release. It currently performs one history lookup per listed item, which is acceptable for the small admin catalog in 06-02; a later publish-boundary store should replace this with a bulk repository query when needed.

PR #141 merged on 2026-06-26 as `8bc8302333a5537acf7cb9bbac59e5789729076a`, and deployment was confirmed after merge.

Deployed runtime signoff passed after merge:

- Authenticated `/admin/api/items`, `/admin/api/items/{id}`, and `/admin/api/items/{id}/history` calls worked as expected.
- A full static rebuild completed successfully.

Local and CI verification completed for the final PR head:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml --test admin_workflow admin_can_list_get_update_and_read_history -- --nocapture
cargo test --manifest-path controller/Cargo.toml --test admin_workflow save_does_not_publish -- --nocapture
cargo test --manifest-path controller/Cargo.toml --test admin_workflow image_upload_response_includes_pending_changes -- --nocapture
cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
gh pr checks 141 --repo jetsaredim/autographs
```
