---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T16:10:48Z
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
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T16:10:48Z
**Depth:** standard
**Files Reviewed:** 38
**Status:** issues_found

## Summary

Reviewed the Phase 7 schema updates, backfill scripts/tooling, Rust catalog/Oracle/publisher/routes changes, static admin/public assets, tests, and docs at standard depth. Commit `2edb3d1` does resolve the previous Phase 6-to-Phase 7 ordering blocker: `07-01` now creates `AUTOGRAPH_SIGNERS` before the signer constraint repair blocks touch it. Remaining issues still block shipping the schema/admin workflow cleanly.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Signer Unique Constraint Is Followed By A Duplicate Normalized-Name Index

**Classification:** BLOCKER
**File:** `controller/db/updates/07-01-taxonomy-schema.sql:356`
**Issue:** `AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ` already enforces and indexes `normalized_name` when the signer table is created or repaired, but the migration later tries to create `autograph_signers_normalized_name_idx` on the exact same column list. Oracle rejects duplicate column-list indexes with ORA-01408, so a fresh Phase 7 migration can still abort after the signer tables are created. The canonical fresh schema has the same problem at `controller/db/schema.sql:229`, so empty-environment bootstrap can fail the same way.
**Fix:** Drop the redundant normal index from both schema surfaces, or guard it by checking for any existing index on `AUTOGRAPH_SIGNERS(NORMALIZED_NAME)` rather than only checking the new index name.

```sql
-- Prefer removing this block entirely; the unique constraint already supplies the lookup index.
-- If retained, check USER_IND_COLUMNS for an existing NORMALIZED_NAME-only index first.
```

### CR-02: Existing Signer Profile Link Edits Are Silently Discarded

**Classification:** BLOCKER
**File:** `controller/src/catalog.rs:1482`
**Issue:** The admin form renders editable Wikipedia/IMDb profile URL fields for each signer and includes them in the save payload, but both repositories return immediately when `signer_id` is present (`catalog.rs:1482-1487`, `oracle_catalog.rs:1010-1014`). That means editing links for an existing signer appears to save successfully while the profile changes are ignored. Only new signer profiles persist those fields.
**Fix:** Either route existing-profile edits through `update_signer_profile` before resolving credits, or make `resolve_*_signer_profile` apply provided profile fields to the selected signer after validating the display name. If profile edits are intentionally separate, hide/disable those fields for rows with `signerId` and expose a working profile edit action.

```rust
if let Some(signer_id) = input.signer_id {
    let mut profile = load_or_get_profile(signer_id)?;
    validate_signer_id_display_name(&profile, input)?;
    apply_signer_input_to_profile(&mut profile, input, now)?;
    return upsert_or_store_profile(profile);
}
```

## Warnings

### WR-01: Checked-In Static Fixture Media Paths Do Not Match The Schema V2 Fingerprint Contract

**Classification:** WARNING
**File:** `controller/static-public/data/collection.json:37`
**Issue:** The committed schema v2 fixture still points at `/media/ahsoka-tano/image-1-thumbnail.webp` and `/media/ahsoka-tano/image-1-detail.webp`, while the documented contract requires `/media/{item-slug}/{image-slug}-{variant}-{derivative-fingerprint}.webp`. The real publisher now emits fingerprinted paths, but the checked-in fixture and media files remain unfingerprinted, so the source static example no longer exercises the same cache-busting/privacy contract as generated releases.
**Fix:** Regenerate or update the checked-in static fixture media and JSON to include derivative fingerprints, and add a static fixture assertion alongside the generated-artifact assertion.

---

_Reviewed: 2026-07-10T16:10:48Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
