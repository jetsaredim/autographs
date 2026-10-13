---
phase: 08-admin-media-review-and-operational-posture
pr: 276
head: 85db903d4dd0ef70887816c9849101fac2827c7a
status: reassessment_required
scope: detector-design-spike-only
requirements:
  - MEDIA-06
  - ADMIN-06
  - MEDIA-05
decisions:
  - D-08-25
  - D-08-26
  - D-08-29
assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
reassessment_iteration: 3
prior_independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
independent_review: pending
---

# PR #276 Detector Convergence Revised Plan — Design Spike Gate

## Objective

Determine why the real `20240415_225421.jpg` slab defeats the four-side
extrema-chord coherence/witness abstraction even though all three sanitized
morphologies, all 17 detector tests, and the exact positive diagnostic pass.
Produce enough privacy-safe intermediate evidence to select and review one
coherent detector contract before any third coder round.

This artifact is deliberately not an implementation-ready detector plan. It
invalidates the authorization granted by the second independent review after
that reviewed plan hit its named stop condition. It authorizes read-only
diagnostics and planning evidence only—no production-code or committed-test
change, commit, push, PR comment, private-asset copy, or deployment.

## Why the second plan is superseded

The second implementation replaced `Qt`-contains-`Q0` with exact `M0 ⊆ Mt`
and retained the reviewed four-side support measurement. It produced all of the
expected intermediate green signals:

- sparse enclosing perimeter => `Unavailable`;
- noise-connected accurate card => accurate `Confident`;
- sanitized containment-bypass slab => `Unavailable`;
- all 17 detector tests => pass; and
- exact `b9912131-8781-4047-a44c-03e5ca1e56a9.webp` => `Confident` with target
  corners.

Despite that, the exact `20240415_225421.jpg` again returned the forbidden
internal quadrilateral near `(0.186441,0.306729)`, `(0.825666,0.286385)`,
`(0.738499,0.870110)`, `(0.188862,0.879499)`. Removing containment was therefore
insufficient. The unchanged extrema-chord coherence/witness abstraction itself
is unvalidated for the real slab. Two follow-on diagnostic agents produced no
usable deeper trace before they were terminated as unresponsive. The changes
were rolled back and the worktree remained source-clean.

## Authorization and file boundaries

### Current spike: read-only repository inputs

- `controller/src/image_adjustments.rs`
- `controller/tests/image_adjustments.rs`
- `controller/src/routes.rs`
- `controller/static-admin/admin.js`
- `controller/tests/admin_workflow.rs`
- `controller/tests/static_admin_behavior.mjs`
- `controller/tests/static_admin.rs`

No repository source or test file may be modified during this spike. An
ephemeral harness, masks, contours, overlays, trace output, and binaries must
live only under `/tmp/autographs-pr276-detector-spike.*` and be removed after a
privacy-safe summary is captured.

### Read-only private controls

- `/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp`
- `/home/jgreenwa/Downloads/PHIL/autographs-3-001/autographs/20240415_225421.jpg`
- the remaining 115 JPEG originals in that corpus only after the paired
  positive/slab analysis produces a candidate explanation

### Future implementation scope, not currently authorized

Any later reviewed coder plan remains limited to:

- `controller/src/image_adjustments.rs`
- `controller/tests/image_adjustments.rs`

Route, API, browser, persistence, publication, package, and dependency changes
remain out of scope. That two-file fence does not grant permission to edit those
files before the resume gate is satisfied.

## Ordered reassessment tasks

### Task 1 — Build a bounded, ephemeral geometry diagnostic

**Repository files:** read-only `controller/src/image_adjustments.rs` and
`controller/tests/image_adjustments.rs`.

**Ephemeral files:** `/tmp/autographs-pr276-detector-spike.*` only.

**Actions:**

1. Reuse the detector's actual border median, adaptive threshold, descending
   relative/fixed probe order, anchor, and 8-connected tracing semantics in a
   temporary harness. Do not change production visibility or add diagnostic
   branches to the repository.
2. Run the exact positive, exact named slab, and three sanitized morphology
   controls through the same thumbnail and threshold path.
3. At the primary threshold and every actual weaker probe, emit the threshold
   mask, exact anchor-component mask, population, extrema, extrema-quadrilateral
   area, and retained-primary membership.
4. Extract the actual component boundary. An ordered contour is preferred; if
   contour ordering is the open design problem, emit all boundary pixels and
   connected boundary runs without pretending they are an ordered polygon.
