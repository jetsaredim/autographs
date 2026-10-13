---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
finding_family: auto_assist_outer_boundary_confidence
status: reassessment_required
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
revised_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
reassessment_iteration: 3
prior_independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
independent_review: pending
---

# PR #276 Detector Assumption Reassessment

## Scope and checkpoint state

This is the durable assumption reassessment required by the third review of PR
#276. It is surgical: it covers the auto-assist detector confidence boundary in
`controller/src/image_adjustments.rs` and its direct tests and consumers. It does
not replan Phase 8, reopen the static-public/Rust-controller architecture, or
authorize coder work by itself.

The first revised plan was independently reviewed, but its execution reached an
explicit held-out-corpus stop condition before any source commit. The second
revised plan was then independently reviewed and authorized, but its execution
also reached the named held-out stop condition before any source commit. The
review file is retained above as prior evidence and lineage; its authorization
is superseded by this later execution evidence. Convergence is again
`reassessment_required`. No third coder round is authorized by this artifact.

## Finding lineage

| Round | Head | Lineage | Evidence |
| --- | --- | --- | --- |
| 1 | `b2d947c` | Evidence gap | The 116-image held-out corpus exposed confident internal-region proposals that the synthetic suite did not represent. |
| 2 | `36946e7` | `incomplete_fix` | Population growth and area growth joined by `&&` missed a sparse connected outer enclosure whose perimeter changed the boundary without adding 25% population. |
| 3 | `85db903` | `fix_regression` | Replacing `&&` with `||` rejects the sparse enclosure but also rejects the exact accurate dark-card diagnostic when low-threshold noise/background connectivity pulls extrema outward. |
| 4 | uncommitted, rolled back | `incomplete_fix` / `test_weakness` | Four-side support passed the paired synthetic fixtures and restored the exact dark-card diagnostic, but `20240415_225421.jpg` remained falsely `Confident`. Strict `Qt`-contains-`Q0` gating prevented support analysis because the extrema-derived weak quadrilateral did not contain every primary quadrilateral corner even though the weak anchor component exposed the larger slab. |
| 5 | uncommitted, rolled back | `incomplete_fix` / `assumption_failure` / `test_weakness` | Replacing polygon containment with exact `M0 ⊆ Mt` made all three synthetic morphologies pass, all 17 detector tests pass, and the exact positive diagnostic return its target corners. The exact named slab nevertheless returned the same forbidden internal `Confident` quadrilateral. The four-side extrema-chord support/witness abstraction therefore failed to represent the real slab even when its cross-threshold identity gate was corrected. |

The invariant family is unchanged across rounds: confidence needs evidence for
the outer item boundary. Round 3 proved that global component population and
quadrilateral area are observations, not independently sufficient boundary
evidence. Round 4 proved that polygon containment between two extrema-derived
quadrilaterals is not a valid prerequisite for asking whether the weaker
component exposes a replacement boundary. Round 5 now proves that removing
that gate is insufficient: four chords joining extrema plus corridor/exterior
witnesses are not a validated proxy for the actual component boundary on the
real slab.

## Third reassessment evidence

- The second rolled-back implementation replaced strict polygon containment
  with exact `M0 ⊆ Mt` while leaving the reviewed four-side coherence/witness
  measurement unchanged.
- All three sanitized morphologies passed, including the new containment-bypass
  slab; the full detector integration suite passed all 17 tests.
- The exact private `b9912131-8781-4047-a44c-03e5ca1e56a9.webp` returned
  `Confident` with the target outer-card corners.
- The exact held-out `20240415_225421.jpg` still returned the forbidden internal
  `Confident` quadrilateral at approximately `(0.186441,0.306729)`,
  `(0.825666,0.286385)`, `(0.738499,0.870110)`, and
  `(0.188862,0.879499)`.
- This time the synthetic containment-bypass case passed, so the remaining miss
  cannot be attributed to the removed `Qt`-contains-`Q0` condition. The failed
  abstraction is the unchanged extrema-chord coherence/witness model itself, or
  an unobserved premise beneath it.
