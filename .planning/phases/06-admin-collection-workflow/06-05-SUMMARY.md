---
phase: 06-admin-collection-workflow
plan: 05
subsystem: ui
tags: [static-admin, javascript, css, admin-workflow, privacy-tests]

requires:
  - phase: 06-04
    provides: redacted admin status, pending publish state, cleanup warnings, release retention, and explicit publish batching
provides:
  - polished static admin hub with backlog-first add flow and item maintenance path
  - no-build same-origin admin client for item, image, history, diagnostics, and publish workflows
  - Phase 6 static admin visual system with responsive work-focused layout
  - source/privacy/accessibility tests for the static admin surface
affects: [phase-06-admin-auth-hardening, admin-ui, static-admin, operator-diagnostics]

tech-stack:
  added: []
  patterns:
    - framework-free static admin UI hydrated by same-origin `/admin/api/*` calls
    - DOM-node/text rendering for untrusted admin API values
    - source-level privacy and accessibility contract tests for committed static admin assets

key-files:
  created:
    - .planning/phases/06-admin-collection-workflow/06-05-SUMMARY.md
  modified:
    - controller/static-admin/index.html
    - controller/static-admin/admin.js
    - controller/static-admin/admin.css
    - controller/tests/static_admin.rs

key-decisions:
  - "The Phase 6 admin workflow remains plain static HTML/CSS/JavaScript with no frontend build system or browser storage."
  - "The browser client renders catalog, history, image, publish, and diagnostics values through DOM node creation and textContent."
  - "Image tiles show safe metadata and actions only; private originals are managed through same-origin admin API calls, not direct object URLs."

patterns-established:
  - "Admin UI source includes a full workflow shell with stable DOM regions for hub, add/edit, item finder, publish, diagnostics, images, and history."
  - "Static admin tests now assert workflow copy, same-origin route safety, denied private terms, and label coverage for form controls."

requirements-completed: [ADMIN-02, ADMIN-03, ADMIN-04, DATA-03, MEDIA-04]

duration: 10min
completed: 2026-06-28
---

# Phase 06-05: Static Admin Collection Workflow Summary

**Framework-free admin hub and collection workflow backed by same-origin private controller APIs**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-28T12:52:23Z
- **Completed:** 2026-06-28T13:01:44Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Replaced the Phase 5 admin shell with an `Admin hub`, top-level workflow tabs, backlog-first add form, item finder, image management surface, history area, publish view, and redacted diagnostics panel.
- Added a no-build static admin client for `/admin/api/status`, item list/detail/save/history, image upload/primary/remove/replace/cleanup retry, and incremental/full publish.
- Applied the approved Phase 6 visual system with neutral surfaces, 44px controls, 8px radius, responsive grids, wrapping tabs, sticky save bar, semantic warning/success/destructive states, and no gradients or decorative hero treatment.
- Expanded static admin tests to cover Phase 6 workflow copy, denied private terms, same-origin route safety, and visible label coverage for form controls.

## Task Commits

1. **Task 1: Build admin workflow markup** - `71466d5` (feat)
2. **Task 2: Wire same-origin admin workflow client** - `0eacbba` (feat)
3. **Task 3: Apply admin visual system** - `e18ee98` (feat)

## Files Created/Modified

- `controller/static-admin/index.html` - Defines logged-out state, admin hub, workflow tabs, add/edit form sections, item finder, image grid, history list, publish controls, diagnostics panel, and required Phase 6 copy.
- `controller/static-admin/admin.js` - Implements same-origin admin API calls, view switching, hub/status rendering, item list/editor/history/images/diagnostics rendering, save/upload/image cleanup actions, publish actions, and auth-expiry handling.
- `controller/static-admin/admin.css` - Implements the approved Phase 6 static admin visual system and responsive states.
- `controller/tests/static_admin.rs` - Adds source-level workflow, privacy, route, and accessibility assertions for static admin assets.

## Decisions Made

- Kept the admin UI static and framework-free because the project constraint explicitly favors generated/static artifacts and one Rust private controller for v1.
- Rendered untrusted admin API values with DOM creation and `textContent` rather than HTML string injection.
- Preserved save and publish as separate explicit actions; `Publish changes` uses incremental publish by default and `Full rebuild` requires confirmation.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- During Task 1, the new static source test initially expected `/admin/api/status` before the JS task existed. The markup now carries that safe endpoint reference and Task 2 also defines it in `const endpoints`.
- During Task 1, the hidden item id input tripped the label coverage test. The markup now exposes a visible read-only Item ID field with a matching label.
- During Task 2, the static source test still expected the legacy redacted health and publish-status endpoints. They were retained in the endpoint map alongside the new Phase 6 status API.

## Verification

- `node --check controller/static-admin/admin.js` - passed
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture` - passed, 3 tests
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` - passed, 12 tests
- `cargo test --manifest-path controller/Cargo.toml` - passed, full controller suite
- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed

## Known Stubs

None. The touched-file scan found only normal form reset/null handling and one intentional input placeholder example for tags; no unwired mock data or blocking UI stubs were introduced.

## Threat Flags

None - the new browser workflow, same-origin admin API calls, static source privacy boundary, and DOM rendering behavior were covered by the plan threat model.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 06-06 can harden the single session-cookie admin auth path against this completed static admin workflow. The UI now uses the private `/admin/api/*` shape consistently and has source tests guarding against browser storage, direct Object Storage details, and privileged non-admin API paths.

## Self-Check: PASSED

- Key files exist, including this summary and all modified static admin/test files.
- Task commits exist: `71466d5`, `0eacbba`, and `e18ee98`.
- Full plan verification passed: JS syntax, static admin tests, admin workflow tests, full controller tests, and Rust formatting.
- Touched-file stub scan found no blocking stubs.
- Static admin privacy scan found only safe `/admin/api/*` route references and no denied private storage/browser-storage terms.

---
*Phase: 06-admin-collection-workflow*
*Completed: 2026-06-28*
