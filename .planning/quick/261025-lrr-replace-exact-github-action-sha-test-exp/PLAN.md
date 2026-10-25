---
id: 261025-lrr
status: implementation_complete
---

# Test managed dependencies through behavior

1. Remove assertions about Renovate-managed GitHub Action pins and versions while retaining token scope and release flow checks.
2. Remove other exact managed dependency version assertions found in repository tests, retain compatibility and behavior checks, and record the testing rule in project guidance.
3. Run focused Python and Rust tests, then update the ready-for-review PR. Leave all PRs unmerged.
