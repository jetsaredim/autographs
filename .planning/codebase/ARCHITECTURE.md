# Architecture

**Analysis Date:** 2026-07-17

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

Phase 7 metadata taxonomy and public facets are implemented on this same
Rust/static foundation. The catalog now supports reusable signer profiles,
item signer credits, first-class item taxonomy, schema version 2 public JSON,
and generated public facets for signer, franchise, productLine, format,
language, origin, role, and tag. Phase 8 is now the pending operational
posture, CDN/cache, and admin media review/adjustment layer before Phase 9
taxonomy media cues and Phase 10 advisory AI-assisted ingest.

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
- The Phase 7 admin shell includes Identity, Classification, and Details
  taxonomy sections, repeatable signer rows, inline suggestions, `Possible
  duplicate signer` warnings, and `Merge signer` repair.

**Rust Controller**
- Location: `controller/src/`
- Purpose: Private admin health/auth routes, session-cookie collection
  management APIs, Oracle catalog and edit history access, private media
  access, media cleanup compensation/retry, pending-change status, static
  publishing, derivative generation, candidate validation, release retention,
  and release promotion.
- Key modules: `auth.rs`, `catalog.rs`, `config.rs`, `contracts.rs`,
  `derivatives.rs`, `media.rs`, `oci_media.rs`, `oracle_catalog.rs`,
  `publisher.rs`, `routes.rs`, `taxonomy_migration.rs`, and
  `storage_keys.rs`.

**Database Layer**
- Location: `controller/db/schema.sql`
- Purpose: Oracle schema used by the Rust controller and static publisher.
- Phase 7 schema includes signer profile tables, item signer credits,
  character/franchise joins, item taxonomy columns, and temporary legacy
  `signer`, `category`, and tag fields retained for rollback/reference.

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
   cleanup events, signer profiles, item signer credits, taxonomy fields, and
   private OCI Object Storage media.
5. The publisher generates candidate static output inside the runtime/OCI
   boundary, validates privacy and completeness, emits schema version 2
   collection/detail/facet artifacts, then promotes the release.
6. GitHub Actions validates code, builds/publishes the controller image,
   deploys runtime changes, and runs production maintenance workflows.

## Key Abstractions

- Rust controller routes: private admin/API boundary with session-cookie
  collection-management authorization.
- Static artifact contracts: public-safe gallery/detail/search/facet data and
  publish manifests, currently schema version 2 for the Phase 7 taxonomy.
- Publisher: candidate generation, validation, derivative creation, and release
  promotion with bounded release retention.
- Oracle catalog adapter: metadata, edit history, cleanup events, publication
  status, publish status, signer profiles, item signer credits, and taxonomy
  persistence for production.
- OCI media adapter: private original media access and retryable cleanup.
- Security patching role: scan, issue rendering, approval validation, patching,
  result reporting, and failure cleanup.

## Current Phase Boundary

Phase 7 metadata taxonomy and public facets are implemented through Plan 07-05
on top of the completed Phase 6 admin workflow. Current behavior includes
reusable signer profiles, item signer credits, signer suggestions, signer merge
repair, first-class character/franchise/productLine/setName/format/origin/
language taxonomy, reviewed backfill artifacts, schema version 2 public static
facets, rollout docs, security review, and live static publish smoke taxonomy
assertions. Phase 8 starts with production security patching repair, aggressive
operational posture cleanup in separate pre-media PRs, enforceable CI hygiene
guardrails where feasible, and a CDN/cache contract. Admin media image
preview/adjustment follows that clean baseline, including required
auto-assisted deskew/perspective correction with manual fallback, then
production CDN enablement is verified after adjusted-media cache behavior is
proven.

## Notable Absences

- Admin image previews and non-destructive image adjustment metadata, including
  auto-assisted deskew/perspective correction, are not implemented yet; they are
  Phase 8 scope.
- Optional taxonomy/media thumbnail cues and AI-assisted metadata suggestions
  are not implemented yet; taxonomy media cues are Phase 9 scope and advisory
  AI-assisted ingest is Phase 10 scope.
- Legacy signer/category/tag cleanup is not complete yet; Phase 7 documents
  temporary retention and a later deprecation path.

---

*Architecture analysis refreshed: 2026-07-28 after Phase 8/9/10 roadmap split*
