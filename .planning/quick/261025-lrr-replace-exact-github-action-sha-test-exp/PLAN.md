---
id: 261025-lrr
status: implementation_complete
---

# Keep release action pin checks compatible with Renovate

1. Replace exact commit assertions for the privileged GitHub Actions with action identity plus full commit SHA assertions.
2. Run the release workflow test suite and verify Renovate PR #288's new digest satisfies the check.
3. Commit the change, record GSD state, and open a ready-for-review PR. Leave all PRs unmerged.
