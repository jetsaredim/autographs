---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed: 2026-10-14T10:23:36Z
reviewed_head: ec7a6bd6796851329480845d8ff903ac1be5f184
depth: deep
review_round: final-status
files_reviewed: 9
files_reviewed_list:
  - controller/src/image_adjustments.rs
  - controller/src/routes.rs
  - controller/static-admin/index.html
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/tests/image_adjustments.rs
  - controller/tests/admin_workflow.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
convergence:
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
  backlog_issue: https://github.com/jetsaredim/autographs/issues/277
---

# Phase 08: PR #276 Final Deep Code Review

**Reviewed:** 2026-10-14T10:23:36Z
**Reviewed commit:** `ec7a6bd6796851329480845d8ff903ac1be5f184`
**PR base:** `origin/main`
**Depth:** deep
**Status:** issues found — one Warning; one accepted Info-level limitation

## Summary

The detector, route, and browser authority boundaries are coherent at the
current head. The gradient/Hough search is bounded, its candidate corners are
veto-only, the route rechecks `mediaRevision`, and auto-detect changes only the
current unsaved draft. Saving and publishing remain separate explicit actions.
The review-state reset, rule-of-thirds grid, projected edge segments, current
corner focus, stale-response guards, and no-op detection outcome are covered by
the deterministic DOM harness and focused integration tests.

The test rename at `ec7a6bd` is truthful: the committed fixture and test now
describe a filled slab, and neither claims to cover the deferred one-pixel
sparse outline. The thin-outline behavior is accurately preserved as an
accepted Info-level limitation in issue #277 and in the superseding disposition
at the top of the detector assumptions and revised plan.

One actionable UI defect remains. The global CSS applies disabled styling to
every checkbox and range input through `input:read-only`, so enabled overlay
checkboxes still display a `not-allowed` cursor. This directly reproduces the
misleading cursor reported during image-review UAT and is not covered by the
current static overlay tests.

## Narrative Findings (AI reviewer)

## Warnings

### WR-02: Enabled overlay checkboxes receive the disabled cursor

**File:** `controller/static-admin/admin.css:72-75`
**Finding family:** `overlay_control_affordance`
**Lineage:** `independent`
**Shared invariant:** An enabled interactive review control must present an
interactive affordance; disabled/read-only styling must apply only to controls
that are actually unavailable.
**Consumers inspected:** overlay checkbox markup in
`controller/static-admin/index.html`; `syncReviewDirtyState` enable/disable
ownership; overlay `change` handlers and state synchronization in `admin.js`;
the CSS overlay and form-control rules; static source checks and the DOM behavior
harness.
**Evidence level:** `source_only`

**Issue:** The selector `input:read-only` matches inputs for which the HTML
`readonly` attribute is not applicable, including `checkbox` and `range`.
Consequently the Grid, Centerline, and Edges checkboxes match this rule even
when enabled and clickable, receiving `cursor: not-allowed` and muted text
color. This explains the prohibited cursor observed during UAT while the
checkboxes still toggle normally. The same rule also affects enabled review
sliders.

**Fix:** Target the actual attribute instead of the broad pseudo-class and bind
the behavior in the static CSS contract:

```css
button:disabled,
input[readonly] {
  color: #626862;
  cursor: not-allowed;
}
```

Add a focused assertion that the stylesheet uses `input[readonly]` and does
not reintroduce `input:read-only` for this disabled affordance.

## Info

### IN-01: Thin sparse enclosure remains an explicitly accepted limitation

**File:** `controller/src/image_adjustments.rs:22-23`
**Issue:** A one-pixel, 14-luma-level enclosing outline can remain below the
frozen Canny threshold and allow the internal primary to be reported as
`Confident`. This is not an actionable PR finding under the operator's explicit
Phase 8 scope decision: auto-detect is advisory, applies only to the unsaved
draft, requires explicit Save, retains the manual four-corner editor, and the
morphology was absent from the 116-image corpus. The exact reproduction and
future acceptance gates remain open in GitHub issue #277.
**Fix:** Deferred to #277; do not tune a detector threshold in this PR.

## Invariant and failure-path audit

| Boundary | Evidence | Assessment |
| --- | --- | --- |
| Detector producer | Adaptive border/component primary plus bounded Canny/Hough ambiguity veto | Gradient corners never become output; 12 lines per axis cap pairing at 4,356 candidates. |
| API authority | Authenticated assist route, private media read, initial and terminal `mediaRevision` checks | Stale or replaced media cannot authoritatively return a proposal. |
| Browser authority | Session, draft, assist, mutation, and drag revisions | Only a current four-corner `confident` response changes the unsaved draft. |
| Persistence/publish | Separate Save and publish actions with dirty-state gates | Detect alone neither persists nor publishes. |
| Cross-item state | `beginReviewSession`, `clearImageReviewState`, per-open overlay defaults, control synchronization | Grid resets off and Edges resets on for each image; stale async responses are rejected. |
| Overlay geometry | Source-image object-fit bounds, four rule-of-thirds lines, current-corner edge segments | Overlay geometry is source-space aligned and updates during manual corner movement. |
| Failure/recovery | Abort/revision checks, preview retry, media-conflict exclusive recovery | Late completions and conflict refreshes do not mutate a newer review authority. |
| Accepted detector gap | Exact reproduction and acceptance gates in #277 | Accurately documented and deferred, not claimed as fixed by the renamed filled-slab test. |

## Verification

- `cargo test --manifest-path controller/Cargo.toml --all-features` — passed:
  253 tests passed; 2 credentialed live tests ignored by contract.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments --test static_admin -- --nocapture`
  — passed: 17 detector tests and 19 static-admin tests.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `node --check controller/static-admin/admin.js` — passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
  — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`
  — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- Changed-source `git diff --check` — passed.
- PR #276 required checks — all nine reported checks passed at `ec7a6bd`.
- PR review threads — all three are resolved; #277 remains open as accepted
  backlog scope.

## Changed Paths

- `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`

No production or test source was modified by this review.

---

_Reviewed: 2026-10-14T10:23:36Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep_
