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
independent_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-PLAN-REVIEW.md
---

# PR #276 Detector Convergence Revised Plan

## Objective

Replace the `85db903` scalar population/area `||` decision with one coherent
outer-boundary support decision that rejects a supported sparse or filled
enclosure while retaining an accurate card whose weak-threshold component is
connected only to noise-driven extrema.

This is not a Phase 8 replan. The only production behavior in scope is the
private Rust detector. The existing route, API shape, browser authority,
draft-local save model, manual fallback, and publishing boundaries remain
unchanged per D-08-25, D-08-26, and D-08-29.

## Mandatory execution gate

Independent review of this plan and
`08-PR-276-DETECTOR-ASSUMPTIONS.md` passed with no findings in
`08-PR-276-DETECTOR-PLAN-REVIEW.md`. The evidence paths are recorded in both
artifacts and the canonical review, so the third-review checkpoint is
`ready_to_resume`.

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

### Task 1 — RED: bind the paired morphologies and capture the head baseline

**Files:** `controller/tests/image_adjustments.rs`; read-only private inputs;
temporary evaluator under `/tmp`.

**Preconditions:** HEAD is `85db903d4dd0ef70887816c9849101fac2827c7a` or
the production detector is byte-equivalent; the exact diagnostic exists; the
corpus directory contains exactly 116 JPEGs; both planning artifacts have been
independently approved.

**Actions:**

1. Before changing production code, use an external temporary Cargo program
   with a path dependency on `controller/` to evaluate the exact diagnostic and
   the sorted 116-image corpus. Record only outside Git:
   `confident`/`unavailable`/`notNeeded` counts, every Confident filename and
   corners, the four known internal-region cases
   `20240415_225421.jpg`, `20251109_164543.jpg`,
   `20251231_002432.jpg`, and `20260317_232446.jpg`, and the conservative
   screen of any Confident proposal that fails to approach every image side
   within 12%. This is the measured `85db903` baseline; do not substitute the
   earlier 19/97/0 evidence from a different detector state.
2. Add `dark_card_with_connected_weak_noise_fixture`, a fully generated,
   non-private 180x140 PNG. Use the existing light-background/skewed dark-card
   geometry, then add low-contrast one-pixel tendrils and deterministic sparse
   branches from that card to distant extrema. The weak pixels must be absent
   from the primary threshold, present at a fixed weak probe, connected to the
   card, and too sparse to form any continuous four-sided outer perimeter.
   This fixture represents the supplied diagnostic's low-threshold
   noise/background connectivity without copying any private pixel.
3. Add
   `auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema`.
   It must require `Confident`, exactly four corners, and the original dark-card
   corners within the existing 0.025 normalized tolerance.
4. Keep
   `auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content` as the
   paired negative oracle: `Unavailable` with no corners. Do not weaken or
   replace the existing filled/noisy enclosure, skew, reliable-card,
   flat-dark-card, or near-identity assertions.
5. Run the new positive test against unmodified `85db903` and capture the
   expected RED result: current `||` behavior returns `Unavailable`. Run the
   sparse test separately and capture that it remains green. If the new fixture
   does not fail for the demonstrated reason—material extrema growth without a
   coherent enclosing perimeter—stop and revise the fixture/assumption evidence
   before touching production.

**RED verification:**

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema -- --nocapture` — must fail on unmodified `85db903` as `Unavailable` versus `Confident`.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content -- --nocapture` — must pass on unmodified `85db903`.

**Done:** The paired sanitized oracles discriminate the same branch: the sparse
enclosing perimeter is negative, the noise-connected accurate card is
positive, and a measured private/corpus head baseline exists outside Git.

### Task 2 — GREEN/REFACTOR: replace the `85db903` boolean with coherent perimeter support

**Files:** `controller/src/image_adjustments.rs`,
`controller/tests/image_adjustments.rs`.

**Actions:**

1. Preserve primary component selection, the original anchor, and the existing
   descending relative/fixed threshold schedule. Reuse the fresh `visited`
   bitmap from each anchor trace as membership for that weak component.
2. Implement the coherent outer-boundary support model exactly as specified in
   `08-PR-276-DETECTOR-ASSUMPTIONS.md`: material enclosure check; four
   supercover-rasterized sides; two-pixel component corridor; in-bounds
   outward-normal background witness at offsets three through five; per-side
   coverage of at least 70%; longest unsupported run at most 20%; all four sides
   required.
3. Replace lines 465-466's population-growth-or-area-growth rejection. Delete
   that `||` decision rather than leaving it in front of or behind the new
   support check. Remove `AUTO_CORRECTION_MAX_STABLE_COMPONENT_GROWTH` and the
   independent population veto. Rename the area-growth constant so it expresses
   its remaining role as the material-enclosure trigger. Population may be
   retained only as non-decisive local diagnostic data; remove it if unused.
4. A weak probe now invalidates the primary boundary only when it supplies a
   materially larger, coherently supported enclosure. Material extrema growth
   without coherent side support is treated as noise/background connectivity
   and does not reject the original accurate boundary. Anchor disappearance,
   invalid geometry, or an actual supported enclosure remains fail-closed.
5. Keep the proposal/public types and status ordering unchanged: projection and
   area validity, stability, near-identity `NotNeeded`, then four-corner
   `Confident`. Add no API fields, dependency, alternate detector, or UI flag.
