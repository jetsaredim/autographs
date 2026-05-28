# Phase 5: Static Runtime Migration Foundation - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 5 proves the migration from the current public Next.js runtime/operator bridge toward a static public catalog generated inside the OCI runtime boundary, with a minimal Rust private admin/controller container and a minimal static admin UI capable of keeping Oracle Autonomous Database and private Object Storage writable.

This phase includes static public artifact generation, public-safe JSON contracts, private original media and generated derivative handling, a Rust controller with minimal functional admin APIs, a minimal admin UI, publish validation, Caddy/runtime cutover, and operator documentation for the new path.

This phase does not deliver the polished daily-use admin workflow, edit-history browsing, rich validation UX, advanced media management, AI/OCR assistance, public accounts, multi-admin roles, or a long-lived compatibility bridge with the old Node public/operator paths.

</domain>

<decisions>
## Implementation Decisions

### Static Public Artifact Shape
- **D-05-01:** Phase 5 should preserve current public browsing and filtering behavior without a live public Node catalog API by generating static HTML plus public-safe JSON data.
- **D-05-02:** The generated output should include static public pages for the landing, collection, and item detail surfaces, plus JSON artifacts for collection data, facets/filtering, image references, and publish metadata.
- **D-05-03:** Phase 5 should profile public JSON artifact shapes before freezing the final split. Test at least a single catalog JSON, split indexes, and a hybrid using realistic generated fixture data.
- **D-05-04:** Profiling should target roughly 500 catalog items with multi-image coverage as reasonable headroom. The real collection/backlog is about 120 items, production currently has about 3 live items, and expected growth is about 10-20 pieces per year.
- **D-05-05:** Phase 5 should define a lightweight versioned public JSON contract. The locked parts are public-safe field intent, privacy rules, and compatibility/migration behavior; public field names and shapes may improve where the current internal schema is ambiguous.
- **D-05-06:** Public JSON fields should be deliberate display/filter fields, not a direct mirror of Oracle rows.

### Rust Admin Controller and Publish Boundary
- **D-05-07:** Phase 5 should build the minimal private admin/controller container in Rust.
- **D-05-08:** The Rust controller is the future home for private admin APIs and publish orchestration. Phase 5 should implement enough functional API behavior to replace the Node operator bridge for content entry and publishing, but not full polished admin product UX.
- **D-05-09:** The Rust controller should support minimal create/update/upload/publish workflows: create autograph metadata, update basic metadata/status, upload or attach private original images, set basic primary/supporting image order, publish/unpublish, trigger incremental publish, trigger full rebuild, and report publish status.
- **D-05-10:** Publishing should be triggered by the admin UI through a private controller API and should run inside the OCI/runtime boundary, not in GitHub-hosted workflows.
- **D-05-11:** GitHub Actions should build and deploy code/images only. Catalog content generation must not expose Oracle data, Object Storage object identifiers, bucket details, image UUIDs, private object keys, or private media through GitHub-hosted workflows.
- **D-05-12:** Incremental publish is the default target. New items should update the item detail page, collection JSON/indexes/facets, publish manifest, and needed public image derivatives. Existing item updates should regenerate only the specific item page and affected indexes/manifests/derivatives where practical.
- **D-05-13:** Phase 5 should not build a generalized dependency graph. Use a small explicit artifact-impact map for known catalog changes, plus an explicit full rebuild option for structural template/schema changes or repair.
- **D-05-14:** The Rust controller should stay intentionally small. Avoid framework-heavy service design or a generic job system unless research proves it is necessary.
- **D-05-15:** Rust is a deliberate project direction, not a performance requirement at current scale. Planning should isolate early risks around Rust Oracle access, OCI/Object Storage or S3-compatible access, multipart uploads, image processing, Caddy integration, and container deployment.

