---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-12T23:50:25Z
depth: deep
files_reviewed: 4
files_reviewed_list:
  - controller/src/image_adjustments.rs
  - controller/tests/image_adjustments.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.js
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
convergence:
  status: ready_to_resume
  recurring_critical_warning: 1
  scope_expansion: false
  requires_reassessment: true
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
  checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
  checker_status: passed_cycle_2
  independent_review_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
  reassessment_iteration: 3
  design_spike_evidence:
    - .planning/spikes/008-detector-mask-contour-diagnostics/README.md
    - .planning/spikes/009-a-actual-contour-boundary/README.md
    - .planning/spikes/009-b-local-gradient-rectangle/README.md
  pr_evidence: https://github.com/jetsaredim/autographs/pull/276#issuecomment-6156047878
---

# Phase 08: PR #276 Deep Code Review — Round 3 Convergence Checkpoint

**Reviewed:** 2026-10-12T23:50:25Z
**Reviewed commit:** `85db903d4dd0ef70887816c9849101fac2827c7a`
**Incremental base:** `36946e74357d49f764df0de46a79f8b423c6a3ec`
**PR base:** `origin/main` (`60eb329df15ca76106298325cd02bed523ac3ad3`)
**Depth:** deep
**Files Reviewed:** 4
**Status:** issues found
**Convergence:** reassessment required

## Summary

The `&&` to `||` change closes round-2 WR-01 for the sparse synthetic
enclosure, and that fixture faithfully exercises the reported low-population,
large-area branch. However, the broader independent area tripwire regresses the
held-out accurate dark-card control that the detector hardening was explicitly
required to preserve. The exact supplied diagnostic now returns `Unavailable`
at `85db903`, whereas the prior detector returned an accurate material
`Confident` quadrilateral. This is a recurring Warning at the mandatory third
review checkpoint, so another point-fix round is not authorized.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: Independent area growth rejects the known accurate dark-card control

**File:** `controller/src/image_adjustments.rs:465-466`
**Finding family:** `auto_assist_outer_boundary_confidence`
**Lineage:** `fix_regression` (continuation of round-2 WR-01)
**Shared invariant:** Auto-assist must abstain when weaker evidence reveals a different enclosing item boundary while retaining material confident corners for the known accurate dark-card control; low-threshold background noise alone must not invalidate an otherwise stable outer boundary.
**Consumers inspected:** `propose_image_adjustment`; adaptive/relative/fixed threshold production; anchor-connected component tracing; route serialization and terminal media-revision check; browser `confident`/`unavailable` handling and draft mutation; sparse, filled-enclosure, noisy-border, dark-card, high-contrast, and near-identity tests.
**Evidence level:** `deterministic_test`

**Issue:** The `||` at lines 465-466 correctly makes material quadrilateral
growth sufficient to reject the new sparse enclosure, but it also makes weak
background/noise extrema sufficient to reject a correct outer-card component.
Running the current compiled detector on the exact local diagnostic
`b9912131-8781-4047-a44c-03e5ca1e56a9.webp` outside the repository
returns `Unavailable` with no corners. The prior held-out evidence recorded
this 914x1280 dark card as accurate and `Confident` through threshold 8, with
only 24.91% population growth; preserving that control was part of the stated
acceptance boundary. The committed `dark_card_fixture` at
`controller/tests/image_adjustments.rs:168-181` is a flat synthetic card and
does not reproduce the diagnostic's low-threshold background connectivity, so
it stays green while the real control regresses.

The sparse regression itself is faithful: its one-pixel weak outer rectangle
and bridge add well under 25% population while materially expanding the
anchor-connected quadrilateral (`controller/tests/image_adjustments.rs:210-217`,
`controller/tests/image_adjustments.rs:388-413`). It proves the old conjunction
was insufficient, but it does not prove that every material weak-mask area
expansion represents an enclosing object rather than noise.

**Fix:** Stop point-fixing the boolean or thresholds. Revise the detector
assumptions and implementation plan around a discriminating boundary-support
signal that separates a coherent sparse enclosing perimeter from isolated
weak-mask/background extrema. The revised plan must bind both the sparse
enclosure (`Unavailable`) and a sanitized analogue of the accurate noisy
dark-card diagnostic (`Confident` with accurate corners), then re-run the
held-out diagnostic and corpus before implementation resumes.

## Convergence Reassessment

### Finding lineage

| Round | Head | Lineage | Result |
| --- | --- | --- | --- |
| 1 | `b2d947c` | Evidence gap | Standard review was clean, but the 116-image corpus later exposed systematic internal-region false confidence. |
| 2 | `36946e7` | `incomplete_fix` | WR-01 showed that the population-and-area conjunction still trusted sparse enclosing boundaries. |
| 3 | `85db903` | `fix_regression` | `||` closes the sparse branch but makes the known accurate dark-card diagnostic `Unavailable`. |