- Two follow-on diagnostic agents were terminated after becoming unresponsive.
  They produced no reliable per-side trace, threshold mask, or contour evidence;
  absence of that evidence must not be interpreted as algorithm validation.
- The attempted changes were rolled back. The worktree remained source-clean,
  and no source commit was created.

## Shared invariants

These four invariants from `08-REVIEW.md` remain authoritative:

1. A `Confident` proposal represents the stable outer item boundary, not a higher-contrast internal region.
2. Weak-mask evidence must distinguish coherent enclosing geometry from background/noise connectivity; either may expand component population or extrema.
3. Safe abstention is preferred for ambiguity, but known accurate material cases remain usable; the supplied dark-card diagnostic is an explicit positive control.
4. `Unavailable` and unexpected outcomes never mutate the browser draft; only a current four-corner `Confident` response may do so.

They preserve the bounded-assistance contract in D-08-26 and MEDIA-06: auto
correction proposes when justified, and uncertain cases retain manual corner
handles. They also preserve D-08-25 and D-08-29: the detector does not mutate a
private original or persist a draft.

## Evidence-backed assumption decisions

### Accepted

| ID | Assumption | Evidence and consequence |
| --- | --- | --- |
| A-01 | The primary adaptive-threshold component remains the candidate whose outer-boundary identity must be tested. | `largest_contrast_component` selects it and stores its row-major anchor; the held-out failures are false confidence in that selected component, not selection of a later unrelated component. Keep primary selection unchanged in this convergence fix. |
| A-02 | Tracing from the original anchor at monotonically weaker thresholds correctly follows mergers involving the same candidate. | `component_boundary_is_stable` deduplicates descending relative/fixed probes, and a lower threshold cannot remove the foreground anchor. Preserve anchor continuity and fail closed if that contract is ever violated. |
| A-03 | Fixed probes down to color distance 8 are necessary because an inflated adaptive threshold can hide a low-contrast enclosure from relative probes. | The known slab failure used adaptive threshold 234; fixed threshold 32 exposed its enclosing region. Preserve the current relative and fixed probe schedule. |
| A-04 | A materially larger weak-threshold quadrilateral is a hypothesis about another outer boundary, not proof of one. | The sparse enclosure, accurate noisy dark card, and graded slab can all grow extrema. A spatial boundary signal is still required, but round 5 invalidates the reviewed extrema-chord support signal as a sufficient classifier. |
| A-05 | Population growth is contextual evidence, not an independent stability veto. | The sparse enclosure adds little population, while noise/background connectivity can add population without defining a replacement boundary. Remove the independent population-growth rejection from the decision contract. |
| A-06 | The route and browser authority boundaries are already correct and must remain unchanged. | The route authenticates, reads the private original, rechecks `mediaRevision`, and serializes the detector result. The browser requires a current session, unchanged draft/assist revisions, no pending mutation/drag, status `confident`, and exactly four corners before changing the draft. |
| A-07 | Private diagnostic and corpus files are held-out acceptance oracles, not repository fixtures. | The diagnostic and all 116 originals exist outside the repository; prior evaluation removed its transient evaluator. Use them read-only and commit neither inputs, paths in executable tests, outputs, nor evaluator artifacts. |
| A-08 | Anchor-connected component membership is monotone as the threshold weakens. | Every pixel in `C0` remains foreground at a lower threshold and remains connected to the retained anchor, so `M0` must be a subset of `Mt`. This remains a valid identity fact, but round 5 proves it is not a sufficient outer-boundary classifier. |
| A-09 | The four extremal points are a lossy boundary summary whose polygon is not set-monotone. | `Ct` can contain every `C0` pixel while the chord polygon through `Qt` excludes a `Q0` corner. Round 5 further shows that the four chords can fail to describe a real slab's boundary evidence even after identity is established. |
| A-10 | The next useful evidence must expose the detector's intermediate geometry on both a true positive and the named false positive. | Synthetic outcome tests did not reveal why the real slab passed. A bounded diagnostic must expose threshold masks, anchor-component membership, actual contours, extrema/chords, and local boundary evidence for both exact images before another classifier is selected. |

