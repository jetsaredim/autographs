---
phase: 04-public-showcase-and-hardening
plan: 01
subsystem: security
tags: [nextjs, headers, health-checks, caddy, security-review]

requires:
  - phase: 03-public-gallery-mvp
    provides: Public gallery, public catalog APIs, image delivery, and operator-only bridge docs
provides:
  - Baseline public security headers
  - Reduced anonymous production data-health detail
  - Regression coverage for public operator-route blocking
  - Current-surface security review
affects: [04-public-showcase-and-hardening, phase-5-admin, phase-6-ai, operations]

tech-stack:
  added: []
  patterns: [next-config-security-headers, production-health-redaction, source-level-edge-regression]

key-files:
  created:
    - docs/security-review.md
  modified:
    - app/next.config.ts
    - app/app/health/data/route.ts
    - app/src/gallery/public-surface.test.ts

key-decisions:
  - "Use Next.js headers() for current public response hardening instead of new middleware or dependencies."
  - "Keep detailed live data readiness behind the existing operator token while redacting anonymous production config detail."
  - "Keep the temporary operator bridge accepted only while Caddy blocks public ingress and Phase 5 remains responsible for real admin auth."

patterns-established:
  - "Public hardening changes should include source-level regression tests for privacy and edge-routing boundaries."
  - "Security reviews should classify current findings as fixed, accepted, or deferred to the phase that introduces the relevant surface."

requirements-completed: [SHIP-01, SHIP-05]

duration: 38 min
completed: 2026-05-25
---

# Phase 04 Plan 01: Security Hardening Summary

**Public security headers, redacted production data health, operator-edge regression coverage, and current-surface security review**

## Performance

- **Duration:** 38 min
- **Started:** 2026-05-25T19:38:00Z
- **Completed:** 2026-05-25T20:16:34Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added baseline public security headers through `next.config.ts`.
- Reduced anonymous production `/health/data` output so config check names and errors are not exposed publicly.
- Added regression coverage for configured security headers, public Caddy operator-route blocking, and production data-health redaction.
- Created `docs/security-review.md` with fixed, accepted, and deferred findings across the current public/deployment surface.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add public security headers** - `be1aa2e` (test)
2. **Task 2: Harden data health and operator exposure checks** - `4b065bc` (fix)
3. **Task 3: Document current attack-surface review** - `84c1c75` (docs)

**Plan metadata:** pending in this summary commit.

## Files Created/Modified

- `app/next.config.ts` - Adds baseline public security headers.
- `app/app/health/data/route.ts` - Redacts anonymous production config-readiness details.
- `app/src/gallery/public-surface.test.ts` - Adds regression tests for headers, Caddy operator blocking, and redacted data health.
- `docs/security-review.md` - Records the current-surface security review and follow-up ownership.

## Decisions Made

- Used `nextConfig.headers()` because it hardens the current app without introducing middleware or new packages.
- Returned only `ok`, `service`, and `scope` from anonymous production `/health/data`; detailed live checks remain token-gated.
- Kept operator bridge hardening scoped to the current temporary bridge and deferred real admin auth to Phase 5.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Typecheck initially rejected test code that treated optional `nextConfig.headers` as always present and assigned directly to readonly `NODE_ENV`. The test was adjusted to narrow `headers` and use `Object.assign` for temporary environment mutation.

## User Setup Required

None - no external service configuration required.

## Verification

- `corepack pnpm --filter app lint` - passed
- `corepack pnpm --filter app typecheck` - passed
- `corepack pnpm --filter app test` - passed
- `corepack pnpm --filter app build` - passed
- `test -s docs/security-review.md && rg -n "Phase 5|Phase 6|deferred|fixed|accepted|Fixed|Accepted|Deferred" docs/security-review.md` - passed

## Next Phase Readiness

Plan 04-02 can proceed with dependency automation, workflow permission review, and the scheduled Image Cleanup fix that was added to the plan before execution.

---
*Phase: 04-public-showcase-and-hardening*
*Completed: 2026-05-25*
