# Project Research Summary

**Project:** Autographs
**Domain:** Self-hosted autograph catalog and private-photo-backed public gallery
**Researched:** 2026-04-18
**Confidence:** MEDIUM-HIGH

## Executive Summary

Autographs is a narrow v1 product: an anonymous public autograph gallery backed by private image storage, with one admin responsible for catalog growth. The research consistently points to a lean, production-real architecture rather than a prototype stack. Experts would build this as a single self-hosted `Next.js` application with Oracle as the metadata store, OCI Object Storage for private images, and a GitHub-driven deployment path, because the core challenge is not feature breadth but delivering a trustworthy end-to-end system that one operator can actually run.

The recommended approach is to establish the delivery spine and hardest infrastructure seams first, then layer in public catalog features, admin upload, and finally AI assistance. That means proving OCI bootstrap, compartment-scoped CI/CD, Oracle connectivity and migrations, and app-mediated private image delivery before spending roadmap weight on UI polish or OCR/AI enhancements. The product should ship as one containerized app on OCI Ampere A1 with `nginx` in front, `node-oracledb` thin mode for Oracle access, and relational metadata search over signer, category, and tags rather than an advanced search platform.

The biggest risks are phase-ordering risks rather than product-definition risks. The research is especially clear that late discovery of Oracle runtime friction, over-privileged GitHub Actions access, and naive image proxying would create the most expensive rework. Roadmap structure should therefore favor early implementation proof for database, storage, and deployment seams, keep OCR/AI advisory-only, and defer anything that smells like multi-user auth, social features, bulk import, or advanced search until the core catalog loop is proven.

## Key Findings

### Recommended Stack

The stack recommendation is unusually crisp because the project brief already narrows the problem well. v1 should be one `Next.js 16.x` full-stack app on `Node.js 22 LTS`, written in `TypeScript 5.x`, running behind `nginx` on a single OCI Always Free Ampere A1 host. Metadata belongs in Oracle Autonomous Database Free, accessed directly from the app through `node-oracledb 6.x` thin mode, and images belong in a private OCI Object Storage bucket with all public access mediated by the app.

This stack is recommended because it matches the explicit product constraints: one deployable app, one operator, private media, GitHub-driven deploys, and OCI Always Free bias. It deliberately avoids platform sprawl, ORM-first abstraction, public object URLs, and extra services that would complicate the first release without improving validation.

**Core technologies:**
- `Next.js 16.x`: Full-stack web app for public pages, admin UI, route handlers, and image mediation — recommended because one self-hosted process satisfies the single-app requirement cleanly.
- `Node.js 22 LTS`: Runtime for local, CI, and production — recommended because it exceeds Next.js minimum support while staying mainstream and durable.
- `TypeScript 5.x`: Shared language for app, server, and scripts — recommended because cross-boundary contracts will matter immediately.
- Oracle Autonomous Database Free plus `node-oracledb 6.x` thin mode: System of record for autograph metadata and admin state — recommended because the brief prefers Oracle and thin mode avoids client-library deployment friction.
- OCI Object Storage: Private image storage — recommended because the product requires app-mediated access to non-public media.
- OCI Ampere A1 plus `nginx` plus Docker: Production host boundary — recommended because it is the simplest durable self-hosted OCI shape for one containerized app.
- Terraform `1.9+` and GitHub Actions: Infrastructure and delivery automation — recommended because CI/CD and OCI bootstrap are foundational requirements, not later polish.
- `zod`, `sharp`, `pino`, `vitest`, and Playwright: Validation, image normalization, observability, and test coverage — recommended because they harden the vertical slice without overcomplicating it.

### Expected Features

The feature research separates true table stakes from useful differentiators cleanly. The MVP is not a social product or a generalized collector platform. It is a browsable autograph catalog with private media, rich metadata, and a single-admin ingestion workflow. The roadmap should treat catalog usefulness and operator reliability as launch criteria, and treat community, scale, and editorial expansion as later bets.

**Must have (table stakes):**
- Anonymous public gallery grid/list — the front door must work without accounts.
- Item detail pages with collectible-specific metadata — each autograph needs a canonical detail view.
- Metadata search and filtering by signer, category, and tags — required discovery surface for a catalog product.
- Private image delivery through the app — central trust and privacy requirement, not optional polish.
- Single-admin authenticated upload path — the catalog must be maintainable by one operator.
- Image upload plus metadata form — image and record creation must succeed as one workflow.
- Searchable metadata schema — signer, category, tags, event/source, inscription, certification details, estimated year, and description.
- Review-before-publish validation — catalog quality matters more than upload speed.

