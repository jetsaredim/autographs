---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T03:29:20Z
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
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T03:29:20Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the full Phase 7 schema, Rust controller, static admin/public, test, and documentation scope at standard depth. The follow-up commits appear to have resolved the earlier direct profile URL validation and duplicate taxonomy-list persistence findings: both memory and Oracle paths now validate profile hosts, and both create/update paths normalize duplicate tag/character/franchise lists before persistence. The remaining issues are in the signer identity handoff, the live backfill script shape, and production signer suggestion completeness.

No tests were run during this review; this was a read-through code review.

## Critical Issues

### CR-01: Editing A Signer Name Can Rename The Existing Reusable Profile

**Severity:** BLOCKER
**File:** `controller/static-admin/admin.js:556`
**Issue:** Loaded signer rows store the existing profile ID in `row.dataset.signerId` at lines 556-558, but the `input` handler at lines 617-620 never clears that hidden ID when the visible signer name changes. The save payload then sends both the stale `signerId` and the newly typed `displayName` at lines 981-994. Both repository implementations trust `signerId` first (`controller/src/catalog.rs:1478` and `controller/src/oracle_catalog.rs:1001`), apply the new display name to that existing profile, and persist it through `upsert_signer_profile()` (`controller/src/oracle_catalog.rs:1633`). A routine item edit can therefore rename a shared signer profile and change every item linked to that signer instead of assigning this item to a new or different signer.
**Fix:**
```javascript
let selectedSignerName = profileValue(credit, "displayName").trim();

nameInput.addEventListener("input", async () => {
  if (nameInput.value.trim() !== selectedSignerName) {
    delete row.dataset.signerId;
  }
  await loadSignerSuggestions(nameInput.value);
  renderDuplicateWarnings();
});

nameInput.addEventListener("change", () => {
  const selected = state.signerSuggestions.find(
    (suggestion) => suggestion.profile.displayName === nameInput.value.trim()
  );
  if (selected) {
    row.dataset.signerId = selected.profile.id;
    selectedSignerName = selected.profile.displayName;
  } else {
    delete row.dataset.signerId;
    selectedSignerName = "";
  }
  renderDuplicateWarnings();
});
```
Also harden the API boundary by rejecting a payload that includes `signerId` plus a conflicting `displayName` unless the normalized name matches the loaded profile, reserving profile renames for the explicit signer-profile update route.

### CR-02: Taxonomy Backfill Does Not Materialize Legacy Signers Into The New Signer Tables

**Severity:** BLOCKER
**File:** `controller/src/taxonomy_migration.rs:178`
**Issue:** The schema migration creates empty `autograph_signers` and `autograph_item_signers` tables (`controller/db/updates/07-01-taxonomy-schema.sql:151` and `controller/db/updates/07-01-taxonomy-schema.sql:183`), but the generated apply script only updates `autograph_item_signers.item_role` for existing rows (`controller/src/taxonomy_migration.rs:237` and `controller/db/updates/07-03-taxonomy-backfill-apply.sql:18`). Existing live items have only legacy `autograph_items.signer` values, so those role updates affect zero rows and no reusable signer profiles are created. The controller can fall back to ephemeral legacy signer credits when loading an item (`controller/src/oracle_catalog.rs:897`), but taxonomy suggestions remain empty/incomplete and the migrated role data is lost until each item is manually saved.
**Fix:**
```sql
merge into autograph_signers signer
using (
  select '<stable-signer-uuid>' id,
         'Mark Hamill' display_name,
         'mark hamill' normalized_name
  from dual
) incoming
on (signer.normalized_name = incoming.normalized_name)
when not matched then
  insert (id, display_name, normalized_name)
  values (incoming.id, incoming.display_name, incoming.normalized_name);

insert into autograph_item_signers (item_id, signer_id, sort_order, item_role)
select '<item-id>', signer.id, 0, 'actor'
from autograph_signers signer
where signer.normalized_name = 'mark hamill'
  and not exists (
    select 1 from autograph_item_signers existing
    where existing.item_id = '<item-id>' and existing.signer_id = signer.id
  );
```
Generate those signer/profile statements from the legacy export before role updates, and keep ambiguous multi-signer strings such as slash-delimited names in the report/manual-review bucket unless they can be split safely. Add a regression that asserts generated PL/SQL inserts or merges signer profiles and item signer credits, not only item-level taxonomy fields.

## Warnings

### WR-01: Oracle Signer Suggestions Only Search The First 50 Profiles

**Severity:** WARNING
**File:** `controller/src/oracle_catalog.rs:1098`
**Issue:** `load_all_signer_profiles()` applies `fetch first 50 rows only` before any caller-specific filtering. `signer_suggestions()` then searches only those 50 rows at lines 737-764, so a production collection with more than 50 signers can miss exact or near matches that sort after the cap. `taxonomy_suggestions()` also exposes only that capped list at lines 1167-1170, which undermines the Phase 7 requirement that reusable profiles and duplicate warnings work before save/publish.
**Fix:**
```sql
select id, display_name, normalized_name, default_role, wikipedia_url, imdb_url, ...
from autograph_signers
where normalized_name = :exact
   or normalized_name like :prefix
   or normalized_name like :contains
order by case when normalized_name = :exact then 0 else 1 end, display_name, id
fetch first 10 rows only
```
Use a query-specific SQL search for `signer_suggestions()` and either return all signer profiles for taxonomy suggestions or make the endpoint explicitly paginated/capped with UI handling so missing profiles are not silently treated as absent.

---

_Reviewed: 2026-07-10T03:29:20Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
