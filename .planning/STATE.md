---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 08-04-PLAN.md
last_updated: "2026-08-26T21:40:00-04:00"
last_activity: 2026-08-26 -- Made Terraform preserve operator-managed OCI Vault secret versions and verified the live tenancy plan is non-destructive.
progress:
  total_phases: 10
  completed_phases: 7
  total_plans: 47
  completed_plans: 43
  percent: 70
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-28)

**Core value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.
**Current focus:** Phase 08 — admin-media-review-and-operational-posture

## Current Position

Phase: 08 (admin-media-review-and-operational-posture) — EXECUTING
Plan: 2 of 8
Status: Ready to execute
Last activity: 2026-08-26 -- Made Terraform preserve operator-managed OCI Vault secret versions and verified the live tenancy plan is non-destructive.

Progress: [█████████░] 89% of milestone plans complete; Phase 8 Plan 4 is next

## Performance Metrics

**Velocity:**

- Total plans completed: 42 of 47
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
| Phase 08 P01 | 8min | 3 tasks | 7 files |
| Phase 08 P02 | live session plus PR checkpoint | 3 tasks | 10 files |
| Phase 08 P03 | 8min | 3 tasks | 6 files |
| Phase 08 P04 | 13min | 3 tasks | 13 files |

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
- Phase 8: Repair the current production security patching workflow, run a repo-wide posture pass, and add private admin image previews plus non-destructive image adjustment foundations before more media-heavy taxonomy or AI work.
- Phase 8 CDN posture: CDN/cache behavior is first-class Phase 8 work; define cache keys, TTLs, purge triggers, admin/API bypass, and rollback before the detailed media editor, then enable CDN only after adjusted-media cache behavior is proven.
- Phase 8 security patching: The old runtime-IP resolution issue is resolved, but latest weekly scans still fail after IP resolution in the Ansible scan step; the repair should reduce host-side data gathering and use authoritative external Oracle Linux advisory sources where practical.
- Quick task: Production Oracle persistence now starts a read-only daily heartbeat that runs lightweight SQL to keep Always Free Autonomous Database inactivity tracking fresh without writing synthetic catalog data.
- Phase 9: Add optional taxonomy/media cues for franchise, product-line, set, and non-default language values, keeping text metadata canonical and any public derivatives small, optional, privacy/copyright reviewed, and independent from OCR/AI provider work.
- Phase 10: Add advisory AI-assisted ingest after the admin workflow and richer metadata model exist, without making manual entry dependent on AI.
- Review follow-up: Phase 6, Phase 7, Phase 8, Phase 9, and Phase 10 now carry explicit security/documentation completion criteria for the new admin, metadata, media, operations, and AI surfaces they introduce.
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
- [Phase 07]: AI-assisted ingest remains pending after the manual metadata taxonomy; it is now Phase 10 after the Phase 8 admin media/posture foundation and Phase 9 taxonomy media cues.
- [Phase 08]: Phase 8 CDN/cache work is first-class: define admin/API bypass, rollback-friendly HTML/JSON, fingerprinted media, purge/rollback behavior before media work, then enable/verify production CDN after adjusted-image cache behavior is proven. — The admin media editor will likely update existing images, so cache behavior must be designed before media implementation and CDN enablement should wait until adjusted derivatives and URLs can be verified.
- [Phase 08]: Phase 8 security reporting should use OpenSCAP `oscap-ssh` with Oracle Linux OVAL as the authoritative finding source, keep hidden approval metadata to advisory IDs, and parse report details from XML on the runner.
- [Phase 08]: Phase 8 posture cleanup should run before media work in separate PRs and add enforceable CI hygiene guardrails where feasible. — Aggressive cleanup is desired, but splitting it before media work keeps the image editor from carrying unrelated repo-wide churn.
- [Phase 08]: Phase 8 media adjustment must include a dedicated review view, multiple overlays, draft-local save/cancel/reset, before/after comparison, and required auto-assisted deskew/perspective correction with manual fallback. — Uploaded images can be misaligned or askew, and visual guide overlays plus auto-assisted/manual correction are necessary for the admin to prepare publish-safe derivatives.
- [Phase 08]: Production security remediation now runs Ksplice before DNF, re-scans after Ksplice, applies only remaining approved advisories with `dnf upgrade-minimal --security --advisories`, and closes issues only after OpenSCAP reports no findings.
- [Phase 08]: Scanner approval metadata remains advisory-ID-only; CVEs, severity, summaries, affected packages, Ksplice-aware status, and advisory links stay visible report detail outside the hidden metadata block.
- [Phase 08]: Production reboot/installonly cleanup uses a separate `approved-production-reboot` issue label, validates current OpenSCAP findings against the approved issue before downtime, requires DNF to prove there is no advisory-scoped package work left, rejects non-kernel-family findings, waits for runtime health after reboot, removes old installonly kernels, re-runs OpenSCAP, and refreshes or closes the original scanner issue.
- [Phase 08-02]: OPS-02 remains open after pre-media posture cleanup because CDN enablement and adjusted-media cache verification are owned by later Phase 8 plans.
- [Phase 08-02]: Plan 08-03 is unblocked by landed pre-media PR #207, merge commit `5b0d8ebb69f5342b84ce75dfae8eb56ade6523e5`.
- [Phase 08-03]: Production CDN enablement remains deferred until adjusted-media cache behavior is proven by later Phase 8 media and publisher work.
- [Phase 08-03]: Cloudflare cache rules are named exactly `Bypass admin and API`, `Respect rollback-sensitive public documents`, and `Cache fingerprinted media and assets`.
- [Phase 08-03]: OPS-02 remains pending overall because production CDN enablement and post-media verification are owned by later Phase 8 plans.
- [Phase ?]: [Phase 08-04]: Use the 08-RESEARCH.md package-legitimacy approval for imageproc 0.27.0 and resolve Cargo.lock through the compile/test gate.
- [Phase ?]: [Phase 08-04]: Keep public static JSON/HTML contracts unchanged in Plan 08-04; Plan 08-07 owns adjustment-aware public publisher/cache behavior.
- [Phase ?]: [Phase 08-04]: Expose adjustment metadata through the private admin item response while keeping public static artifacts untouched.

