# Phase 05: Static Runtime Migration Foundation - Pattern Map

**Mapped:** 2026-05-28
**Status:** Complete

## Existing Patterns to Reuse

| New Area | Closest Existing Analog | Reuse Guidance |
|----------|-------------------------|----------------|
| Static public DTO contract | `app/src/catalog/public-view-models.ts` | Preserve public-safe field intent, facets, primary/supporting images, and detail groups. Replace `/api/catalog/.../images/...` routes with generated `/media/...` paths. |
| Privacy regression scans | `app/src/gallery/public-surface.test.ts` | Extend deny-list scanning to generated static HTML/JSON/manifests and static admin shell source. |
| Catalog source-of-truth schema | `app/db/migrations/001_catalog_core.sql` | Add minimal migrations for UUID-only original object keys, original filename metadata, publish jobs/status, release manifests, and derivative accounting. |
| Mutation and media behavior | `app/src/catalog/service.ts` | Mirror create/update/upload/primary-image semantics, but change new object key generation away from filename-bearing keys. |
| Oracle repository behavior | `app/src/catalog/repository.ts` | Preserve published-only reads, tag normalization, image ordering, and update semantics in Rust repository code. |
| Media abstraction | `app/src/media/types.ts`, `app/src/media/local-store.ts`, `app/src/media/oci-store.ts` | Build equivalent Rust traits for local filesystem and OCI/S3-compatible media so CI/local validation does not require live credentials. |
| Runtime deployment | `deploy/ansible/roles/autographs_deploy/tasks/main.yml` | Add Rust controller image, static release directories, secrets/env files, and quadlet templates using the existing role style. |
| Runtime containers | `deploy/ansible/roles/autographs_deploy/templates/autographs-app.container.j2` | Add a distinct `autographs-controller.container.j2` on the existing Podman network, with private localhost/container-network exposure. |
| Public routing | `deploy/ansible/roles/autographs_deploy/files/Caddyfile` | Shift from reverse-proxying all public traffic to serving generated static files, blocking retired routes, and proxying `/admin/api/*` privately. |
| Operator docs | `docs/temporary-production-data-entry.md`, `docs/deployment-runbook.md`, `docs/configuration-contract.md` | Replace temporary Node operator procedure with Rust admin/controller seed/publish and cutover procedure. |
| CI validation | `.github/workflows/ci.yml`, `app/package.json` | Extend validation with Rust format/test/build and static-publisher tests without making CI read private catalog content. |

## Candidate New File Areas

| Path | Role |
|------|------|
| `controller/` | New Rust private controller and publisher workspace. |
| `controller/src/main.rs` | Controller entrypoint, routing, config load, graceful startup. |
| `controller/src/config.rs` | Runtime env contract for Oracle, Object Storage, admin secret, release paths, media settings. |
| `controller/src/auth.rs` | Single-admin login, session cookie/token support, rate-limit/lockout helpers. |
| `controller/src/catalog.rs` | Catalog repository trait and Oracle/local implementations. |
| `controller/src/media.rs` | Private originals and generated derivative storage abstraction. |
| `controller/src/publisher.rs` | Static artifact generation, derivative generation, candidate validation, atomic promotion. |
| `controller/src/contracts.rs` | Versioned public JSON DTOs and publish manifest structs. |
| `controller/static-admin/` | Minimal static admin shell served by Caddy. |
| `deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2` | Podman quadlet for Rust controller. |
| `docs/static-runtime-runbook.md` | Operator guide for publish, validation, cutover, full rebuild, and old-path retirement. |

## Pattern Warnings

- Existing `buildObjectKey` includes the original filename in private object keys. Phase 5 should replace this for new uploads with UUID-only original keys and optional filename metadata.
- Existing public image DTO exposes image IDs in image route paths. Generated public media paths should avoid raw private image UUIDs.
- Existing Caddyfile blocks `/api/operator/*`; Phase 5 must preserve/expand blocking for retired `/api/catalog/*` and old operator routes after cutover.
- Existing app package is Node-only. Adding Rust should not disturb `corepack pnpm --filter app ...` commands.

## PATTERN MAPPING COMPLETE
