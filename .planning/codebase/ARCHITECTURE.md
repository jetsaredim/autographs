# Architecture

**Analysis Date:** 2026-05-28

## Pattern Overview

Autographs is now an implemented single-application system, not a planning-only repository. The current architecture is a full-stack `Next.js` App Router application under `app/`, backed by Oracle Autonomous Database for catalog metadata and private OCI Object Storage for autograph images. Public visitors browse only published items, while temporary operator-only mutation routes remain token-guarded and blocked at the public Caddy edge until Phase 5 replaces or retires them with the Rust private controller and minimal static admin seed/publish path.

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
- Purpose: Transitional token-guarded create, update, image attach, image delete, and item delete workflows for production data entry before the Phase 5 static-runtime/private-controller foundation.
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
- Operator API bridge: Temporary mutation surface used until Phase 5 replaces or retires it with the Rust private controller and minimal static admin seed/publish path.

## Current Phase Boundary

Phases 1-4 are complete. Phase 5 context is gathered and needs formal GSD phase planning before implementation. Phase 5 should prove the static runtime migration foundation: static public artifacts, public-safe JSON, generated derivatives, a Rust private admin/controller container, a minimal static admin seed/publish UI, publish validation, Caddy/runtime cutover, and retirement or replacement of the temporary Node operator bridge. Phase 6 should polish the daily-use admin collection workflow on that foundation. Phase 7 should add advisory AI-assisted ingest. Do not re-scaffold the existing delivery spine.

## Notable Absences

- Phase 5 Rust private controller, static publisher, and minimal static admin seed/publish path are not implemented yet.
- Polished Phase 6 admin collection workflow is not implemented yet.
- Edit history persistence/rendering is not implemented yet.
- AI-assisted metadata suggestions are not implemented yet.

---

*Architecture analysis refreshed: 2026-05-28 after Phase 5 static-runtime context gathering*
