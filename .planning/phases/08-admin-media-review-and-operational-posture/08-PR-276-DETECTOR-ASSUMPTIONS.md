---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
finding_family: auto_assist_outer_boundary_confidence
status: ready_to_resume
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
revised_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
---

# PR #276 Detector Assumption Reassessment

## Scope and checkpoint state

This is the durable assumption reassessment required by the third review of PR
#276. It is surgical: it covers the auto-assist detector confidence boundary in
`controller/src/image_adjustments.rs` and its direct tests and consumers. It does
not replan Phase 8, reopen the static-public/Rust-controller architecture, or
authorize coder work by itself.

Independent review of this artifact and the linked revised plan passed with no
findings in `08-PR-276-DETECTOR-PLAN-REVIEW.md`. The evidence path is recorded
above, so the convergence state is `ready_to_resume`; coder work remains bound
by every acceptance gate and stop condition in the reviewed plan.

## Finding lineage

| Round | Head | Lineage | Evidence |
| --- | --- | --- | --- |
| 1 | `b2d947c` | Evidence gap | The 116-image held-out corpus exposed confident internal-region proposals that the synthetic suite did not represent. |
| 2 | `36946e7` | `incomplete_fix` | Population growth and area growth joined by `&&` missed a sparse connected outer enclosure whose perimeter changed the boundary without adding 25% population. |
| 3 | `85db903` | `fix_regression` | Replacing `&&` with `||` rejects the sparse enclosure but also rejects the exact accurate dark-card diagnostic when low-threshold noise/background connectivity pulls extrema outward. |

The invariant family is unchanged across rounds: confidence needs evidence for
the outer item boundary. Round 3 proves that global component population and
quadrilateral area are observations, not independently sufficient boundary
evidence.

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
| A-04 | A materially larger weak-threshold quadrilateral is a hypothesis about another outer boundary, not proof of one. | Both the sparse enclosure and the accurate noisy dark-card grow extrema materially, but only the sparse enclosure contains spatially distributed support along the larger perimeter. Require coherent perimeter support before invalidating the primary boundary. |
| A-05 | Population growth is contextual evidence, not an independent stability veto. | The sparse enclosure adds little population, while noise/background connectivity can add population without defining a replacement boundary. Remove the independent population-growth rejection from the decision contract. |
| A-06 | The route and browser authority boundaries are already correct and must remain unchanged. | The route authenticates, reads the private original, rechecks `mediaRevision`, and serializes the detector result. The browser requires a current session, unchanged draft/assist revisions, no pending mutation/drag, status `confident`, and exactly four corners before changing the draft. |
| A-07 | Private diagnostic and corpus files are held-out acceptance oracles, not repository fixtures. | The diagnostic and all 116 originals exist outside the repository; prior evaluation removed its transient evaluator. Use them read-only and commit neither inputs, paths in executable tests, outputs, nor evaluator artifacts. |

### Rejected

| ID | Rejected assumption | Evidence |
| --- | --- | --- |
| R-01 | Any material weak-mask area expansion proves a different outer boundary. | At `85db903`, this is the independent right-hand side of the `||`; the exact accurate dark-card diagnostic becomes `Unavailable`. |
| R-02 | The flat `dark_card_fixture` represents the noisy/background-connected positive class. | It remains geometrically clean through weak probes and stays green while the exact supplied diagnostic regresses. |
| R-03 | Population growth and area growth can be classified independently by two global scalar limits. | The round-2 `&&` misses the sparse perimeter; the round-3 `||` rejects the noisy accurate card. The paired morphologies contradict both boolean arrangements. |
| R-04 | Threshold ordering or anchor discontinuity explains the regression. | The probe sequence is descending after deduplication, and every weaker trace starts from the original foreground anchor. |
| R-05 | False abstention is acceptable merely because manual fallback remains safe. | Manual fallback is required, but D-08-26 also requires useful bounded assistance; the supplied accurate dark-card diagnostic is an explicit positive acceptance control. |
| R-06 | Another threshold or boolean can be layered around the `85db903` `||`. | The recurrence shows that the measured quantities do not encode the missing spatial fact. The `||` must be replaced by the coherent support decision below, not wrapped in another ad hoc gate. |

## Revised coherent outer-boundary support model

### Definitions

- `D(p)` is the existing RGB distance from pixel `p` to the border-median
  background.
- `C0`, `Q0`, and `t0` are the primary anchor-connected component, its four
  extremal corners, and the adaptive threshold.
