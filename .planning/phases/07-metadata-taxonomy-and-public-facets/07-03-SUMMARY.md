---
phase: 07-metadata-taxonomy-and-public-facets
plan: "03"
subsystem: admin-ui
tags: [rust, axum, static-admin, taxonomy, signer-profiles]

requires:
  - phase: 07-metadata-taxonomy-and-public-facets
    provides: 07-02 Oracle and memory repository signer suggestions, profile edits, merge repair, and taxonomy suggestions
provides:
  - session-only admin signer suggestion, signer profile update, signer merge, and taxonomy suggestion API routes
  - taxonomy-aware admin item filters, summaries, and detail DTO fields
  - framework-free static admin editor sections for Identity, Classification, Details, Publication, Images, and History
  - signer rows with suggestions, duplicate warnings, optional profile links, merge repair, and taxonomy payload saving
  - UI-SPEC constrained admin styling and source-level accessibility/privacy tests
affects: [phase-07-public-facets, phase-07-rollout-docs, phase-08-ai-assisted-ingest]

tech-stack:
  added: []
  patterns:
    - session-cookie-only admin route handlers for taxonomy management
    - static admin DOM creation with textContent for signer/taxonomy values
    - source-level static admin tests for privacy, labels, ARIA hooks, and UI-SPEC styling constraints

key-files:
  created:
    - .planning/phases/07-metadata-taxonomy-and-public-facets/07-03-SUMMARY.md
  modified:
    - controller/src/routes.rs
    - controller/src/routes/admin_items.rs
    - controller/src/catalog_admin.rs
    - controller/static-admin/index.html
    - controller/static-admin/admin.js
    - controller/static-admin/admin.css
    - controller/tests/admin_workflow.rs
    - controller/tests/static_admin.rs

key-decisions:
  - "Admin signer/taxonomy management routes use the same session-cookie-only boundary as collection management and reject bearer operator tokens."
  - "Admin item summaries no longer expose legacy category as a primary field; they return signerText, signerNames, format, franchises, productLine, language, publicationStatus, imageCount, pending-change state, and update time."
  - "The static admin editor keeps legacy signer/category compatibility in save payloads while making signerCredits and first-class taxonomy fields the primary editing path."

patterns-established:
  - "Signer rows render dynamic API values with DOM creation and textContent, with optional Wikipedia/IMDb fields behind an aria-expanded inline panel."
  - "Taxonomy source tests assert required copy, payload fields, no browser storage, no gradients, visible labels, ARIA state hooks, and focus styling."

requirements-completed: [DATA-03, ADMIN-02, ADMIN-03]

duration: 35min
completed: 2026-07-09
---

# Phase 07-03: Admin Taxonomy Workflow Summary

**Session-protected admin signer/taxonomy APIs and a framework-free taxonomy editor with reusable signer rows and merge repair**

## Performance

- **Duration:** 35 min
- **Started:** 2026-07-09T18:38:00Z
- **Completed:** 2026-07-09T19:13:18Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added authenticated admin routes for signer suggestions, signer profile edits, signer merge repair, and taxonomy suggestions, all behind `authorize_admin_session`.
- Reworked admin item list/detail DTOs and filters around Phase 7 signer/taxonomy fields while preserving existing image, history, diagnostics, save, and publish behavior.
- Rebuilt the static admin editor around Identity, Classification, Details, Publication, Images, and History with signer rows, duplicate warnings, merge repair, suggestions, and taxonomy payloads.
- Added UI-SPEC styling for signer rows, token editors, duplicate warnings, merge panels, suggestions, focus states, wrapping, and no-gradient/privacy constraints.

## Task Commits

1. **Task 1 RED: Add failing admin taxonomy API tests** - `4cbeb8f` (test)
2. **Task 1 GREEN: Expose admin signer taxonomy APIs** - `edacce4` (feat)
3. **Task 2 RED: Add failing static admin taxonomy tests** - `8a51c00` (test)
4. **Task 2 GREEN: Rebuild admin taxonomy editor** - `78f04c9` (feat)
5. **Task 3: Style admin taxonomy controls** - `443a037` (feat)

