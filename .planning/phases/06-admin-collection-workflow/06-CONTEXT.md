# Phase 6: Admin Collection Workflow - Context

**Gathered:** 2026-06-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 6 turns the completed Rust/static foundation into a polished, single-admin collection management workflow. It delivers the daily admin experience for creating and editing autograph items, uploading multiple private images, marking a primary image, saving private source-of-truth changes, publishing batched changes into static public output, reviewing field-level edit history, and keeping image metadata plus Object Storage objects in sync during normal operation.

This phase does not add public accounts, multiple admin roles, bulk import, AI-assisted metadata suggestions, or a public multi-service architecture. Phase 7 owns advisory AI-assisted ingest after the manual admin workflow exists.

</domain>

<decisions>
## Implementation Decisions

### Daily Admin Flow
- **D-06-01:** After login, the admin experience should start with a landing/status hub rather than directly in an editor.
- **D-06-02:** The landing/status hub should show site/controller/publish health and recent publish state, then offer two clear paths: add new item / enter backlog data as the high-priority path, and find or modify existing items as the secondary path.
- **D-06-03:** The admin workflow should optimize for a large initial backlog entry push, then low-frequency ongoing additions a few times per year.
- **D-06-04:** Existing-item maintenance should be available and competent, but it should not dominate the product.

### Image Management
- **D-06-05:** Keep multi-image management deliberately simple. The admin can upload multiple images for an item and mark one image as primary.
- **D-06-06:** Do not add drag/reorder, rich review queues, captions, or elaborate gallery management unless implementation discovers a hard need.
- **D-06-07:** Image ordering may stay implementation-defined or default to upload order, with the primary image displayed first in public output.
- **D-06-08:** Removing or replacing images still matters for keeping metadata and Object Storage in sync, but the admin UX should not become a digital asset management system.

### Edit History
- **D-06-09:** Use field-level diffs for edit history. Meaningful item metadata changes should record and display before/after values in the admin UI.
- **D-06-10:** Image and publication changes should be represented clearly, either as structured diff entries or closely related history events.
- **D-06-11:** Edit history is for the single collection owner, not an enterprise audit system. No multi-user attribution, role model, or public audit surface is required.

### Save, Publish, and Cleanup
- **D-06-12:** Saving and publishing should remain separate operations. Saving updates the private Oracle/Object Storage source of truth only.
- **D-06-13:** Publishing is an explicit operation that can batch multiple saved, new, or edited items into one static release.
- **D-06-14:** The landing/status hub should show pending unpublished changes so backlog entry can work as: enter several items, then publish once.
- **D-06-15:** Normal publish should use incremental publish. Full rebuild remains an explicit repair or structural-change action.
- **D-06-16:** Publish should keep public artifacts consistent, and admin image deletion/replacement should avoid orphaned metadata or Object Storage objects in normal operation.
- **D-06-17:** Cleanup should be cautious and observable rather than silent magic.
- **D-06-18:** Static release retention must become more judicious in Phase 6. The current backlog of preserved generated releases is acceptable at tiny catalog size but will waste filesystem space as the collection grows. Planning should define a retention/pruning policy for promoted and failed releases that preserves enough rollback/debug value without accumulating unbounded release directories.

### Admin Security and Diagnostics
- **D-06-19:** Session and lockout clarity are mandatory: logged-in/logged-out state, logout, expired-session handling, understandable lockout/rate-limit errors, preserved secure cookie/CSRF/origin behavior, and updated admin access docs.
- **D-06-20:** Include a small operator diagnostics/status panel as part of the landing hub. It should show safe/redacted provider modes, controller health, last publish result, pending unpublished changes, and cleanup or live-smoke guidance.
- **D-06-21:** The diagnostics panel should reduce routine SSH/container-log dives without exposing secrets or internal storage details.
- **D-06-22:** A broad standalone admin audit trail is not the priority, but item edit history, image events, and publish events should be recorded enough to support field diffs and operational troubleshooting.

### Carried Forward From Earlier Phases
- **D-06-23:** Use the existing Rust controller, static admin shell, Caddy `/admin` and `/admin/api/*` route shape, Oracle catalog source of truth, private OCI Object Storage originals, and static publisher foundation.
- **D-06-24:** Keep public output generated inside the OCI/runtime boundary. GitHub-hosted workflows must not read catalog data, Object Storage object identifiers, Oracle content, image UUIDs, or private media.
- **D-06-25:** Public static output must remain published-only and free of Object Storage URLs, private object keys, bucket names, namespaces, Oracle internals, image UUIDs, credentials, or unpublished records.
- **D-06-26:** The project remains single-admin and personal-collection scoped. Do not introduce public user accounts, multi-admin roles, social features, bulk import, or AI metadata suggestions in Phase 6.

