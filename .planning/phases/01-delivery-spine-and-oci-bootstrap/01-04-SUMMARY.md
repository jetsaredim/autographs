---
phase: 01-delivery-spine-and-oci-bootstrap
plan: 04
subsystem: delivery
tags: [github-actions, ci, deploy, ghcr, oci, runbook]
requires:
  - phase: 01-02
    provides: Docker Compose runtime topology
  - phase: 01-03
    provides: OCI Terraform bootstrap baseline
provides:
  - explicit local and GitHub configuration contract
  - pull request CI validation workflow
  - main-branch GHCR publish and OCI VM deploy workflow
  - operator deployment runbook
affects: [phase-01, ci-cd, deployment, operations]
tech-stack:
  added: [github-actions, ghcr, terraform-ci]
  patterns: [repo-level secrets-and-variables contract, registry-backed VM deploy, nginx proof-of-life verification]
key-files:
  created:
    - .env.example
    - .github/.env.github.example
    - .github/actions/setup-runtime/action.yml
    - .github/workflows/ci.yml
    - .github/workflows/deploy.yml
    - docs/configuration-contract.md
    - docs/deployment-runbook.md
    - scripts/validate-ci.sh
    - scripts/deploy-vm.sh
  modified: []
key-decisions:
  - "Kept GitHub Environments optional so repo-level Secrets and Variables are sufficient for the baseline."
  - "Published a prebuilt app image to GHCR and made the VM pull that exact artifact instead of building on host."
  - "Kept OCI API signing keys isolated as a replaceable auth adapter for a later OIDC migration."
patterns-established:
  - "CI pattern: shared validation script runs app lint/typecheck/build and Terraform fmt/init/validate."
  - "Deploy pattern: GitHub Actions builds and pushes the image, applies Terraform, copies runtime files, restarts compose, and probes /health through nginx."
requirements-completed: [PLAT-02, PLAT-03]
duration: 42min
completed: 2026-04-30
---

# Phase 01 Plan 04 Summary

**Added the Phase 1 delivery spine: configuration contract, GitHub Actions validation/deploy workflows, VM deploy script, and operator runbook**

## Accomplishments

- Added `.env.example` and `.github/.env.github.example` to document local values, repo-level GitHub Secrets, repo-level GitHub Variables, and optional GitHub Environments.
- Added `docs/configuration-contract.md` as the source of truth for sensitive versus non-sensitive deployment inputs.
- Added `scripts/validate-ci.sh` and wired it into `.github/workflows/ci.yml` for pull request validation.
- Added `.github/workflows/deploy.yml` to validate, publish the app image to `ghcr.io`, run Terraform, deploy to the OCI VM over SSH, and verify `/health` through nginx.
- Added `scripts/deploy-vm.sh` to copy runtime assets, log in to GHCR, pull the published image, restart Docker Compose, and smoke-check the live route.
- Added `docs/deployment-runbook.md` with local validation, OCI bootstrap, GitHub setup, workflow behavior, and proof-of-life verification.

## Verification

- `bash scripts/validate-ci.sh` -> passed with the repo-local Terraform 1.11.4 binary; ran pnpm frozen install, app lint, app typecheck, app build, Terraform fmt check, Terraform init, and Terraform validate.

## User Setup Required

Before the deploy workflow can run successfully, populate the repo-level GitHub Secrets and GitHub Variables documented in `docs/configuration-contract.md` and `.github/.env.github.example`. OCI tenancy resources, runtime image OCID, availability domain, SSH keys, and VM Docker readiness are still operator-owned setup inputs.

## Next Phase Readiness

- Phase 1 now has the committed delivery spine required before Phase 2 starts proving Oracle and private media behavior.
- Remaining external gate: populate GitHub Actions configuration and run the real OCI deploy proof once tenancy/VM inputs are available.
