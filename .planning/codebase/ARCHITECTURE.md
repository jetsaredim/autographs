# Architecture

**Analysis Date:** 2026-05-25

## Pattern Overview

Autographs is now an implemented single-application system, not a planning-only repository. The current architecture is a full-stack `Next.js` App Router application under `app/`, backed by Oracle Autonomous Database for catalog metadata and private OCI Object Storage for autograph images. Public visitors browse only published items, while temporary operator-only mutation routes remain token-guarded and blocked at the public Caddy edge until Phase 5 replaces them with the real single-admin workflow.

## Layers

**Public Web Layer**
- Location: `app/app/`
- Purpose: Anonymous landing page, collection grid, detail pages, image viewer, architecture page, not-found states, and shared public components.
- Key files: `app/app/page.tsx`, `app/app/collection/page.tsx`, `app/app/collection/[id]/page.tsx`, `app/app/components/*`.

**Public API Layer**
- Location: `app/app/api/catalog/`
- Purpose: Published-only catalog list/detail access and app-mediated image delivery.
- Key files: `app/app/api/catalog/route.ts`, `app/app/api/catalog/[id]/route.ts`, `app/app/api/catalog/[id]/images/[imageId]/route.ts`.

**Temporary Operator API Layer**
- Location: `app/app/api/operator/catalog/`
- Purpose: Transitional token-guarded create, update, image attach, image delete, and item delete workflows for production data entry before Phase 5.
- Boundary: Must remain operator-only by deployment/routing procedure and bearer token; it is not the v1 admin UX.

**Catalog Service Layer**
- Location: `app/src/catalog/`
- Purpose: Domain types, Oracle repository, catalog service orchestration, public-safe view models, and tests.
- Key files: `service.ts`, `repository.ts`, `public-view-models.ts`, `types.ts`.

**Media Layer**
- Location: `app/src/media/`
- Purpose: Private media abstraction with OCI Object Storage and local filesystem-backed modes.
- Key files: `oci-store.ts`, `local-store.ts`, `config.ts`.

**Database Layer**
- Location: `app/src/db/`, `app/db/migrations/`, `app/scripts/`
- Purpose: Oracle configuration, migration execution, schema, seed data, and data smoke helpers.

**Infrastructure and Delivery Layer**
- Locations: `infra/terraform/`, `deploy/ansible/`, `.github/workflows/`
- Purpose: OCI infrastructure, runtime VM configuration, Podman quadlets, PR validation, image publishing, deploy, data smoke, and image cleanup.

**Public Showcase and Hardening Layer**
- Locations: `README.md`, `renovate.json`, `docs/security-review.md`, `docs/dependency-updates.md`, `docs/architecture.drawio`, `app/public/architecture-diagram.svg`
- Purpose: Public repository framing, dependency-update policy, current-surface security review, architecture presentation, and hardening documentation.

**Planning and Operator Documentation**
- Locations: `.planning/`, `docs/`, `.prompts/`
- Purpose: GSD state, roadmap, phase artifacts, codebase intelligence, bootstrap/runbook docs, and original product prompt.

## Data Flow

1. Anonymous visitors request `/`, `/collection`, or `/collection/{id}`.
2. Public pages call the catalog service, which lists/reads only `published` records by default.
3. Public view models convert private catalog records into public-safe DTOs and route image access through `/api/catalog/{itemId}/images/{imageId}`.
4. Image API routes resolve the published item and stream bytes from the configured private media store without exposing Object Storage URLs or object identifiers in the public UI.
5. Temporary operator API calls use `AUTOGRAPHS_OPERATOR_API_TOKEN`, then create/update Oracle rows and upload/delete private media through the same service layer. Production operator use goes through the documented SSH tunnel path because public Caddy blocks `/api/operator/*`.
6. GitHub Actions validates changes and deploys the containerized app/runtime changes to OCI on the documented path.

## Key Abstractions

- `CatalogService`: Coordinates metadata and media operations.
- `CatalogRepository`: Persists catalog records, tags, and image metadata through Oracle.
- `PrivateMediaStore`: Abstracts OCI Object Storage and local media modes.
- Public view models: Strip private storage fields and build app-mediated image routes.
- Operator API bridge: Temporary mutation surface used until Phase 5 admin workflow exists.

## Current Phase Boundary

Phases 1-3 are complete. Phase 4 is in progress: public-surface security headers, production health-detail redaction, Caddy/operator-route regression coverage, Renovate configuration, dependency-update docs, cleanup-job hardening, root README showcase content, and architecture/doc reconciliation have landed. Final Phase 4 readiness packaging remains pending. Phase 5 should build the admin workflow on the existing catalog service, media abstraction, public gallery, and operator bridge. It should not re-scaffold the application or replace the delivery spine.

## Notable Absences

- Real single-admin authentication and admin UX are not implemented yet.
- Edit history persistence/rendering is not implemented yet.
- AI-assisted metadata suggestions are not implemented yet.
- Final public-readiness checklist/signoff remains Phase 4 work.

---

*Architecture analysis refreshed: 2026-05-25 after repo-state reconciliation*
