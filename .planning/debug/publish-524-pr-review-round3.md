---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-11-11T10:41:04Z
depth: deep
files_reviewed: 10
files_reviewed_list:
  - controller/src/routes.rs
  - controller/src/publisher.rs
  - controller/src/catalog.rs
  - controller/src/oracle_catalog.rs
  - controller/static-admin/admin.js
  - controller/tests/admin_workflow.rs
  - controller/tests/static_admin_behavior.mjs
  - deploy/ansible/roles/autographs_deploy/files/Caddyfile
  - deploy/ansible/roles/autographs_deploy/files/install_publish_freeze_gate.py
  - docs/controller-walkthrough.md
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
convergence:
  status: ready_to_resume
  recurring_critical_warning: 1
  scope_expansion: false
  requires_reassessment: true
  assumptions_evidence: .planning/debug/publish-524-revised-assumptions.md
  plan_evidence: .planning/debug/publish-524-revised-plan.md
---

# Publish 524: PR #337 code review, round 3

**Reviewed:** 2026-11-11T10:41:04Z  
**Depth:** deep  
**Status:** issues_found — mandatory convergence checkpoint

## Summary

The typed outcome closes the original cross-publish result race on ordinary completion. The bounded recent-operation store closes the lost-result race. The task-owned finalizer closes the observed nonterminal error and active-gate leak for the tested publisher error, panic, and abort paths. One recurrence of the operation-identity invariant remains in the finalizer fallback. No further point-fix round should begin until the assumptions and implementation plan below have been revised and reviewed.

## Warnings

### WR-02: Unconfirmed fallback can report another publish's release ID

**File:** `controller/src/routes.rs:130-137`  
**Finding family:** `publish_operation_result_identity`  
**Lineage:** `fix_regression` of round-1 CR-01  
**Shared invariant:** A release ID in an operation result must be proven to belong to that accepted operation; uncertainty must not borrow the global publisher's latest release.  
**Consumers inspected:** Task panic/abort fallback; `LocalPublisher::publish_locked` lock lifetime and status mutation; synchronous POST queue; retained operation lookup; `current_publish_status`; `publishSummaryText`; panic/abort, queued sync, and sequential operation tests.  
**Evidence level:** `source_only`  
**Issue:** `PublishOperationFinalizer::drop` reads `self.publisher.status()` after a failed or aborted task unwinds. That status is global. A panic before this operation starts can copy the previous successful release ID. A synchronous POST queued behind this operation can acquire the publish lock during unwind and set a different release ID before the finalizer reads it. `unconfirmed_publish_operation_status` retains that ID, and the admin UI displays it as this operation's release. `current_publish_status` can also replace the prior successful status with this unconfirmed one when the IDs match. The new panic and abort tests start with an idle publisher and do not exercise either provenance case.  
**Fix requirement:** Define one coherent operation-result contract in which a release ID is attached only from an operation-owned value. If identity is unknown on panic or abort, the fallback result should carry no release ID and instruct the operator to inspect the active release. The finalizer must still clear the active gate and retain a terminal result.

## Convergence Reassessment

Revised assumptions and executable plan are recorded in the frontmatter above.
Both passed the review in `publish-524-revised-plan-review.md` before coder work resumed.

### Finding lineage

| Family | Round 1 | Round 2 | Round 3 |
| --- | --- | --- | --- |
| `publish_operation_result_identity` | CR-01: normal completion read global status after lock release | Closed by typed `PublishRouteOutcome` and queued sync test | WR-02: exceptional fallback again reads global status (`fix_regression`) |
| `publish_operation_result_retention` | WR-01: later publish overwrote only result | Closed by bounded recent-operation store | Remains closed |
| `publish_operation_terminalization` | Not reported | CR-02: error or panic left operation running or gate active | Closed for tested error, panic, and abort paths by finalizer and normalization |

### Shared invariants

1. **Identity:** Any reported release ID must belong to the accepted operation ID.
2. **Commit truth:** `succeeded` is visible only after the catalog publish record commits; promotion without that record is `incomplete` or `unconfirmed` according to what can be established.
3. **Terminalization:** Every accepted operation reaches a terminal status and releases the active gate on success, ordinary error, panic, or abort.
4. **Observation:** A terminal result remains queryable for the documented poll window, and admin reporting does not substitute a different operation's result.
5. **Safety boundary:** Admin operations remain session-authorized, `no-store`, and covered by the deployment freeze gate.

### Complete consumer inventory

