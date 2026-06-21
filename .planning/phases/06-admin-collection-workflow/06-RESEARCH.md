# Phase 6: Admin Collection Workflow - Research

**Researched:** 2026-06-20
**Domain:** Rust controller admin workflow, Oracle edit history, OCI media cleanup, static publishing, private admin security
**Confidence:** HIGH for codebase-bound planning; MEDIUM for official-doc patterns; LOW for unavailable Context7 cache provider.

<user_constraints>
## User Constraints (from CONTEXT.md)

Copied verbatim from `.planning/phases/06-admin-collection-workflow/06-CONTEXT.md`. [VERIFIED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md]

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)

- Advisory OCR/AI metadata suggestions remain Phase 7 scope.
- Public accounts, multi-admin roles, social features, bulk import, and a public multi-service split remain out of scope.
- Rich image management features such as drag/reorder, captions, dedicated review queues, and full digital asset management are deferred unless future use proves they are needed.
- A broad standalone admin audit trail beyond item edit history, image events, publish events, and minimal security/session clarity is not a Phase 6 priority.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DATA-03 | Application records edit history for autograph items so the admin can see what changed over time in v1. [VERIFIED: .planning/REQUIREMENTS.md] | Add controller-owned edit event persistence with field diffs, image events, publication events, item-history read API, and admin timeline rendering. [VERIFIED: controller/src/catalog.rs] |
| MEDIA-04 | Application keeps image objects and metadata references in sync so uploads and edits do not leave orphaned records or orphaned files in normal operation. [VERIFIED: .planning/REQUIREMENTS.md] | Extend current upload rollback behavior into delete/replace/primary operations, using `PrivateMediaStore::delete` plus repository metadata transitions. [VERIFIED: controller/src/media.rs] [VERIFIED: controller/tests/seed_content.rs] |
| ADMIN-01 | Exactly one admin authentication path exists for collection management, and no public user account system is required for v1. [VERIFIED: .planning/REQUIREMENTS.md] | Preserve cookie-based browser admin path and remove bearer-token authorization from collection-management routes; any remaining token compatibility must be non-management only. [VERIFIED: controller/src/auth.rs] [VERIFIED: docs/configuration-contract.md] |
| ADMIN-02 | Admin can create a new autograph item by uploading images and reviewing/editing metadata in one workflow before publish. [VERIFIED: .planning/REQUIREMENTS.md] | Replace Phase 5 seed shell with status hub plus create form; keep save separate from publish. [VERIFIED: controller/static-admin/admin.js] |
| ADMIN-03 | Admin can edit an existing autograph item, including metadata and associated images. [VERIFIED: .planning/REQUIREMENTS.md] | Add list/search/get routes, edit form hydration, image delete/replace/primary APIs, and history display. [VERIFIED: controller/src/routes.rs] |
| ADMIN-04 | Admin can save reviewed metadata and publish the item so it becomes visible in the public gallery. [VERIFIED: .planning/REQUIREMENTS.md] | Build on existing publication update and incremental publish endpoints, adding pending unpublished change status. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/publisher.rs] |
| ADMIN-05 | Admin routes, secrets, edit-history behavior, media cleanup, and operator-bridge retirement are reviewed for security and documented before the admin workflow is considered complete. [VERIFIED: .planning/REQUIREMENTS.md] | Include final security review, docs update, route/secret grep, Caddy guard checks, and live smoke guidance in the plan. [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- Use the completed Rust/static foundation; do not re-scaffold the retired Next.js app or infrastructure. [VERIFIED: AGENTS.md]
- Keep public static artifacts free of private storage identifiers and unpublished records. [VERIFIED: AGENTS.md]
- Keep persistence/media details in controller adapters and services, not scattered through route handlers or static assets. [VERIFIED: AGENTS.md]
- Use plain static HTML/CSS/JavaScript for admin/public surfaces unless a later phase intentionally changes that constraint. [VERIFIED: AGENTS.md]
- Do not introduce public accounts, multi-admin roles, direct Object Storage URLs, or a split frontend/backend service architecture for v1. [VERIFIED: AGENTS.md]
- Run Rust checks for current runtime code: `cargo fmt`, `cargo test`, `cargo check --features production-persistence`, and `cargo clippy`. [VERIFIED: AGENTS.md]
- Keep static contract/privacy tests mandatory for public artifact changes. [VERIFIED: AGENTS.md]
- Run Ansible syntax/lint checks for deployment, cleanup, and security patching changes. [VERIFIED: AGENTS.md]
- Live Oracle/Object Storage verification remains operator-run because it needs real credentials and tenancy state. [VERIFIED: docs/static-runtime-runbook.md]
- Never commit directly to `main` or `master`; current branch is `gsd/phase-06-context`. [VERIFIED: git status --short --branch]

## Summary

Phase 6 should be planned as an extension of the existing Rust private controller, static admin shell, Oracle catalog adapter, OCI media adapter, and static publisher. The repository already has authenticated admin routes, item create/update, image upload, publication toggles, incremental/full publish, redacted health/status, privacy validation, and Caddy routing for `/admin` and `/admin/api/*`. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/publisher.rs] [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile]

The largest missing planning surfaces are not framework selection; they are domain behavior: edit-history persistence and rendering, item list/get/search routes for editing, multi-image delete/replace/primary management, cautious media cleanup with observable failures, pending unpublished change status, static release retention/pruning, and final security/docs closure. [VERIFIED: controller/src/catalog.rs] [VERIFIED: controller/static-admin/index.html] [VERIFIED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md]

