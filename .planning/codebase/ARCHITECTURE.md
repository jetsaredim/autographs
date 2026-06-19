# Architecture

**Analysis Date:** 2026-06-19

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
- Purpose: Minimal private browser shell for seed/publish operations. It is the
  Phase 5 foundation, not the polished Phase 6 collection-management UX.

**Rust Controller**
- Location: `controller/src/`
- Purpose: Private admin health/auth routes, minimal content seed APIs, Oracle
  catalog access, private media access, static publishing, derivative
  generation, candidate validation, and release promotion.
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
   for health, minimal seed, and publish operations.
4. The controller reads and writes Oracle catalog metadata and private OCI
   Object Storage media.
5. The publisher generates candidate static output inside the runtime/OCI
   boundary, validates privacy and completeness, then promotes the release.
6. GitHub Actions validates code, builds/publishes the controller image,
   deploys runtime changes, and runs production maintenance workflows.

## Key Abstractions

- Rust controller routes: private admin/API boundary.
- Static artifact contracts: public-safe gallery/detail/search/facet data and
  publish manifests.
- Publisher: candidate generation, validation, derivative creation, and release
  promotion.
- Oracle catalog adapter: metadata persistence for production.
- OCI media adapter: private original media access.
- Security patching role: scan, issue rendering, approval validation, patching,
  result reporting, and failure cleanup.

## Current Phase Boundary

Phase 5 static runtime migration foundation is mostly implemented in the
checked-out code: Rust controller, static public artifacts, static admin shell,
publisher, deployment wiring, live/static runbooks, and retired Node/Next.js
runtime guidance are present. Plans 05-01 through 05-06 are done, but 05-07
live static publish proof and closure summary remain pending before closing the
phase.

After the 05-07 checkpoint passes, Phase 6 should focus on the polished daily-use admin
collection workflow: create/edit forms, publication controls, edit history,
richer media cleanup, and admin UX hardening on top of the Rust/static
foundation. Phase 7 remains advisory AI-assisted ingest.

## Notable Absences

- Polished Phase 6 admin collection workflow is not implemented yet.
- Edit history persistence/rendering is not implemented yet.
- Full media replacement/orphan cleanup ergonomics are not implemented yet.
- AI-assisted metadata suggestions are not implemented yet.

---

*Architecture analysis refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
