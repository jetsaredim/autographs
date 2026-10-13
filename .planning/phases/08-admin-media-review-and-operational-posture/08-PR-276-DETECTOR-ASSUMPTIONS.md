---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
finding_family: auto_assist_outer_boundary_confidence
status: ready_to_resume
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
revised_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
reassessment_iteration: 2
prior_independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
---

# PR #276 Detector Assumption Reassessment

## Scope and checkpoint state

This is the durable assumption reassessment required by the third review of PR
#276. It is surgical: it covers the auto-assist detector confidence boundary in
`controller/src/image_adjustments.rs` and its direct tests and consumers. It does
not replan Phase 8, reopen the static-public/Rust-controller architecture, or
authorize coder work by itself.

The first revised plan was independently reviewed, but its execution reached an
explicit held-out-corpus stop condition before any source commit. That review is
retained above as prior evidence, not current authorization. This second
reassessment and its linked plan passed fresh independent review in
`08-PR-276-DETECTOR-PLAN-REVIEW.md`; that evidence is recorded in frontmatter,
so convergence is `ready_to_resume` under every gate in the revised plan.

## Finding lineage

| Round | Head | Lineage | Evidence |
| --- | --- | --- | --- |
| 1 | `b2d947c` | Evidence gap | The 116-image held-out corpus exposed confident internal-region proposals that the synthetic suite did not represent. |
| 2 | `36946e7` | `incomplete_fix` | Population growth and area growth joined by `&&` missed a sparse connected outer enclosure whose perimeter changed the boundary without adding 25% population. |
| 3 | `85db903` | `fix_regression` | Replacing `&&` with `||` rejects the sparse enclosure but also rejects the exact accurate dark-card diagnostic when low-threshold noise/background connectivity pulls extrema outward. |
| 4 | uncommitted, rolled back | `incomplete_fix` / `test_weakness` | Four-side support passed the paired synthetic fixtures and restored the exact dark-card diagnostic, but `20240415_225421.jpg` remained falsely `Confident`. Strict `Qt`-contains-`Q0` gating prevented support analysis because the extrema-derived weak quadrilateral did not contain every primary quadrilateral corner even though the weak anchor component exposed the larger slab. |

The invariant family is unchanged across rounds: confidence needs evidence for
the outer item boundary. Round 3 proved that global component population and
quadrilateral area are observations, not independently sufficient boundary
evidence. Round 4 proves that polygon containment between two extrema-derived
quadrilaterals is not a valid prerequisite for asking whether the weaker
component exposes a coherent replacement boundary.

## Second reassessment evidence

- The rolled-back implementation made the sparse enclosing perimeter
  `Unavailable` and the sanitized noisy accurate dark card accurately
  `Confident`.
- The exact private `b9912131-8781-4047-a44c-03e5ca1e56a9.webp` returned
  `Confident` with the target outer-card corners.
- The held-out corpus still contained a known internal-region false confidence:
  `20240415_225421.jpg` returned approximately `(0.186441,0.306729)`,
  `(0.825666,0.286385)`, `(0.738499,0.870110)`, and
  `(0.188862,0.879499)`.
- Trace evidence showed that the coherence path was not reached: the weak
  quadrilateral did not geometrically contain all four primary quadrilateral
  corners despite the anchor-connected weak component exposing a larger slab.
