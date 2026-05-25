# Roadmap: Autographs

## Overview

Autographs will ship as a lean personal collection app with the riskiest seams proven first. The roadmap starts by establishing OCI bootstrap and delivery automation, then proves Oracle and private media handling, then delivers the anonymous public gallery. Before adding the larger admin and AI surfaces, the roadmap now runs a public-showcase and hardening pass to make the current system safe, understandable, and presentable. It then completes the single-admin collection workflow with multi-image management and edit history, and finally adds advisory AI-assisted ingest.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Delivery Spine and OCI Bootstrap** - Stand up the deployable app foundation, OCI baseline, and GitHub-driven validation/deploy path. Complete; live OCI deploy proof passed from `main`.
- [x] **Phase 2: Oracle and Private Media Core** - Prove the database and object-storage seams that every collection record depends on. Implementation complete; live Oracle/Object Storage smoke path documented for operator execution with real credentials.
- [x] **Phase 3: Public Gallery MVP** - Deliver anonymous browse, filter, and detail views for published autograph items. Complete; anonymous public gallery, filters, detail pages, image viewer, quote states, and privacy gates are implemented.
- [ ] **Phase 4: Public Showcase and Hardening** - Tie up loose ends, audit the current security posture, polish documentation, and prepare the current repository state to be public as a human+AI showcase.
- [ ] **Phase 5: Admin Collection Workflow** - Complete the single-admin create, edit, publish, multi-image, and edit-history loop.
- [ ] **Phase 6: AI-Assisted Ingest** - Add advisory OCR/AI metadata suggestions without making ingest depend on them.

## Phase Details

### Phase 1: Delivery Spine and OCI Bootstrap
**Goal**: The operator can provision and configure the OCI-backed application foundation and rely on GitHub as the delivery spine for validation and deployment.
**Depends on**: Nothing (first phase)
**Requirements**: PLAT-01, PLAT-02, PLAT-03
**Success Criteria** (what must be TRUE):
  1. Operator can provision the documented OCI baseline using committed infrastructure code plus the required one-time manual bootstrap steps.
  2. Operator can configure local and GitHub-based deployment from a clear committed environment and secret contract.
  3. Pull requests run repository validation, and merges to `main` can execute the documented deployment path.
**Plans**: 4 plans
Plans:
- [x] 01-01-PLAN.md - Scaffold the proof-of-life Next.js app inside the single-repo workspace.
- [x] 01-02-PLAN.md - Add the committed Docker, nginx, and compose runtime assets for the locked OCI topology.
- [x] 01-03-PLAN.md - Define the OCI/Terraform bootstrap baseline, remote state path, and import/runbook guidance.
- [x] 01-04-PLAN.md - Define the config contract, GitHub Actions delivery spine, and OCI proof-of-life deployment runbook.

### Phase 2: Oracle and Private Media Core
**Goal**: The application can persist autograph records in Oracle and manage private multi-image media in OCI Object Storage through app-controlled access.
**Depends on**: Phase 1
**Requirements**: DATA-01, DATA-02, DATA-04, MEDIA-01, MEDIA-02, MEDIA-03
**Success Criteria** (what must be TRUE):
  1. Application can create, read, update, and list autograph metadata records in Oracle using the full v1 catalog schema.
  2. Operator can load representative sample data for local development and verification.
  3. Admin can attach multiple private images to one autograph item, including one designated primary image and supporting images.
  4. Public users can view item images through app-mediated delivery without exposing direct object URLs or storage credentials.
**Plans**: 4 plans
Plans:
- [x] 02-01-PLAN.md - Add the Oracle schema, migration runner, and typed data layer foundation.
- [x] 02-02-PLAN.md - Provision ADB and private Object Storage through Terraform and the config contract.
- [x] 02-03-PLAN.md - Implement autograph record and media APIs over Oracle and Object Storage.
- [x] 02-04-PLAN.md - Add sample-data seeding and app-mediated private image delivery verification.

