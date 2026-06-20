# Phase 6: Admin Collection Workflow - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-20
**Phase:** 6-Admin Collection Workflow
**Areas discussed:** Daily Admin Flow, Image Management, Edit History, Publish And Cleanup, Admin Security Polish

---

## Daily Admin Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Collection dashboard | Start with searchable/filterable existing items and actions to add, edit, publish, unpublish, and view public pages. | |
| Create/edit workspace | Start directly in an item editor optimized for backlog entry. | |
| Publish/status command center | Start with operational publish health, release state, and recent changes. | |
| Landing/status hub | Show site health/status first, then offer paths to add new data or modify existing data. | yes |

**User's choice:** Landing/status hub.
**Notes:** The user expects a large initial backlog of items/data to enter. After that, ongoing use is expected to be low-frequency new item entry a couple times per year. Existing-item maintenance should be available but not dominate the experience.

---

## Image Management

| Option | Description | Selected |
|--------|-------------|----------|
| Simple ordered gallery | Upload multiple images, reorder them, choose primary, edit alt text/captions, and remove images. | |
| Upload queue with review step | Confirm primary/supporting roles, alt text, and metadata before saving images to the item. | |
| Lightweight append-only uploads | Add images and mark primary, avoiding rich reorder/review tools. | yes |

**User's choice:** Lightweight image management.
**Notes:** The user said even the simple ordered gallery felt too feature-rich. The final product should be fine with uploading multiple images and marking one as primary. Avoid over-complicating image management.

---

## Edit History

| Option | Description | Selected |
|--------|-------------|----------|
| Human-readable timeline | Show events such as metadata updated, image uploaded, primary image changed, and published. | |
| Field-level diffs | Show before/after values for meaningful changed fields. | yes |
| Operational audit only | Record internally for diagnostics/security but show only last updated/last published in the UI. | |

**User's choice:** Field-level diffs.
**Notes:** Field-level before/after visibility is the desired edit-history behavior. Image and publication changes should still be represented clearly, but this is not a multi-user enterprise audit system.

---

## Publish And Cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Save draft, then publish separately | Save private source-of-truth changes first, then explicitly publish when ready. | yes |
| Save and publish in one action | Save the item and immediately update the public static site. | |
| Two actions with smart prompt | Save first, then prompt to publish with a preview/status summary. | |

**User's choice:** Separate save and publish.
**Notes:** The user asked whether multiple saved items can be batched into one publish operation. The locked direction is that saving updates Oracle/Object Storage only; a later explicit publish can include multiple saved/new/edited items in one static release. The landing/status hub should show pending unpublished changes. Normal publish should be incremental; full rebuild remains a repair/structural-change action. The user also noted that the runtime currently appears to preserve a large backlog of generated releases; Phase 6 should define release retention/pruning so filesystem usage does not grow unbounded as the catalog grows.

---

## Admin Security Polish

| Option | Description | Selected |
|--------|-------------|----------|
| Session and lockout clarity | Show login/session state, keep secure cookie/CSRF behavior, and make lockout/expired-session behavior understandable. | yes |
| Admin audit trail | Record broader admin/security events such as login failures, lockouts, publish attempts, and image deletion. | |
| Operator diagnostics panel | Surface provider modes, controller health, last publish, cleanup warnings, and smoke guidance. | yes |

**User's choice:** Session clarity plus a small diagnostics panel.
**Notes:** The user agreed that session and lockout clarity should be mandatory and that a small diagnostics/status panel fits the desired landing hub. A broad standalone audit trail is not the priority, though item edit history, image events, and publish events should support troubleshooting.

---

## the agent's Discretion

- Exact admin layout and static UI organization, provided the first screen remains a landing/status hub with add-new and modify-existing paths.
- Exact field-diff persistence and rendering shape, provided meaningful before/after values are recorded and visible.
- Exact cleanup retry/confirmation details, provided normal image edits avoid orphaned metadata or Object Storage objects and failures are observable.

## Deferred Ideas

- Advisory OCR/AI metadata suggestions remain Phase 7 scope.
- Public accounts, multi-admin roles, social features, bulk import, and a public multi-service split remain out of scope.
- Rich image management such as drag/reorder, captions, review queues, and a full digital asset management workflow is intentionally deferred.
- A broad standalone admin audit trail beyond item edit history, image events, publish events, and minimal session/security clarity is not a Phase 6 priority.