### Rejected

| ID | Rejected assumption | Evidence |
| --- | --- | --- |
| R-01 | Any material weak-mask area expansion proves a different outer boundary. | At `85db903`, this is the independent right-hand side of the `||`; the exact accurate dark-card diagnostic becomes `Unavailable`. |
| R-02 | The flat `dark_card_fixture` represents the noisy/background-connected positive class. | It remains geometrically clean through weak probes and stays green while the exact supplied diagnostic regresses. |
| R-03 | Population growth and area growth can be classified independently by two global scalar limits. | The round-2 `&&` misses the sparse perimeter; the round-3 `||` rejects the noisy accurate card. The paired morphologies contradict both boolean arrangements. |
| R-04 | Threshold ordering or anchor discontinuity explains the regression. | The probe sequence is descending after deduplication, and every weaker trace starts from the original foreground anchor. |
| R-05 | False abstention is acceptable merely because manual fallback remains safe. | Manual fallback is required, but D-08-26 also requires useful bounded assistance; the supplied accurate dark-card diagnostic is an explicit positive acceptance control. |
| R-06 | Another threshold or boolean can be layered around the `85db903` `||`. | The recurrence shows that the measured quantities do not encode the missing spatial fact. The `||` must be replaced by the coherent support decision below, not wrapped in another ad hoc gate. |
| R-07 | A coherent weak replacement boundary must have a quadrilateral that contains every `Q0` corner. | `20240415_225421.jpg` retained a falsely confident internal region because this predicate skipped coherence analysis even while the weak component exposed a larger slab. Rejected as the direct failed assumption. |
| R-08 | The sparse-enclosure/noisy-card pair spans every relevant weak-boundary morphology. | A third sanitized non-nested slab was added in the rolled-back attempt and all three passed, while the exact slab still failed. Even the three-fixture set does not yet reproduce the real mechanism. |
| R-09 | `M0 ⊆ Mt`, material area growth, and the reviewed four-side extrema-chord coherence test are sufficient to recognize the real slab. | That exact combination passed all three sanitized morphologies and the exact positive diagnostic, yet `20240415_225421.jpg` still returned the forbidden internal quadrilateral. |
| R-10 | Passing the three sanitized morphologies is enough to authorize corpus validation as a final confirmation step. | All three passed while the named private negative failed. The exact positive and named slab must participate in design characterization before, not only after, a third implementation. |
| R-11 | A missing per-side trace can be filled in by inference from final classifications. | The terminated diagnostic attempts produced no trustworthy trace. The decisive intermediate masks and contours are currently unknown, so choosing another implementation now would be another unsupported point fix. |

## Current model boundary: no approved classifier

The second reviewed model is invalidated as an implementation contract. Exact
component retention remains a sound cross-threshold identity relationship, but
neither `Qt` containment nor the four extrema chords establish what the real
component boundary does between those extrema. The 70% coverage, 20% gap,
two-pixel corridor, outward offsets, and 0.05 area trigger are historical
parameters of the failed attempt, not approved constants for a third round.
They must not be retained by default, retuned against the named image, or wrapped
in another scalar or Boolean gate.

The open design question is narrower and more fundamental: what observable
spatial evidence distinguishes a genuine enclosing item boundary from a
noise-connected component when the detector's four extrema summarize both
poorly? That question requires intermediate evidence before an algorithm can be
selected.

## Bounded detector-design spike contract

Before any third coder round, build an ephemeral diagnostic harness outside Git
that runs the current detector primitives read-only on exactly these initial
controls:

1. the exact positive
   `b9912131-8781-4047-a44c-03e5ca1e56a9.webp`;
2. the exact negative `20240415_225421.jpg`;
3. the three sanitized sparse-enclosure, noise-connected accurate-card, and
   containment-bypass slab fixtures; and
4. only if needed to disprove a candidate explanation, the existing filled and
   noisy-border enclosure fixtures.

