# Revised publish assumptions and plan review

**Reviewed:** 2026-11-11  
**Verdict:** PASS  
**Scope:** Revised assumptions and plan, round-3 convergence failure matrix, and current publish route/publisher behavior.

The revised assumptions and plan satisfy the round-3 resume criteria. They define an operation-owned panic/abort fallback with no release ID, terminalization and active-gate release, bounded per-ID polling, and route-owned latest-request status shared by synchronous and asynchronous callers. The request-ID check prevents an older queued completion from overwriting newer general status. The plan includes deterministic prior-release, queued-sync, post-promotion catalog-record failure, and supersession tests. This is proportionate for one operator and does not add a durable queue.

The ordinary error provenance is now explicit: `publish_authorized` may read publisher status while holding the publish lock, after `publish_locked` initialized this operation's release ID (`controller/src/publisher.rs:931-950`; `controller/src/routes.rs:2429-2450`). Pre-publisher errors, panic/abort fallback, and reads after lock release cannot use that global status as identity evidence. The plan extends the publisher-error test to check this boundary. Its broad invariant and verification wording should be read with this specific locked-error exception; aligning those two sentences during implementation would improve clarity, but does not block the reviewed plan.

The convergence artifact may now record `.planning/debug/publish-524-revised-assumptions.md` and `.planning/debug/publish-524-revised-plan.md` as reviewed evidence and change to `ready_to_resume`. Coder work may proceed after that evidence update. This is a plan review, not a clean code review of the forthcoming implementation.