- For each existing weaker probe `t`, `Ct` is traced from `C0.anchor`; `Mt` is
  the membership bitmap written by that trace; and `Qt` is its extremal-corner
  quadrilateral.
- `Qt` is a *material enclosing candidate* only when it contains `Q0` and its
  normalized area exceeds `Q0` by the existing 0.05 material-growth amount.
  Area growth remains a cheap trigger for examining a possible enclosure; it
  is no longer sufficient to reject confidence.

### Support measurement

For each material enclosing candidate, measure whether `Mt` actually supports
the perimeter of `Qt`:

1. Rasterize each of the four `Qt` sides with a supercover/Bresenham walk at the
   detector thumbnail resolution.
2. A side sample is component-supported only when an `Mt` pixel lies within a
   two-pixel Chebyshev corridor of that sample. Use `Mt`, not the full weak
   foreground mask, so unrelated weak components cannot manufacture support.
3. A supported sample is boundary-local only when an in-bounds outward-normal
   band three through five pixels from the side contains a non-`Mt` pixel.
   Out-of-bounds locations do not count as exterior evidence. This admits both
   a filled enclosure edge (foreground inside/background outside) and a sparse
   one-pixel ridge (background on either side), while rejecting a broadly
   flooded region with no localized in-image boundary.
4. For each side, compute supported-sample coverage and the longest consecutive
   unsupported run. A side is coherent when coverage is at least 70% and its
   longest unsupported run is at most 20% of that side. `Qt` is coherently
   supported only when all four sides are coherent.
5. A weaker probe invalidates `Q0` only when `Qt` is both a material enclosing
   candidate and coherently supported. Population growth may be recorded while
   evaluating/debugging, but it does not independently accept or reject.

The 70% coverage and 20% gap values are named structural support constants,
not replacements for the old component-growth boolean. Their purpose is to
encode distributed continuity on every side. If the paired fixtures cannot
falsify the old behavior and prove this topology, stop for reassessment rather
than tuning those values against individual private images.

### Why the model discriminates the paired morphologies

| Morphology | Weak-threshold observation | Coherent-support result | Required classification |
| --- | --- | --- | --- |
| Sparse connected outer enclosure | A one-pixel rectangle and bridge add little population but move all four extrema outward. The rectangle supplies continuous `Mt` pixels along each expanded side with exterior background witnesses. | All four sides meet coverage/maximum-gap requirements, so the larger enclosure is real evidence against `Q0`. | `Unavailable` |
| Noisy accurate dark-card analogue | Deterministic weak tendrils/speckles connect the accurate card to distant extrema and materially enlarge `Qt`, but the lines between those extrema have no continuous component support. | One or more sides have low coverage or a long unsupported gap, so extrema growth is classified as noise connectivity rather than another boundary. | Accurate four-corner `Confident` for `Q0` |
| Filled low-contrast card/slab enclosure | The weak component fills the enclosing item and its exterior transition follows all four larger sides. | All four sides receive localized continuous support. | `Unavailable` |
| Accurate outer boundary with interior fill/noise | Population changes inside or through isolated branches without a supported larger perimeter. | No coherently supported material enclosure replaces `Q0`. | Preserve accurate `Confident`/`NotNeeded` result |

This model uses existing data, the Rust standard library, and current image
buffers. It adds no package and no new public/API type.

## Complete producer, consumer, mutation, reporting, and test inventory

