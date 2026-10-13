---
phase: 08-admin-media-review-and-operational-posture
pr: 276
head: 85db903d4dd0ef70887816c9849101fac2827c7a
status: ready_to_resume
scope: detector-convergence-only
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
checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
checker_status: passed_cycle_2
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
spike_evidence:
  - .planning/spikes/008-detector-mask-contour-diagnostics/README.md
  - .planning/spikes/009-a-actual-contour-boundary/README.md
  - .planning/spikes/009-b-local-gradient-rectangle/README.md
pr_evidence: https://github.com/jetsaredim/autographs/pull/276#issuecomment-6156047878
---

# PR #276 Detector Convergence Revised Implementation Plan — Iteration 3

## Objective

Replace the current independent weak-mask
`population_growth || area_growth` stability veto with the spike-validated,
bounded local-gradient/Hough conservative veto. Preserve the existing primary
component and its corners; return `Unavailable` only when a materially larger
rectangle has independently localized edge and signed contrast support.

This is a surgical detector correction, not a Phase 8 replan. Route, API,
browser authority, manual adjustment, persistence, derivative, and publication
contracts remain unchanged per D-08-25, D-08-26, and D-08-29.

## Mandatory execution gate

Spikes 008 and 009 establish design feasibility, not production authorization.
The iteration-2 review approved a different algorithm and was superseded by its
named stop condition. Fresh iteration-3 checker cycle 2 passed this plan and the
linked assumptions with zero actionable concerns in
`08-PR-276-DETECTOR-PLAN-REVIEW.md`; its evidence path is recorded in
frontmatter, so detector/test execution is `ready_to_resume` under every gate
below. This authorization does not itself permit a commit, push, PR comment,
private-asset copy, or deployment.

## Evidence basis

- Spike 008 showed that the named slab's border-median component floods the
  frame before a usable physical outer contour appears, while local Canny/Hough
  boundary evidence remains visible.
- Spike 009a invalidated contour-only behavior: ignoring flood trusts the
  internal primary and abstaining on all floods rejects the exact positive.
- Spike 009b froze a local-gradient/Hough contract before running the exact
  controls or corpus. It retained the exact positive, rejected the named slab
  with four valid sides, produced 79 accurate primary confidences and 37 safe
  abstentions across 116 images, and left zero known/screened internal proposals.
- The spike evaluator processed two prepared thumbnails in `0.21s` with about
  `8.2 MiB` peak resident memory. That is feasibility evidence, not the
  production rollback baseline; the plan establishes a warmed identical
  public-path workload against head before source changes. `imageproc 0.27`
  already exists in the controller, so no dependency change is needed.

## Exact file scope

### Files allowed to change after fresh review

- `controller/tests/image_adjustments.rs`
- `controller/src/image_adjustments.rs`

### Verification-only files that must remain byte-unchanged

- `controller/Cargo.toml`
- `controller/Cargo.lock`
- `controller/src/routes.rs`
- `controller/static-admin/admin.js`
- `controller/tests/admin_workflow.rs`
- `controller/tests/static_admin_behavior.mjs`
- `controller/tests/static_admin.rs`

### Read-only evidence outside Git

- the exact `b991...webp` positive control;
- the exact `20240415_225421.jpg` negative control;
- the other 115 originals in the held-out corpus; and
- a temporary production-detector evaluator and its reports under
  `/tmp/autographs-pr276-detector-eval.*`.

Do not place private paths, images, masks, overlays, contact sheets, evaluator
source/binaries, per-image output, or generated derivatives in Git.

## Frozen implementation contract

### Preserve the primary

Keep the current 640-pixel thumbnail, border median, adaptive threshold, largest
qualifying 8-connected component, extrema corner order, normalized projection
validation, primary area bounds, near-identity decision, and returned primary
corners. The gradient candidate is never returned or applied.

Delete the weak-threshold stability loop and its component-growth/area-growth
constants. The new veto replaces that behavior; it must not run before, after,
or inside the old scalar veto.

