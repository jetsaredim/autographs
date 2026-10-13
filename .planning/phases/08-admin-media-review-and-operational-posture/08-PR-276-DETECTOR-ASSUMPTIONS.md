---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
finding_family: auto_assist_outer_boundary_confidence
status: ready_to_resume
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
revised_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
reassessment_iteration: 3
checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
checker_status: passed_cycle_2
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
spike_evidence:
  - .planning/spikes/008-detector-mask-contour-diagnostics/README.md
  - .planning/spikes/009-a-actual-contour-boundary/README.md
  - .planning/spikes/009-b-local-gradient-rectangle/README.md
pr_evidence: https://github.com/jetsaredim/autographs/pull/276#issuecomment-6156047878
---

# PR #276 Detector Assumption Reassessment — Post-Spike Iteration 3

## Scope and checkpoint state

This is the third durable reassessment of PR #276's auto-assist outer-boundary
confidence contract. It is surgical: only the detector in
`controller/src/image_adjustments.rs`, its direct integration tests, and the
unchanged route/browser consumers are in scope. It does not replan Phase 8 or
change the Rust-controller/static-admin architecture.

Spikes 008, 009a, and 009b completed the bounded investigation required by the
previous stop condition. The focused checker cycle resolved both iteration-3
Blockers and passed with no actionable concerns in
`08-PR-276-DETECTOR-PLAN-REVIEW.md`. The current plan and review evidence paths
are recorded above, so status is `ready_to_resume` under every implementation,
exact-control, corpus, performance, consumer, privacy, and rollback gate.

## Finding lineage

| Stage | Head/evidence | Lineage | Result |
| --- | --- | --- | --- |
| Round 1 | `b2d947c` plus 116-image screen | Evidence gap | The held-out corpus exposed confident internal-region proposals absent from the synthetic suite. |
| Round 2 | `36946e7` | `incomplete_fix` | Population and area growth joined by `&&` missed a sparse connected outer enclosure. |
| Round 3 | `85db903` | `fix_regression` | Changing `&&` to `||` closed the sparse case but rejected the exact accurate dark-card positive when weak noise expanded extrema. |
| Round 4 | uncommitted, rolled back | `incomplete_fix` / `test_weakness` | Extrema-chord four-side support passed paired sanitized controls and the exact positive but missed the named slab because `Qt` did not contain every `Q0` corner. |
| Round 5 | uncommitted, rolled back | `incomplete_fix` / `assumption_failure` / `test_weakness` | Replacing polygon containment with `M0 ⊆ Mt` passed all three sanitized morphologies, 17 detector tests, and the exact positive, yet the exact named slab still returned the forbidden internal quadrilateral. The four-side extrema-chord abstraction itself was insufficient. |
| Spike 008 | committed diagnostic spike | `diagnostic_confirmation` | Reproduced masks, anchor components, contours, extrema chords, Canny edges, and Hough lines without committing private output. The slab's weak component flooded the frame at threshold 32 before exposing a usable physical outer contour; local edge evidence remained visible. |
| Spike 009a | committed comparison spike | `candidate_invalidated` | A contour-only policy could not separate the exact controls: ignoring flood kept the slab's internal primary, while abstaining on every flood also rejected the exact positive. |
| Spike 009b | committed comparison spike and PR evidence | `candidate_validated_for_planning` | Frozen local-gradient/Hough evidence retained the exact positive, vetoed the named slab with four valid sides, and produced 79 accurate retained primaries plus 37 safe abstentions across all 116 images with zero known/screened internal confidence. |

The finding family remains the same throughout: `Confident` must mean that the
existing primary corners represent the outer item boundary. The new evidence
does not rehabilitate a scalar weak-mask rule. It validates an independent,
localized physical-boundary signal as a conservative veto.

## Spike evidence and limits

### Spike 008: diagnosis

- The exact positive primary was dense, approximately `0.764` of the frame, and
  changed only slightly through its early weak probes.
- The named slab primary was the known internal card at approximately `0.343`.
  Its weak anchor component became almost full-frame at threshold 32.
