---
phase: 05-static-runtime-migration-foundation
plan: 05
subsystem: static-admin
tags: [html, css, javascript, axum, csrf, privacy]
requires:
  - phase: 05-04
    provides: authenticated publish endpoints and redacted status API
provides:
  - Minimal browser-operable static admin seed and publish shell
  - Rich private metadata and upload alt-text support in the Rust catalog boundary
  - Static admin source privacy and same-origin API contract tests
affects: [05-06-runtime-cutover, 05-07-live-proof, 06-admin-workflow]
tech-stack:
  added: []
  patterns: [http-only-cookie-admin-shell, same-origin-admin-api, static-source-privacy-test]
key-files:
  created:
    - controller/static-admin/index.html
    - controller/static-admin/admin.js
    - controller/static-admin/admin.css
    - controller/tests/static_admin.rs
  modified:
    - controller/src/catalog.rs
    - controller/src/routes.rs
    - docs/static-runtime-runbook.md
key-decisions:
  - "Keep the Phase 5 shell framework-free and browser-storage-free; rely on the controller HTTP-only cookie and same-origin mutation checks."
  - "Persist the richer private catalog metadata already represented by the Oracle schema instead of presenting non-functional form fields."
patterns-established:
  - "Static admin source is scanned for embedded secrets, private storage terms, browser credential storage, and privileged URLs outside /admin/api/*."
requirements-completed: [STATIC-05]
duration: 10m
completed: 2026-06-02
---

# Phase 05 Plan 05: Minimal Static Admin Shell Summary

**Framework-free private admin shell for login, seed/edit, upload, publication control, rebuild triggers, and publish status**

## Accomplishments

- Added a minimal `/admin` shell with login/logout, status refresh, incremental/full publish controls, a guided item form, image upload, alt text, and publish/unpublish actions.
- Extended the Rust catalog boundary to retain event, source, inscription, certification, estimated-year, object-reference, and image alt-text values already modeled by Oracle.
- Added source privacy tests, same-origin privileged-path checks, JavaScript syntax validation, and logout session invalidation regression coverage.

## Verification

- `cargo test --manifest-path controller/Cargo.toml static_admin -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml -- --nocapture`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- `node --check controller/static-admin/admin.js`
- `git diff --check`

## Next Phase Readiness

- `05-06` can package the Rust controller and static admin shell, add Caddy routes, and extend CI/deploy wiring.
- The shell remains intentionally minimal; Phase 6 owns polished collection-management UX.

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-02*
