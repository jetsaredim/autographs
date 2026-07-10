---
phase: 07-metadata-taxonomy-and-public-facets
review: 07-REVIEW.md
fixed: 2026-07-10T02:30:00Z
status: fixed
fix_scope: critical_warning
commits:
  - 561d840
  - 27731f2
  - d7fb1a1
---

# Phase 07 Review Fix Report

## Summary

Applied fixes for all blocker and warning findings from `07-REVIEW.md`, then completed
the follow-up fixes requested by the re-review.

## Fixes

- `CR-01`: tightened signer profile URL validation to allow only HTTPS Wikipedia and IMDb profile hosts before values can reach public detail-page links.
- `CR-02`: changed admin signer payload collection to read row-scoped `data-signer-field` inputs so removing signer rows does not drop the remaining signer credits.
- `CR-03`: stopped sending item role edits as reusable signer `defaultRole` updates from the item editor.
- `WR-01`: mapped known validation failures to `400 Bad Request` instead of `500 Internal Server Error`.
- `WR-02`: deduplicated comma-separated taxonomy token lists in the admin payload path.
- `WR-03`: corrected the static runtime runbook create-item payload to include compatibility fields and `itemRole`.

## Follow-up Fixes

- `CR-01`: mirrored HTTPS/host signer profile URL validation in the Oracle repository so production persistence rejects unsafe public detail-page link values.
- `WR-01`: deduplicated direct API taxonomy lists at the Rust repository boundary and before Oracle insert loops so duplicate tags, characters, and franchises do not trigger Oracle key collisions.

## Second Follow-up Fixes

- `CR-01`: cleared stale signer IDs in the admin item editor when the visible signer name diverges, and rejected direct/stale payloads that combine a signer ID with a conflicting display name.
- `CR-02`: updated the taxonomy backfill generator and checked-in apply SQL to merge unambiguous legacy signers into `autograph_signers` and link items through `autograph_item_signers` before role updates.
- `WR-01`: changed Oracle signer suggestions to query matching profiles in SQL instead of filtering the first 50 rows, and removed the silent 50-profile cap from taxonomy suggestions.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `node --check controller/static-admin/admin.js && node --check controller/static-public/assets/browse.js`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml direct_taxonomy_payloads_are_trimmed_and_deduplicated`
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_profile_urls_require_https_expected_hosts`
- `cargo run --manifest-path controller/Cargo.toml --bin taxonomy_backfill -- plsql --mapping .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json --input controller/fixtures/taxonomy-legacy-export.json --out controller/db/updates/07-03-taxonomy-backfill-apply.sql`
- `cargo test --manifest-path controller/Cargo.toml item_signer_credit_rejects_conflicting_profile_id_and_display_name`
- `cargo test --manifest-path controller/Cargo.toml --test taxonomy_migration`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin static_admin_signer_payload_uses_row_scoped_fields_and_item_role_only`