- At that flood point, the physical slab edge was absent from the
  border-median component contour even though Canny/Hough still exposed long
  outer horizontal and vertical evidence.
- Therefore a contour can support diagnostics when present, but the current
  threshold mask cannot be the sole production source of the physical boundary.

### Spike 009a: rejected design

- No usable materially larger contour appeared for the slab before its mask
  flooded.
- Ignoring flood returned the internal primary for both controls; treating any
  flood as ambiguity rejected both controls.
- Keying behavior to a particular flood threshold would be another private-image
  scalar tuning. Contour-only is rejected.

### Spike 009b: validated feasibility

- Structural constants were declared before running either exact control and
  remained unchanged through the complete corpus evaluation.
- The exact positive retained its accurate `0.764`-area primary and exposed no
  qualifying larger gradient rectangle.
- The named slab exposed a `0.731`-area independently supported rectangle around
  its `0.343` internal primary; all four candidate sides were valid, so the
  result abstained.
- With the same constants, 79 corpus images kept their primary, 37 safely
  abstained, and none returned `NotNeeded`. All 79 retained overlays were
  visually inspected and none selected an internal region.
- The prior 12%-side-distance screen flagged only `20251231_011140`; full-image
  inspection confirmed its rotated landscape primary follows the outer card.
- All three graded-slab siblings abstained.
- The optimized two-prepared-control spike smoke completed in `0.21s` with
  about `8.2 MiB` peak resident memory on the development host. This establishes
  feasibility only: prepared thumbnails and the spike binary are not a
  production-path performance baseline.

These results validate feasibility and freeze an implementation contract. They
do not prove that a production port is correct until sanitized causal tests,
the exact controls, the full corpus, consumer contracts, and a fresh review all
pass against the production code.

## Shared invariants

The four convergence invariants remain authoritative:

1. A `Confident` proposal represents the stable outer item boundary, never a
   higher-contrast internal card or region.
2. Ambiguity detection distinguishes an independently supported enclosing
   physical boundary from weak-mask population/extrema growth and from raw
   line/noise abundance.
3. Safe abstention is preferred for ambiguity, while known accurate material
   cases remain usable; the exact dark-card control is a binding positive.
4. `Unavailable`, `NotNeeded`, stale, error, and unexpected outcomes do not
   mutate the browser draft. Only a current four-corner `Confident` response may
   update the unsaved perspective draft.

These preserve D-08-25, D-08-26, and D-08-29: originals stay immutable,
assistance is bounded with manual fallback, and persistence still requires an
explicit Save.

## Evidence-backed assumption decisions

### Accepted