### Admin Access Model
- **D-05-16:** The admin/controller should accept requests from the private static admin UI and from local/operator system calls when needed.
- **D-05-17:** Phase 5 should use same-host admin routes as the target shape: `/admin` serves the minimal static admin UI and `/admin/api/*` reaches the Rust controller.
- **D-05-18:** SSH tunnel or local curl access can still hit the same internal admin routes for maintenance and repair.
- **D-05-19:** The admin login surface may be internet-reachable, so Phase 5 must treat authentication, rate limiting/basic abuse resistance, and error redaction as real security work.
- **D-05-20:** Use a simple single-admin login endpoint in the Rust controller. Base64 is not security and must not be treated as protection.
- **D-05-21:** Prefer a same-origin secure HTTP-only session cookie for browser UI if feasible. Provide a CLI-friendly token/session path for local operator calls.
- **D-05-22:** Admin secrets must live in runtime/operator secret stores, not git.

### Minimal Static Admin UI
- **D-05-23:** Phase 5 should include a minimal static admin UI shell backed by the Rust controller.
- **D-05-24:** The Phase 5 admin UI should be functional rather than polished. A single guided create/edit form with upload and publish controls is acceptable for this phase.
- **D-05-25:** The admin UI should prove enough end-to-end behavior to operate the new path: create/edit basic metadata, upload at least one original image, publish/unpublish, trigger incremental publish, trigger full rebuild, and show publish status/failure details.
- **D-05-26:** Phase 5 admin UI exists to replace curl-only or Node-operator workflows, especially for loading and publishing the real 120-item collection backlog after migration. Phase 6 can turn this into a polished daily-use admin workflow.

### Media Originals, Derivatives, and Object Storage
- **D-05-27:** Private originals remain the durable source of truth in Object Storage. Public derivatives are generated artifacts and can be rebuilt.
- **D-05-28:** Move media object naming away from filename-bearing object keys. New private originals should use UUID-only object keys. Original filenames may be stored as metadata if useful but should not be part of Object Storage keys.
- **D-05-29:** Existing filename-bearing private objects do not need full zero-downtime compatibility. Phase 5 may use planned downtime, object-key/schema migration, manual recreation, or VM rebuild to reach the cleaner target architecture.
- **D-05-30:** Avoid local derivative copies as the long-term media serving model, and avoid adding extra buckets unless research proves a hard OCI/Caddy limitation.
- **D-05-31:** Prefer Object Storage-backed public-safe derivatives, ideally in the same bucket with strict prefixes/namespaces unless research shows separate bucket/prefix/permissions are cleaner.
- **D-05-32:** Caddy may serve `/media/...` by proxying to the derivative Object Storage prefix through an S3-compatible plugin or equivalent. It must not proxy private originals.
- **D-05-33:** `github.com/lindenlab/caddy-s3-proxy` is a candidate, not a locked dependency. Phase 5 research/planning should evaluate OCI S3 compatibility, read-only enforcement, maintenance status, cache/header behavior, missing-object behavior, and custom Caddy build/deploy impact. Maintained alternatives or forks may be preferable.
- **D-05-34:** Public derivative paths should be sanitized and deterministic, and should not expose private original object keys, Object Storage URLs, bucket names, namespaces, original filenames, Oracle row IDs, or private image UUIDs.
- **D-05-35:** Generate only the minimum useful derivative set in Phase 5: thumbnail and detail variants, with an extensible manifest that can add more variants/formats later.
- **D-05-36:** Do not generate full responsive variant sets in Phase 5 unless profiling/research shows a clear need with little complexity.
- **D-05-37:** Derivatives should be public display derivatives, not merely smaller copies. They should resize, strip metadata, normalize orientation where possible, and use format conversion/compression if the Rust image ecosystem validates cleanly.
- **D-05-38:** Generate derivatives during publish, not in the public request path. Draft preview behavior, if needed, should stay private rather than writing public derivatives for drafts.
- **D-05-39:** The OCI Always Free 10 GB Object Storage limit should be considered, but it is not the primary design constraint at expected scale. Current originals are usually around 2-2.5 MB with a few roughly 20 MB outliers.
- **D-05-40:** Track storage usage and derivative byte sizes enough to spot extreme growth, enforce reasonable upload limits, and clean stale derivatives.