### Generate bounded independent candidates

Use focused private helpers in `image_adjustments.rs` to implement the complete
009b contract on the same thumbnail:

1. Convert the RGB thumbnail to luma; run Canny at `20/60`.
2. Detect Hough lines with vote threshold `min(width, height) / 3` and
   suppression radius `8`.
3. Group vertical lines at `0..=15` or `165..180` degrees and horizontal lines
   at `75..=105` degrees.
4. Measure each in-frame infinite line using a two-pixel edge corridor. Keep
   lines with full-line support `>= 0.25` and longest unsupported gap `<= 0.55`.
5. Rank by `support - 0.25 * longest_gap`, truncate to at most 12 lines per
   axis, then sort by mid-frame position.
6. Enumerate no more than 66 pairs per axis and 4,356 rectangle combinations.
   Require pair span `>= 0.50` of the corresponding image dimension.
7. Intersect the four lines in imageproc's continuous geometric coordinates.
   The geometric image boundary is `[0,width] × [0,height]`; pixel centers end
   at `width - 1` and `height - 1`. Port the spike's exact inclusive condition:
   every corner satisfies `x >= -4`, `y >= -4`, `x <= width + 4`, and
   `y <= height + 4`. This extends four coordinate units beyond the geometric
   image boundary, not four units beyond the final pixel center. Require
   candidate area `> max(0.50, primary_area + 0.05)`.
8. On each finite side, require edge support `>= 0.35`, maximum gap `<= 0.40`,
   mean absolute luma contrast `>= 5`, and contrast-sign coherence `>= 0.60`.
   Edge support uses the two-pixel corridor; contrast samples are four pixels
   inward and outward along the center-facing normal.
9. Rank candidates first by valid-side count, then by
   `area * mean_side_support * valid_side_fraction`. A candidate with at least
   three valid sides triggers `Unavailable` with no corners.

No constant may change after exact-control or corpus evaluation. Do not add
containment, centroid, population, weak-threshold, filename, orientation, or
coordinate special cases around this contract.

## Ordered implementation tasks

### Task 1 — RED/CHARACTERIZE: bind the mechanism with sanitized fixtures

**Files:** `controller/tests/image_adjustments.rs`; temporary read-only
diagnostics under `/tmp`.

**Preconditions:** Production detector code is byte-equivalent to reviewed head
`85db903`; this iteration has passed fresh independent review; all private
evidence is available read-only outside Git.

**Actions:**

1. Before any source or committed-test edit, create a temporary release
   evaluator under `/tmp` that reads each original, then measures only the call
   to public `propose_image_adjustment` with `Instant`. Freeze a workload
   manifest containing the exact positive, exact named slab, then all 116 corpus
   originals in bytewise-sorted filename order; retain the slab's corpus entry,
   so both baseline and post-change measurements execute the same 118 calls.
   Build the evaluator once against `85db903`, run the already-built binary once
   over the complete manifest as an unrecorded warm-up, then run the binary
   directly under `/usr/bin/time` to capture per-image, summed total, p50, p95,
   maximum, and maximum RSS. Keep the manifest, source, binary, and output under
   `/tmp`; do not use `cargo run` for the measured pass.
2. Preserve the 15 existing detector tests. Treat
   `auto_assist_keeps_outer_card_when_its_border_has_reliable_contrast` as the
   reliable-positive gradient contract: it must remain accurately `Confident`
   because no materially larger supported rectangle exists.
3. Add a deterministic low-contrast physical-slab fixture enclosing a
   high-contrast internal card. Its slab-to-background RGB distance must stay
   below the primary color-distance threshold while its local luma transition
   yields four long outer Canny/Hough sides. The primary component and corners
   must be the internal card; the outer candidate must exceed both area gates
   and have at least three valid sides.