| ID | Assumption | Evidence and consequence |
| --- | --- | --- |
| A-01 | The existing adaptive-threshold largest component remains the primary proposal producer. | Spike 009b kept the current primary on all 79 confident results; its gradient rectangle served only as an ambiguity veto. Preserve primary selection, ordering, normalized corners, projection validation, area gate, and near-identity classification. |
| A-02 | Gradient candidate corners are evidence, not proposal output. | The spike validated only whether a materially larger independently supported rectangle exists. Never return, serialize, or apply those corners; a qualifying candidate maps to `Unavailable`. |
| A-03 | Local gradients provide information independent of the flooded border-median mask. | Spike 008 showed no usable slab contour at the flood point while Canny/Hough retained the physical slab lines. Production may use the existing `imageproc 0.27` Canny/Hough APIs without a dependency change. |
| A-04 | Raw Hough lines are insufficient; finite side-local evidence is the discriminator. | Text and texture generated many lines. Spike 009b required finite-segment edge coverage/gap plus inside/outside contrast magnitude and sign coherence, preventing line abundance alone from vetoing a reliable primary. |
| A-05 | The 009b structural contract is frozen as one coherent candidate model. | Constants were fixed before exact-control and corpus evaluation and remained unchanged across all 116 images. Production must port the whole contract without image-specific tuning or a bounded-search expansion. |
| A-06 | Candidate enumeration is predictably bounded. | Filtering and ranking occur before truncation to 12 vertical and 12 horizontal lines. Each axis has at most 66 pairs, so at most `66 × 66 = 4,356` rectangles reach intersection/side evaluation per image. |
| A-07 | Three valid sides are sufficient for a conservative veto under the complete side contract. | The validated rule requires at least three sides meeting edge support, maximum gap, mean local contrast, and sign coherence; the named slab supplied four. The fourth side may be occluded or weak without converting the gradient rectangle into output corners. |
| A-08 | Safe abstentions can include accurate primaries when a larger sleeve or surrounding boundary is independently supported. | The corpus produced 37 safe abstentions. Manual controls remain available, while zero internal confident proposals is non-negotiable. |
| A-09 | The route and browser authority boundaries are already correct and stay unchanged. | The route authenticates, reads private bytes, checks `mediaRevision` before and after analysis, and serializes only status/corners/message. The browser applies only a current four-corner `confident` result under session/draft/assist/mutation/drag guards. |
| A-10 | Held-out private images are acceptance oracles, not repository fixtures. | The exact controls and 116 originals remained outside Git throughout the spikes. Production evaluation must likewise keep inputs, paths, rendered overlays, evaluator source/binaries, and per-image output outside Git. |
| A-11 | A mechanism-derived causal fixture is required even after corpus validation. | The prior sanitized set passed while the real slab failed. The new slab fixture must prove that bypassing the gradient veto exposes the internal primary, while the localized gradient rule changes only the classification to `Unavailable`. |
| A-12 | Spike feasibility is not production authorization. | Spike code is intentionally self-contained and includes rendering/report concerns inappropriate for the controller. Production work may start only after fresh independent review of these artifacts. |

### Rejected

| ID | Rejected assumption | Evidence |
| --- | --- | --- |
| R-01 | Population growth or weak extrema-area growth can independently determine stability. | `&&` missed sparse enclosure; `||` rejected the exact positive. The current `population_growth || area_growth` decision must be replaced, not wrapped. |
| R-02 | Exact anchor-membership retention plus extrema-chord side support is a sufficient replacement. | The round-5 implementation passed three sanitized morphologies and the positive but still trusted the named slab's internal primary. |
| R-03 | A physical slab boundary can always be recovered from the threshold-component contour. | Spike 008 showed the slab component flooding before a usable physical contour emerged; spike 009a failed both contour-only policies. |
| R-04 | Flood threshold itself can classify the controls. | Both exact controls eventually flood. Selecting a distinguishing threshold would tune a scalar to private images. |
| R-05 | Hough line count or infinite-line intersections establish a boundary. | Text/texture create many lines. Only localized finite-side edge and contrast evidence made the corpus-safe result. |
| R-06 | A detected gradient rectangle should replace the primary proposal corners. | No spike evaluated those corners as correction output. Applying them would expand the contract and could introduce unsafe automatic transformations. |
| R-07 | Increasing abstention alone demonstrates detector quality. | The exact positive must remain accurate `Confident`, and every retained corpus proposal requires geometry inspection. |
| R-08 | The spike prototype should be copied wholesale into production. | The prototype reads reports, renders overlays, and serializes evaluation data. Production needs focused private helpers and no new API/reporting surface. |
| R-09 | Corpus counts can replace per-image geometry review. | The 79/37/0 result was accepted only after all retained overlays were visually inspected and the lone side-distance screen exception was confirmed accurate. |
| R-10 | The prior passed plan review authorizes this different algorithm. | That review approved the failed membership/extrema-chord plan. A fresh checker must review the gradient/Hough contract and implementation tasks. |

## Frozen production boundary contract

### Primary proposal remains unchanged

The controller still decodes and thumbnails to a maximum dimension of 640,
computes the border-median RGB reference and adaptive threshold, selects the
largest qualifying 8-connected contrast component, derives its four extrema,
validates the projection and primary area, and later applies the existing
near-full-frame rule. The gradient path never changes `primary.corners`.

The current weak-threshold stability loop and its independent
`population_growth || area_growth` rejection are removed. They are not retained
as a prefilter, fallback, or second veto.

