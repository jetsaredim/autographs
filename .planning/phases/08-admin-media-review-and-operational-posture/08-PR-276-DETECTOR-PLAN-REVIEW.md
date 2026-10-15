---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_at: 2026-10-13T22:16:21Z
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
reassessment_iteration: 3
review_cycle: 2
reviewed_assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
reviewed_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
prior_cycle_result: blocked_2
status: passed
blockers: 0
warnings: 0
info: 0
---

# PR #276 Detector Convergence Plan Review — Iteration 3, Cycle 2

## VERIFICATION PASSED

**Verdict:** PASS

**Issues:** 0 Blocker, 0 Warning, 0 Info

Both cycle-1 blockers are resolved, the iteration-3 gradient/Hough plan remains
goal-complete, and no previously passing coverage regressed. The assumptions,
revised plan, and convergence record may move to `ready_to_resume` with this
file as the fresh independent-review evidence.

## Blocker resolution

### 1. Exact geometric bounds — resolved

The assumptions and plan now use one exact coordinate contract matching spike
009b source rather than the ambiguous phrase “four pixels outside.” Candidate
intersections use imageproc continuous geometric image coordinates with image
boundary `[0,width] × [0,height]` and inclusive admission interval
`[-4,width+4] × [-4,height+4]`. The text explicitly distinguishes that boundary
from pixel centers ending at `width-1` / `height-1`.

Task 2 adds the private unit test
`gradient_candidate_bounds_match_imageproc_geometric_coordinates`, which binds
acceptance at `-4`, `width`, `height`, `width+4`, and `height+4`, rejects values
immediately outside the interval, requires inclusive comparisons, and forbids
deriving the upper bound from the final pixel center. The acceptance and stop
sections repeat the identical inequalities. There is no remaining choice for
an executor to make.

### 2. Comparable performance verification — resolved

The spike’s `0.21s` / `8.2 MiB` prepared-thumbnail smoke is now explicitly
advisory and is not used as a rollback baseline. Task 1 establishes a head
baseline before source changes using a temporary release evaluator that reads
original bytes and invokes public `propose_image_adjustment` exactly 118 times:
the exact positive, exact slab, then all 116 bytewise-sorted originals, retaining
the slab’s corpus occurrence.

Both head and post-change measurements use the same evaluator source, workload
manifest, call order, output mode, timer placement, host, and production path.
Each evaluator is built once, run once as an unrecorded complete warm-up, then
its already-built binary is timed directly; `cargo run`, compilation, and cache
population are excluded. Per-image durations, total, p50, p95, maximum, and
binary maximum RSS are captured.

The mandatory gates are explicit and meaningful:

- post p95 ≤ 1.0s and ≤ 3× head p95;
- post maximum ≤ 2.0s and ≤ 3× head maximum;
- post summed proposal time ≤ 3× head total; and
- post maximum RSS ≤ `max(1.5× head RSS, head RSS + 32 MiB)`.

These combine synchronous-interaction absolute ceilings with identical-workload
relative limits and a bounded memory allowance. The stop section requires a new
reassessment if any gate fails without an accepted measurement explanation.

## Regression check of previously passing coverage

| Required property | Cycle-2 evidence | Result |
| --- | --- | --- |
| Replace rather than layer the scalar veto. | Task 2 removes the weak-threshold loop, `component_boundary_is_stable`, and its population/area constants. The gradient helper is the sole outer-boundary ambiguity veto after primary validation. | Pass |
| Gradient candidates remain veto-only. | Candidate corners cannot enter normalized primary corners, `ImagePerspective`, response data, draft state, persistence, derivatives, or publication. A qualifying candidate only returns ordinary `Unavailable`. | Pass |
| Frozen spike constants and bounded search remain exact. | Canny/Hough, angle groups, line support/gap/rank, 12-per-axis truncation, 66 pairs per axis, 4,356 rectangles, span/area, finite-side evidence, score, and three-side threshold remain unchanged; only the previously ambiguous bounds prose was made exact. | Pass |
| Sanitized fixtures remain causal. | Reliable primary, physical slab/internal card, and Hough-rich/no-localized-contrast fixtures remain binding. The intermediate scalar bypass and final forced-false gradient mutation must restore forbidden internal confidence for the slab. | Pass |
| Exact controls and corpus remain binding. | Exact `b991...` must be accurate `Confident`; exact `20240415_225421.jpg` and all three slab siblings must be `Unavailable`; expected corpus result is 79/37/0; every retained overlay and every changed outcome is assessed; zero internal confidence is mandatory; the rotated side-distance exception is reinspected. | Pass |
| Two-file scope and privacy remain intact. | Only detector source and integration tests may change. Manifest, lockfile, route, browser, and consumer tests are no-diff. Private inputs, paths, evaluator source/binary, masks, overlays, contact sheets, and per-image output remain under `/tmp` and outside Git. | Pass |
| Consumer/session/manual fallback contracts remain complete. | Authentication, initial/terminal media revision, camelCase response, current session/draft/assist authority, mutation/drag guards, non-mutating fallback, manual handles, explicit Save, private-original immutability, and publication boundaries remain unchanged and verified. | Pass |
| Full verification remains complete. | Focused detector/mechanism/bounds tests, route/browser/static contracts, all-feature tests, production-persistence check, Clippy, formatting, diff checks, exact controls, corpus review, performance, and cleanup all remain required. | Pass |
| Stop and rollback satisfy convergence guard. | Any mechanism, exact, corpus, count-review, frozen-constant, candidate-cap, performance, scope, consumer, privacy, or artifact failure restores only the two in-scope files and returns to reassessment before another coder round. | Pass |

## Structured issues

```yaml
issues: []
```

## Resume authorization

PASS. The cycle-2 review closes both prior blockers with zero actionable
concerns. Execution may resume only under the complete reviewed task order and
all stated stop conditions; any stop returns convergence to
`reassessment_required` rather than authorizing a point fix.
