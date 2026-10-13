---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
reviewed_assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
reviewed_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
status: passed
blockers: 0
warnings: 0
info: 0
---

# PR #276 Detector Convergence Plan Review

## VERIFICATION PASSED

**Scope:** Surgical detector-convergence work for PR #276  
**Artifacts reviewed:** Detector assumptions and revised implementation plan  
**Status:** Both artifacts are implementation-ready; no blocker, warning, or advisory remains.

## Goal-backward verdict

The assumptions and plan jointly satisfy the round-3 resume criteria in
`08-REVIEW.md`. They replace the failed global population/area scalar model
with one coherent boundary-support criterion, bind the negative and positive
morphologies that defeated the prior `&&` and `||` arrangements, require the
private positive control and complete held-out corpus evaluation, preserve all
consumer and mutation invariants, and fence the work to the two detector/test
files.

Coder work may resume after the convergence artifacts record this review as
their independent-review evidence. That evidence update must retain the
existing rule that a status-only edit is invalid.

## Required-property coverage

| Required property | Evidence in assumptions/plan | Result |
| --- | --- | --- |
| The failed `population_growth &&/|| area_growth` model is replaced rather than wrapped. | Assumptions reject both Boolean arrangements in R-03/R-06. Task 2 deletes the independent population veto and old `||`, keeps area growth only as a material-enclosure trigger, and makes coherent four-side support the deciding signal. | Pass |
| The signal distinguishes sparse enclosing perimeter support from noise-driven extrema. | The model measures the anchor-connected component bitmap along supercover-rasterized candidate sides, requires a two-pixel component corridor, an in-bounds exterior witness, at least 70% coverage, at most a 20% unsupported run, and all four sides. The morphology table explains that a connected sparse rectangle supplies distributed support while tendrils/speckles merely pulling extrema do not. | Pass |
| Paired sanitized negative and positive fixtures are binding. | Task 1 preserves the sparse enclosure as `Unavailable`, adds a generated non-private noisy dark-card analogue that must be accurate `Confident`, requires the positive to fail RED on `85db903`, and stops before production changes if the fixture does not reproduce the claimed mechanism. Task 2 makes both GREEN before the full suite. | Pass |
| The exact private diagnostic is accurate `Confident`. | Task 3 requires four finite corners near the recorded oracle, with at most 0.03 normalized error per coordinate. `Unavailable`, `NotNeeded`, empty corners, or a materially different quadrilateral explicitly fails acceptance. | Pass |
| The 116-image corpus has zero known or screened internal-region confident proposals. | Tasks 1 and 3 require a fresh baseline and rerun of all 116 sorted JPEGs, explicit treatment of the four known failures, a 12%-side-proximity geometry screen, inspection of flagged confidence, preservation/assessment of safe slab siblings, before/after counts, and image-by-image assessment of every changed result. Aggregate abstention counts cannot substitute for geometry safety. | Pass |
| Producers, consumers, mutation boundaries, reporting paths, and tests are inventoried. | The assumptions inventory covers border/adaptive/weak-threshold producers, primary selection, anchor trace/membership, stability, proposal classification, route reporting, browser/session authority, draft mutation, persistence/publication, detector regressions, route/browser/static contracts, and held-out inputs. Source inspection confirms these are the direct paths in the supplied Rust and JavaScript files. | Pass |
| Tasks are executable and ordered with exact files and red/green proof. | The plan orders RED fixture/baseline work, GREEN/REFACTOR implementation, then private/corpus/consumer VALIDATE work. Each task names files, actions, commands, expected outcomes, and done conditions. The hardest discriminator is proven before production code changes. | Pass |
| Failure and rollback behavior is explicit. | The stop list covers morphology mismatch, enclosure regression, private-control failure, corpus false confidence, per-image tuning, consumer/scope drift, verification failures, unexplained corpus changes, and private-artifact leakage. Rollback is limited to the two in-scope edits while preserving unrelated work and evidence. | Pass |
| Private evidence cannot enter commits. | The plan makes the WEBP and corpus read-only, keeps the evaluator and outputs under `/tmp`/outside Git, forbids absolute private paths in executable tests, deletes temporary evaluation material, checks the diff/status before commit consideration, and permits only generated sanitized fixtures in the repository. | Pass |
| Shared invariants and Phase 8 boundaries are preserved. | The assumptions carry forward all four review invariants. The plan freezes API shapes, authorization, terminal media revision, browser authority, unsaved-draft semantics, manual controls, private originals, save/publication boundaries, dependencies, and verification-only consumer files. It excludes OpenCV, AI/OCR, new services, taxonomy/media cues, batch correction, and broad admin redesign. | Pass |

## Constraint, failure, and reversibility assessment

The hardest constraint is not implementing raster support; it is proving that
the same rule rejects a coherent weak enclosure without losing the noisy but
accurate card. The plan addresses that constraint first with paired RED oracles
and then treats the exact private diagnostic and corpus geometry screen as
independent acceptance gates. Its pre-mortem covers the three material failure
modes: an unfaithful sanitized positive, renewed internal false confidence, and
consumer-authority regression.

The production decision is local and reversible: only
`controller/src/image_adjustments.rs` changes, and rollback restores the last
reviewed detector state. Tests change only to encode the missing paired
morphology. No schema, response contract, dependency, persistent data, or
public artifact format changes.

## Plan summary

| Task | Purpose | Modified files | Acceptance boundary |
| --- | --- | --- | --- |
| 1 | Bind paired morphology and measure the actual head | `controller/tests/image_adjustments.rs` | Positive RED for the demonstrated reason; sparse negative remains green; private/corpus baseline stays outside Git |
| 2 | Implement coherent perimeter support | `controller/src/image_adjustments.rs`, `controller/tests/image_adjustments.rs` | Paired tests and complete detector suite green; old independent scalar veto absent |
| 3 | Validate held-out safety and unchanged consumers | No additional files may change | Accurate private diagnostic; zero known/screened internal confidence across 116 images; route/UI/static/full Rust gates green; no private artifacts |

## Structured issues

```yaml
issues: []
```

## Resume recommendation

The convergence review is clean. Record this file as the independent review
evidence in both convergence artifacts, then change their state from
`reassessment_required` to `ready_to_resume` in the same evidence-bearing
update. Execution must continue to honor every RED/GREEN/VALIDATE gate and stop
condition in the revised plan.
