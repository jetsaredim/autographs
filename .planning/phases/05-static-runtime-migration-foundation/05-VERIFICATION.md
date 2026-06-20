---
phase: 05-static-runtime-migration-foundation
verified: 2026-06-20T16:19:26Z
status: passed
score: 7/7 must-haves verified
---

# Phase 05: Static Runtime Migration Foundation Verification Report

**Phase Goal:** Prove a static public catalog generated inside the OCI boundary, validated through Caddy, with a minimal static admin seed shell and private publisher/API path replacing the prior public Next.js runtime foundation.
**Verified:** 2026-06-20T16:19:26Z
**Status:** passed

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Public-safe static artifact contracts are defined for gallery, detail, search/facet data, generated media paths, and publish manifests. | VERIFIED | `controller/src/contracts.rs`, `controller/src/publisher.rs`, `controller/tests/static_contract.rs`, and `docs/static-artifact-contract.md`; summarized in `05-01-SUMMARY.md`. |
| 2 | Minimal private content seed path can write metadata and one private original image into Oracle/Object Storage through the new private admin/API boundary. | VERIFIED | `controller/src/routes.rs`, `controller/src/oracle_catalog.rs`, `controller/src/oci_media.rs`, `controller/tests/live_persistence_smoke.rs`; live persistence proof and later cleanup evidence recorded in `05-03-SUMMARY.md` and `05-07-SUMMARY.md`. |
| 3 | Publisher can generate complete static public output inside the OCI/runtime boundary without GitHub-hosted workflows reading catalog content or private storage identifiers. | VERIFIED | `controller/src/publisher.rs`, `controller/tests/publisher.rs`, `.github/workflows/deploy.yml`, and `05-04-SUMMARY.md`; security audit confirms GitHub workflows build/deploy only. |
| 4 | Published image derivatives are sanitized, complete, and referenced only through public-safe generated paths. | VERIFIED | WebP derivative generation and privacy scans in `controller/src/derivatives.rs`, `controller/src/publisher.rs`, `controller/tests/publisher.rs`, and `05-SECURITY.md`. |
| 5 | Caddy serves generated static output and the static admin shell through the Rust/static route shape. | VERIFIED | Caddy and quadlet wiring in `deploy/ansible/roles/autographs_deploy/files/Caddyfile`, `autographs-controller.container.j2`, `controller/tests/caddy_static_routes.rs`, public edge checks in `05-07-SUMMARY.md`, and UAT Test 1/4 in `05-UAT.md`. |
| 6 | Thin private admin/publisher API foundation reports health, enforces the private boundary, accepts minimal seed content, and triggers/reports publish jobs. | VERIFIED | Auth, health, seed, publication, and publish routes in `controller/src/auth.rs` and `controller/src/routes.rs`; tested by `controller/tests/auth_and_health.rs`, `controller/tests/static_admin.rs`, `controller/tests/seed_content.rs`, and live static smoke. |
| 7 | Rust/static cutover and retired public Next.js/API paths are backed by recorded live proof and closure evidence. | VERIFIED | `05-07-SUMMARY.md` records live static smoke revision `23b6289`, release `2cc81313-0638-4de2-8143-1a613391519d`, retired public route checks, no remaining smoke rows, and Object Storage cleanup; `05-UAT.md` records 5/5 operator confirmations. |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `controller/src/contracts.rs` | Public-safe static DTO contract | VERIFIED | Versioned catalog, facets, detail, media, and manifest DTOs. |
| `controller/src/publisher.rs` | Static publisher and candidate validation | VERIFIED | Generates pages/data/derivatives, validates candidates, promotes `current`, records publish status. |
| `controller/src/routes.rs` | Private controller/admin/publish API | VERIFIED | Health, auth, admin seed/upload/publication, publish trigger/status routes. |
| `controller/src/oracle_catalog.rs` | Oracle source-of-truth persistence | VERIFIED | Production catalog repository with atomic image attach transaction after PR review fix. |
| `controller/src/oci_media.rs` | OCI Object Storage private media path | VERIFIED | Instance-principal media store with bounded requests and cleanup diagnostics. |
| `controller/static-admin/*` | Minimal static admin seed/publish shell | VERIFIED | Framework-free, same-origin `/admin/api/*` admin shell with source privacy tests. |
| `deploy/ansible/roles/autographs_deploy/files/Caddyfile` | Public static and private controller routing | VERIFIED | Static root, admin shell, private `/admin/api/*` proxy, retired route blocking. |
| `deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2` | Private controller runtime wiring | VERIFIED | Private Podman network and private `:ro,Z` wallet/secrets mounts. |
| `.github/workflows/ci.yml` and `.github/workflows/deploy.yml` | CI/deploy integration | VERIFIED | Controller checks/image build/deploy wiring without catalog generation on GitHub. |
| `docs/static-runtime-runbook.md` | Operator proof and recovery procedure | VERIFIED | Live persistence/static smoke, copied-wallet smoke mount, cleanup, and diagnostics documented. |
| `05-UAT.md` | Human/operator verification | VERIFIED | Complete, 5 passed, 0 gaps. |
| `05-SECURITY.md` | Threat verification | VERIFIED | `status: verified`, `threats_open: 0`, 24/24 threats closed. |

