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
  requires_reassessment: false
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
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
`/home/jgreenwa/Downloads/b9912131-8781-4047-a44c-03e5ca1e56a9.webp`
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

## Resume Authorization — 2026-10-13

The revised assumptions are recorded in
`08-PR-276-DETECTOR-ASSUMPTIONS.md`, and the implementation-ready plan is
recorded in `08-PR-276-DETECTOR-REVISED-PLAN.md`. Independent review in
`08-PR-276-DETECTOR-PLAN-REVIEW.md` passed with zero Blocker, Warning, or Info
findings. The required evidence paths are now present in frontmatter, so the
convergence status is `ready_to_resume`. Execution remains bound by the paired
RED/GREEN morphology tests, exact private diagnostic, 116-image held-out
screen, consumer verification, privacy fence, and stop conditions in the
reviewed plan.

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
