---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: 98ee2e1eb1e5e8654af56c49182a97010758d1b0
reviewed: 2026-10-03
verdict: approved
blockers: 0
warnings: 0
info: 0
---

# PR 263 Round 3 Revised Plan Review

## Verdict

**APPROVED — zero blockers, zero warnings, and zero advisories.**

The convergence reassessment and revised implementation plan are implementation-ready for one coherent coder pass. The plan preserves the Round 1–3 finding lineage, revises the failed assumptions, covers the complete open invariant set, and specifies decisive executable evidence for the remaining Critical and Warning findings.

## Goal-Backward Coverage

| Required property | Evidence in the reassessment and revised plan | Verdict |
|---|---|---|
| Finding lineage remains attributable across rounds | `08-PR-263-CONVERGENCE.md` classifies every inherited and Round 3 finding as resolved, incomplete fix, sibling-path miss, fix regression, test weakness, or open; the revised plan targets only the four open findings. | Covered |
| Mounted preview authority covers every producer, consumer, invalidation boundary, reporting path, and test seam | The authority tuple includes review session, item/image, draft revision, preview request/blob identity, output-render generation, and exact mounted node. It governs preview status, displayed revision, Save availability, stage content, messages, retry, rerender, replacement, teardown, and detached-node tests. | Covered |
| Network ordering and DOM render-instance ordering remain distinct | The plan explicitly keeps preview request revision separate from output-render generation. Every adjusted-output mount and teardown advances render authority, including same-draft comparison and overlay rerenders. | Covered |
| Blob and node ownership remain safe | The plan binds callbacks to the exact mounted node and current blob identity, makes superseded callbacks inert, and restricts revocation to URLs no longer owned by a current render. | Covered |
| Perspective geometry remains normalized and responsive | Normalized corners are the sole model. A centralized projection runs after intrinsic load, normalized mutation, and observed frame-size change; visual-center clamping or an overlay gutter preserves 44px targets without changing source coordinates. | Covered |
| Save/Reset is serialized with every named egress path | One review-scoped pending-mutation guard is required in Back, tabs, item selection/navigation, logout, and other route/navigation handlers, with both presentation disabling and handler-level refusal. No discard prompt is allowed for submitted work. | Covered |
| Mutation settlement is reconciled before egress resumes | Success updates current item, saved baseline, draft/dirty state, publish-boundary reporting, messages, and controls. Failure retains the review and baseline, reports the error, and reopens egress consistently. | Covered |
| The harness models browser user events and decisive interleavings | User dispatch must ignore disabled controls, programmatic dispatch must be explicit, and the test matrix retains detached nodes, drives old/new load/error orderings, changes frame rectangles through a fake `ResizeObserver`, and defers Save/Reset across Back/tab/item/logout attempts and success/failure settlement. | Covered |
| The work is bounded as one coherent contract change | The plan modifies the existing admin lifecycle, CSS, and established DOM/Rust contract tests; it excludes resolved publisher, migration, Oracle, and public-media contracts unless regression evidence requires them. | Covered |

## Implementation Readiness Checks

- The current implementation seams match the plan: `renderImageReview()` recreates same-revision image nodes, preview requests already have a separate revision, perspective pixels are currently projected only on load/mutation, Save/Reset currently expose a pending flag, and all named egress paths are centralized enough to share and independently enforce a guard.
- The plan closes the Round 3 failure matrix rather than prescribing isolated point fixes.
- Verification includes executable JavaScript behavior tests, static admin contracts, focused admin workflow tests, the retained publisher failure matrix, full Cargo tests, production-persistence checks, Clippy, formatting, syntax checks, and diff hygiene.
- The non-goals prevent scope drift into already-closed publisher, release migration, persistence, and public media contracts.

## Resume Recommendation

Record this artifact and its PR comment as the independent plan-review evidence in `08-PR-263-CONVERGENCE.md`, then change the convergence status to `ready_to_resume`. One coherent coder pass may proceed, followed by the required lineage-preserving deep source review.