**Should have (competitive):**
- AI-assisted metadata suggestions during upload — valuable operator acceleration as long as humans stay in control.
- Hybrid OCR plus AI extraction — useful for inscriptions and certificate text, but only if it remains replaceable and non-blocking.
- Structured collectible-specific metadata — differentiates the product from a generic photo gallery.
- App-mediated private media model — both an architectural and user-trust differentiator.

**Defer (v2+):**
- Multiple admin accounts or roles — too much auth and permission surface for v1.
- Public user accounts, submissions, or social features — outside the validation target.
- Multi-image items — adds editorial and storage complexity too early.
- Bulk import tooling — scale optimization before the single-item loop is proven.
- Advanced search infrastructure, relevance tuning, or vector search — relational filtering is enough for launch.
- Edit history, moderation workflows, analytics, and collection trends — useful later, not needed to validate the core product.

### Architecture Approach

The architecture should stay monolithic in deployment shape but modular in code structure. The browser talks only to the `Next.js` app, the app talks to Oracle and Object Storage, and GitHub Actions is the only automated deploy actor. Internally, route handlers should stay thin and delegate to service modules so OCI SDK logic, SQL, and auth concerns remain isolated without introducing extra runtime services.

**Major components:**
1. `Next.js` web app — public gallery pages, item detail pages, admin UI, auth checks, route handlers, and image mediation.
2. Metadata service layer — validates inputs, owns SQL queries, and maps Oracle rows into application models.
3. Storage service layer — uploads objects, generates app-owned keys, and streams private images from Object Storage.
4. Oracle Autonomous Database Free — canonical source of metadata, searchable fields, object references, and admin state.
5. OCI Object Storage private bucket — stores uploaded autograph images and possible derived variants later.
6. OCI Compute host with `nginx` and Docker — stable runtime boundary for the self-hosted app.
7. GitHub Actions deploy pipeline — validation on PRs and controlled rollout on merge to `main`.

### Critical Pitfalls

1. **Late Oracle proof** — avoid by validating container build, Oracle connection, migrations, and a real read/write cycle before feature-heavy phases.
2. **Over-privileged CI/CD** — avoid by separating manual break-glass bootstrap from routine deploy automation and scoping GitHub Actions to compartment-level permissions only.
3. **Naive app-side image proxying** — avoid by streaming objects, adding thumbnails or constrained variants, and setting deliberate cache behavior from the first media phase.
4. **Phase 1 scope collapse** — avoid by defining the first deliverable as a thin vertical slice with explicit non-goals rather than “the whole prompt.”
5. **Blurred manual versus automated OCI bootstrap** — avoid by labeling every setup step as `manual once`, `automated`, or `manual break-glass only`.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Delivery Spine and OCI Bootstrap
**Rationale:** The repo’s biggest current gap is absence of an executable backbone. Delivery, secret contracts, and OCI ownership boundaries need to exist before application work spreads.
**Delivers:** Repo/app scaffold, container shape, env contract, GitHub Actions validation/deploy skeleton, Terraform baseline, documented manual bootstrap and IAM ownership model.
**Addresses:** Foundational operability for every later feature.
**Avoids:** Prompt-as-product confusion, over-privileged CI/CD, ambiguous tenancy bootstrap ownership.

### Phase 2: Oracle and Storage Proof Slice
**Rationale:** The most expensive hidden risk is discovering Oracle or private media friction after UI and workflow code already depend on them.
**Delivers:** Oracle connectivity, schema and migration path, health/read-write proof, private Object Storage upload/download, app-mediated image streaming endpoint, seedable test data.
**Uses:** `node-oracledb` thin mode, OCI Object Storage, `zod`, Docker, OCI A1 runtime assumptions.
**Implements:** Metadata service layer and storage service layer.
**Avoids:** Late Oracle runtime friction and naive image delivery design.

### Phase 3: Public Catalog MVP
**Rationale:** Once persistence and image mediation are proven, the public anonymous experience can be built against stable infrastructure seams.
**Delivers:** Gallery list/grid, item detail page, searchable metadata model, signer/category/tag filters, conservative image presentation strategy.
**Addresses:** Core table-stakes user experience for anonymous browsing.
**Avoids:** Premature advanced search work and metadata/storage coupling.