| Boundary | Producer or mutation | Consumer or report | Evidence |
| --- | --- | --- | --- |
| Browser request | `publishChanges` sends `Prefer: respond-async` | Existing publish POST route | Static admin behavior test |
| Admission | `start_publish_operation` allocates operation ID and active marker | 202 body and duplicate rejection | Async integration test |
| Publish execution | `publish_authorized` snapshots edit boundary, calls `publish_locked`, records catalog publish | Typed route outcome and finalizer | Route source and memory test |
| Static media | `LocalPublisher::publish_locked` mutates global status and promotes release | Public Caddy media path and route outcome | Publisher source; production image observation |
| Catalog commit | Memory or Oracle `record_successful_publish` records edit boundary | Pending-change queries and route success log | Catalog and Oracle source |
| Exceptional completion | `PublishOperationFinalizer::drop` converts panic/abort to `unconfirmed` | Operation store and active gate | Panic/abort tests; missing provenance test |
| Operation history | `PublishOperations` retains recent terminal results | `GET /publish/operations/{id}` | Sequential operation test |
| Admin reporting | `current_publish_status` and `publishSummaryText` | Admin status, disabled buttons, release summary | Source and browser behavior test |
| Edge/deploy | Caddy admin API route and freeze injector | Cloudflare proxy, controller | Caddy source tests and existing production bypass evidence |

### Assumption audit

| Assumption | Verdict | Consequence |
| --- | --- | --- |
| A global publisher status read during finalizer drop belongs to the dropped operation. | False. The publisher status may still be from a prior run or may change after the lock is released during unwind. | An unconfirmed operation may carry another release ID. |
| An `unconfirmed` state makes an unverified release ID harmless. | False. The UI and operation JSON still display the ID, and the admin status overlay uses it. | Operator may inspect or act on the wrong release. |
| Panic and abort tests prove release identity safety. | False. They verify terminal state and gate release with an initially idle publisher only. | The test matrix misses previous-release and queued-sync cases. |
| The single-operator design rules out overlapping API requests. | False. A synchronous caller and two tabs can overlap; the code intentionally permits a queued synchronous publish. | Identity must hold under concurrency regardless of normal usage frequency. |
| Operation history survives controller restart. | Not claimed. It is in-memory and resets on restart. | Production verification should distinguish in-process completion from restart recovery. |
| The `Prefer` path retains auth, no-store, and freeze. | Supported by route and Caddy source. | Preserve this boundary in the revised plan. |

### Failure matrix

| Scenario | Expected result | Current evidence or gap |
| --- | --- | --- |
| Async publish and Oracle record succeed | Own release ID, `succeeded`, edits cleared | Tested in memory; live Oracle pending |
| Promotion succeeds, Oracle record fails | Own release ID, `incomplete` | Typed route source; fault test absent |
| Publisher returns error while its status remains running | Terminal `unconfirmed`, gate released | Deterministic test passes |
| Panic before publisher starts after a previous successful release | Terminal `unconfirmed` with no unverified release ID | Missing; fallback currently copies prior ID |
| Panic or abort during promotion while sync publish is queued | Terminal `unconfirmed`; no other job's ID | Missing; fallback can read queued job's status |
| Completed first operation followed by a second accept | First result remains queryable | Deterministic test passes |
| Controller restart during operation | Outcome must be checked from active release and catalog state | Operational limitation, not covered by in-memory operation store |
| Deployment freeze during operation | No new publish accepted through Caddy; current operation completes or becomes uncertain on shutdown | Existing Caddy gate, live behavior pending |

### Revised plan requirements

1. Specify an operation-owned result model before coding: which fields can be known at acceptance, after publisher start, after promotion, and after Oracle commit. State explicitly when `releaseId` must be absent.
2. Inventory every fallback source and consumer of `PublishStatus`, including `Drop`, ordinary `Err`, `current_publish_status`, operation polling, admin summary, and synchronous callers. Remove the assumption that global publisher status proves operation identity.
3. Keep finalization and active-gate release atomic for each operation. Handle panic/abort without reading mutable global status for identity.
4. Add deterministic tests for prior successful release then panic, queued synchronous publish during exceptional unwind, promotion before Oracle failure, and normal success. Assert operation ID to release ID provenance and pending-edit behavior, not only terminal state.
5. Preserve the existing session auth, Caddy no-store/freeze paths, bounded operation retention, and release-please deployment gate.
6. Review the revised assumptions and the implementation plan before any coder change. Record both reviewed evidence paths in this artifact before changing convergence status to `ready_to_resume`.

### Resume criteria

- A revised assumptions artifact explicitly rejects global-status provenance and is reviewed against the source and failure matrix above.
- A revised executable plan covers the full operation-result contract and deterministic tests; it is reviewed for producer, consumer, and recovery coverage.
- The reviewed assumptions and plan paths are recorded in `convergence.assumptions_evidence` and `convergence.plan_evidence` here. A status-only edit is insufficient.
- Only then may `convergence.status` become `ready_to_resume` and coder work resume. Until then, PR #337 should not be merged.

---

_Reviewer: gsd-code-reviewer_  
_Depth: deep_
