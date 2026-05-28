# Phase 5: Static Runtime Migration Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 05-Static Runtime Migration Foundation
**Areas discussed:** Static Artifact Shape, Private Publish Boundary, Image Derivative Rules, Side-by-Side Runtime Preview, Seed Shell Minimalism, Rust Controller Access Model, Migration Strategy, Static Publish Validation, Admin UI Shape

---

## Static Artifact Shape

| Option | Description | Selected |
|--------|-------------|----------|
| HTML + assets only | Generate static pages and public image derivatives only. | |
| HTML + JSON indexes | Generate pages plus public-safe JSON for collection behavior. | ✓ |
| Manifest-first prototype | Emphasize artifact contracts before UX parity. | |

**User's choice:** Preserve current public page functionality without a live Node backend by generating JSON data used for filters and collection behavior.
**Notes:** Public JSON shape should be profiled before being locked. The user wants roughly 500-item headroom, while the real backlog is about 120 items and production currently has about 3 live items. The public JSON contract should be versioned but lightweight and may refine ambiguous current schema fields.

---

## Private Publish Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| VM-local command | SSH to the runtime VM and run publish locally. | |
| Token-guarded private API | Private/tunnel-only API triggers publish and reports status. | |
| Admin shell button | Minimal admin UI action calls private API. | ✓ |

**User's choice:** Future admin UI should trigger an internal API that runs incremental static regeneration, with a full rebuild option.
**Notes:** The user clarified that the controller should be Rust and should own both publish orchestration and minimal functional admin APIs. Phase 5 should build this Rust controller now rather than defer it.

---

## Image Derivative Rules

| Option | Description | Selected |
|--------|-------------|----------|
| VM filesystem derivatives | Store generated derivatives locally under static output. | |
| Public derivative bucket/path | Store generated derivatives in Object Storage and serve with clean paths. | ✓ |
| Caddy private-original proxy | Proxy image requests directly to private originals. | |

**User's choice:** Avoid long-term local file copies and avoid additional buckets unless needed. Use Object Storage-backed public-safe derivatives, served through Caddy `/media/...` if feasible.
**Notes:** The user asked about Caddy proxying and specifically mentioned `github.com/lindenlab/caddy-s3-proxy`. The captured decision is to evaluate that kind of plugin for public-safe derivatives only, not private originals. The user also chose UUID-only object naming and accepted downtime/schema/key migration to cleanly move away from filename-bearing keys.

---

## Image Variant Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal variants | Generate thumbnail and detail variants only. | ✓ |
| Responsive variants | Generate many sizes/formats for `srcset`. | |
| Original-only serving | Serve originals and resize in HTML/CSS. | |

**User's choice:** Generate only the minimum useful derived images.
**Notes:** The user considered storage limits and confirmed originals are usually around 2-2.5 MB, with a few around 20 MB. The decision is to keep private originals for re-derivation but only generate thumbnail and detail derivatives in Phase 5.

---

## Side-by-Side Runtime Preview

| Option | Description | Selected |
|--------|-------------|----------|
| Preview path | Serve static output under a preview path while Next.js remains primary. | |
| Preview subdomain | Serve static output on a separate host/subdomain. | |
| Short migration cutover | Validate privately, then switch routes/runtime during planned downtime. | ✓ |

**User's choice:** Use a short migration cutover with downtime allowed.
**Notes:** The user explicitly prefers roll-forward only and is comfortable with VM rebuild if needed. No rollback mechanism is required beyond preflight validation and fix-forward behavior.

---

## Static Publish Validation

| Option | Description | Selected |
|--------|-------------|----------|
| Manifest/privacy checks | Check generated manifest and privacy boundaries. | |
| End-to-end smoke checks | Check generated pages, filters, media, and Caddy routes. | |
| Full validation gate | Combine manifest/privacy, smoke checks, media checks, storage summary, and publish status. | ✓ |

**User's choice:** Use staged candidate releases and validation before promotion.
**Notes:** The user explored a `current`/`previous`/`next` model, then refined it to incremental artifact generation with complete candidate-release validation. The candidate can be based on the current release, with only affected files regenerated, then promoted atomically by symlink/rename after validation.

---

## Failed Publish Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Retain failed candidates | Keep all failed outputs/logs for inspection. | |
| Delete failed candidates | Remove failed outputs automatically. | |
| Retain latest failed only | Keep only the latest failed candidate and logs. | ✓ |

**User's choice:** Keep the latest failed candidate only.
**Notes:** Failure notification should happen through the admin UI publish/status panel and API/CLI response. No email/SMS/webhook notification is needed in Phase 5.

---

## Seed Shell Minimalism

| Option | Description | Selected |
|--------|-------------|----------|
| Create-first flow | Create a new item, upload image, publish, verify static page. | |
| Manage-existing flow | List/edit existing items and regenerate affected output. | |
| End-to-end thin admin | Create, edit, upload, publish/unpublish, rebuild, status. | ✓ |

**User's choice:** Build the end-to-end thin admin path.
**Notes:** This is in addition to static generation. The user clarified that production only has about 3 items live, while the full collection/backlog is about 120 items. Phase 5 should prove the path for loading and publishing the real collection after migration.

---

## Rust Controller Access Model

| Option | Description | Selected |
|--------|-------------|----------|
| SSH tunnel only | Admin/controller reachable only through local/tunnel access. | |
| Same-host private route | `/admin` and `/admin/api/*` exist on the site host and require auth. | ✓ |
| Hybrid future route | Keep Phase 5 tunnel-only but design future same-host routes. | |

**User's choice:** Use same-host admin routes because it gets closer to the end state faster.
**Notes:** Admin UI and local/operator system calls should be able to reach the controller. The user asked about username/password login returning a bearer token; the captured guidance rejects base64 as security and prefers a simple login with secure same-origin session cookies for browser use plus CLI-friendly token/session behavior.

---

## Migration Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Clean break migration | Planned downtime and direct replacement. | |
| Compatibility bridge | Keep old Node paths while new Rust/static path coexists. | |
| Two-step cutover | Deploy/prove internally, then short downtime cutover. | ✓ |

**User's choice:** Two-step cutover.
**Notes:** For the current 3 production items, the planner can choose migration or manual recreation based on effort/risk. Heavy compatibility migration tooling is not required.

---

## Admin UI Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Single guided form | One create/edit form with upload and publish controls. | ✓ |
| Simple list + edit screens | List items, open create/edit screens, upload, publish, status. | |
| Workflow wizard | Step-by-step create/upload/review/publish flow. | |

**User's choice:** Single guided form is probably fine for Phase 5.
**Notes:** This is only the Phase 5 implementation shape. Phase 6 can improve the overall admin workflow later.

---

## the agent's Discretion

- Exact Rust framework and crates, provided high-risk integrations are validated early.
- Exact JSON artifact split after profiling.
- Exact derivative dimensions and format settings.
- Exact Caddy S3 proxy plugin or equivalent.
- Exact migration/recreation choice for the current production items.
- Exact release directory layout, provided it supports candidate validation, incremental updates, and atomic promotion.

## Deferred Ideas

- Polished admin workflow and rich daily-use UX remain Phase 6.
- Edit-history browsing, advanced media management, media cleanup guarantees, and richer validation UX remain Phase 6.
- AI/OCR assistance remains Phase 7.
- Full responsive image variant sets can wait until profiling or UX needs justify them.
- Email/webhook/push publish failure notification can wait until async operations need out-of-band alerts.
- A Go/Rust comparison spike was discussed but not selected.
