# Revised plan: operation-owned exceptional outcomes

**Trigger:** Third review of PR #337 found that the panic/abort fallback borrowed a release ID from global publisher status. This plan narrows the correction to provenance of an exceptional result and how the admin page reports it.

## Invariant

An operation result may include a release ID only when that ID came from that operation's own publisher return or from its publisher status while the same operation still holds the publish lock after publisher initialization. If the task exits before producing a typed outcome, report `unconfirmed` with no release ID. This result must remain queryable, and the active gate must be released.

## Steps

1. In `controller/src/routes.rs`, remove `LocalPublisher` from `PublishOperationFinalizer`. On panic/abort, finalize from an operation-local fallback status with `release_id: None`. Keep normal completion using the typed `PublishRouteOutcome`. Do not read global publisher status for fallback identity.
2. Use the existing `PublishOperations` mutex as one route-owned general-status authority. Store `latest_request_id` and `latest_status` when an authorized synchronous or asynchronous POST is admitted. Reuse one finalizer for both paths; it writes a terminal status to general reporting only when its request ID still matches `latest_request_id`. The async path also keeps its bounded per-ID polling result and active gate. `current_publish_status` reads `latest_status` (falling back to the publisher's idle/current status only before any route-owned request). A later sync or async admission supersedes the older report immediately; an older completion cannot overwrite it. A canceled synchronous handler becomes `unconfirmed` with no guessed release ID. Keep the synchronous response shape and Caddy paths.
   The ordinary `publish_locked` `Err` branch may use `state.publisher.status()` only while its publish lock is still held, after that operation initialized the publisher status. This is operation-owned evidence under the lock. Errors before `publish_locked` starts and panic/abort fallback carry no release ID. No status read after releasing the lock may establish identity.
3. Add deterministic tests in `controller/src/routes.rs` for (a) a prior successful release followed by an async panic, and (b) a queued synchronous publish changing global status during exceptional unwind. Both fallback operation results must have `releaseId: null`, state `unconfirmed`, and a released active gate. Add a fault-injected catalog-record test where static promotion succeeds but Oracle-equivalent recording fails; assert `incomplete`, this operation's release ID, and pending edits remaining. Test that a later synchronous completion supersedes an older unconfirmed report while the old async operation remains queryable. Keep normal success, abort, and second-operation retention tests. Extend the publisher-error test to assert that a release ID observed in the ordinary `Err` branch belongs to the locked operation, or is absent when failure preceded publisher initialization.
4. Update `docs/controller-walkthrough.md` and the debug note to state that an unconfirmed result intentionally omits the release ID. The operator inspects the active release and pending edits before retrying.

## Verification

- `cargo fmt --check`, focused route and admin workflow tests, the static admin behavior harness, `cargo check --features production-persistence`, and PR CI pass.
- Review the producer and consumer inventory in `publish-524-pr-review-round3.md` against the changed code. Confirm panic/abort fallback never reads global publisher status for identity, the ordinary error branch reads it only under its own publish lock, and no admin report substitutes another release ID.
- After a Release Please deployment, run one incremental publish; expect a prompt 202, terminal `succeeded` only after the route logs successful recording, and the expected pending-edit count to clear.

## Review gate

Review this plan and `publish-524-revised-assumptions.md` before coder work. Record both reviewed evidence paths in `publish-524-pr-review-round3.md`, then change its convergence status to `ready_to_resume`. If the review rejects the reporting rule in step 2, revise the plan before coding.
