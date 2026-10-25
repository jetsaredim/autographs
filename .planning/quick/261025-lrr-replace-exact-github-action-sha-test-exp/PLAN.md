---
id: 261025-lrr
status: implementation_complete
---

# Keep release workflow tests focused on release behavior

1. Remove assertions about Renovate-managed GitHub Action pins and versions while retaining token scope and release flow checks.
2. Run the release workflow test suite and verify Renovate PR #288 changes only the action digest.
3. Commit the change, record GSD state, and open a ready-for-review PR. Leave all PRs unmerged.
