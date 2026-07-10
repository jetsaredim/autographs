---
phase: 07-metadata-taxonomy-and-public-facets
review: 07-REVIEW.md
fixed: 2026-07-10T02:30:00Z
status: fixed
fix_scope: critical_warning
commits:
  - 561d840
---

# Phase 07 Review Fix Report

## Summary

Applied fixes for all blocker and warning findings from `07-REVIEW.md`.

## Fixes

- `CR-01`: tightened signer profile URL validation to allow only HTTPS Wikipedia and IMDb profile hosts before values can reach public detail-page links.
- `CR-02`: changed admin signer payload collection to read row-scoped `data-signer-field` inputs so removing signer rows does not drop the remaining signer credits.
- `CR-03`: stopped sending item role edits as reusable signer `defaultRole` updates from the item editor.
- `WR-01`: mapped known validation failures to `400 Bad Request` instead of `500 Internal Server Error`.
- `WR-02`: deduplicated comma-separated taxonomy token lists in the admin payload path.
- `WR-03`: corrected the static runtime runbook create-item payload to include compatibility fields and `itemRole`.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `node --check controller/static-admin/admin.js && node --check controller/static-public/assets/browse.js`
- `cargo test --manifest-path controller/Cargo.toml`
