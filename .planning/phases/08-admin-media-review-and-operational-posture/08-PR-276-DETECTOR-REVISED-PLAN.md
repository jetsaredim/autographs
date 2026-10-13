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
reassessment_iteration: 2
prior_independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
---

# PR #276 Detector Convergence Revised Plan

## Objective

Replace the `85db903` scalar population/area `||` decision with one coherent
boundary-change decision that rejects sparse, filled, and non-nested-extrema
slab boundaries while retaining an accurate card whose weak-threshold component
is connected only to noise-driven extrema. Cross-threshold identity is proven
by retained anchor-component pixels, never extrema-quadrilateral containment.

This is not a Phase 8 replan. The only production behavior in scope is the
private Rust detector. The existing route, API shape, browser authority,
draft-local save model, manual fallback, and publishing boundaries remain
unchanged per D-08-25, D-08-26, and D-08-29.

## Mandatory execution gate

The prior independent review is superseded by execution evidence: its strict
quadrilateral-containment prerequisite skipped coherence analysis for the known
`20240415_225421.jpg` slab failure. The attempted implementation was rolled
back without a source commit. This second revision and its assumptions passed
fresh independent review in `08-PR-276-DETECTOR-PLAN-REVIEW.md`; the evidence
path is recorded in frontmatter, so the checkpoint is `ready_to_resume`.

This planning artifact does not authorize a commit, push, PR comment, private
asset copy, or deployment.

## Exact file scope

### Files to modify

- `controller/tests/image_adjustments.rs`
- `controller/src/image_adjustments.rs`

### Verification-only files; do not modify unless a new reviewed plan expands scope

- `controller/src/routes.rs`
- `controller/static-admin/admin.js`
- `controller/tests/admin_workflow.rs`
- `controller/tests/static_admin_behavior.mjs`
- `controller/tests/static_admin.rs`

### Read-only private inputs and ephemeral output

- `/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp`
- `/home/jgreenwa/Downloads/PHIL/autographs-3-001/autographs/` (exactly 116 JPEG originals at planning time)
- A temporary evaluator created under `/tmp/autographs-pr276-detector-eval.*`
  and deleted after results are recorded outside Git.

No dependency or package-file change is permitted.

## Ordered implementation tasks

### Task 1 — RED/CHARACTERIZE: bind noise, sparse perimeter, and non-nested slab morphologies

**Files:** `controller/tests/image_adjustments.rs`; read-only private inputs;
temporary evaluator under `/tmp`.

**Preconditions:** Production detector code is byte-equivalent to `85db903`; the
exact diagnostic exists; the corpus directory contains exactly 116 JPEGs; this
second revision has passed independent review.

**Actions:**

1. Before production edits, run an external temporary Cargo evaluator against
   the exact diagnostic and the sorted 116-image corpus. Record outside Git the
   three status counts, every Confident filename/corners, the four known
   internal-region cases, and the 12%-side-proximity screen. Record
   `20240415_225421.jpg` separately with the known rejected internal coordinates.
   Use this measured head baseline; do not reuse counts from another detector.
2. Re-add the fully generated, non-private
   `dark_card_with_connected_weak_noise_fixture` and
   `auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema`.
   Weak tendrils must join the accurate dark card to distant extrema without
   forming a continuous four-sided perimeter. Require accurate `Confident`
   corners within the existing 0.025 normalized tolerance.
3. Add `non_nested_weak_slab_fixture`, a deterministic generated PNG containing
   high-contrast internal content inside a weak, coherent, asymmetrically
   tapered slab. At the primary threshold the internal region is `C0`; at a
   fixed weak probe, `Ct` retains every `C0` pixel, materially increases
   quadrilateral area, and supports all four slab sides, while the chord polygon
   through `Qt` leaves at least one `Q0` corner outside. Validate those facts
   with temporary trace output before accepting the fixture; commit only the
   generated fixture function and behavior assertion, not trace output.
4. Add
   `auto_assist_abstains_when_coherent_weak_slab_extrema_are_not_quad_nested`.
   Require `Unavailable` with no corners. Preserve the existing sparse, filled,
   noisy-border, skew, reliable-card, flat-dark-card, and near-identity tests.