### Static Publish, Validation, and Cutover
- **D-05-41:** Phase 5 should use a two-step cutover. First deploy/prove the Rust controller and static generator internally, then use a short planned downtime window for route/runtime cutover.
- **D-05-42:** The project can tolerate downtime, schema/object-key migration, and even VM rebuild if that yields a cleaner architecture. The site is live but not mission critical.
- **D-05-43:** Avoid maintaining a long-lived compatibility bridge with the old Node public/operator paths.
- **D-05-44:** Planner discretion is allowed on whether to migrate or manually recreate the current roughly 3 production items. The priority is a clean target architecture, not preserving old filename-bearing key conventions.
- **D-05-45:** Use a roll-forward-only migration posture. No elaborate rollback mechanism is required. Preflight validation is still mandatory, and full rebuild remains a repair tool.
- **D-05-46:** Static generation should use incremental artifact generation with complete candidate-release validation.
- **D-05-47:** Caddy should serve a live `current` release path. The publisher should create a candidate release based on the current release, regenerate only affected HTML/JSON/manifest artifacts, validate the candidate as a complete site, and atomically promote the candidate to `current`.
- **D-05-48:** Atomic promotion can use symlink swap or equivalent same-filesystem atomic rename/pointer update. Caddy should see either the old current release or the new one, never a half-written site.
- **D-05-49:** Candidate validation should be possible before promotion through filesystem checks and a local-only Caddy listener or equivalent route bound to `127.0.0.1`.
- **D-05-50:** Validation should include manifest consistency, privacy scans, generated page smoke checks, collection/filter JSON checks, detail page existence checks, media derivative existence checks, storage usage summary, Caddy route/content-type checks, and a publish status/result record.
- **D-05-51:** If validation fails, retain only the latest failed candidate release and logs for debugging; prune older failed candidates.
- **D-05-52:** Admin notification for publish failures should be in-product and API/CLI-based for Phase 5. The Rust controller records publish attempts and exposes status/log details to the admin UI and local/operator calls. No email/SMS/webhook notification is required in Phase 5.

### Carried Forward From Earlier Phases
- **D-05-53:** Public visitors browse anonymous, published-only catalog content.
- **D-05-54:** Public pages and public JSON must not expose direct Object Storage URLs, private object keys, bucket names, namespaces, storage credentials, Oracle data, or browser-visible private storage identifiers.
- **D-05-55:** Temporary Node operator routes remain transitional and should be replaced or retired by this phase's Rust controller/admin path.
- **D-05-56:** Keep the system operable by one developer and biased toward OCI Always Free-compatible primitives.
- **D-05-57:** Continue using the existing OCI/Oracle/Object Storage/GitHub/Ansible/Podman/Caddy delivery spine rather than re-scaffolding the whole project.

### the agent's Discretion
- Exact Rust framework, crate choices, and internal module structure, provided the implementation stays small and validates the high-risk integrations early.
- Exact public JSON split after profiling, provided current public browse/filter behavior is preserved without a live public Node API.
- Exact derivative dimensions, formats, and compression settings, provided Phase 5 starts with only thumbnail and detail variants and keeps the manifest extensible.
- Exact Caddy/S3 proxy plugin or alternative, provided it serves only public-safe derivatives and does not expose private originals or unsafe identifiers.
- Exact migration choice for the current 3 production items, provided the target architecture is clean and old filename-bearing key conventions are not preserved unnecessarily.
- Exact static release directory layout, provided it supports candidate validation, atomic promotion, and incremental artifact updates.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Requirements
- `.planning/ROADMAP.md` — Defines Phase 5 goal, success criteria, dependencies, and static runtime migration scope.
- `.planning/REQUIREMENTS.md` — Defines `STATIC-01` through `STATIC-07` plus current admin/media/public constraints.
- `.planning/PROJECT.md` — Defines product constraints, key decisions, and current static-runtime pivot.
- `.planning/STATE.md` — Records Phase 4 completion, Phase 5 planning readiness, static-runtime boundary decisions, and current concerns.