4. Add
   `auto_assist_abstains_when_gradient_rectangle_encloses_stable_internal_primary`.
   The committed expectation is `Unavailable` with no corners. Before accepting
   the fixture, use the temporary harness to record that bypassing the new veto
   would return `Confident` at the internal-card corners.
5. Add a Hough-rich/noisy accurate-card fixture. Place long/dashed line evidence
   outside the accurate primary with alternating inside/outside polarity or
   discontinuities so raw Hough pairs exist but fewer than three candidate sides
   satisfy the complete localized support/gap/contrast/sign contract.
6. Add
   `auto_assist_keeps_accurate_primary_when_hough_lines_lack_localized_coherent_contrast`.
   Require accurate `Confident` primary corners within the existing 0.025
   normalized tolerance. This proves line abundance alone cannot veto.
7. Establish two-stage RED evidence. On `85db903`, require the noisy accurate
   test to fail if the scalar weak-mask `||` sees its expansion. Then temporarily
   remove/bypass the old stability decision without adding the gradient veto:
   the physical slab test must fail as `Confident` at the internal primary while
   the reliable and noisy accurate positives remain accurate. If the slab does
   not expose the internal primary in this intermediate, reject the fixture and
   stop rather than accepting a final-status-only test.

**RED/characterization verification:**

- `cargo build --release --manifest-path /tmp/autographs-pr276-detector-eval/Cargo.toml`
- `/tmp/autographs-pr276-detector-eval/target/release/autographs-pr276-detector-eval --manifest "$PR276_WORKLOAD" --warm-up`
- `/usr/bin/time -v -o /tmp/autographs-pr276-detector-baseline.time /tmp/autographs-pr276-detector-eval/target/release/autographs-pr276-detector-eval --manifest "$PR276_WORKLOAD" --metrics /tmp/autographs-pr276-detector-baseline.json`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_outer_card_when_its_border_has_reliable_contrast -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_gradient_rectangle_encloses_stable_internal_primary -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_primary_when_hough_lines_lack_localized_coherent_contrast -- --nocapture`

**Done:** The three sanitized contracts exist; the low-contrast slab's causal
internal-primary RED is observed without the gradient veto; and no private or
diagnostic artifact is committed.

### Task 2 — GREEN/REFACTOR: replace scalar stability with the bounded gradient veto

**Files:** `controller/src/image_adjustments.rs`,
`controller/tests/image_adjustments.rs`.

**Actions:**

1. Import only the existing `imageproc` luma/Canny/Hough primitives needed by
   the contract. Add no crate, feature, manifest, lockfile, API type, or public
   function.
2. Remove `AUTO_CORRECTION_MIN_STABILITY_THRESHOLD`,
   `AUTO_CORRECTION_MAX_STABLE_COMPONENT_GROWTH`, the old stability use of
   `AUTO_CORRECTION_MAX_STABLE_AREA_GROWTH`, and
   `component_boundary_is_stable`. Remove the component anchor if it becomes
   unused; retain component population for the existing minimum-component gate.
3. Before implementing the bounds helper, add a private `#[cfg(test)]` unit test
   named `gradient_candidate_bounds_match_imageproc_geometric_coordinates`.
   For representative `width` and `height`, require acceptance at `x = -4`,
   `y = -4`, `x = width + 4`, and `y = height + 4`, including `x = width` and
   `y = height` as geometric-boundary coordinates; require rejection just below
   `-4` and just above `width + 4` / `height + 4`. The test and helper must use
   inclusive comparisons and must not derive the upper bound from
   `width - 1` / `height - 1`.
4. Add small private types/helpers for bounded axis lines, points, side
   evidence, intersections, polygon area, full-line/finite-side edge sampling,
   local signed contrast, axis selection, exact geometric-bounds admission, and
   candidate search. Port the frozen contract exactly; keep rendering, file I/O,
   labels, JSON, and spike report structures out of production.
