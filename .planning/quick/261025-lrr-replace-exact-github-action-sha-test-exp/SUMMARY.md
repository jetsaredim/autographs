---
status: implementation_complete
commits: [f7a0ea8, cafdcc2, 8592d12]
---

# Release workflow contract tests

The release workflow contract no longer asserts the `uses` values for `actions/create-github-app-token`, `googleapis/release-please-action`, two `actions/checkout` steps, or `docker/login-action`. Renovate manages their pins and versions. The tests continue to check App token inputs, narrow permissions, release ordering, checkout refs and paths, registry credentials, and token handoff to release-please.

The Dockerfile contract test no longer requires a particular cargo-chef or CA certificate package build. It still checks the cargo-chef prepare/cook flow and CA certificate installation. Project conventions now direct tests toward dependency behavior and compatibility contracts rather than exact managed versions or digests. The dependency guide no longer duplicates the current privileged action SHAs.

## Verification

- `python3 -m unittest scripts/test_release_workflow.py`: 22 tests passed.
- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes`: 8 tests passed.
- Renovate PR #288 changes only the `actions/create-github-app-token` digest in `.github/workflows/deploy.yml`.
- A repository test search found no remaining exact GitHub Action `uses` assertions.
- `git diff --check` passed.

PR #288 remains unmerged and will need fresh CI after this test change reaches `main`. PR #290 remains unmerged for review.
