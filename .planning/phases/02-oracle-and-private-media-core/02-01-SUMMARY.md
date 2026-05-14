---
phase: 02-oracle-and-private-media-core
plan: 01
status: complete
completed: 2026-05-14
commit: b31b677
requirements:
  - DATA-01
  - DATA-02
  - DATA-04
---

# Phase 02 Plan 01 Summary

Added the Oracle catalog data foundation for autograph metadata and image metadata.

## Accomplishments

- Added `node-oracledb`, `oci-sdk`, and `tsx` dependencies for Phase 2 data/media work.
- Added Oracle schema migration support under `app/db/migrations/` and `app/src/db/`.
- Added typed catalog domain types and an Oracle-backed repository for create, update, read, and list operations.
- Added representative seed fixtures covering published/draft records, categories, tags, certification fields, event/source data, and primary/supporting image metadata.
- Added `db:migrate`, `db:seed`, and `db:seed:dry-run` scripts for operator/local verification.
- Extended `.env.example` with Oracle ADB and private media bucket placeholders while keeping real secrets out of VCS.

## Validation

- `rg -n "DATABASE_URL|ORACLE|ADB|WALLET" .env.example app/src app/scripts` -> passed
- `rg -n "autograph|image|primary|publication|migration" app/db app/src/db` -> passed
- `rg -n "seed|sample|autograph" app/package.json app/scripts app/db` -> passed
- `corepack pnpm --filter app typecheck` -> passed
- `corepack pnpm --filter app lint` -> passed
- `corepack pnpm --filter app db:seed:dry-run` -> passed with 3 records and primary/supporting image coverage

## Deviations from Plan

### [Rule 3 - Blocking] Encoded pnpm build-script policy for new dependencies

- **Found during:** Task 1 validation
- **Issue:** `pnpm` blocked validation because new dependencies introduced build scripts for `esbuild` and `oracledb`.
- **Fix:** Updated `pnpm-workspace.yaml` build policy so `esbuild` can build and `oracledb` remains ignored for Thin-mode usage; also documented ignored built dependencies in root `package.json`.
- **Files modified:** `pnpm-workspace.yaml`, `package.json`
- **Verification:** `corepack pnpm install`, `corepack pnpm --filter app typecheck`, and `corepack pnpm --filter app lint` passed.

### [Rule 3 - Blocking] Added local TypeScript declaration for node-oracledb

- **Found during:** Task 1 validation
- **Issue:** `oracledb@6.10.0` did not expose TypeScript declarations usable by the current compiler setup.
- **Fix:** Added a narrow local declaration file for the subset of `oracledb` used by the app.
- **Files modified:** `app/src/types/oracledb.d.ts`, `app/src/db/oracle.ts`
- **Verification:** `corepack pnpm --filter app typecheck` passed.

**Total deviations:** 2 auto-fixed blocking issues.
**Impact:** No scope change; both fixes were required to make the planned Oracle foundation compile and validate repeatably.

## Next Phase Readiness

- `02-02` can extend Terraform/configuration knowing the app now has explicit Oracle and media environment variable names.
- `02-03` can build the private Object Storage adapter and route/service seams on top of the catalog repository and image metadata model.
