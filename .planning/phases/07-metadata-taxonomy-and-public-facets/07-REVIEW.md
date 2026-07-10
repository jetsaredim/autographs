---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T03:18:01Z
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

**Reviewed:** 2026-07-10T03:18:01Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Summary

Re-reviewed the Phase 7 taxonomy/schema/admin/public/docs scope after commit `561d840`. Previous findings CR-02, CR-03, WR-01, and WR-03 are resolved in the submitted fixes. Previous CR-01 is only fixed in the memory repository and remains open in the Oracle repository, which is the production persistence path. Previous WR-02 is only fixed in the browser payload path and remains open for direct API callers and Oracle persistence.

Validation run:

- `cargo test --manifest-path controller/Cargo.toml` passed.
- Targeted regression tests for memory-mode profile URL validation, admin validation status mapping, and static-admin row-scoped signer payloads passed.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Oracle Profile URL Validation Still Allows Unsafe Public `href` Values

**Severity:** BLOCKER
**File:** `controller/src/oracle_catalog.rs:1268`
**Issue:** Commit `561d840` tightened `validate_profile_url()` in the memory repository, but the Oracle repository still only checks profile URL length at lines 1268-1273. Production create/update calls resolve signer credits through `resolve_oracle_signer_credits()` and `apply_signer_input_to_profile()`, so an admin or direct API caller can still persist `javascript:alert(1)` or `data:` values in Oracle signer profile links. The publisher renders non-empty profile links into public detail-page `href` attributes at `controller/src/publisher.rs:1909` and `controller/src/publisher.rs:1921`, so this remains a public XSS/security issue in production mode.
**Fix:**
```rust
fn validate_profile_url(value: Option<&str>, field: &str) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if value.len() > 1000 {
        return Err(format!("{field} must be 1000 characters or fewer"));
    }
    let Some(rest) = value.strip_prefix("https://") else {
        return Err(format!("{field} must be an https URL"));
    };
    let host = rest
        .split(['/', '?', '#', ':'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let allowed = match field {
        "wikipediaUrl" => host == "wikipedia.org" || host.ends_with(".wikipedia.org"),
        "imdbUrl" => host == "imdb.com" || host.ends_with(".imdb.com"),
        _ => false,
    };
    if !allowed {
        return Err(format!(
            "{field} must point to {}",
            if field == "wikipediaUrl" { "wikipedia.org" } else { "imdb.com" }
        ));
    }
    Ok(())
}
```
Prefer sharing one validator between `catalog.rs` and `oracle_catalog.rs`, and add an Oracle/live-persistence or unit-seam regression test so production persistence cannot drift from memory-mode validation again.

## Warnings

### WR-01: Oracle Saves Still Fail On Duplicate Direct-API Taxonomy Values

**Severity:** WARNING
**File:** `controller/src/oracle_catalog.rs:1527`, `controller/src/oracle_catalog.rs:1693`
**Issue:** The browser `splitList()` now deduplicates admin-entered taxonomy lists, but Oracle persistence still inserts the caller-provided `tags`, normalized `characters`, and normalized `franchises` vectors directly into tables whose primary keys are `(item_id, tag)`, `(item_id, character_name)`, and `(item_id, franchise)`. Direct API callers, tests, migration tools, or any non-browser client can still submit duplicate taxonomy values and trigger Oracle constraint errors during `replace_tags()` or `replace_ordered_values()`, while memory mode accepts the same payload. This leaves the previous duplicate-token production robustness issue unresolved outside the static admin UI.
**Fix:** Deduplicate in the Rust persistence/service boundary before Oracle insert loops, not only in browser JavaScript. For example:
```rust
fn normalize_unique_string_list(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    normalize_string_list(values)
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}
```
Use that for tags, characters, and franchises on create/update before calling `replace_tags()`, `replace_characters()`, and `replace_franchises()`, and add an Oracle-path regression test for duplicate direct-API taxonomy values.

---

_Reviewed: 2026-07-10T03:18:01Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
