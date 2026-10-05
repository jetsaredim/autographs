---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: ce984bc6129ad5f073514e5b50c5981666d54402
reviewed: 2026-10-05
review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5996802316
verdict: revisions_required
blockers: 3
warnings: 2
info: 0
---

# PR 263 Media Revision Authority Plan Review

## Verdict

**REVISIONS REQUIRED — three blockers and two warnings.**

The opaque `mediaRevision`, route pre/post checks, and atomic expected-object-key predicate are the right authority model, but the addendum does not yet close every state transition it claims to cover. Keep `08-PR-263-CONVERGENCE.md` at `reassessment_required`; no coder pass should start from this revision.

## Goal-Backward Coverage

| Required property | Current plan evidence | Verdict |
|---|---|---|
| Replacement changes an opaque private identity without exposing storage metadata | Domain-separated SHA-256 includes the image UUID, private object key, checksum/ETag, content type, and byte size; admin-only DTOs carry only the digest; the matrix excludes it from public artifacts. | Covered in design; exact identical-byte replacement proof is incomplete (WR-02). |
| Review/source/draft/assist/Save/Reset bind to one media snapshot | The token is required in admin item/review responses, URLs/requests, session/callback authority, and repository mutations. | Covered. |
| Replacement during media work cannot return an old-media result or be reported as an unrelated provider failure | The plan revalidates after successful read/render/assist work. | Not covered for failed work after a concurrent replacement (BL-01). |
| Save/Reset compare the server-private expected object key atomically | Memory compares under the item lock; Oracle adds the object key to the update predicate; canonical no-op responses require a fresh revision check. | Covered in design; status-class separation lacks a decisive matrix (WR-01). |
| Every returned admin item reconciles or invalidates the active review before assignment | The plan compares a returned reviewed image token and gives `replaceImage` a special pre-assignment rule. | Not closed for a missing reviewed image or for existing call sites that preassign `state.currentItem` (BL-02). |
| Cleanup success, warning, and rollback preserve one coherent persisted snapshot | The matrix requires response revision and persisted object agreement. | The plan does not restore or otherwise invalidate the original adjustment when rollback reuses replacement methods that force it to `None` (BL-03). |
| Old geometry cannot mutate replacement metadata | The route token checks, repository predicate, 409 client teardown, and concurrency matrix cover replacement before/within preview, assist, gesture, Save, and Reset boundaries. | Covered once BL-01 through BL-03 are resolved. |

## Blockers

### BL-01 — Post-validation must arbitrate failed media work as well as successful work

**Dimension:** key links / failure-matrix completeness  
**Required property:** After initial revision validation, every source/private-preview, draft-preview, and assist terminal path must re-read authority before choosing its final response; if the media revision changed, HTTP 409 with the stable redacted conflict contract must supersede both success and read/render/assist errors.

**Evidence:** `08-PR-263-ROUND3-REVISED-PLAN.md:185-186` requires post-validation only “before returning a successful result.” In the current seams, `image_preview_response` returns a provider-shaped 500 immediately when `media.read` or `generate_adjusted_derivative` fails (`controller/src/routes.rs:848-871`), and assist does the same for read/proposal errors (`controller/src/routes.rs:1081-1098`). A replacement can commit and delete the old object after pre-validation but before the old-key read completes. The proposed wording permits that authority mismatch to be reported as a generic preview/provider failure instead of the mandatory 409, so the client need not tear down the stale review.

**Example revision (non-binding):** Require a common finalizer for success and error branches: re-load the image, compare the expected revision, return the stable 409 on mismatch, and only otherwise return the operation's original success/error. Add replacement-during-read, replacement-during-render, and replacement-during-assist-error cases.

### BL-02 — Central reconciliation must treat absence as invalidation and run before every item assignment

**Dimension:** key links / sibling-path coverage  
**Required property:** Every admin item response must pass through one reconciliation boundary before `state.currentItem` or review state is assigned; an active reviewed image is authoritative only if the returned item still contains that image ID with the exact `mediaRevision`. Missing image and token mismatch both invalidate gesture, async work, pending actions, and the review before the new item becomes current.

**Evidence:** The addendum says to compare “the active reviewed image's returned `mediaRevision`” and gives only `replaceImage` an explicit before-assignment rule (`08-PR-263-ROUND3-REVISED-PLAN.md:193-196`). Its inventory then claims remove teardown is an existing invalidation (`:211`). The current `renderEditor` clears a review only when the item ID changes (`controller/static-admin/admin.js:1283-1291`); same-item deletion leaves no returned image/token to compare. Existing upload, primary, delete, replacement, and cleanup call sites also assign `state.currentItem` before calling `renderEditor` (`admin.js:2712-2715`, `2735-2736`, `2753-2754`, `2785-2787`, `2808-2809`). The plan therefore leaves a literal implementation path where a missing reviewed image bypasses comparison or the new item is assigned before invalidation, contrary to the required sibling-path contract.

**Example revision (non-binding):** Specify one `reconcileAdminItemResponse(item)` boundary used by every item-returning path. It should locate the active reviewed image, treat absence or revision mismatch as invalidation, settle the pending-review policy, and only then assign/render the item. Add deferred delete plus upload/primary/cleanup sibling cases, and assert invalidation occurs before assignment/render side effects when required.

### BL-03 — Rollback must restore the complete original adjustment snapshot or explicitly invalidate it

**Dimension:** cross-contract data consistency  
**Required property:** If replacement metadata rolls back to the original private object, the persisted adjustment baseline and client/review authority must also match the pre-replacement snapshot; a failed replacement may not silently clear the saved adjustment while restoring a `mediaRevision` identical to the old review.

