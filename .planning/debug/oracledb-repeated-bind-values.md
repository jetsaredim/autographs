---
status: fixing
trigger: "Live static publish smoke fails creating an item after the production controller migrated to oracledb"
created: 2026-08-21
updated: 2026-08-21
---

# Debug Session: oracledb-repeated-bind-values

## Symptoms

- Expected behavior: the deployed live static publish smoke creates a temporary item through the production controller and completes the publish/cleanup workflow.
- Actual behavior: authentication succeeds, but `POST /admin/api/items` returns HTTP 400 before an item ID is returned.
- Error message: `failed to create catalog item error=read Oracle signer profile: 3 positional bind values are required but 2 were provided`.
- Timeline: first observed on 2026-08-21 after deploying controller `v0.0.23`, which migrated from `oracle` to `oracledb 26.0.0-beta.2`.
- Reproduction: run `live_static_publish_smoke` against the deployed controller with `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`.

## Current Focus

- hypothesis: `oracledb` treats every placeholder occurrence as a positional bind, while signer SQL reuses `:1` and supplies only one value for both occurrences.
- test: audit repeated positional placeholders in `oracle_catalog.rs`, make every occurrence unique, and add a regression assertion for the expected bind sequence.
- expecting: the signer-profile upsert lookup needs three values and the signer-suggestion lookup needs four values.
- next_action: open and review the hotfix PR, then run the live static publish smoke against the corrected candidate controller.

## Evidence

- timestamp: 2026-08-21
  observation: production logged `3 positional bind values are required but 2 were provided` from `upsert_signer_profile` before the item insert completed.
- timestamp: 2026-08-21
  observation: the lookup SQL contains `id = :1`, `normalized_name = :2`, and repeats `id = :1` in `order by`, but passes only `requested_id` and `normalized_name`.
- timestamp: 2026-08-21
  observation: signer-suggestion SQL similarly repeats `:1` in `order by` after three earlier placeholder occurrences and passes only three values.
- timestamp: 2026-08-21
  observation: both affected queries now use unique contiguous placeholders and supply duplicated values in occurrence order; the production-feature regression, full controller suite, and Clippy pass.

## Eliminated

- hypothesis: the smoke request failed authentication or CSRF validation.
  reason: login succeeded and the create route reached the Oracle repository; authentication/CSRF failures return before the logged `creating catalog item` operation.

## Resolution

- root_cause: the migrated signer queries retained `oracle` crate behavior that allowed a repeated numbered placeholder to share one bind value, but `oracledb` counts every placeholder occurrence positionally.
- fix: assign unique bind positions to the repeated signer lookup/order expressions, provide the repeated Rust values in occurrence order, and enforce unique contiguous positions with a regression test.
- verification: `cargo fmt --check`; focused production-feature regression; full `cargo test --features production-persistence` (139 passed, 2 live tests ignored); production-feature Clippy with warnings denied. Live candidate-controller smoke remains required.
- files_changed: `controller/src/oracle_catalog.rs`, `.planning/debug/oracledb-repeated-bind-values.md`
