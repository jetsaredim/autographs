---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T11:00:09Z
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
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T11:00:09Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the listed Phase 7 schema, Rust controller, static admin/public, test, and documentation files at standard depth. Commit `c069a4d` does resolve the stale/deleted `signerId` recreation issue: both the memory and Oracle repositories now fail closed when a supplied signer ID no longer exists, while requests without `signerId` still create or reuse profiles by display name. The merge cleanup regression is also covered by the new memory repository test at `controller/tests/admin_workflow.rs:1603`.

The remaining blocker is adjacent to signer selection. The admin create-or-select path can still submit partial signer profile fields for an existing signer, and both repository implementations treat missing optional fields as authoritative clears. That can erase reusable profile metadata such as Wikipedia/IMDb links and default roles while saving an item.

## Critical Issues

### CR-01: Existing Signer Selection Can Clear Profile Metadata

**Severity:** BLOCKER
**File:** `controller/static-admin/admin.js:997`
**Affected:** `controller/static-admin/admin.js:629`, `controller/src/catalog.rs:1478`, `controller/src/catalog.rs:1552`, `controller/src/oracle_catalog.rs:1001`, `controller/src/oracle_catalog.rs:1301`
**Issue:** Selecting an existing signer from suggestions only stores `row.dataset.signerId`; it does not hydrate that row with the selected profile's existing Wikipedia/IMDb/default-role metadata. The save payload then always sends `wikipediaUrl` and `imdbUrl`, using `null` when those row fields are empty. On the server, the `signerId` branch calls `update_signer_profile()` / `apply_signer_input_to_profile()`, and those helpers normalize every missing optional profile field to `None` and write it back. A normal "select existing signer, save item" flow can therefore clear public signer profile links/default roles in both local and Oracle persistence without the operator intending to edit that profile.
**Fix:**
```rust
if let Some(signer_id) = input.signer_id {
    let profile = signers
        .get(&signer_id)
        .ok_or_else(|| "signer profile was not found".to_owned())?;
    validate_signer_id_display_name(profile, input)?;
    return Ok(profile.clone());
}
```
Do the same in `resolve_oracle_signer_profile()`: when an existing profile is selected by `signerId`, validate it and return it without applying item-credit optional fields to the reusable profile. For the no-`signerId` path, reuse an existing normalized-name profile without clearing optional metadata; only initialize optional profile fields when creating a brand-new signer profile. Add regressions that first create a signer with role and profile links, then save another item using that signer's ID or exact display name with no profile fields, and assert the existing profile metadata is retained.

---

_Reviewed: 2026-07-10T11:00:09Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
