---
spike: 009a
idea: outer-boundary-detection
name: actual-contour-boundary
type: comparison
validates: "Given the diagnostic masks and actual component contours, when contour-derived quadrilateral candidates are scored, then the positive outer card is retained while the slab's internal card is rejected without per-image tuning."
verdict: INVALIDATED
related: [008]
tags: [rust, imageproc, contours, detector]
---

# Spike 009a: Actual-Contour Boundary

## What This Validates

Given the diagnostic masks and actual component contours, when contour-derived
quadrilateral candidates are scored, then the positive outer card should be
retained while the slab's internal card is rejected without per-image tuning.

## Research

Spike 008 established that `imageproc::contours` preserves the true outline of
a binary component, but cannot recover a physical edge absent from that binary
segmentation. This comparison deliberately tests that limitation before adding
more contour approximation machinery.

| Policy | Benefit | Failure mode |
|---|---|---|
| Keep the primary when no larger contour exists | Preserves the exact positive | Keeps the slab's internal card |
| Abstain whenever a weaker component floods the frame | Rejects the slab internal card | Also rejects the exact positive |
| Key abstention to the flood threshold | Can separate the two exact controls | Tunes a scalar to these images and is forbidden |

**Chosen approach:** evaluate the first two structural policies. A threshold-keyed
policy is documented but intentionally not implemented as a candidate.

## How to Run

First run Spike 008 for the exact controls, then:

```bash
cargo run --manifest-path .planning/spikes/009-a-actual-contour-boundary/Cargo.toml -- \
  /tmp/autographs-detector-diagnostics/report.json
```

## What to Expect

The evaluator reports each input's first full-frame flood threshold, whether a
usable materially larger contour exists, and the outcomes of ignoring versus
abstaining on a flood. It reads only Spike 008's redacted report.

## Investigation Trail

- Reused the exact diagnostic evidence rather than copying private images or
  duplicating threshold logic.
- Defined structural accept/abstain policies before inspecting their outcome.
- The positive retained an accurate primary and exposed a materially larger
  non-flooded contour before its eventual full-frame flood at threshold 8.
- The slab exposed no usable materially larger contour before its full-frame
  flood at threshold 32. Its only contour-derived proposal remained the known
  internal card.
- Ignoring floods therefore returns `confident-primary` for both controls and is
  unsafe. Abstaining on every flood returns `Unavailable` for both and violates
  the positive control. Keying behavior to the observed threshold would merely
  replace the failed heuristic with per-image scalar tuning.

## Results

**INVALIDATED.** The contour-only candidate cannot satisfy the exact controls.
The slab's physical outer edge never appears as a usable contour in the current
border-median segmentation, so no contour approximation or corner selector can
recover it. A production design may still use component contours as supporting
evidence, but it needs an independent boundary source such as local gradients.

Because the exact negative gate fails before corpus evaluation, this candidate
is not eligible for the 116-image recommendation screen.
