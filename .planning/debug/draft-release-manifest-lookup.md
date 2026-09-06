---
status: resolved
trigger: Actions job 101524004247 failed while reconciling the v0.2.0 draft release manifest
created: 2026-09-06
updated: 2026-09-06
---

## Symptoms

- Expected behavior: merging the release-please Release PR creates a draft release, attaches the release manifest, deploys the selected release, and publishes the GitHub Release.
- Actual behavior: the controller image was published, but manifest reconciliation failed before deployment with `gh: Not Found (HTTP 404)`.
- Error messages: the `Reconcile release manifest` step exited after querying the release asset inventory.
- Timeline: first observed in Actions job `101524004247` for release `v0.2.0` on 2026-09-06.
- Reproduction: create a release-please draft release, then query `/repos/{owner}/{repo}/releases/tags/{tag}` for its assets.

## Current Focus

- hypothesis: the workflow uses GitHub's published-release-by-tag REST endpoint for a draft release that only draft-aware GitHub CLI release commands can resolve.
- test: replace both manifest asset-count lookups with `gh release view --json assets`, exercise the structural release workflow tests, and verify the live draft lookup succeeds.
- expecting: both automatic reconciliation and retry can count manifest assets on a draft release without a 404.
- next_action: complete.

## Evidence

- 2026-09-06: release-please created draft release ID `383655084` for `v0.2.0` with an `untagged-*` URL.
- 2026-09-06: `gh release view v0.2.0` resolved the draft release successfully.
- 2026-09-06: `gh api /repos/jetsaredim/autographs/releases/tags/v0.2.0` returned HTTP 404.
- 2026-09-06: `.github/workflows/deploy.yml` used the failing REST endpoint in both retry manifest loading and release manifest reconciliation.

## Resolution

- root_cause: the workflow validated draft releases with `gh release view`, but counted manifest assets through GitHub's published-release-by-tag REST endpoint, which returns HTTP 404 for drafts.
- fix: use `gh release view --json assets` for both automatic reconciliation and retry asset counts, and enforce the draft-aware lookup in the structural workflow contract.
- verification: the live `v0.2.0` draft lookup returned an asset count of zero; all 80 repository automation tests passed; repository hygiene validation and `git diff --check` passed. Local `actionlint` was unavailable, so GitHub CI remains the authoritative actionlint gate.
- files_changed: `.github/workflows/deploy.yml`, `scripts/test_release_workflow.py`, `scripts/test-fixtures/release-workflow/cases.json`