This is the mandatory third-review checkpoint. The recurring Warning keeps the
family in `reassessment_required`; no additional local boolean/threshold tweak
should proceed without revised assumptions and a reviewed plan.

### Shared invariants

1. A `Confident` proposal represents the stable outer item boundary, not a
   higher-contrast internal region.
2. Weak-mask evidence must distinguish coherent enclosing geometry from
   background/noise connectivity; either may expand component population or
   extrema.
3. Safe abstention is preferred for ambiguity, but known accurate material
   cases remain usable; the supplied dark-card diagnostic is an explicit
   positive control.
4. `Unavailable` and unexpected outcomes never mutate the browser draft;
   only a current four-corner `Confident` response may do so.

### Complete consumer inventory

| Boundary | Inspected evidence | Assessment |
| --- | --- | --- |
| Background/threshold producer | Border median, adaptive minimum 32, relative probes, fixed 32/16/8 (`image_adjustments.rs:271-321`, `435-448`) | Probe ordering is strictly descending after deduplication. When a later fixed probe is skipped, a weaker mask was already evaluated. |
| Primary component producer | Largest 8-connected contrast component, row-major anchor, corner extrema (`image_adjustments.rs:323-423`) | Anchor is guaranteed to remain foreground as thresholds weaken and correctly tracks mergers of the original candidate. |
| Stability decision | Independent population or area rejection (`image_adjustments.rs:426-471`) | Sparse false confidence is closed, but noise-driven area growth now causes the demonstrated false abstention. |
| Proposal classification | Projection/area validation, stability, near-identity classification (`image_adjustments.rs:229-269`) | Ordering remains fail-closed and internally consistent. |
| API reporting | Authenticated assist route, source read, terminal revision recheck, camelCase serialization (`routes.rs:1131-1188`, `routes.rs:2425-2432`) | Safe; it does not manufacture or retain corners for `Unavailable`. |
| Browser action/mutation | Session/draft/assist revision guards and four-corner confident gate (`admin.js:2822-2856`) | Safe; `Unavailable`, `notNeeded`, stale, error, and unexpected responses do not mutate the draft. |
| Save/publish boundary | Assist only changes the unsaved draft; normal preview and explicit save gates still apply (`admin.js:2892-2960`) | No automatic persistence or publication from detector output. |
| Regression tests | 15 image-adjustment tests plus route and static-admin behavior suites | Sparse/filled negative cases and flat positive cases pass; the held-out noisy dark-card positive control is not represented. |

### Assumption audit

| Assumption | Evidence | Verdict |
| --- | --- | --- |
| Material weak-mask area expansion always indicates a different outer item boundary. | Sparse fixture supports it, but the accurate dark-card diagnostic now abstains after area becomes an independent tripwire. | Rejected. |
| The flat `dark_card_fixture` represents the positive dark-card class sufficiently. | It has a uniform card and background, while the held-out diagnostic gains low-threshold background/noise connectivity. | Rejected. |
| Population growth and area growth can be classified independently with global scalar thresholds. | `&&` misses sparse enclosures; `||` rejects the known accurate noisy control. | Unproven and currently contradicted by the paired controls. |
| Threshold ordering or anchor discontinuity caused the regression. | Thresholds are monotonically weakened/deduplicated, and the original foreground anchor is retained. | Eliminated. |
| False abstention is harmless under the safe-abstention policy. | Manual fallback is safe, but the task explicitly requires retaining the known accurate dark-card behavior and the fix report claims it is preserved. | Rejected as an acceptance argument. |

### Failure matrix

| Input/outcome | Required result | Head `85db903` |
| --- | --- | --- |
| Filled low-contrast enclosure around dark content | `Unavailable` | Pass. |
| Noisy-border filled enclosure | `Unavailable` | Pass. |
| Sparse connected weak outer enclosure | `Unavailable` | Pass; WR-01 negative regression is faithful. |
| High-contrast skewed item | Accurate `Confident` | Pass in synthetic test. |
| Reliable high-contrast outer card | Accurate `Confident` | Pass in synthetic test. |
| Flat synthetic dark card | Accurate `Confident` | Pass, but does not bind the real noise path. |
| Supplied accurate dark-card diagnostic | Accurate `Confident` | **Fail: `Unavailable`, no corners.** |
| Near-identity inset 2 / inset 5 | `notNeeded` / `Confident` | Pass in tests. |
| `Unavailable` API/UI consumption | No corners applied; manual adjustment remains available | Pass by source and behavior tests. |

### Revised plan requirements