5. Truncate each axis group to 12 before pair enumeration. Keep the maximum
   search at 4,356 rectangle combinations and sample only the bounded 640-pixel
   thumbnail. Add `debug_assert!` guards around line caps and the derived
   candidate ceiling so later refactors cannot silently make the search
   unbounded.
6. After existing projection and primary-area validation, call a boolean helper
   equivalent to `has_materially_larger_gradient_boundary(image, area_ratio)`.
   If true, return the ordinary `Unavailable` response. Otherwise continue to
   the existing near-identity and original-primary `Confident` paths.
7. Never copy candidate corners into `normalized_corners`, response corners, an
   `ImagePerspective`, or any adjustment. The detector's only new observable
   behavior is a conservative status change from primary confidence to
   `Unavailable`.
8. Make the three mechanism tests, exact geometric-bounds unit test, and all
   existing detector tests green. Then
   perform the causal mutation: temporarily force the gradient helper to return
   false or bypass its call and rerun the physical-slab test. It must fail with
   the internal `Confident` primary. Restore the real veto and rerun green. If a
   constant must change to pass, stop and reassess instead of tuning.

**GREEN verification:**

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_outer_card_when_its_border_has_reliable_contrast -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_gradient_rectangle_encloses_stable_internal_primary -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_primary_when_hough_lines_lack_localized_coherent_contrast -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --lib gradient_candidate_bounds_match_imageproc_geometric_coordinates -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`

**Done:** The old population/area stability loop is absent; the frozen bounded
gradient veto is the sole outer-boundary ambiguity decision; sanitized slab and
noise cases are causally green; original primary corners remain the only
possible confident output.

### Task 3 — VALIDATE: exact controls, complete corpus, boundedness, consumers, and privacy

**Files:** the two allowed implementation files; verification-only consumer and
contract files; private inputs and evaluator under `/tmp`.

**Actions:**

1. Build an ephemeral evaluator that calls the production
   `propose_image_adjustment`; do not copy the spike classifier. Run the exact
   positive first. Require `Confident` primary corners near
   `(0.061,0.061)`, `(0.925,0.056)`, `(0.939,0.934)`, `(0.057,0.933)` with at
   most 0.03 normalized error per coordinate.
2. Run the exact named slab second. Require `Unavailable`. Any `Confident`
   result stops execution, especially the forbidden internal quadrilateral near
   `(0.186441,0.306729)`, `(0.825666,0.286385)`, `(0.738499,0.870110)`,
   `(0.188862,0.879499)`.
3. Only after both exact gates pass, evaluate all 116 sorted originals with the
   unchanged production constants. Expected counts are 79 `Confident`, 37
   `Unavailable`, and 0 `NotNeeded`. A count difference is acceptable only when
   every changed image is geometry-inspected and the reason is recorded; zero
   known or screened internal-region `Confident` results is non-negotiable.
4. Visually inspect every production `Confident` overlay. Repeat the 12%-side
   proximity screen; `20251231_011140` may remain the sole exception only if its
   full-image geometry still follows the outer rotated card. Require all three
   graded-slab siblings to remain `Unavailable`.
5. Produce an outside-Git before/after table for every status or primary-corner
   change relative to the spike result and reviewed head. Counts alone cannot
   accept geometry. Do not post or commit that output until the normal reviewed
   PR audit-trail workflow authorizes it.
6. Rebuild the unchanged temporary evaluator once after implementation, run its
   already-built binary once as an unrecorded warm-up on the frozen 118-call
   manifest, then measure the binary directly under `/usr/bin/time` using the
   identical manifest, order, host, output mode, and per-call timer placement as
   the head baseline. Capture per-image, summed total, p50, p95, maximum, and
   maximum RSS. Compilation and cache population are excluded from both sides.
   Require post p95 `<= 1.0s` and `<= 3.0×` head p95; post maximum `<= 2.0s`
   and `<= 3.0×` head maximum; summed total `<= 3.0×` head total; and max
   RSS `<= max(1.5× head RSS, head RSS + 32 MiB)`. These are mandatory rollback
   gates: the absolute latency limits protect the synchronous admin action,
   while the relative and RSS limits allow bounded Canny/Hough work without
   accepting pathological enumeration or working-set growth. Record p50 and all
   per-image deltas for assessment even though tail/total/RSS drive the gate.
   Independently confirm 12-per-axis truncation precedes nested pairing and the
   candidate ceiling is 4,356.
7. Run route/browser/static contracts and full controller checks. Confirm
   route/UI/manifest/lockfile diffs are empty, then delete the temporary
   evaluator and private outputs.

**Private evaluator invocation shape:**

- `cargo build --release --manifest-path /tmp/autographs-pr276-detector-eval/Cargo.toml`
- `/tmp/autographs-pr276-detector-eval/target/release/autographs-pr276-detector-eval --manifest "$PR276_WORKLOAD" --warm-up`
- `/usr/bin/time -v -o /tmp/autographs-pr276-detector-post.time /tmp/autographs-pr276-detector-eval/target/release/autographs-pr276-detector-eval --manifest "$PR276_WORKLOAD" --metrics /tmp/autographs-pr276-detector-post.json`

The evaluator source and manifest are identical between baseline and post
measurements; only the path dependency's detector implementation changes. The
task-specific variable is operator-local and must not be written into a
repository file. Measured commands invoke the evaluator binary, never Cargo.

**Consumer/controller verification:**

- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow admin_image_adjustment_routes_save_reset_review_and_assist -- --nocapture`
- `node controller/tests/static_admin_behavior.mjs`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `node --check controller/static-admin/admin.js`
- `cargo test --manifest-path controller/Cargo.toml --all-features`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `git diff --check -- controller/src/image_adjustments.rs controller/tests/image_adjustments.rs`
- `git diff --exit-code -- controller/Cargo.toml controller/Cargo.lock controller/src/routes.rs controller/static-admin/admin.js controller/tests/admin_workflow.rs controller/tests/static_admin_behavior.mjs controller/tests/static_admin.rs`