### Phase 3: Public Gallery MVP
**Goal**: Anonymous visitors can browse the published autograph collection and inspect full item details against the proven metadata and media backbone.
**Depends on**: Phase 2
**Requirements**: GALL-01, GALL-02, GALL-03, GALL-04
**Success Criteria** (what must be TRUE):
  1. Anonymous user can browse a public gallery of published autograph items.
  2. Anonymous user can filter or search the gallery by signer name, category, and tags.
  3. Anonymous user can open a detail page for one autograph item and see its full stored metadata.
  4. Anonymous user can view all published images for an item, with a clear primary presentation.
**Plans**: 5 plans
Plans:
- [x] 03-01-PLAN.md - Create public-safe view models and approved quote state foundation.
- [x] 03-02-PLAN.md - Build the branded landing page, footer, and hidden admin access affordance.
- [x] 03-03-PLAN.md - Implement the public collection grid and URL-backed curated filters.
- [x] 03-04-PLAN.md - Implement item detail pages and the in-place multi-image viewer.
- [x] 03-05-PLAN.md - Document temporary production data entry and run final public gallery gates.
**UI hint**: yes

### Phase 4: Public Showcase and Hardening
**Goal**: The current project state is safe, understandable, and polished enough to make public as a showcase of the delivered personal collection system and the human+AI development process behind it.
**Depends on**: Phase 3
**Requirements**: SHIP-01, SHIP-02, SHIP-03, SHIP-04, SHIP-05
**Success Criteria** (what must be TRUE):
  1. Security, secrets, and attack-vector review has been completed across the current app, infrastructure, deployment, and repository configuration, with actionable findings fixed or explicitly tracked.
  2. Dependency update automation has been revisited and either Dependabot, Renovate, or a documented equivalent has been configured for the repo's package, container, workflow, and Terraform surfaces.
  3. The root `README.md` clearly explains the project goals, current architecture, local development, deployment model, and what makes the project a human+AI collaboration showcase.
  4. Repository badges and public-facing project metadata accurately reflect CI, linting, type checking, test/coverage posture, deployment or release status, and other useful quality signals.
  5. Loose-end issues, docs gaps, stale planning artifacts, and operational warnings for the current system have been triaged so the public repository tells a coherent story.
**Boundary Note**: This phase hardens and presents the current public-gallery/deployment surface before admin and AI are added. Phase 5 and Phase 6 must still include their own security and documentation updates for the new admin and AI surfaces they introduce.
**Plans**: TBD
**UI hint**: no

### Phase 5: Admin Collection Workflow
**Goal**: The collection owner can securely manage the catalog end to end, including creating, editing, publishing, multi-image maintenance, and reviewing edit history.
**Depends on**: Phase 4
**Requirements**: DATA-03, MEDIA-04, ADMIN-01, ADMIN-02, ADMIN-03, ADMIN-04
**Success Criteria** (what must be TRUE):
  1. Exactly one admin authentication path exists for collection management, with no public account system required.
  2. Admin can create a new autograph item by uploading images, reviewing metadata, and saving the item before publication.
  3. Admin can edit an existing autograph item's metadata and associated images, and can review an edit history showing what changed over time.
  4. Admin can publish changes so items become visible in the public gallery without leaving orphaned metadata references or image files in normal operation.
**Plans**: TBD
**UI hint**: yes

### Phase 6: AI-Assisted Ingest
**Goal**: The admin workflow gains advisory OCR/AI metadata suggestions that speed up cataloging without blocking manual control.
**Depends on**: Phase 5
**Requirements**: AI-01, AI-02, AI-03
**Success Criteria** (what must be TRUE):
  1. Upload workflow can generate AI-assisted metadata suggestions for relevant autograph fields such as signer, item type, tags, or inscription text.
  2. Admin can review, correct, or ignore OCR and AI suggestions before saving the item.
  3. Upload workflow still succeeds with fully manual metadata entry when OCR or AI assistance is unavailable or inaccurate.
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Delivery Spine and OCI Bootstrap | 4/4 | Complete | 2026-05-14 |
| 2. Oracle and Private Media Core | 4/4 | Complete | 2026-05-14 |
| 3. Public Gallery MVP | 5/5 | Complete | 2026-05-21 |
| 4. Public Showcase and Hardening | 0/TBD | Not started | - |
| 5. Admin Collection Workflow | 0/TBD | Not started | - |
| 6. AI-Assisted Ingest | 0/TBD | Not started | - |