### Pending Todos

- Future naming/config refinement: after the admin rename and instance-principal Object Storage path settle, review service names, env vars, Terraform variables, IAM identities, and deploy resources for over-wording or stale terminology. Include unnecessary create/enable Terraform booleans where resources are intended to be end-state managed by Terraform state.
- Future IAM refinement: review deploy-user permissions after the dedicated admin runtime identity exists, but do not assume permissions should be removed. The deploy user runs Terraform for much of the infrastructure, so broad permissions may remain justified when they are needed for provisioning even if runtime access moves to narrower identities.
- Future OCI crate contribution: propose an upstream patch adding binary-safe request signing/Object Storage APIs, including byte-body `PUT`, byte-returning `GET`, object `DELETE`, and rustls-friendly TLS configuration. The local controller adapter can then collapse back onto a maintained crate instead of carrying its own binary Object Storage request path.

### Blockers/Concerns

- Phase 6 is complete; keep its admin workflow, session auth, media cleanup, cache posture, and runtime cleanup guidance intact.
- Phase 7 is complete; keep its first-class multi-signer records, reusable signer profiles, optional Wikipedia/IMDb links, format/origin/franchise/product-line/set fields, public facet updates, and reviewed backfill path intact while planning Phase 8.
- Phase 8 should first repair production security patching, run repo-wide posture cleanup with CI hygiene guardrails before media work, define the CDN/cache contract before the detailed editor, add admin-private preview/review with required auto-assisted deskew/perspective correction and manual fallback, then enable/verify CDN only after adjusted-media cache behavior is proven.
- Phase 9 should then look at optional taxonomy/media cues for franchise, product-line, set, and non-default language values, which can use upcoming event items as practical test material without requiring OCR/AI provider work.
- Phase 10 owns advisory AI-assisted ingest after manual admin workflows, richer taxonomy, and media-review foundations exist, with OCR/AI provider, prompt, privacy, and configuration-security review still required.
- Keep production security patching action pins, approval allowlist, and Ansible role behavior reviewed with deploy/runtime changes.

