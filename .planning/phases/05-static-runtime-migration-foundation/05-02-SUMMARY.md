---
phase: 05-static-runtime-migration-foundation
plan: 02
subsystem: private-controller-auth
tags: [rust, axum, argon2, sessions, csrf, config]
requires:
  - phase: 05-01
    provides: Rust controller crate and public artifact DTO foundation
provides:
  - Axum controller runtime with public and redacted admin health endpoints
  - Single-admin session cookie and operator bearer-token guards
  - Same-origin enforcement for cookie-authenticated mutations and failed-login lockout
  - Controller runtime secret and OCI Customer Secret configuration contract
affects: [05-03-seed-path, 05-05-static-admin, 05-06-runtime-cutover]
tech-stack:
  added: [axum, tokio, argon2, tower]
  patterns: [redacted health DTOs, secure strict session cookies, bearer operator bypass, same-origin mutation guard]
key-files:
  created:
    - controller/src/main.rs
    - controller/src/config.rs
    - controller/src/auth.rs
    - controller/src/routes.rs
    - controller/tests/auth_and_health.rs
  modified:
    - .env.example
    - docs/configuration-contract.md
key-decisions:
  - "Use an Argon2 password hash for deployed single-admin login, with plaintext password accepted only as an explicit local-development shortcut."
  - "Require same-origin Origin or Referer for cookie-authenticated mutations while allowing explicit bearer-token operator calls."
patterns-established:
  - "Controller health responses expose readiness booleans only, never secret or OCI/Oracle values."
  - "Browser admin access uses HttpOnly SameSite=Strict cookies with Secure enabled by default."
requirements-completed: [STATIC-02, STATIC-05]
duration: 12 min
completed: 2026-06-01
---

# Phase 05 Plan 02: Private Controller Auth Foundation Summary

**Axum private controller with redacted health, strict admin sessions, operator bearer access, CSRF checks, and runtime secret documentation**

## Performance

- **Duration:** 12 min
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added the Rust controller binary and redacted health endpoints.
- Added single-admin login/logout, session cookies, bearer operator access, throttling, and cross-origin mutation rejection.
- Documented runtime-only controller credentials and OCI Customer Secret inputs.

## Task Commits

1. **Tasks 1-2: Add runtime health and single-admin auth foundation** - `621fe22`
2. **Task 3: Update configuration contract for controller secrets** - `2773fbf`

## Decisions Made

- Kept browser auth same-origin and cookie-based while retaining an explicit bearer path for SSH tunnel and maintenance calls.
- Defaulted session cookies to secure deployment behavior with a separately tested local HTTP exception.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed unavailable transitive RNG feature requirement**
- **Found during:** Tasks 1-2
- **Issue:** The Argon2 salt helper initially imported `OsRng` without the transitive `getrandom` feature.
- **Fix:** Generate salt bytes from `Uuid::new_v4()` and encode them for Argon2.
- **Files modified:** `controller/src/auth.rs`
- **Verification:** Full Rust suite passes.
- **Committed in:** `621fe22`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Authentication behavior is unchanged; dependency surface stays smaller.

## Issues Encountered

- Axum and authentication dependencies required one network-backed Cargo fetch before local tests could run.

## User Setup Required

- Production must supply `AUTOGRAPHS_ADMIN_PASSWORD_HASH`, `AUTOGRAPHS_OPERATOR_API_TOKEN`, and OCI Customer Secret values through runtime/operator secret stores.

## Next Phase Readiness

- `05-03` can place minimal catalog and private media seed routes behind the tested auth boundary.

## Self-Check: PASSED

- `cargo test --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml auth_and_health`
- `rg -n "AUTOGRAPHS_CONTROLLER_BIND_ADDR|AUTOGRAPHS_ADMIN_PASSWORD_HASH|AUTOGRAPHS_OPERATOR_API_TOKEN|Customer Secret|static release|GitHub-hosted" .env.example docs/configuration-contract.md`

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-01*
