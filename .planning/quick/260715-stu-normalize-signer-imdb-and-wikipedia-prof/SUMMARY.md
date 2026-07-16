---
status: complete
completed: 2026-07-16
branch: gsd/quick-add-admin-signer-profile-management-tab-
---
# Normalize Signer IMDb And Wikipedia Profile Links

Implemented compact signer profile storage without a schema update.

## Completed

- Normalized Wikipedia profile values to `w.wiki` short IDs, accepting either the ID or `https://w.wiki/{id}`.
- Normalized IMDb profile values to `nm...` name IDs, accepting either the ID or `https://www.imdb.com/name/{id}/`.
- Reused the shared normalizer from both memory and Oracle persistence paths.
- Expanded compact IDs back to full public profile links during static publishing.
- Replaced public detail profile-link text with inline Wikipedia and IMDb SVG icon badges sourced from Simple Icons.
- Updated Signers admin labels to `Wikipedia short ID` and `IMDb name ID`.
- Updated tests for compact storage, public full-link rendering, and production-persistence normalization.

## Verification

- `cargo fmt --check`
- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`
- `cargo test --test admin_workflow`
- `cargo test --test publisher publisher_generates_phase7_signer_taxonomy_facets_and_detail_links`
- `cargo test --test static_contract`
- `cargo test --features production-persistence oracle_profile_links_normalize_to_compact_ids`
- `cargo check --features production-persistence`
