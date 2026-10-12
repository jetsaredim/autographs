---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-12T00:15:20Z
depth: deep
files_reviewed: 20
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/media.rs
  - controller/src/oci_media.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/db/schema.sql
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/static-admin/index.html
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
  - controller/tests/logging_contract.rs
  - controller/tests/media_cleanup.rs
  - controller/tests/publisher.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
  - controller/tests/static_contract.rs
  - docs/static-runtime-runbook.md
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
convergence:
  status: reassessment_required
  recurring_critical_warning: 1
  scope_expansion: false
  requires_reassessment: true
  assumptions_evidence: null
  plan_evidence: null
---

# Phase 08: PR #263 Post-Convergence Deep Code Review

**Reviewed:** 2026-10-12T00:15:20Z
**Reviewed commit:** `91fad0cc0a12fbc685742dfdf44ae66e258bdf5a`
**Implementation commit:** `d61a38e3608a29f4454a1d5c2ec7ec86b293abad`
**Base:** `main` (`f751364c0856bc82ce229e247341a4c2935f4a53` merge base)
**Depth:** deep
**Files Reviewed:** 20
**Status:** issues_found
**Convergence:** reassessment_required

## Narrative Findings (AI reviewer)

### Summary

The complete PR diff was reviewed against `main`, with a fresh producer,
consumer, mutation-boundary, reporting-path, failure-matrix, and test-fidelity
audit of the single authorized post-convergence coder pass.

CR-01 (`exclusive_conflict_recovery_surface`) is closed. The recovery status
and retry now live outside the hidden/inert form; pending, retryable, success,
404, auth, navigation, direct-render, and newer-review paths retain the
existing generation and editor-authority checks. The updated DOM harness now
exercises the decisive ancestor hidden/inert behavior.

WR-01 (`live_smoke_cleanup_evidence_privacy`) is also closed. Normal,
already-absent, timeout, provider-failure, verification, panic, fallback, and
runbook paths use safe categories and domain-separated fingerprints without
forwarding raw keys or provider-controlled text.

CR-02 (`live_smoke_cleanup_discovery`) is not closed. The new discovery-first
guard destructively moves its locally known candidates before running Oracle
discovery. A transient inventory failure therefore erases the only in-process
copy of candidates that may not exist in Oracle. If the `Drop` retry then
succeeds, it can delete the current object and all discovery rows while leaving
the lost old object orphaned. The deterministic fixture does not execute this
production state transition and consequently gives false-green retry evidence.

Because this is a recurring Critical finding after the one authorized coder
pass, the convergence guard requires a new durable reassessment and an
independently reviewed revised plan before any further code change.

### Stable finding lineage

| Stable finding | Finding family | Lineage classification | Disposition |
| --- | --- | --- | --- |
| CR-01 | `exclusive_conflict_recovery_surface` | `sibling_path_miss` | Closed: the dedicated status/action surface is outside `#item-form`, remains perceivable while the form is hidden/inert, and is retired on authority changes. |
| CR-02 | `live_smoke_cleanup_discovery` | `incomplete_fix` | Open: failed discovery consumes locally known candidates before fallback cleanup retries. |
| WR-01 | `live_smoke_cleanup_evidence_privacy` | `sibling_path_miss` | Closed: cleanup and runbook evidence is fingerprint-only and provider text is not forwarded. |

Previously closed media-revision CAS, replacement-authoritative ambiguous
delete behavior, scoped conflict recovery, gesture isolation, mounted-output
authority, publisher promotion, comparison-map migration, and public-artifact
privacy remain closed in the inspected implementation and regressions.

## Critical Issues

### CR-02: Failed inventory consumes the guard's only copy of locally known cleanup candidates

**Classification:** BLOCKER
**Finding family:** `live_smoke_cleanup_discovery`
**Lineage:** `incomplete_fix`
**File:** `controller/tests/live_persistence_smoke.rs:1391-1398`
**Related:** `controller/tests/live_persistence_smoke.rs:100-106`, `controller/tests/live_persistence_smoke.rs:247-250`, `controller/tests/live_persistence_smoke.rs:1080-1111`, `controller/tests/live_persistence_smoke.rs:1419-1467`, `controller/tests/media_cleanup.rs:917-1014`
**Evidence level:** `source_only` (credential-gated live Oracle/OCI execution was unavailable)

**Shared invariant:** Every locally known, current-image, and cleanup-event
object identity must remain recoverable until object absence is verified and
only then may Oracle discovery rows be deleted. A failed inventory attempt must
be non-destructive to the candidate set used by fallback or operator recovery.

**Consumers inspected:** Initial A candidate registration, post-replacement B
inventory, `discover_cleanup_object_keys`, explicit cleanup, `Drop` fallback,
OCI delete/read and 404 handling, Oracle item cascade/commit/count
verification, recovery-required reporting, the cleanup authority fixture,
source-order logging contracts, the runbook, and the replacement route's
ambiguous-delete/warning-persistence branches.

**Issue:** `refresh_inventory()` passes
`std::mem::take(&mut self.object_keys)` into the fallible discovery helper and
assigns the result back only on `Ok`. If either Oracle query, row read, or
decode fails, `self.object_keys` remains empty. During the post-replacement
refresh, that vector contains the original A key, which may be the only record
of A when the old-object delete outcome is ambiguous and cleanup-warning
persistence failed. Unwinding invokes `Drop`; if its second inventory attempt
succeeds, it discovers current B but not A, deletes/verifies B, then deletes the
item and cascading discovery rows. If the ambiguous DELETE did not apply, A is
now an orphan with no retained key. If the second attempt also fails, rows are
retained but the process still loses the local candidate that the guard was
created to preserve.

