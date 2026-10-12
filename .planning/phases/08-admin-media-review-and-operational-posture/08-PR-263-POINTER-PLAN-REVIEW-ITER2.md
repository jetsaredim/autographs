---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: f5b40f4e6d23951dbe5bcddca5fcdd97cbe9c027
reviewed: 2026-10-04
verdict: approved
blockers: 0
warnings: 0
info: 0
---

# PR 263 Pointer Lifecycle Plan Review — Iteration 2

## Verdict

**APPROVED — zero blockers, zero warnings, and zero advisories.**

The revised pointer-lifecycle reassessment is implementation-ready for one coherent coder pass. It closes every finding from the first pointer-plan review and preserves the required lineage, authority boundaries, closed contracts, and bounded scope.

## Prior Finding Closure

| Prior finding | Revised-plan evidence | Verdict |
|---|---|---|
| Active-drag producers lacked one deterministic policy and terminal contract | The per-producer table selects in-place, reject, defer, or teardown behavior for source load/error, preview success/failure, adjusted-output load/error, assist, synchronous controls/actions, resize, and review invalidation. The terminal-state table selects commit, flush, supersede, or discard semantics for pointerup, pointercancel, external lost capture, source error, and review/session/item/image invalidation. | Closed |
| Grab-offset authority was undefined before intrinsic source geometry existed | Source projection readiness is now a hard precondition: handles are absent or disabled until intrinsic dimensions and fitted bounds are non-zero. Pointerdown before readiness cannot create a gesture. After readiness, the plan defines `grabOffset = authoritativeCorner - pointerPosition` in normalized fitted-source coordinates. | Closed |
| Resize behavior was unresolved | Resize is explicitly in-place. It stores the last client coordinates, reprojects the same connected handles, and rebases the normalized grab offset as `currentCorner - normalized(lastPointer)` under the new fitted bounds. Resize itself cannot mutate the model, and the next delta uses current geometry. | Closed |
| Source-image error was absent from the matrix | Source error has a selected producer policy, terminal behavior, and before/after-first-move regressions. It terminates once before frame replacement, preserves accepted geometry, reconciles preview work once, and leaves no enabled handles. | Closed |
| Pointer capture release and reentrant lost capture were not modeled | Termination marks the gesture terminal before listener removal and conditional capture release. Reentrant `lostpointercapture` is required to observe terminal state and do nothing. The regression matrix requires exactly one settlement and preview/deferred-render reconciliation. | Closed |

## Goal-Backward Coverage

| Required property | Evidence | Verdict |
|---|---|---|
| Finding lineage remains accurate | The async capture replacement is retained as an incomplete fix/sibling-path miss; the inset-handle jump remains a fix regression/test weakness. No previously closed finding is reopened without evidence. | Covered |
| Every stage-replacement producer respects active-drag ownership | The plan requires inventorying every direct `renderImageReview()` caller and indirect replacement callback, routing it through one barrier or proving that it mutates only connected geometry in place. The producer table covers the current source, preview, adjusted-output, assist, retry/control, resize, and teardown seams. | Covered |
| Deferred work retains complete authority | Reconciliation must preserve request revision, blob ownership, output-render generation, exact mounted node, review session, item/image, draft revision, and perspective generation. Pre-gesture outputs/errors are explicitly superseded by a moved gesture; authoritative no-move work flushes once; retired-review work is discarded. | Covered |
| Gesture geometry is reversible and stable | The normalized corner is authoritative; inset handle pixels remain presentation-only. Same-coordinate movement is a no-op, source-space deltas map to matching normalized deltas with clamping, portrait/landscape fitted bounds are exercised, and resize rebasing prevents a model jump. | Covered |
| Every terminal path is deterministic and idempotent | Pointerup, pointercancel, external/reentrant lost capture, source error, review teardown, and session/item/image invalidation have explicit cleanup and model/deferred-work outcomes. Terminal marking precedes release, preventing double settlement. | Covered |
| Tests are browser-faithful at the relevant seams | The matrix rejects pre-readiness and disconnected-node user input, retains explicit programmatic dispatch only for intentional stale/guard tests, controls async producer order, uses connected capture owners across moves, exercises source error and resize, and requires capture-release/reentrant-lost-capture proof. | Covered |
| Work remains one coherent bounded change | The implementation remains confined to the established static admin lifecycle/CSS and behavior/contract tests. Publisher promotion, legacy migration, Oracle persistence, public media, privacy, dirty-state, and publish-boundary contracts remain protected non-goals with retained regression coverage. | Covered |

## Implementation Readiness

- Source readiness removes the impossible pre-load coordinate basis while retaining an in-place redundant-load path.
- The producer table and terminal table together define queue ownership, blob/render/session/node authority, and exactly-once reconciliation rather than leaving choices to coder discretion.
- The coordinate equations cover all four inset edge handles, zero-distance moves, clamping, portrait/landscape letterboxing, and resize during capture.
- The decisive matrix directly exercises both surviving review findings and every relevant termination family, including source error and reentrant lost capture.
- The full syntax, executable DOM, focused Rust, publisher, all-target, production-persistence, Clippy, formatting, and diff-hygiene verification set remains required.

## Resume Recommendation

Record this artifact and its PR comment as the independent implementation-plan review evidence in `08-PR-263-CONVERGENCE.md`, then change its status from `reassessment_required` to `ready_to_resume`. One coherent coder pass may proceed, followed by the required lineage-preserving deep source review with zero Critical, Warning, and Info findings.

No implementation or tests were changed or executed during this plan-only review.
