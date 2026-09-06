---
phase: quick-260904-j6b-release-please-production-model
plan: 02
subsystem: release-deployment
tags: [release-please, github-actions, ghcr, ansible, rollback]
requires:
  - phase: quick-260904-j6b-release-please-production-model
    provides: Release state and manifest contracts from Plan 01
provides:
  - Release-please-gated production deployment
  - Immutable semantic controller image selection
  - Retryable draft Release reconciliation
  - Current-main controller-only rollback
affects: [deployment, release-management, production-operations]
tech-stack:
  added: [release-please-action]
  patterns: [publish-last releases, digest-before-mutation, current-automation rollback]
key-files:
  created: [scripts/test_release_workflow.py, scripts/test-fixtures/release-workflow/cases.json, deploy/ansible/playbooks/controller-rollback.yml]
  modified: [.github/workflows/deploy.yml, .github/workflows/ci.yml, .github/docker-bake.hcl, deploy/ansible/roles/autographs_deploy/tasks/main.yml, deploy/ansible/roles/autographs_deploy/templates/app.env.j2]
key-decisions:
  - A normal main push can reach production only when that release-please invocation creates a Release.
  - Semantic controller tags are deployment identities, with immutable digests recorded and rechecked immediately before mutation.
  - Rollback uses current-main automation and changes controller state only.
requirements-completed: [ISSUE-192]
completed: 2026-09-05
---

# Plan 02: Release-gated deployment and rollback

The production workflow now accumulates ordinary merges in a release-please Release PR. Merging that PR creates a draft semantic Release, resolves or builds the semantic controller image, records a deterministic manifest, deploys and verifies production, commits release status, and publishes the Release last. Manual retry reconciles an unresolved draft; manual rollback consumes a published manifest while running current-main automation and skipping Terraform.

## Commits

- `0c7fb1e`: fixture-backed release workflow contracts.
- `a4f0987`: release-gated production workflow and semantic-only image publication.
- `47061ed`: verified controller-only rollback and active/previous runtime metadata.
- Final pre-PR hardening makes both controller image probes distinguish a missing manifest from registry authentication or availability failures.

## Verification

- Release and workflow contract tests pass, including automatic gating, retry/manifest reconciliation, rollback source selection, step ordering, and fail-closed image probing.
- All repository GitHub Actions workflows pass actionlint v1.7.12.
- Full-deploy, controller-rollback, and system-cleanup playbooks pass syntax checks; `ansible-lint deploy/ansible/` reports no warnings.
- Terraform format/validation passes for both runtime and tenancy roots.
- Cargo format, tests, production-persistence check, and all-target clippy with warnings denied pass.
- Repository hygiene and whitespace checks pass.

## Deviations

- Main advanced through v0.1.8 during implementation, so release-please and production status were rebased to that exact pre-PR baseline.
- Added fail-closed registry error classification during final audit so network/authentication failures cannot be interpreted as permission to build or reuse an image.
- The permanent workflow tests remain in the repository because they protect the privileged release graph; no disposable migration or test scripts were retained.

## User setup and live validation

Before merge, install the release GitHub App only on this repository with Contents and Pull requests read/write, configure `RELEASE_PLEASE_APP_CLIENT_ID` and `RELEASE_PLEASE_APP_PRIVATE_KEY`, and let the workflow mint its short-lived token. The first Release PR after this implementation includes a controller build because the Docker build definition changed. After its Release deploys, verify controller health and complete an incremental publish. Retry and rollback remain deliberate operator actions documented in `docs/release-management.md`.
