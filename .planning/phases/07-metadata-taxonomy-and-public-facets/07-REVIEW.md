---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T15:55:38Z
depth: standard
files_reviewed: 37
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
  critical: 3
  warning: 1
  info: 0
  total: 4
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T15:55:38Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the listed Phase 7 schema, taxonomy backfill, Rust controller, Oracle adapter, static admin/public artifacts, tests, and documentation at standard depth. Commit `d4c1489` improves the Oracle preflight by adding Phase 7 constraint checks, but it does not fully resolve the missing schema-guarantee warning: incompatible constraints can still pass, the signer uniqueness check does not prove the required column, and legacy rows left for manual taxonomy review can be persisted back through Oracle with a nil signer UUID.

## Critical Issues

### CR-01: BLOCKER - Phase 7 Preflight Accepts Constraints That Still Reject Valid Phase 7 Values

**Severity:** BLOCKER
**File:** `controller/src/oracle_schema.rs:58`
**Issue:** The new preflight entries only search for one token inside each check constraint. `AUTOGRAPH_ITEMS_ORIGIN_CK` requires text containing `Official`, and `AUTOGRAPH_ITEMS_LANGUAGE_CK` requires text containing `English`, while the query at `controller/src/oracle_schema.rs:139` uses `search_condition_vc like '%' || :3 || '%'`. A live schema with enabled constraints such as `origin in ('Official')` or `language in ('English')` will pass startup preflight, but the runtime accepts and writes `Custom`, `Japanese`, and `Chinese` values in `controller/src/oracle_catalog.rs:136` and `controller/src/catalog.rs:1421`. That means the controller can boot successfully and then fail valid Phase 7 admin saves or live publish smoke data at Oracle DML time.
**Fix:** Validate every allowed enum value for each required check constraint instead of one representative fragment. Add a regression test or fixture proving that `origin in ('Official')` and `language in ('English')` are rejected by preflight.

```rust
const REQUIRED_CHECK_CONSTRAINTS: &[(&str, &str, &[&str], &str)] = &[
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_ORIGIN_CK",
        &["Official", "Custom"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_LANGUAGE_CK",
        &["English", "Japanese", "Chinese"],
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
];
```

### CR-02: BLOCKER - Signer Unique Constraint Preflight Does Not Verify `NORMALIZED_NAME`

**Severity:** BLOCKER
**File:** `controller/src/oracle_schema.rs:152`
**Issue:** `REQUIRED_UNIQUE_CONSTRAINTS` verifies only that an enabled unique constraint named `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ` exists on `AUTOGRAPH_SIGNERS`; it never checks the constrained column list. A wrongly recreated constraint with the expected name but backed by `ID`, `DISPLAY_NAME`, or multiple columns will pass preflight even though the backfill SQL and runtime assume uniqueness on `NORMALIZED_NAME`. The Phase 7 migration uses `merge into autograph_signers ... on (signer.normalized_name = incoming.normalized_name)` in `controller/db/updates/07-03-taxonomy-backfill-apply.sql:6`, and the Oracle adapter resolves profiles by `normalized_name` in `controller/src/oracle_catalog.rs:1660`, so missing normalized-name uniqueness can produce duplicate signer profiles or unstable merge/upsert behavior.
**Fix:** Join `USER_CONS_COLUMNS` and verify the expected column set exactly, not just the constraint name/type/status. Also reject constraints with additional columns.

```sql
select c.constraint_name
from user_constraints c
join user_cons_columns col
  on col.table_name = c.table_name
 and col.constraint_name = c.constraint_name
where c.table_name = :1
  and c.constraint_name = :2
  and c.constraint_type = 'U'
  and c.status = 'ENABLED'
group by c.constraint_name
having count(*) = 1
   and max(case when col.position = 1 and col.column_name = 'NORMALIZED_NAME' then 1 else 0 end) = 1
```

### CR-03: BLOCKER - Legacy Items Without Signer Joins Can Be Saved Back With A Nil Signer UUID

**Severity:** BLOCKER
**File:** `controller/src/oracle_catalog.rs:900`
**Issue:** `load_item()` synthesizes fallback signer credits for legacy rows with no `AUTOGRAPH_ITEM_SIGNERS` joins, and `legacy_signer_credits()` assigns `Uuid::nil()` at `controller/src/oracle_catalog.rs:947`. `update()` then always calls `replace_signer_credits()` after a metadata save at `controller/src/oracle_catalog.rs:271`. Phase 7 intentionally leaves some rows for manual review, such as the multi-signer fixture row `Mark Hamill / Carrie Fisher` in `controller/fixtures/taxonomy-legacy-export.json:27`, and the backfill generator skips those multi-signer rows in `controller/src/taxonomy_migration.rs:204`. Editing any unrelated field on one of those legacy rows can therefore delete real joins and upsert a persisted signer with `00000000-0000-0000-0000-000000000000`; editing another legacy row can reuse/update that same nil signer and corrupt signer-to-item associations.
**Fix:** Do not persist synthetic fallback credits. Preserve a separate flag for whether credits came from the join table before applying the display-only fallback, and only replace signer credits when the request explicitly updates signer credits or the item already has real persisted credits. If a legacy fallback must be migrated during an edit, allocate a real signer UUID through the same profile resolver and never use `Uuid::nil()` for a row that can be written.

```rust
let persisted_signer_credits = load_signer_credits(&connection, id)?;
let has_persisted_signer_credits = !persisted_signer_credits.is_empty();
item.signer_credits = if has_persisted_signer_credits {
    persisted_signer_credits
} else {
    legacy_signer_credits(&item.signer, &item.signing_role, &item.format)
};

if input.signer_credits.is_some() || has_persisted_signer_credits {
    replace_signer_credits(&connection, id, &item.signer_credits)?;
}
```

## Warnings

### WR-01: WARNING - Preflight Remediation Script Cannot Repair Existing Signer Constraint Drift

**Severity:** WARNING
**File:** `controller/db/updates/07-01-taxonomy-schema.sql:143`
**Issue:** The new preflight failure messages point operators at `controller/db/updates/07-01-taxonomy-schema.sql`, but that script only creates `AUTOGRAPH_SIGNERS` constraints inside the `if table_count = 0` branch. If a live database already has the `AUTOGRAPH_SIGNERS` table and columns but is missing `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK` or `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ`, rerunning the recommended script will not add them. The deployment remains blocked after following the remediation, or worse, an operator may manually patch the schema inconsistently.
**Fix:** Make the Phase 7 schema update idempotently repair signer constraints on existing tables, with duplicate detection before adding the unique constraint.

```sql
declare
  constraint_count number;
begin
  select count(*) into constraint_count
  from user_constraints
  where table_name = 'AUTOGRAPH_SIGNERS'
    and constraint_name = 'AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK';

  if constraint_count = 0 then
    execute immediate q'[
      alter table autograph_signers add constraint autograph_signers_normalized_name_ck
      check (trim(normalized_name) is not null)
    ]';
  end if;
end;
/
```

Add a similar guarded block for `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ` after failing closed if duplicate `normalized_name` values already exist.

---

_Reviewed: 2026-07-10T15:55:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
