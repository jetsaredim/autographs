---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: b80f4975a4843c16267c469480d60c2e8ce03e7b
reviewed: 2026-10-05
review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-5996956707
verdict: approved
blockers: 0
warnings: 0
info: 0
---

# PR 263 Media Revision Authority Plan Review — Iteration 2

## Verdict

**APPROVED — zero blockers, zero warnings, and zero advisories.**

The revised media-revision authority addendum closes all three blockers and both warnings from the first review. It is implementation-ready as one coherent coder pass while `08-PR-263-CONVERGENCE.md` remains `reassessment_required` until this artifact and its PR comment are recorded as the independent review evidence.

## Prior Finding Closure

| Prior finding | Revised-plan evidence | Verdict |
|---|---|---|
| BL-01: post-validation covered only successful media work | One common terminal finalizer now re-loads and validates authority on every success and error path. Revision mismatch takes precedence as the stable 409; original read/render/assist success or redacted 500 is returned only while the revision remains current. The matrix explicitly covers replacement-induced source-read, derivative, and assist failures. | Closed |
| BL-02: missing reviewed images and preassignment bypassed reconciliation | One `reconcileAdminItemResponse(item)` boundary must run before any `state.currentItem` assignment or editor render. It treats missing reviewed image and revision mismatch identically, invalidates gesture/async/mutation authority first, and covers upload, primary, remove, replace, cleanup retry, Save, Reset, item save/load refresh, and sibling direct render calls. | Closed |
| BL-03: rollback could restore the original token while clearing its adjustment | Repository replacement now persists the adjustment supplied in `ImageReplacementInput`. Normal replacement explicitly supplies `None`; rollback restores the complete original `AutographImage`, including the saved adjustment. Partial restoration is an error with no success-shaped item or token. The matrix starts from a non-identity adjustment and checks persisted object, adjustment, response contract, and token. | Closed |
| WR-01: 409/404/500 classification lacked negative controls | The plan requires a typed media-revision conflict and stable redacted 409 body, while missing image/item remain 404 and injected repository/provider failures remain 500. The matrix asserts all classes independently. | Closed |
| WR-02: replacement identity test could pass because checksum changed | The matrix now replaces with byte-identical media and identical checksum/ETag, content type, byte size, alt text, primary state, and sort order. Only the random private object identity changes; rollback must restore the original token. | Closed |

## Goal-Backward Coverage

| Required property | Evidence | Verdict |
|---|---|---|
| A same-ID replacement has immutable, opaque review authority | `mediaRevision` is a domain-separated SHA-256 digest over server-private media identity fields including object key. Only the digest enters authenticated admin responses; raw key/checksum and the token remain absent from public output. | Covered |
| Admin response and request contracts carry the correct token | Authenticated admin item/image and review responses expose the token. Source/private/draft/assist/Save/Reset contracts carry the expected revision; review session, preview request, output node, gesture, mutation, and deferred-render authority all require it. | Covered |
| Replacement during read/render/assist cannot publish or misclassify old work | Pre-validation runs before private work; the common terminal finalizer revalidates every success/error outcome. A changed revision always wins as 409, so old results cannot mount or appear as ordinary provider retry failures. | Covered |
| Save/Reset close the validation/write race atomically | Routes retain the validated server-private object key. Memory compares it under the item lock; Oracle includes it in the adjustment update predicate. Canonical no-op paths revalidate, and zero-row mismatch is a typed conflict distinct from missing/error. | Covered |
| Every client item reconciliation invalidates before assignment when required | The central response boundary checks reviewed-image presence and exact token before current-item assignment or editor rendering. Replacement begun before review and every sibling item-returning path use the same boundary; 409 also tears down stale authority without retrying old geometry. | Covered |
| Cleanup branches preserve one coherent persisted snapshot | Successful replacement clears adjustment; cleanup-warning success reports the committed replacement; rollback restores the complete original object and adjustment snapshot. Returned token or error contract cannot describe a different persisted object. | Covered |
| The concurrency matrix proves the authority rather than only happy paths | The matrix covers replacement before source load/first move/between moves, during preview/assist, across Save/Reset route-to-repository boundaries, error-path revalidation, missing-image reconciliation, every item-returning mutation, external-session conflicts, byte-identical replacement, rollback, privacy, and retained gesture/publisher regressions. | Covered |

## Implementability and Boundedness

- The route seams already converge through `load_admin_image`, `image_preview_response`, and the preview/assist handlers, so a shared terminal authority finalizer can govern success and error branches without duplicating policy.
- The repository change is bounded to the adjustment mutation result/expected-key contract and replacement snapshot persistence in the in-memory and Oracle adapters. A typed conflict removes dependence on the generic substring mapper while leaving unrelated repository failures unchanged.
- The client already has centralized review invalidation and item-render concepts; requiring all item-returning call sites to use one pre-assignment reconciliation boundary is a coherent replacement for their current direct assignments.
- Existing integration and DOM harnesses expose deferred fetches, replace/delete/upload paths, fake media stores, repository seams, image events, and current-item rendering. The specified route/repository/client races are testable without a new service or architecture.
- The work remains within the existing Rust controller, Oracle adapter, plain admin JavaScript, and established tests. It does not broaden public media, publisher promotion, legacy-release migration, authentication, or CDN scope.

## Full-Addendum Recheck

- Opaque-token derivation, privacy, and public-output exclusion are mutually consistent.
- Review response/session, source/private preview, draft preview, assist, adjusted-output callbacks, Save, Reset, replacement, and all item-returning mutations have explicit producer, consumer, invalidation, and test paths.
- Revision conflict has one stable server/client meaning and cannot collapse into not-found, provider failure, or ordinary preview retry.
- Rollback is now complete rather than token-only, so restoring the old revision cannot leave a divergent adjustment baseline.
- Existing gesture barrier, mounted-output generation, pointer capture, resize/grab-offset, pending-mutation egress, publication, legacy migration, and privacy contracts remain required regressions.

## Resume Recommendation

Record this artifact and its PR comment as `implementation_plan_review_evidence` and `implementation_plan_review_comment` in `08-PR-263-CONVERGENCE.md`, then change convergence status from `reassessment_required` to `ready_to_resume`. One coherent coder pass may proceed, followed by the required lineage-preserving deep source review with zero Critical, Warning, and Info findings.

No implementation or tests were changed or executed during this plan-only review.