### Gradient/Hough ambiguity veto

On the same 640-pixel RGB thumbnail and the primary's normalized area:

1. Convert to luma and run Canny with low/high thresholds `20/60`.
2. Run the standard Hough transform with vote threshold
   `min(width, height) / 3` and suppression radius `8`.
3. Axis-group lines within 15 degrees: vertical is `0..=15` or `165..180`, and
   horizontal is `75..=105`.
4. Across each infinite in-frame line, measure edge support in a two-pixel
   corridor. Retain only lines with support at least `0.25` and longest
   unsupported gap at most `0.55`.
5. Rank retained lines by `support - 0.25 * longest_gap`, keep at most 12 per
   axis, then sort them by mid-frame position. Truncation precedes pair
   enumeration.
6. Enumerate at most 66 pairs per axis and 4,356 pair combinations. Require
   each vertical and horizontal pair to span at least `0.50` of the corresponding
   image dimension.
7. Intersect the four lines in imageproc's continuous geometric image
   coordinates. The geometric image boundary is `[0,width] × [0,height]`,
   while pixel centers end at `width - 1` and `height - 1`. Port the spike's
   exact inclusive admission interval: every corner must satisfy
   `x >= -4`, `y >= -4`, `x <= width + 4`, and `y <= height + 4`. This is four
   coordinate units beyond the geometric image boundary, not four units beyond
   the final pixel center. Require candidate area to exceed both `0.50` of the
   frame and `primary_area + 0.05`.
8. For each finite candidate side, measure edge support in the same two-pixel
   corridor and the longest unsupported run. Sample luma four pixels inward and
   outward along the center-facing normal; compute mean absolute contrast and
   dominant-sign coherence.
9. A side is valid only with support at least `0.35`, maximum gap at most `0.40`,
   mean local contrast at least `5`, and sign coherence at least `0.60`.
10. Rank candidates by valid-side count, then by
    `area * mean_side_support * valid_side_fraction`. The existence of a
    candidate with at least three valid sides vetoes the primary and returns the
    ordinary `Unavailable` response with no corners.

If no qualifying candidate exists, classification continues using the existing
primary corners. No gradient corner reaches the response, browser, draft,
persistence, derivative generation, or publication path.

## Comparable production-path performance contract

The spike's prepared-thumbnail `0.21s` / `8.2 MiB` smoke is advisory evidence,
not a rollback baseline. Production performance is measured with a temporary
release evaluator under `/tmp` that reads original bytes and invokes the public
`propose_image_adjustment` path. The workload manifest is fixed before source
changes: exact positive first, exact named slab second, then all 116 corpus
originals in bytewise-sorted filename order. The slab remains in the corpus
portion even though it also serves as the explicit negative, yielding the same
118-call sequence before and after.

Before source changes, build the evaluator once against `85db903`, execute its
already-built binary once as a discarded warm-up over the complete manifest,
then execute that binary directly for measurement. The evaluator records each
`propose_image_adjustment` duration plus summed total, p50, p95, and maximum;
`/usr/bin/time` wraps the evaluator binary itself to record maximum RSS. After
implementation, rebuild once, warm once, and measure the already-built binary
with the identical manifest, order, output mode, and host conditions. Neither
measurement includes compilation or Cargo/cache population.

The post-implementation result must satisfy all of these mandatory gates:

- p95 proposal latency is at most `1.0s` and at most `3.0×` the head p95;
- maximum proposal latency is at most `2.0s` and at most `3.0×` the head max;
- summed proposal time is at most `3.0×` the head total; and
- maximum RSS is at most the larger of `1.5×` head RSS or head RSS plus
  `32 MiB`.

p50 and every per-image duration are captured and compared even though the
mandatory latency gates focus on tail and total behavior. The one-second p95 and
two-second maximum protect the synchronous private-admin interaction; the 3×
relative ceilings allow the intentionally bounded Canny/Hough work while
catching a pathological search or port; the RSS allowance accommodates decoder
and allocator noise while bounding the additional 640-pixel luma/edge/Hough
working set.