- The implementation was rolled back, and no source commit was created.

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
| A-04 | A materially larger weak-threshold quadrilateral is a hypothesis about another outer boundary, not proof of one. | The sparse enclosure, accurate noisy dark card, and graded slab can all grow extrema. Coherent perimeter support decides whether the material shape change represents a replacement boundary. Polygon containment is not part of this premise. |
| A-05 | Population growth is contextual evidence, not an independent stability veto. | The sparse enclosure adds little population, while noise/background connectivity can add population without defining a replacement boundary. Remove the independent population-growth rejection from the decision contract. |
| A-06 | The route and browser authority boundaries are already correct and must remain unchanged. | The route authenticates, reads the private original, rechecks `mediaRevision`, and serializes the detector result. The browser requires a current session, unchanged draft/assist revisions, no pending mutation/drag, status `confident`, and exactly four corners before changing the draft. |
| A-07 | Private diagnostic and corpus files are held-out acceptance oracles, not repository fixtures. | The diagnostic and all 116 originals exist outside the repository; prior evaluation removed its transient evaluator. Use them read-only and commit neither inputs, paths in executable tests, outputs, nor evaluator artifacts. |
| A-08 | Anchor-connected component membership is monotone as the threshold weakens. | Every pixel in `C0` remains foreground at a lower threshold and remains connected to the retained anchor, so `M0` must be a subset of `Mt`. This pixel-set relationship, not quadrilateral containment, is the valid identity link between probes. |
| A-09 | The four extremal points are a lossy boundary summary whose polygon is not set-monotone. | `Ct` can contain every `C0` pixel while the chord polygon through `Qt` excludes a `Q0` corner. Use `Qt` for boundary-support sampling and material shape measurement, never as proof that component membership was or was not retained. |

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
| R-08 | The sparse-enclosure/noisy-card pair spans every relevant weak-boundary morphology. | Both passed while a non-nested extrema quadrilateral around a larger coherent slab failed. Add a sanitized non-nested slab fixture as a third binding morphology. |

## Second revised coherent boundary-change model

### Definitions and valid cross-threshold identity

- `D(p)` remains the existing RGB distance from pixel `p` to the border-median
  background.
- `C0`, `M0`, `Q0`, and `t0` are the primary anchor-connected component, its
  exact pixel membership, its four extremal corners, and the adaptive threshold.
- For each existing weaker probe `t`, `Ct` is traced from `C0.anchor`; `Mt` is
  that component's exact membership; and `Qt` is its extremal-corner
  quadrilateral.
- The valid identity check is `M0 ⊆ Mt`. Lowering the threshold cannot remove a
  primary foreground pixel or disconnect it from the retained anchor. If this
  assertion fails, stability fails closed because the implementation violated
  its own probe contract.
- `Qt` is a lossy shape summary. No `Qt`-contains-`Q0` predicate is permitted:
  extrema scores may expand monotonically while the polygon chords between the
  selected points cross inside a prior corner.

### Material change trigger

The existing normalized quadrilateral-area increase greater than 0.05 remains
a cheap trigger for examining a materially different weak boundary. It is not
a confidence decision, and it does not require polygon containment. Population
growth remains non-decisive. If a future corpus case shows a coherent
replacement boundary without material area growth, that is new evidence for a
separate reassessment, not permission to tune this trigger per image.

### Coherent support measurement

For every weak probe with retained component membership and material area
growth, measure whether `Mt` supports `Qt` as a localized, continuous boundary:

1. Rasterize each of the four `Qt` sides with a supercover/Bresenham walk at the
   detector thumbnail resolution.
2. A side sample is component-supported only when an `Mt` pixel lies within a
   two-pixel Chebyshev corridor of that sample. Use `Mt`, not the full weak mask,
   so unrelated weak components cannot manufacture support.
3. A supported sample is boundary-local only when an in-bounds outward-normal
   band three through five pixels from the side contains a non-`Mt` pixel.
   Out-of-bounds locations do not count as exterior evidence. This admits a
   filled edge and a sparse one-pixel ridge but rejects an unlocalized flood.
4. For each side, compute supported-sample coverage and the longest consecutive
   unsupported run. A side is coherent when coverage is at least 70% and its
   longest unsupported run is at most 20% of that side. All four sides must be
   coherent.
5. Reject `Q0` as unstable when `M0 ⊆ Mt`, `Qt` has material area growth, and
   all four `Qt` sides have coherent localized support. Do this even when the
   chord polygon for `Qt` does not contain every `Q0` corner.
6. If material extrema growth lacks coherent support, treat it as
   noise/background connectivity and retain the primary boundary. Population
   growth does not independently accept or reject.

The 70% coverage and 20% gap values already passed the sparse/noisy-card pair
and exact private positive control. This reassessment does not change them. The
algorithmic correction is replacing invalid polygon containment with exact
anchor-component retention. If the new non-nested slab fixture or held-out
corpus still fails, stop rather than tuning support constants against the image.

### Why the model now covers all observed morphologies

