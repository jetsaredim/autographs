# Publish operation assumptions after the third review

## Product scope

- One operator runs this site. Ordinary use is one publish at a time; a second tab or an operator script can still send another request. The publisher lock serializes execution, and the admin async path rejects another active async request.
- The reason for asynchronous admin publishing is the observed Cloudflare 524 after 125 seconds. The controller must finish Oracle recording after the browser request ends.
- Operation history is an in-memory convenience for polling. It is not a durable job queue. After controller restart, the operator checks the active release and pending edits before retrying.

## Result contract

| Point in lifecycle | Operation-owned facts | Reported result |
| --- | --- | --- |
| Accepted | Operation ID; no release ID yet | `running` |
| Publisher returns normally | Its returned release ID and publisher status | Continue until catalog recording completes |
| Catalog record commits | The same release ID and committed boundary | `succeeded` |
| Promotion succeeds but catalog recording fails | The returned release ID; no committed boundary | `incomplete` |
| Publisher fails before a proven promotion | Its own returned status, if terminal | `failed` or `unconfirmed` |
| Task panics or is aborted | Operation ID only; no trustworthy release ID | `unconfirmed` with `releaseId: null` |

The global `LocalPublisher::status()` describes the most recent publisher activity and cannot generally prove which accepted operation owns a release ID. There is one narrow exception: while `publish_authorized` still holds the publisher lock and after `publish_locked` has initialized its status, an ordinary `Err` branch may read that status as belonging to this operation. The lock excludes another publisher from replacing it before this branch returns. `begin_publish_boundary` errors occur before publisher initialization and carry no release ID. After the lock is released, and in any panic/abort fallback, global status is not valid identity evidence and must not supply a release ID. The admin operation endpoint is authoritative for the accepted ID.

General admin status reports the **latest authorized publish request**, whether synchronous or asynchronous. The route-owned operation store records that request's ID and status at admission. A completion updates general status only if its request ID is still the latest; an older queued request cannot overwrite a newer report. A later synchronous or asynchronous request therefore supersedes an earlier unconfirmed report without borrowing identity from global publisher status. A canceled synchronous handler also resolves to `unconfirmed` with no guessed release ID.

## Scope decision

Keep the existing bounded operation retention and synchronous API compatibility in PR #337. Do not add a durable queue, multi-admin coordination, or a Cloudflare timeout override. Fix the fallback provenance and its reporting path, then verify the actual long-publish/Oracle path after Release Please deployment. Optimize repeated media validation in a separate change.