| Boundary | Producer/consumer or mutation path | Evidence and invariant |
| --- | --- | --- |
| Background producer | `border_median_rgb` and `adaptive_color_distance_threshold` in `controller/src/image_adjustments.rs:273-318` | Produces the background reference and primary threshold. Preserve behavior. |
| Threshold producer | Relative probes plus fixed 32/16/8 in `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Reaches weak evidence hidden by an inflated adaptive threshold. Preserve ordering/deduplication. |
| Primary candidate producer | `largest_contrast_component`, `image_adjustments.rs:332-363` | Selects the largest qualifying 8-connected contrast component. No selection rewrite is in scope. |
| Anchor/component producer | `trace_contrast_component`, `image_adjustments.rs:365-424` | Produces anchor, count, and extremal corners; its fresh visited bitmap can serve as `Mt` for the anchor-connected support model. |
| Stability producer | `component_boundary_is_stable`, `image_adjustments.rs:426-472` | Current independent population-or-area rejection is the regression site. Replace this decision coherently; do not layer another veto around it. |
| Proposal classifier | `propose_image_adjustment`, `image_adjustments.rs:229-271` | Decode, projection/area validation, stability, then near-identity classification. Preserve fail-closed ordering and response statuses. |
| API consumer/reporting | `assist_image_adjustment`, `controller/src/routes.rs:1131-1190`; `AdminImageAssistResponse`, `routes.rs:2425-2432` | Authenticates, checks initial and terminal media revision, reads private bytes, and serializes camelCase status/corners/message without creating corners. |
| Browser consumer | `detectImageEdges`, `controller/static-admin/admin.js:2822-2892` | A current four-corner `confident` response alone updates `reviewDraftAdjustment.perspective`; other outcomes update copy only. |
| Session/revision authority | `beginReviewSession`/`isCurrentReviewSession`, draft and assist revisions, plus `mediaRevision` recovery in `admin.js`; terminal server recheck in `routes.rs` | Stale responses, concurrent drag/mutation, and changed media remain non-authoritative. |
| Draft mutation boundary | `setPerspectiveCorner`, `markReviewDraftChanged`, `saveImageAdjustments`, and `resetImageAdjustments` in `admin.js` | Assist only changes the unsaved draft. Manual handles remain available; persistence requires explicit Save; Reset is explicit. This is D-08-29. |
| Persistence/publication boundary | `save_image_adjustment`/`reset_image_adjustment` in `routes.rs`, normal publish flow elsewhere | Detector output itself neither changes the private original (D-08-25) nor persists/publishes an adjustment (D-08-31). |
| Detector regressions | `controller/tests/image_adjustments.rs` | Fifteen tests cover skew/card positives, flat dark card, filled/noisy/sparse enclosures, low confidence, near identity, serialization, and transform behavior. Missing: a sanitized accurate dark card with weak noise-driven extrema. |
| Route contract | `admin_image_adjustment_routes_save_reset_review_and_assist` in `controller/tests/admin_workflow.rs:1318-1588` | Binds authenticated assist serialization, four-corner confident output, unavailable copy, private save/reset behavior, and revisions. |
| Browser behavior | `controller/tests/static_admin_behavior.mjs` | Binds stale assist rejection, non-mutating `notNeeded`, session/revision authority, drag authority, overlay/draft behavior, and conflict recovery. |
| Static contract/privacy | `controller/tests/static_admin.rs:566-690` | Binds manual fallback copy, confident gate, same-origin endpoint use, and absence of storage/provider identifiers and browser storage. |
| Held-out acceptance | Exact diagnostic `/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp` and 116 JPEGs under `/home/jgreenwa/Downloads/PHIL/autographs-3-001/autographs/` | Read-only private evidence. The diagnostic must be accurate `Confident`; the corpus must contain zero known/screened internal-region confident proposals. |

## Preserved failure matrix

| Input/outcome | Required result | Head `85db903` |
| --- | --- | --- |
| Filled low-contrast enclosure around dark content | `Unavailable` | Pass. |
| Noisy-border filled enclosure | `Unavailable` | Pass. |
| Sparse connected weak outer enclosure | `Unavailable` | Pass; the round-2 negative regression is faithful. |
| High-contrast skewed item | Accurate `Confident` | Pass in synthetic test. |
| Reliable high-contrast outer card | Accurate `Confident` | Pass in synthetic test. |
| Flat synthetic dark card | Accurate `Confident` | Pass, but does not bind the real noise path. |
| Sanitized noisy accurate dark-card analogue | Accurate `Confident` | Missing at head; must fail red before production changes. |
| Supplied accurate dark-card diagnostic | Accurate `Confident` | **Fail: `Unavailable`, no corners.** |
| Near-identity inset 2 / inset 5 | `notNeeded` / `Confident` | Pass in tests. |
| `Unavailable` API/UI consumption | No corners applied; manual adjustment remains available | Pass by source and behavior tests. |
| 116-image held-out corpus | Zero known/screened internal-region confident proposals; status-count changes explicitly assessed | Not yet evaluated for the revised model. |

## Scope fences

- Do not add OpenCV, AI/OCR, a new image library, or another service. That would
  violate the bounded heuristic/manual-fallback decision in D-08-26 and the
  deferred AI/advanced-enhancement scope.
- Do not change public response shapes, route authorization, media-revision
  authority, browser draft/save semantics, or manual controls.
- Do not commit the private WEBP, any corpus image, absolute private paths in
  executable tests, evaluator source/binaries, per-image output, or generated
  thumbnails.
- Do not treat more abstention as proof of correctness. The explicit positive
  controls and the confidence-count assessment are equally binding.

