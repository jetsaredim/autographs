---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 05-06-PLAN.md
last_updated: "2026-06-02T12:51:48Z"
last_activity: 2026-06-02
progress:
  total_phases: 7
  completed_phases: 4
  total_plans: 25
  completed_plans: 24
  percent: 96
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-26)

**Core value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.
**Current focus:** Phase 05 — static-runtime-migration-foundation

## Current Position

Phase: 05 (static-runtime-migration-foundation) — EXECUTING
Plan: 7 of 7
Status: Ready to execute
Last activity: 2026-06-02

Progress: [██████████] 96%

## Performance Metrics

**Velocity:**

- Total plans completed: 24 of 25
- Average duration: 29 min
- Total execution time: 1.9 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 4 | 118 min | 29 min |
| 02 | 4 | - | - |
| 03 | 5 | - | - |
| 04 | 5/5 | 54 min | 11 min |
| 05 | 6/7 | - | - |

**Recent Trend:**

- Last 5 plans: 04-01, 04-02, 04-03, 04-04, 04-05
- Trend: Positive

| Phase 04 P01 | 38 min | 3 tasks | 4 files |
| Phase 04 P02 | 4 min | 3 tasks | 5 files |
| Phase 04 P03 | 3 min | 3 tasks | 3 files |
| Phase 04 P04 | 5 min | 3 tasks | 12 files |
| Phase 04 P05 | 4 min | 3 tasks | 3 files |
| Phase 05 P01 | 7 min | 3 tasks | 10 files |
| Phase 05 P02 | 12 min | 3 tasks | 10 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Phase 1: Start with OCI bootstrap and delivery automation before feature work spreads.
- Phase 1: Live GitHub-to-OCI deploy proof passed on 2026-05-14.
- Phase 2: Prove Oracle and private media seams before building gallery or admin UX on top of them.
- Phase 2: Keep public image delivery app-mediated through `/api/catalog/{itemId}/images/{imageId}` rather than direct Object Storage URLs.
- Phase 2: Use token-guarded operator endpoints only as a temporary verification seam until the Phase 5 Rust controller/static admin seed path replaces them.
- Quick task: Manage both production containers with Podman quadlets on a dedicated Podman network instead of compose/podman-compose.
- Quick task: Keep runtime VM host configuration in the merge-triggered Ansible deploy rather than cloud-init user data.
- Quick task: Reconciled `.planning/codebase/*` docs so they describe the implemented Phase 1-3 app instead of the original planning-only repo.
- Quick task: Updated generated `AGENTS.md` codebase sections and public Caddy operator-route blocking after PR review.
- Phase 4: Run public-readiness and hardening before adding admin and AI surfaces, focused on the current public-gallery/deployment system.
- Phase 5: Prove the static public runtime plus a minimal private seed/publish path into Oracle/Object Storage before expanding admin CRUD.
- Phase 6: Treat multi-image support and edit history as v1 core collection capabilities, not later polish.
- Phase 7: Add advisory AI-assisted ingest after the admin workflow exists, without making manual entry dependent on AI.
- Review follow-up: Phase 6 and Phase 7 now carry explicit security/documentation completion criteria for the new admin and AI surfaces they introduce.
- Pivot research: The current live Next.js public runtime and data-smoke path may be replaced by a static public catalog, static admin shell, and thin private admin/publisher API that generates content inside the OCI boundary.
- Static-runtime boundary: GitHub Actions should build and deploy code artifacts only; catalog content generation should not expose private OCI object identifiers, URLs, Oracle data, or image UUIDs through GitHub-hosted workflows.
- Planning guidance: Formally plan Phase 5 to prove the static publishing contract, Rust private controller, minimal static admin seed/publish path, and local/private Caddy candidate validation before Phase 6 admin polish.
- Phase 5 publisher: Generate static candidates and sanitized WebP derivatives locally, validate the complete public inventory and privacy boundary, then atomically promote the `current` symlink.
- Phase 5 static admin: Keep the minimal browser shell framework-free and browser-storage-free, backed by the HTTP-only cookie and same-origin `/admin/api/*` calls.
- Phase 5 deployment: Keep the public hostname on Next.js until the explicit live cutover checkpoint; stage localhost-only generated-release preview and private `/admin/api/*` controller routing first.
- Phase 5 controller persistence: Use Rust `oci_api` instance-principal auth for native OCI Object Storage access from the runtime instance. A dev-node binary smoke on 2026-06-14 proved non-UTF-8 media bytes can be PUT, read back, and deleted from `autographs-media-prod` with instance principals; do not revive the OCI S3 Customer Secret path for controller media.