## Complete producer, consumer, mutation, reporting, and test inventory

| Boundary | Producer/consumer or mutation path | Required contract |
| --- | --- | --- |
| Decode/thumbnail | `propose_image_adjustment`, `controller/src/image_adjustments.rs` | Preserve decoding, 640-pixel maximum dimension, zero-size fail-closed behavior, and private-byte-only input. |
| Primary background/threshold | `border_median_rgb`, `adaptive_color_distance_threshold`, `color_distance` | Preserve the current primary component source; no gradient input changes it. |
| Primary component/corners | `largest_contrast_component`, `trace_contrast_component` | Preserve largest qualifying 8-connected component, population ratio, corner order, and normalization. The obsolete anchor may be removed when the weak-probe loop is removed. |
| Existing stability decision | `component_boundary_is_stable` and constants in `image_adjustments.rs` | Replace the population/area weak-probe loop entirely. Do not retain it around the new veto. |
| Gradient producer | `imageproc::edges::canny` over luma | Fixed `20/60` on the same bounded thumbnail. |
| Line producer/filter | `imageproc::hough::{detect_lines, intersection_points, LineDetectionOptions, PolarLine}` plus focused private helpers | Fixed vote/suppression, axis windows, whole-line support/gap, ranking, and 12-per-axis cap. |
| Candidate producer | Bounded vertical-pair × horizontal-pair enumeration | At most 4,356 candidates; fixed span and area requirements; exact inclusive geometric interval `[-4,width+4] × [-4,height+4]` under imageproc's `[0,width] × [0,height]` image-boundary convention. Candidate corners are diagnostic only. |
| Candidate evidence/classifier | Finite-side edge and signed local-contrast helpers | At least three sides must meet every frozen side criterion; any qualifying rectangle returns `Unavailable`. |
| Proposal classifier | `propose_image_adjustment` | Preserve projection/area fail-closed checks, new veto, then near-identity `NotNeeded`, then original four-corner `Confident`. |
| API reporting | `assist_image_adjustment` and `AdminImageAssistResponse` in `controller/src/routes.rs` | Preserve authentication, private media read, initial/terminal `mediaRevision`, camelCase serialization, and no corners for `Unavailable`. No route edit. |
| Browser authority | `detectImageEdges`, session/draft/assist revisions, mutation/drag guards in `controller/static-admin/admin.js` | Only a current four-corner `confident` response mutates the unsaved draft. No browser edit. |
| Draft mutation/persistence | `setPerspectiveCorner`, `markReviewDraftChanged`, `saveImageAdjustments`, `resetImageAdjustments`, save/reset routes | Manual fallback remains available; assist never persists, publishes, or mutates the private original. |
| Detector regressions | `controller/tests/image_adjustments.rs` | Preserve all 15 current tests, designate the existing reliable-contrast test as the positive gradient contract, and add physical-slab/internal-primary plus Hough-rich/no-localized-contrast tests. |
| Route/browser/privacy tests | `controller/tests/admin_workflow.rs`, `static_admin_behavior.mjs`, `static_admin.rs` | Preserve serialization, authority, non-mutation, same-origin/manual fallback, and identifier privacy; files remain unchanged. |
| Spike evidence | Spikes 008, 009a, 009b and PR comment 6156047878 | Feasibility/reference only. Do not add spike rendering/report behavior to production. |
| Held-out acceptance | Exact positive, exact named slab, three slab siblings, and all 116 private originals | Run read-only outside Git. Expected 79 `Confident`, 37 `Unavailable`, 0 `NotNeeded`, with zero internal confident results and complete change assessment. |

## Post-spike failure matrix