### Prior Phase Decisions
- `.planning/phases/04-public-showcase-and-hardening/04-CONTEXT.md` — Public-readiness, security, documentation, and operator-route hardening decisions.
- `.planning/phases/03-public-gallery-mvp/03-CONTEXT.md` — Public gallery UX, filtering, detail page, image viewer, app-mediated image privacy, and temporary operator bridge decisions.
- `.planning/phases/02-oracle-and-private-media-core/02-CONTEXT.md` — Oracle/private media source-of-truth, private Object Storage, and app-mediated image delivery decisions.

### Codebase Maps
- `.planning/codebase/STACK.md` — Current stack, runtime, validation, and deployment tools.
- `.planning/codebase/ARCHITECTURE.md` — Current layers, data flow, service boundaries, and temporary operator API.
- `.planning/codebase/INTEGRATIONS.md` — OCI, Oracle, Object Storage, GitHub Actions, GHCR, Caddy, and operator integration context.
- `.planning/codebase/CONVENTIONS.md` — Naming, TypeScript, testing, documentation, and planning conventions.
- `.planning/codebase/STRUCTURE.md` — Repository structure and likely locations for new app/deploy/docs work.
- `.planning/codebase/TESTING.md` — Current validation contract and regression-test expectations.
- `.planning/codebase/CONCERNS.md` — Security and fragility concerns to re-check against the new Rust/static/admin surfaces.

### Current Implementation and Operations
- `app/src/catalog/service.ts` — Current catalog service behavior, image attachment, object-key generation, and private media reads.
- `app/src/catalog/repository.ts` — Current Oracle repository behavior and UUID record creation.
- `app/src/catalog/public-view-models.ts` — Current public-safe DTO/view-model shaping to preserve or replace in generated JSON.
- `app/src/catalog/public-view-models.test.ts` — Current public DTO privacy regression tests.
- `app/src/gallery/public-surface.test.ts` — Current public-surface privacy regression tests.
- `app/db/migrations/001_catalog_core.sql` — Current Oracle schema for items/images/tags and object-key storage.
- `app/app/collection/page.tsx` — Current collection page behavior to preserve in static output.
- `app/app/collection/[id]/page.tsx` — Current detail page behavior to preserve in static output.
- `app/app/api/catalog/` — Current public catalog/image APIs to retire or replace with static artifacts.
- `app/app/api/operator/catalog/` — Current temporary Node operator bridge to replace/retire.
- `app/scripts/smoke-data.ts` — Current data/media smoke behavior and temporary content loop reference.
- `docs/temporary-production-data-entry.md` — Current SSH-tunnel/operator bridge procedure to replace.
- `docs/configuration-contract.md` — Current runtime/GitHub/operator configuration and secret contract to extend.
- `docs/deployment-runbook.md` — Current deployment/operator workflow to update for Rust/static cutover.
- `deploy/ansible/roles/autographs_deploy/files/Caddyfile` — Current public Caddy routing and operator blocking; target for static/admin/media routing changes.
- `deploy/ansible/` — Runtime VM configuration, Podman quadlets, Caddy, app image deployment, and likely Rust controller/static path integration point.
- `.github/workflows/` — CI/deploy/data-smoke/image-cleanup workflows; GitHub must deploy code/images, not generate private catalog content.

