---
quick_id: 260906-h0x
status: complete
implementation_commit: 58a26f7
completed: 2026-09-06
---

# Quick Task 260906-h0x Summary

Added a dedicated pull request workflow that validates the PR title and every non-merge commit subject against the Conventional Commit types configured for release-please.

## Delivered

- Added `.github/workflows/validate-release-please-inputs.yml` with read-only permissions and PR title/update triggers.
- Added `scripts/validate_release_please_inputs.py`, which reads allowed types from `release-please-config.json`, validates the event title, walks the PR's base/head commit range, and emits GitHub error annotations.
- Added focused validator tests and included them in the existing CI automation suite.
- Skipped true merge commits while validating every non-merge commit, because integration merges are not release changelog inputs.

## Verification

- `python3 -m unittest scripts/test_validate_release_please_inputs.py` — 11 tests passed.
- Full repository automation suite — 79 tests passed.
- `python3 scripts/validate_repo_hygiene.py` — passed.
- All workflow YAML files loaded successfully with PyYAML.
- Local end-to-end validation of `origin/main..HEAD` with a conventional PR title — passed.
- Deliberately malformed PR title — rejected with exit code 1 and a GitHub error annotation.

Implementation commit: `58a26f7`

## Follow-up

- Renamed the standalone workflow and job so the PR checks list displays the concise, repository-consistent label `CI / Conventional commits` without rerunning the full CI suite on title edits. Follow-up commit: `bb967df`.