**Primary recommendation:** Plan Phase 6 as focused waves: repository/schema/history foundation; admin list/create/edit workflow; media maintenance/cleanup; pending-change publish/status/retention; static admin UI; auth hardening; security/docs/operator-bridge retirement; and codebase-map closeout. [VERIFIED: .planning/ROADMAP.md] [ASSUMED]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Admin authentication/session/CSRF | Rust controller/API | Caddy/static admin | Auth state, secure cookies, and mutation checks are implemented in controller routes; Caddy only routes private admin traffic. [VERIFIED: controller/src/auth.rs] [VERIFIED: controller/src/routes.rs] |
| Admin landing/status hub | Static admin browser | Rust controller/API | Browser renders the hub; controller supplies redacted health, publish, pending-change, and diagnostics data. [VERIFIED: controller/static-admin/admin.js] [VERIFIED: controller/src/routes.rs] |
| Item create/edit metadata | Rust controller/API | Oracle catalog adapter, static admin browser | Controller owns validation/auth and delegates persistence; browser should stay a thin same-origin client. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/oracle_catalog.rs] |
| Field-level edit history | Oracle catalog adapter | Rust controller/API, static admin browser | History must persist with the source of truth and be queryable for admin rendering. [VERIFIED: controller/db/schema.sql] [VERIFIED: controller/src/catalog.rs] |
| Multi-image upload/delete/primary | Rust controller/API | OCI media adapter, Oracle catalog adapter | The controller must coordinate object writes/deletes and metadata updates so normal edits do not orphan data. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/media.rs] |
| Static publish batching | Rust publisher | Static admin browser, Caddy static runtime | Existing publisher creates, validates, and promotes releases; browser triggers publish and displays status. [VERIFIED: controller/src/publisher.rs] |
| Release retention/pruning | Rust publisher/runtime filesystem | Operator docs | Release directories live under the runtime static root, and current code retains latest failed candidate only but not promoted release count. [VERIFIED: controller/src/publisher.rs] [VERIFIED: docs/configuration-contract.md] |
| Public privacy boundary | Rust publisher | Caddy static runtime | Publisher writes public-safe artifacts and validates privacy before promotion; Caddy serves only generated `current`. [VERIFIED: controller/src/publisher.rs] [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] |

## Standard Stack

### Core

| Library/Tool | Version | Purpose | Why Standard |
|--------------|---------|---------|--------------|
| Rust / Cargo | rustc 1.96.0, cargo 1.96.0 | Controller, publisher, adapters, tests | Existing active implementation language and local toolchain. [VERIFIED: rustc --version] [VERIFIED: cargo --version] |
| `axum` | 0.8.9 | HTTP routes, extractors, multipart upload, JSON/admin API | Existing controller framework; docs support Router routes, extractors, JSON, and typed shared state. [VERIFIED: controller/Cargo.toml] [CITED: https://docs.rs/axum/0.8.9/axum/] |
| `tokio` | 1.52.3 | Async runtime and tests | Existing async runtime paired with Axum. [VERIFIED: controller/Cargo.toml] |
| `serde` / `serde_json` | 1.0.228 / 1.0.150 | JSON API DTOs, public contracts, history payloads | Existing DTO and contract serialization stack. [VERIFIED: controller/Cargo.toml] |
| `oracle` | 0.6.3 | Oracle Autonomous Database adapter | Existing production persistence feature uses native Oracle crate. [VERIFIED: controller/Cargo.toml] [VERIFIED: controller/src/oracle_catalog.rs] |
| `image` | 0.25.10 | Upload validation and WebP derivative generation | Existing upload validation and publisher derivative stack. [VERIFIED: controller/Cargo.toml] [VERIFIED: controller/src/derivatives.rs] |
| OCI instance-principal media adapter | In-repo native adapter | Private Object Storage reads/writes/deletes | Existing runtime media path avoids long-lived Object Storage customer secrets. [VERIFIED: controller/src/oci_media.rs] [VERIFIED: .planning/STATE.md] |
| Caddy | Deployed container config | Static public serving and `/admin/api/*` proxy | Existing route boundary for generated public output and private admin controller. [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] |

### Supporting

| Library/Tool | Version | Purpose | When to Use |
|--------------|---------|---------|-------------|
| `uuid` | 1.23.3 | Item/image/release/event IDs | Use for new edit-history, cleanup-event, and release IDs. [VERIFIED: controller/Cargo.toml] |
| `time` | 0.3.49 | Publish timestamps and history timestamps | Use for event `created_at`/publish status times. [VERIFIED: controller/Cargo.toml] |
| `argon2` | 0.5.3 | Admin password hash verification | Preserve for the single-admin browser login path. [VERIFIED: controller/Cargo.toml] [VERIFIED: controller/src/auth.rs] |
| Terraform | 1.15.6 | Infra config validation if Phase 6 changes env/retention variables | Use if adding deploy/runtime env variables. [VERIFIED: terraform version] |
| Ansible / ansible-lint | core 2.19.0 / 25.6.1+really25.2.1 | Runtime env/Caddy/deploy docs validation | Use documented `/tmp` temp vars in this sandbox. [VERIFIED: ansible-playbook --version] [VERIFIED: ansible-lint --version] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Plain static admin JS | React/Vite/admin SPA | Adds a frontend build/runtime surface that contradicts the current v1 simplicity constraint unless Phase 6 discovers a hard need. [VERIFIED: AGENTS.md] |
| Controller-owned field-diff events | Database triggers for every table | Triggers could record changes close to Oracle, but current repository pattern keeps domain decisions in Rust adapters and services. [VERIFIED: controller/src/oracle_catalog.rs] [ASSUMED] |
| Explicit Object Storage deletes on image edit | Lifecycle-only cleanup | OCI lifecycle policies are delayed and best-effort; normal admin edits need synchronous, observable cleanup. [CITED: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usinglifecyclepolicies.htm] |

**Installation:** No new external packages are recommended for Phase 6 planning. [VERIFIED: controller/Cargo.toml]

## Package Legitimacy Audit

No new package installation is recommended by this research, so the package legitimacy gate is not required for Phase 6 planning. If implementation adds a new crate or frontend dependency, the planner must add a package-legitimacy checkpoint before installation. [VERIFIED: controller/Cargo.toml] [ASSUMED]

## Architecture Patterns

### System Architecture Diagram

```text
Admin browser /admin
  -> same-origin fetch /admin/api/*
  -> Rust controller auth + CSRF + redacted DTOs
  -> CatalogRepository trait
      -> Memory repository for local tests
      -> Oracle repository for production
  -> PrivateMediaStore trait
      -> Local media for tests
      -> OCI instance-principal Object Storage for production
  -> LocalPublisher
      -> generate candidate static release
      -> validate privacy and completeness
      -> promote current symlink
      -> prune retained releases
  -> Caddy serves /srv/autographs/static/current to anonymous public users
```

This data flow is the current Phase 5 foundation with Phase 6 additions inserted at controller/repository/publisher boundaries. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/catalog.rs] [VERIFIED: controller/src/publisher.rs] [VERIFIED: docs/deployment-runbook.md]