### Pending Todos

- Future naming/config refinement: after the admin rename and instance-principal Object Storage path settle, review service names, env vars, Terraform variables, IAM identities, and deploy resources for over-wording or stale terminology. Include unnecessary create/enable Terraform booleans where resources are intended to be end-state managed by Terraform state.
- Future IAM refinement: review deploy-user permissions after the dedicated admin runtime identity exists, but do not assume permissions should be removed. The deploy user runs Terraform for much of the infrastructure, so broad permissions may remain justified when they are needed for provisioning even if runtime access moves to narrower identities.
- Future `oci_api` contribution: propose an upstream patch adding binary-safe request signing/Object Storage APIs, including byte-body `PUT`, byte-returning `GET`, object `DELETE`, and rustls-friendly TLS configuration. The local controller adapter can then collapse back onto the crate instead of carrying its own binary Object Storage request path.

### Blockers/Concerns

- The open smoke-test fix PR targets the current live-app data smoke path; it can be abandoned if the next phase pivots away from that production smoke model.
- Live VM static publish proof, cutover criteria, and legacy-runtime retirement guidance remain the final Phase 5 step.
- Phase 5 is formally planned with seven execution plans covering static artifact contracts, Rust controller/auth, private seed/upload, publisher/derivatives, minimal static admin shell, Caddy/deploy wiring, and live cutover verification/docs.

### Roadmap Evolution

- Phase 5 inserted: Static Runtime Migration Foundation; former Admin Collection Workflow moved to Phase 6 and AI-Assisted Ingest moved to Phase 7.

## Session Continuity

Last session: 2026-06-01T16:36:05.609Z
Stopped at: Completed 05-06-PLAN.md
Resume file: .planning/phases/05-static-runtime-migration-foundation/05-07-PLAN.md

## Quick Tasks Completed

| Date | Task | Summary |
|------|------|---------|
| 2026-05-20 | podman-quadlet-deploy | Replaced compose/cloud-init runtime setup with Ansible-managed Podman quadlets and added manual runtime VM taint support. |
| 2026-05-21 | phase-6-scope | Originally added Public Showcase and Hardening after AI-assisted ingest; later reordered to Phase 4. |
| 2026-05-25 | reconcile-docs-and-workflow-guardrails | Refreshed stale codebase maps after out-of-GSD implementation progress and prepared workflow guardrail updates. |
| 2026-05-25 | add-protected-branch-commit-guardrails | Added project and global GSD guardrails to prevent direct commits to `main` or `master`. |
| 2026-05-25 | address-pr-review-findings | Refreshed stale `AGENTS.md` generated sections and made public Caddy routing block temporary operator API paths. |
| 2026-05-25 | reorder-showcase-before-admin | Moved Public Showcase and Hardening ahead of Admin Collection Workflow so hardening/docs polish run next. |
| 2026-05-25 | reconcile-phase-reorder-review | Addressed post-merge review warnings from PR #65 by aligning phase criteria and generated docs. |
| 2026-05-26 | reconcile-planning-state-after-phase-4-c | Marked Phase 4 requirements complete and captured static-runtime pivot research as next-phase planning context. |
| 2026-06-11 | remove-obsolete-runtime-vault-s3-credent | Removed runtime Terraform Vault/KMS/secret resources that are no longer needed for the instance-principal Object Storage direction. |
| 2026-06-12 | tighten-tenancy-iam-for-instance-princip | Replaced the admin-runtime/Vault IAM path with runtime dynamic-group media object access and state-bucket-scoped deploy object access. |
| 2026-06-13 | remove-obsolete-tenancy-split-doc | Removed the historical Terraform tenancy split migration runbook from active operator docs. |
