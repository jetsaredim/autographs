---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-11-11T01:51:17Z
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
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
convergence:
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: null
  plan_evidence: null
---

# Publish 524: PR #337 code review, round 2

**Reviewed:** 2026-11-11T01:51:17Z  
**Depth:** deep  
**Status:** issues_found

## Summary

The first review's `publish_operation_result_identity` finding is closed: `publish_authorized` returns this operation's typed result, so a queued synchronous publish cannot replace its terminal release ID. The `publish_operation_result_retention` finding is closed: recent completed operations remain keyed by ID for a bounded interval, and a second accepted publish does not make the first result unqueryable. The tests exercise both cases. One independent terminalization defect remains in existing error and panic paths.

## Critical Issues

### CR-02: Background operation can finish without a terminal status or active cleanup

**File:** `controller/src/routes.rs:2223-2243`  
**Finding family:** `publish_operation_terminalization`  
**Lineage:** `independent`  
**Shared invariant:** Every accepted asynchronous publish must eventually reach a terminal operation state and release the active-operation gate, including publisher errors and task panics.  
**Consumers inspected:** `LocalPublisher::publish_locked` and `record_pre_promotion_failure`; `publish_authorized` error branch; background task update; `PublishOperations::active` admission; operation lookup; `current_publish_status`; admin polling and button availability; tests for async success, duplicate, queued sync, and sequential results.  
**Evidence level:** `source_only`  
**Issue:** A publisher error can leave its global status as `running`. For example, `active_release_id` can error at `controller/src/publisher.rs:1024`, or failed-candidate cleanup can error at lines 1115-1116 before `record_pre_promotion_failure` sets `failed`. The route's `Err` branch copies that still-running status into `PublishRouteOutcome`; the background task then sets `completed_at` and clears `active` while storing `running`. The operation poll waits 15 minutes, and `current_publish_status` disables the admin publish buttons until the 30-minute retention expires. Separately, if the spawned task panics, code after `publish_authorized(...).await` never runs, leaving `active = Some(operation_id)` indefinitely and every later async POST returns 409 until restart.  
**Fix:** Normalize every returned outcome to a safe terminal state before storing it, distinguishing known pre-promotion failure from uncertain post-promotion state. Put active-gate cleanup and terminal fallback under an owner that runs on task completion, error, and panic, or monitor the task join result. Add deterministic tests for an error that leaves publisher status `running` and a panicking repository task.

## Finding lineage

| Family | Round 1 | Round 2 |
| --- | --- | --- |
| `publish_operation_result_identity` | CR-01 | Closed by typed route outcome and queued sync test |
| `publish_operation_result_retention` | WR-01 | Closed by bounded recent operation store and sequential operation test |
| `publish_operation_terminalization` | Not reported | CR-02 independent |

---

_Reviewer: gsd-code-reviewer_  
_Depth: deep_
