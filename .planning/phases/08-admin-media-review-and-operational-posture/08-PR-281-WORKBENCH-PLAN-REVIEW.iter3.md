---
status: passed
blocker_count: 0
warning_count: 0
info_count: 0
reviewed_assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-ASSUMPTIONS.md
reviewed_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-REVISED-PLAN.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
reviewed_head: 71f43f5b38d97077874d7e7edfdeb7f9feed8c42
review_cycle: 2
---

# PR #281 Workbench Convergence Plan Review — Cycle 2

## Verdict

**Passed.** The amended assumption record and revised implementation plan fully
incorporate cycle 1's BL-01, BL-02, BL-03, and WR-01 corrections. They close the
goal-backward design gaps for source-review findings WR-01 `incomplete_fix` and
WR-06 `sibling_path_miss` without introducing a new endpoint, production
persistence change, dependency, or material executor choice.

This file is the independently reviewed plan evidence that **authorizes recording
it in the convergence artifacts and changing convergence from
`reassessment_required` to `ready_to_resume`**. Execution remains subject to the
plan's RED-first tests, stop conditions, full verification, PR audit trail, and
fresh deep source review.

Reviewed artifact digests:

- assumptions: `626a5bf7c91306893a3371bb5b99d9813b1b01a33464988ffda28520c47bc8fa`
- revised plan: `400038c9cbfe42b7d1cd94bd3338686ff790455955d0aa28f921bd9fd97c0b5b`
- source review: `0428fe9140d31aec6aa7e69d8cf111e085ba12e3d7859248d0cba2192dce63e9`

## Cycle 1 amendment closure

| Prior item | Amended evidence | Cycle 2 result |
| --- | --- | --- |
| BL-01: client response completion was not a commit-order oracle | Assumption A-04 and the shared invariant now define a checked server-owned per-image ordinal issued after successful rename. The fixed response header is `x-workbench-fixture-commit`; the deterministic router test uses a two-party pre-lock barrier and compares GET with the higher-ordinal response without sleeps, repetition, clocks, or future-completion order. | Closed |
| BL-02: abort could settle Save before the server mutation | A-02/A-07 and the browser contract require a dedicated Save request outside `ownedApi` and operation-authority abort controllers. Pending begins synchronously and lasts through response-body settlement. The exported `createSaveCoordinator` is the production/test seam; its deferred test covers duplicate suppression, stale non-publication, success/error cleanup, and a later explicit snapshot. | Closed |
| BL-03: atomic cleanup/rollback lacked a frozen design and tests | Scope permits only Tokio's existing-crate `io-util` feature and requires a byte-unchanged lockfile. The plan freezes `OpenOptions(create_new)`, `AsyncWriteExt`, same-directory rename, eight collision attempts, an injectable candidate generator, exact failpoints, and deterministic collision/partial-write/pre-rename/success cleanup tests. | Closed |
| WR-01: image deep links could discard the capability | Task 3 explicitly replaces bare `?image=N`, retains `cap` through `&image=N` or the capability-preserving UI URL, and smokes both the authorized deep link and missing-cap `403`. The current UI's `selectImage` history update already preserves both parameters, so the documented alternative has a real consumer path. | Closed |

## Goal-backward verification

### WR-01 — fixture write and browser Save invariant

The planned protocol now covers every direct boundary identified by the source
review:

- **Producers:** browser Save captures one canonical snapshot; duplicate
  activation is synchronously suppressed. Direct API callers remain supported
  by server-side serialization rather than relying on browser ownership.
- **Mutation/order boundary:** request validation precedes the per-image mutex;
  the mutex owns the ordinal and complete replacement boundary. A successful
  rename is the publication event, and the returned server ordinal makes its
  order observable independently of response scheduling.
- **Atomicity and cleanup:** every request exclusively creates its own
  same-directory temp, writes and closes a complete document, then renames. All
  specified post-create failures remove only that temp and preserve the prior
  destination. Collision exhaustion fails closed.
- **Consumers/recovery:** immediate GET is compared with the highest successful
  per-image ordinal. Existing fixture list/reload, stable source identity,
  restart/rescan, and manual workspace recovery remain covered by full
  regression scope.
- **Browser publication/reporting:** edits/navigation invalidate ownership but
  do not abort persistence. A stale settlement cannot mutate the draft,
  selection marker, success status, or event log. The pending control state is
  cleared only after the persistence promise and body settle.
- **Tests:** the barrier supplies deterministic concurrent arrival; the ordinal
  supplies a server-order oracle; injected candidate/failure seams prove
  rollback; the same exported coordinator used by production supplies a
  DOM-independent behavioral browser test. Source-text assertions and timing
  races are not accepted as proof.

The contracts are mutually constraining rather than optional. In particular,
the checked next ordinal must be available before entering a fallible publish
sequence while becoming authoritative only after rename, because the separate
error invariant forbids a post-publication error from changing the destination.
Ordinal comparisons are per image; the plan separately and explicitly keeps
different-image writes independent. These consequences leave no reason to
introduce a global lock, client clock, client sequence, or implicit save queue.

### WR-06 — capability-aware operator contract

The plan covers the capability producer and all operator consumers: the server's
emitted URL remains authoritative; run instructions require copying it exactly;
API access from the bare URL remains a documented `403`; the token is treated as
sensitive; image deep links retain it; and the smoke traverses list, source,
save, reload, deep-link, and negative missing-cap paths. Host, Origin, token, and
no-store protections are preserved rather than weakened.

## Executor-choice audit

- **Ordering:** fixed as per-image mutex plus checked server ordinal and response
  header. Client observation order, clocks, and page-local counters are excluded.
- **Temporary files:** fixed as eight exclusive, same-directory candidates with
  a random production generator and deterministic injected tests.
- **Write/commit API:** fixed as Tokio `OpenOptions`/`AsyncWriteExt`, close, then
  rename; the manifest allowance is exact and introduces no new crate.
- **Cleanup/rollback:** fixed failpoints and byte-equality assertions define both
  ownership and recovery. Cleanup failure remains best effort while unique
  creation prevents stale-temp reuse.
- **Browser concurrency:** fixed as one unabortable same-page persistence promise,
  one exported coordinator, no implicit queue, and ownership-gated publication.
- **Operator contract:** fixed as exact emitted URL, capability-retaining image
  deep link, sensitive-output guidance, and positive/negative smoke.

No material design decision remains for the executor. Ordinary implementation
details such as Rust helper naming or local struct layout do not alter the frozen
protocol or its test oracles.

## Scope, rollback, and resume authorization

The writable source scope is limited to the workbench server, its existing Tokio
feature declaration, browser Save implementation/tests, and the workbench
README. Production controller/image-processing paths remain read-only regression
scope. `Cargo.lock` is required to remain byte-identical, and production
controller persistence is an explicit stop boundary.

The rollback gates are adequate: any inability to prove ordinal order,
temp-cleanup ownership, prior-destination preservation, unabortable Save
settlement, or capability smoke behavior requires plan revision. Any fresh
recurring, sibling-path, regression, test-weakness, or scope-expansion
Critical/Warning returns to reassessment rather than another point fix.

With zero Blockers and zero Warnings in cycle 2, the orchestrator may record this
review as the independent evidence in the assumptions, revised plan, and
`08-REVIEW.md`, set convergence to `ready_to_resume`, and dispatch the bounded
implementation plan.