For the primary threshold and every actual descending relative/fixed probe, the
harness must make the following inspectable without changing production code:

- threshold value and probe order;
- binary threshold mask and exact anchor-connected component mask;
- component count, pixel population, retained-primary membership, extrema, and
  normalized extrema-quadrilateral area;
- the actual ordered component boundary/contour or every boundary pixel when a
  single ordered contour is not yet available;
- the candidate quadrilateral, its four rasterized chords, per-side support/gap
  and outward-witness values from the failed model, so the named slab's precise
  bypass is observed rather than guessed;
- contour perimeter, connected boundary runs, distance from contour to candidate
  sides, and simple shape summaries sufficient to compare the positive and
  negative; and
- local image-gradient or color-transition strength and continuity immediately
  across candidate boundary segments, computed as diagnostic data rather than a
  classifier.

Private masks, overlays, paths, per-image traces, evaluator source, binaries,
and outputs stay under a temporary directory such as
`/tmp/autographs-pr276-detector-spike.*` and never enter Git. The durable result
is a text-only, privacy-safe design-spike report that records aggregate facts,
the exact divergent mechanism, competing model assessments, and sanitized
reproduction requirements without embedding private pixels or executable
absolute paths.

### Design directions to compare, not approve

The recommended first direction is to represent the actual anchor-component
boundary instead of the four extrema chords. The spike should compare whether a
contour, contour-derived hull, or contour-to-candidate-side distance exposes the
named slab's enclosing geometry while treating noise tendrils as discontinuous
or non-rectangular. A second direction is local-gradient rectangular-boundary
evidence: measure whether four plausible sides have sustained, correctly sided
image transitions rather than merely nearby component pixels. Multi-threshold
contour persistence may be evaluated as a combination of those signals.

These are investigation hypotheses only. No contour simplification, hull,
line-fitting, gradient threshold, rectangularity score, or combination rule is
approved until the harness shows how it separates the exact positive, exact
negative, and sanitized controls without per-image constants. If none separates
them, the spike must say so and convergence remains blocked.

### Spike decision output

The spike must answer all of the following:

1. At which probe does the named slab first expose geometry outside the internal
   `Q0`, and what does its actual component contour look like relative to `Qt`?
2. Why do the failed four chords/witnesses accept that slab: no material area
   trigger, one or more incoherent chords, exterior-witness behavior, or another
   measured cause?
3. At the corresponding probes, which contour or gradient property separates
   the accurate private card from the slab?
4. Can the property be reproduced in sanitized generated fixtures, including a
   RED fixture for the real failure mechanism rather than only a visually
   similar shape?
5. What false-positive and false-abstention risks would a proposed classifier
   introduce across the existing morphology matrix and 116-image corpus?

Only after those questions have evidence-backed answers may the assumptions be
revised into one coherent candidate contract and a new implementation-ready
plan be independently reviewed.

## Complete producer, consumer, mutation, reporting, and test inventory

