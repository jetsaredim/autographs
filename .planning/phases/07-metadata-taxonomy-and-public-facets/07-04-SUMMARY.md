---
phase: 07-metadata-taxonomy-and-public-facets
plan: "04"
subsystem: static-public
tags: [rust, static-public, facets, taxonomy, publisher, javascript]

requires:
  - phase: 07-metadata-taxonomy-and-public-facets
    provides: 07-02 Oracle/memory taxonomy persistence and 07-03 admin signer/taxonomy workflow
provides:
  - public schema version 2 DTOs for signer credits, signer text, signer names, signer roles, and first-class taxonomy fields
  - generated public facets for signer, franchise, product line, format, language, origin, role, and tags
  - public detail signer rows with optional Wikipedia/IMDb icon links and hidden default language/origin metadata
  - URL-backed public browse filters for all Phase 7 primary and secondary facets
  - schema v2 static fixture JSON/HTML and source-level public contract tests
affects: [phase-07-rollout-docs, phase-08-ai-assisted-ingest, public-static-contracts]

tech-stack:
  added: []
  patterns:
    - schema-versioned public DTO expansion with Serde camelCase output
    - static publisher privacy-preserving signer/taxonomy projection
    - framework-free URLSearchParams public filtering over generated JSON

key-files:
  created:
    - controller/static-public/data/collection.json
    - controller/static-public/data/facets.json
    - controller/static-public/items/ahsoka-tano/index.html
    - .planning/phases/07-metadata-taxonomy-and-public-facets/07-04-SUMMARY.md
  modified:
    - controller/src/contracts.rs
    - controller/src/publisher.rs
    - controller/static-public/assets/browse.js
    - controller/static-public/assets/site.css
    - controller/tests/publisher.rs
    - controller/tests/static_contract.rs

key-decisions:
  - "Public static artifacts now use schema version 2 and no longer expose Category as a public facet identifier."
  - "Collection cards expose compact signer text and full signer names, while detail pages alone render optional Wikipedia/IMDb profile icon links."
  - "Public browse filters use single-select semantic query params with AND behavior across signer, franchise, productLine, format, language, origin, role, and tag."

patterns-established:
  - "Static public source tests assert browse query keys, recovery copy, v2 fixture JSON, and fixture detail link/default-hiding behavior."
  - "Detail metadata is split into signer rows plus Details/Story/Provenance/Certification groups, with Language: English and Origin: Official suppressed."

requirements-completed: [DATA-03, ADMIN-02, ADMIN-03]

duration: 14min
completed: 2026-07-09
---

# Phase 07-04: Public Schema V2 and Facets Summary

**Schema v2 static public artifacts with multi-signer detail credits and URL-backed taxonomy facets**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-09T19:17:56Z
- **Completed:** 2026-07-09T19:31:21Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Bumped public contracts to schema version 2 with signer credit/link DTOs, compact signer text, full signer names/roles, and first-class taxonomy fields.
- Updated the static publisher to derive signer/franchise/productLine/format/language/origin/role/tag facets from published items and to render detail signer rows with safe optional Wikipedia/IMDb icon links.
- Replaced the public browse script's legacy signer/category/tag state with the Phase 7 semantic filter set, preserving collapsible filters, selected chips, URL sync, and DOM/textContent rendering.
- Refreshed committed public fixture JSON/HTML to schema v2 and added static contract tests for filter copy, fixture examples, and detail profile/default-hiding behavior.

## Task Commits

1. **Task 1 RED: Add failing public schema v2 contract test** - `1bd6390` (test)
2. **Task 1 GREEN: Define public schema v2 contracts** - `4380779` (feat)
3. **Task 2 RED: Add failing publisher taxonomy detail tests** - `bb953e5` (test)
4. **Task 2 GREEN: Generate signer/taxonomy facets and detail metadata** - `1742039` (feat)
5. **Task 3: Update public browse filters and responsive facet styling** - `8159232` (feat)

## Files Created/Modified