**Done:** Production passes both exact controls, the complete corpus has no
internal confidence and all changes are assessed, bounded release performance
is preserved, all consumer/controller/privacy gates pass, and only the two
allowed files differ.

## Acceptance criteria

- The old `population_growth || area_growth` stability behavior and its weak
  threshold loop are replaced, not layered with or around the gradient veto.
- The fixed 009b contract is ported exactly: Canny `20/60`; Hough vote
  `min-dimension/3`, suppression `8`; 15-degree axis grouping; 12 lines per
  axis; full-line support/gap `0.25/0.55`; pair span `0.50`; candidate area
  floor `0.50` and growth `0.05`; side support/gap `0.35/0.40`; mean contrast
  `5`; sign coherence `0.60`; at least three valid sides; two-pixel edge
  corridor and four-pixel contrast offset. Candidate corners use imageproc's
  geometric image boundary `[0,width] × [0,height]` and the exact inclusive
  spike interval `[-4,width+4] × [-4,height+4]`; this is four coordinate units
  beyond the geometric boundary, not beyond the last pixel center.
- Candidate enumeration is capped at 4,356 rectangles after line truncation.
- Gradient candidates can only veto. Every `Confident` or `NotNeeded`
  classification still derives from the original primary corners.
- The reliable-positive, physical-slab/internal-primary, and Hough-rich/noisy
  accurate-card contracts pass, and bypassing the veto causally restores the
  slab's forbidden internal confidence.
- Existing high-contrast skew, dark card, sparse/filled/noisy enclosures,
  low-confidence, transformation, serialization, and inset-2/inset-5 behaviors
  retain their contracts.
- The exact positive is accurate `Confident`; the exact named slab and all three
  slab siblings are `Unavailable`.
- All 116 images are evaluated with unchanged constants. Expected counts are
  79 `Confident`, 37 `Unavailable`, 0 `NotNeeded`; any justified difference is
  individually recorded, and zero internal `Confident` is mandatory.
