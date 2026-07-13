---
status: resolved
trigger: "Admin UI item save fails with ORA-12839 after updating catalog item"
created: 2026-07-12
updated: 2026-07-12
---

# Debug Session: admin-item-save-ora-12839

## Symptoms

- Expected behavior: saving an edited admin catalog item should persist the item and return the updated item response.
- Actual behavior: the controller logs a failed update and the admin save fails.
- Error message:
  - `failed to update catalog item item_id=1e98b0f0-91c1-4631-ace6-65bbbb013533 error=update Oracle signer profile: OCI Error: ORA-12839: cannot modify an object in parallel after modifying it`
- Timeline: observed on 2026-07-12 while using the current admin UI/site.
- Reproduction: edit an existing admin item and click save.

## Current Focus

- hypothesis: Oracle signer profile upsert/update is attempting to modify a table after a previous signer profile modification in the same transaction.
- test: inspect Oracle item update and signer profile upsert transaction ordering for parallel/direct-path DML hints or session settings.
- expecting: item update path resolves/creates signer profiles before replacing item signer credits, then redundantly upserts the same profile again.
- next_action: verified fix locally; deploy and retry admin save against live Oracle.

## Evidence

- timestamp: 2026-07-12
  observation: `OracleCatalogRepository::update` resolves signer credits before applying the item update, and `resolve_oracle_signer_profile` inserts a new signer profile when a display-name input does not match an existing normalized name.
- timestamp: 2026-07-12
  observation: `replace_signer_credits` then called `upsert_signer_profile` for every already-resolved credit before inserting `autograph_item_signers`, causing a redundant `update autograph_signers` in the same item-save transaction.
- timestamp: 2026-07-12
  observation: Oracle can raise `ORA-12839` when a transaction modifies an object again after a parallel/direct-path modification; the controller log failed specifically at `update Oracle signer profile`.
- timestamp: 2026-07-12
  observation: Existing memory repository tests already assert selected signer reuse preserves existing profile metadata, matching the intended behavior of not rewriting the profile during item association replacement.

## Eliminated

- hypothesis: The local SQL contains explicit `APPEND`, `PARALLEL`, or `alter session enable parallel dml` hints.
  reason: Repository search found no such hints in controller Oracle SQL; the code-level bug was the redundant second signer profile write.

## Resolution

- root_cause: `replace_signer_credits` performed a second `autograph_signers` upsert for credits that had already been resolved or created, which could trip Oracle's same-transaction object modification restriction under the live table/session settings.
- fix: Use the already-resolved `credit.signer.id` when inserting `autograph_item_signers` and leave signer profile persistence to `resolve_oracle_signer_credits` / explicit signer profile update routes.
- verification: `cargo fmt`; `cargo test --test admin_workflow item_signer`; `cargo test oracle_catalog::tests`; `cargo test`; `cargo check --features production-persistence`.
- files_changed: `controller/src/oracle_catalog.rs`, `.planning/debug/admin-item-save-ora-12839.md`
