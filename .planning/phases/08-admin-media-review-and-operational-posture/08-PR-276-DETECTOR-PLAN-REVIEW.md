---
phase: 08-admin-media-review-and-operational-posture
pr: 276
reviewed_at: 2026-10-13T10:05:41Z
reviewed_head: 85db903d4dd0ef70887816c9849101fac2827c7a
reassessment_iteration: 2
reviewed_assumptions: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-ASSUMPTIONS.md
reviewed_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-276-DETECTOR-REVISED-PLAN.md
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
prior_review_result: superseded_by_stop_condition
status: passed
blockers: 0
warnings: 0
info: 0
---

# PR #276 Detector Convergence Plan Review — Second Reassessment

## VERIFICATION PASSED

**Verdict:** PASS — the second reassessment and revised plan are
implementation-ready.

**Issues:** 0 Blocker, 0 Warning, 0 Info.
**Authorization boundary:** This review clears planning only. Execution remains
bound by every RED/characterization, GREEN, private-control, corpus, consumer,
privacy, rollback, and stop gate in the revised plan.

## Goal-backward conclusion

The first reviewed coherence plan correctly replaced the global scalar
`&&`/`||` model, but execution proved that its strict
`Qt`-contains-`Q0` prerequisite could bypass coherence analysis for a real
larger slab. The second reassessment identifies the failed assumption precisely:
anchor-connected pixel membership is monotone as the threshold weakens, while
the quadrilateral joining four extrema is only a lossy summary and is not
polygon-set-monotone.

The revised contract therefore uses exact `M0 ⊆ Mt` membership as the sole
cross-threshold identity prerequisite, retains material area growth only as a
trigger, and lets the already reviewed four-side localized-support test decide
whether the weak component exposes a coherent replacement boundary. This is a
coherent correction to the model, not another image-specific scalar or Boolean
patch.

## Required-property verification

| Required property | Evidence | Result |
| --- | --- | --- |
| Preserve finding lineage and react correctly to the prior stop condition. | Both artifacts record the rolled-back uncommitted attempt as `incomplete_fix` / `test_weakness`, preserve the earlier review as superseded evidence rather than authorization, identify `20240415_225421.jpg` as the stopping case, and require a fresh review before resumption. | Pass |
| Use a valid cross-threshold identity relationship. | A lower threshold cannot remove a `C0` foreground pixel; tracing the retained anchor at the weaker threshold must therefore include every exact primary member in `Mt`. The plan explicitly retains primary member indices because the scan-wide primary `visited` bitmap includes other components and is not valid `M0`. Any membership-retention violation fails closed. | Pass |
| Remove the invalid extrema-polygon containment assumption. | R-07 rejects `Qt` containing every `Q0` corner as a prerequisite. Task 2 bans that predicate and any centroid/corner substitute, while a causal mutation check temporarily restores containment and requires only the non-nested slab to regress. | Pass |
| Preserve the discriminating spatial signal. | The plan retains supercover side sampling, the two-pixel `Mt` corridor, in-bounds outward non-`Mt` witnesses at offsets 3–5, per-side coverage ≥70%, maximum unsupported run ≤20%, and all-four-sides coherence. These thresholds already passed the sparse negative, noisy-card positive, and exact private positive; the revision does not retune them. | Pass |
| Bind every observed morphology without private pixels. | Task 1 restores the noisy accurate positive, preserves sparse/filled/noisy-border negatives, and adds a deterministic asymmetric slab whose `M0` is retained, area grows materially, four sides are supported, and extrema polygons are non-nested. It requires the exact containment-gated intermediate to fail `Confident` before GREEN work. | Pass |
| Make the known failed private case a hard causal gate. | Task 3 runs `20240415_225421.jpg` before the remaining corpus, requires `Unavailable`, explicitly forbids the recorded internal quadrilateral, and stops immediately on any `Confident` result even if all synthetic and positive controls pass. | Pass |
| Preserve useful positive assistance, not merely maximize abstention. | The sanitized noisy card and exact private WEBP must both be accurate `Confident`; the exact WEBP has a four-corner oracle and 0.03 per-coordinate tolerance. Existing high-contrast, reliable-card, and near-identity contracts remain binding. | Pass |
| Screen the complete held-out corpus for false confidence. | The plan measures a fresh head baseline, evaluates all 116 sorted images, requires zero known or screened internal-region confident proposals, preserves named safe cases unless an accurate outer boundary is demonstrated, lists every status/corner change, and requires image-by-image assessment rather than aggregate-count scoring. | Pass |
| Keep producer, consumer, mutation, and reporting boundaries complete. | The reassessment inventories background/threshold generation, primary selection, exact component tracing, stability, proposal classification, route reporting and terminal revision authority, browser session/draft/assist authority, draft mutation, explicit persistence, publication, detector/route/browser/static tests, and held-out evidence. | Pass |
| Be executable, ordered, and reversible. | Tasks proceed through head characterization and two-stage RED, component-membership GREEN/refactor, then named-private/full-corpus/consumer validation. Exact files, commands, expected outcomes, done criteria, causal mutation, stop conditions, and rollback to the reviewed head are specified. | Pass |
| Preserve privacy and Phase 8 scope. | Only `image_adjustments.rs` and its integration test may change. Private inputs are read-only; evaluator code/output stays under `/tmp` and outside Git; absolute private paths are forbidden in executable tests. No dependency, API, route, UI, storage, public artifact, AI/OCR, OpenCV, service, or broad Phase 8 change is authorized. | Pass |

## Feasibility and ambiguity assessment

The implementation path matches the current source structure. Extending
`ContrastComponent` with exact member indices is sufficient to retain `M0` and
construct `Mt`; no new dependency or architectural layer is required. The
existing descending probe schedule and retained anchor make the subset check
deterministic. The plan also avoids a subtle implementation trap by stating
that `largest_contrast_component`'s scan-wide `visited` vector is not primary
membership.

The geometry rules are concrete enough to implement and test: rasterization,
corridor metric, exterior-witness offsets, coverage, gap limit, and four-side
requirement are all fixed. The new fixture must prove its structural premises
with temporary trace evidence before it is accepted, and the intermediate
containment model plus mutation check distinguishes a causal regression from a
test that passes only because the broad head `||` abstains.

The remaining false-positive and false-negative risks are bounded rather than
assumed away. The exact positive prevents indiscriminate abstention; the new
slab and existing enclosure cases prevent renewed internal confidence; the
named private failure is checked first; and the complete corpus geometry screen
is a mandatory stop gate. A new morphology that defeats the 0.05 trigger or
support model explicitly requires another reassessment, not threshold tuning.

## Convergence-guard decision

This revision satisfies the third-review reassessment contract:

- finding lineage and the failed assumption are durable;
- shared invariants and the complete consumer inventory remain explicit;
- the failure matrix includes the newly observed non-nested slab class;
- assumptions and implementation steps changed coherently before coder work;
- this fresh review records both reviewed artifact paths; and
- both artifacts now reference this review before moving to
  `ready_to_resume`.

## Structured issues

```yaml
issues: []
```

## Resume authorization

PASS. The assumptions and revised plan are `ready_to_resume` with this file as
their independent-review evidence. If any stated stop condition fires,
execution must roll back the two in-scope edits and return to
`reassessment_required`; it must not begin another point-fix round.
