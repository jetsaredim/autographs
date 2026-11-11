---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-11-11T01:41:51Z
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
  warning: 1
  info: 0
  total: 2
status: issues_found
convergence:
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: null
  plan_evidence: null
---

# Publish 524: PR #337 code review

**Reviewed:** 2026-11-11T01:41:51Z  
**Depth:** deep  
**Status:** issues_found

## Summary

The new admin request returns promptly and the background task contains the Oracle recording await, so a browser disconnect cannot by itself cancel that task. The authenticated operation endpoint remains under the existing Caddy admin bypass and deployment freeze paths. Two operation identity races remain. The new integration test covers one async operation and a duplicate attempt while it is running; it does not cover a synchronous publish queued behind it or a later operation replacing a completed result.

## Critical Issues

### CR-01: Terminal result can be taken from a different publish

**File:** `controller/src/routes.rs:2191-2219`  
**Finding family:** `publish_operation_result_identity`  
**Lineage:** `independent`  
**Shared invariant:** The terminal state of an accepted operation must describe that operation's static release and Oracle publish-record transaction.  
**Consumers inspected:** `publish_with_request_id` lock and response paths; `LocalPublisher::publish_locked` status mutations; `record_successful_publish` for memory and Oracle; synchronous POST routes; operation polling, admin status, and admin UI; focused integration and browser tests.  
**Evidence level:** `source_only`  
**Issue:** After `publish_with_request_id(...).await` returns, its publish lock is released. The task then reads the publisher's global status at line 2194. A synchronous POST queued behind this operation can acquire the lock and replace that global status before the read. The first operation can therefore be marked as the second job's `running`, `failed`, or `succeeded` state. On a recording error, the same race can hide the first job's active-release/incomplete outcome.  
**Fix:** Return the operation's own typed publish outcome from the locked publish/recording path, or atomically store the operation's terminal status before releasing the publish lock. Coordinate synchronous and asynchronous POST admission through the same operation gate. Add a deterministic test that queues a synchronous publish behind the async job and asserts the first operation retains its own release ID and recording result.

## Warnings

### WR-01: Starting a later publish deletes the previous operation result

**File:** `controller/src/routes.rs:2174-2188`  
**Finding family:** `publish_operation_result_retention`  
**Lineage:** `independent`  
**Shared invariant:** An accepted operation ID must remain queryable long enough for its client to observe its terminal result.  
**Consumers inspected:** Operation creation and replacement, status lookup at lines 2225-2243, two-tab admin polling in `admin.js`, current status reporting, and the async integration test.  
**Evidence level:** `source_only`  
**Issue:** Once the current operation is no longer `running`, the next accepted publish overwrites the only stored `PublishOperation`. A browser polling the earlier ID then gets 404 and reports an unconfirmed outcome even though the first publish completed. This can happen with two admin tabs or a retry immediately after completion; the test only checks a duplicate while the first job is running.  
**Fix:** Keep a bounded, time-limited map of recent operations keyed by ID, with a separate active-operation marker. Retain completed results through a defined poll window, and test a second accepted publish before the first client reads its result.

---

_Reviewer: gsd-code-reviewer_  
_Depth: deep_
