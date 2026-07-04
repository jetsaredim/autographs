---
status: resolved
trigger: "Admin login page initially shows a login failed message after submitting the admin password, then later refreshes/reveals the admin console."
created: 2026-07-03
updated: 2026-07-03
---

# Debug Session: admin-login-initial-failure

## Symptoms

- expected_behavior: Submitting the correct admin password should transition cleanly into the admin console without showing a failed-login message.
- actual_behavior: The page shows a login failed message initially, then later refreshes or updates into the admin console.
- error_messages: UI displays a login failed message; no server log or browser console text provided yet.
- timeline: Observed after the session-cookie admin auth flow replaced API-token publishing.
- reproduction: Open admin login page, submit admin password, observe transient failed-login message before the console appears.

## Current Focus

- hypothesis: Resolved; post-authentication errors were handled as login failures.
- test: Focused static admin source contract test.
- expecting: Invalid credentials redirect home, lockouts show lockout copy, and successful login transitions without false failed-login copy.
- next_action: none
- reasoning_checkpoint:
- tdd_checkpoint:

## Evidence

- 2026-07-03: `controller/static-admin/admin.js` login submit handler wrapped authentication, form reset, in-place render, and navigation in one `try` block. Any post-login exception could show `Login failed.` after the server had already issued the session cookie.
- 2026-07-03: `request()` already handles `204 No Content` safely, so the original no-content parsing hypothesis was eliminated.
- 2026-07-03: User clarified that obviously invalid admin passwords also do not redirect home as expected. The fix preserves `401` redirect-to-home while narrowing the success path error boundary.

## Eliminated

- hypothesis: Successful `204 No Content` login response is parsed as JSON and throws.
  reason: `request()` reads text for non-JSON responses and returns `null` for status `204`.

## Resolution

- root_cause: The login UI error boundary treated post-authentication navigation/render failures as login failures, even after the cookie had been accepted.
- fix: Limit login failure handling to the `/admin/api/login` request, restore `401` redirect-to-home behavior, and transition/navigate separately after successful authentication.
- verification: `cargo test --manifest-path controller/Cargo.toml static_admin` passed on 2026-07-03.
- files_changed: controller/static-admin/admin.js, controller/tests/static_admin.rs