### Phase 4: Single-Admin Upload and Review
**Rationale:** Catalog growth depends on one reliable admin workflow, but auth and upload should build on already-working database and storage boundaries.
**Delivers:** Single-admin login path, upload form, image-plus-metadata workflow, manual review/edit/confirm before publish, validation around object references and searchable metadata.
**Addresses:** Single-admin authenticated upload, review-before-publish, image upload plus metadata form.
**Avoids:** Mini identity-system creep and orphaned records/files.

### Phase 5: AI/OCR Assist and Hardening
**Rationale:** AI assistance is useful only after the manual workflow works. Hardening belongs here because logging, caching, backups, and runbooks become meaningful once the end-to-end product exists.
**Delivers:** Advisory OCR/AI suggestion pipeline, graceful fallback to manual entry, thumbnail/caching improvements, structured logging, backup/export docs, deployment/runbook polish, smoke tests.
**Addresses:** Differentiating metadata suggestions and operational resilience.
**Avoids:** Making AI/OCR a blocking dependency and leaving media performance or recovery posture underspecified.

### Phase Ordering Rationale

- Delivery infrastructure comes first because every later phase depends on trusted deployment, secrets, and OCI ownership boundaries.
- Oracle and Object Storage proof comes before user-facing features because those are the highest-risk integrations and the most likely sources of rework.
- Public catalog comes before admin AI enhancements because anonymous browse/search/detail is the main product value and can validate the metadata model early.
- Admin upload follows stable storage and database seams so the write path is built on proven foundations rather than assumptions.
- AI/OCR lands last in the MVP sequence because it is a differentiator, not a prerequisite, and the research is explicit that it must remain advisory-only.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** OCI IAM/bootstrap specifics still need careful translation into exact manual and automated steps for this tenancy.
- **Phase 2:** Oracle migration tooling choice and containerized connection handling need proof, not just a default preference.
- **Phase 5:** OCR performance on OCI A1 and the right AI metadata extraction contract need implementation-time validation.

Phases with standard patterns (skip research-phase):
- **Phase 3:** Public gallery list/detail/filter patterns are well-understood once the schema exists.
- **Phase 4:** Single-admin form workflows and review-before-publish patterns are straightforward if scope stays narrow.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | The stack direction is strongly constrained by the project brief and supported by official Next.js and OCI guidance summarized in the research. |
| Features | HIGH | Must-have versus later scope is clear and consistent across `PROJECT.md` and `FEATURES.md`. |
| Architecture | HIGH | The single-app, app-mediated, OCI-hosted shape is well justified for the operator model and product constraints. |
| Pitfalls | MEDIUM-HIGH | The major risks are credible and actionable, though some are forward-looking implementation risks that still need hands-on proof. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Oracle migration/tooling proof:** The research favors SQL-first Oracle access, but the exact migration workflow still needs implementation-time validation in the real container/runtime setup.
- **Admin authentication implementation choice:** Scope is clear, but the exact single-admin auth mechanism still needs to be selected to balance simplicity, recoverability, and security.
- **OCR viability on Always Free compute:** OCR is a good differentiator, but A1 runtime performance and failure behavior need proof before it becomes a committed roadmap deliverable.
- **Backup and recovery specifics:** Oracle Free limitations and private media storage imply a real export/recovery plan, but that plan is not yet concretely defined.
- **Traffic and image caching assumptions:** App-mediated delivery is the right default, but thumbnail strategy and cache policy should be validated early enough to avoid the app becoming a media bottleneck.

## Sources

### Primary (HIGH confidence)
- `.planning/PROJECT.md` — product scope, constraints, and out-of-scope boundaries
- `.planning/research/STACK.md` — recommended technologies, version families, and OCI/Next.js fit
- `.planning/research/FEATURES.md` — table stakes, differentiators, anti-features, and MVP scope
- `.planning/research/ARCHITECTURE.md` — deployment topology, data flow, component boundaries, and build order
- `.planning/research/PITFALLS.md` — critical implementation and phase-ordering risks

### Secondary (MEDIUM confidence)
- Official guidance summarized inside the research files for Next.js self-hosting, Node support, OCI Always Free primitives, and Oracle thin-mode access

---
*Research completed: 2026-04-18*
*Ready for roadmap: yes*
