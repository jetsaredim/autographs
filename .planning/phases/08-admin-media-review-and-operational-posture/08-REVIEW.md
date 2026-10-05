---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-05T01:35:28Z
depth: deep
files_reviewed: 11
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/static-admin/index.html
  - controller/tests/admin_workflow.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Phase 08: Code Review Report — Gesture Lifecycle Authority Follow-up

**Reviewed:** 2026-10-05T01:35:28Z
**Depth:** deep
**Files Reviewed:** 11
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

Commit `0559c82` closes both pointer-lifecycle findings from the prior review. The centralized stage-render barrier keeps the capture owner connected across current source, preview, adjusted-output, assist, control, resize, and terminal interleavings; the fitted-source grab offset is stable for inset handles, portrait/landscape geometry, and resize rebasing. Focused browser-behavior coverage and the complete Rust verification suite pass.

One image-identity boundary remains outside that authority model. An editor image mutation that crossed the network before the review opened can complete after the review or a drag becomes authoritative. Because the review and gesture identify an image only by stable item/image UUIDs, replacing the bytes under that UUID does not invalidate the old source projection or draft. The old-pixel adjustment can then be previewed and saved against the replacement image.

### Finding lineage

| Prior finding or invariant | Current disposition | Evidence |
| --- | --- | --- |
| Pointer follow-up WR-01: async completion replaces capture owner | Resolved for the inventoried review-stage producers | `renderImageReview()` is the centralized barrier; source load/resize mutate the connected projection in place; source error uses the idempotent terminal path; preview/output/assist/control callbacks follow their approved defer, supersede, reject, or teardown policies. |
| Pointer follow-up WR-02: inset target changes normalized corner | Resolved | Pointerdown stores normalized fitted-source grab offset, same-coordinate moves are no-ops, resize rebases against the last client coordinates, and all four inset corners plus portrait/landscape deltas execute in the DOM harness. |
| Round 3 WR-03: browser-faithful lifecycle coverage | Incomplete fix / sibling mutation-path test weakness | Connected-node, disabled-control, async review producer, capture release, and reentrant lost-capture cases now execute, but no test defers an editor image replacement across review open/drag and verifies that source-content identity invalidates the review (CR-01). |
| Round 2 CR-05 / convergence async authority | Incomplete fix / sibling-path miss | Session, item UUID, image UUID, projection generation, request/render generation, node, and draft checks reject stale review work, but none represents a same-UUID replacement of the private source bytes (CR-01). |
| Mounted output, Save/Reset egress, promotion, legacy migration, private-original, privacy, and public-output contracts | Resolved / no regression found | Exact mounted-node and blob authority, pending mutation serialization, active-release protection, comparison-map migration, authenticated no-store previews, redacted errors, and public privacy tests all remain green. |

### Critical Issues

#### CR-01: Same-ID image replacement can leave an old-source review authoritative

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1283-1290`
**Related:** `controller/static-admin/admin.js:1725-1750`, `controller/static-admin/admin.js:2181-2204`, `controller/static-admin/admin.js:2293-2302`, `controller/static-admin/admin.js:2765-2787`, `controller/tests/static_admin_behavior.mjs:482-997`

**Issue:** Editor image operations are not represented in the review authority tuple. `replaceImage()` can issue its `PUT` and, before that response settles, the operator can press the still-enabled Review action and begin a perspective drag. Replacement preserves the image UUID while changing the private bytes and clearing the saved adjustment. When the delayed replacement response arrives, `renderEditor(item)` clears the review only when the *item* UUID differs; for the same item it updates `state.currentItem` and leaves the review session, source node, draft, projection generation, and active capture intact. Both the move and terminal checks compare only the same stable item/image UUIDs, so the drag remains authoritative. Its final preview reads the newly replaced bytes using geometry derived from the old image, and Save can persist that stale adjustment onto the replacement image. The same missing operation boundary lets delayed delete/upload/primary completions refresh editor state without deterministically reconciling or invalidating the active review. The behavior harness covers explicit review/session teardown but cannot reproduce this race because it never defers an editor image mutation across review opening and capture.

**Fix:** Add one editor image-mutation authority/serialization boundary. Track pending image operations before issuing upload, primary, delete, replacement, or cleanup requests; prevent `openImageReview()` while one is pending, and make every successful mutation completion invalidate or reload any review whose item/image content may have changed before applying `renderEditor()`. For replacement specifically, bind review authority to an immutable source-content revision such as checksum/etag returned by the review endpoint, or always terminate and reopen the same-UUID review when replacement settles. The termination must release capture, discard deferred render work, cancel previews, and prevent the old draft from being saved. Add a connected-node test that defers replacement, opens review, starts and moves a drag, then resolves replacement and proves exactly-once teardown, no final old-source preview, no enabled Save, and a fresh replacement-source review; add sibling delete/upload/primary completion cases or a shared-operation test proving they use the same guard.

## Verification

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets` — passed; 163 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- Source-only `git diff --check origin/main...HEAD` across all 11 reviewed files — passed. The repository-wide check still reports pre-existing trailing spaces in the planning-only pointer-plan review artifact, which is outside this source-review scope.

---

_Reviewed: 2026-10-05T01:35:28Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
