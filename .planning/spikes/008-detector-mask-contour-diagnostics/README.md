---
spike: 008
idea: outer-boundary-detection
name: detector-mask-contour-diagnostics
type: standard
validates: "Given the exact positive and slab controls, when threshold masks, anchor components, contours, extrema chords, and edge evidence are rendered, then the failed coherence decision is observable without committing private media."
verdict: VALIDATED
related: []
tags: [rust, imageproc, diagnostics, contours, privacy]
---

# Spike 008: Detector Mask and Contour Diagnostics

## What This Validates

Given the exact positive and slab controls, when the detector's threshold masks,
anchor-connected components, actual contours, extrema chords, Canny edges, and
Hough lines are rendered, then the failed coherence decision is observable
without committing private media.

## Research

The existing `imageproc 0.27` dependency already exposes Suzuki/Abe-style binary
contour extraction, Canny edge detection, connected-component labelling, and a
standard Hough line transform. This spike uses contours, Canny, and Hough
directly; it does not introduce OpenCV or a native runtime dependency.

| Approach | Tool | Pros | Cons | Status |
|---|---|---|---|---|
| Threshold-component diagnostics | Existing detector logic | Exact reproduction of current decisions | Absolute background distance misses some local edges | Chosen baseline |
| Actual component contours | `imageproc::contours` | Preserves real boundary shape instead of extrema chords | Still depends on threshold segmentation | Included |
| Local edges and infinite lines | `imageproc::edges` and `imageproc::hough` | Independent of border-median component fill | Texture can generate many competing lines | Included |

## How to Run

```bash
cargo run --manifest-path .planning/spikes/008-detector-mask-contour-diagnostics/Cargo.toml -- \
  --output /tmp/autographs-detector-diagnostics \
  positive=/path/to/positive.webp \
  slab=/path/to/slab.jpg
```

Then open `/tmp/autographs-detector-diagnostics/index.html` locally. The command
prints that path when it succeeds.

Use `--compact` for a corpus handoff to a comparison spike. Compact mode keeps
only source thumbnails and redacted component metrics, skipping diagnostic
overlays and the three redundant Canny/Hough passes.

## What to Expect

Each labelled input receives a source thumbnail, primary mask and overlay,
weaker-threshold masks and overlays, three Canny edge maps, three Hough overlays,
and redacted component metrics in `report.json`. Cyan is the actual component
contour; magenta is the four-extrema chord used by the failed model.

## Observability

`report.json` records redacted per-input metrics and timestamped start/completion
events. Input filesystem paths are never serialized. All rendered private pixels
remain in the caller-selected output directory, which must stay outside Git.

## Investigation Trail

- Established the current `imageproc` feature surface before building, avoiding
  a new OpenCV/native dependency.
- Reproduced the controller's border median, adaptive threshold, foreground
  distance, anchor-connected probes, and extrema selection in an isolated spike.
- Ran the exact positive and slab controls into `/tmp`. The positive primary was
  dense (`0.927` bounding-box density) with a `0.764` quadrilateral area and
  changed only slightly through thresholds 49 and 33. It did not flood the full
  thumbnail until threshold 16.
- The slab primary was the known internal card (`0.343` quadrilateral area). It
  became a dense internal rectangle through thresholds 175, 117, and 78, then
  jumped to a `0.963` almost-full-frame quadrilateral at threshold 32.
- At that flood point the component touched and occupied the image boundary, so
  `find_contours` produced no usable outer component contour. This explains why
  an extrema-chord coherence test cannot recover the physical slab edge from
  the existing absolute-background mask.
- Canny/Hough overlays still contained long outer vertical and horizontal slab
  evidence alongside many internal card/text lines. That preserves the
  local-gradient comparison while showing that line selection, not edge
  visibility, is its principal risk.

## Results

**VALIDATED.** The harness makes the failed decision observable and keeps all
private pixels and detailed outputs outside Git. It falsified a contour-only
extension of the current border-median component model: the real slab's weak
component floods the frame before it yields a meaningful outer contour. Actual
component contours remain useful diagnostics and may help at non-flooded
thresholds, but they cannot be the sole production boundary source.

The next comparison must therefore test contour-derived candidates as a bounded
baseline and local-gradient/Hough candidates as the independent alternative.