| Morphology | Cross-threshold evidence | Coherent-support result | Required classification |
| --- | --- | --- | --- |
| Sparse connected outer enclosure | `M0 ⊆ Mt`; a one-pixel rectangle and bridge materially enlarge `Qt`. | The rectangle supplies localized continuous support on all four expanded sides. | `Unavailable` |
| Noisy accurate dark-card analogue | `M0 ⊆ Mt`; weak tendrils/speckles pull extrema outward and enlarge `Qt`. | Lines between extrema lack continuous support, so one or more sides fail coverage/gap requirements. | Accurate four-corner `Confident` for `Q0` |
| Filled low-contrast enclosure | `M0 ⊆ Mt`; the weak filled item materially enlarges `Qt`. | Its exterior transition supplies localized support on all four sides. | `Unavailable` |
| Non-nested extrema slab (`20240415_225421` class) | `M0 ⊆ Mt`; the weak component exposes a much larger slab, but `Qt`'s chords do not contain every `Q0` corner. | Coherence is evaluated without the invalid containment gate; supported material boundary change invalidates the internal `Q0`. | `Unavailable` |
| Accurate outer boundary with interior fill/noise | Membership grows without a supported material replacement perimeter. | Material extrema either do not grow or fail coherent four-side support. | Preserve accurate `Confident`/`NotNeeded` result |

This model uses existing data, the Rust standard library, and current image
buffers. It adds no package and no new public/API type.

## Complete producer, consumer, mutation, reporting, and test inventory

| Boundary | Producer/consumer or mutation path | Evidence and invariant |
| --- | --- | --- |
| Background producer | `border_median_rgb` and `adaptive_color_distance_threshold` in `controller/src/image_adjustments.rs:273-318` | Produces the background reference and primary threshold. Preserve behavior. |
| Threshold producer | Relative probes plus fixed 32/16/8 in `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Reaches weak evidence hidden by an inflated adaptive threshold. Preserve ordering/deduplication. |
| Primary candidate producer | `largest_contrast_component`, `image_adjustments.rs:332-363` | Selects the largest qualifying 8-connected contrast component. No selection rewrite is in scope. |
| Anchor/component producer | `trace_contrast_component`, `image_adjustments.rs:365-424` | Produces anchor, count, and extremal corners. The revised implementation must retain the exact member indices for both `M0` and each `Mt`; the scan-wide `visited` bitmap used while choosing the largest primary component is not a primary-membership bitmap because it includes every traced component. |
| Stability producer | `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Current independent population-or-area rejection is the head regression site. The rolled-back coherence attempt replaced it but incorrectly gated support on `Qt` containing `Q0`. The second revision uses `M0 ⊆ Mt` plus material coherent support and no polygon-containment prerequisite. |
| Proposal classifier | `propose_image_adjustment`, `image_adjustments.rs:229-271` | Decode, projection/area validation, stability, then near-identity classification. Preserve fail-closed ordering and response statuses. |
| API consumer/reporting | `assist_image_adjustment`, `controller/src/routes.rs:1131-1190`; `AdminImageAssistResponse`, `routes.rs:2425-2432` | Authenticates, checks initial and terminal media revision, reads private bytes, and serializes camelCase status/corners/message without creating corners. |
| Browser consumer | `detectImageEdges`, `controller/static-admin/admin.js:2822-2892` | A current four-corner `confident` response alone updates `reviewDraftAdjustment.perspective`; other outcomes update copy only. |
| Session/revision authority | `beginReviewSession`/`isCurrentReviewSession`, draft and assist revisions, plus `mediaRevision` recovery in `admin.js`; terminal server recheck in `routes.rs` | Stale responses, concurrent drag/mutation, and changed media remain non-authoritative. |
| Draft mutation boundary | `setPerspectiveCorner`, `markReviewDraftChanged`, `saveImageAdjustments`, and `resetImageAdjustments` in `admin.js` | Assist only changes the unsaved draft. Manual handles remain available; persistence requires explicit Save; Reset is explicit. This is D-08-29. |
| Persistence/publication boundary | `save_image_adjustment`/`reset_image_adjustment` in `routes.rs`, normal publish flow elsewhere | Detector output itself neither changes the private original (D-08-25) nor persists/publishes an adjustment (D-08-31). |
| Detector regressions | `controller/tests/image_adjustments.rs` | Fifteen committed tests cover skew/card positives, flat dark card, filled/noisy/sparse enclosures, low confidence, near identity, serialization, and transforms. The rolled-back attempt proved a sanitized noisy accurate card but did not durably bind the non-nested extrema slab morphology. Both generated fixtures are required in the next test set. |
| Route contract | `admin_image_adjustment_routes_save_reset_review_and_assist` in `controller/tests/admin_workflow.rs:1318-1588` | Binds authenticated assist serialization, four-corner confident output, unavailable copy, private save/reset behavior, and revisions. |
| Browser behavior | `controller/tests/static_admin_behavior.mjs` | Binds stale assist rejection, non-mutating `notNeeded`, session/revision authority, drag authority, overlay/draft behavior, and conflict recovery. |
| Static contract/privacy | `controller/tests/static_admin.rs:566-690` | Binds manual fallback copy, confident gate, same-origin endpoint use, and absence of storage/provider identifiers and browser storage. |
| Held-out acceptance | Exact diagnostic `/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp` and 116 JPEGs under `/home/jgreenwa/Downloads/PHIL/autographs-3-001/autographs/` | Read-only private evidence. The diagnostic must be accurate `Confident`; `20240415_225421.jpg` must not reproduce the recorded internal quadrilateral; the corpus must contain zero known/screened internal-region confident proposals. |