### Recommended Project Structure

```text
controller/src/
├── catalog.rs          # trait/domain types; add history/image operation contracts
├── oracle_catalog.rs   # Oracle persistence for items, images, edit events, pending changes
├── media.rs            # private media trait; delete already exists
├── routes.rs           # admin API handlers and redacted DTOs
├── publisher.rs        # publish status, candidate validation, release pruning
└── auth.rs             # single-admin session/login behavior

controller/static-admin/
├── index.html          # status hub, create/edit shell
├── admin.js            # same-origin admin client
└── admin.css           # quiet work-focused admin layout

controller/tests/
├── admin_workflow.rs   # new create/edit/list/history browser/API contracts
├── media_cleanup.rs    # new delete/replace/primary cleanup contracts
└── publisher.rs        # extend pending changes and retention coverage
```

The structure mirrors existing module ownership and test placement. [VERIFIED: rg --files controller]

### Pattern 1: Thin Route, Repository-Owned Domain Mutation

**What:** Routes authenticate, parse DTOs, call repository/media/publisher services, and return redacted DTOs. [VERIFIED: controller/src/routes.rs]

**When to use:** Use for item create/update, history fetch, image delete/replace/primary, pending status, and publish actions. [VERIFIED: controller/src/routes.rs]

**Example:**

```rust
// Source: existing controller route shape, adapted for Phase 6.
async fn item_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if authenticate(&state, &headers).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Ok(item_id) = Uuid::parse_str(&id) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match state.repository.history(item_id).await {
        Ok(events) => Json(HistoryResponse::from(events)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
```

This pattern is consistent with existing Axum handlers and shared state. [VERIFIED: controller/src/routes.rs] [CITED: https://docs.rs/axum/0.8.9/axum/]

### Pattern 2: Field-Diff History as Append-Only Events

**What:** Add append-only edit events keyed by item ID, event type, and timestamp; store field diffs as structured JSON/CLOB payloads plus relational columns for common filters. [VERIFIED: controller/db/schema.sql] [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/adjsn/json-data-type.html]

**Recommended schema shape:**

```sql
create table autograph_item_events (
  id varchar2(36) primary key,
  item_id varchar2(36) not null,
  event_type varchar2(32) not null,
  diff_json clob,
  summary varchar2(512),
  created_at timestamp default current_timestamp not null,
  constraint autograph_item_events_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade,
  constraint autograph_item_events_type_ck
    check (event_type in ('created', 'metadata', 'image', 'publication', 'cleanup', 'publish'))
);

create index autograph_item_events_item_idx
  on autograph_item_events(item_id, created_at);
```

Use JSON only for the variable diff payload; keep `item_id`, `event_type`, and `created_at` relational for simple admin queries. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/adjsn/creating-b-tree-indexes-json_value.html] [ASSUMED]

### Pattern 3: Explicit Media Cleanup with Compensating State

**What:** For normal delete/replace, update metadata and delete Object Storage objects in a controller-owned flow; record cleanup events and expose failures on the status hub. [VERIFIED: controller/src/media.rs] [VERIFIED: controller/tests/seed_content.rs]

**When to use:** Use whenever an image metadata row is removed, primary image changes, or replacement upload supersedes a private original. [VERIFIED: controller/src/catalog.rs]

