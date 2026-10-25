---
status: issues_found
blocker_count: 3
warning_count: 1
info_count: 0
reviewed_assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-ASSUMPTIONS.md
reviewed_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-REVISED-PLAN.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
reviewed_head: 71f43f5b38d97077874d7e7edfdeb7f9feed8c42
review_cycle: 1
---

# PR #281 Workbench Convergence Plan Review

## Verdict

**Issues found.** The reassessment correctly narrows the convergence scope and
identifies the shared persistence, browser-ownership, and capability invariants,
but the revised plan does not yet give an executor a deterministic commit-order
oracle, an abort-safe Save lifetime, or executable rollback tests. It also leaves
one capability-bearing URL sibling path implicit.

This review **does not authorize** changing convergence to `ready_to_resume`.
Amend the assumptions and plan as specified below, then obtain a new independent
plan review. The reviewed artifact digests were:

- assumptions: `937cdf2ec3ab547b34707f5d828b5ece90728ade9e14f8b522f8902944aae41d`
- revised plan: `61879b7d6a77d0669613229a96865143603e2ebbf816a054e4936cc6da1f410e`
- source review: `0428fe9140d31aec6aa7e69d8cf111e085ba12e3d7859248d0cba2192dce63e9`

## Goal-backward closure

| Finding | Current closure | Result |
| --- | --- | --- |
| WR-01 `incomplete_fix` | Per-image locking, unique temporary files, browser single-flight, stale-publication suppression, and full regression scope are all named. Commit ordering is not observably deterministic, Save can still settle on client abort before its server mutation settles, and the promised cleanup/rollback behavior has no deterministic failure tests. | Not closed |
| WR-06 `sibling_path_miss` | The emitted capability URL is made the primary launch contract and a positive/negative smoke is required. The existing `?image=N` reopen instruction is not explicitly corrected to retain `cap`. | Not closed |

## Blockers

### BL-01 — The proposed final-winner oracle observes client task completion, not server commit order

The assumptions define commit order as authoritative, but then infer that “the
last successful response” identifies it (`ASSUMPTIONS.md:81-84`, failure matrix
line 102). The plan repeats that inference (`REVISED-PLAN.md:73-76`) and asks the
test to collect response completion order (`REVISED-PLAN.md:110-115`). Atomic
rename and response construction occur inside the handler; the mutex is released
before Axum/Tower delivers a response to the caller. Two response futures can
therefore be observed in an order different from the serialized rename order.
Repeating the race “enough times” is probabilistic and does not repair the oracle.
The plan's own stop condition at lines 190-191 is already met.

Exact amendment:

1. Replace assumption A-04 and the matching failure-matrix/acceptance wording
   with a server-owned per-image commit ordinal. Store `next_commit_ordinal` in
   the state protected by each image's mutex; increment it only after a successful
   rename, using checked arithmetic.
2. Return the committed ordinal on every successful PUT in a fixed
   `x-workbench-fixture-commit` response header. Define the durable winner as the
   successful response with the highest server-issued ordinal, not the response
   future observed last by a client. A GET must match that response's adjustment.
3. Add an optional `#[cfg(test)]` pre-lock `tokio::sync::Barrier` to the test state.
   The two router requests must both pass parse/validation and rendezvous at that
   barrier before lock acquisition. The test must dispatch exactly those two
   requests, require two non-500 successes with distinct ordinals, and compare
   GET with the response carrying the higher ordinal. Do not use sleeps,
   repetition-as-probability, wall clocks, or response completion order.
4. Update Task 1, the frozen server contract, acceptance criteria, and stop
   conditions to name this exact oracle and seam.

This amendment preserves the intended per-image protocol while making its
ordering independently observable and the test deterministic.

### BL-02 — The Save lifetime still permits an aborted request to overlap the next explicit Save

The current Save uses `ownedApi` (`app.js:740-758`). That helper registers an
`AbortController`, and every draft edit/navigation invalidation aborts all
registered controllers (`app.js:48-80`, `409-425`). The revised plan says edits
continue to invalidate ownership, always clears single-flight state after abort,
and assumes a later Save dispatched after browser settlement is a later server
commit (`REVISED-PLAN.md:80-93`, acceptance lines 170-173). The reassessment
correctly states that abort is not rollback (`ASSUMPTIONS.md:48-50`). Those two
contracts conflict: an aborted fetch can settle in the browser while its Axum
handler continues, allowing the newly enabled Save to overlap it. Per-image
locking prevents corruption but does not guarantee that the later browser click
acquires the lock last.

The requested Node test is also not implementable against the real Save protocol
without inventing a test architecture: `app.test.cjs` imports only three pure
helpers, and `app.js:1755-1756` does not export Save behavior. A source-text test
would repeat the test weakness called out in rejected assumption R-06.

Exact amendment:

