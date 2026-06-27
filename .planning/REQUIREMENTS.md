# Requirements: Autographs

**Defined:** 2026-04-18
**Core Value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

## v1 Requirements

### Platform Bootstrap

- [x] **PLAT-01**: Operator can provision the documented OCI baseline for the app using committed infrastructure code plus clearly documented one-time manual bootstrap steps.
- [x] **PLAT-02**: GitHub Actions validates the repository on pull requests and can deploy approved infrastructure and application changes on merge to `main`.
- [x] **PLAT-03**: Operator can configure the application using an explicit committed environment and secret contract for local and GitHub-based deployment.

### Catalog Data

- [x] **DATA-01**: Admin can store an autograph record in Oracle Autonomous Database Free with title, signer, description, category, tags, object references, event/source, event/location, inscription text, certification company and ID, estimated year, and publication status fields.
- [x] **DATA-02**: Application can create, read, update, and list autograph metadata records through the chosen Oracle-backed data layer.
- [ ] **DATA-03**: Application records edit history for autograph items so the admin can see what changed over time in v1.
- [x] **DATA-04**: Application can seed or otherwise load representative sample data for local development and verification.

### Media Storage

- [x] **MEDIA-01**: Admin can upload multiple images for a single autograph item into a private OCI Object Storage bucket.
- [x] **MEDIA-02**: Application stores and retrieves one designated primary image plus additional supporting images for an autograph item.
- [x] **MEDIA-03**: Public users can view autograph images through app-mediated delivery without exposing public object URLs or bucket credentials.
- [x] **MEDIA-04**: Application keeps image objects and metadata references in sync so uploads and edits do not leave orphaned records or orphaned files in normal operation.

### Public Gallery

- [x] **GALL-01**: Anonymous user can browse a public gallery of published autograph items.
- [x] **GALL-02**: Anonymous user can filter or search the gallery by signer name, category, and tags.
- [x] **GALL-03**: Anonymous user can open a detail page for a single autograph item and view its full stored metadata.
- [x] **GALL-04**: Anonymous user can view all published images attached to an autograph item, including a clear primary presentation.

### Admin Workflow

- [ ] **ADMIN-01**: Exactly one admin authentication path exists for collection management, and no public user account system is required for v1.
- [ ] **ADMIN-02**: Admin can create a new autograph item by uploading images and reviewing/editing metadata in one workflow before publish.
- [ ] **ADMIN-03**: Admin can edit an existing autograph item, including metadata and associated images.
- [x] **ADMIN-04**: Admin can save reviewed metadata and publish the item so it becomes visible in the public gallery.
- [x] **ADMIN-05**: Admin routes, secrets, edit-history behavior, media cleanup, and operator-bridge retirement are reviewed for security and documented before the admin workflow is considered complete.

### Static Runtime Migration

- [x] **STATIC-01**: Public static artifact contracts are defined for gallery pages, item detail pages, search/facet data, generated media paths, and publish manifests.
- [x] **STATIC-02**: Static catalog generation runs inside the OCI/runtime boundary and does not require GitHub-hosted workflows to read catalog data, private image identifiers, Object Storage object keys, bucket details, or Oracle content.
- [x] **STATIC-03**: Published image derivatives are generated from private originals with sanitized filenames, stripped private metadata, and no leaked Object Storage URLs, UUIDs, namespaces, bucket names, or object keys.
- [x] **STATIC-04**: Caddy can serve generated static output through a local/private candidate validation path before planned public cutover.
- [x] **STATIC-05**: A static admin shell and thin private admin/publisher API foundation exist for health, private access enforcement, minimal content seeding into Oracle/Object Storage, and publish trigger/status behavior without implementing the full polished admin workflow yet.
- [x] **STATIC-06**: Cutover and retirement criteria are documented for the public Next.js runtime, public catalog APIs, app-mediated image streaming, current data smoke path, and temporary operator bridge; the current repo docs describe those paths as retired.
- [x] **STATIC-07**: The implemented static publishing path has a recorded live end-to-end proof: submit minimal metadata and an image through the private admin/API boundary, persist them to Oracle/Object Storage, generate static output, verify the generated public page and derivative image, and capture the Phase 5 closure summary.

### AI-Assisted Ingest