**Important sequencing:** For replacement, upload new object first, attach metadata, switch primary if requested, then delete old object and record a cleanup event. If final delete fails, keep an observable cleanup warning rather than silently hiding it. [VERIFIED: controller/src/routes.rs] [CITED: https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/os/object/delete.html] [ASSUMED]

### Pattern 4: Pending Changes as First-Class Status

**What:** Track saved changes after the last successful publish and show them on the landing hub. [VERIFIED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md]

**Implementation direction:** Store `published_at` or last successful release timestamp and compare against item/image/event `updated_at` or append-only events. This should feed a redacted `/admin/api/status` or expanded publish status response. [VERIFIED: controller/db/schema.sql] [ASSUMED]

### Pattern 5: Release Retention as Publisher Responsibility

**What:** Add a deterministic count-based pruning policy after successful publish: retain 5 promoted releases by default, counting the active `current` target as one retained release, and retain 1 failed candidate by default. Expose both counts through runtime env vars. [VERIFIED: controller/src/publisher.rs] [ASSUMED]

**Why:** Current code retains only the latest failed candidate but does not prune successful `releases/` directories. [VERIFIED: controller/src/publisher.rs]

### Anti-Patterns to Avoid

- **Reintroducing a public app/API runtime:** The retired Next.js/API path is out of scope and blocked by project constraints. [VERIFIED: AGENTS.md] [VERIFIED: docs/deployment-runbook.md]
- **Treating edit history as logging only:** Runtime logs are not queryable admin history and are not durable item metadata. [VERIFIED: docs/controller-walkthrough.md] [ASSUMED]
- **Deleting metadata before object cleanup without recording failure:** This can make orphaned Object Storage keys hard to discover later. [VERIFIED: controller/tests/live_persistence_smoke.rs] [ASSUMED]
- **Making lifecycle policies the normal cleanup path:** OCI lifecycle policies can delete objects, but they are delayed and best-effort. [CITED: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usinglifecyclepolicies.htm]
- **Leaking original filenames or object keys in admin/public DTOs:** Existing tests assert redaction and source privacy; Phase 6 must extend them. [VERIFIED: controller/tests/seed_content.rs] [VERIFIED: controller/tests/static_admin.rs]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP routing/extraction | Custom request parser | Axum Router, extractors, Json, Multipart | Existing stack already provides typed route extraction and response conversion. [VERIFIED: controller/src/routes.rs] [CITED: https://docs.rs/axum/0.8.9/axum/] |
| Password hashing | Custom hash or reversible secret | Existing Argon2 verifier | Existing auth code already supports Argon2 password hashes. [VERIFIED: controller/src/auth.rs] |
| Image validation/derivatives | Custom byte sniffing or image transforms | Existing `image` crate path | Current upload validation and derivative generation already use `image`. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/derivatives.rs] |
| Object Storage signing | New SDK wrapper from scratch | Existing in-repo OCI instance-principal adapter | Phase 5 proved this adapter live; do not revive S3 Customer Secret path. [VERIFIED: .planning/STATE.md] [VERIFIED: controller/src/oci_media.rs] |
| Public artifact publishing | Ad hoc file writes from routes | Existing `LocalPublisher` validation/promote path | Publisher already validates completeness/privacy and promotes atomically. [VERIFIED: controller/src/publisher.rs] |
| Admin persistence state | Browser storage | Controller/Oracle source of truth | Existing static admin tests forbid local/session storage. [VERIFIED: controller/tests/static_admin.rs] |

**Key insight:** Phase 6 complexity is coordination and observability, not missing libraries. Use existing primitives and add durable domain events around them. [VERIFIED: controller/src/catalog.rs] [ASSUMED]

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | Oracle has `autograph_items`, `autograph_item_tags`, `autograph_images`, `autograph_publish_jobs`, and `autograph_public_derivatives`; no edit-history table exists. [VERIFIED: controller/db/schema.sql] | Add schema end-state table(s), repository methods, and live schema preflight updates. [VERIFIED: controller/src/oracle_schema.rs] |
| Stored data | Private Object Storage originals use keys shaped `originals/{item-uuid}/{image-uuid}`. [VERIFIED: controller/src/storage_keys.rs] | Add normal image delete/replace cleanup paths and smoke cleanup coverage. [VERIFIED: controller/tests/live_static_publish_smoke.rs] |
| Live service config | Caddy serves `/admin/*`, proxies `/admin/api/*`, and blocks `/api/operator/*` with 404. [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] | Keep route shape; final audit should verify retired operator route docs are gone or clearly historical. [VERIFIED: docs/deployment-runbook.md] |
| OS-registered state | Runtime VM uses systemd-managed Podman quadlets for controller/Caddy and shared static volume. [VERIFIED: docs/deployment-runbook.md] | If env vars are added for retention/status, update Ansible templates and restart handlers. [VERIFIED: deploy/ansible/roles/autographs_deploy/templates/app.env.j2] |
| Secrets/env vars | `AUTOGRAPHS_ADMIN_PASSWORD_HASH`, optional local `AUTOGRAPHS_ADMIN_PASSWORD`, and `AUTOGRAPHS_OPERATOR_API_TOKEN` exist. [VERIFIED: docs/configuration-contract.md] | Phase 6 must make the session cookie from `/admin/api/login` the only collection-management auth path and remove bearer-token collection-management docs/tests. [VERIFIED: .planning/REQUIREMENTS.md] |
| Secrets/env vars | Oracle wallet/password and OCI media namespace/bucket are deployed through environment/operator secret stores. [VERIFIED: docs/configuration-contract.md] | Do not expose these in diagnostics, DTOs, static admin source, or public artifacts. [VERIFIED: controller/tests/static_admin.rs] |
| Build artifacts | Static releases accumulate under `${AUTOGRAPHS_STATIC_RELEASE_ROOT}/releases/`; failed candidates retain only latest under `failed/`. [VERIFIED: controller/src/publisher.rs] | Add promoted-release retention/pruning policy and status visibility. [VERIFIED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md] |

