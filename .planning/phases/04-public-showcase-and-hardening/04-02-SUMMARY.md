---
phase: 04-public-showcase-and-hardening
plan: 02
subsystem: ci
tags: [renovate, github-actions, terraform, docker, ansible, cleanup]

requires:
  - phase: 04-public-showcase-and-hardening
    provides: Phase 4 security hardening and current-surface security review
provides:
  - Renovate dependency automation configuration
  - Dependency update review policy
  - Workflow permission review
  - Runtime image cleanup fix for multi-tag Podman image IDs
affects: [04-public-showcase-and-hardening, operations, phase-5-admin, phase-6-ai]

tech-stack:
  added: []
  patterns: [renovate-conservative-updates, workflow-permission-review, forced-stale-image-removal]

key-files:
  created:
    - renovate.json
    - docs/dependency-updates.md
  modified:
    - .github/workflows/deploy.yml
    - deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml

key-decisions:
  - "Use Renovate with conservative schedules, grouped non-major updates, and dashboard approval for majors."
  - "Remove unused deploy workflow actions: write permission while preserving package publishing permission."
  - "Use podman rmi --force only for selected stale image IDs after the cleanup selector preserves active, latest, protected, and retained images."

patterns-established:
  - "Dependency automation changes should document covered surfaces, validation commands, and manual smoke triggers."
  - "Scheduled operational workflows should be represented in Phase 4 readiness and supply-chain review."

requirements-completed: [SHIP-01, SHIP-02, SHIP-05]

duration: 4 min
completed: 2026-05-25
---

# Phase 04 Plan 02: Dependency And Workflow Hygiene Summary

**Renovate automation, workflow permission review, dependency policy, and resilient runtime image cleanup**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-25T20:16:34Z
- **Completed:** 2026-05-25T20:20:04Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added `renovate.json` with weekly conservative updates, grouped routine non-major updates, separate major review, lockfile maintenance, and custom managers for pinned Terraform, pnpm, and Caddy surfaces.
- Documented workflow permissions, action/image update surfaces, Renovate review expectations, manual smoke triggers, and Phase 5/6 lifecycle updates.
- Removed unused `actions: write` from the deploy workflow.
- Fixed the observed scheduled Image Cleanup failure by forcing removal only for selected stale Podman image IDs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Configure Renovate** - `f403946` (chore)
2. **Task 2: Review workflow permissions and action surfaces** - `075de75` (fix)
3. **Task 3: Document dependency update policy** - `281d406` (docs)

**Plan metadata:** pending in this summary commit.

## Files Created/Modified

- `renovate.json` - Conservative dependency automation configuration.
- `docs/dependency-updates.md` - Dependency update review and workflow hygiene policy.
- `.github/workflows/deploy.yml` - Removes unused `actions: write` permission.
- `deploy/ansible/roles/autographs_system_cleanup/tasks/main.yml` - Uses `podman rmi --force` for selected stale image IDs.

## Decisions Made

- Kept Renovate automerge disabled so dependency PRs require human review.
- Used custom Renovate regex managers for repo-specific pinned values that standard managers may not detect.
- Preserved the cleanup selector's safety boundaries and fixed only the multi-tag deletion failure mode.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `actionlint` is unavailable locally; existing CI workflow-checks covers workflow linting.

## User Setup Required

None - no external service configuration required.

## Verification

- `node -e "JSON.parse(require('fs').readFileSync('renovate.json','utf8'))"` - passed
- `corepack pnpm --filter app lint` - passed
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/system-cleanup.yml` - passed
- `test -s docs/dependency-updates.md && rg -n "Renovate|GitHub Actions|Terraform|Docker|manual smoke|Phase 5|Phase 6" docs/dependency-updates.md` - passed
- `actionlint` - unavailable locally; covered by CI workflow-checks.

## Next Phase Readiness

Wave 1 is complete. Plan 04-03 can now refresh the README, metadata, and quality signal badges using the security and dependency docs created by Plans 04-01 and 04-02.

---
*Phase: 04-public-showcase-and-hardening*
*Completed: 2026-05-25*