5. Instrument the failed model as diagnostics: rasterized extrema chords,
   component corridor hits, outward-witness hits, per-side coverage, longest
   gaps, and the exact trigger/check that allows the named slab to remain
   `Confident`.
6. Record contour perimeter/runs and contour-to-candidate-side distances. Also
   sample local image-gradient or color-transition strength and continuity on
   both sides of plausible rectangular boundary segments. These are observations
   for model comparison, not new thresholds or production logic.
7. Keep private images and all derived visual or per-image artifacts outside
   Git. Do not include executable absolute private paths in repository files.

**Verification:**

- The paired exact controls each have a complete probe-by-probe trace with masks,
  component boundary evidence, failed-model per-side values, and gradient data.
- The three sanitized controls have the same schema so explanations can be
  compared rather than inferred from final statuses.
- `git diff --exit-code -- controller/src/image_adjustments.rs controller/tests/image_adjustments.rs controller/src/routes.rs controller/static-admin/admin.js controller/tests/admin_workflow.rs controller/tests/static_admin_behavior.mjs controller/tests/static_admin.rs`
- `git status --short` contains no private input, mask, overlay, evaluator,
  binary, trace, or generated derivative.

**Stop:** If the harness does not reproduce the exact positive target corners
and exact slab internal corners through the same detector path, fix the harness
only; do not draw a detector-design conclusion or edit production code.

**Done:** The missing intermediate evidence exists privately and the exact check
that lets the real slab escape is measured rather than hypothesized.

### Task 2 — Compare spatial design hypotheses and create a privacy-safe report

**Repository output:** a new linked planning report may be created only as
`.planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-DESIGN-SPIKE.md`.

**Actions:**

1. Compare at least two design families against every row of the failure matrix:
   actual component-contour evidence and local-gradient rectangular-boundary
   evidence. A contour-derived hull or multi-threshold contour persistence may
   be included, but four extrema chords alone are the failed baseline, not a
   candidate presumed valid.
2. For each family, state what measured property separates the exact positive
   from the named slab, which sanitized cases support or contradict it, and what
   false-confidence and false-abstention risks it creates.
3. Prefer investigation of the actual anchor-component contour or boundary runs:
   determine whether the slab exposes a sustained enclosing boundary that the
   extrema chords cut across, while the accurate card's weak tendrils remain
   discontinuous or non-rectangular. Independently assess whether local image
   transitions provide four sustained rectangular sides on the slab but not the
   noise-connected positive.
4. Do not select numeric constants from one private image. If a design cannot be
   stated in normalized/geometric terms and reproduced in sanitized inputs,
   reject it or leave it unresolved.
5. Specify a sanitized generated RED fixture derived from the measured bypass,
   including which intermediate facts it must assert. It must reproduce the
   real failure mechanism, not merely resemble the slab or finish with the same
   status.
6. Record only privacy-safe aggregate facts and normalized geometry. Do not
   embed private pixels, masks, thumbnails, complete per-image traces, or machine
   paths beyond the already documented read-only controls.

**Verification:**

- The report answers where the named slab first diverges, why the four-side
  model misses it, and what property does or does not separate it from the exact
  positive.
- Every design family is assessed against accurate dark card, sparse enclosure,
  sanitized containment-bypass slab, exact slab, high-contrast card/skew,
  near-identity, and prior corpus risks.
- The report explicitly says `no approved algorithm` unless one coherent model
  is supported by both exact controls and reproducible sanitized evidence.

**Stop:** If neither design family discriminates the paired exact controls, or
if the explanation depends on filename, exact coordinates, or per-image
constants, record the negative result and keep `reassessment_required`.

**Done:** A durable, privacy-safe report narrows or rejects the design options
without claiming implementation authorization.

### Task 3 — Revise and independently review before coder work

**Files:** this artifact, the assumptions artifact, the design-spike report, and
a new independent review artifact or an explicitly versioned new review section.

**Actions:**

1. Update the assumptions from measured spike evidence. Preserve all five
   finding-lineage rounds, four shared invariants, producer/consumer/mutation
   inventory, and full failure matrix.
2. Only if the spike identifies a coherent discriminating model, replace this
   spike gate with a new test-first implementation plan. It must specify the
   diagnostic-derived sanitized RED, exact positive, exact named slab, full
   116-image screen, count/corner-change assessment, consumer contracts,
   rollback, and source/privacy fences.
3. Require independent review of both updated assumptions and the new
   implementation plan. The review must assess the spike evidence and must not
   reuse the iteration-2 `passed` verdict as authorization.
