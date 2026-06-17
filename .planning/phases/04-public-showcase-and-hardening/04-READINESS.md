# Phase 04 Public Readiness Audit

**Date:** 2026-05-25
**Scope:** Phase 4 public showcase and hardening
**Status:** READY FOR PR CI VALIDATION

This audit records the final Phase 4 readiness gate. Per operator preference, CI is the authoritative validation surface for this PR; local validation was stopped as a required gate and only the checks already completed are listed as supporting evidence.

**Current CI note, 2026-06-17:** Phase 5 static-runtime work has since retired the Node/Next.js `app` package on `main`. The Phase 4 app commands below remain historical local evidence only. Current PR CI gates the Rust controller/static runtime, container image, workflow/infra/deploy checks, and repository secret scanning under read-only workflow permissions (`contents: read`, `pull-requests: read`).

## Requirement Traceability

| Requirement | Evidence | Status |
|-------------|----------|--------|
| SHIP-01 security/secrets/attack-vector review | `docs/security-review.md`; security headers; production health redaction; public operator-route block tests | PASS |
| SHIP-02 dependency automation | `renovate.json`; `docs/dependency-updates.md`; manual-review policy | PASS |
| SHIP-03 public README and architecture story | `README.md`; app metadata; architecture docs | PASS |
| SHIP-04 badges and quality signals | `README.md` workflow badges plus static Renovate signal | PASS |
| SHIP-05 stale docs/planning/ops reconciliation | `docs/*`; `.planning/codebase/*`; `04-04-SUMMARY.md` | PASS |

## CI Gate

| Check | Source | Status |
|-------|--------|--------|
| Workflow validation | GitHub Actions PR validation (`workflow-checks`) | PENDING CI |
| Repository secret scan | GitHub Actions PR validation (`secret-scan` / Gitleaks) | PENDING CI |
| Controller formatting, tests, production-persistence check, build, and Clippy | GitHub Actions PR validation (`controller-checks`) | PENDING CI |
| Controller image build | GitHub Actions PR validation (`image-build`) | PENDING CI |
| Dockerfile lint | GitHub Actions PR validation (`dockerfile-checks`) | PENDING CI |
| Terraform formatting, tenancy validate, and plan | GitHub Actions PR validation (`terraform-checks`) | PENDING CI |
| Ansible syntax and lint | GitHub Actions PR validation (`validate-ansible`) | PENDING CI |
| Retired Node app lint, typecheck, tests, and build | Removed Phase 4 app package | N/A - RETIRED |
| Deploy path | Merge-triggered Deploy workflow | MANUAL/POST-MERGE |
| Live Oracle/Object Storage data smoke | Manual Data Smoke workflow with real secrets | MANUAL |

## Supporting Local Evidence

These commands completed before local validation was deprioritized in favor of CI. The `corepack pnpm --filter app ...` commands are retained as historical Phase 4 evidence only; they are not current gates after the static-runtime/controller migration retired the Node app package.

| Command | Result |
|---------|--------|
| `corepack pnpm --filter app lint` | PASS, historical retired-app evidence |
| `corepack pnpm --filter app typecheck` | PASS, historical retired-app evidence |
| `corepack pnpm --filter app test` | PASS, 17 tests, historical retired-app evidence |
| `corepack pnpm --filter app build` | PASS, historical retired-app evidence |
| `terraform -chdir=infra/terraform fmt -check -recursive` | PASS |
| `ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml` | PASS |
| `ansible-playbook --syntax-check deploy/ansible/playbooks/data-smoke.yml` | PASS |
| `ansible-playbook --syntax-check deploy/ansible/playbooks/system-cleanup.yml` | PASS |
| `ansible-lint deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/data-smoke.yml deploy/ansible/playbooks/system-cleanup.yml` | PASS |

The Terraform validate path was not used as a readiness gate because provider initialization required registry network access in the sandbox. CI or an operator-run validation should cover Terraform provider-backed validation when network access is available.

## Manual And Deferred Items

| Item | Status | Owner |
|------|--------|-------|
| Real single-admin auth and admin UX | DEFERRED by design | Phase 5 |
| Edit history persistence/rendering | DEFERRED by design | Phase 5 |
| Media replacement/orphan cleanup guarantees | DEFERRED by design | Phase 5 |
| Temporary operator bridge retirement | DEFERRED by design | Phase 5 |
| OCR/AI-assisted metadata suggestions | DEFERRED by design | Phase 6 |
| AI provider/prompt/privacy review | DEFERRED by design | Phase 6 |
| Live Data Smoke against production Oracle/Object Storage | MANUAL | Operator |
| Deploy workflow proof after merge | MANUAL | Operator |

## Release Notes For PR Review

- Current public routes are hardened with baseline headers and app-mediated image delivery.
- Anonymous production data-health responses avoid detailed configuration leakage.
- Temporary operator mutation routes remain token-guarded and blocked at the public Caddy edge.
- Renovate is configured conservatively with manual review and no automerge.
- Cleanup job reliability was improved for multi-tag stale Podman images.
- Public docs now separate implemented public-gallery/deployment scope from Phase 5 admin and Phase 6 AI scope.

## Decision

Phase 4 is ready to proceed through PR review and CI. No current-surface high-risk issue is intentionally deferred to Phase 5 or Phase 6.