6. Run the paired tests first. Then run the entire image-adjustment suite. If a
   support constant must move to make only one private image pass, stop; the
   paired structural model and corpus, not per-image tuning, control the change.

**GREEN verification:**

- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_keeps_accurate_dark_card_when_weak_noise_connects_to_extrema -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments auto_assist_abstains_when_sparse_weak_boundary_encloses_dark_content -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture`
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`

**Done:** The noisy accurate fixture is accurate `Confident`; sparse, filled,
and noisy-border enclosures are `Unavailable`; skew/reliable card and
near-identity cases retain their exact classifications; and the old independent
`||` behavior no longer exists in the decision path.

### Task 3 — VALIDATE: prove private positive control, held-out safety, and unchanged consumers

**Files:** production/test files above; verification-only route/browser/test
files; read-only private inputs; temporary `/tmp` evaluator.

**Actions:**

1. Re-run the same temporary evaluator on the exact supplied WEBP. Require
   `Confident` with four finite normalized corners accurate to the established
   diagnostic oracle, approximately `(0.061,0.061)`, `(0.925,0.056)`,
   `(0.939,0.934)`, `(0.057,0.933)`, with at most 0.03 normalized error per
   coordinate. `Unavailable`, `NotNeeded`, empty corners, or a materially
   different quadrilateral fails acceptance.
2. Re-run all 116 sorted corpus images. Require zero Confident results for the
   four known internal-region cases unless direct visual review proves the new
   corners follow the outer item boundary, and require zero Confident proposals
   from the 12%-side-proximity screen that remain internal-region selections on
   inspection. Preserve the two already-safe slab siblings
   `20240415_225356.jpg` and `20240415_225455.jpg` as non-confident unless an
   accurate outer-boundary result is explicitly demonstrated.
3. Produce a before/after table with all three status counts and the exact list
   of images whose status or corners changed. Assess every confidence-count
   change: decreases are not automatically safe, increases are not
   automatically improvements, and no aggregate count can override a known or
   screened internal-region proposal. Keep this evidence out of Git; after
   authorization it belongs in the coder/reviewer PR audit-trail comment.
4. Run the focused route and browser contracts to prove camelCase status/corner
   serialization, authentication/media-revision authority, stale-session and
   draft-revision rejection, non-mutating `Unavailable`/`NotNeeded` behavior,
   and manual fallback. `controller/src/routes.rs` and
   `controller/static-admin/admin.js` must remain unchanged.
5. Run the full adjacent Rust/static-admin guardrails. Delete the temporary
   evaluator directory and verify that Git contains neither private media nor
   evaluator/output artifacts before any commit is considered.

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

**Done:** The exact private diagnostic is accurately `Confident`; the corpus
has zero known/screened internal-region confident proposals; before/after counts
and every changed result have an explicit assessment; all consumer/authority
and manual-fallback gates pass; and no private/evaluator artifact is present in
the repository.

## Acceptance criteria

- The paired sanitized fixture contract is green: coherent sparse outer
  perimeter => `Unavailable`; noise-connected accurate dark card => accurate
  `Confident`.
- The exact supplied dark-card diagnostic returns accurate `Confident` corners
  within the stated coordinate tolerance.
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
  replaced, not supplemented by another scalar/boolean point gate.
- Only `controller/src/image_adjustments.rs` and
  `controller/tests/image_adjustments.rs` differ from the pre-execution source
  scope, and no private image, absolute private test path, evaluator, binary,
  per-image output, or generated derivative is committed.

## Rollback and stop conditions

Stop and return to `reassessment_required` without another point-fix round if
any condition below occurs:

- the sanitized noisy accurate fixture is not RED on `85db903` for the same
  material-extrema/no-coherent-perimeter mechanism;
- the sparse, filled, or noisy-border enclosure becomes `Confident`;
- the exact diagnostic is not accurate `Confident`;
- any known or screened internal-region corpus proposal is `Confident`;
- a support constant needs image-specific tuning or another population/area
  boolean is proposed around the coherent model;
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
| Round-3 revised requirement: coherent support, not scalar tweak | The four-side support model replaces the `||`. | 2 |
| Paired sanitized fixtures | Sparse perimeter negative and noise-connected accurate card positive. | 1-2 |
| Exact diagnostic and 116-image corpus | Accurate diagnostic, zero known/screened internal-region confidence, count-change assessment. | 1, 3 |
| Deferred AI/advanced enhancement | Excluded; no OCR/AI, content-aware correction, batch correction, or broad admin redesign. | Scope fence |

No source item is intentionally deferred within this surgical convergence
scope. Phase-wide OPS/CDN/media-publication work is already owned elsewhere and
is not a gap in this PR #276 detector plan.

## Pre-mortem guardrails

| Likely failure | Early detector | Mitigation already in plan |
| --- | --- | --- |
| The sanitized positive does not reproduce the real regression. | It is green on unmodified `85db903` or fails for a different reason. | Task 1 halts before production changes and requires RED plus morphology evidence. |
| The support model merely trades false abstention for renewed false confidence. | Exact diagnostic passes but sparse/corpus screen fails, or counts improve without geometry safety. | Paired tests plus exact diagnostic plus full 116-image before/after screen are jointly binding. |
| Consumer safety regresses despite a detector-only edit. | Route/browser/authority suites fail or verification-only files change. | Task 3 runs focused consumer tests and requires zero diff in those files. |

