---
phase: 02-oracle-and-private-media-core
plan: 04
status: complete
completed: 2026-05-14
commit: 3c70e1f
requirements:
  - DATA-04
  - MEDIA-02
  - MEDIA-03
---

# Phase 02 Plan 04 Summary

Finished Phase 2 by adding generated media fixtures, app-mediated private image delivery, and the operator smoke path for Oracle plus private Object Storage verification.

## Accomplishments

- Updated the seed path to create representative records through the catalog service and upload generated SVG fixture images instead of only inserting image metadata.
- Added the public app-mediated image route at `/api/catalog/{itemId}/images/{imageId}` for published catalog records.
- Kept the public image route free of Object Storage object keys, signed URLs, bucket credentials, and direct bucket access contracts.
- Expanded the data/media smoke script to create a published smoke item, upload a private image, verify list/detail readback, and optionally verify the deployed app route with `AUTOGRAPHS_SMOKE_BASE_URL`.
- Added `scripts/smoke-data-media.sh` to run migrations, seed sample data, and execute the live data/media smoke in one operator command.
- Updated deployment/config docs and planning status so Phase 3 can assume catalog reads and app-mediated image URLs exist.

## Validation

- `corepack pnpm --filter app typecheck` -> passed
- `corepack pnpm --filter app lint` -> passed
- `corepack pnpm --filter app db:seed:dry-run` -> passed
- `bash -n scripts/smoke-data-media.sh scripts/deploy-vm.sh` -> passed
- `git diff --check` -> passed
- `bash scripts/validate-ci.sh` -> passed
- `rg -n "primary|supporting|publication|certification|inscription|event" app/db/seed app/scripts` -> passed
- `rg -n "Oracle|ADB|Object Storage|seed|media|smoke|private" docs scripts` -> passed
- `rg -n "Phase 2|Oracle and Private Media Core|DATA-01|MEDIA-03|Phase 3" .planning/ROADMAP.md .planning/STATE.md .planning/REQUIREMENTS.md` -> passed

## Live Operator Gate

Live ADB/Object Storage credentials were not available in this local execution context, so the smoke command was documented but not run against OCI. Operator proof command:

```bash
AUTOGRAPHS_SMOKE_BASE_URL=https://autographs.jetsaredim.net bash scripts/smoke-data-media.sh
```

The command requires real Oracle wallet/connect credentials, private Object Storage coordinates, OCI API signing credentials, and an initialized schema. It proves migrations, seed loading, catalog read/list, private object upload/read, and deployed app-mediated image delivery when `AUTOGRAPHS_SMOKE_BASE_URL` is set.

## Deviations from Plan

### [Rule 4 - Adjusted] Documented live proof as an operator gate

- **Found during:** Task 3 smoke verification
- **Issue:** The local execution context does not include live ADB/Object Storage credentials.
- **Fix:** Added and validated the smoke command structure, documented required environment, and recorded the exact operator command for live proof.
- **Files modified:** `scripts/smoke-data-media.sh`, `app/scripts/smoke-data.ts`, `docs/deployment-runbook.md`, `.planning/STATE.md`
- **Verification:** Full CI and static smoke/dry-run checks passed.

**Total deviations:** 1 handled deviation.
**Impact:** Phase 2 implementation is complete and CI-valid; the final live cloud proof remains an explicit operator command rather than hidden tribal knowledge.

## Phase 3 Readiness

- Published records can be listed through `/api/catalog` and read through `/api/catalog/{id}`.
- Published images can be displayed through `/api/catalog/{itemId}/images/{imageId}`.
- Phase 3 gallery work should use app-mediated image URLs and avoid direct Object Storage URLs.