### the agent's Discretion
- Exact admin layout, component structure, and static HTML/CSS/JavaScript organization, provided the first-screen status hub and two-path admin flow are preserved.
- Exact field-diff storage schema and rendering details, provided meaningful before/after values are persisted and visible to the admin.
- Exact metadata field grouping and form ergonomics, provided backlog entry remains efficient and the UI fits the existing quiet, work-focused admin direction.
- Exact cleanup confirmation and retry mechanics, provided normal image edits do not leave orphaned metadata or Object Storage objects and failures are observable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope and Requirements
- `.planning/ROADMAP.md` - Phase 6 goal, requirements, dependencies, and success criteria.
- `.planning/REQUIREMENTS.md` - Phase 6 requirement IDs: `DATA-03`, `MEDIA-04`, `ADMIN-01`, `ADMIN-02`, `ADMIN-03`, `ADMIN-04`, and `ADMIN-05`.
- `.planning/PROJECT.md` - Current product constraints, Phase 5 closeout state, key decisions, and out-of-scope boundaries.
- `.planning/STATE.md` - Current project state, Phase 5 proof outcome, and Phase 6 planning concerns.

### Static Runtime Foundation
- `.planning/phases/05-static-runtime-migration-foundation/05-CONTEXT.md` - Phase 5 decisions for the Rust controller, admin shell, publisher, media naming, and static runtime boundary.
- `.planning/phases/05-static-runtime-migration-foundation/05-07-SUMMARY.md` - Live static publish proof and closeout evidence that Phase 6 builds on.
- `docs/static-runtime-runbook.md` - Operator live-smoke and static publishing runbook.
- `docs/controller-walkthrough.md` - Existing controller/admin/publisher walkthrough.
- `docs/deployment-runbook.md` - Runtime deployment, Caddy/controller routing, provider modes, and live-smoke guidance.

### Codebase Maps
- `.planning/codebase/STACK.md` - Current Rust/static runtime stack and validation expectations.
- `.planning/codebase/ARCHITECTURE.md` - Current static-public/Rust-controller architecture and Phase 6 boundary.
- `.planning/codebase/CONVENTIONS.md` - Rust/static/admin conventions, testing habits, and documentation habits.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `controller/static-admin/index.html`, `controller/static-admin/admin.js`, and `controller/static-admin/admin.css` provide the Phase 5 static admin shell, same-origin fetch pattern, login/logout, item create/update, image upload, publication toggles, publish buttons, and status rendering.
- `controller/src/routes.rs` already exposes `/admin/api/health`, login/logout, item create/update, image upload, publication status updates, incremental/full publish, and publish status routes.
- `controller/src/catalog.rs` and `controller/src/oracle_catalog.rs` already model autograph items, images, publication status, Oracle persistence, image insert behavior, and primary-image handling.
- `controller/src/publisher.rs` already generates public static artifacts, orders primary image first, validates privacy, promotes releases, and supports incremental/full publish modes.
- `controller/tests/seed_content.rs`, `controller/tests/publisher.rs`, `controller/tests/static_admin.rs`, and live smoke tests provide existing verification patterns for admin API behavior, publication, image handling, static privacy, and live Oracle/Object Storage paths.

### Established Patterns
- Browser admin calls should stay same-origin under `/admin/api/*` with credentials included.
- The admin surface should remain private and simple, using static HTML/CSS/JavaScript unless planning finds a strong reason to introduce heavier frontend tooling.
- Controller routes should avoid leaking internal OCI, Oracle, filesystem, or secret details in responses.
- Public static generation should fail closed during validation rather than publish incomplete or privacy-leaking artifacts.
- Live Oracle/Object Storage verification remains operator-run because it needs real credentials and tenancy state.

### Integration Points
- The landing/status hub connects to existing health and publish-status endpoints and may need new redacted status fields for pending unpublished changes and cleanup warnings.
- Item creation/editing extends the current `/admin/api/items` and `/admin/api/items/{id}` routes from minimal seed behavior into daily workflow behavior.
- Image removal/replacement likely needs new controller-owned API paths that coordinate Oracle image metadata and OCI Object Storage deletion.
- Edit history likely needs schema additions in `controller/db/schema.sql`, Oracle adapter changes, route/service behavior changes, and admin UI rendering.
- Publish batching can build on existing explicit incremental publish semantics; save operations should not automatically publish.
- Static release promotion currently preserves multiple release directories. Phase 6 should add retention/pruning behavior for generated releases before catalog growth makes disk usage more significant.

</code_context>

<specifics>
## Specific Ideas

- The admin landing page should feel like a status hub with clear choices: "Add new item" for backlog/new acquisitions and "Find or modify existing items" for occasional maintenance.
- The workflow should support entering several backlog items before pressing a single publish action.
- The admin UI should show pending unpublished changes so the operator knows when private saves have not reached the public static site.
- The admin/publisher should avoid unbounded release accumulation; preserve only a deliberate number or window of prior releases plus the latest failed candidate/logs needed for debugging.
- Multi-image management should be limited to upload multiple images and mark one primary image.
- Field-level edit history should be visible enough to answer what changed on an item over time.
- Operator diagnostics should be safe and redacted: provider modes, controller health, last publish result, pending changes, and cleanup/live-smoke guidance.

</specifics>

<deferred>
## Deferred Ideas

- Advisory OCR/AI metadata suggestions remain Phase 7 scope.
- Public accounts, multi-admin roles, social features, bulk import, and a public multi-service split remain out of scope.
- Rich image management features such as drag/reorder, captions, dedicated review queues, and full digital asset management are deferred unless future use proves they are needed.
- A broad standalone admin audit trail beyond item edit history, image events, publish events, and minimal security/session clarity is not a Phase 6 priority.

</deferred>

---

*Phase: 6-Admin Collection Workflow*
*Context gathered: 2026-06-20*
