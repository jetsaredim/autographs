---
id: 261105-ee0
status: complete
completed: 2026-11-05
commits: [3c9aee6, 38e6368]
pr: https://github.com/jetsaredim/autographs/pull/321
---

# Disable publish controls during deployment freeze

The deployed Caddy freeze now marks the authenticated, proxied publish-status response with `X-Autographs-Publish-Frozen: true`. Both the in-place preflight gate and full-deploy rendered gate provide the same signal while retaining exact 503 responses for publish POSTs. The admin shell disables all three publish controls while frozen or status is unavailable, refreshes status every 15 seconds while open, and rechecks immediately before publishing. The server-side Caddy gate remains authoritative.

## Verification

- `node controller/tests/static_admin_behavior.mjs` passed, including frozen, available, unavailable, and pre-publish recheck behavior.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin --test caddy_static_routes` passed: 20 admin and 9 Caddy route tests.
- The in-place gate unit test passed.
- The Docker-backed `test_actual_unfreeze_all_marked_against_real_caddy` passed after 1061 seconds, covering preflight, freeze, unfreeze, and recovery scenarios.
- Ansible freeze render test, syntax check, targeted lint, Rust formatting, JS syntax, repository hygiene, and `git diff --check` passed.
- PR #321 review round 1 found two Warning issues: an edit during the async status check could still publish, and a retry from the previous version's valid frozen gate would fail. Commit `38e6368` rechecks unsaved edits after the await and upgrades a validated legacy gate in place. Focused regression tests, Rust route/admin tests, Ansible syntax/lint, and PR CI passed after the fixes.
- Review round 2 at `38e6368` was clean; both findings were resolved and the reviewer posted a clean-review confirmation on PR #321.

## Rollout note

The static admin shell is bundled into generated releases. The new controls become visible after a publish with the updated controller code; the deployment that first installs this change still serves the previously published shell until that publish occurs.