## Second-reassessment failure matrix

| Input/outcome | Required result | Head `85db903` | Rolled-back four-side attempt |
| --- | --- | --- | --- |
| Filled low-contrast enclosure around dark content | `Unavailable` | Pass. | Must be rerun after second revision. |
| Noisy-border filled enclosure | `Unavailable` | Pass. | Must be rerun after second revision. |
| Sparse connected weak outer enclosure | `Unavailable` | Pass; round-2 regression is faithful. | Pass. |
| High-contrast skewed item | Accurate `Confident` | Pass in committed test. | Must remain unchanged. |
| Reliable high-contrast outer card | Accurate `Confident` | Pass in committed test. | Must remain unchanged. |
| Flat synthetic dark card | Accurate `Confident` | Pass, but does not bind real noise connectivity. | Must remain unchanged. |
| Sanitized noisy accurate dark-card analogue | Accurate `Confident` | Missing at head; broad `||` would abstain. | Pass. |
| Supplied accurate dark-card diagnostic | Accurate `Confident` near recorded target corners | **Fail: `Unavailable`, no corners.** | Pass. |
| Sanitized coherent slab with non-nested extrema quadrilateral | `Unavailable` | New committed fixture missing; current broad `||` should abstain. | Required RED reproduction of the failed containment assumption. |
| Held-out `20240415_225421.jpg` | `Unavailable` unless a separately proven accurate outer-slab quadrilateral exists | Pass under broad `||` based on prior evidence. | **Fail: internal region `Confident` at approximately `(0.186441,0.306729)`, `(0.825666,0.286385)`, `(0.738499,0.870110)`, `(0.188862,0.879499)`.** |
| Other previously named internal/slab corpus cases | No internal-region `Confident` proposal | Prior hardened evidence reached safe abstention; fresh run required. | Fresh complete results were not accepted because the known slab stop condition fired. |
| Near-identity inset 2 / inset 5 | `notNeeded` / `Confident` | Pass in committed tests. | Must remain unchanged. |
| `Unavailable` API/UI consumption | No corners applied; manual adjustment remains available | Pass by source and behavior tests. | Consumer code was unchanged. |
| Complete 116-image corpus | Zero known/screened internal-region confident proposals; status-count and corner changes explicitly assessed | Baseline must be measured from head. | Failed acceptance because one known internal-region confidence remained. |

## Scope fences

- Do not add OpenCV, AI/OCR, a new image library, or another service. That would
  violate the bounded heuristic/manual-fallback decision in D-08-26 and the
  deferred AI/advanced-enhancement scope.
- Do not change public response shapes, route authorization, media-revision
  authority, browser draft/save semantics, or manual controls.
- Do not reintroduce strict `Qt`-contains-`Q0` as a prerequisite, special-case
  `20240415_225421.jpg`, or tune support/area constants to its coordinates.
- Do not commit the private WEBP, any corpus image, absolute private paths in
  executable tests, evaluator source/binaries, per-image output, or generated
  thumbnails.
- Do not treat more abstention as proof of correctness. The explicit positive
  controls and the confidence-count assessment are equally binding.