**Artifacts:** 12/12 verified

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| Static admin shell | Private controller | same-origin `/admin/api/*` fetch calls | VERIFIED | `controller/static-admin/admin.js` only calls `/admin/api/*`; tests assert privileged paths stay private. |
| Controller seed/upload routes | Oracle/Object Storage | `OracleCatalogRepository` and `OciInstancePrincipalMediaStore` | VERIFIED | Live static smoke proved create/upload/publish against ADB and OCI Object Storage. |
| Oracle/Object Storage source | Static release | publisher repository/media abstractions | VERIFIED | Publisher tests and live smoke prove generated item pages/data/media derive from private source-of-truth. |
| Candidate release | Public `current` release | validation before symlink promotion | VERIFIED | Candidate validation and atomic promotion in publisher; Caddy serves `current`. |
| Caddy public edge | Rust/static runtime | Caddyfile and quadlet private network | VERIFIED | Public `/collection/` returned 200; retired `/api/operator/catalog` and `/api/catalog` returned 404. |
| Smoke containers | Oracle wallet | copied wallet directory mounted `:ro,Z` | VERIFIED | Operator confirmed copied-wallet smoke process works without relabeling the controller wallet. |

**Wiring:** 6/6 connections verified

## Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| STATIC-01 | SATISFIED | - |
| STATIC-02 | SATISFIED | - |
| STATIC-03 | SATISFIED | - |
| STATIC-04 | SATISFIED | - |
| STATIC-05 | SATISFIED | - |
| STATIC-06 | SATISFIED | - |
| STATIC-07 | SATISFIED | - |

**Coverage:** 7/7 requirements satisfied

## Anti-Patterns Found

None blocking.

Review follow-up fixed two issues before closeout:

| File | Pattern | Severity | Resolution |
|------|---------|----------|------------|
| `controller/src/oracle_catalog.rs` | Primary-image demotion committed before replacement insert | Blocker | Fixed in PR 131 by keeping demotion and insert in one transaction. |
| `autographs-controller.container.j2` | Shared SELinux relabeling for secret mounts | Warning | Fixed in PR 131 by using private `:ro,Z` mounts and documenting copied-wallet smoke runs. |

**Anti-patterns:** 0 open

## Human Verification Required

None remaining. Human/operator checks are complete in `05-UAT.md`.

## Gaps Summary

**No gaps found.** Phase goal achieved. Ready to proceed.

### Deferred Scope

- Full polished admin collection workflow, multi-image maintenance ergonomics, edit history, and controller-owned media deletion UX remain Phase 6 scope.
- Advisory AI-assisted ingest remains Phase 7 scope.
- Nyquist validation is intentionally skipped because `workflow.nyquist_validation=false`.

## Verification Metadata

**Verification approach:** Goal-backward closeout from ROADMAP success criteria plus Phase 5 plans, summaries, UAT, security audit, live smoke evidence, PR review, and CI.
**Must-haves source:** ROADMAP Phase 5 success criteria.
**Automated checks:** Controller tests, Caddy route tests, static admin tests, publisher tests, production-persistence checks, Ansible syntax, GitHub CI, and live smoke gates recorded across Phase 5 summaries and PR 131.
**Human checks required:** 0 remaining; `05-UAT.md` complete.
**Security gate:** `05-SECURITY.md` verified, `threats_open: 0`.
**Validation gate:** Nyquist disabled by configuration; no `05-VALIDATION.md` required under current settings.
**Total verification time:** Retroactive closeout session.

---
*Verified: 2026-06-20T16:19:26Z*
*Verifier: Codex orchestrator from completed Phase 5 evidence*