**Nothing found in category:** No project skill `SKILL.md` files were found under `.codex/skills` or `.agents/skills`. [VERIFIED: find .codex/skills .agents/skills -maxdepth 3 -name SKILL.md]

## Common Pitfalls

### Pitfall 1: Nullable Update DTO Cannot Clear Optional Fields

**What goes wrong:** Current `AutographItemUpdate` uses `Option<String>` for optional fields, so absent fields and explicit clears are indistinguishable. [VERIFIED: controller/src/catalog.rs]

**Why it happens:** `None` means "do not update" in the existing apply-update logic. [VERIFIED: controller/src/oracle_catalog.rs]

**How to avoid:** Plan a patch/field representation that can express unchanged, set, and clear before building polished edit forms. [ASSUMED]

**Warning signs:** Admin saves cannot remove description/source/certification values once set. [ASSUMED]

### Pitfall 2: Upload Is Primary by Default

**What goes wrong:** Current upload sets every uploaded image as primary, clearing previous primary in Oracle. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/oracle_catalog.rs]

**Why it happens:** Phase 5 seed shell optimized for one image. [VERIFIED: controller/static-admin/index.html]

**How to avoid:** Add explicit `isPrimary` handling and a separate primary-selection API. [ASSUMED]

**Warning signs:** Uploading a supporting image unexpectedly changes the public gallery primary image. [VERIFIED: controller/tests/publisher.rs]

### Pitfall 3: Pending Changes Are Not Persisted Yet

**What goes wrong:** Publish status is in-memory and shows only current/last publisher run, not durable pending saved changes. [VERIFIED: controller/src/publisher.rs]

**Why it happens:** Phase 5 had no persisted change events. [VERIFIED: controller/src/publisher.rs]

**How to avoid:** Make item/image/publication events durable and compute pending status from events or timestamps relative to last successful publish. [ASSUMED]

**Warning signs:** The hub says publish is idle even after private edits that are not in the public static release. [ASSUMED]

### Pitfall 4: Object Cleanup Failure Becomes Invisible

**What goes wrong:** If metadata changes succeed but Object Storage delete fails, orphaned files can remain. [VERIFIED: controller/src/media.rs] [ASSUMED]

**Why it happens:** Distributed cleanup spans Oracle and OCI; there is no shared transaction across both systems. [ASSUMED]

**How to avoid:** Record cleanup event status, retry guidance, and safe diagnostics; use live smoke cleanup patterns for verification. [VERIFIED: controller/tests/live_persistence_smoke.rs]

**Warning signs:** Metadata image count is zero but Object Storage still lists `originals/{item-id}/...`. [VERIFIED: docs/static-runtime-runbook.md]

### Pitfall 5: Admin Diagnostics Leak Internals

**What goes wrong:** Status panels can accidentally expose bucket names, object keys, namespaces, original filenames, or secret values. [VERIFIED: controller/tests/static_admin.rs]

**Why it happens:** Diagnostics often reuse internal status structs without redaction. [ASSUMED]

**How to avoid:** Create dedicated redacted DTOs and extend privacy tests for every new endpoint/source file. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/tests/static_admin.rs]

**Warning signs:** Static admin source or JSON responses contain `objectKey`, `bucketName`, `OCI_`, or original filenames. [VERIFIED: controller/tests/static_admin.rs]

## Code Examples

### Field Diff Payload

```json
{
  "fields": [
    { "name": "signer", "before": "Old Name", "after": "New Name" },
    { "name": "tags", "before": ["cards"], "after": ["cards", "star-wars"] }
  ]
}
```

This JSON shape is a recommended planning contract, not an existing implementation. [ASSUMED]

### Cleanup Event Payload

```json
{
  "imageId": "uuid",
  "operation": "delete",
  "objectDeleted": true,
  "metadataDeleted": true,
  "retryable": false
}
```

Keep object keys out of admin-visible payloads; store private keys only in the repository/media layer as needed for retry operations. [VERIFIED: controller/tests/static_admin.rs] [ASSUMED]

### Redacted Status DTO Fields

```json
{
  "providers": { "database": "oracle", "media": "oci-instance-principal" },
  "publish": { "state": "succeeded", "releaseId": "uuid" },
  "pendingChanges": { "count": 3, "oldestEpochSeconds": 1780000000 },
  "cleanup": { "warnings": 0 }
}
```

This extends current health/publish status without exposing storage identifiers. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/tests/auth_and_health.rs] [ASSUMED]

## Likely Plan Breakdown