5. Establish the two-stage RED evidence. On unmodified head, the noisy accurate
   card must fail as `Unavailable`, while the sparse and non-nested slab
   negatives remain safely `Unavailable` under the broad `||`. Then recreate
   the prior containment-gated four-side decision only as an uncommitted
   intermediate: the noisy card and sparse perimeter must pass, while the new
   non-nested slab test must fail as `Confident`. If that second RED does not
   reproduce the observed containment bypass, stop before the revised GREEN
   implementation.

**RED/characterization verification:**

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema -- --nocapture` — RED on unmodified head.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content -- --nocapture` — green on head and the intermediate model.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_coherent_weak_slab_extrema_are_not_quad_nested -- --nocapture` — green on broad head behavior, RED as `Confident` on the containment-gated intermediate.

**Done:** Three sanitized morphologies bind both failed assumptions, the exact
containment-bypass RED is reproduced without private pixels, and a fresh
head/corpus baseline exists only outside Git.

### Task 2 — GREEN/REFACTOR: use component retention, not quadrilateral containment

**Files:** `controller/src/image_adjustments.rs`,
`controller/tests/image_adjustments.rs`.

**Actions:**

1. Preserve primary selection, the original anchor, and the descending
   relative/fixed threshold schedule. Extend `ContrastComponent` to retain its
   exact member indices while tracing. This is necessary because the scan-wide
   `visited` bitmap used by `largest_contrast_component` contains every traced
   foreground component and cannot represent `M0`.
2. For each weaker anchor trace, build/use exact `Mt` membership and assert that
   every primary member index is present. A retention failure makes stability
   fail closed. This `M0 ⊆ Mt` check is the only cross-threshold identity
   prerequisite.
3. Preserve the reviewed four-side support implementation: supercover sides,
   two-pixel `Mt` corridor, in-bounds outward non-`Mt` witness at offsets three
   through five, at least 70% support per side, no unsupported run above 20%,
   and all four sides coherent.
4. Run that support measurement whenever the weak quadrilateral area grows by
   more than the existing 0.05 material amount. Do not require `Qt` to contain
   `Q0`, do not require all `Q0` corners to be inside `Qt`, and do not replace
   containment with an image-specific centroid/corner exception. When retained
   membership, material area growth, and coherent four-side support are all
   present, reject the primary boundary as `Unavailable`.
5. Delete the `85db903` independent population/area `||`, the independent
   population veto, and the rolled-back strict-containment prerequisite. Keep
   area growth only as a support-analysis trigger; population may be retained
   only as non-decisive diagnostics or removed if unused.
6. Keep proposal types and ordering unchanged: projection/area validity,
   stability, near-identity `NotNeeded`, then four-corner `Confident`. Add no API
   field, dependency, alternate detector, UI flag, or private-image special case.
7. Make the three morphology tests green, then run the full detector suite. As a
   causal guard, temporarily reinsert strict quadrilateral containment and
   prove only the non-nested slab regression returns to `Confident`; restore the
   component-retention implementation and rerun green. If support constants or
   the 0.05 trigger need per-image adjustment, stop.

**GREEN verification:**

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_coherent_weak_slab_extrema_are_not_quad_nested -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`

**Done:** The accurate noisy card is accurate `Confident`; sparse, filled,
noisy-border, and non-nested coherent slabs are `Unavailable`; high-contrast and
near-identity classifications remain unchanged; and neither the old `||` nor
strict quadrilateral containment is in the stability decision.

### Task 3 — VALIDATE: prove the exact positive, failed slab, full corpus, and consumers

**Files:** the two implementation files; verification-only route/browser/test
files; read-only private inputs; temporary `/tmp` evaluator.

**Actions:**

1. Run the exact private WEBP. Require `Confident` with four finite corners near
   `(0.061,0.061)`, `(0.925,0.056)`, `(0.939,0.934)`, `(0.057,0.933)`, with at
   most 0.03 normalized error per coordinate.
