---
phase: 06-admin-collection-workflow
plan: 06
subsystem: auth
tags: [rust, axum, admin-session, csrf, live-smoke]

requires:
  - phase: 06-admin-collection-workflow
    provides: polished static admin workflow and collection-management routes
provides:
  - session-only collection management authorization
  - generic login and lockout responses
  - live static publish smoke using admin login cookies
affects: [admin-auth, collection-management, live-smoke, operator-docs]

tech-stack:
  added: []
  patterns: [session-only mutation authorization, cookie-authenticated smoke helpers]

key-files:
  created: []
  modified:
    - controller/src/config.rs
    - controller/src/routes.rs
    - controller/src/routes/admin_items.rs
    - controller/tests/auth_and_health.rs
    - controller/tests/admin_workflow.rs
    - controller/tests/media_cleanup.rs
    - controller/tests/publisher.rs
    - controller/tests/seed_content.rs
    - controller/tests/live_static_publish_smoke.rs
    - .env.example

key-decisions:
  - "Collection-management item and publish routes require the HTTP-only admin session path, not bearer operator tokens."
  - "Bearer token compatibility remains available only for non-management diagnostics such as the protected health/auth probe."
  - "Live static publish smoke now logs in through /admin/api/login and sends Cookie plus Origin headers for management calls."

patterns-established:
  - "Session-only admin helper: authorize_admin_session wraps authenticate plus CSRF/origin checks and rejects AuthKind::OperatorToken."
  - "Controller integration tests authenticate collection-management calls through /admin/api/login and reuse the Set-Cookie pair."

requirements-completed: [ADMIN-01, ADMIN-05]

duration: live session
completed: 2026-07-02
---

# Phase 06-06: Admin Auth Boundary Summary

**Session-cookie-only collection management with generic login/lockout behavior and cookie-authenticated live smoke coverage**

## Performance

- **Duration:** live session
- **Started:** 2026-07-02
- **Completed:** 2026-07-02
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added `authorize_admin_session` and moved `/admin/api/items*` plus `/admin/api/publish*` management routes to session-only authorization.
- Tightened runtime auth validation so `AUTOGRAPHS_ADMIN_PASSWORD_HASH` or local/test `AUTOGRAPHS_ADMIN_PASSWORD` is required; `AUTOGRAPHS_OPERATOR_API_TOKEN` alone no longer satisfies startup auth validation.
- Converted admin workflow, media cleanup, publisher, and seed-content management tests to login-cookie auth.
- Added bearer rejection, session management, and generic lockout-message regressions.
- Updated the ignored live static publish smoke to log in through `/admin/api/login` and use the resulting cookie for management calls.

## Task Commits

1. **Task 1: Make session cookie login the only collection-management auth path** - `ad566a7` (`feat`)
2. **Task 2: Preserve clear session, lockout, and live-smoke behavior** - `d00be6c` (`feat`)

## Files Created/Modified

- `controller/src/config.rs` - Requires admin password hash or local/test password for runtime auth validation.
- `controller/src/routes.rs` - Adds session-only authorization helper and generic login/lockout responses.
- `controller/src/routes/admin_items.rs` - Requires session auth for admin item list/detail/history routes.
- `controller/tests/auth_and_health.rs` - Covers bearer rejection, session management, cookie attributes, logout/expired sessions, and generic lockout copy.
- `controller/tests/admin_workflow.rs` - Uses admin login cookie for collection workflow requests.
- `controller/tests/media_cleanup.rs` - Uses admin login cookie for upload/delete/replace/cleanup retry requests.
- `controller/tests/publisher.rs` - Uses admin login cookie for publish and publish-status requests.
- `controller/tests/seed_content.rs` - Uses admin login cookie for create/upload/publication seed workflow.
- `controller/tests/live_static_publish_smoke.rs` - Uses operator-supplied admin password to log in and authenticate live smoke management calls.
- `.env.example` - Labels `AUTOGRAPHS_ADMIN_PASSWORD` as local/smoke-only and token compatibility as non-management only.

## Decisions Made

- Kept `AuthKind::OperatorToken` parsing for non-management compatibility routes, but collection-management item and publish paths reject it.
- Returned `429 Too Many Requests` with the approved lockout copy after failed-login lockout while keeping invalid credentials generic.
- Required live smoke operators to provide `AUTOGRAPHS_ADMIN_PASSWORD` for the cookie login path instead of `AUTOGRAPHS_OPERATOR_API_TOKEN`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Updated broader management test suites**
- **Found during:** Task 1
- **Issue:** Existing admin workflow, media cleanup, and publisher tests still used bearer tokens for management routes. Leaving them unchanged would make the final controller test suite fail and would keep stale auth assumptions alive.
- **Fix:** Converted those suites to obtain an admin session via `/admin/api/login` and send same-origin cookie-authenticated requests.
- **Files modified:** `controller/tests/admin_workflow.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/publisher.rs`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture`, `cargo test --manifest-path controller/Cargo.toml --test media_cleanup -- --nocapture`, and `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` passed.
- **Committed in:** `ad566a7`

---

**Total deviations:** 1 auto-fixed (Rule 2).
**Impact on plan:** The fix kept the test surface aligned with the new single-auth-path contract; no scope expansion beyond the planned auth boundary.

## Issues Encountered

- Initial route helper implementation partially moved `AuthKind::Session`; fixed by matching by reference.
- Mechanical test conversion introduced normal router ownership issues in Rust tests; fixed with explicit router clones and stable login-cookie helpers.

## User Setup Required

None for local execution. Operator-run live static publish smoke now requires `AUTOGRAPHS_ADMIN_PASSWORD` in addition to the existing live runtime coordinates.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` passed.
- `cargo test --manifest-path controller/Cargo.toml --test auth_and_health -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --test seed_content -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture` passed with the live smoke safely skipped because `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` was not set to `true`.
- `cargo test --manifest-path controller/Cargo.toml` passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` passed.
- `! rg -n "Authorization: Bearer" controller/tests/live_static_publish_smoke.rs` passed.

## Self-Check: PASSED

- Summary created after task commits.
- Required source assertions are represented in code and tests.
- No live Oracle/Object Storage smoke was claimed; only the ignored skip path was verified locally.

## Next Phase Readiness

Plan `06-07` can now close Phase 6 operator/security documentation against the session-only collection-management path and retired bearer-token collection workflow.

---
*Phase: 06-admin-collection-workflow*
*Completed: 2026-07-02*