| Plan | Scope | Sequencing Risk |
|------|-------|-----------------|
| 06-01 | Schema/repository foundation for edit events, pending changes, item list/get, and update semantics that can clear optional fields. [VERIFIED: controller/src/catalog.rs] | Must land before UI and publish status can be meaningful. [ASSUMED] |
| 06-02 | Admin landing/status hub and item finder/list path. [VERIFIED: controller/static-admin/index.html] | Hub needs pending-change/status endpoints from 06-01. [ASSUMED] |
| 06-03 | Polished create/edit form with multi-image upload and primary selection. [VERIFIED: controller/static-admin/admin.js] | Must preserve same-origin cookie/CSRF behavior. [VERIFIED: controller/tests/auth_and_health.rs] |
| 06-04 | Image delete/replace cleanup and observable cleanup warnings/retries. [VERIFIED: controller/src/media.rs] | Distributed Oracle/OCI cleanup needs rollback/compensation tests. [ASSUMED] |
| 06-05 | Publish batching, pending changes, release retention/pruning, and diagnostics panel. [VERIFIED: controller/src/publisher.rs] | Retention must not delete active `current` target. [VERIFIED: controller/src/publisher.rs] |
| 06-06 | Security/docs closeout: admin auth path, operator-bridge retirement, route/secret/privacy audit, runbook updates, live smoke guidance. [VERIFIED: .planning/ROADMAP.md] | Requires all new surfaces to exist before final review. [ASSUMED] |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Public Next.js runtime with app-mediated image streaming | Caddy-served generated static output plus Rust private controller | Phase 5 closeout on 2026-06-20 [VERIFIED: .planning/STATE.md] | Phase 6 must not revive Next.js/API routes. [VERIFIED: AGENTS.md] |
| Temporary operator data entry endpoints | `/admin` static shell and `/admin/api/*` controller route shape | Phase 5 [VERIFIED: docs/deployment-runbook.md] | Phase 6 should retire or reframe temporary operator docs. [VERIFIED: docs/temporary-production-data-entry.md] |
| Minimal seed shell | Polished single-admin collection workflow | Phase 6 target [VERIFIED: .planning/ROADMAP.md] | UI work is in scope, but keep plain static assets. [VERIFIED: AGENTS.md] |
| Full static rebuild as repair | Incremental publish as normal path, full rebuild as explicit repair | D-06-15 [VERIFIED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md] | Pending-change tracking becomes important. [ASSUMED] |