## Files Created/Modified

- `controller/src/routes.rs` - Added signer/taxonomy route wiring and taxonomy fields in admin item detail responses.
- `controller/src/routes/admin_items.rs` - Added signer/taxonomy handlers, camelCase DTOs, merge/update requests, richer item summaries, and changes filtering.
- `controller/src/catalog_admin.rs` - Added signer/title/franchise/productLine/format/language/publicationStatus filtering across first-class taxonomy fields.
- `controller/static-admin/index.html` - Replaced legacy primary signer/category/tag layout with Identity, Classification, Details, Publication, Images, and History sections.
- `controller/static-admin/admin.js` - Added signer rows, suggestions, duplicate warnings, merge repair, taxonomy suggestions, taxonomy payloads, and updated item-list rendering.
- `controller/static-admin/admin.css` - Added UI-SPEC constrained taxonomy editor, signer row, warning, merge, token editor, wrapping, and focus styles.
- `controller/tests/admin_workflow.rs` - Added route/auth/filter/redaction tests for signer and taxonomy admin APIs.
- `controller/tests/static_admin.rs` - Added static source tests for taxonomy editor copy, payloads, labels, ARIA hooks, styling selectors, privacy, and no-gradient/browser-storage constraints.

## Decisions Made

- Kept route logic in the existing `routes/admin_items.rs` module because these endpoints share collection-management auth, DTO redaction, and repository boundaries.
- Kept legacy `signer` and `category` in save payloads as compatibility fields while the visible admin workflow now prioritizes `signerCredits` and `format`.
- Implemented optional signer profile URLs behind an inline `aria-expanded` panel rather than making Wikipedia/IMDb fields required primary controls.

## Deviations from Plan

None - plan executed exactly as written.

**Total deviations:** 0 auto-fixed.
**Impact on plan:** No scope changes or unplanned dependency additions.

## Issues Encountered

- The Task 1 GREEN pass initially hit a Rust visibility error for the signer query extractor; making the extractor `pub(super)` fixed route wiring without changing the endpoint contract.
- `cargo fmt --check` found wrapping-only formatting in the new static admin test; rustfmt was applied and the Task 3 commit was amended to keep the task atomic.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed.
- `node --check controller/static-admin/admin.js` - passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` - passed, 11 tests.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` - passed, 20 tests.
- `cargo test --manifest-path controller/Cargo.toml --test auth_and_health -- --nocapture` - passed, 9 tests.
- Task-focused checks also passed: `cargo test --manifest-path controller/Cargo.toml --test admin_workflow signer -- --nocapture` and `cargo test --manifest-path controller/Cargo.toml --test admin_workflow taxonomy -- --nocapture`.

## Known Stubs

None - stub scan found no blocking mock-data or placeholder UI paths. The remaining `placeholder` attributes are user-facing input examples for characters, franchise, and loose tags, not unwired data sources.

## Threat Flags

None - the new admin routes, signer merge behavior, DTO redaction, and DOM rendering surfaces were covered by the plan threat model and verified by route/static tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-04 can now consume the richer admin-side taxonomy model and expose schema-versioned public collection/detail/facet artifacts. The admin workflow is ready to create and edit repeatable signers, controlled format/origin/language, characters, franchise, product line, set, and loose tags while preserving save/publish separation.

## Self-Check: PASSED

- Key files exist: `controller/src/routes.rs`, `controller/src/routes/admin_items.rs`, `controller/src/catalog_admin.rs`, `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/tests/admin_workflow.rs`, `controller/tests/static_admin.rs`, and this summary.
- Task commits exist: `4cbeb8f`, `edacce4`, `8a51c00`, `78f04c9`, and `443a037`.
- Plan-level verification commands passed.
- Stub scan found only intentional input example placeholders and normal empty/null handling.
- No accidental file deletions were detected after task commits.

---
*Phase: 07-metadata-taxonomy-and-public-facets*
*Completed: 2026-07-09*
