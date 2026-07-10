---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T10:52:15Z
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

**Reviewed:** 2026-07-10T10:52:15Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the listed Phase 7 schema, Rust controller, static admin/public, test, and documentation files at standard depth. Commit `d7fb1a1` appears to resolve the prior backfill signer materialization and Oracle signer suggestion cap findings: the generated/apply PL/SQL now merges simple legacy signers into `autograph_signers` and inserts `autograph_item_signers`, while the Oracle signer suggestion query is SQL-filtered and capped with `fetch first 10 rows only`.

The remaining issue is a signer identity regression at the repository boundary. Stale or deleted signer IDs can be treated as authority to create a new profile, which can resurrect merged typo profiles and undo signer merge cleanup. No tests were run during this review; this was a read-through code review.

## Critical Issues

### CR-01: Stale Signer IDs Can Recreate Deleted/Merged Signer Profiles

**Severity:** BLOCKER
**File:** `controller/src/catalog.rs:1478`
**Affected:** `controller/src/oracle_catalog.rs:1001`
**Issue:** Both repository implementations only validate `signerId` when the ID currently resolves. If a request includes a stale/deleted `signerId`, `resolve_signer_profile()` falls through to display-name creation at `controller/src/catalog.rs:1486`, and `resolve_oracle_signer_profile()` does the same at `controller/src/oracle_catalog.rs:1017`. After merging typo profile `Mark Hamel` into canonical `Mark Hamill`, any stale admin form or API client still carrying the deleted source ID plus display name `Mark Hamel` can save an item and silently recreate the deleted typo profile. That breaks the signer merge repair workflow and reintroduces duplicate signer identity after it has been explicitly consolidated.
**Fix:**
```rust
if let Some(signer_id) = input.signer_id {
    let Some(profile) = signers.get_mut(&signer_id) else {
        return Err("signer profile was not found".to_owned());
    };
    validate_signer_id_display_name(profile, input)?;
    update_signer_profile(profile, input, now);
    return Ok(profile.clone());
}
```
Apply the same fail-closed branch in `resolve_oracle_signer_profile()` before falling back to display-name lookup/creation. Keep create-or-select behavior only for requests without `signerId`, and add a regression that merges a typo signer, then verifies saving with the deleted source ID returns an error and does not recreate that source profile.

---

_Reviewed: 2026-07-10T10:52:15Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