**Deprecated/outdated:**
- `docs/temporary-production-data-entry.md` still documents retired `/api/operator/catalog` paths and should be archived, removed, or clearly marked historical during Phase 6. [VERIFIED: docs/temporary-production-data-entry.md] [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile]
- Historical `.planning/research/*` still describes Next.js stack research and should not guide Phase 6 implementation. [VERIFIED: .planning/codebase/STACK.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Phase 6 can be split into focused waves without over-fragmenting, including a separate closeout map refresh if checker scope limits require it. | Summary / Likely Plan Breakdown | Planner may choose a different granularity. |
| A2 | Controller-owned edit events are preferable to Oracle triggers. | Standard Stack / Architecture Patterns | If trigger-based audit is required later, repository design may shift. |
| A3 | CLOB/JSON diff payload plus relational query columns is enough for v1 history. | Architecture Patterns | If Oracle JSON type compatibility differs in the live ADB, schema may use CLOB with `is json` instead. |
| A4 | Cleanup should record retryable failures instead of attempting cross-system transactional semantics. | Common Pitfalls | Planner must design retry/repair ergonomics carefully. |
| A5 | Bearer-token compatibility, if any remains, must not authorize collection-management routes after Phase 6; session-cookie login is the only collection-management path. | Security Domain | Non-management token compatibility may still need explicit docs if retained. |

## Open Questions — RESOLVED

1. **[RESOLVED] Should the compatibility bearer token remain after Phase 6?**
   - What we know: It is currently documented as a compatibility/admin token and accepted by controller auth. [VERIFIED: docs/configuration-contract.md] [VERIFIED: controller/src/routes.rs]
   - Resolution: Phase 6 makes `/admin/api/login` plus the HTTP-only session cookie the only collection-management authentication path. Bearer-token authorization must be removed from item, image, cleanup, publication, and publish routes; ignored live-smoke tests must log in through the session path. If the code retains token parsing for non-management compatibility, docs must clearly state it is not a collection-management path. [RESOLVED: .planning/REQUIREMENTS.md] [RESOLVED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md]
   - Planning impact: Plan 06-06 hardens collection-management routes to `AuthKind::Session`, and Plan 06-07 removes bearer-token collection-management wording from operator docs. [RESOLVED: .planning/phases/06-admin-collection-workflow/06-06-PLAN.md] [RESOLVED: .planning/phases/06-admin-collection-workflow/06-07-PLAN.md]

2. **[RESOLVED] How much release rollback should be retained?**
   - What we know: Current code prunes failed candidates to one but does not prune promoted releases. [VERIFIED: controller/src/publisher.rs]
   - Resolution: Use count-based retention, not time-window retention, for Phase 6. Default `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT=5`, counting the active `current` target as one retained promoted release, and default `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT=1`. Both counts are configurable through controller runtime env vars and rendered by the Ansible controller env template. [RESOLVED: .planning/phases/06-admin-collection-workflow/06-CONTEXT.md]
   - Planning impact: Plan 06-04 implements publisher pruning, config parsing, `.env.example`, and the Ansible `controller.env.j2` entries for production runtime rendering. [RESOLVED: .planning/phases/06-admin-collection-workflow/06-04-PLAN.md]

3. **[RESOLVED] Should history capture create events for seed/imported records already present?**
   - What we know: Existing live/static smoke data can create items before edit history exists. [VERIFIED: controller/tests/live_static_publish_smoke.rs]
   - Resolution: Do not synthesize baseline create events for pre-existing records. Edit history is forward-only from the Phase 6 history schema deployment; the first post-Phase 6 edit to an existing record records real before/after values from the current row, and records with no events show the UI copy `No history recorded yet. Changes made after the Phase 6 history update will appear here.` [RESOLVED: .planning/phases/06-admin-collection-workflow/06-UI-SPEC.md]
   - Planning impact: Plan 06-01 creates the history table and event contracts without a backfill task, and Plan 06-05 renders the approved empty-history copy. [RESOLVED: .planning/phases/06-admin-collection-workflow/06-01-PLAN.md] [RESOLVED: .planning/phases/06-admin-collection-workflow/06-05-PLAN.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust | Controller implementation/tests | yes | rustc 1.96.0 | None needed. [VERIFIED: rustc --version] |
| Cargo | Controller implementation/tests | yes | cargo 1.96.0 | None needed. [VERIFIED: cargo --version] |
| Terraform | Infra/env validation if touched | yes | 1.15.6 | Use existing CI if not touched locally. [VERIFIED: terraform version] |
| Ansible | Deploy/runtime template validation | yes with `/tmp` temp vars | core 2.19.0 | Use documented `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote`. [VERIFIED: ansible-playbook --version] |
| ansible-lint | Deploy/runtime lint | yes with `/tmp` temp vars | 25.6.1+really25.2.1 | Use CI if local lint is unavailable. [VERIFIED: ansible-lint --version] |
| OCI CLI | Operator diagnostics/live object cleanup checks | yes | 3.79.0 | Runtime VM also installs OCI CLI for diagnostics. [VERIFIED: oci --version] [VERIFIED: docs/deployment-runbook.md] |
| curl | Local/live smoke HTTP checks | yes | 8.14.1 | None needed. [VERIFIED: curl --version] |
| Podman | Local container smoke | no | - | Use GitHub/VM runtime or non-container Cargo tests locally. [VERIFIED: command -v podman] |
| Oracle/OCI live credentials | Live persistence/static smoke | operator-only | - | Keep ignored smoke tests as operator gates. [VERIFIED: controller/tests/live_persistence_smoke.rs] [VERIFIED: controller/tests/live_static_publish_smoke.rs] |

**Missing dependencies with no fallback:**
- None for planning and local controller implementation. [VERIFIED: cargo test --manifest-path controller/Cargo.toml]

**Missing dependencies with fallback:**
- Podman is not installed locally; use Cargo tests locally and runtime/CI/VM paths for container validation. [VERIFIED: command -v podman] [VERIFIED: docs/deployment-runbook.md]

## Validation Notes

`workflow.nyquist_validation` is explicitly `false`, so the formal Validation Architecture section is skipped. [VERIFIED: .planning/config.json]

Existing local suite passed with `cargo test --manifest-path controller/Cargo.toml`; live persistence/static smoke tests are present but ignored unless compiled with `--features live-persistence` and supplied live credentials. [VERIFIED: cargo test --manifest-path controller/Cargo.toml] [VERIFIED: controller/tests/live_persistence_smoke.rs] [VERIFIED: controller/tests/live_static_publish_smoke.rs]

Recommended Phase 6 gates:
- `cargo fmt --manifest-path controller/Cargo.toml --check` for every Rust plan. [VERIFIED: docs/deployment-runbook.md]
- `cargo test --manifest-path controller/Cargo.toml` for every controller/static admin plan. [VERIFIED: cargo test --manifest-path controller/Cargo.toml]
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` for Oracle/OCI adapter changes. [VERIFIED: docs/deployment-runbook.md]
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` before phase closeout. [VERIFIED: docs/deployment-runbook.md]
- Ansible syntax/lint with `/tmp` temp vars if deploy templates or Caddy change. [VERIFIED: docs/deployment-runbook.md]
- Operator-run live static publish smoke after media cleanup, Oracle schema, or publisher behavior changes. [VERIFIED: docs/static-runtime-runbook.md]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not explicitly set `security_enforcement: false`. [VERIFIED: .planning/config.json]

OWASP ASVS 5.0.0 is the current stable ASVS version according to OWASP's project page. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | yes | Preserve single-admin login, Argon2 password hash, lockout clarity, logout, expired-session UI. [VERIFIED: controller/src/auth.rs] |
| V3 Session Management | yes | Preserve HTTP-only SameSite Strict cookie, secure cookies in deployment, logout invalidation, and no browser storage. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/tests/auth_and_health.rs] |
| V4 Access Control | yes | Keep all collection-management mutations authenticated through the single admin session cookie plus same-origin checks; keep public output static and published-only. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/publisher.rs] |
| V5 Input Validation | yes | Keep required metadata validation, image MIME/content validation, size limits, and upload byte decoding. [VERIFIED: controller/src/catalog.rs] [VERIFIED: controller/src/routes.rs] |
| V6 Cryptography / Secrets | yes | Use Argon2 for admin password hashes and keep Oracle/OCI/admin secrets out of static source and DTOs. [VERIFIED: controller/src/auth.rs] [VERIFIED: docs/configuration-contract.md] |
| V8 Data Protection | yes | Do not expose Object Storage URLs, namespaces, bucket names, object keys, image UUIDs, unpublished records, or original filenames. [VERIFIED: controller/tests/static_admin.rs] [VERIFIED: controller/tests/publisher.rs] |
| V12 File and Resources | yes | Validate image uploads, constrain body size, use UUID-only object keys, and coordinate media cleanup. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/src/storage_keys.rs] |
| V14 Configuration | yes | Keep provider modes redacted, deploy env files secret-owned, and Caddy route blocks for retired operator APIs. [VERIFIED: deploy/ansible/roles/autographs_deploy/tasks/main.yml] [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] |

