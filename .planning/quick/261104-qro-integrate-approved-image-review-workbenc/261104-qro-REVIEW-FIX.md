---
phase: quick-261104-qro
review_rounds: 2
status: clean
open_critical: 0
open_warning: 0
---

# PR #317 review and fix lineage

| Finding | Round 1 | Invariant and consumer inventory | Fix | Round 2 |
| --- | --- | --- | --- | --- |
| WR-01 `review_json_semantic_equality` | Independent Warning: raw saved `null` or full-frame perspective appeared changed in the JSON dialog while `reviewDirty` was false. [Inline finding](https://github.com/jetsaredim/autographs/pull/317#discussion_r4475132643). | Highlights must agree with `reviewAdjustmentsEqual`. Inspected `openImageReview`, `canonicalReviewAdjustment`, `syncReviewDirtyState`, `renderReviewJsonComparison`, draft mutations, save/reset, egress/publish gates, and behavior tests. | `d6837e5` derives changed paths from canonical saved/draft values while retaining raw saved display. Added untouched `null`, full-frame perspective, and real-change tests. [Fix comment](https://github.com/jetsaredim/autographs/pull/317#issuecomment-6646260891). | Clean at pushed `d6837e5`; no actionable findings. [Review confirmation](https://github.com/jetsaredim/autographs/pull/317#issuecomment-6646281952). |

No third review or convergence reassessment was needed.