**Evidence:** The plan requires the original revision to return after cleanup-warning rollback and permits an exact-revision review to continue (`08-PR-263-CONVERGENCE.md:142`; revised plan `:189`, `:219`, `:221`). It also says replacement continues to reset adjustment metadata to `None` (`08-PR-263-ROUND3-REVISED-PLAN.md:197`) without distinguishing successful replacement from rollback. The current rollback calls `replace_image_metadata(existing_image.clone())` (`controller/src/routes.rs:1358-1367`), but both repositories force the adjustment to `None`: memory at `controller/src/catalog.rs:897-902` and Oracle through `adjustment_json = null` at `controller/src/oracle_catalog.rs:48-62`. Thus rollback can restore the original object key and therefore the original token while losing the original adjustment. The still-open old-revision client can then retain a stale saved baseline even though the server changed it.

**Example revision (non-binding):** Define rollback as restoration of the complete original image snapshot, including adjustment, without applying successful-replacement clearing semantics; alternatively define a distinct authoritative invalidation mechanism that cannot reuse the old revision. Test a non-identity saved adjustment through cleanup-warning persistence failure in memory and Oracle-contract coverage, asserting persisted object, adjustment, returned/error contract, and subsequent admin token all agree.

## Warnings

### WR-01 — Conflict, not-found, and repository/provider error classification needs an explicit matrix

**Dimension:** verification derivation  
**Required property:** Automated evidence must distinguish an expected-object-key mismatch (409) from a missing item/image (404) and an actual repository/provider failure (500), with the stable redacted conflict body asserted separately.

**Evidence:** The prose requires zero affected rows with an existing image to be a conflict (`08-PR-263-ROUND3-REVISED-PLAN.md:187`), but the regression matrix only asserts the conflict case (`:217`, `:220`). The current string-based `repository_update_error_status` maps only validation, “not found,” and fallback 500 (`controller/src/routes.rs:2017-2031`), so implementing a new conflict class without dedicated negative tests can easily collapse into 404 or 500.

**Example revision (non-binding):** Add focused repository/route cases for mismatched expected object key, absent image, absent item, and injected repository failure, asserting 409/404/404/500 plus redacted response bodies.

### WR-02 — Revision-change proof must hold all public/basic metadata and bytes constant

**Dimension:** verification derivation  
**Required property:** Automated evidence must prove that a successful same-ID replacement changes `mediaRevision` even when UUID, bytes/checksum/ETag, content type, and byte size are unchanged, leaving only the private object identity different.

**Evidence:** The design includes the object key in the digest (`08-PR-263-ROUND3-REVISED-PLAN.md:178`), but the matrix only holds content type and byte size constant (`:218`). Different replacement bytes can change the checksum and let a derivation that accidentally omits object key pass. That would fail the stated invariant that replacement itself changes revision and weaken the rollback/identity proof.

**Example revision (non-binding):** Replace an image with identical bytes and metadata under the newly generated replacement object key, assert the token changes, then roll metadata back and assert the original token is restored without exposing either object key or checksum.

## Structured Issues

```yaml
issues:
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "key_links_planned"
    severity: "blocker"
    required_property: "Every post-validation terminal path returns media-revision conflict when replacement supersedes the operation, including failed reads/renders/assist"
    description: "The addendum revalidates only before successful responses; current read/render/assist error branches return 500 immediately, so concurrent replacement can be misreported and leave stale client authority mounted."
    fix_hint: "Revalidate before finalizing both success and error branches, with 409 taking precedence on revision mismatch."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "key_links_planned"
    severity: "blocker"
    required_property: "Every admin item response checks active-image presence and exact revision before assigning current item or rendering"
    description: "The plan specifies token comparison but not missing-image invalidation, incorrectly calls remove invalidation existing, and current item-returning call sites assign state.currentItem before renderEditor."
    fix_hint: "Centralize response reconciliation, treat absence and mismatch as invalidation, and route all item-returning paths through it before assignment."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "cross_plan_data_contracts"
    severity: "blocker"
    required_property: "Rollback to the original media revision restores a matching original adjustment baseline or otherwise creates explicit invalidation authority"
    description: "Current rollback reuses replacement methods that clear adjustment metadata, while the plan restores the original token and permits exact-revision review continuity; persisted and client baselines can diverge."
    fix_hint: "Restore the complete original image snapshot on rollback and test a non-identity adjustment, or define an equivalent invalidation contract."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "verification_derivation"
    severity: "warning"
    required_property: "Tests distinguish media conflict, missing resource, and internal failure response classes"
    description: "The matrix proves 409 only; it has no 404/500 negative controls despite the current generic string-to-status mapper."
    fix_hint: "Add route/repository tests for mismatch, missing item/image, and injected failure with exact statuses and redacted bodies."
  - plan: "08-PR-263-ROUND3-REVISED-PLAN"
    dimension: "verification_derivation"
    severity: "warning"
    required_property: "Revision-change tests isolate private object identity as the only changed derivation input"
    description: "The matrix holds only content type and size constant, so checksum changes could mask omission of object key."
    fix_hint: "Use identical replacement bytes and metadata, then assert replacement changes and rollback restores the token."
```

## Required Re-review Evidence

Before `reassessment_required` can change to `ready_to_resume`, revise the media addendum and convergence failure matrix to close BL-01 through BL-03 and WR-01 through WR-02, then obtain a new independent plan review with zero findings. Record that final artifact path and PR comment URL in the convergence frontmatter.

No implementation or tests were changed or executed during this plan-only review.