### External Candidate References To Verify During Research
- `github.com/lindenlab/caddy-s3-proxy` — Candidate Caddy S3-compatible proxy plugin for serving public-safe derivatives from OCI Object Storage. Must be evaluated before adoption.
- OCI Object Storage S3 Compatibility API documentation — Required if using a Caddy S3-compatible proxy or Rust S3-compatible client path against OCI Object Storage.
- Rust image processing ecosystem documentation — Required before locking derivative generation crate/format/orientation behavior.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/src/catalog/service.ts`: Current service defines the existing CRUD/media behavior and reveals the current filename-bearing private object key shape: `autographs/{itemId}/{randomUUID()}-{safeFilename}`.
- `app/src/catalog/repository.ts`: Existing Oracle repository creates UUID item/image rows and can guide the Rust controller's source-of-truth behavior even if direct code reuse is not possible.
- `app/src/catalog/public-view-models.ts`: Current public DTO stripping can guide generated JSON privacy boundaries.
- `app/src/catalog/public-view-models.test.ts` and `app/src/gallery/public-surface.test.ts`: Existing privacy tests should inspire static JSON/HTML/media privacy regression coverage.
- `app/app/collection/page.tsx` and `app/app/collection/[id]/page.tsx`: Current browse/detail behavior should be treated as the UX parity baseline for generated static output.
- `app/scripts/smoke-data.ts`: Existing smoke script proves a create/read/image/delete loop and can inform the new Rust/controller publish smoke.
- `deploy/ansible/roles/autographs_deploy/files/Caddyfile`: Current public edge can be extended or replaced for static root, admin routes, local-only preview, and derivative media proxying.

### Established Patterns
- Public DTOs must stay free of private storage identifiers.
- Public catalog behavior is anonymous and published-only.
- Temporary operator routes are protected and not part of public browsing.
- Configuration and secret contracts are documented in committed examples/docs, with real values supplied via environment, GitHub secrets, or operator stores.
- Runtime deployment is Ansible-managed with Podman quadlets and Caddy, not compose/cloud-init sprawl.
- CI and local tests should validate without live production OCI credentials where possible.

### Integration Points
- New Rust controller container must integrate with Ansible/Podman/Caddy alongside or in place of the current app container.
- Caddy must route static public pages, `/admin`, `/admin/api/*`, local-only candidate preview, and `/media/...` derivative access without exposing private originals.
- Oracle schema may need migration for UUID-only object keys, original filename metadata, publish status, generated artifact metadata, publish jobs, and storage accounting.
- Object Storage layout must distinguish private originals from public-safe generated derivatives, likely by strict prefixes/namespaces.
- GitHub Actions should build/deploy code and images while leaving content generation to the OCI runtime/controller.
- Docs must replace the temporary Node operator bridge procedure with the Rust admin/controller, static publish, and cutover runbooks.

</code_context>

<specifics>
## Specific Ideas

- Candidate static release layout can look like `/var/lib/autographs/static/releases/{release-id}/` with `current` as a symlink to the active release.
- Publisher can seed candidate releases from the current release with hard links or another efficient local-copy strategy, regenerate only affected artifacts, validate the complete candidate, then atomically swap `current`.
- A local-only Caddy listener bound to `127.0.0.1` can serve the candidate release for route/header/content-type smoke validation before promotion.
- Public derivatives can be served by Caddy at clean `/media/...` paths while proxying to Object Storage derivative objects behind the scenes.
- Private originals and public derivatives should use UUID-only or public-safe deterministic naming; original filenames should not be object-key components.
- Admin UI can be a single guided form in Phase 5, as long as it can create/edit/upload/publish and show publish status/failure details.
- Publish failure notification in Phase 5 can be the admin UI status panel plus API/CLI responses, not email/SMS/webhook alerts.
- The current production data set is small enough that planners can choose manual recreation through the new admin UI if that better proves the new path.

</specifics>

<deferred>
## Deferred Ideas

- Polished admin workflow, richer validation UX, edit-history browsing, advanced media reordering/removal, orphan cleanup guarantees, dashboard polish, and daily-use ergonomics remain Phase 6.
- AI/OCR metadata suggestions remain Phase 7 and should stay advisory.
- Full responsive image variant sets and additional public image formats can be added later if profiling or UX needs justify them.
- Out-of-band publish notifications such as email, webhook, or push alerts can be added later if async publish operations become long-running enough to warrant them.
- A Go-versus-Rust implementation spike was discussed but not selected; Rust is the Phase 5 direction.

</deferred>

---

*Phase: 5-Static Runtime Migration Foundation*
*Context gathered: 2026-05-28*