- Every retained primary is visually screened, including the accurate
  `20251231_011140` side-distance exception.
- The already-built production-path evaluator uses the identical warmed
  118-call workload before and after, excluding build/cache work. Post p95 is
  `<= 1.0s` and `<= 3.0×` baseline, post max is `<= 2.0s` and `<= 3.0×`
  baseline, total is `<= 3.0×` baseline, and max RSS is
  `<= max(1.5× baseline, baseline + 32 MiB)`.
- Authentication, terminal `mediaRevision`, camelCase serialization,
  session/draft/assist authority, non-mutating unavailable/unexpected behavior,
  explicit Save, private-original immutability, and manual corner fallback are
  unchanged.
- Only `controller/src/image_adjustments.rs` and
  `controller/tests/image_adjustments.rs` change; no dependency or private
  artifact enters Git.

## Stop and rollback conditions

Stop, restore only the two in-scope files to the last reviewed state, preserve
unrelated worktree changes, and return to `reassessment_required` if:

- fresh independent review has not approved both artifacts;
- the mechanism-derived slab does not expose internal primary confidence when
  the gradient veto is bypassed;
- the Hough-rich accurate-card fixture is vetoed merely because lines exist;
- any frozen constant, line cap, candidate ceiling, or scoring order must change
  after exact or corpus evaluation;
- the scalar weak-mask veto remains in any production decision path;
- gradient candidate corners reach response or adjustment data;
- the exact positive is not accurate `Confident` or the exact slab is
  `Confident`;
- any known/screened corpus proposal selects an internal region;
- a 79/37/0 difference is not individually inspected and recorded;
- any slab sibling becomes `Confident` or the side-distance exception is no
  longer geometrically accurate;
- the bounded search expands beyond 12 lines per axis or 4,356 candidates;
- geometric bounds differ from the exact inclusive
  `[-4,width+4] × [-4,height+4]` interval under imageproc's
  `[0,width] × [0,height]` boundary convention;
- the warmed already-built evaluator's p95, maximum, total, or max RSS exceeds
  any mandatory absolute or relative guardrail;
- route/UI/API/persistence/dependency changes appear necessary;
- any consumer, formatting, production-persistence, Clippy, or privacy gate
  fails; or
- any private/evaluator artifact appears in Git.

Do not start another point-fix round from a stop condition. Record the new
evidence and revise the shared contract first.

## Surgical source coverage audit

| Source item | Coverage | Task |
| --- | --- | --- |
| MEDIA-06 / ADMIN-06 / D-08-26 | Useful primary assistance remains; independently supported larger boundaries abstain; manual controls remain authoritative. | 1-3 |
| MEDIA-05 / D-08-25 | Private originals and evaluator artifacts remain outside Git and are never mutated. | 3 |
| D-08-29 | Detector output remains draft-local and persistence requires explicit Save. | 3 |
| Four convergence invariants | Original outer-primary confidence, physical-boundary discrimination, positive usability, and non-mutating fallback are jointly tested. | 1-3 |
| Spike 008 diagnosis | Contour flood explains why component contours are not the production boundary source. | Contract |
| Spike 009a invalidation | Contour-only/flood policies stay excluded. | Contract, stop gates |
| Spike 009b validation | Frozen bounded local-gradient/Hough veto is ported and revalidated against production. | 1-3 |
| Exact controls and 116-image corpus | Positive first, slab second, then 79/37/0 expectation, zero internal confidence, complete visual/change review. | 3 |
| Consumer and privacy boundaries | Route/browser/static checks and no-diff gates preserve authority, fallback, and private identifiers. | 3 |
| Deferred AI/advanced enhancement | Excluded; no OCR/AI, content-aware replacement corners, service, dependency, or admin redesign. | Scope fence |

No in-scope source item is deferred. Execution remains unauthorized until a
fresh checker passes this iteration and both artifact frontmatters record that
evidence.
