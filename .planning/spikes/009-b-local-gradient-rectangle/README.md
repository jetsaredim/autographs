---
spike: 009b
idea: outer-boundary-detection
name: local-gradient-rectangle
type: comparison
validates: "Given the same controls and corpus, when Canny/Hough rectangular-boundary evidence is scored, then it outperforms or falsifies the contour approach under the same acceptance gates."
verdict: VALIDATED
related: [008, 009a]
tags: [rust, imageproc, canny, hough, detector]
---

# Spike 009b: Local-Gradient Rectangle

## What This Validates

Given the same controls and corpus, when Canny/Hough rectangular-boundary
evidence is scored, then it should outperform or falsify the contour approach
under the same acceptance gates.

## Research

`imageproc 0.27` provides Canny edge detection and a standard polar Hough line
transform over binary edge pixels. Unlike the border-median component masks,
these operations preserve local transitions even when the physical item reaches
the image border. Hough returns infinite lines rather than segments, so this
spike independently measures corridor support, unsupported gaps, and signed
inside/outside contrast on the candidate side segments.

| Approach | Pros | Cons | Status |
|---|---|---|---|
| Projection peaks | Simple and fast | Assumes axis alignment | Rejected for skewed inputs |
| Raw Hough extrema | Finds skewed long lines | Texture and text create false rectangles | Rejected alone |
| Hough plus localized side evidence | Skew-tolerant and verifies the proposed segments | More computation and explicit geometric contract | Chosen |

The candidate acts as a conservative veto: a materially larger, independently
supported rectangle makes the existing primary `Unavailable`; it does not
replace the primary corners.

## How to Run

First run Spike 008, then:

```bash
cargo run --manifest-path .planning/spikes/009-b-local-gradient-rectangle/Cargo.toml -- \
  /tmp/autographs-detector-diagnostics/report.json \
  /tmp/autographs-gradient-rectangle
```

## What to Expect

The command writes redacted `evaluation.json` plus private local overlays. The
primary quadrilateral is magenta; a materially larger gradient candidate is
cyan. The exact positive must remain `keep-primary`, while the slab must become
`unavailable-larger-gradient-boundary` before corpus evaluation is allowed.

## Investigation Trail

- Chose axis-grouped Hough lines rather than axis-aligned projections so mild
  rotation and perspective do not disappear by construction.
- Defined structural thresholds before running the exact controls: minimum line
  support/gap, material area growth, side support/gap, local contrast, and sign
  coherence.
- The first exact-control run passed without changing those constants. The
  positive retained its accurate `0.764`-area primary with no larger candidate.
  The slab produced a `0.731`-area candidate outside its `0.343` internal-card
  primary; all four candidate sides were valid, with `0.894` to `1.000` edge
  support and coherent local contrast.
- Ran the unchanged candidate contract across all 116 held-out originals. It
  retained 79 primaries and safely abstained on 37 materially larger gradient
  boundaries. The three graded-slab siblings all abstained.
- Visually screened contact sheets for all 116 outcomes. None of the 79 retained
  primaries selected an internal region. A repeat of the prior 12%-side-distance
  screen flagged only rotated landscape card `20251231_011140`; full-resolution
  inspection confirmed its primary follows the outer card.
- The 79 retained primary areas form a narrow structural band from `0.741` to
  `0.818`; none violates the controller's existing area gate. The 37 abstentions
  remain manual-fallback cases, including some accurate primaries where a sleeve
  or surrounding boundary is independently visible.
- An optimized run over the two prepared exact-control thumbnails took `0.21s`
  total with about `8.2 MiB` maximum resident memory on the development host.

## Results

**VALIDATED.** Localized Canny/Hough rectangle evidence is a feasible conservative
veto for the existing primary candidate. It passes both exact controls, removes
the known slab/internal-card failure, and yields 79 visually accurate retained
primaries with zero known or screened internal-region proposals across the full
116-image corpus. No structural constant changed after viewing either exact
control or the corpus.

This spike does not authorize copying the prototype directly into production.
The implementation plan must extract the candidate contract into focused Rust
helpers, add sanitized positive/noise/slab fixtures, preserve the exact private
and corpus gates, and independently review the new assumptions. The gradient
candidate should remain a veto; it must not silently replace the primary corners.
