---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed: 2026-10-14T10:00:15Z
reviewed_head: 2e2dc2356a9c8a4b3a9ccb6ba9b97056da19b63e
depth: deep
review_round: post-reassessment
files_reviewed: 2
files_reviewed_list:
  - controller/src/image_adjustments.rs
  - controller/tests/image_adjustments.rs
findings:
  critical: 0
  warning: 0
  info: 1
  total: 1
status: clean
convergence:
  status: accepted_risk_deferred
  recurring_critical_warning: 1
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
  checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
  checker_status: passed_cycle_2_then_invalidated_by_implementation_regression
  independent_review_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
  reassessment_iteration: 3
  design_spike_evidence:
    - .planning/spikes/009-b-local-gradient-rectangle/README.md
  pr_evidence: https://github.com/jetsaredim/autographs/pull/276#discussion_r4269355141
  backlog_issue: https://github.com/jetsaredim/autographs/issues/277
---

# Phase 08: PR #276 Deep Code Review — Post-Reassessment Gradient Detector

**Reviewed:** 2026-10-14T10:00:15Z
**Reviewed commit:** `2e2dc2356a9c8a4b3a9ccb6ba9b97056da19b63e`
**Approved-contract base:** `af6c769`
**Prior detector checkpoint:** `85db903`
**PR base:** `origin/main` (`60eb329df15ca76106298325cd02bed523ac3ad3`)
**Depth:** deep
**Status:** clean with one accepted advisory-detector limitation
**Convergence:** accepted risk deferred to #277

## Product disposition — 2026-10-14

The operator accepted WR-01's thin sparse-outline morphology as out of scope
for Phase 8 and deferred it to GitHub issue #277. Auto-detect only mutates an
unsaved review draft, requires operator inspection and an explicit Save, and
has a manual four-corner fallback. The limitation was not observed in the
116-image production corpus; the exact supplied positive and named slab still
pass their gates.

The finding remains preserved below as audit history, but is reclassified from
an in-scope Warning to an Info-level accepted limitation. PR #276 is not blocked
on further detector research. The misleading sparse-test name and fixture name
were changed to describe the filled slab they actually exercise; the deferred
issue retains the exact one-pixel reproduction and future acceptance criteria.

## Summary

The implementation ports spike 009b's frozen gradient/Hough constants and
geometry faithfully, removes the obsolete scalar weak-mask veto, keeps the
search bounded to 12 lines per axis and 4,356 pair combinations, and never
promotes gradient candidate corners into a proposal. The exact private positive
still returns accurate `Confident` corners and the named slab returns
`Unavailable`; the committed detector, coordinate-bound, browser-consumer,
formatting, and strict Clippy checks pass.

However, the implementation regresses the original sparse-enclosure case that
caused round-2 WR-01. The committed test retains the old name but replaces its
one-pixel weak outer rectangle with a filled low-contrast slab. Re-running the
exact prior fixture through production `propose_image_adjustment` at `2e2dc23`
returns `Confident` on the dark internal region. This violates the approved
acceptance requirement to preserve the sparse enclosure as `Unavailable` and
recurs in the same false-confidence family after reassessment. Another local
constant adjustment is not authorized; the assumptions and coherent detector
contract must be reassessed with the original morphology restored as a binding
regression.

## Narrative Findings

## Accepted limitations

### INFO-01 (formerly WR-01): Thin sparse enclosure can return internal `Confident`

**File:** `controller/tests/image_adjustments.rs:425`
**Finding family:** `auto_assist_outer_boundary_confidence`
**Lineage:** `recurrence` / `fix_regression` / `test_weakness` (not an independent finding)
**Shared invariant:** A `Confident` proposal must represent the stable outer
item boundary, never a higher-contrast internal region; the previously bound
sparse outer enclosure must remain `Unavailable` even when its weak edge is
too thin or low-contrast to form a filled slab.
**Consumers inspected:** primary adaptive component and corner production;
gradient/Hough line selection and candidate veto; proposal classification;
unchanged route serialization and terminal media-revision guard; unchanged
browser current-session/current-draft/four-corner confidence gate; detector,
route, and browser regressions.
**Evidence level:** `deterministic_test`