The added fixture cannot catch this. Its `cleanup()` borrows
`local_candidates` on every retry, so the failure loop at
`controller/tests/media_cleanup.rs:994-1014` proves behavior stronger than the
production guard actually implements.

**Fix:** Make inventory failure atomic with respect to locally known
candidates. Clone or borrow the current candidates, merge discovered values
into a temporary set, and replace `self.object_keys` only after discovery
succeeds. Exercise the real state machine (or an extracted shared helper) with
an injected first-discovery failure followed by success, asserting that A and
B are both deleted/verified before row deletion. Also cover persistent
discovery failure and prove the original local candidate remains available for
recovery evidence/retry.

## Convergence Reassessment

### Finding lineage

The resumed review's CR-02 identified that cleanup could delete Oracle rows
before discovering a route-generated B. Commit `d61a38e` correctly added
discovery before destructive row cleanup, but the failure transition of that
discovery destructively consumes the pre-existing candidate set. This is an
`incomplete_fix` in the same `live_smoke_cleanup_discovery` family, not an
independent defect or scope expansion. CR-01 and WR-01 are closed.

### Shared invariants

1. **Candidate ownership survives fallible discovery.** Locally known keys are
   durable guard state until every candidate is verified absent.
2. **Discovery precedes destruction.** The union of durable local candidates,
   current image keys, and cleanup-event targets is complete before any row
   deletion.
3. **Failure remains recoverable.** Inventory/delete/verification failure
   retains both Oracle discovery metadata and all locally held candidates.
4. **Evidence remains opaque.** Recovery reports continue to expose item IDs,
   safe steps, and fingerprints only.

### Complete consumer inventory

| Boundary | Producers | Consumers / mutations | Failure/reporting paths | Required evidence |
| --- | --- | --- | --- | --- |
| Local candidate ownership | Initial A registration; post-success B registration | `refresh_inventory`, explicit cleanup, `Drop` | Oracle query/row/decode failure; panic unwind | Candidate set unchanged after failed refresh |
| Oracle discovery | `autograph_images.object_key`; `autograph_cleanup_events.target_object_key` | Merge/deduplicate; object delete/read | Transient and persistent discovery failures | Failure-then-success and persistent-failure tests using production-equivalent state |
| Object cleanup before rows | Local/discovered union | OCI DELETE, absence GET, item cascade delete, commit, zero-row verification | delete error, timeout, non-404 read error, object still present | Every candidate verified absent before first destructive SQL |
| Recovery evidence | Safe cleanup message/fingerprint helpers | manual cleanup, guard error, fallback `Drop`, runbook | inventory, delete, verify, runtime/thread, Oracle failures | No raw candidate/provider data and enough safe correlation to retry |

### Assumption audit

| Assumption | Result | Evidence |
| --- | --- | --- |
| `mem::take` is safe because discovery immediately returns all candidates | False | The helper has fallible query, row-read, and decode operations before returning the merged set. |
| Oracle can always rediscover a lost local candidate | False | In the motivating ambiguous-delete plus warning-persistence-failure path, old A may have no cleanup-event row and B is the current image row. |
| Retaining Oracle rows alone preserves every cleanup identity | False | The guard's local A candidate can be the only remaining identity for an old object whose delete outcome is ambiguous. |
| The deterministic fixture proves retry behavior of the guard | False | The fixture re-borrows `local_candidates` on retry and does not model the production `mem::take` transition. |
| Green targeted/full tests cover this failure | False | The live guard is credential-gated; executed tests use a separate model with stronger retention semantics. |

### Failure matrix

| Scenario | Required outcome | Current outcome |
| --- | --- | --- |
| First inventory succeeds | Merge A/B/event targets, delete/verify all, then delete rows | Correct |
| First inventory fails, `Drop` inventory succeeds | Preserve A, merge rediscovered B, delete/verify both, then delete rows | A is consumed by the first failure; fallback may delete B and rows while orphaning A |
| First and fallback inventory both fail | Retain rows and local A for later recovery | Rows remain, but local A is lost from guard state/process evidence |
| Delete or absence verification fails after successful inventory | Retain rows and full candidate set | Correct |
| Evidence paths run for any failure | Emit only safe item/step/fingerprint data | Correct |

### Revised plan requirements

1. Define failed inventory as an atomic, non-mutating operation on the guard's
   durable local candidate set.
2. Extract or directly test the production candidate-state transition rather
   than a parallel fixture that cannot reproduce ownership loss.
3. Cover first-failure/second-success, repeated failure, partial discovery,
   duplicate candidates, and successful retry before any row deletion.
4. Preserve the closed recovery-surface and evidence-privacy contracts without
   expanding Phase 8 scope.

### Resume criteria

1. Record this recurring finding and revised assumptions as durable convergence
   evidence on the PR branch and PR audit trail.
2. Produce a revised implementation plan covering atomic candidate ownership,
   the complete consumer inventory, and the expanded failure matrix.
3. Obtain an independent plan review with zero Critical, Warning, and Info
   findings; record both assumption and plan-review evidence before changing
   convergence status to `ready_to_resume`.
4. Only then may another coherent coder pass begin.
5. A subsequent lineage-preserving deep source review must return zero
   actionable findings before convergence closes.

## Verification

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin` — passed, 19 tests.
- `cargo test --manifest-path controller/Cargo.toml --test media_cleanup` — passed, 20 tests; the CR-02 fixture fidelity limitation is described above.
- `cargo test --manifest-path controller/Cargo.toml --test logging_contract` — passed, 3 tests.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 2 credential-gated live tests ignored.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check main...HEAD -- controller docs/static-runtime-runbook.md` — passed.
- Credentialed Oracle/OCI cleanup execution was not available; CR-02 is supported by the guard's deterministic ownership transition in source.

---

_Reviewed: 2026-10-12T00:15:20Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
