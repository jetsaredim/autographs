---
phase: 07-metadata-taxonomy-and-public-facets
reviewed: 2026-07-10T02:09:58Z
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
  warning: 3
  info: 0
  total: 6
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-07-10T02:09:58Z
**Depth:** standard
**Files Reviewed:** 37
**Status:** issues_found

## Summary

Reviewed the Phase 7 taxonomy schema, Rust catalog/controller/publisher paths, static admin/public assets, tests, generated fixture artifacts, and operator docs. The main risks are unsafe public signer profile links, admin signer-row saves that can drop remaining credits after row removal, and item-level role edits mutating shared signer defaults.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Unsafe Signer Profile URLs Reach Public `href` Attributes

**Severity:** BLOCKER
**File:** `controller/src/catalog.rs:1541`, `controller/src/oracle_catalog.rs:1268`, `controller/src/publisher.rs:1909`
**Issue:** Signer profile URL validation only checks length. The publisher then renders non-empty `wikipediaUrl` and `imdbUrl` directly into public detail-page `href` attributes. HTML escaping prevents tag injection, but it does not make schemes like `javascript:alert(1)` or `data:text/html,...` safe; a malicious or mistaken admin value can become an executable public link.
**Fix:**
```rust
fn validate_profile_url(value: Option<&str>, field: &str) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if value.len() > MAX_PROFILE_URL_LENGTH {
        return Err(format!("{field} must be {MAX_PROFILE_URL_LENGTH} characters or fewer"));
    }
    let url = url::Url::parse(value).map_err(|_| format!("{field} must be a valid HTTPS URL"))?;
    if url.scheme() != "https" {
        return Err(format!("{field} must use https"));
    }
    match field {
        "wikipediaUrl" if !url.domain().is_some_and(|d| d.ends_with("wikipedia.org")) => {
            return Err("wikipediaUrl must point to wikipedia.org".to_owned());
        }
        "imdbUrl" if !url.domain().is_some_and(|d| d.ends_with("imdb.com")) => {
            return Err("imdbUrl must point to imdb.com".to_owned());
        }
        _ => {}
    }
    Ok(())
}
```
Share this validation between memory and Oracle paths, and add publisher/API tests that reject `javascript:` and non-HTTPS profile URLs.

### CR-02: Removing A Signer Row Can Drop Remaining Signers On Save

**Severity:** BLOCKER
**File:** `controller/static-admin/admin.js:595`
**Issue:** The remove handler deletes a row and calls `normalizeSignerRowHeadings()`, but that function only updates `data-index` and the visible heading. `signerCreditPayload()` later reads each row by index-specific IDs such as `signer-name-0`. After removing the first signer, the remaining row still contains `signer-name-1`, so the payload for row index `0` reads `null` fields and filters the signer out. Saving after a row removal can silently remove the wrong signer credits from the item.
**Fix:**
```javascript
function signerCreditPayload() {
  return Array.from(elements.signerRows.children)
    .map((row) => {
      const value = (selector) => row.querySelector(selector)?.value.trim() || null;
      return {
        signerId: row.dataset.signerId || null,
        displayName: value(".signer-name-input"),
        itemRole: value(".signer-role-input"),
        itemContext: value(".signer-context-input"),
        wikipediaUrl: value(".signer-wikipedia-input"),
        imdbUrl: value(".signer-imdb-input"),
      };
    })
    .filter((credit) => credit.displayName || credit.signerId);
}
```
Alternatively, fully renumber every input `id`, `name`, `label[for]`, and profile panel ID after remove/reorder. Add a static admin test that removes the first of two signer rows and verifies the second signer remains in the payload.

### CR-03: Item Role Edits Mutate Shared Signer Defaults

**Severity:** BLOCKER
**File:** `controller/static-admin/admin.js:968`
**Issue:** The admin payload sends the same Role input as both `defaultRole` and `itemRole`. Repository code treats `defaultRole` as signer-profile metadata, so editing a signer role for one item can overwrite the reusable signer's default role for every linked and future item. This is cross-item metadata corruption from an item editor field.
**Fix:**
```javascript
return {
  signerId: row.dataset.signerId || null,
  displayName: value("name"),
  itemRole: value("role"),
  itemContext: value("context"),
  wikipediaUrl: value("wikipedia"),
  imdbUrl: value("imdb"),
};
```
Only send `defaultRole` from a clearly separate signer-profile edit control, or only when creating a new profile and the UI labels it as the reusable default.

## Warnings

### WR-01: Update Validation Failures Are Reported As 500s

**Severity:** WARNING
**File:** `controller/src/routes.rs:1130`
**Issue:** `repository_update_error_status()` maps only the shared required-fields error to `400`; other validation failures such as unsupported language, blank format, duplicate signer credits, or overlong profile URLs become `500 Internal Server Error` on item update/publication routes. These are client-correctable validation errors, and treating them as server failures hides actionable feedback and pollutes operational error logs.
**Fix:** Return typed repository errors, or extend the mapper for known validation messages:
```rust
fn repository_update_error_status(error: &str) -> StatusCode {
    if error == REQUIRED_FIELDS_ERROR
        || error.contains("required")
        || error.contains("must be")
        || error.contains("duplicate signer credits")
        || error.contains("not allowed")
    {
        StatusCode::BAD_REQUEST
    } else if error.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
```

### WR-02: Duplicate Taxonomy Tokens Can Fail Oracle Saves

**Severity:** WARNING
**File:** `controller/static-admin/admin.js:528`, `controller/src/oracle_catalog.rs:1535`, `controller/src/oracle_catalog.rs:1705`
**Issue:** The admin `splitList()` trims but does not deduplicate values, and the Oracle persistence layer inserts tags, characters, and franchises directly into tables whose primary keys include the value. A user entering `Star Wars, Star Wars` or duplicate tags can make the Oracle save fail with a constraint error, while memory mode accepts the same payload.
**Fix:** Normalize and deduplicate list fields before persistence and in the admin payload:
```javascript
const splitList = (value) =>
  [...new Set(String(value || "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean))];
```
Mirror the same dedupe in Rust before `replace_tags()` and `replace_ordered_values()` so direct API callers behave consistently.

### WR-03: Static Runtime Runbook Create Payload No Longer Matches The API

**Severity:** WARNING
**File:** `docs/static-runtime-runbook.md:55`
**Issue:** The documented `POST /admin/api/items` payload omits required `signer` and `category` compatibility fields and uses `role` inside `signerCredits`, but the API expects `itemRole` and ignores unknown `role`. An operator following this smoke step will fail item creation or create an item without the intended signer role.
**Fix:**
```json
{
  "title": "Signed card",
  "signer": "Example Signer",
  "category": "Trading Card",
  "signerCredits": [{"displayName": "Example Signer", "itemRole": "actor"}],
  "format": "Trading Card",
  "origin": "Official",
  "language": "English",
  "franchises": ["Example Franchise"],
  "tags": ["fixture"]
}
```

---

_Reviewed: 2026-07-10T02:09:58Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