- `controller/src/contracts.rs` - Public schema version 2 DTOs, signer credit/profile link structs, and Phase 7 facet identifiers.
- `controller/src/publisher.rs` - Schema v2 item projection, signer text/credit helpers, semantic facet derivation, detail signer rows, profile links, and default metadata hiding.
- `controller/static-public/assets/browse.js` - URL-backed semantic filter state, single-select controls, AND matching, chips, no-results recovery, and facet-load error handling.
- `controller/static-public/assets/site.css` - Responsive primary/secondary facet grids, wrapping chips/actions, and detail profile link touch/focus styling.
- `controller/static-public/data/collection.json` - Schema v2 static fixture with multi-signer, custom-origin, and non-English examples.
- `controller/static-public/data/facets.json` - Schema v2 static fixture with all Phase 7 facet groups.
- `controller/static-public/items/ahsoka-tano/index.html` - Schema v2 detail fixture with signer rows, profile links, and Custom origin display.
- `controller/tests/publisher.rs` - Publisher regressions for compact signer text, semantic facets, detail-only profile links, and default metadata hiding.
- `controller/tests/static_contract.rs` - Static contract regressions for schema v2 DTOs, browse copy/query keys, and checked-in fixtures.

## Decisions Made

- Kept optional external profile links out of collection JSON/cards; only detail pages render Wikipedia/IMDb as icon links.
- Kept facet filtering single-select within each group and ANDed across groups, matching the UI-SPEC first-pass scope.
- Force-staged only the plan-listed static fixture files even though generated public outputs are normally gitignored, because this plan explicitly required refreshed committed fixtures and tests now include them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Applied minimal publisher compatibility during Task 1 contract migration**
- **Found during:** Task 1 (Define public schema version 2 contracts)
- **Issue:** Updating `contracts.rs` to remove `FacetId::Category` and add required schema v2 fields made the existing publisher fail to compile before `static_contract` could deserialize generated artifacts.
- **Fix:** Added the minimal publisher projection needed for schema v2 fixture generation in the Task 1 GREEN commit; Task 2 then completed the richer detail/facet rendering behavior.
- **Files modified:** `controller/src/publisher.rs`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test static_contract -- --nocapture` passed.
- **Committed in:** `4380779`

**2. [Rule 3 - Blocking] Staged plan-listed gitignored static fixtures explicitly**
- **Found during:** Task 3 (Update public browse filters and responsive facet styling)
- **Issue:** `controller/static-public/data/*.json` and `controller/static-public/items/*` are gitignored generated-output paths, but Task 3 required schema v2 fixture JSON/HTML to be regenerated and committed.
- **Fix:** Force-staged only the three plan-listed fixture files after verifying the exact paths and adding source/fixture tests.
- **Files modified:** `controller/static-public/data/collection.json`, `controller/static-public/data/facets.json`, `controller/static-public/items/ahsoka-tano/index.html`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test static_contract -- --nocapture` passed and git status was clean after commit.
- **Committed in:** `8159232`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were necessary to satisfy the plan's own static contract and fixture requirements. No dependency, endpoint, infrastructure, or admin scope was added.

## Issues Encountered

- The Task 3 closeout initially failed `cargo fmt --check` on wrapping-only formatting in `controller/tests/static_contract.rs`; rustfmt was applied and the Task 3 commit was amended to keep the task atomic.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed.
- `node --check controller/static-public/assets/browse.js` - passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_contract -- --nocapture` - passed, 4 tests.
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` - passed, 29 tests.
- `cargo test --manifest-path controller/Cargo.toml` - passed; live smoke tests remain ignored behind explicit live features.

## Known Stubs

None - stub scan found no blocking TODO/FIXME/placeholder data source. Existing `.admin-placeholder` CSS selectors are pre-existing static admin styling, and `state[id] = ""` in `browse.js` is intentional filter reset state.

## Threat Flags

None - the new public DTOs, generated JSON/HTML, external detail links, browser filter surface, and privacy/fail-closed validation are covered by the plan threat model and regression tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-05 can now document rollout, live-smoke checks, security review, codebase-map refresh, and source coverage against schema v2 public contracts. A production rollout should still follow the Phase 7 schema/data-first path, then run a full static rebuild so live public artifacts match the new signer/taxonomy model.

## Self-Check: PASSED

- Key files exist: `controller/src/contracts.rs`, `controller/src/publisher.rs`, `controller/static-public/assets/browse.js`, `controller/static-public/assets/site.css`, `controller/static-public/data/collection.json`, `controller/static-public/data/facets.json`, `controller/static-public/items/ahsoka-tano/index.html`, `controller/tests/publisher.rs`, `controller/tests/static_contract.rs`, and this summary.
- Task commits exist: `1bd6390`, `4380779`, `bb953e5`, `1742039`, and `8159232`.
- Plan-level verification commands passed.
- Stub scan found only documented intentional/pre-existing matches.
- No accidental file deletions were detected after task commits.

---
*Phase: 07-metadata-taxonomy-and-public-facets*
*Completed: 2026-07-09*