1. Write a revised detector assumption describing what evidence establishes a
   coherent outer boundary across thresholds, including sparse transparent
   enclosures and noisy/background-connected accurate cards.
2. Replace the paired global-scalar boolean choice with a coherent boundary
   model or support criterion. The plan must explain why it distinguishes the
   two held-out morphologies before code changes begin.
3. Add two paired, sanitized derived-oracle fixtures: the current sparse
   enclosure must abstain, and a fixture reproducing the supplied diagnostic's
   low-threshold noise/extrema behavior must remain accurately confident.
4. Preserve high-contrast skew/card and near-identity classifications, API
   serialization, session/revision authority, non-mutating `Unavailable`
   behavior, and manual fallback.
5. Re-run the exact supplied dark-card diagnostic and all 116 corpus images.
   Record confidence/unavailability counts plus a false-confidence geometry
   screen; counts alone are not acceptance evidence.
6. Obtain review of both the revised assumptions and revised implementation
   plan before resuming coder work. Record those paths in this artifact when
   they have actually been reviewed; a status-only change is invalid.

### Explicit resume criteria

Coder work may resume only when all of the following are true:

- durable revised-assumption evidence and revised-plan evidence exist, have
  been independently reviewed, and are referenced in frontmatter;
- the plan binds both paired derived-oracle fixtures and explains the
  discriminating signal;
- the exact supplied diagnostic is required to return accurate `Confident`
  corners and the sparse enclosure is required to return `Unavailable`;
- held-out corpus evaluation requires zero known/screened internal-region
  confident proposals, with any confidence-count change explicitly assessed;
- existing image-adjustment, route, browser behavior, formatting,
  production-persistence, and Clippy gates remain required.

Until then, convergence status remains `reassessment_required`.

## Superseded Resume Authorization — 2026-10-13

The iteration-2 revised assumptions were recorded in
`08-PR-276-DETECTOR-ASSUMPTIONS.md`, and its implementation-ready plan formerly
occupied `08-PR-276-DETECTOR-REVISED-PLAN.md`; that path now contains the
iteration-3 design-spike gate. Independent review in
`08-PR-276-DETECTOR-PLAN-REVIEW.md` passed with zero Blocker, Warning, or Info
findings. That authorization was valid for the reviewed iteration, but it is now
superseded by the second implementation attempt's explicit stop condition. The
review file remains lineage evidence and does not authorize a third coder round.

## Second Implementation Stop — 2026-10-13

The authorized `M0 ⊆ Mt` implementation kept the reviewed four-side
coherence/witness logic. It passed all three sanitized morphologies, including
the containment-bypass slab, passed all 17 detector tests, and returned target
`Confident` corners for the exact private
`b9912131-8781-4047-a44c-03e5ca1e56a9.webp` positive. The exact held-out
`20240415_225421.jpg` nevertheless returned the forbidden internal
`Confident` quadrilateral near `(0.186441,0.306729)`,
`(0.825666,0.286385)`, `(0.738499,0.870110)`, and
`(0.188862,0.879499)`.

This is an `incomplete_fix` / `assumption_failure` / `test_weakness`, not a new
independent finding. Removing quadrilateral containment was insufficient; the
four-side extrema-chord support abstraction itself does not capture the real
slab. Two follow-on diagnostic agents were terminated after becoming
unresponsive and produced no trustworthy per-side trace. The attempted source
changes were rolled back without a commit, leaving the worktree source-clean.

Convergence is therefore `reassessment_required`. Before any third coder round,
a bounded detector-design spike must inspect the actual threshold masks,
anchor-connected component masks, component contours/boundary runs, failed-model
per-side diagnostics, and local-gradient evidence on both the exact positive
and named slab. Actual-contour and local-gradient rectangular-boundary models
are recommended investigation directions, not approved algorithms. A
privacy-safe spike report, diagnostic-derived sanitized RED, revised coherent
contract, new implementation plan, and fresh independent review are all required
before status may return to `ready_to_resume`.

## Completed Detector Design Spike — 2026-10-13

The required bounded investigation is now complete and recorded in spikes 008,
009a, and 009b plus PR comment `6156047878`.

- Spike 008 reproduced the exact controls and showed that the named slab's
  border-median component floods the frame before a usable physical outer
  contour appears. Local Canny/Hough edge evidence remains visible.
- Spike 009a invalidated contour-only classification: ignoring flood trusts the
  internal slab primary, while abstaining on every flood also rejects the exact
  accurate positive.
- Spike 009b froze its constants before exact-control/corpus evaluation and
  validated a bounded local-gradient/Hough rectangle as a conservative veto on
  the existing primary. It does not promote gradient corners.