**Issue:** At `85db903`, `sparse_weak_outer_boundary_fixture` consisted of a
one-pixel `[228,228,228]` rectangle from `(18,10)` through `(161,129)`, a
one-pixel bridge to the internal component, and dense dark content from
`(50,40)` through `(129,99)`. At `2e2dc23`, lines 431-442 replace that sparse
geometry with a fully filled `[210,210,210]` slab plus adaptive-threshold border
noise. The renamed body is therefore another filled physical-slab case, not the
required sparse regression. It also substantially overlaps the fixture already
used by both low-contrast slab tests at lines 183-208.

Running the exact `85db903` fixture bytes through the current public
`propose_image_adjustment` path deterministically returns `Confident` with the
internal corners approximately `(0.275430,0.279678)`,
`(0.719875,0.279678)`, `(0.719875,0.712274)`, and
`(0.273865,0.712274)`. The original outer transition is only 14 luma levels,
below the frozen Canny low threshold of 20, so the new veto has no qualifying
outer gradient rectangle while the primary producer still selects the dark
internal component. The green test at lines 219-225 therefore masks rather
than prevents the original false-confidence behavior.

This directly contradicts the reviewed plan's requirements to preserve all 15
existing detector tests, retain the sparse/filled/noisy enclosure contracts,
and keep the sparse enclosure `Unavailable`. It also demonstrates that the
79/37 corpus result does not cover this already-known morphology.

**Disposition:** Accepted as an advisory-detector limitation for Phase 8 and
deferred to GitHub issue #277. Any future correction must restore the original
sparse fixture as a binding regression and satisfy the complete issue gates;
do not tune a frozen threshold in isolation.

