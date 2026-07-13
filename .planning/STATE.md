---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: ready-for-next-phase
stopped_at: Phase 07 complete; Phase 08 not started
last_updated: "2026-07-11T01:35:40.725Z"
last_activity: 2026-07-11
progress:
  total_phases: 8
  completed_phases: 7
  total_plans: 39
  completed_plans: 39
  percent: 88
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-26)

**Core value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.
**Current focus:** Phase 08 — AI-Assisted Ingest

## Current Position

Phase: 8
Plan: Not started
Status: Phase 7 complete; Phase 8 not started
Last activity: 2026-07-11

Progress: [████████░░] 88% of milestone phases complete; Phase 8 is next

## Performance Metrics

**Velocity:**

- Total plans completed: 39 of 39
- Average duration: 29 min
- Total execution time: 1.9 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 4 | 118 min | 29 min |
| 02 | 4 | - | - |
| 03 | 5 | - | - |
| 04 | 5/5 | 54 min | 11 min |
| 05 | 7/7 | - | - |
| 06 | 9/9 | - | - |
| 07 | 5/5 | - | - |

**Recent Trend:**

- Last 5 plans: 07-01, 07-02, 07-03, 07-04, 07-05
- Trend: Positive; Phase 7 is complete with reusable signer profiles, multi-signer item credits, first-class taxonomy fields, schema version 2 public facets, admin taxonomy editing, rollout docs, and security verification

| Phase 04 P01 | 38 min | 3 tasks | 4 files |
| Phase 04 P02 | 4 min | 3 tasks | 5 files |
| Phase 04 P03 | 3 min | 3 tasks | 3 files |
| Phase 04 P04 | 5 min | 3 tasks | 12 files |
| Phase 04 P05 | 4 min | 3 tasks | 3 files |
| Phase 05 P01 | 7 min | 3 tasks | 10 files |
| Phase 05 P02 | 12 min | 3 tasks | 10 files |
| Phase 05 P07 | live session | 2 tasks | 14 files |
| Phase 06 P01 | external PR | 2 tasks | 5 files |
| Phase 06 P02 | PR #141 | 2 tasks | 6 files |
| Phase 06 P03 | 45 min | 2 tasks | 8 files |
| Phase 06 P04 | 10 min | 3 tasks | 10 files |
| Phase 06 P05 | 10min | 3 tasks | 4 files |
| Phase 06 P06 | live session | 2 tasks | 10 files |
| Phase 06 P07 | live session | 3 tasks | 7 files |
| Phase 06 P08 | live session | 2 tasks | 6 files |
| Phase 06 P09 | live session | 3 tasks | 11 files |
| Phase 07 P01 | 14min | 3 tasks | 9 files |
| Phase 07 P02 | 15min | 3 tasks | 11 files |
| Phase 07 P03 | 35min | 3 tasks | 8 files |
| Phase 07 P04 | 14min | 3 tasks | 9 files |
| Phase 07 P05 | 11min | 3 tasks | 14 files |

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
- Phase 7: Add a metadata taxonomy and public facet upgrade before advisory AI-assisted ingest so signer credits, reusable signer profiles, franchise, product line, format, and origin exist as stable suggestion targets.
- Phase 8: Add advisory AI-assisted ingest after the admin workflow and richer metadata model exist, without making manual entry dependent on AI.
- Review follow-up: Phase 6, Phase 7, and Phase 8 now carry explicit security/documentation completion criteria for the new admin, metadata, and AI surfaces they introduce.
- Pivot outcome: The former live Next.js public runtime and data-smoke path were replaced by the static public catalog, static admin shell, and thin private admin/publisher API that generate content inside the OCI boundary.
- Static-runtime boundary: GitHub Actions should build and deploy code artifacts only; catalog content generation should not expose private OCI object identifiers, URLs, Oracle data, or image UUIDs through GitHub-hosted workflows.
- Phase 5 proof outcome: The static publishing contract, Rust private controller, minimal static admin seed/publish path, and local/private Caddy candidate validation are planned, implemented, and closed as the foundation for Phase 6 admin polish.
- Phase 5 publisher: Generate static candidates and sanitized WebP derivatives locally, validate the full public inventory and privacy boundary, then atomically promote the `current` symlink.
- Phase 5 static admin: Keep the minimal browser shell framework-free and browser-storage-free, backed by the HTTP-only cookie and same-origin `/admin/api/*` calls.
- Phase 5 deployment: The public hostname now serves the Rust/static runtime through Caddy; keep the localhost generated-release preview and private `/admin/api/*` controller route documented for live proof and diagnostics.
- Phase 5 controller persistence: Use native OCI instance-principal request signing for Object Storage access from the runtime instance. A dev-node binary smoke on 2026-06-14 proved non-UTF-8 media bytes can be PUT, read back, and deleted from `autographs-media-prod` with instance principals; do not revive the OCI S3 Customer Secret path for controller media.
- Phase 5 closure: Live static smoke on 2026-06-20 proved private controller seeding, Oracle persistence, OCI Object Storage upload, generated static output, Caddy serving, unpublish republish, and cleanup against image revision `23b6289`.
- Production security patching: PR 129 added weekly/manual security update scans, scanner issue create/update behavior, allowlisted label approval, drift-checked apply, result/failure comments, and operator runbook coverage.
- [Phase 06]: The browser client renders catalog, history, image, publish, and diagnostics values through DOM node creation and textContent.
- [Phase 06]: Image tiles show safe metadata and actions only; private originals are managed through same-origin admin API calls, not direct object URLs.
- [Phase 06]: The Phase 6 admin workflow remains plain static HTML/CSS/JavaScript with no frontend build system or browser storage.
- [Phase 07]: [Phase 07-01]: Phase 7 schema changes are additive and retain legacy signer, category, and autograph_item_tags through migration.
- [Phase 07]: [Phase 07-01]: Memory repository signer credits reuse normalized signer profiles and derive a legacy credit from signer when older inputs omit signerCredits.
- [Phase 07]: [Phase 07-01]: Oracle persistence keeps a legacy taxonomy fallback in this plan; later Phase 7 adapter work should wire full signer/taxonomy table reads and writes.
- [Phase 07]: Oracle now writes legacy signer as compact signer text and legacy category as format while Phase 7 keeps rollback/reference fields.
- [Phase 07]: Signer profile edits and merges record item-level metadataUpdated events for every linked item.
- [Phase 07]: Likely duplicate physical items stay report-only in backfill output and are not auto-merged by generated PL/SQL.
- [Phase 07]: Admin signer/taxonomy management routes use the same session-cookie-only boundary as collection management and reject bearer operator tokens.
- [Phase 07]: Admin item summaries now prioritize signerText, signerNames, format, franchises, productLine, language, publication status, image count, pending-change state, and update time over legacy category.
- [Phase 07]: The static admin editor keeps legacy signer/category compatibility in save payloads while making signerCredits and first-class taxonomy fields the primary editing path.
- [Phase 07]: Public static artifacts now use schema version 2 and no longer expose Category as a public facet identifier.
- [Phase 07]: Collection cards expose compact signer text and full signer names, while detail pages alone render optional Wikipedia/IMDb profile icon links.
- [Phase 07]: Public browse filters use single-select semantic query params with AND behavior across signer, franchise, productLine, format, language, origin, role, and tag.
- [Phase 07]: Phase 7 rollout requires reviewed migration report/PLSQL, optional SQL Developer application, deploy, full static publish, and admin/public verification.
- [Phase 07]: Category remains a temporary legacy database/reference field but is not a schema version 2 public facet.
- [Phase 07]: Phase 8, not Phase 7, is the pending advisory AI-assisted ingest phase.

