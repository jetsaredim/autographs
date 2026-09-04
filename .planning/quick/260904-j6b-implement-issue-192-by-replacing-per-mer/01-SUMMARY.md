---
phase: quick-260904-j6b-release-please-production-model
plan: 01
subsystem: release-automation
tags: [release-please, semver, github-releases, deployment-state, tdd]

requires:
  - phase: issue-165-semver-versioning
    provides: Existing v0.1.3 tag line and production release status
provides:
  - Manifest-mode release-please configuration seeded at 0.1.3
  - Deterministic full-range release planning and manifest generation
  - Idempotent automatic, retry, and controller-only rollback status transitions
affects: [release-workflow, deploy-workflow, image-cleanup, deployment-runbook]

tech-stack:
  added: [release-please]
  patterns: [semantic-tag deployment with digest verification, deterministic release manifests]

key-files:
  created: [release-please-config.json, .release-please-manifest.json, version.txt, CHANGELOG.md, scripts/release.py, scripts/test_release.py]
  modified: [.release-status.json]

key-decisions:
  - "release-please is the sole semantic version and Git tag authority; version.txt replaces the custom VERSION file."
  - "Release planning classifies the complete previous-release-to-target range and deploys semantic controller tags while verifying recorded sha256 digests."
  - "Repository-only releases advance latestRepositoryVersion without claiming that production repository source changed."

patterns-established:
  - "Release manifest assets are deterministic and may only be created or accepted byte-for-byte; conflicting bytes fail closed."
  - "Controller-only rollback swaps controller mappings without changing repository, infrastructure, or deployed source status."

requirements-completed: [ISSUE-192]

duration: 23min
completed: 2026-09-04
---

# Quick Task Plan 01: Release-Please State Contracts Summary

**Release-please now owns semantic version inputs, while a pure Python state machine validates full release ranges, immutable image digests, deterministic manifests, retries, and controller-only rollback.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-09-04T18:41:32Z
- **Completed:** 2026-09-04T19:04:23Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Seeded a single root release-please package at 0.1.3 with ready accumulating Release PRs, draft GitHub Releases, forced tag creation, and always-update behavior.
- Removed the custom version-bump, tag-selection, GitHub API, and status-rendering helper in favor of deterministic release validation and reconciliation primitives.
- Added 14 fixture-backed tests covering complete-range classification, draft preflight, manifest conflicts, digest checks, idempotent retry, and controller-only rollback.
- Evolved production status to distinguish latest and deployed repository releases plus active and previous controller tag/digest mappings.

## Task Commits

1. **Task 1: Establish release-please as the single version authority** - `9ce0279` (chore)
2. **Task 2 RED: Define release reconciliation contracts** - `db4aa87` (test)
3. **Task 2 GREEN: Implement release reconciliation state machine** - `68016ae` (feat)
4. **Task 2 hardening: Reject ambiguous release manifests** - `edd0032` (fix)

## Files Created/Modified

- `release-please-config.json` - Root simple-package release, changelog, tag, Release PR, and draft Release policy.
- `.release-please-manifest.json` - Existing root package version baseline.
- `version.txt` - Release-please-managed plain semantic version.
- `CHANGELOG.md` - Release-please-managed release notes entry point.
- `VERSION` - Removed obsolete custom version authority.
- `.release-status.json` - Latest/deployed repository and active/previous controller mapping.
- `scripts/release.py` - Pure release range, draft, digest, manifest, and status state machine with CLI adapters.
- `scripts/release-version.py` - Removed custom bump/tag/GitHub mutation implementation.
- `scripts/test_release.py` - Permanent release contract regression suite.
- `scripts/test_release_version.py` - Removed obsolete merged-PR bump tests.

## Decisions Made

- Kept the release-please manifest limited to package-path/version entries because its schema treats every property as a package path; the config file retains the supported editor schema URL.
- A runtime-config or repository-only manifest must reuse the active controller tag and digest; it cannot disguise a controller mapping change.
- The first release after bootstrap may fill the currently unknown production controller digests, after which reuse requires exact digest equality.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unsupported metadata from the release-please versions manifest**
- **Found during:** Task 2 verification
- **Issue:** The initial manifest included `$schema`, but release-please treats every manifest property as a package path and could interpret it as a second package.
- **Fix:** Kept `.release-please-manifest.json` to the single `.` package and added a permanent exact-shape regression.
- **Files modified:** `.release-please-manifest.json`, `scripts/test_release.py`
- **Verification:** `python3 -m unittest scripts/test_release.py`
- **Committed in:** `edd0032`

**2. [Rule 2 - Missing Critical] Enforced manifest impact/controller consistency**
- **Found during:** Task 2 threat-boundary review
- **Issue:** A syntactically valid remote manifest could label itself repository-only while selecting a different controller mapping.
- **Fix:** Enforced impact/change/reuse invariants and exact active tag/digest reuse before updating status.
- **Files modified:** `scripts/release.py`, `scripts/test_release.py`
- **Verification:** Permanent disguised-controller-change regression plus the full 14-test suite.
- **Committed in:** `edd0032`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical validation)
**Impact on plan:** Both fixes preserve release-please compatibility and close tampering paths already identified by the plan threat model; no scope expansion.

## Issues Encountered

- The repository Git index is outside the writable sandbox, so task commits required the existing approved Git escalation path.

## User Setup Required

- Before Plan 02 can run the release workflow, create a repository-scoped fine-grained PAT with Contents read/write and Pull requests read/write and store it as the Actions secret `RELEASE_PLEASE_TOKEN`.

## Next Phase Readiness

- Plan 02 can consume `scripts/release.py` to replace the current merge-triggered bump/tag/deploy jobs with a release-please-gated workflow.
- The existing deploy workflow still references the removed helper until Plan 02 rewires it; Plans 01 and 02 must ship together.

## Self-Check: PASSED

- All six created files and four modified/deleted paths match the committed diff.
- Task commits `9ce0279`, `db4aa87`, `68016ae`, and `edd0032` exist on the feature branch.
- `python3 -m unittest scripts/test_release.py` passes all 14 tests, JSON parses, obsolete files are absent, and `git diff --check origin/main...HEAD` is clean.

---
*Phase: quick-260904-j6b-release-please-production-model*
*Completed: 2026-09-04*
