# Publish-history performance review

## Production baseline

The owner ran an image-adjustment incremental publish on 2026-11-14 using the
unoptimized controller. Release `65246df6-85db-4fce-96ba-a29d4fca9981`
completed successfully in 210,489 ms with 133 published items, 135 images,
270 derivatives, 2 newly generated derivatives, and 555 artifacts totaling
267,781,677 bytes. The route recorded the publish in 1,191 ms after the
publisher returned, and the admin pending-change count went from 2 to 0.
These figures come from the owner-supplied production controller logs for that
release; the complete log excerpt is retained in the project conversation.

The timing summary reported `reconciliation_ms=29248` and
`promotion_ms=156225`. The five full media-history verification passes consumed
153,235 ms, or 72.8% of publisher time, and read 8,437,628,058 bytes in
aggregate. Each pass had a distinct placement:

| Boundary | Verification duration | Release bytes read | Shared bytes read | Why it exists |
|---|---:|---:|---:|---|
| Publish start | 29,234 ms | 1,309,864,672 | 268,443,076 | Recover an interrupted pointer commit and reject existing history or media corruption. |
| Before commit | 30,524 ms | 1,309,864,672 | 268,443,076 | Detect drift while the candidate was built, before switching the public pointer. |
| After commit | 34,172 ms | 1,576,648,902 | 268,443,076 | Reconcile the commit intent, new pointer, and new release receipt. |
| Before pruning releases | 33,308 ms | 1,576,648,902 | 270,401,666 | Verify retained releases and shared media before deleting old release directories. |
| Before pruning shared media | 25,997 ms | 1,318,468,350 | 270,401,666 | Verify history again before removing unreferenced shared media. |

The two cleanup passes targeted by this change took 59,305 ms and reread
3,435,920,584 bytes. The earlier
[2026-11-11 incident record](../.planning/debug/publish-524-pending-edits.md)
observed the same five-pass pattern: 147.7 seconds of a 198.6-second publish.

## Integrity invariant and call inventory

The current pointer must agree with the committed history. Every retained
release and protected shared-media path must match its committed receipt before
promotion or destructive pruning. Emergency tombstones must be checked against
the current filesystem state, and a failed cleanup must leave the active
release intact with a retry warning.

`reconcile_history` performs the publish-start, pre-commit, and post-commit
passes. Candidate validation checks manifest contents and tombstones before
commit; commit verification reads the candidate receipt and checks tombstones
again. `post_promotion_cleanup` synchronizes current media, then
`prune_promoted_releases` previously called `load_history` before removing old
releases. `prune_shared_media` called `load_history` again immediately after
release pruning. These caused the fourth and fifth full verification passes.

## Deployed optimization

Carry the verified history returned by post-commit reconciliation through both
cleanup steps. Each pruning step checks that the current pointer and on-disk
history still match the snapshot and that no new commit intent appeared.
Shared-media pruning reads retained manifests, checks tombstones at each
decision, and retains time-protected media. Release pruning rechecks
each soon-to-be-removed release's media while copying it into shared storage;
current-media synchronization checks bytes before either pruning step. The
next publish still starts with a full history verification, so corruption
between publishes remains a blocking error. This removes two full scans from
the normal cleanup path while preserving the post-commit verification before
destructive cleanup.

The implementation landed through [PR #349](https://github.com/jetsaredim/autographs/pull/349)
as squash commit `86a93cb0de6182c8aaa4127f29579f40093ba654`.
Its reviewer recorded a clean review on the PR; all PR checks passed. Release
`v0.10.1` deployed successfully through
[workflow run 43664833424](https://github.com/jetsaredim/autographs/actions/runs/43664833424)
after a transient Release Please failure on the first attempt. The successful
rerun passed deployment health, frozen origin readiness, explicit unfreeze,
production status recording, and GitHub Release publication.

## Release and production measurement

The owner ran a comparable image-adjustment incremental publish on 2026-11-15
with the deployed controller. Release `19f572ef-0b15-4762-8c37-da55e2819e01`
completed successfully with 133 published items, 135 images, 270 derivatives,
2 newly generated derivatives, and 555 artifacts totaling 267,730,926 bytes.
The route recorded success 1,173 ms after the publisher returned; cleanup had
no warning and pruned one old release. These figures come from the owner-supplied
production controller log excerpt in the project conversation.

| Measure | Before | After | Change |
|---|---:|---:|---:|
| Publisher total `elapsed_ms` | 210,489 ms | 146,236 ms | 64,253 ms faster (30.5%) |
| `reconciliation_ms` | 29,248 ms | 28,757 ms | 491 ms faster |
| `promotion_ms` | 156,225 ms | 95,590 ms | 60,635 ms faster |
| Retention cleanup | 74,538 ms | 20,371 ms | 54,167 ms faster |
| Full history verification passes | 5 | 3 | 2 fewer |
| Full history verification time | 153,235 ms | 92,197 ms | 61,038 ms less |
| Full history verification bytes read | 8,437,628,058 | 5,028,396,333 | 3,409,231,725 fewer |

The remaining three verification passes took 28,750 ms at publish start,
29,897 ms before commit, and 33,550 ms after commit. Their respective
release/shared byte reads were 1,318,468,350/268,755,282,
1,318,468,350/268,755,282, and 1,585,193,787/268,755,282. Release pruning
took 10,029 ms, including 9,837 ms to verify and synchronize one old release;
shared-media pruning took 73 ms and deleted no files. The later release had
50,751 fewer artifact bytes, so the catalog and release sizes were comparable
but not identical. This is one before/after production pair, not a controlled
throughput benchmark.

The route success log proves the publish job was recorded. The post-publish
admin pending-change count is awaiting owner confirmation.
