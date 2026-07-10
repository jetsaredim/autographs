---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T11:18:35Z
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
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T11:18:35Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the listed Phase 7 schema, Rust controller, static admin/public artifacts, tests, and documentation at standard depth. Commit `91a519c` resolves the two called-out regressions in the reviewed code: signer profile PATCH now uses `FieldPatch` so omitted optional fields are preserved while explicit `null` clears them, and taxonomy backfill generation now marks items with multiple mapped role rows as `NeedsReview` instead of auto-collapsing to one role.

One deployment-safety issue remains in the Oracle schema preflight: it recognizes the new Phase 7 tables and columns but does not verify the Phase 7 constraint/index guarantees that the controller and migration assume.

## Warnings

### WR-01: Oracle Preflight Misses Phase 7 Schema Guarantees

**Severity:** WARNING
**File:** `controller/src/oracle_schema.rs:43`
**Issue:** `ensure_initialized_on_connection()` treats the Phase 7 schema as valid once the expected tables and columns exist, but `REQUIRED_CHECK_CONSTRAINTS` still only verifies the older `AUTOGRAPH_EDIT_EVENTS_TYPE_CK` cleanup value. A live schema where `07-01-taxonomy-schema.sql` was interrupted after adding tables/columns, or manually changed without the Phase 7 checks and uniqueness guarantees, will pass controller startup preflight. That allows invalid taxonomy enum values or duplicate signer normalized names into the Oracle catalog despite the canonical schema requiring constraints such as `autograph_items_format_ck`, `autograph_items_origin_ck`, `autograph_items_language_ck`, and `autograph_signers_normalized_name_uq`.
**Fix:** Extend the schema preflight to validate the Phase 7 constraints and unique indexes/constraints, and point failures at `controller/db/updates/07-01-taxonomy-schema.sql` rather than only the Phase 6 cleanup script. For example:

```rust
const REQUIRED_CHECK_CONSTRAINTS: &[(&str, &str, &str, &str)] = &[
    (
        "AUTOGRAPH_EDIT_EVENTS",
        "AUTOGRAPH_EDIT_EVENTS_TYPE_CK",
        "cleanupChanged",
        "controller/db/updates/06-03-media-cleanup.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_FORMAT_CK",
        "trim(format) is not null",
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_ORIGIN_CK",
        "Official",
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
    (
        "AUTOGRAPH_ITEMS",
        "AUTOGRAPH_ITEMS_LANGUAGE_CK",
        "English",
        "controller/db/updates/07-01-taxonomy-schema.sql",
    ),
];

const REQUIRED_UNIQUE_CONSTRAINTS: &[(&str, &str, &str)] = &[(
    "AUTOGRAPH_SIGNERS",
    "AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ",
    "controller/db/updates/07-01-taxonomy-schema.sql",
)];
```

---

_Reviewed: 2026-07-10T11:18:35Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
