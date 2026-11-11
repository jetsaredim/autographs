---
status: fixing
trigger: Incremental publish promoted a release, but Cloudflare returned 524 and the admin page still showed three pending edits.
created: 2026-11-11
updated: 2026-11-11
---

# Publish 524 with pending edits

## Evidence

- Release `e6c07a4f-826a-463f-ba51-6790357a8436` started at 00:33:24 UTC and the publisher logged `succeeded` after 198,595 ms. The adjusted image is visible publicly.
- Cloudflare returned 524 at 00:35:29 UTC, about 125 seconds after the request began. [Cloudflare documents a 125-second default proxy read timeout](https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-524/).
- The supplied logs stop at the publisher timing summary; there is no route-level `published static release` or `failed to record successful publish job` line. The refreshed admin page still shows three pending edits and the item as modified.
- `publish()` records the Oracle publish job only after `publisher.publish_locked()` returns. Thus media promotion can succeed while the edit-event publish boundary remains unrecorded. The exact production Oracle job state has not been queried.

## Fix

- The admin page requests asynchronous publishing with `Prefer: respond-async`. The controller returns 202 with an operation ID and completes publishing and Oracle recording in a spawned task independent of the browser connection.
- The admin page polls the operation through its final recording result. If the release is active but catalog recording fails, it displays `incomplete` and tells the operator to check logs before retrying. A transport error is shown as an unconfirmed outcome, not a failed release.
- Existing synchronous API callers keep their response contract. The Caddy deployment freeze gate still applies because both modes use the same POST paths.

## Performance follow-up

The same release spent about 148 seconds across five full media history verification passes, reading retained releases and shared media repeatedly. These passes occur at publish start, before commit, after commit, and during both pruning steps. Review the shared safety invariant and mutation boundaries before reusing a verification result or removing a pass; keep this change separate from the response and recording fix.

## Verification

- Focused async integration test: dropping the initial response does not stop publishing or leave memory-catalog edits pending.
- Static admin behavior test: 202 operation polling reports success; a 524 transport response never displays raw Cloudflare HTML or calls the outcome failed.
- Production verification after deployment: repeat an incremental publish and confirm a terminal operation result, a route-level `published static release` log, and the expected pending edit count.
