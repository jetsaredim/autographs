---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T16:04:36Z
depth: standard
files_reviewed: 38
files_reviewed_list:
  - controller/db/schema.sql
  - controller/db/updates/07-01-taxonomy-schema.sql
  - controller/db/updates/07-02-taxonomy-backfill-review.sql
  - controller/db/updates/07-03-taxonomy-backfill-apply.sql
  - controller/fixtures/taxonomy-legacy-export.json
  - controller/src/bin/taxonomy_backfill.rs
  - controller/src/catalog.rs
  - controller/src/catalog_admin.rs
  - controller/src/contracts.rs
  - controller/src/lib.rs
  - controller/src/oracle_catalog.rs
  - controller/src/oracle_schema.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/src/routes/admin_items.rs
  - controller/src/taxonomy_migration.rs
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/static-admin/index.html
  - controller/static-public/assets/browse.js
  - controller/static-public/assets/site.css
  - controller/static-public/data/collection.json
  - controller/static-public/data/facets.json
  - controller/static-public/items/ahsoka-tano/index.html
  - controller/tests/admin_workflow.rs
  - controller/tests/live_static_publish_smoke.rs
  - controller/tests/media_cleanup.rs
  - controller/tests/publisher.rs
  - controller/tests/seed_content.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_contract.rs
  - controller/tests/taxonomy_migration.rs
  - docs/controller-walkthrough.md
  - docs/deployment-runbook.md
  - docs/security-review.md
  - docs/static-artifact-contract.md
  - docs/static-runtime-runbook.md
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T16:04:36Z
**Depth:** standard
**Files Reviewed:** 38
**Status:** issues_found

## Summary

Reviewed the Phase 7 schema updates, taxonomy backfill tooling, Rust catalog/Oracle/publisher/routes changes, static admin/public assets, tests, and docs at standard depth. Commit `3d79fb4` appears to resolve the Oracle preflight checks for all Phase 7 enum/check values, exact one-column `NORMALIZED_NAME` unique constraint verification, and the synthetic nil UUID legacy signer credit writeback path. One migration-order blocker remains in `07-01`, so the phase is not ready to ship.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: 07-01 Alters Signer Constraints Before Creating The Signer Table

**Classification:** BLOCKER
**File:** `controller/db/updates/07-01-taxonomy-schema.sql:143`
**Issue:** On an existing Phase 6 production schema, `AUTOGRAPH_ITEMS` exists but `AUTOGRAPH_SIGNERS` does not. The migration attempts to inspect and add `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK` and `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ` before the `create table autograph_signers` block later in the file. Because those constraint counts are zero on upgrade, Oracle executes `alter table autograph_signers ...` against a missing table and aborts the update before the Phase 7 signer tables are created. This means the intended idempotent signer constraint remediation in `07-01` still fails for the normal Phase 6-to-Phase 7 path.
**Fix:** Move the existing `AUTOGRAPH_SIGNERS` create-table block before any signer constraint remediation, or guard the remediation so it only runs after the table exists. Keep duplicate normalized-name detection and the exact unique-constraint add/verify path after the table is known to exist.

```sql
-- Run the AUTOGRAPH_SIGNERS create-table block before constraint remediation.
-- Then run remediation only once the table is present.
declare
  table_count number;
begin
  select count(*) into table_count
  from user_tables
  where table_name = 'AUTOGRAPH_SIGNERS';

  if table_count = 1 then
    -- inspect existing normalized_name constraints, check duplicates,
    -- then add/verify AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK and _UQ
    null;
  end if;
end;
/
```

---

_Reviewed: 2026-07-10T16:04:36Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
