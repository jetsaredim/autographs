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

## Proposed optimization

Carry the verified history returned by post-commit reconciliation through both
cleanup steps. Each pruning step checks that the current pointer and on-disk
history still match the snapshot and that no new commit intent appeared.
Shared-media pruning reads retained manifests, checks tombstones
at each decision, and retains time-protected media. Release pruning rechecks
each soon-to-be-removed release's media while copying it into shared storage;
current-media synchronization checks bytes before either pruning step. The
next publish still starts with a full history verification, so corruption
between publishes remains a blocking error. This removes two full scans from
the normal cleanup path while preserving the post-commit verification before
destructive cleanup.

The current baseline suggests a saving near the fourth and fifth passes'
combined 59.3 seconds,
but no production speedup is claimed until a comparable post-deploy publish
records its total duration, `reconciliation_ms`, `promotion_ms`, verification
pass count, bytes read, and pending-change reconciliation result.

## Release and production measurement

Pending reviewed PR, deployment, and post-deploy publish. Compare against the
2026-11-14 baseline using a similar item/image count and release size. Record
the same timing fields, all verification-pass byte counters, and the admin
pending-change count before and after publishing.