1. State that Save persistence is deliberately not registered with
   `operationAuthority.controller` and is not sent through `ownedApi`. Use a
   dedicated unabortable persistence call for an accepted Save. Edit/navigation
   invalidation must still make the captured owner stale, but it must not settle
   the Save promise early by aborting the request.
2. Keep `savePending` true until the full HTTP response and JSON body settle.
   Only then may `finally` re-enable Save. Remove ordinary client abort from the
   same-page re-enable acceptance path; page unload/transport loss remains an
   explicitly documented indeterminate-delivery case, not a later-wins promise.
3. Extract and export one DOM-independent
   `createSaveCoordinator({ captureSnapshot, persist, owns, publish, setPending,
   reportError })`. Production `saveFixture` must call this same coordinator.
   It must synchronously reject a second activation, preserve the captured
   snapshot, publish only when `owns` remains true, and clear pending only after
   `persist` settles.
4. Make the deferred Node test invoke that exported coordinator. Assert exactly
   one persistence call from two activations, synchronous pending/disabled state,
   no publish/status/log callback after owner invalidation, pending cleared on
   success and error, and a subsequent explicit invocation persisting the new
   snapshot only after the first promise settles.
5. Amend A-02/A-07, the browser contract, Tasks 1-2, acceptance, and rollback
   text to use this precise lifetime.

### BL-03 — Atomic cleanup and rollback are acceptance claims without a frozen I/O design or failure tests

The server contract promises bounded `create_new` retries and exact-temp cleanup
while preserving the old destination (`REVISED-PLAN.md:65-72`, acceptance
lines 168-169), but Task 1 adds no collision, partial-write, or rename-failure
test. Task 2 leaves retry count, write API, and failure injection unspecified.
The current Tokio feature set has `fs` but not `io-util` (`Cargo.toml:15`), while
the scope makes any Cargo change a stop/replan event (`REVISED-PLAN.md:45-47`). An
executor must therefore choose between changing features, blocking synchronous
I/O, reopening a reserved path, or revising the plan. That is a material design
choice at the mutation and rollback boundary.

Exact amendment:

1. Add `workbench/Cargo.toml` to the allowed change set solely to add Tokio's
   `io-util` feature; no new crate is permitted and `Cargo.lock` should remain
   unchanged. Freeze the implementation on `tokio::fs::OpenOptions` with
   `create_new(true)` plus `AsyncWriteExt::write_all`, flush/shutdown/drop, then
   same-directory rename.
2. Freeze the collision retry limit at eight candidates. Extract the replacement
   helper with an injectable temp-name generator and a test-only failpoint enum:
   `None`, `AfterCreate`, `AfterPartialWrite`, and `BeforeRename`. Production uses
   random suffixes and `None`; tests use a fixed candidate sequence and injected
   `io::Error`s.
3. Add deterministic helper tests proving: a pre-created collision is never
   opened/truncated and advances to the next candidate; eight collisions fail
   closed; partial-write and pre-rename failures remove only their own temp;
   both failures leave an existing destination byte-for-byte unchanged and
   readable; and success leaves no temp file.
4. Keep the router concurrency test from BL-01 for protocol integration, and add
   these helper tests to Task 1/acceptance so rollback is executable rather than
   inferred.

## Warning

### WR-01 — The image-deep-link instruction can still discard the capability

The README says a specific image can be reopened with bare `?image=N`
(`README.md:66-67`). The plan broadly says to update every instruction, but Task
3 only explicitly replaces the bare launch URL (`REVISED-PLAN.md:139-149`). This
is the same documentation consumer class as WR-06, so leaving it implicit risks
another sibling-path miss.

Exact amendment: add an explicit Task 3 item replacing that text with
`?cap=<emitted-token>&image=N`, or instruct the operator to use the UI-updated URL
while retaining the emitted `cap` parameter. The smoke must deep-link to an image
with the capability retained and confirm that dropping `cap` yields `403`. Keep
the rule against recording a live token.

## Coverage that is already sufficient

- Scope is correctly limited to the disposable workbench and its phase evidence;
  production controller persistence is read-only regression scope.
- Parse-before-lock, per-image rather than global serialization, stable source
  identity, same-directory unique temporary files, and rename publication are the
  right server boundaries.
- The stale snapshot is honestly allowed to persist while stale UI publication is
  forbidden.
- The emitted capability is correctly treated as ephemeral sensitive output, and
  Host/Origin/no-store protections remain non-negotiable.
- The full verification and fresh-review gates are proportionate, and the stated
  rollback is to stop and revise rather than begin another point-fix loop.

## Resume criteria

Revise both the assumptions and implementation plan to incorporate every exact
amendment above, update their evidence pointers without changing convergence
status, and run a new independent plan review. Only a subsequent review with zero
Blockers and zero Warnings may authorize recording its path in the convergence
artifacts and changing status to `ready_to_resume`.
