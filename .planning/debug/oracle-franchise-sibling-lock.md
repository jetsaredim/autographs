---
status: fixing
trigger: "Production admin item update fails inserting Oracle catalog franchises with ORA-12860: deadlock detected while waiting for a sibling row lock"
created: 2026-08-22
updated: 2026-08-22
---

# Debug Session: Oracle Franchise Sibling Lock

## Symptoms

- Expected behavior: updating an existing item in the admin UI persists the
  submitted metadata and returns the updated item.
- Actual behavior: the update fails and the controller logs an Oracle
  persistence error while inserting franchise rows.
- Error message: `insert Oracle catalog franchises: ORA-12860: deadlock
  detected while waiting for a sibling row lock`.
- Timeline: observed in production on 2026-08-22 after the `oracledb` driver
  migration and successful live static-publish smoke.
- Reproduction: edit item `78b82eb8-e6ec-423c-8062-dd5d347e999e` in the admin
  UI and save the form.

## Current Focus

- hypothesis: confirmed. The production Autonomous service enables parallel DML
  by default, while an admin update performs multiple DML statements in one
  OLTP transaction; franchise delete/reinsert exposes that transaction to a
  PDML sibling-row deadlock.
- test: initialize every controller Oracle connection with serial DML, skip
  unchanged submitted taxonomy collection writes, and exercise a changed
  franchise list through the live static-publish smoke.
- expecting: all controller DML runs serially; unchanged collections perform no
  DML; the candidate smoke changes and publishes franchise rows without
  ORA-12860.
- next_action: open the fix PR, counter-review it, then run the updated live
  static-publish smoke against the candidate controller on the production VM.

## Evidence

- timestamp: 2026-08-22
  observation: Oracle documents ORA-12860 as a deadlock between PDML sibling
    transactions waiting on row locks and recommends serial retry when parallel
    retry fails.
- timestamp: 2026-08-22
  observation: `OracleCatalogRepository::update` calls `replace_franchises`
    whenever `input.franchises` is present, while the admin submits full item
    metadata and `apply_update` already records whether franchises changed.
- timestamp: 2026-08-22
  observation: `replace_franchises` deletes every franchise row for the item and
    immediately reinserts the desired rows; tags and characters use the same
    replacement pattern.
- timestamp: 2026-08-22
  observation: signer credits previously produced live Oracle update failures
    from redundant writes and now use diff gating plus stable row-level sync.
- timestamp: 2026-08-22
  observation: Oracle documents that Autonomous HIGH and MEDIUM services enable
    parallel DML by default and explicitly recommends `alter session disable
    parallel dml` when serial execution is needed:
    https://docs.oracle.com/en/cloud/paas/autonomous-database/serverless/adbsb/predefined-database-services-names.html
- timestamp: 2026-08-22
  observation: all production-feature unit and integration tests, production
    compilation, live-smoke compilation, and Clippy with warnings denied pass
    after serial session initialization and taxonomy diff gating.

## Eliminated

- hypothesis: the error is caused by repeated or out-of-order `oracledb`
    positional bind placeholders.
  reason: the failing franchise insert uses unique sequential placeholders
    `:1`, `:2`, and `:3` with three positional values.
- hypothesis: the error requires two independent controller requests updating
    the same item concurrently.
  reason: ORA-12860 identifies sibling transactions created by one PDML
    operation, and Autonomous MEDIUM enables PDML within a session by default.

## Resolution

- root_cause: the Autonomous database service enables PDML by default, but the
    controller uses multi-statement item-update transactions. Deleting and then
    reinserting franchise rows allowed PDML sibling workers to deadlock on row
    locks. Full admin payloads also caused unchanged taxonomy collections to be
    rewritten unnecessarily.
- fix: initialize every controller Oracle session with `alter session disable
    parallel dml`; replace tags, characters, and franchises only when their
    submitted value actually produced a field diff; extend the live static
    smoke with a real item PATCH that changes franchises and resubmits an
    unchanged tag.
- verification: `cargo fmt --check`; focused serial-session and collection
    writeback regressions; live static-publish smoke production compilation;
    full `cargo test --features production-persistence`; `cargo check --features
    production-persistence`; `cargo clippy --all-targets --features
    production-persistence -- -D warnings`. Candidate production smoke remains
    required.
- files_changed: `controller/src/oracle_connection.rs`,
    `controller/src/oracle_catalog.rs`,
    `controller/tests/live_static_publish_smoke.rs`,
    `.planning/debug/oracle-franchise-sibling-lock.md`.
