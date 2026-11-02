---
status: ready_to_resume
trigger: Third PR #301 review found an in-scope Warning after the v0.5.0 retry failure.
updated: 2026-11-02
---

# Publish freeze convergence reassessment

## Finding lineage

1. The first v0.5.0 deployment failed before cutover because its tagged preflight required an operator-installed publish gate.
2. The first retry passed that gate, then the tagged audit counted 242 media paths from retained older release manifests as unresolved even though those paths were also in the active release. PR #301 fixes that audit behavior for future releases. Four older directories were archived on the VM without changing the current release or media; the next retry passed preflight.
3. The next retry deployed the reviewed controller and passed deployment health, then failed the tagged unfreeze assertion at `deploy/ansible/playbooks/publish-unfreeze.yml:229`: it requires `release_id` while the Rust `PublishManifest` serializes `releaseId` (`controller/src/contracts.rs`). This is an independent post-deployment finding.
4. PR #301 commit `f4aeece` changed the assertion to `releaseId`. Its third review found the end-to-end fixture still writes `release_id` in `MediaAuditFixture.add_release_media` and `promote_release` (`deploy/ansible/tests/test_publish_freeze_deploy.py`). This is a test weakness exposed by the production contract fix. The finding is posted on PR #301: https://github.com/jetsaredim/autographs/pull/301#issuecomment-6567281396.
5. The first post-reassessment review found an incomplete implementation of the planned post-unfreeze retry path: the new completion workflow originally required a frozen gate on every dispatch, so a status or publication failure after unfreeze could not be finalized. The finding is posted inline: https://github.com/jetsaredim/autographs/pull/301#discussion_r4438360629. The workflow now has a `finalize` operation that requires a prior successful unfreeze step, matching additive attempt asset, healthy site and removed gate, then repeats only the GitHub bookkeeping steps.
6. The next review found an incomplete attempt binding in `finalize`: the run-level jobs endpoint returns the latest attempt, which could be paired with an earlier attempt asset. The finding is posted inline: https://github.com/jetsaredim/autographs/pull/301#discussion_r4438374808. The workflow now queries attempt-specific run and jobs endpoints and requires the matching workflow path and attempt number before trusting the prior unfreeze result.
7. The first live completion attempt passed the corrected manifest assertion but failed at an obsolete authenticated status proof: `/admin/api/publish/status` requires an admin session, while the playbook used an operator bearer token. The live unauthenticated route returns 401, matching `controller/src/routes.rs::authorize_admin_session`. The follow-up correction checks that the route remains private and updates the fixture to reject operator token access. The v0.5.0 completion workflow now proves both audited playbook changes against the immutable tag before VM action.

## Shared invariants and consumers

- The Rust publisher produces a camelCase public manifest containing `releaseId`; the live `/manifest.json` confirms it. The unfreeze playbook must verify this identity against the current release target, not merely field presence.
- The audit helper reads release manifests and currently accepts either `releaseId` or `release_id`. The test fixture produces snake_case; both direct writers in that fixture must produce the production shape.
- The unfreeze playbook reads the live Caddy manifest, freeze record, static pointer, commit history, marker inventory, running image, controller health, and origin routes. Its verify-only and unfreeze invocations share the same manifest assertion.
- The deploy workflow checks out the immutable release tag for retries. Its `retry-recovery-audit.json` release asset is reconciled byte-for-byte on later attempts. The v0.5.0 asset records `recoveryApplied=false` and no approved files. Any corrective tagged-playbook execution needs explicit, separate provenance; overwriting the existing audit would violate the contract.
- The release completion gate requires deployment health and successful unfreeze before committing `.release-status.json` and publishing the draft GitHub Release. The ordinary main push release-please job blocks while that draft remains unresolved.

## Assumptions audit

- Confirmed: the current VM release pointer still targets `a0805fb9-4570-4ad1-8f75-8c06a0e44f7a`; only that directory remains in the live releases set after archival.
- Confirmed: the latest retry passed frozen preflight, full deployment, and deployment health, then failed frozen origin readiness at the manifest field assertion. The public manifest and controller health return HTTP 200; unauthenticated publish POST returns HTTP 503 while frozen.
- Rejected: merging PR #301 alone changes the already-tagged v0.5.0 retry playbook. Retry checks out the tag.
- Rejected: a new v0.5.1 release can proceed while v0.5.0 remains draft. The main push preflight blocks release-please on unresolved drafts.
- Unverified: later unfreeze and release completion checks will pass after the manifest assertion is corrected. They must run before publication.

## Failure matrix

| Path | Expected behavior | Current result |
| --- | --- | --- |
| v0.5.0 tagged retry | Pass frozen origin readiness | Fails at `release_id` assertion |
| PR #301 with production manifest | Accept `releaseId` | Assertion corrected; live retry cannot use it yet |
| PR #301 with existing end-to-end fixture | Verify and unfreeze | Fixture uses `release_id`, so it fails the new assertion |
| Main push with unresolved v0.5.0 draft | Advance release-please | Explicitly blocked by `preflight-drafts` |
| Publish while frozen | Return 503 | Confirmed 503 |

## Revised plan requirements

1. Align both fixture manifest writers with the Rust `PublishManifest` shape. Add a meaningful assertion that a missing or mismatched active release identity is rejected, while the production shape passes verify-only and unfreeze paths.
2. Add a dedicated, manually dispatched `v0.5.0` completion workflow on `main`, separate from the immutable tagged retry. It must refuse any other tag, require the existing draft and exact tag commit, download and validate the existing release manifest and retry audit without changing either, and prove the selected main `publish-unfreeze.yml` differs from the tagged playbook only by the `release_id` to `releaseId` assertion and the pinned replacement of the obsolete operator-token status proof. It must record the tag SHA, both playbook hashes, main automation SHA, original retry audit hash, and workflow run identity in a new attempt-specific release asset before acting on the VM.
3. The dedicated workflow must run the corrected playbook in verify-only mode and then explicit unfreeze mode against the frozen production VM. The playbook itself checks the freeze record, current pointer, manifest/history/markers, reviewed controller image, health, origin routes, and post-unfreeze state. The workflow must then commit `.release-status.json` from the existing release manifest using the original immutable retry audit and publish the existing draft release. It must never make a new tag or controller image.
4. Exact order: verify draft/tag/assets and patched-playbook delta; upload additive attempt audit; verify-only while frozen; unfreeze with postchecks; commit status; publish draft. The gate is removed at unfreeze, before the status commit and publication. If a later GitHub status or publication step fails, do not rerun unfreeze or the full deployment. Reconcile only those GitHub steps after verifying the controller identity, public manifest, origin routes, absent freeze record/gate, and additive attempt audit; retain the draft until that finalization succeeds.
5. Review the revised plan and assumptions before resuming code changes. Review PR #301 again after implementation; every actionable finding must be posted on the PR.

## Resume criteria

Status may change to `ready_to_resume` only after this reassessment's assumptions and implementation plan have been revised and reviewed, with both evidence references recorded here. A status-only change is invalid.

- Revised assumptions and implementation plan evidence: this document, Finding lineage through Revised plan requirements, updated 2026-11-02.
- Independent review evidence: https://github.com/jetsaredim/autographs/pull/301#issuecomment-6567327000. The reviewer found no remaining plan blocker after the dedicated completion path and post-unfreeze failure handling were specified.
