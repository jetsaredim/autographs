---
phase: 03-public-gallery-mvp
plan: 01
subsystem: ui
tags: [nextjs, catalog, public-view-models, privacy, quotes]

requires:
  - phase: 02-oracle-and-private-media-core
    provides: Oracle-backed catalog records and app-mediated private image routes
provides:
  - Public-safe gallery and detail view models
  - App-mediated public image URL builder
  - Curated public facet derivation
  - Approved quote inventory for public empty and error states
affects: [03-public-gallery-mvp, public-gallery, collection-detail, empty-states]

tech-stack:
  added: [node-test-script]
  patterns: [server-safe public view models, deterministic approved quote selection]

key-files:
  created:
    - app/src/catalog/public-view-models.ts
    - app/src/catalog/public-view-models.test.ts
    - app/src/gallery/approved-quotes.ts
    - app/src/gallery/approved-quotes.test.ts
  modified:
    - app/package.json

key-decisions:
  - "Public image references use only /api/catalog/{itemId}/images/{imageId} paths."
  - "Public item outputs intentionally omit every private Object Storage image field."
  - "Approved public quote inventory is a durable TypeScript module for Phase 3, not a schema change."
  - "The approved quote inventory contains 20 short attributed quotes selected by the user."

patterns-established:
  - "Public view models are transformed from AutographItem before reaching public UI components."
  - "Public images expose id, src, and altText only."
  - "Curated facets are grouped as signer, category, and tag options."
  - "Approved quotes are selected deterministically when a seed is provided."

requirements-completed: [GALL-01, GALL-02, GALL-03, GALL-04]

duration: 12min
completed: 2026-05-21
---

# Phase 03: Public Gallery MVP Summary

**Public-safe catalog view models and approved quote inventory for Phase 3 gallery surfaces**

## Performance

- **Duration:** 12 min
- **Started:** 2026-05-21T17:39:19Z
- **Completed:** 2026-05-21T17:51:47Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added a `test` script for the app package using Node's built-in test runner through `tsx`.
- Created typed public gallery/detail view models that strip private media fields and build app-mediated image URLs.
- Added curated public facet derivation for signer, category, and tag filters.
- Added a durable approved quote inventory with 20 short, attributed quote objects and deterministic selection.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add public view model contract tests** - `a536a88` (test)
2. **Task 1 GREEN: Add sanitized public catalog view models** - `d3e4546` (feat)
3. **Task 3: Store approved quotes in durable source** - `712b156` (feat)

**Plan metadata:** pending in this summary commit.

## Files Created/Modified

- `app/package.json` - Adds the app-level `test` script.
- `app/src/catalog/public-view-models.ts` - Provides public item/detail models, public image URLs, and curated facets.
- `app/src/catalog/public-view-models.test.ts` - Covers private-field sanitization, image route construction, primary image selection, facets, and detail groups.
- `app/src/gallery/approved-quotes.ts` - Stores the user-approved quote inventory and selector.
- `app/src/gallery/approved-quotes.test.ts` - Covers quote count, structure, length, uniqueness, and selection.

## Decisions Made

- Stored approved quotes as a TypeScript source module because Phase 3 needs a durable approved list without adding schema or migration scope.
- Included all eight originally approved quotes plus twelve additional short sourced quotes to reach a full inventory of 20.
- Kept `tone` in each quote object so later empty-state UI can choose or audit editorial flavor without parsing quote text.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The initial null-state test helper used `??` defaults, which unintentionally replaced explicit `null` test values. The helper was corrected so empty detail groups are tested honestly.
- Typecheck caught optional `description` and nullable image array typing in the view-model implementation; both were normalized before commit.

## User Setup Required

None - no external service configuration required.

## Verification

- `corepack pnpm --filter app test` - passed
- `corepack pnpm --filter app lint` - passed
- `corepack pnpm --filter app typecheck` - passed
- `rg "storageNamespace|bucketName|objectKey|checksum|etag" app/src/catalog/public-view-models.ts` - no matches
- No files under `app/db/migrations/` were modified

## Next Phase Readiness

Phase 3 public UI plans can now consume `toPublicGalleryItem`, `toPublicItemDetail`, `derivePublicFacets`, and the shared approved quote inventory without exposing private Object Storage fields.

---
*Phase: 03-public-gallery-mvp*
*Completed: 2026-05-21*