2. Run `20240415_225421.jpg` before the rest of the corpus. It must be
   `Unavailable`; the recorded internal quadrilateral around
   `(0.186441,0.306729)` through `(0.188862,0.879499)` is explicitly forbidden.
   Stop immediately if it is `Confident`, even when paired synthetic and private
   positive controls pass.
3. Re-run all 116 sorted originals. Require zero known or screened
   internal-region confident proposals. Keep the other three named internal
   cases and two safe slab siblings non-confident unless a separately inspected
   result demonstrably follows the outer item boundary.
4. Produce a before/after table with all three status counts and every filename
   whose status or corners changed. Assess changes image by image; aggregate
   abstention/confidence counts cannot override geometry safety. Keep evaluator
   code/output outside Git; after authorization the final evidence belongs in
   the coder/reviewer PR audit-trail comment.
5. Run route/browser/static contracts to preserve camelCase serialization,
   authentication and terminal `mediaRevision`, stale session/draft/assist
   rejection, non-mutating `Unavailable`/`NotNeeded`, explicit Save, and manual
   fallback. Verification-only files must remain unchanged.
6. Run full adjacent Rust checks. Delete the temporary evaluator and verify no
   private input, absolute private executable-test path, evaluator, per-image
   output, or generated derivative appears in Git.

**Verification commands:**

- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow admin_image_adjustment_routes_save_reset_review_and_assist -- --nocapture`
- `node controller/tests/static_admin_behavior.mjs`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin -- --nocapture`
- `node --check controller/static-admin/admin.js`
- `cargo test --manifest-path controller/Cargo.toml --all-features`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `git diff --check -- controller/src/image_adjustments.rs controller/tests/image_adjustments.rs`
- `git diff --exit-code -- controller/src/routes.rs controller/static-admin/admin.js controller/tests/admin_workflow.rs controller/tests/static_admin_behavior.mjs controller/tests/static_admin.rs`

**Done:** The exact diagnostic is accurately `Confident`;
`20240415_225421.jpg` and every known/screened internal-region case are not
Confident; complete count/corner changes are assessed; all consumer/manual
fallback gates pass; and no private/evaluator artifact enters the repository.

## Acceptance criteria

- The three sanitized fixture contract is green: coherent sparse outer
  perimeter => `Unavailable`; noise-connected accurate dark card => accurate
  `Confident`; coherent larger slab whose extrema quadrilateral is not nested =>
  `Unavailable`.
- The exact supplied dark-card diagnostic returns accurate `Confident` corners
  within the stated coordinate tolerance.
- `20240415_225421.jpg` is `Unavailable` and never returns the recorded internal
  quadrilateral from the rolled-back attempt.
- All 116 held-out images are evaluated; no known or screened internal-region
  proposal remains Confident; confidence-count changes are enumerated and
  assessed rather than treated as a score.
- High-contrast skew/card, reliable outer card, filled/noisy enclosures,
  low-confidence, and inset-2/inset-5 `NotNeeded`/`Confident` behavior retain
  their current test contracts.
- The assist route preserves authentication, terminal `mediaRevision` recheck,
  and existing camelCase `status`/`corners`/`message` serialization.
- The browser preserves session/draft/assist authority, applies only a current
  four-corner `confident` response, leaves `Unavailable` and unexpected results
  non-mutating, and retains manual corner fallback per D-08-26.
- Private originals remain unchanged and drafts require explicit Save per
  D-08-25 and D-08-29.
- `85db903`'s independent `population_growth || area_growth` behavior is
  replaced, not supplemented by another scalar/boolean point gate; `M0 ⊆ Mt`
  replaces strict `Qt`-contains-`Q0` as the cross-threshold identity contract.
- Only `controller/src/image_adjustments.rs` and
  `controller/tests/image_adjustments.rs` differ from the pre-execution source
  scope, and no private image, absolute private test path, evaluator, binary,
  per-image output, or generated derivative is committed.

## Rollback and stop conditions

Stop and return to `reassessment_required` without another point-fix round if
any condition below occurs:

- the sanitized noisy accurate fixture is not RED on `85db903`, or the
  non-nested slab fixture does not reproduce a RED false confidence under the
  containment-gated intermediate;
