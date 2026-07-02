# Architecture

**Analysis Date:** 2026-07-02

## Pattern Overview

Autographs is now a static-public, Rust-controller system. The former active
Next.js runtime has been retired from the repository; public behavior lives in
generated static artifacts under `controller/static-public/`, and private
operator/admin behavior lives in the Rust controller under `controller/`.

Caddy is the public edge. It serves the generated static release, blocks
retired operator routes, and routes private `/admin` and `/admin/api/*`
traffic to the controller/admin surfaces. Oracle Autonomous Database remains
the metadata source of truth, and OCI Object Storage remains the private media
source of truth. The controller publishes public-safe pages, JSON, manifests,
and derived media from those private sources.

Phase 6 optimization adds explicit Caddy cache headers: admin routes are
`no-store`, public HTML/JSON stay short-lived for rollback, and public
assets/media use moderate cache lifetimes until paths become content-addressed
or release-addressed.

## Layers

**Static Public Layer**
- Location: `controller/static-public/`
- Purpose: Generated public landing, collection, detail template, architecture,
  data JSON, client assets, approved quote states, and derived-media paths.
- Boundary: Public artifacts must not expose private Object Storage URLs,
  bucket names, namespaces, object keys, Oracle internals, image UUIDs, or
  unpublished records.

**Static Admin Shell**
- Location: `controller/static-admin/`
- Purpose: Phase 6 private browser workflow for single-admin catalog
  management: health/diagnostics, item search/list, create/edit forms, image
  upload/primary/remove/replace controls, edit history, pending changes,
  publish actions, cleanup warnings, and release retention status.

**Rust Controller**
- Location: `controller/src/`
- Purpose: Private admin health/auth routes, session-cookie collection
  management APIs, Oracle catalog and edit history access, private media
  access, media cleanup compensation/retry, pending-change status, static
  publishing, derivative generation, candidate validation, release retention,
  and release promotion.
- Key modules: `auth.rs`, `catalog.rs`, `config.rs`, `contracts.rs`,
  `derivatives.rs`, `media.rs`, `oci_media.rs`, `oracle_catalog.rs`,
  `publisher.rs`, `routes.rs`, and `storage_keys.rs`.

**Database Layer**
- Location: `controller/db/schema.sql`
- Purpose: Oracle schema used by the Rust controller and static publisher.

**Media Layer**
- Location: `controller/src/media.rs`, `controller/src/oci_media.rs`
- Purpose: Private original media storage and retrieval, including OCI
  Object Storage access through runtime credentials/instance-principal signing.

**Infrastructure and Delivery Layer**
- Locations: `infra/terraform/`, `deploy/ansible/`, `.github/workflows/`
- Purpose: OCI infrastructure, runtime VM configuration, Podman quadlets, Caddy
  static/controller routing, controller image publishing, deploy validation,
  image cleanup, and production security patching.

**Production Security Patching Layer**
- Locations: `.github/workflows/weekly-security-scan.yml`,
  `.github/workflows/apply-security-updates.yml`,
  `.github/production-patch-approvers.yml`,
  `deploy/ansible/roles/security_patching/`, and `docs/security-patching.md`.
- Purpose: Weekly/manual production security update scans, scanner issue
  creation/update, allowlisted label approval, drift-checked `dnf` security
  updates, result comments, and failure cleanup.

**Planning and Operator Documentation**
- Locations: `.planning/`, `docs/`, `.prompts/`
- Purpose: GSD state, roadmap, phase artifacts, codebase intelligence,
  bootstrap/runbook docs, dependency policy, and historical prompt context.

## Data Flow

1. Anonymous visitors request the public site through Caddy.
2. Caddy serves the current generated static release: HTML, public-safe JSON,
   static assets, and generated media derivatives.
3. Operators use the private admin shell and `/admin/api/*` controller routes
   for session-cookie collection management, health, diagnostics, edit history,
   image maintenance, pending changes, and publish operations.
4. The controller reads and writes Oracle catalog metadata, edit history,
   cleanup events, and private OCI Object Storage media.
5. The publisher generates candidate static output inside the runtime/OCI
   boundary, validates privacy and completeness, then promotes the release.
6. GitHub Actions validates code, builds/publishes the controller image,
   deploys runtime changes, and runs production maintenance workflows.

## Key Abstractions

- Rust controller routes: private admin/API boundary with session-cookie
  collection-management authorization.
- Static artifact contracts: public-safe gallery/detail/search/facet data and
  publish manifests.
- Publisher: candidate generation, validation, derivative creation, and release
  promotion with bounded release retention.
- Oracle catalog adapter: metadata, edit history, cleanup events, publication
  status, and publish status persistence for production.
- OCI media adapter: private original media access and retryable cleanup.
- Security patching role: scan, issue rendering, approval validation, patching,
  result reporting, and failure cleanup.

## Current Phase Boundary

Phase 6 admin collection workflow is implemented through Plan 06-07 on top of
the completed Phase 5 Rust/static foundation. Current behavior includes the
polished static admin workflow, private item APIs, field-level edit history,
multi-image maintenance, retryable media cleanup, pending-change status,
explicit incremental/full publish controls, bounded release retention, a
session-cookie-only collection-management auth path, operator docs, and security
review. Plan 06-09 adds public detail derivative size reduction, explicit cache
headers, deferred Cloudflare/CDN guidance, and post-Phase 6 runtime cleanup
guidance. Phase 7 remains advisory AI-assisted ingest.

## Notable Absences

- AI-assisted metadata suggestions are not implemented yet.

---

*Architecture analysis refreshed: 2026-07-02 after Phase 6 optimization closeout*