### Known Threat Patterns for Rust Static Admin Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| CSRF against admin mutations | Tampering | SameSite Strict HTTP-only cookie plus Origin/Referer checks for cookie-authenticated mutations. [VERIFIED: controller/src/routes.rs] |
| Stored XSS from catalog metadata | Tampering / Elevation of Privilege | Escape generated HTML and use DOM `textContent`/`replaceChildren` patterns; tests already assert public browse safety. [VERIFIED: controller/src/publisher.rs] [VERIFIED: controller/tests/publisher.rs] |
| Private media identifier leakage | Information Disclosure | Dedicated redacted DTOs and privacy scans over static/admin source. [VERIFIED: controller/src/routes.rs] [VERIFIED: controller/tests/static_admin.rs] |
| Orphaned private originals | Information Disclosure / Repudiation | Explicit controller cleanup, event recording, retries/guidance, live smoke cleanup mode. [VERIFIED: controller/src/media.rs] [VERIFIED: controller/tests/live_persistence_smoke.rs] |
| Bearer token reaches collection management | Elevation of Privilege | Remove bearer-token authorization from collection-management routes and document the session cookie as the only collection-management path. [VERIFIED: docs/configuration-contract.md] [ASSUMED] |
| Unbounded release accumulation | Denial of Service | Add publisher-owned retention/pruning after successful promotion. [VERIFIED: controller/src/publisher.rs] [ASSUMED] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/06-admin-collection-workflow/06-CONTEXT.md` - user decisions, boundaries, and Phase 6 specifics.
- `.planning/REQUIREMENTS.md` - Phase 6 requirement IDs and descriptions.
- `.planning/ROADMAP.md` - Phase 6 goal and success criteria.
- `.planning/STATE.md` - Phase 5 closeout and current project state.
- `AGENTS.md` - project constraints, conventions, and guardrails.
- `controller/src/routes.rs` - current admin API/auth/publish route shape.
- `controller/src/catalog.rs` - current repository trait and domain types.
- `controller/src/oracle_catalog.rs` - Oracle persistence pattern.
- `controller/src/media.rs` and `controller/src/oci_media.rs` - media store abstraction and OCI adapter.
- `controller/src/publisher.rs` - static generation, validation, promotion, failed candidate retention.
- `controller/static-admin/*` - Phase 5 minimal admin shell.
- `controller/tests/*` - existing auth, upload, privacy, publisher, and live smoke tests.
- `docs/configuration-contract.md`, `docs/deployment-runbook.md`, `docs/static-runtime-runbook.md` - runtime config, deployment, smoke, and operator docs.
- `cargo test --manifest-path controller/Cargo.toml` - local test suite passed during research.

### Secondary (MEDIUM confidence)
- Axum 0.8.9 docs - router, handlers, extractors, state, JSON response patterns: https://docs.rs/axum/0.8.9/axum/
- Oracle JSON data type docs - JSON storage/serialization behavior: https://docs.oracle.com/en/database/oracle/oracle-database/26/adjsn/json-data-type.html
- Oracle JSON value index docs - function-based JSON indexes: https://docs.oracle.com/en/database/oracle/oracle-database/26/adjsn/creating-b-tree-indexes-json_value.html
- OCI Object Storage object docs - object replacement/delete model: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/managingobjects.htm
- OCI Object Lifecycle Management docs - lifecycle deletion and delay caveats: https://docs.oracle.com/en-us/iaas/Content/Object/Tasks/usinglifecyclepolicies.htm
- OCI CLI object delete docs - delete command shape: https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/os/object/delete.html
- OWASP ASVS project page - ASVS 5.0.0 current stable and purpose: https://owasp.org/www-project-application-security-verification-standard/

### Tertiary (LOW confidence)
- GSD research-plan selected Context7, but Context7 MCP and `ctx7` CLI were unavailable in this session; official docs were accessed through web search/open and cached as LOW provider confidence. [VERIFIED: gsd-tools query research-plan --input /tmp/research-plan-input.json] [VERIFIED: command -v ctx7]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - existing dependency versions and tools were verified from `controller/Cargo.toml` and local commands.
- Architecture: HIGH - phase work maps directly to existing controller/repository/media/publisher/Caddy boundaries.
- Edit-history schema: MEDIUM - Oracle JSON/index docs support the shape, but exact live ADB compatibility and migration strategy need implementation validation.
- Media cleanup: HIGH for existing primitives, MEDIUM for final retry/observable failure design.
- Security: HIGH for existing controls, MEDIUM for final operator-token decision.

**Research date:** 2026-06-20
**Valid until:** 2026-07-20 for codebase planning assumptions; 2026-06-27 for external docs/package currency.
