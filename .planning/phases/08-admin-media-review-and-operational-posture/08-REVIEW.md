---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-06T21:47:50Z
depth: deep
files_reviewed: 16
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/media.rs
  - controller/src/oci_media.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.js
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
  - controller/tests/logging_contract.rs
  - controller/tests/media_cleanup.rs
  - controller/tests/publisher.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
  - controller/tests/static_contract.rs
findings:
  critical: 2
  warning: 0
  info: 0
  total: 2
status: issues_found
---

# Phase 08: Code Review Report — Media Failure Recovery Follow-up

**Reviewed:** 2026-10-06T21:47:50Z
**Depth:** deep
**Files Reviewed:** 16
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

Commit `6cb2f5d` closes the two directly reported media-recovery failures on their modeled paths. Initial replacement and rollback now use typed expected-object-key compare-and-set transitions in memory and Oracle; metadata restoration is checked before replacement deletion; error bodies and UUID-correlated fingerprint-only diagnostics are redacted; source acquisition is status-bearing fetch-to-blob; and operation-specific conflict predicates plus a recovery generation make stale 409 responses inert. The full all-feature suite, browser behavior harness, formatting, Clippy, implementation diff hygiene, privacy contracts, publisher promotion matrix, legacy migration, media-revision, mounted-output, gesture barrier, grab-offset/resize, and mutation/egress regressions remain green.

Two unmodeled recovery boundaries remain unsafe. First, an Object Storage DELETE error is not proof that the old object still exists: OCI transport can fail after the service applied the delete. The rollback path restores metadata to that unverified original and then deletes the known-good replacement. Second, conflict recovery uses only view/navigation generation as editor ownership; it renders an active blank editor while recovery is pending, so typing or saving in that same view does not invalidate the old recovery result and can be silently overwritten.

### Finding lineage

| Prior finding or invariant | Current disposition | Evidence |
| --- | --- | --- |
| Media follow-up CR-01: failed rollback deletes referenced replacement object | Incomplete fix / failure-mode sibling miss | Repository/CAS failure, concurrent winner, verification mismatch, and replacement-delete failure preserve candidates. CR-01 remains for ambiguous old-object DELETE outcomes because restoration verifies only metadata, not that the restored private original is readable, before deleting the replacement. |
| Media follow-up CR-02: unscoped 409 tears down newer review and retains stale reopen token | Incomplete fix / sibling action-path miss | Stale operation conflicts are inert and current recovery refreshes the revision. CR-02 remains because recovery generation covers view/navigation changes but not same-view editor ownership or form mutation; a late refresh can overwrite newly entered or newly saved item state. |
| Media-revision BL-01: terminal conflict precedence | Resolved | Preview/source/assist terminal revalidation still gives a current media mismatch stable 409 precedence over underlying read/render/provider errors. |
| Media-revision BL-02: central response reconciliation | Resolved on item-returning media paths | Active reviewed-image absence/revision mismatch invalidates before assignment. The open client finding is a separate recovery/editor ownership race after the review has already been invalidated. |
| Media-revision BL-03: complete rollback snapshot | Resolved for metadata | Original object metadata and saved adjustment are restored together. The open server finding concerns object existence, not snapshot completeness. |
| Typed 409/404/recovery-required/recovered-failure responses and private diagnostics | Resolved | Exact bodies remain distinct; recovery IDs are UUIDs; HTTP stays redacted; private diagnostics contain transition/outcome plus domain-separated key fingerprints rather than raw keys. |
| Source fetch-to-blob and per-operation conflict authority | Resolved | Source request/controller/blob/node/session/media authority is explicit, and stale source/draft/assist/Save/Reset conflicts do not mutate a newer review. |
| Gesture render barrier, source projection, grab offset, terminal capture lifecycle, mounted output, Save/Reset egress, publisher promotion, legacy migration, private-original, and public output | Resolved / no regression found | Executable DOM behavior, focused recovery suites, full all-feature suite, static privacy contracts, formatting, and Clippy pass. |

### Critical Issues

#### CR-01: Ambiguous old-object deletion can roll metadata back to missing bytes and then delete the valid replacement

**Classification:** BLOCKER  
**File:** `controller/src/routes.rs:1626-1700`  
**Related:** `controller/src/oci_media.rs:77-89`, `controller/tests/media_cleanup.rs:760-859`

**Issue:** The rollback branch is entered because deleting `existing_image.object_key` returned an error. It immediately restores catalog metadata to that key, verifies only the catalog snapshot, and then deletes `replacement_key`. An OCI DELETE transport error is outcome-ambiguous: the service can apply the delete and the client can still receive a connection/read failure. `OciInstancePrincipalMediaStore::delete` propagates those transport errors without proving postcondition. In that case the route restores metadata to an object that no longer exists and deletes the only known-readable replacement, recreating the data-availability loss this recovery change is intended to prevent. The tests model delete failure as “return `Err` without deleting,” so they cannot expose this branch.

**Fix:** Treat old-object existence/readability as part of rollback proof. Before restoring metadata—or at minimum before deleting the replacement—read or otherwise strongly verify the original object and validate it against the original snapshot checksum/size. If the original cannot be proved readable and matching, keep replacement metadata/object authoritative, preserve all candidates, and return `imageRecoveryRequired` with private correlated evidence. Add an ambiguous-delete media-store fixture that deletes the original and then returns `Err`; assert metadata remains on the readable replacement and the replacement is never deleted.

#### CR-02: Conflict refresh can overwrite unsaved or newly saved same-view editor work

**Classification:** BLOCKER  
**File:** `controller/static-admin/admin.js:1708-1725`  
**Related:** `controller/static-admin/admin.js:1758-1767`, `controller/static-admin/admin.js:3034-3050`, `controller/tests/static_admin_behavior.mjs:1138-1208`

**Issue:** An authoritative conflict clears the item and calls `renderEditor(null)`, which exposes a fully interactive new-item editor while `recoverConflictedItem()` is pending. Recovery accepts a result when its object/generation, `navigationRevision`, and `currentView` still match. Editing the form does not change any of those values. Neither does `saveItem()` or its same-view `renderEditor(item)` path. Therefore a user can type a new item while recovery is delayed and have line 1724 reset the form and clear dirty state, silently losing the input. More severely, a new-item save can complete first, render the newly persisted item, and then the old recovery response can overwrite `state.currentItem` and the editor with the conflicted item because the recovery generation remains valid. Existing tests cover retry, 404, stale operation conflict, and immediate reopen, but not same-view edit/save ownership.

**Fix:** Give conflict recovery exclusive, explicit editor ownership. Render a non-editable recovery state rather than an actionable blank item form, or disable all item/form actions until recovery settles. In addition, invalidate recovery whenever editor ownership changes in the same view—including form mutation, `saveItem`, and any direct `renderEditor`/item reconciliation not initiated by that recovery—and include that owner token in the acceptance predicate. Add deferred-recovery tests for typing and saving a new item before the old refresh resolves; the recovery must be inert and must not reset dirty state or replace the newly saved item.

## Verification

- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 230 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check 6cb2f5d^..HEAD -- controller` — passed.

---

_Reviewed: 2026-10-06T21:47:50Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: deep_
