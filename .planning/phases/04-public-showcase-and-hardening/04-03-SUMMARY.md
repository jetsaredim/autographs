---
phase: 04-public-showcase-and-hardening
plan: 03
subsystem: docs
tags: [readme, metadata, badges, showcase, gsd]

requires:
  - phase: 04-public-showcase-and-hardening
    provides: Security review and dependency update policy from Plans 04-01 and 04-02
provides:
  - Public showcase README
  - Accurate quality signal badges
  - Current app metadata
affects: [04-public-showcase-and-hardening, public-review, operations]

tech-stack:
  added: []
  patterns: [public-showcase-readme, honest-status-badges, planned-vs-current-scope]

key-files:
  created: []
  modified:
    - README.md
    - app/app/layout.tsx
    - package.json

key-decisions:
  - "Use badges only for real workflows and a static Renovate configured signal."
  - "Frame Data Smoke as manual because it depends on real Oracle/Object Storage credentials."
  - "Label Phase 5 admin and Phase 6 AI as planned rather than current."

patterns-established:
  - "Public README should separate implemented, planned, and out-of-scope capabilities."
  - "Showcase documentation should link to operational/security docs without exposing private OCI details."

requirements-completed: [SHIP-03, SHIP-04, SHIP-05]

duration: 3 min
completed: 2026-05-25
---

# Phase 04 Plan 03: Public Showcase Summary

**Public README with honest badges, current architecture, operations links, and human+AI/GSD story**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-25T20:20:04Z
- **Completed:** 2026-05-25T20:23:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Replaced the placeholder README with a public-facing showcase entry point.
- Added badges for CI, Deploy, Data Smoke, Image Cleanup, and Renovate configuration.
- Documented current implemented scope, planned Phase 5 admin work, planned Phase 6 AI work, and v1 out-of-scope items.
- Updated app/package metadata to describe the current project accurately.

## Task Commits

Plan tasks were completed in one cohesive documentation commit:

1. **Tasks 1-3: README, badges, metadata, and status-language verification** - `f744ecc` (docs)

**Plan metadata:** pending in this summary commit.

## Files Created/Modified

- `README.md` - Public showcase entry point.
- `app/app/layout.tsx` - Updated public app metadata.
- `package.json` - Added repository package description.

## Decisions Made

- Did not add a live URL or uptime claim to avoid implying a guarantee not backed by current checks.
- Kept badges tied to existing workflows and marked Data Smoke as manual in prose.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `test -s README.md && rg -n "GSD|human\\+AI|Architecture|Local Development|Deployment|Security|Phase 5|Phase 6" README.md` - passed
- `rg -n "badge|actions/workflows|CI|Deploy|Data Smoke|Renovate" README.md` - passed
- `rg -n "docs/|Phase 5|Phase 6|out of scope|temporary operator" README.md && ! rg -n "AI metadata processing is implemented|Phase 4 admin|direct Object Storage URL" README.md` - passed
- `corepack pnpm --filter app lint` - passed

## Next Phase Readiness

Plan 04-04 can reconcile stale docs, diagrams, and codebase maps against the new README and the Phase 4/5/6 boundaries.

---
*Phase: 04-public-showcase-and-hardening*
*Completed: 2026-05-25*