| Input/outcome | Required production result | `85db903` scalar `||` | Failed coherence attempts | Spike 009b |
| --- | --- | --- | --- | --- |
| Reliable high-contrast/skewed outer item | Accurate primary `Confident` | Pass. | Pass. | Keep primary. |
| Exact accurate dark-card positive | Target primary `Confident` | **False abstention.** | Pass. | Pass: accurate `0.764` primary retained. |
| Sparse connected outer enclosure | `Unavailable` | Pass for scalar reason. | Pass. | Contract must veto via supported larger rectangle. |
| Filled/noisy-border low-contrast enclosure | `Unavailable` | Pass for scalar reason. | Pass. | Contract must remain safe. |
| Mechanism-derived physical slab around internal card | `Unavailable` | May abstain for the obsolete weak-mask reason. | Prior sanitized slabs were insufficient. | New fixture must show a supported gradient rectangle and causal internal-primary exposure when veto is bypassed. |
| Hough-rich/noisy accurate card | Accurate primary `Confident` | Expected false abstention when weak extrema grow. | Prior noise fixture passed but did not bind gradient line abundance. | New fixture must contain lines yet fail localized coherent side evidence. |
| Exact `20240415_225421.jpg` | `Unavailable`; recorded internal quadrilateral forbidden | Pass by broad scalar abstention. | **False internal `Confident` twice.** | Pass: `0.731` candidate, four valid sides, safe abstention. |
| Three graded-slab siblings | `Unavailable` | Prior safe abstention. | Must not regress. | All three abstained. |
| Near-identity inset 2 / inset 5 | `NotNeeded` / primary `Confident` | Pass. | Pass. | Preserve current tests. |
| `20251231_011140` | Accurate outer-card primary `Confident` | Previously screened exception. | Not a named failure. | Retained; visual inspection confirmed accurate rotated landscape boundary. |
| Full 116-image corpus | Zero internal `Confident`; expected 79/37/0 | Safe but over-abstaining head baseline. | Failed on named slab before acceptance. | 79 `Confident`, 37 `Unavailable`, 0 `NotNeeded`; every retained proposal visually accurate. |
| API/UI `Unavailable` consumption | No applied corners; manual fallback | Pass. | Consumer code unchanged. | Consumer code remains out of scope. |
| Two-prepared-control spike smoke | Feasibility only | Not comparable. | Not accepted. | `0.21s`, about `8.2 MiB` peak RSS; not a production rollback baseline. |
| Warm production-path benchmark | Interactive latency and relative regression over exact controls plus all 116 originals | Must be measured before source changes against head. | Not measured for rolled-back attempts. | Must be repeated after implementation with the same already-built evaluator binary shape, file list/order, and metrics. |

## Privacy and scope fences

- Production changes are limited to `controller/src/image_adjustments.rs` and
  `controller/tests/image_adjustments.rs` unless new evidence forces another
  reassessment.
- Do not modify routes, admin JavaScript, consumer tests, API types, response
  shapes, persistence, derivatives, publication, Cargo manifests, or lockfiles.
- Do not copy spike report/rendering code wholesale, add OpenCV/AI/OCR, or add a
  package; `imageproc 0.27` already supplies the required primitives.
- Do not retain the old scalar weak-boundary decision before or after the new
  veto, return gradient corners, or tune constants after exact/corpus results.
- Do not commit private paths, private images, masks, overlays, evaluator
  source/binaries, per-image output, contact sheets, or generated derivatives.
- Do not treat the 79/37/0 count as sufficient without visual geometry review,
  and do not accept any internal-region `Confident` result.

## Resume criteria

The status may change to `ready_to_resume` only after a fresh checker confirms:

1. the frozen gradient/Hough contract is represented completely and replaces
   the current scalar stability loop;
2. primary selection/corners remain authoritative and gradient corners can only
   trigger `Unavailable`;
3. the three sanitized tests and causal bypass mutation are explicit;
4. exact positive and named negative precede the full 116-image evaluation;
5. expected 79/37/0 results, zero internal confidence, full change assessment,
   three slab siblings, and the accurate exception are binding;
6. the exact imageproc geometric interval is bound by a private-helper unit test,
   and the warmed already-built production-path benchmark compares identical
   head/post workloads with explicit latency, total-time, and RSS rollback
   thresholds;
7. boundedness, consumer contracts, no-diff gates, and privacy checks are
   executable; and
8. both this artifact and the revised plan record the new review evidence path.

Until that review exists, spike feasibility does not authorize coder work.
