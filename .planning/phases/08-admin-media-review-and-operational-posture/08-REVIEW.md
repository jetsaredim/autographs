---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-05T20:58:59Z
depth: deep
files_reviewed: 12
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.js
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
  - controller/tests/media_cleanup.rs
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

# Phase 08: Code Review Report — Media Revision Authority Follow-up

**Reviewed:** 2026-10-05T20:58:59Z
**Depth:** deep
**Files Reviewed:** 12
**Status:** issues_found

## Narrative Findings (AI reviewer)

### Summary

Commit `68431b4` closes the original same-ID media-replacement authority gap across the normal paths. Admin-only opaque revisions are carried through item/review/source/draft/assist/Save/Reset contracts; media operations revalidate success and error terminals; in-memory and Oracle adjustment writes compare the expected private object key; replacement rollback restores the complete image snapshot when it succeeds; and public artifacts remain revision-free. The prior gesture render barrier, grab-offset and resize projection, mounted-output authority, mutation/egress serialization, publisher promotion, legacy migration, private-original, and public-output contracts remain green.

Two failure-path authority defects remain. A failed cleanup-warning rollback deletes the replacement object even though metadata can still reference it. Separately, any late 409 conflict is handled before its originating session/request/mutation tuple is checked, so stale work can tear down a newer review; an authoritative conflict also keeps the stale item revision, making the instructed “reopen” action repeat the same conflict.

### Finding lineage

| Prior finding or invariant | Current disposition | Evidence |
| --- | --- | --- |
| Media-replacement CR-01: same-ID replacement preserves stale review authority | Incomplete fix / failure-path and recovery sibling misses | Normal source/draft/assist/Save/Reset and item-response paths are revision-bound, but CR-01 leaves replacement metadata pointing at deleted bytes when restoration fails, and CR-02 lets stale conflict responses invalidate unrelated current authority while retaining the obsolete item token. |
| Media plan BL-01: terminal conflict precedence | Resolved on the server | Shared terminal revalidation runs after preview/read/render and assist on both success and error paths; the blocking-read regression returns the stable redacted 409 after replacement. |
| Media plan BL-02: central pre-assignment reconciliation | Resolved for returned item responses | `reconcileAdminItemResponse()` owns the only `state.currentItem` assignment and invalidates missing-image or mismatched-revision reviews before assignment. Direct 409 responses do not contain an item and expose the separate recovery defect in CR-02. |
| Media plan BL-03: complete rollback snapshot | Incomplete fix / rollback-failure sibling path | Successful rollback restores object metadata and saved adjustment, but the route unconditionally deletes the replacement even when that rollback returns an error (CR-01). |
| Media plan WR-01: typed 409 / 404 / 400 / 500 separation | Resolved | Typed repository adjustment errors and route responses preserve stable redacted conflict, not-found, validation, and repository classes. |
| Media plan WR-02: private object identity changes the token | Resolved | The revision test holds image ID and all other populated derivation inputs constant while changing only the private object key; the token changes and raw identity is absent from the response. |
| Round 2–gesture async/session authority | Incomplete fix / new conflict-terminal regression | Success paths retain their request/session/render authority, but media-conflict catch paths call the global teardown before checking the captured authority tuple (CR-02). |
| Gesture render barrier, source projection, grab offset, terminal capture lifecycle, mounted output, Save/Reset egress, promotion, legacy migration, privacy, and public output | Resolved / no regression found | Browser behavior harness, focused route/repository tests, full all-features suite, publisher tests, static privacy contract, formatting, and Clippy all pass. |

### Critical Issues

#### CR-01: Rollback failure deletes the object still referenced by catalog metadata

**Classification:** BLOCKER
**File:** `controller/src/routes.rs:1514-1534`
**Related:** `controller/tests/media_cleanup.rs:670-725`

**Issue:** After replacement metadata has committed, an old-object cleanup failure enters cleanup-warning persistence. If warning persistence also fails, the route attempts to restore the original image snapshot. The result of that restoration is only logged: whether it succeeds or fails, lines 1527-1534 delete `replacement_key`. When restoration fails, the committed catalog row still points at `replacement_key`; deleting it leaves the active image metadata referencing a missing private original. This is a data-availability failure, and a later publish/preview cannot recover from the catalog without manual repair. The existing test proves only the successful rollback case because its repository always accepts the second metadata replacement.

**Fix:** Make replacement cleanup conditional on verified restoration. The rollback mutation should compare the currently persisted replacement object key (so it cannot overwrite a concurrent replacement), restore the complete original snapshot atomically, and return a typed result. Delete `replacement_key` only after reloading proves the catalog no longer references it and the original object/adjustment snapshot is authoritative. If rollback fails or loses the compare-and-set race, preserve every possibly referenced object, return the redacted 500, and log/manual-cleanup the orphan candidate. Add a repository that succeeds the initial replacement but fails restoration and assert the replacement object remains readable and agrees with persisted metadata; also cover a concurrent newer replacement so rollback cannot overwrite or delete its active object.

#### CR-02: Unscoped 409 handling can tear down a newer review and cannot recover the current one

**Classification:** BLOCKER
**File:** `controller/static-admin/admin.js:1655-1665`
**Related:** `controller/static-admin/admin.js:1746-1755`, `controller/static-admin/admin.js:1807-1813`, `controller/static-admin/admin.js:2497-2507`, `controller/static-admin/admin.js:2545-2557`, `controller/static-admin/admin.js:2608-2619`, `controller/tests/static_admin_behavior.mjs:1081-1110`

**Issue:** Every conflict catch calls `handleMediaRevisionConflict(error)` before checking whether the response still belongs to the captured preview revision, review session, draft revision, or mutation token. A deferred assist or draft request from review A can therefore return a media-revision 409 after review B becomes current and unconditionally clear review B. This is the conflict-path equivalent of the previously fixed stale async callback authority defect. For a genuinely current conflict, the handler clears the review but merely rerenders `state.currentItem`, which still contains the obsolete `mediaRevision`. Clicking Review again begins another session with that old token; the review endpoint returns the current token, the mismatch invokes the same handler, and the operator cannot actually reopen the review until some unrelated item reload occurs. The current DOM test checks teardown and copy only, so both behaviors pass.

**Fix:** Make conflict handling require the same authority tuple as the operation that produced it. In each catch, reject stale/aborted preview, session, draft, assist, and mutation results before allowing a 409 to affect UI state, or pass an authority predicate/token into a centralized handler. For an authoritative conflict, capture the item ID, invalidate the review exactly once, then fetch and reconcile the current admin item (or return an equally fresh redacted item/revision contract from the server) before presenting “Reopen the review.” Add browser-faithful tests where an old assist/draft 409 resolves after a newer review is mounted and cannot alter it, plus a current 409 followed by Review that succeeds using the refreshed revision.

## Verification

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow --test media_cleanup --test static_admin` — passed; 70 tests.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 223 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- Source/test `git diff --check origin/main...HEAD` across all implementation files — passed.

---

_Reviewed: 2026-10-05T20:58:59Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