### Pending Todos

- Future naming/config refinement: after the admin rename and instance-principal Object Storage path settle, review service names, env vars, Terraform variables, IAM identities, and deploy resources for over-wording or stale terminology. Include unnecessary create/enable Terraform booleans where resources are intended to be end-state managed by Terraform state.
- Future IAM refinement: review deploy-user permissions after the dedicated admin runtime identity exists, but do not assume permissions should be removed. The deploy user runs Terraform for much of the infrastructure, so broad permissions may remain justified when they are needed for provisioning even if runtime access moves to narrower identities.
- Future OCI crate contribution: propose an upstream patch adding binary-safe request signing/Object Storage APIs, including byte-body `PUT`, byte-returning `GET`, object `DELETE`, and rustls-friendly TLS configuration. The local controller adapter can then collapse back onto a maintained crate instead of carrying its own binary Object Storage request path.

### Blockers/Concerns

- Phase 6 is complete; keep its admin workflow, session auth, media cleanup, cache posture, and runtime cleanup guidance intact while planning Phase 7.
- Phase 7 should replace overloaded signer/category/tag assumptions with first-class multi-signer records, reusable signer profiles, optional Wikipedia/IMDb links, format/origin/franchise/product-line fields, public facet updates, and a reviewed backfill path from current live tags.
- Phase 8 remains advisory AI-assisted ingest after manual admin workflows and richer taxonomy exist, with OCR/AI provider, prompt, privacy, and configuration-security review still required.
- Keep production security patching action pins, approval allowlist, and Ansible role behavior reviewed with deploy/runtime changes.

### Roadmap Evolution

- Phase 5 inserted: Static Runtime Migration Foundation; former Admin Collection Workflow moved to Phase 6 and AI-Assisted Ingest moved to Phase 7.
- Phase 6 edited: added 06-09 optimization wave for public delivery, image size, CDN/cache posture, and deployed instance/codebase cleanup before Phase 6 closeout.
- Phase 7 inserted: Metadata Taxonomy and Public Facets; former AI-Assisted Ingest moved to Phase 8 so AI suggestions target the richer manual metadata model.

## Session Continuity

Last session: 2026-07-10T02:03:16.577Z
Stopped at: Completed 07-05-PLAN.md
Resume file: None

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
| 2026-06-19 | reconcile-current-state-docs | Reconciled GSD and operator docs with the implemented Rust/static runtime foundation and production security patching workflow; follow-up review identified the remaining Phase 5 05-07 live static publish proof and closure summary checkpoint. |
| 2026-06-20 | close-phase-5-static-runtime | Recorded the live static publish proof, public edge checks, cleanup verification, and Phase 5 closure summary. |
| 2026-07-02 | increase-media-cache-ttl | Split generated `/media/*` cache headers from other assets and set media to `public, max-age=86400` for CDN/browser image caching while preserving admin no-store and short-lived HTML/JSON. |
| 2026-07-01 | fix-catalog-filename-privacy-scan | Addressed PR #155 publisher privacy-scan review warnings by narrowing filename scan surfaces and adding filename/static-data regressions. |
| 2026-07-02 | admin-ui-density-and-public-detail-clean | Simplified the admin hub, made dashboard/filter surfaces collapsible, added icon row actions and item sorting/filtering, and restored richer public item detail metadata. |
| 2026-07-13 | improve-controller-publish-progress-logg | Added privacy-safe publish progress stages, safe public counts, redacted failure logging, and publisher/status regressions for issue 168. |
| 2026-07-13 | add-privacy-safe-controller-operation-lo | Added broader privacy-safe controller operation logs and regression coverage preventing private object-key tracing. |
| 2026-07-13 | address-issue-167-align-resolve-runtime- | Aligned the shared runtime-IP Terraform default with the production root and added CI validation to catch future version drift. |
