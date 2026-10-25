---
status: implementation_complete
commit: edf1a02
---

# Privileged action pin checks

The release workflow contract now requires the exact action repositories and full lowercase 40-character commit SHAs for `actions/create-github-app-token` and `googleapis/release-please-action`. It no longer embeds their current digest values in tests, so Renovate digest updates can pass after review while the actions stay immutably pinned.

## Verification

- `python3 -m unittest scripts/test_release_workflow.py`: 22 tests passed.
- Renovate PR #288's proposed `actions/create-github-app-token` digest matches the new full-SHA assertion.
- `git diff --check` passed.

PR #288 remains unmerged and will need fresh CI after this test change reaches `main`.