**PR audit trail:** [inline review comment](https://github.com/jetsaredim/autographs/pull/276#discussion_r4269355141)

## Finding Lineage

| Stage | Head/evidence | Lineage | Result |
| --- | --- | --- | --- |
| Round 1 | `b2d947c` | evidence gap | The 116-image corpus invalidated the initial clean synthetic review. |
| Round 2 | `36946e7` | `incomplete_fix` | Population and area joined by `&&` trusted the sparse one-pixel enclosure. |
| Round 3 | `85db903` | `fix_regression` | `||` closed sparse confidence but falsely abstained on the exact dark-card positive, triggering mandatory reassessment. |
| Reassessment iteration 3 | assumptions, plan, checker, spike 009b | approved redesign | Frozen local-gradient/Hough veto approved with sparse/filled/noisy preservation explicitly required. |
| Post-reassessment review | `2e2dc23` | `recurrence` / `fix_regression` / `test_weakness` | Exact prior sparse fixture again returns internal `Confident`; committed fixture was changed into a filled slab. |

WR-01 is not a new independent algorithm concern. It is recurrence of the
same shared invariant and a test-coverage regression introduced while porting
the approved redesign.

## Shared Invariant Audit

| Boundary | Evidence | Assessment |
| --- | --- | --- |
| Primary producer | Border median, adaptive threshold, largest 8-connected component, extrema corners, projection and area validation remain unchanged. | Correct for the approved primary contract, but still selects the dark internal component in the sparse fixture. |
| Gradient producer | Canny `20/60`; Hough vote `min-dimension/3`; suppression `8`; approved axis windows. | Exact frozen constants. The sparse 14-level transition is absent by construction. |
| Line/candidate bounds | Full-line `0.25/0.55`, 12-per-axis truncation before pairing, pair span `0.50`, inclusive `[-4,width+4] × [-4,height+4]`, area `> max(0.50, primary+0.05)`. | Exact and bounded; coordinate unit test passes. |
| Side classifier | Two-pixel corridor, four-pixel contrast offset, side `0.35/0.40/5/0.60`, at least three valid sides. | Matches spike 009b. No candidate exists for the prior sparse case. |
| Proposal authority | Gradient helper only returns a boolean veto; original primary corners remain the only confident output. | Correctly veto-only, but the missing sparse signal lets the internal primary escape as `Confident`. |
| API/browser consumer | Route and admin JavaScript are unchanged from the reviewed authority contract. | Safe consumption: `Unavailable` has no corners and only a current four-corner `Confident` response mutates the unsaved draft. This limits consequence but does not make false confidence acceptable. |
| Regression tests | 17 committed tests pass. | Incomplete: the sparse test name remains but its defining morphology was deleted; two tests use the same slab fixture. |

## Failure Matrix

| Input/outcome | Required | `2e2dc23` |
| --- | --- | --- |
| Exact `b991...webp` positive | Accurate `Confident` near the approved corners | Pass: `(0.0614,0.0610)`, `(0.9254,0.0563)`, `(0.9386,0.9343)`, `(0.0570,0.9327)`. |
| Exact `20240415_225421.jpg` slab | `Unavailable`, no corners | Pass. |
| Committed filled physical slab | `Unavailable`, no corners | Pass. |
| Committed Hough-rich accurate card | Accurate `Confident` | Pass. |
| Original one-pixel sparse enclosure from `85db903` | `Unavailable`, no corners | **Fail: internal `Confident`.** |
| Near-identity inset 2 / inset 5 | `NotNeeded` / `Confident` | Pass in committed tests. |
| `Unavailable` API/UI consumption | No applied corners; manual fallback | Unchanged and passing. |
| Bounded search/performance | 12 lines per axis, ≤4,356 combinations; approved benchmark gates | Static bounds pass; PR evidence reports baseline/post gates passing. |

## Assumption Audit

| Assumption | Evidence | Verdict |
| --- | --- | --- |
| Spike 009b's frozen gradient signal covers every previously required sparse/filled/noisy enclosure. | It covers filled/slab corpus cases, but the exact prior one-pixel sparse fixture returns internal `Confident`. | Rejected. |
| Preserving the sparse test name preserves its contract. | The fixture implementation changed from a one-pixel rectangle and bridge to a filled slab. | Rejected. |
| Corpus equality 79/37/0 is sufficient regression evidence. | The production corpus is valuable and exact controls pass, but the known sanitized sparse morphology is outside that result. | Rejected as a complete acceptance argument. |
| Gradient candidates remain veto-only and bounded. | Direct source trace and coordinate/boundedness checks. | Accepted. |
| Route/UI safety contains false-confidence impact. | Assist only mutates an unsaved draft and persistence requires explicit Save. | Accepted for containment, not detector correctness. |

## Deferred follow-up criteria (#277)

1. Restore the exact one-pixel sparse enclosure and bridge morphology as a
   committed regression; do not reuse the filled slab fixture for that test.
2. Revise the detector assumptions to include coherent physical boundaries
   whose local contrast is below the frozen Canny threshold but whose topology
   previously demonstrated an enclosing item boundary.
3. Produce one coherent revised design that separates this sparse enclosure
   from the exact accurate dark-card positive; do not append a scalar exception
   or tune one frozen constant against the fixture.
4. Re-run the exact positive, named slab, all three slab siblings, Hough-rich
   accurate primary, restored sparse regression, all retained overlays, frozen
   corpus, and warmed 118-call performance gates.
5. Obtain fresh review of the revised assumptions and implementation plan
   before future detector source work resumes. These are backlog acceptance
   criteria, not blockers for PR #276.

## Verification

- Exact prior sparse fixture through public `propose_image_adjustment` at
  `2e2dc23` — **failed**: internal `Confident` with four corners.
- Exact private `b991...webp` positive — passed with target accurate
  `Confident` corners.
- Exact private `20240415_225421.jpg` slab — passed as `Unavailable` with no
  corners.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture`
  — passed, 17 tests.
- `cargo test --manifest-path controller/Cargo.toml --lib gradient_candidate_bounds_match_imageproc_geometric_coordinates -- --nocapture`
  — passed, 1 focused test.
- `node --test controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
  — passed.
- No-diff check for manifest, lockfile, route, admin JavaScript, and consumer
  tests from `85db903..2e2dc23` — passed.
- PR implementation evidence reports the frozen 116-image result `79/37/0`
  and all warmed release performance gates passing; this does not override the
  deterministic sparse regression.

## Changed Paths

- `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
- `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md`
- `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md`
- `controller/tests/image_adjustments.rs`

No production detector code was modified by this disposition. The test-only
change removes a misleading sparse-coverage claim; it does not change behavior.

---

_Reviewed: 2026-10-14T10:00:15Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep, post-reassessment gradient detector review_