| Boundary | Producer/consumer or mutation path | Evidence and invariant |
| --- | --- | --- |
| Background producer | `border_median_rgb` and `adaptive_color_distance_threshold` in `controller/src/image_adjustments.rs:273-318` | Produces the background reference and primary threshold. Preserve behavior. |
| Threshold producer | Relative probes plus fixed 32/16/8 in `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Reaches weak evidence hidden by an inflated adaptive threshold. Preserve ordering/deduplication. |
| Primary candidate producer | `largest_contrast_component`, `image_adjustments.rs:332-363` | Selects the largest qualifying 8-connected contrast component. No selection rewrite is in scope. |
| Anchor/component producer | `trace_contrast_component`, `image_adjustments.rs:365-424` | Produces anchor, count, and extremal corners. The revised implementation must retain the exact member indices for both `M0` and each `Mt`; the scan-wide `visited` bitmap used while choosing the largest primary component is not a primary-membership bitmap because it includes every traced component. |
| Stability producer | `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Current independent population-or-area rejection is the head regression site. The first rolled-back coherence attempt incorrectly gated support on `Qt` containing `Q0`; the second removed containment but still missed the exact slab. No replacement stability classifier is currently approved. |
| Proposal classifier | `propose_image_adjustment`, `image_adjustments.rs:229-271` | Decode, projection/area validation, stability, then near-identity classification. Preserve fail-closed ordering and response statuses. |
| API consumer/reporting | `assist_image_adjustment`, `controller/src/routes.rs:1131-1190`; `AdminImageAssistResponse`, `routes.rs:2425-2432` | Authenticates, checks initial and terminal media revision, reads private bytes, and serializes camelCase status/corners/message without creating corners. |
| Browser consumer | `detectImageEdges`, `controller/static-admin/admin.js:2822-2892` | A current four-corner `confident` response alone updates `reviewDraftAdjustment.perspective`; other outcomes update copy only. |
| Session/revision authority | `beginReviewSession`/`isCurrentReviewSession`, draft and assist revisions, plus `mediaRevision` recovery in `admin.js`; terminal server recheck in `routes.rs` | Stale responses, concurrent drag/mutation, and changed media remain non-authoritative. |
| Draft mutation boundary | `setPerspectiveCorner`, `markReviewDraftChanged`, `saveImageAdjustments`, and `resetImageAdjustments` in `admin.js` | Assist only changes the unsaved draft. Manual handles remain available; persistence requires explicit Save; Reset is explicit. This is D-08-29. |
| Persistence/publication boundary | `save_image_adjustment`/`reset_image_adjustment` in `routes.rs`, normal publish flow elsewhere | Detector output itself neither changes the private original (D-08-25) nor persists/publishes an adjustment (D-08-31). |
| Detector regressions | `controller/tests/image_adjustments.rs` | Fifteen tests are committed at the reviewed head. The rolled-back second attempt ran 17 detector tests and passed all three generated morphologies, including the containment-bypass slab, yet the exact slab still failed. Existing generated fixtures are useful controls but do not authorize a third implementation until a diagnostic-derived sanitized RED binds the real mechanism. |
| Route contract | `admin_image_adjustment_routes_save_reset_review_and_assist` in `controller/tests/admin_workflow.rs:1318-1588` | Binds authenticated assist serialization, four-corner confident output, unavailable copy, private save/reset behavior, and revisions. |
| Browser behavior | `controller/tests/static_admin_behavior.mjs` | Binds stale assist rejection, non-mutating `notNeeded`, session/revision authority, drag authority, overlay/draft behavior, and conflict recovery. |
| Static contract/privacy | `controller/tests/static_admin.rs:566-690` | Binds manual fallback copy, confident gate, same-origin endpoint use, and absence of storage/provider identifiers and browser storage. |
| Held-out acceptance | Exact diagnostic `/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp` and 116 JPEGs under `/home/jgreenwa/Downloads/PHIL/autographs-3-001/autographs/` | Read-only private evidence. In the spike they are paired design controls, not a final-only oracle. Any later implementation must keep the diagnostic accurately `Confident`, prevent `20240415_225421.jpg` from reproducing the recorded internal quadrilateral, and leave the corpus with zero known/screened internal-region confident proposals. |

## Third-reassessment failure matrix

