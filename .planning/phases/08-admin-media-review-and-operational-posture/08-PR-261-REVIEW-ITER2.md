---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-09-30T22:50:06Z
depth: deep
files_reviewed: 7
files_reviewed_list:
  - controller/db/schema.sql
  - controller/db/updates/08-01-image-adjustments.sql
  - controller/src/oracle_schema.rs
  - controller/src/oracle_catalog.rs
  - controller/src/catalog.rs
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
---

# Phase 08 Plan 05: Code Review Report — Iteration 2

**Reviewed:** 2026-09-30T22:50:06Z
**Depth:** deep
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The rewritten fixes resolve the original CR-02 transaction defect: all seven Oracle mutations that return an item now perform full readback before commit, use the same rollback helper on readback failure, preserve the original read error when rollback succeeds, and retain both errors when rollback itself fails. The CLOB save/reset/replacement paths and the new live regression now exercise real Oracle bind/read behavior when credentials are supplied. CR-01's normal successful end state is also repaired in both canonical and upgrade DDL.

Three Warning-level gaps remain. The upgrade script can leave the edit-event table unconstrained after a failed replacement; production preflight does not actually validate seven of the nine event values it claims as a shared Rust/DDL contract; and the live smoke can report success while silently leaving Oracle and OCI fixtures behind.

Lineage status:

- **CR-01:** successful fresh-schema and successful-upgrade behavior is fixed, but WR-02 is a **fix regression** in migration failure safety and WR-03 is an **incomplete fix** in runtime contract validation.
- **CR-02:** resolved across create, update, attach-image, set-primary, remove-image, replace-image, and adjustment-update mutation paths. No actionable recurrence or sibling-path miss found.
- **WR-01:** live Oracle coverage is now present, but WR-04 is a remaining **test weakness** in the live fixture lifecycle.

## Narrative Findings (AI reviewer)

## Warnings

### WR-02: The constraint replacement can commit the drop before the replacement exists

**Classification:** WARNING
**Lineage classification:** **fix regression** — introduced by the CR-01 repair.
**File:** `controller/db/updates/08-01-image-adjustments.sql:25-53`

**Shared invariant:** At every migration boundary, `AUTOGRAPH_EDIT_EVENTS` must retain an enabled check constraint that admits every event emitted by the controller. A failed or interrupted upgrade must not temporarily or permanently widen the table to arbitrary event values.

**Inspected boundary:** The review traced all nine `EditEventKind` producers, `insert_edit_event`, history and pending-change readers, publish snapshot consumption, fresh-schema DDL, upgrade DDL, schema preflight, and the static/live schema tests. The successful migration end state contains the complete event set.

**Issue:** Lines 34-37 drop `AUTOGRAPH_EDIT_EVENTS_TYPE_CK`, then lines 39-52 add its replacement. Oracle DDL implicitly commits before and after each DDL statement, so the final `commit` cannot make these operations atomic. If the add fails after the drop—for example because of an operational interruption, insufficient quota, or an unexpected existing object—the old constraint is already gone. The script exits failed but the currently deployed controller can continue inserting unchecked `event_type` values until the migration is repaired. This contradicts the script's safety claim and weakens the audit-history integrity boundary.

**Fix:** Add the expanded check under a temporary name while the old constraint remains enabled, then drop the old constraint and rename the temporary constraint to the canonical name. Make the PL/SQL recognize and recover each rerun state (old-only, both constraints, temporary-only after an interrupted rename, and canonical-new). Add a credentialed migration test that exercises a legacy schema, a fresh schema, rerun, and each recoverable partial state.

### WR-03: Runtime preflight accepts an edit-event constraint missing seven supported values

**Classification:** WARNING
**Lineage classification:** **incomplete fix** — CR-01 added a complete static text check but not a complete production preflight check.
**Files:** `controller/src/oracle_schema.rs:46-58`, `controller/src/oracle_schema.rs:141-159`, `controller/src/catalog.rs:395-419`

**Shared invariant:** Every event kind that a production mutation can emit must be admitted by the enabled database constraint before the controller is considered schema-ready.

**Inspected boundary:** `EditEventKind::ALL` contains nine values and the canonical/migration text test checks all nine. Production producers emit created, metadata, image-added, image-removed, image-replaced, image-adjustment, primary-image, publication, and cleanup events. History/pending/publish consumers all rely on those persisted records.

**Issue:** Production preflight only searches `AUTOGRAPH_EDIT_EVENTS_TYPE_CK` for `cleanupChanged` and `imageAdjustmentChanged`. A partially repaired or manually drifted constraint containing those two strings but omitting any of the other seven passes startup validation. The controller then fails the corresponding otherwise-valid mutation at edit-event insertion. The `EditEventKind::ALL` loop at lines 524-534 only inspects committed SQL text during unit tests; it is not used by live preflight and therefore does not close this runtime gap.

**Fix:** Define the complete event list once from `EditEventKind::ALL` and require every value in the enabled canonical constraint during `verify_oracle_schema`. Keep remediation tied to the latest migration. Add a preflight test/fixture whose constraint deliberately contains the two currently checked values but omits one older event, and assert startup fails.

### WR-04: A passing live smoke can silently leave production fixtures behind

**Classification:** WARNING
**Lineage classification:** **test weakness** — follow-on to WR-01's new credentialed Oracle/OCI proof.
**File:** `controller/tests/live_persistence_smoke.rs:730-775`

**Shared invariant:** A successful live smoke means both that the persistence contract passed and that every Oracle row and OCI object created by the run was durably removed. Cleanup failure must either fail the run or leave explicit, actionable recovery evidence.

**Inspected boundary:** The review followed fixture creation, the uploaded object, target and malformed-sibling image rows, edit-history rows, item cascade relationships, automatic `Drop` cleanup, and the separate manual cleanup mode. The manual mode verifies row/object absence, but the normal successful path does not use it.

**Issue:** The cleanup guard ignores the thread join result, returns silently when runtime construction fails, only logs OCI deletion failures/timeouts, discards both Oracle delete results, and discards the commit result. Consequently, all persistence assertions can pass while the test leaves the private object and an item containing deliberately malformed CLOB metadata plus history rows in the live database. The test still exits successfully, so operators and CI have no reliable signal that manual cleanup is required.

**Fix:** Give the fixture an explicit async `cleanup_and_verify()` used on the success path. Require OCI deletion and confirmed absence, require Oracle delete and commit success, and query that the item/images/events are gone before allowing the test to pass. Keep `Drop` only as a best-effort panic fallback; if fallback cleanup cannot complete, print the item ID and object key in a stable recovery message rather than suppressing database failures.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow` — 29 passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_ -- --nocapture` — 42 matching tests passed; the credentialed live test remained filtered.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `git diff --check` — passed before this review artifact was written.
- The credentialed Oracle/OCI smoke was not executed in this environment.

---

_Reviewed: 2026-09-30T22:50:06Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
