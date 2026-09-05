---
phase: quick-260904-j6b-release-please-production-model
plan: 03
subsystem: release-retention
tags: [ghcr, podman, releases]
requires:
  - phase: quick-260904-j6b-release-please-production-model
    provides: Active and previous controller status schema from Plan 01
provides:
  - Rollback-aware local and remote image retention
  - Manual remote deletion and scheduled inventory
  - Release and recovery operator procedures
affects: [deployment, operations]
tech-stack:
  added: []
  patterns: [inventory before remote deletion]
key-files:
  created: [scripts/test_select_removable_images.py, docs/release-management.md]
  modified: [.github/workflows/image-cleanup.yml, scripts/cleanup-ghcr-images.py]
key-decisions:
  - Retain untagged remote manifests until dependency reachability is proven.
  - Serialize cleanup against production deployment.
requirements-completed: [ISSUE-192]
completed: 2026-09-05
---

# Plan 03: Retention and operations

Local cleanup protects active/previous image tags and digests, container-used images, and the configured newest-image window. GHCR cleanup requires valid deployed status, defaults to inventory, prints candidates before deletion, and protects active/previous mappings. Schedules cannot request remote deletion.

## Commits

- `ec1ea98`: selector, workflow, and initial regressions.
- `9d38f66`: VM digest metadata parsing and fail-closed/default-inventory tests.
- `59b9c51`: operator docs and release-please initial changelog boundary.

## Verification

- Ten cleanup/selector tests passed; the broader 37 automation tests also passed.
- Cleanup playbook syntax and role ansible-lint passed with no warnings.
- Image cleanup actionlint passed using v1.7.12.
- Repository hygiene and diff checks passed.
- Combined deployment workflow/runtime verification remains part of Plan 02 and the final PR gate.

## Deviations

- Executed independent retention code and docs alongside Plan 02 after the shared status interface was established; no overlapping ownership.
- Added protection for container-used images and retained untagged GHCR manifests because force deletion or removal of a multi-platform child could damage running/retained artifacts.
- Updated bootstrap from 0.1.3 to 0.1.5 after main advanced during pause. Current deployed controller is v0.1.4.

## User setup and remaining validation

Configure `RELEASE_PLEASE_TOKEN` before merging. Live release creation, deploy/retry/rollback, and incremental publish require post-merge operator validation. No production resources or remote artifacts were changed during implementation.
