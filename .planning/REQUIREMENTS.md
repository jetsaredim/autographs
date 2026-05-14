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
- [ ] **MEDIA-04**: Application keeps image objects and metadata references in sync so uploads and edits do not leave orphaned records or orphaned files in normal operation.

### Public Gallery

- [ ] **GALL-01**: Anonymous user can browse a public gallery of published autograph items.
- [ ] **GALL-02**: Anonymous user can filter or search the gallery by signer name, category, and tags.
- [ ] **GALL-03**: Anonymous user can open a detail page for a single autograph item and view its full stored metadata.
- [ ] **GALL-04**: Anonymous user can view all published images attached to an autograph item, including a clear primary presentation.

### Admin Workflow

- [ ] **ADMIN-01**: Exactly one admin authentication path exists for collection management, and no public user account system is required for v1.
- [ ] **ADMIN-02**: Admin can create a new autograph item by uploading images and reviewing/editing metadata in one workflow before publish.
- [ ] **ADMIN-03**: Admin can edit an existing autograph item, including metadata and associated images.
- [ ] **ADMIN-04**: Admin can save reviewed metadata and publish the item so it becomes visible in the public gallery.

### AI-Assisted Ingest

- [ ] **AI-01**: Upload workflow can generate AI-assisted metadata suggestions for fields such as signer, item type, tags, or inscription text.
- [ ] **AI-02**: OCR and AI suggestions remain advisory, and the admin can correct or ignore them before final save.
- [ ] **AI-03**: Upload workflow still succeeds with manual metadata entry when OCR or AI assistance is unavailable or inaccurate.

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
| Separate frontend and backend services | v1 intentionally uses one `Next.js` full-stack app |
| Public direct object-storage URLs | Conflicts with the private-media requirement and centralized access control |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PLAT-01 | Phase 1 | Complete |
| PLAT-02 | Phase 1 | Complete |
| PLAT-03 | Phase 1 | Complete |
| DATA-01 | Phase 2 | Complete |
| DATA-02 | Phase 2 | Complete |
| DATA-03 | Phase 4 | Pending |
| DATA-04 | Phase 2 | Complete |
| MEDIA-01 | Phase 2 | Complete |
| MEDIA-02 | Phase 2 | Complete |
| MEDIA-03 | Phase 2 | Complete |
| MEDIA-04 | Phase 4 | Pending |
| GALL-01 | Phase 3 | Pending |
| GALL-02 | Phase 3 | Pending |
| GALL-03 | Phase 3 | Pending |
| GALL-04 | Phase 3 | Pending |
| ADMIN-01 | Phase 4 | Pending |
| ADMIN-02 | Phase 4 | Pending |
| ADMIN-03 | Phase 4 | Pending |
| ADMIN-04 | Phase 4 | Pending |
| AI-01 | Phase 5 | Pending |
| AI-02 | Phase 5 | Pending |
| AI-03 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0

---
*Requirements defined: 2026-04-18*
*Last updated: 2026-05-14 after Phase 2 implementation*