- [ ] **AI-01**: Upload workflow can generate AI-assisted metadata suggestions for fields such as signer, item type, tags, or inscription text.
- [ ] **AI-02**: OCR and AI suggestions remain advisory, and the admin can correct or ignore them before final save.
- [ ] **AI-03**: Upload workflow still succeeds with manual metadata entry when OCR or AI assistance is unavailable or inaccurate.
- [ ] **AI-04**: AI/OCR providers, prompts, failure modes, privacy boundaries, and configuration/secrets are reviewed for security and documented before AI-assisted ingest is considered complete.

### Public Showcase and Hardening

- [x] **SHIP-01**: Project has a security and attack-vector review covering the current app routes, temporary operator access, media delivery, secrets handling, infrastructure permissions, CI/CD permissions, container images, and repository settings, with later admin and AI phases responsible for follow-up reviews of their new surfaces.
- [x] **SHIP-02**: Dependency update automation is revisited and either Dependabot, Renovate, or a documented equivalent is configured for package dependencies, GitHub Actions, container images, Terraform providers/modules, and other maintained surfaces.
- [x] **SHIP-03**: Root `README.md` explains the project goals, architecture, local development flow, deployment model, operational notes, and the human+AI collaboration story clearly enough for a public audience.
- [x] **SHIP-04**: Repository badges and public metadata reflect current quality signals such as CI, linting, type checking, tests or coverage, image/dependency health, and deployment status where appropriate.
- [x] **SHIP-05**: Loose ends, stale docs, planning artifacts, open security findings, operational warnings, and showcase-readiness issues are triaged, fixed, or intentionally tracked before the repository is made public.

## v2 Requirements

None currently. Future scope should be added only if it directly supports the personal collection goal and is not already represented in v1.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Public user accounts in v1 | The site is a personal collection showcase, not a user platform |
| Social features such as likes, comments, or follows | Adds community and moderation scope irrelevant to the collection goal |
| Bulk import in v1 | Single-item workflows should be proven before adding ingestion complexity |
| Advanced search infrastructure | Metadata filters and simple search are enough for launch |
| Multi-admin roles and permissions | The brief only needs one admin path for the collection owner |
| Public multi-service platform split | v1 intentionally uses generated static public output plus one private Rust controller, not a public frontend/backend platform |
| Public direct object-storage URLs | Conflicts with the private-media requirement and centralized access control |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PLAT-01 | Phase 1 | Complete |
| PLAT-02 | Phase 1 | Complete |
| PLAT-03 | Phase 1 | Complete |
| DATA-01 | Phase 2 | Complete |
| DATA-02 | Phase 2 | Complete |
| DATA-03 | Phase 6 | Pending |
| DATA-04 | Phase 2 | Complete |
| MEDIA-01 | Phase 2 | Complete |
| MEDIA-02 | Phase 2 | Complete |
| MEDIA-03 | Phase 2 | Complete |
| MEDIA-04 | Phase 6 | Complete |
| GALL-01 | Phase 3 | Complete |
| GALL-02 | Phase 3 | Complete |
| GALL-03 | Phase 3 | Complete |
| GALL-04 | Phase 3 | Complete |
| ADMIN-01 | Phase 6 | Pending |
| ADMIN-02 | Phase 6 | Pending |
| ADMIN-03 | Phase 6 | Pending |
| ADMIN-04 | Phase 6 | Complete |
| ADMIN-05 | Phase 6 | Complete |
| STATIC-01 | Phase 5 | Complete |
| STATIC-02 | Phase 5 | Complete |
| STATIC-03 | Phase 5 | Complete |
| STATIC-04 | Phase 5 | Complete |
| STATIC-05 | Phase 5 | Complete |
| STATIC-06 | Phase 5 | Complete |
| STATIC-07 | Phase 5 | Complete |
| AI-01 | Phase 7 | Pending |
| AI-02 | Phase 7 | Pending |
| AI-03 | Phase 7 | Pending |
| AI-04 | Phase 7 | Pending |
| SHIP-01 | Phase 4 | Complete |
| SHIP-02 | Phase 4 | Complete |
| SHIP-03 | Phase 4 | Complete |
| SHIP-04 | Phase 4 | Complete |
| SHIP-05 | Phase 4 | Complete |

**Coverage:**
- v1 requirements: 36 total
- Mapped to phases: 36
- Unmapped: 0

---
*Requirements defined: 2026-04-18*
*Last updated: 2026-06-20 after recording the Phase 5 live static publish proof*