### Roadmap Evolution

- Phase 5 inserted: Static Runtime Migration Foundation; former Admin Collection Workflow moved to Phase 6 and AI-Assisted Ingest later moved to Phase 8 after the Phase 7 taxonomy insertion.
- Phase 6 edited: added 06-09 optimization wave for public delivery, image size, CDN/cache posture, and deployed instance/codebase cleanup before Phase 6 closeout.
- Phase 7 inserted: Metadata Taxonomy and Public Facets; former AI-Assisted Ingest was first moved to Phase 8 so AI suggestions would target the richer manual metadata model, then later moved to Phase 10 during the admin media/posture split.
- Phase 8 realigned: Admin Media Review and Operational Posture inserted as the next phase; prior Phase 8 taxonomy/media cue work moved to Phase 9, and advisory AI-assisted ingest moved to Phase 10.

## Session Continuity

Last session: 2026-08-30T04:34:34Z
Stopped at: PR #223 clean re-review and all GitHub CI checks passed for C4 core/Kdump slice 260827-vd8
Resume file: None

## Quick Tasks Completed

| Date | Task | Summary |
|------|------|---------|
| 2026-08-28 | c4-kernel-persistence-gates-disable-cont | Verified and clean-reviewed on PR #223: disabled controller/system userspace core persistence and Kdump, added live fail-closed deploy assertions, exact structural CI contracts, and checked reboot proof commands while keeping encrypted swap and OLED reclamation separate. Implementation commits `3c24e0d`, `f960dba`, `49052e0`; review fixes `b5faa89`, `961023b`. |
| 2026-08-27 | make-oci-vault-secrets-manual-with-ignor | Replaced automatically generated placeholders with ignored manual bootstrap content and live-planned the already-rotated tenancy secrets with zero replacement, zero content change, and zero destruction. Implementation commit `719b55b`. |
| 2026-08-25 | address-pr-222-review-findings-for-vault | Replaced unsafe runtime env mutation with typed secret overrides, enforced production hash-only admin deploy authentication, added executable fail-closed regression coverage, and fixed the follow-up Workflow checks warning. Implementation commits `01194d4`, `273750e`. |
| 2026-08-24 | vault-instance-principal-smoke | Added a one-shot Vault smoke container, extracted reusable OCI instance-principal signing, documented the VM runbook path, and live-proved retrieval of the Terraform-generated proof secret from the runtime VM container. |
| 2026-08-23 | remove-legacy-controller-secret-copying | Removed the candidate-controller deploy-key copy/mount, added a runtime contract regression, and recorded the review finding on PR #217. Implementation commit `cd521aa`. |
| 2026-08-22 | replace-the-ecosystem-cleanup-prototype | Added Clippy, measured LLVM coverage, fixture-tested ast-grep structural rules, and maintained engineering standards while retaining Python only for operational inventory. |
| 2026-08-22 | make-ecosystem-inventory-boundary-aware | Verified: classified repository configuration by execution boundary, limited VM drift to like-for-like runtime and smoke comparisons, and received a clean independent counter-review. |
| 2026-08-22 | address-every-blocker-and-warning-from-p | Verified: addressed all seven PR #213 review findings with hardened analyzers, regression fixtures, accurate swap/core boundaries, an executable old-image rollback contract, regenerated evidence, and a clean reviewer-agent confirmation. |
| 2026-08-20 | address-pr-211-reviewer-documentation-fi | Needs Review: fixed all reviewer documentation warnings and documented a fail-safe pre-merge candidate-controller static-publish gate; VM execution and PR evidence remain required. Implementation commits `7078503`, `393700f`, `e36e544`. |
| 2026-08-20 | remove-the-redundant-host-cargo-build-fr | Verified: removed the redundant host-side production controller link while retaining formatting, tests, production compilation, Clippy, and the parallel release Docker image build. Implementation commit `eb96da0`. |
| 2026-08-20 | migrate-the-production-rust-controller-a | Verified: migrated production Oracle persistence and established smoke images to `oracledb 26.0.0-beta.2`, removed Instant Client, enforced the wallet password deploy contract, and retained OL10 slim as the measured runtime base. Implementation commit `56bf5a5`. |
| 2026-08-16 | run-oracle-heartbeat-once-shortly-after- | Made the production Oracle catalog heartbeat run its first lightweight SQL command immediately after controller startup, then continue on the configured interval, with docs updated for immediate deploy/reboot confirmation. |
| 2026-08-16 | add-a-production-oracle-database-heartbe | Added a production Oracle catalog heartbeat that defaults to daily `select 1 from dual`, can be disabled with `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS=0`, and is documented in deploy/config surfaces. |
| 2026-08-14 | improve-security-reboot-drift-failure-co | Reboot drift failures now persist operator-facing failure context, refresh the scanner issue from current pre-reboot OpenSCAP findings, reset the approval instruction to `approved-production-update`, and keep refusing downtime until the refreshed issue is reviewed. |
| 2026-08-14 | add-approved-production-reboot-workflow | Added a separately approved production reboot workflow that drift-checks OpenSCAP findings, requires a DNF no-op/package-family gate, reboots healthy targets, removes old installonly kernels, re-scans, and refreshes or closes the scanner issue. |
| 2026-08-03 | replace-production-security-patch-scanne | Replaced the production security scanner with OpenSCAP Oracle OVAL findings, advisory-ID approval metadata, Ksplice-first remediation, and DNF advisory-scoped fallback updates. |
| 2026-08-02 | harden-apply-security-update-issue-metad | Replaced brittle apply-side scanner metadata extraction with a quoted role task, added diagnostics and CI fixture coverage, and verified live issue #198 parses under Ansible 2.21. |
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
| 2026-07-13 | fix-admin-item-list-visual-alignment-and | Replaced admin item-list status/change words with accessible icon badges, fixed action-cell row-line alignment, and captured taxonomy thumbnail support as a follow-up recommendation. |
| 2026-07-14 | refine-admin-item-list-vertical-alignmen | Centered admin item-list row content, shortened updated timestamps to relative labels, and combined publish-status navigation into the clickable status icon. |
| 2026-07-16 | add-admin-signer-profile-management-tab | Added a Signers admin tab for reusable profile edits, linked item signer rows and item-list signer cells to profile management, and kept item signer edits focused on item-level role/context. |
| 2026-07-16 | normalize-signer-imdb-and-wikipedia-prof | Normalized reusable signer profile links to compact `w.wiki` and IMDb `nm...` IDs while rendering full public URLs from the stored identifiers. |
| 2026-07-16 | show-linked-items-on-admin-signer-profil | Added linked item counts and edit/history actions to each signer profile card in the admin Signers editor. |
| 2026-07-17 | public-detail-signer-icons | Rendered public detail signer roles inline as `Name (role)`, reduced visible Wikipedia/IMDb profile icon badges, split multi-signer fact chips, opened profile links in new tabs, and removed generated public content/media from repo and Docker build context. |
| 2026-07-19 | move-phase-8-ai-05-taxonomy-thumbnail-me | Moved Phase 8 taxonomy/media cue exploration to the first success criterion and renumbered the pending AI requirements so non-AI-specific taxonomy media work is looked at before OCR/AI ingest. |
| 2026-07-24 | issue-165-semver-versioning | Added repo semver/status automation, semver-tagged controller deploys, runtime/static release version metadata, and semver-aware image cleanup. |
| 2026-07-26 | issue-175-bulk-catalog-derivative-cache | Added Oracle bulk catalog loading for list/published publish paths, checksum-backed derivative cache source keys, upload/replacement checksum persistence, timing instrumentation, and regression coverage for checksum reuse plus legacy fallback. |
| 2026-07-29 | address-issue-193-oracle-signer-credit-deadlock | Skipped Oracle signer-credit writes for unchanged submitted credits and replaced update-time delete/reinsert churn with stable row-level sync. Commit 341e425. |