- The exact positive retained accurate primary corners; the exact named slab
  exposed a materially larger candidate with four valid localized sides and
  became `Unavailable`.
- Across all 116 originals with unchanged constants, 79 primaries were retained,
  37 cases safely abstained, and none returned `NotNeeded`. Visual inspection
  found zero internal retained proposals. `20251231_011140` was the sole repeat
  of the prior 12%-side-distance screen and was confirmed accurate. All three
  slab siblings abstained.
- The two-control optimized smoke measured `0.21s` and about `8.2 MiB` peak RSS.
  The existing `imageproc 0.27` dependency supplies the required primitives.

The iteration-3 assumptions and plan now replace the current
`population_growth || area_growth` decision with that complete frozen veto,
preserve primary selection/corners, require three mechanism-derived sanitized
contracts and a causal bypass mutation, and bind exact/corpus/performance,
consumer, no-diff, and privacy gates. This design evidence establishes
feasibility only. Convergence remains `reassessment_required` until a fresh
independent checker approves both updated artifacts and its evidence path is
recorded; the iteration-2 plan review cannot authorize this algorithm.

## Iteration-3 Checker Block Revision — 2026-10-13

The fresh iteration-3 checker review in
`08-PR-276-DETECTOR-PLAN-REVIEW.md` returned two Blockers while confirming that
all other coverage was complete. The assumptions and revised plan now address
those Blockers without changing the spike-validated detector behavior:

1. Candidate intersection bounds now use one exact coordinate contract.
   imageproc's continuous geometric image boundary is
   `[0,width] × [0,height]`, while pixel centers end at `width - 1` and
   `height - 1`. The spike's inclusive admission interval is exactly
   `x >= -4`, `y >= -4`, `x <= width + 4`, `y <= height + 4`—four coordinate
   units beyond the geometric boundary, not four units beyond the final pixel
   center. A private-helper unit test binds both inclusive endpoints and rejects
   values immediately outside them.
2. The prepared-thumbnail spike smoke is now advisory only. Before source
   changes, a temporary release evaluator is built once against head, warmed,
   then its already-built binary directly measures public
   `propose_image_adjustment` calls over a frozen 118-call manifest: exact
   positive, exact slab, and the 116 sorted originals. After implementation it
   is rebuilt once, warmed once, and measured identically. Both sides capture
   per-image, total, p50, p95, max, and max RSS without Cargo/build/cache time.
   Mandatory gates are p95 `<= 1.0s` and `<= 3×` baseline, max `<= 2.0s` and
   `<= 3×` baseline, total `<= 3×` baseline, and RSS
   `<= max(1.5× baseline, baseline + 32 MiB)`.

The checker verdict itself remains unchanged and authoritative for the reviewed
version. Status stays `reassessment_required` and checker linkage is
`blocked_pending_re_review` until a focused re-review passes the revised
contracts.

## Iteration-3 Cycle-2 Resume Authorization — 2026-10-13

Focused independent re-review in
`08-PR-276-DETECTOR-PLAN-REVIEW.md` verified that both iteration-3 Blockers are
fully resolved and found zero additional Blocker, Warning, or Info issues.

- Candidate bounds now bind the exact inclusive spike inequalities
  `[-4,width+4] × [-4,height+4]` under the stated continuous geometric image
  boundary, with a private endpoint unit test.
- Performance now compares warmed, already-built binaries over the identical
  118-call public production path before and after, excludes Cargo/build/cache
  time, and applies explicit absolute/relative latency, total, and RSS gates.
- The checker revalidated scalar-veto replacement, veto-only gradient corners,
  bounded 4,356-candidate search, frozen constants, causal sanitized fixtures,
  exact/corpus/slab gates, consumer/no-diff/privacy boundaries, full Cargo
  checks, and stop/rollback behavior.

The current assumptions, revised plan, and fresh independent-review evidence
paths are recorded in convergence frontmatter. Convergence is therefore
`ready_to_resume`. Execution remains bound by every task, acceptance criterion,
and stop condition in the reviewed iteration-3 plan; any stop condition returns
the family to `reassessment_required` before another coder round.

## Verification

- `cargo test --test image_adjustments --quiet` — passed, 15 tests.
- `cargo test --test admin_workflow admin_image_adjustment_routes_save_reset_review_and_assist --quiet` — passed, 1 focused route test.
- `node tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo check --features production-persistence --quiet` — passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `git diff --check 36946e7..85db903 -- src/image_adjustments.rs tests/image_adjustments.rs` — passed.
- Current compiled detector against the exact supplied dark-card diagnostic — returned `Unavailable` with empty corners; expected accurate `Confident`.

---

_Reviewed: 2026-10-12T23:50:25Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep, round 3 convergence checkpoint_
