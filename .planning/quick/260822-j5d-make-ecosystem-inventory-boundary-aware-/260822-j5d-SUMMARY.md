---
quick_id: 260822-j5d
status: complete
completed: 2026-08-22T17:56:31Z
review_confirmation: https://github.com/jetsaredim/autographs/pull/214#issuecomment-5381830227
---

# Quick Task 260822-j5d Summary

Made the ecosystem inventory boundary-aware so repository configuration is no
longer flattened into a single, misleading VM expectation.

- Added multi-boundary classifications for VM runtime, VM smoke, build,
  deployment, CI, maintenance, infrastructure, local/test, and
  documentation-only variables.
- Restricted VM drift findings to like-for-like runtime templates/consumers and
  smoke consumers while retaining external-boundary variables as reviewable
  repository evidence.
- Separated controller inputs that are not deployed because they may use code
  defaults or optional overrides; the current report identifies only
  `OCI_REALM_DOMAIN` in this category.
- Classified retained VM env files as runtime, smoke, or other and grouped
  persistent secret-like keys by the file boundary where they exist.
- Replaced per-file key counts with actual pairwise key intersections.
- Preserved schema-1 comparison compatibility through source-path fallback and
  prevented generated inventory artifacts from feeding names back into later
  scans.

All 23 ecosystem spike tests, five repository-hygiene tests, Python compilation,
repository hygiene validation, and diff checks pass. Independent counter-review
found no actionable findings and reproduced the committed inventory from a
fresh scan apart from its expected `generated_at` timestamp.
