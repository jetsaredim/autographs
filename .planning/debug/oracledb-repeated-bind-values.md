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

- hypothesis: `oracledb` binds positional slices strictly by placeholder occurrence, so repeated names and SQL whose textual positions differ from occurrence order can both misbind.
- test: audit numeric placeholders across the Oracle adapter, convert order-sensitive signer statements to named binding, and exercise signer suggestions in the live static-publish smoke.
- expecting: named binds remove placeholder repetition/order ambiguity from all three affected signer statements while ordinary sequential positional statements remain unchanged.
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
- timestamp: 2026-08-21
  observation: reviewer audit found `sync_signer_credits` also listed placeholders as `:3, :4, :5, :1, :2` while supplying values in numeric-label order; `oracledb` would bind them by occurrence instead.
- timestamp: 2026-08-21
  observation: all three order-sensitive signer statements now use meaningful named binds; an adapter-wide scan found no remaining SQL string with repeated or out-of-order numeric placeholders.
- timestamp: 2026-08-21
  observation: the live static-publish smoke now queries `/admin/api/signers?query=...` and asserts that the newly created signer is returned before proceeding to media and publish validation.

## Eliminated

- hypothesis: the smoke request failed authentication or CSRF validation.
  reason: login succeeded and the create route reached the Oracle repository; authentication/CSRF failures return before the logged `creating catalog item` operation.

## Resolution

- root_cause: migrated signer statements retained `oracle` crate binding assumptions: repeated placeholder names shared a value and numeric labels determined mapping. `oracledb` positional calls instead bind strictly by left-to-right occurrence.
- fix: use `query_named`/`execute_named` for repeated and clause-reordered signer SQL, keep simple sequential statements positional, audit the adapter's numeric placeholders, and add signer-suggestion coverage to the live smoke.
- verification: `cargo fmt --check`; focused Oracle catalog tests; live-smoke compilation; full `cargo test --features production-persistence`; production-feature Clippy with warnings denied. Live candidate-controller smoke remains required.
- files_changed: `controller/src/oracle_catalog.rs`, `controller/tests/live_static_publish_smoke.rs`, `.planning/debug/oracledb-repeated-bind-values.md`