4. Change status to `ready_to_resume` only after both reviewed artifact paths are
   recorded. A status-only edit is invalid.

**Verification:**

- Frontmatter remains `status: reassessment_required` and
  `independent_review: pending` until a fresh review actually passes.
- `08-REVIEW.md` records that the iteration-2 authorization is superseded by its
  stop condition.
- The reviewed plan's future implementation scope is exactly
  `controller/src/image_adjustments.rs` and
  `controller/tests/image_adjustments.rs`.

**Done:** Either fresh evidence and review authorize a third coder round, or the
detector remains durably blocked with the unresolved design question recorded.

## Acceptance criteria for this reassessment stage

- The exact positive and named slab have comparable probe-by-probe masks,
  component contours/boundaries, failed-model per-side diagnostics, and local
  transition evidence.
- The real slab's escape mechanism is identified with measured evidence; the
  absence of traces from the two terminated diagnostic attempts is not filled by
  inference.
- Actual-contour and local-gradient boundary families are compared at design
  level; no algorithm or constants are pre-approved.
- A future sanitized RED is specified from the observed mechanism before
  production edits.
- The four shared invariants remain binding: outer-boundary confidence,
  enclosure-versus-noise discrimination, useful positive assistance with safe
  abstention, and non-mutating unavailable/unexpected UI outcomes.
- The future plan retains high-contrast skew/card, reliable-card,
  near-identity, API serialization, authentication and terminal media-revision,
  browser session/draft/assist authority, explicit Save, manual fallback, and
  private-original immutability.
- The full 116-image later evaluation still requires zero known/screened
  internal-region confident proposals and explicit assessment of every status or
  corner change; counts alone cannot establish safety.
- No private image, mask, overlay, evaluator, binary, per-image output,
  generated derivative, or executable absolute private path enters Git.
- No source/test edit, commit, push, or PR comment occurs during this stage.

## Stop and guardrail conditions

Keep or restore `reassessment_required` immediately if:

- any production or committed test edit begins before the spike and independent
  review gates complete;
- the exact positive or exact slab cannot be reproduced by the harness;
- a proposed model preserves the failed extrema-chord abstraction and merely
  layers another threshold, Boolean, containment, centroid, or coordinate gate;
- a proposed contour/gradient rule is selected before its distinguishing
  evidence is observed on both exact controls and reproduced in sanitized data;
- the named slab, any known/screened corpus case, or a sanitized enclosure is
  allowed internal `Confident` geometry;
- the exact positive is sacrificed merely to increase abstention;
- route/UI changes are proposed to hide a detector failure;
- the two-file future implementation scope expands without a new reviewed
  reassessment; or
- private/evaluator artifacts appear in Git status.

## Resume criteria

A third coder round may begin only after all of these are true:

1. the bounded harness completed on the exact positive, exact slab, and required
   sanitized fixtures;
2. a durable privacy-safe spike report explains the measured bypass and compares
   at least the actual-contour and local-gradient design families;
3. a diagnostic-derived sanitized RED binds the real mechanism;
4. updated assumptions state one coherent candidate contract, or explicitly
   remain inconclusive;
5. a new implementation-ready plan preserves the two-file scope, all consumer
   and privacy constraints, the exact paired controls, and the 116-image screen;
6. an independent reviewer passes those updated artifacts and the new evidence
   paths are recorded in frontmatter; and
7. no source/test change has preceded that authorization.

Until then, the status is `reassessment_required`; the iteration-2 plan review
is historical evidence only and cannot authorize implementation.

## Source coverage audit

| Source item | Current coverage |
| --- | --- |
| MEDIA-06 / ADMIN-06 / D-08-26 | The spike preserves useful bounded assistance and manual fallback while investigating the unresolved confidence boundary. |
| MEDIA-05 / D-08-25 | Private originals and derived diagnostics stay read-only/outside Git. |
| D-08-29 | Detector diagnostics do not mutate or persist browser drafts; explicit Save remains authoritative. |
| Round-3 convergence guard | Authorization is revoked after a named stop condition; lineage, assumptions, inventory, matrix, and fresh-review gate remain durable. |
| Round-5 evidence | All synthetic tests and the positive control passing no longer count as sufficient evidence because the exact slab still fails. |
| Two-file implementation boundary | Preserved as future scope, with no current source modification authorized. |
| Deferred AI/advanced enhancement | Still excluded; no OCR/AI, service, broad admin redesign, or package addition is proposed. |

No source item is intentionally deferred inside this reassessment. Detector
implementation is intentionally unauthorized because the necessary design
evidence does not yet exist.
