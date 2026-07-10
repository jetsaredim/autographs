---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T11:07:36Z
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
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T11:07:36Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

Reviewed the listed Phase 7 schema, Rust controller, static admin/public, tests, and documentation at standard depth. Commit `bd6ae3b` does resolve the specific item-save regression where selecting an existing signer or reusing an exact signer name cleared profile metadata: the item signer-credit resolution paths now return the existing profile instead of applying blank item-row optional fields, and `controller/tests/admin_workflow.rs:1603` covers both selected-ID reuse and exact-name reuse while preserving new signer creation with optional links/default role.

The remaining issues are adjacent data-loss paths: the signer-profile PATCH endpoint still clears omitted optional profile fields, and the generated taxonomy backfill can silently collapse multiple role mappings for one legacy item.

## Critical Issues

### CR-01: Partial Signer Profile PATCH Clears Omitted Profile Metadata

**Severity:** BLOCKER
**File:** `controller/src/catalog.rs:1592`
**Affected:** `controller/src/routes/admin_items.rs:284`, `controller/src/routes/admin_items.rs:293`, `controller/src/oracle_catalog.rs:783`
**Issue:** `PATCH /admin/api/signers/{id}` cannot distinguish omitted optional fields from intentional clears. `AdminSignerUpdateRequest` uses plain `Option<String>` fields, so a request body such as `{"displayName":"Mark Richard Hamill"}` deserializes `defaultRole`, `wikipediaUrl`, and `imdbUrl` as `None`. `apply_signer_profile_update()` then normalizes each `None` to `None` and writes it over the existing reusable profile. A partial profile edit can erase the default role and public profile links across both memory and Oracle repositories.
**Fix:**
```rust
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerProfileUpdateInput {
    #[serde(default)]
    pub display_name: FieldPatch<String>,
    #[serde(default)]
    pub default_role: FieldPatch<String>,
    #[serde(default)]
    pub wikipedia_url: FieldPatch<String>,
    #[serde(default)]
    pub imdb_url: FieldPatch<String>,
}
```
Then update `apply_signer_profile_update()` so `FieldPatch::Unchanged` preserves the current value, `FieldPatch::Clear` clears it, and `FieldPatch::Set(value)` validates/normalizes and writes it. Add a regression that creates a signer with `default_role`, `wikipedia_url`, and `imdb_url`, calls `update_signer_profile()` with only `display_name`, and asserts the three optional fields remain unchanged.

## Warnings

### WR-01: Backfill Generator Silently Drops Multiple Role Mappings Per Item

**Severity:** WARNING
**File:** `controller/src/taxonomy_migration.rs:314`
**Issue:** `mapped_roles_by_item()` collects mapped `role` rows into `BTreeMap<String, String>`, so if a legacy row has more than one role-like category/tag, the later role overwrites the earlier one without being reported. The generated PL/SQL then inserts the signer credit with only that single surviving role at `controller/src/taxonomy_migration.rs:235`, and the later role update statements only run when `item_role is null` at `controller/src/taxonomy_migration.rs:301`, so the discarded role never reaches the new signer-credit taxonomy. The original loose tags remain, but schema-v2 role facets and signer credits lose one of the operator-reviewed mappings.
**Fix:** Detect more than one mapped role per `item_id` during report generation and classify those rows as `NeedsReview` instead of emitting an automatic `item_role`. Alternatively, model roles as signer-specific review input before generating PL/SQL, so the script never chooses an arbitrary winner.

---

_Reviewed: 2026-07-10T11:07:36Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