| Input/outcome | Required result | Head `85db903` | Attempt 1: containment-gated coherence | Attempt 2: `M0 ⊆ Mt` coherence |
| --- | --- | --- | --- | --- |
| Filled low-contrast enclosure around dark content | `Unavailable` | Pass. | Preserved by reported tests. | Pass in 17-test suite. |
| Noisy-border filled enclosure | `Unavailable` | Pass. | Preserved by reported tests. | Pass in 17-test suite. |
| Sparse connected weak outer enclosure | `Unavailable` | Pass; round-2 regression is faithful. | Pass. | Pass. |
| High-contrast skewed item | Accurate `Confident` | Pass. | Pass. | Pass in 17-test suite. |
| Reliable high-contrast outer card | Accurate `Confident` | Pass. | Pass. | Pass in 17-test suite. |
| Flat synthetic dark card | Accurate `Confident` | Pass, but does not bind real noise connectivity. | Pass. | Pass in 17-test suite. |
| Sanitized noisy accurate dark-card analogue | Accurate `Confident` | Broad `||` abstains. | Pass. | Pass. |
| Exact accurate dark-card diagnostic | Accurate `Confident` near recorded target corners | **Fail: `Unavailable`, no corners.** | Pass. | Pass with target corners. |
| Sanitized coherent non-nested slab | `Unavailable` | Broad `||` abstains. | Failed before round-4 reassessment. | Pass. |
| Exact `20240415_225421.jpg` | Not the recorded internal `Confident` quadrilateral | Broad `||` abstains based on prior evidence. | **Fail: recorded internal `Confident`.** | **Fail again at approximately `(0.186441,0.306729)`, `(0.825666,0.286385)`, `(0.738499,0.870110)`, `(0.188862,0.879499)`.** |
| Other prior internal/slab corpus cases | No internal-region `Confident` proposal | Prior hardened evidence reached safe abstention. | Full results not accepted after named stop. | Full results not accepted after named stop. |
| Near-identity inset 2 / inset 5 | `notNeeded` / `Confident` | Pass. | Preserved. | Pass in 17-test suite. |
| `Unavailable` API/UI consumption | No corners applied; manual adjustment remains available | Pass by source and behavior tests. | Consumer code unchanged. | Consumer code unchanged. |
| Complete 116-image corpus | Zero known/screened internal-region confident proposals; status/corner changes assessed | Baseline required. | Failed acceptance at named slab. | Failed acceptance at named slab before a qualifying full-corpus result. |

## Explicit resume criteria

Status must remain `reassessment_required` until every condition below is met:

1. A bounded diagnostic harness has run on both exact private controls and the
   required sanitized fixtures, and it has produced inspectable masks,
   components, contours, failed-model per-side values, and local-gradient data.
2. A privacy-safe durable spike report explains the named slab's measured bypass
   and the measured separation, or lack of separation, from the exact positive.
3. At least two design alternatives are compared against the full failure
   matrix. Any recommended classifier is stated as one coherent spatial contract
   with no filename, coordinate, or per-image parameter special case.
4. A sanitized generated fixture reproduces the measured real-slab mechanism as
   a RED test against the reviewed head or a clearly identified intermediate;
   visual resemblance and a final-status assertion alone are insufficient.
5. The assumptions and revised implementation plan are updated from the spike,
   independently reviewed together, and both reviewed evidence paths are
   recorded before their statuses change to `ready_to_resume`.
6. The reviewed plan preserves the later two-file implementation scope,
   exact-positive and named-negative gates, full 116-image geometry screen,
   confidence-count/corner-change assessment, four shared invariants, consumer
   authority, manual fallback, and privacy constraints.
7. `controller/src/image_adjustments.rs` and
   `controller/tests/image_adjustments.rs` remain source-clean until that review
   authorizes a third coder round.

## Scope fences

- Do not add OpenCV, AI/OCR, a new image library, or another service. That would
  violate the bounded heuristic/manual-fallback decision in D-08-26 and the
  deferred AI/advanced-enhancement scope.
- Do not change public response shapes, route authorization, media-revision
  authority, browser draft/save semantics, or manual controls.
- Do not modify detector production code or committed tests during the design
  spike. A third implementation is not authorized by the previously passed
  review or by this reassessment.
- Do not reintroduce strict `Qt`-contains-`Q0` as a prerequisite, special-case
  `20240415_225421.jpg`, or tune support/area constants to its coordinates.
- Do not assume the failed four-side constants or extrema chords survive into a
  future plan. Preserve them only as diagnostic measurements until evidence
  justifies a replacement contract.
- Do not commit the private WEBP, any corpus image, absolute private paths in
  executable tests, evaluator source/binaries, per-image output, or generated
  thumbnails.
- Do not treat more abstention as proof of correctness. The explicit positive
  controls and the confidence-count assessment are equally binding.