- the sparse, filled, or noisy-border enclosure becomes `Confident`;
- the sanitized non-nested slab or `20240415_225421.jpg` is `Confident`;
- the exact diagnostic is not accurate `Confident`;
- any known or screened internal-region corpus proposal is `Confident`;
- a support constant needs image-specific tuning or another population/area
  boolean is proposed around the coherent model;
- strict quadrilateral containment, a special case for the named corpus image,
  or coordinate-specific logic appears in the decision path;
- any primary member is absent from a weaker anchor-connected component;
- route/UI changes appear necessary to make detector tests pass;
- API serialization, session/revision authority, non-mutating fallback, manual
  controls, privacy, formatting, production-persistence, or Clippy gates fail;
- corpus count changes cannot be explained image by image; or
- private/evaluator artifacts appear in Git status or the proposed diff.

When a stop condition fires, restore only the two in-scope production/test
edits to the last reviewed state, preserve unrelated worktree changes, retain
the failing evidence outside Git, and revise the assumptions/plan before any
new coder round.

## Surgical source coverage audit

| Source item | Coverage | Task |
| --- | --- | --- |
| ROADMAP Phase 8 criterion 5; MEDIA-06; ADMIN-06 | Bounded assisted perspective correction remains useful and manual fallback remains authoritative. | 1-3 |
| MEDIA-05/private boundary | No private asset or storage identifier enters committed tests/output; authenticated same-origin route stays unchanged. | 3 |
| D-08-25 | Detector and evaluation never mutate private originals. | 3 |
| D-08-26 | Accurate positive controls remain assisted; ambiguous enclosures abstain; manual handles remain available. | 1-3 |
| D-08-29 | Assist changes only a current unsaved draft; persistence still requires Save. | 3 |
| RESEARCH auto-assist constraint | Deterministic bounded Rust heuristic, no OCR/AI/OpenCV, with high-contrast confident and ambiguous unavailable fixtures. | 1-2 |
| REVIEW shared invariants/failure matrix | All four invariants and every detector/API/UI outcome are bound by tests and held-out evaluation. | 1-3 |
| Round-3 revised requirement: coherent support, not scalar tweak | Four-side support replaces the `||`; exact component retention replaces extrema-polygon containment. | 2 |
| First paired sanitized fixtures | Sparse perimeter negative and noise-connected accurate card positive remain binding. | 1-2 |
| Second reassessment morphology | A coherent larger slab with non-nested extrema quadrilateral is explicitly `Unavailable`. | 1-2 |
| Exact diagnostic, failed slab, and 116-image corpus | Accurate diagnostic, `20240415_225421.jpg` unavailable, zero known/screened internal-region confidence, count-change assessment. | 1, 3 |
| Deferred AI/advanced enhancement | Excluded; no OCR/AI, content-aware correction, batch correction, or broad admin redesign. | Scope fence |

No source item is intentionally deferred within this surgical convergence
scope. Phase-wide OPS/CDN/media-publication work is already owned elsewhere and
is not a gap in this PR #276 detector plan.

## Pre-mortem guardrails

| Likely failure | Early detector | Mitigation already in plan |
| --- | --- | --- |
| The sanitized positive does not reproduce the real regression. | It is green on unmodified `85db903` or fails for a different reason. | Task 1 halts before production changes and requires RED plus morphology evidence. |
| Extrema-polygon containment silently bypasses a real weak boundary again. | The non-nested slab stays `Confident` while the original pair passes. | Task 1 reproduces that exact intermediate RED; Task 2 uses `M0 ⊆ Mt`, bans containment, and mutation-checks the new regression. |
| The support model merely trades false abstention for renewed false confidence. | Exact diagnostic passes but the named slab/corpus screen fails, or counts improve without geometry safety. | Three morphology tests, exact diagnostic, named slab, and full 116-image before/after screen are jointly binding. |
| Consumer safety regresses despite a detector-only edit. | Route/browser/authority suites fail or verification-only files change. | Task 3 runs focused consumer tests and requires zero diff in those files. |

