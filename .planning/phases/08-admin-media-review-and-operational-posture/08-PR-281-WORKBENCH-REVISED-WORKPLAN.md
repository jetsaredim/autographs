---
phase: 08-admin-media-review-and-operational-posture
pr: 281
head: 71f43f5b38d97077874d7e7edfdeb7f9feed8c42
status: ready_to_resume
scope: single-operator-workbench-closeout
assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-ASSUMPTIONS.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
reassessment_iteration: 2
checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-PLAN-REVIEW.md
checker_status: passed_cycle_3
---

# PR #281 Workbench Convergence Revised Plan — Iteration 2

## Objective

Close PR #281 proportionately for a single-operator local workbench: prevent
duplicate browser saves, persist fixture JSON through the existing `tempfile`
library, suppress stale UI publication, and correct capability-bearing run
instructions. Concurrent external writers remain unsupported and multi-user
productization is explicitly blocked pending a version/conflict design.

## Exact source scope

- `workbench/Cargo.toml`: move `tempfile = "3.27.0"` from dev dependencies to
  dependencies; no new crate or feature.
- `workbench/src/main.rs`: replace fixed `.json.tmp` with `NamedTempFile`.
- `workbench/static/app.js` and `app.test.cjs`: one in-flight Save and tests.
- spike `README.md`: capability-aware run, deep-link, smoke, and supported-scope
  guidance.

`Cargo.lock` must remain byte-identical because the root package already lists
`tempfile`. Production controller and image-processing code are read-only.

## Frozen implementation contract

### Fixture replacement

1. Parse/validate the adjustment and serialize the complete fixture document.
2. Create `NamedTempFile::new_in(state.workspace)`; never derive/expose a temp
   filename.
3. Use `std::io::Write::write_all`, then `flush`. This small synchronous local
   write is accepted for the single-user tool.
4. Call `persist(destination)` for same-directory publication. Map create,
   write, flush, and persist failures to existing non-leaking API errors.
5. Pre-persist failure relies on drop cleanup. Persist failure drops the
   returned temporary file from `PersistError`.
6. Add no mutex, ordinal, client sequence, barrier, failpoint, or custom random
   naming. Concurrent external PUT ordering is unspecified and unsupported.

### Browser Save

1. Extract/export a DOM-independent coordinator used by production and tests.
   It captures owner/snapshot, synchronously marks pending, rejects a second
   activation, awaits persistence, publishes only for the current owner,
   reports a current-owner error, and clears pending after settlement.
2. Persistence uses `api`, not `ownedApi`; draft invalidation does not abort it.
3. Pending disables Save. Stale success changes no adjustment, option label,
   status, or event log. There is no implicit queue or autosave.

### Operator and support contract

1. Direct operators to the exact printed `?cap=...` URL.
2. Document missing-cap API `403` and sensitive-token handling.
3. Replace bare `?image=N` with the UI URL or
   `?cap=<emitted-token>&image=N`.
4. State that the tool supports one operator/session. Concurrent external
   writers and multi-user use are unsupported.
5. Link the productization blocker: multi-user support requires version/ETag
   conflicts, actor/audit identity, serialized or CAS commits, conflict UX, and
   deterministic concurrency/recovery tests.

## Ordered tasks

1. **RED:** Add deferred coordinator tests for duplicate suppression,
   synchronous pending, stale non-publication, success/error cleanup, and a
   later new snapshot. Add focused Rust coverage for valid persisted fixture
   output and no leftover workbench temp after success.
2. **GREEN:** Promote `tempfile` without lockfile change; implement
   `NamedTempFile` persistence and the production coordinator; run focused
   checks.
3. **DOCS/SMOKE:** Update run, deep-link, supported-scope, and productization
   blocker text. Exercise positive list/source/save/reload/deep-link and
   missing-cap `403` without recording token/private details.
4. **VERIFY:** Run workbench fmt, release tests, strict Clippy, Node
   syntax/tests, shared image-processing tests, controller focused/full tests,
   production-persistence check, ordinary controller Clippy, and diff checks.
   Commit each finding family, comment on PR #281, update the fix report, and
   run a fresh deep source review.

## Acceptance criteria

- Double-click Save dispatches one request through full settlement.
- Stale completion does not mutate or claim the current draft.
- Fixture writes use `NamedTempFile`; no fixed temp path remains.
- Success yields valid stable JSON and no leftover workbench temp.
- `Cargo.lock` stays byte-identical; no new package/feature is resolved.
- Concurrent writers are explicitly unsupported and multi-user productization
  is recorded as blocked pending the complete conflict/version contract.
- Capability run/deep-link instructions and smoke are correct.
- Full verification and fresh deep review are clean.

## Stop conditions

Stop if promoting `tempfile` changes `Cargo.lock`; `persist` cannot replace the
destination on the target; Save still uses abort authority; source expands into
production persistence; capability checks must weaken; or fresh review finds a
recurring/scope-expanding Critical or Warning.

